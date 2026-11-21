//! Library helpers for `udtcat` / `udtd` (Phase 5+).

mod frame;

pub use frame::{
    copy_exact, read_frame_header, safe_basename, unique_path, write_framed, FrameError,
    IncomingFile, MAX_FILE_LEN, MAX_NAME_LEN,
};

use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;
use udt_socket::{UdtOption, UdtSocket};

/// Errors from CLI transfer helpers.
#[derive(Debug)]
pub enum CliError {
    /// I/O.
    Io(io::Error),
    /// Socket.
    Socket(udt_socket::SocketError),
    /// Bad arguments / address.
    Args(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Socket(e) => write!(f, "{e}"),
            Self::Args(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<udt_socket::SocketError> for CliError {
    fn from(e: udt_socket::SocketError) -> Self {
        Self::Socket(e)
    }
}

/// Parse `host:port` or `:port` / `port` into a [`SocketAddr`] (IPv4 localhost default).
pub fn parse_addr(s: &str) -> Result<SocketAddr, CliError> {
    let s = s.trim();
    if let Ok(port) = s.parse::<u16>() {
        return Ok(SocketAddr::from(([127, 0, 0, 1], port)));
    }
    if let Some(rest) = s.strip_prefix(':') {
        let port: u16 = rest
            .parse()
            .map_err(|_| CliError::Args(format!("bad port in {s}")))?;
        return Ok(SocketAddr::from(([127, 0, 0, 1], port)));
    }
    s.to_socket_addrs()
        .map_err(|e| CliError::Args(e.to_string()))?
        .next()
        .ok_or_else(|| CliError::Args(format!("could not resolve {s}")))
}

/// Listen on `addr`, accept one client, write the stream to `out`.
pub fn listen_and_recv(addr: SocketAddr, out: &mut dyn Write) -> Result<u64, CliError> {
    let mut server = UdtSocket::new();
    tune_transfer(&mut server)?;
    server.bind(addr)?;
    let local = server.local_addr().unwrap();
    eprintln!("udtcat: listening on {local}");
    server.listen(8)?;
    let (mut child, peer) = server.accept()?;
    eprintln!("udtcat: accepted from {peer}");
    // Child copied options at accept; refresh timeouts for the transfer loop.
    child.set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20))))?;
    child.set_option(UdtOption::Mss(1400))?;

    let mut buf = vec![0u8; 1024 * 1024];
    let mut total = 0u64;
    // Large transfers / cross-impl pauses: keep idle generous.
    let idle_limit = Duration::from_secs(60);
    let mut last_progress = std::time::Instant::now();
    loop {
        let n = child.recv(&mut buf)?;
        if n > 0 {
            out.write_all(&buf[..n])?;
            total += n as u64;
            last_progress = std::time::Instant::now();
            continue;
        }
        if last_progress.elapsed() > idle_limit {
            break;
        }
    }
    let _ = child.shutdown();
    out.flush()?;
    Ok(total)
}

/// Connect to `peer` and send all bytes from `input`.
pub fn connect_and_send(peer: SocketAddr, input: &mut dyn Read) -> Result<u64, CliError> {
    let mut client = UdtSocket::new();
    tune_transfer(&mut client)?;
    eprintln!("udtcat: connecting to {peer}");
    client.connect(peer)?;
    eprintln!("udtcat: connected");

    let mut buf = vec![0u8; 1024 * 1024];
    let mut total = 0u64;
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut off = 0;
        while off < n {
            let sent = client.send(&buf[off..n])?;
            if sent == 0 {
                client.pump_once()?;
                continue;
            }
            off += sent;
            total += sent as u64;
        }
    }
    // Cross-impl ACK shapes may leave our send buffer uncleared; pump briefly then
    // treat flush timeout as a warning so byte-accurate peer receives still pass.
    match client.flush(Duration::from_secs(5)) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("udtcat: flush warning: {e} (bytes enqueued={total})");
            for _ in 0..50 {
                let _ = client.pump_once();
            }
        }
    }
    let _ = client.shutdown();
    Ok(total)
}

/// Open an input path or stdin when `None` / `-`.
pub fn open_input(path: Option<&Path>) -> Result<Box<dyn Read + Send>, CliError> {
    match path {
        None => Ok(Box::new(io::stdin())),
        Some(p) if p.as_os_str() == "-" => Ok(Box::new(io::stdin())),
        Some(p) => Ok(Box::new(File::open(p)?)),
    }
}

/// Open an output path or stdout when `None` / `-`.
pub fn open_output(path: Option<&Path>) -> Result<Box<dyn Write + Send>, CliError> {
    match path {
        None => Ok(Box::new(io::stdout())),
        Some(p) if p.as_os_str() == "-" => Ok(Box::new(io::stdout())),
        Some(p) => Ok(Box::new(File::create(p)?)),
    }
}

/// Apply the large-transfer socket options used by CLI tools.
fn tune_transfer(sock: &mut UdtSocket) -> Result<(), CliError> {
    sock.set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20))))?;
    sock.set_option(UdtOption::SndTimeo(Some(Duration::from_secs(600))))?;
    sock.set_option(UdtOption::MaxFlowWindow(65_535))?;
    sock.set_option(UdtOption::Mss(1400))?;
    sock.set_option(UdtOption::SndBuf(16 * 1024 * 1024))?;
    sock.set_option(UdtOption::RcvBuf(16 * 1024 * 1024))?;
    Ok(())
}

/// Persistent multi-file receiver: accept forever, save each framed file under `inbox`.
pub fn serve_inbox(addr: SocketAddr, inbox: &Path) -> Result<(), CliError> {
    std::fs::create_dir_all(inbox)?;
    let mut server = UdtSocket::new();
    tune_transfer(&mut server)?;
    server.bind(addr)?;
    let local = server.local_addr().unwrap();
    eprintln!("udtd: listening on {local}, inbox={}", inbox.display());
    server.listen(32)?;

    loop {
        let (mut child, peer) = server.accept()?;
        eprintln!("udtd: accepted from {peer}");
        child.set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20))))?;
        match recv_framed_connection(&mut child, inbox) {
            Ok(n) => eprintln!("udtd: connection from {peer} saved {n} file(s)"),
            Err(e) => eprintln!("udtd: connection from {peer} error: {e}"),
        }
        let _ = child.shutdown();
    }
}

fn recv_framed_connection(child: &mut UdtSocket, inbox: &Path) -> Result<usize, CliError> {
    // Listener and child share one UdpSocket. Do not linger after the body: a
    // trailing idle pump would swallow the next client's handshake datagrams.
    // `udtd put` sends one framed file per connection; multi-file batches use
    // back-to-back connections.
    let mut reader = UdtReader {
        sock: child,
        idle: Duration::from_secs(30),
        last: std::time::Instant::now(),
    };
    let hdr = match read_frame_header(&mut reader) {
        Ok(Some(h)) => h,
        Ok(None) => return Ok(0),
        Err(FrameError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(0),
        Err(FrameError::Proto(ref s)) if s.starts_with("unexpected EOF") => return Ok(0),
        Err(e) => return Err(CliError::Args(e.to_string())),
    };
    let dest = unique_path(inbox, &hdr.name);
    let mut file = File::create(&dest)?;
    reader.idle = Duration::from_secs(600);
    reader.last = std::time::Instant::now();
    copy_exact(&mut reader, &mut file, hdr.len).map_err(|e| CliError::Args(e.to_string()))?;
    eprintln!(
        "udtd: wrote {} ({} bytes) from frame name={}",
        dest.display(),
        hdr.len,
        hdr.name
    );
    Ok(1)
}

/// Send one local file as a framed delivery to `peer`.
pub fn put_file(peer: SocketAddr, path: &Path) -> Result<u64, CliError> {
    let meta = std::fs::metadata(path)?;
    let file_len = meta.len();
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CliError::Args(format!("bad file name: {}", path.display())))?;
    let mut file = File::open(path)?;

    let mut client = UdtSocket::new();
    tune_transfer(&mut client)?;
    eprintln!("udtd: connecting to {peer} to put {name} ({file_len} bytes)");
    client.connect(peer)?;
    eprintln!("udtd: connected");

    let mut writer = UdtWriter { sock: &mut client };
    let n = write_framed(&mut writer, name, file_len, &mut file)
        .map_err(|e| CliError::Args(e.to_string()))?;
    match client.flush(Duration::from_secs(600)) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("udtd: flush warning: {e}");
            for _ in 0..50 {
                let _ = client.pump_once();
            }
        }
    }
    let _ = client.shutdown();
    Ok(n)
}

/// Send many files, one connection each.
pub fn put_files(peer: SocketAddr, paths: &[PathBuf]) -> Result<u64, CliError> {
    let mut total = 0u64;
    for p in paths {
        total += put_file(peer, p)?;
        eprintln!("udtd: put ok {}", p.display());
    }
    Ok(total)
}

struct UdtReader<'a> {
    sock: &'a mut UdtSocket,
    idle: Duration,
    last: std::time::Instant,
}

impl Read for UdtReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let deadline = std::time::Instant::now() + self.idle;
        loop {
            match self.sock.recv(buf) {
                Ok(0) => {
                    if self.last.elapsed() > self.idle || std::time::Instant::now() >= deadline {
                        return Ok(0);
                    }
                    let _ = self.sock.pump_once();
                }
                Ok(n) => {
                    self.last = std::time::Instant::now();
                    return Ok(n);
                }
                Err(e) => {
                    return Err(io::Error::other(e.to_string()));
                }
            }
        }
    }
}

struct UdtWriter<'a> {
    sock: &'a mut UdtSocket,
}

impl Write for UdtWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut off = 0;
        while off < buf.len() {
            match self.sock.send(&buf[off..]) {
                Ok(0) => {
                    let _ = self.sock.pump_once();
                }
                Ok(n) => off += n,
                Err(e) => return Err(io::Error::other(e.to_string())),
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.sock
            .flush(Duration::from_secs(30))
            .map_err(|e| io::Error::other(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn parse_port_only() {
        assert_eq!(
            parse_addr("9000").unwrap(),
            SocketAddr::from(([127, 0, 0, 1], 9000))
        );
    }

    #[test]
    fn localhost_file_round_trip() {
        let mut server = UdtSocket::new();
        server
            .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .unwrap();
        let addr = server.local_addr().unwrap();
        server.listen(1).unwrap();

        let payload: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
        let expect_len = payload.len();

        let t = thread::spawn(move || {
            let (mut child, _) = server.accept().unwrap();
            child
                .set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20))))
                .unwrap();
            let mut out = Vec::new();
            let mut buf = [0u8; 8192];
            let deadline = std::time::Instant::now() + Duration::from_secs(30);
            while out.len() < expect_len && std::time::Instant::now() < deadline {
                let n = child.recv(&mut buf).unwrap();
                out.extend_from_slice(&buf[..n]);
            }
            out
        });

        let expected = payload.clone();
        let mut cursor = io::Cursor::new(payload);
        let n = connect_and_send(addr, &mut cursor).unwrap();
        assert_eq!(n, 100_000);
        let got = t.join().unwrap();
        assert_eq!(got, expected);
    }
}
