//! Socket facades over [`udt_core`].
//!
//! Phase 2: UDP three-way handshake. Phase 3: reliable stream send/recv with
//! ACK/NAK and retransmission. Async APIs are Tokio-first (feature `tokio-rt`).

#![deny(missing_docs)]

mod options;

pub use options::{SocketOptionState, UdtOption, UdtOptionKind};

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use udt_core::{
    client_cookie_confirm, client_request, is_client_request, is_cookie_challenge,
    is_cookie_confirm, is_server_response, peer_ip_from_ipv4, server_cookie_challenge,
    server_final_response, syn_cookie, Connection, ConnectionState, HandshakePacket, Outgoing,
    PacketError, ReliabilityEngine, HANDSHAKE_PACKET_LEN, SOCK_TYPE_DGRAM, SOCK_TYPE_STREAM,
};

/// Errors from socket operations.
#[derive(Debug, Error)]
pub enum SocketError {
    /// Underlying I/O failure.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// State machine rejected the operation.
    #[error(transparent)]
    State(#[from] udt_core::StateError),
    /// Packet codec failure.
    #[error(transparent)]
    Packet(#[from] PacketError),
    /// Handshake did not complete successfully.
    #[error("handshake failed: {0}")]
    Handshake(&'static str),
    /// Reliable transfer failure.
    #[error("transfer failed: {0}")]
    Transfer(&'static str),
    /// Feature not implemented yet.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

/// UDT socket type (stream vs datagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdtSockType {
    /// Reliable byte stream (default).
    Stream,
    /// Messaging / datagram mode (handshake only in Phase 3).
    Datagram,
}

impl UdtSockType {
    fn wire(self) -> u32 {
        match self {
            Self::Stream => SOCK_TYPE_STREAM,
            Self::Datagram => SOCK_TYPE_DGRAM,
        }
    }
}

/// Synchronous UDT socket.
#[derive(Debug)]
pub struct UdtSocket {
    udp: Option<Arc<UdpSocket>>,
    conn: Connection,
    local: Option<SocketAddr>,
    peer: Option<SocketAddr>,
    listening: bool,
    sock_type: UdtSockType,
    init_seq: u32,
    peer_init_seq: u32,
    read_timeout: Option<Duration>,
    rel: Option<ReliabilityEngine>,
    /// Probability in `[0.0, 1.0]` of dropping an *outbound data* packet (tests).
    loss_rate: f64,
    loss_rng: u64,
    /// Socket options (`setsockopt` / `getsockopt` analogues).
    opts: SocketOptionState,
}

impl UdtSocket {
    /// Create a new unbound stream socket in [`ConnectionState::Init`].
    ///
    /// # Examples
    ///
    /// ```
    /// use udt_core::ConnectionState;
    /// use udt_socket::UdtSocket;
    ///
    /// let socket = UdtSocket::new();
    /// assert_eq!(socket.state(), ConnectionState::Init);
    /// assert!(socket.local_addr().is_none());
    /// ```
    pub fn new() -> Self {
        Self::new_with_type(UdtSockType::Stream)
    }

    /// Create a socket with an explicit stream/datagram mode.
    pub fn new_with_type(sock_type: UdtSockType) -> Self {
        let sock_id = next_sock_id();
        let opts = SocketOptionState::default();
        Self {
            udp: None,
            conn: Connection::new(sock_id, 0),
            local: None,
            peer: None,
            listening: false,
            sock_type,
            init_seq: sock_id.wrapping_mul(17).wrapping_add(1),
            peer_init_seq: 0,
            read_timeout: opts.rcv_timeo,
            rel: None,
            loss_rate: 0.0,
            loss_rng: sock_id as u64 ^ 0xC0FFEE,
            opts,
        }
    }

    /// Set a UDT socket option (`setsockopt` analogue).
    pub fn set_option(&mut self, opt: UdtOption) -> Result<(), SocketError> {
        if self.opts.rendezvous && matches!(opt, UdtOption::Rendezvous(false)) {
            // allow clearing
        }
        if self.conn.state() != ConnectionState::Init
            && matches!(opt, UdtOption::Rendezvous(_))
        {
            return Err(SocketError::Transfer(
                "rendezvous must be set before connect/listen",
            ));
        }
        self.opts.set(opt.clone());
        match opt {
            UdtOption::RcvTimeo(t) => {
                self.read_timeout = t;
                if let Some(udp) = &self.udp {
                    let _ = udp.set_read_timeout(t);
                }
            }
            UdtOption::SndBuf(n) => {
                if let Some(udp) = &self.udp {
                    apply_snd_buf(udp, n);
                }
            }
            UdtOption::RcvBuf(n) => {
                if let Some(udp) = &self.udp {
                    apply_rcv_buf(udp, n);
                }
            }
            UdtOption::Mss(m) => {
                if let Some(rel) = &mut self.rel {
                    rel.set_mss(m);
                }
            }
            UdtOption::MaxFlowWindow(w) => {
                if let Some(rel) = &mut self.rel {
                    rel.set_max_cwnd(w);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get a UDT socket option (`getsockopt` analogue).
    pub fn get_option(&self, kind: UdtOptionKind) -> UdtOption {
        self.opts.get(kind)
    }

    /// Borrow the full option state.
    pub fn options(&self) -> &SocketOptionState {
        &self.opts
    }

    /// Current connection state.
    pub fn state(&self) -> ConnectionState {
        self.conn.state()
    }

    /// Local UDP address if bound.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local
            .or_else(|| self.udp.as_ref().and_then(|u| u.local_addr().ok()))
    }

    /// Peer address after connect/accept.
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer
    }

    /// Local UDT socket ID.
    pub fn sock_id(&self) -> u32 {
        self.conn.sock_id()
    }

    /// Peer UDT socket ID (`0` until handshake completes).
    pub fn peer_sock_id(&self) -> u32 {
        self.conn.peer_sock_id()
    }

    /// Whether this socket is a listening server.
    pub fn is_listening(&self) -> bool {
        self.listening
    }

    /// Socket mode (stream / datagram).
    pub fn sock_type(&self) -> UdtSockType {
        self.sock_type
    }

    /// Configure induced outbound data loss for tests (`0.0`..=`1.0`).
    pub fn set_loss_rate(&mut self, rate: f64) {
        self.loss_rate = rate.clamp(0.0, 1.0);
    }

    /// Set the read timeout used by handshake and transfer pumps.
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.read_timeout = timeout;
        if let Some(udp) = &self.udp {
            let _ = udp.set_read_timeout(timeout);
        }
    }

    /// Bind to a local UDP address.
    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        if self.udp.is_some() {
            return Err(SocketError::Handshake("already bound"));
        }
        let udp = UdpSocket::bind(addr)?;
        udp.set_read_timeout(self.read_timeout)?;
        apply_snd_buf(&udp, self.opts.snd_buf);
        apply_rcv_buf(&udp, self.opts.rcv_buf);
        self.local = Some(udp.local_addr()?);
        self.udp = Some(Arc::new(udp));
        Ok(())
    }

    /// Mark this bound socket as a listening server.
    pub fn listen(&mut self, _backlog: i32) -> Result<(), SocketError> {
        if self.udp.is_none() {
            return Err(SocketError::Handshake("listen requires bind first"));
        }
        if self.conn.state() != ConnectionState::Init {
            return Err(SocketError::State(udt_core::StateError::Illegal {
                from: self.conn.state(),
                to: ConnectionState::Init,
            }));
        }
        self.listening = true;
        Ok(())
    }

    /// Connect to a peer (UDT4 cookie handshake) then enable reliable transfer.
    pub fn connect(&mut self, peer: SocketAddr) -> Result<(), SocketError> {
        if self.listening {
            return Err(SocketError::Handshake("listening socket cannot connect"));
        }
        if self.conn.state() != ConnectionState::Init {
            return Err(SocketError::Handshake("connect requires Init state"));
        }
        self.ensure_bound_for(peer)?;
        let udp = self.udp.as_ref().expect("bound").clone();
        self.conn.begin_handshake()?;

        let local_ip = ipv4_octets(self.local_addr().unwrap_or(peer));
        let req = client_request(
            self.conn.sock_id(),
            self.init_seq,
            self.sock_type.wire(),
            peer_ip_from_ipv4(local_ip),
            now_ts(),
        );
        send_hs(&udp, peer, &req)?;

        // Cookie challenge: conn_type=1 with cookie set (C++ UDT4 / PureRust).
        let (challenge, from) = recv_hs_from(&udp, |p| {
            is_cookie_challenge(p) && p.dest_sock_id == self.conn.sock_id()
        })?;
        if from.ip() != peer.ip() {
            return Err(SocketError::Handshake("challenge from unexpected address"));
        }

        let conf = client_cookie_confirm(
            &challenge,
            self.conn.sock_id(),
            self.init_seq,
            peer_ip_from_ipv4(local_ip),
            now_ts(),
        );
        send_hs(&udp, from, &conf)?;

        // Final: conn_type=-1 with peer sock id; ISN echoed (UDT4 security check).
        let (resp, from2) = recv_hs_from(&udp, |p| {
            is_server_response(p)
                && p.dest_sock_id == self.conn.sock_id()
                && p.syn_cookie == challenge.syn_cookie
                && p.init_seq == self.init_seq
        })?;
        if from2.ip() != peer.ip() {
            return Err(SocketError::Handshake("final from unexpected address"));
        }

        self.conn.set_peer_sock_id(resp.sock_id);
        // UDT4: both sides use the client's ISN for the session.
        self.peer_init_seq = resp.init_seq;
        self.peer = Some(from2);
        self.conn.mark_connected()?;
        self.install_reliability()?;
        Ok(())
    }

    /// Accept one incoming connection (UDT4 cookie handshake).
    ///
    /// The child shares the listener's UDP socket (`Arc`) so post-handshake data
    /// sent to the listen port is visible to the accepted connection.
    pub fn accept(&mut self) -> Result<(Self, SocketAddr), SocketError> {
        if !self.listening {
            return Err(SocketError::Handshake("accept requires listen first"));
        }
        match self.conn.state() {
            ConnectionState::Shutdown | ConnectionState::Closed => {
                return Err(SocketError::State(udt_core::StateError::Illegal {
                    from: self.conn.state(),
                    to: ConnectionState::Handshaking,
                }));
            }
            ConnectionState::Init
            | ConnectionState::Handshaking
            | ConnectionState::Connected => {}
        }
        let udp = self
            .udp
            .as_ref()
            .ok_or(SocketError::Handshake("not bound"))?
            .clone();

        let (req, peer) = recv_hs_from(&udp, is_client_request)?;
        if !is_client_request(&req) {
            return Err(SocketError::Handshake("expected client handshake request"));
        }

        let mut child = UdtSocket::new_with_type(self.sock_type);
        child.conn.begin_handshake()?;
        child.conn.set_peer_sock_id(req.sock_id);
        child.peer = Some(peer);
        // UDT4: session ISN is the client's ISN (echoed for security).
        child.peer_init_seq = req.init_seq;
        child.init_seq = req.init_seq;
        child.read_timeout = self.read_timeout;
        child.loss_rate = self.loss_rate;
        child.opts = self.opts.clone();

        let listen_ip = ipv4_octets(self.local_addr().unwrap_or(peer));
        let cookie = syn_cookie(req.sock_id, child.conn.sock_id());
        let challenge = server_cookie_challenge(&req, cookie, now_ts());
        send_hs(&udp, peer, &challenge)?;

        let (conf, conf_from) = recv_hs_from(&udp, |p| {
            is_cookie_confirm(p) && p.syn_cookie == cookie && p.sock_id == req.sock_id
        })?;
        if conf_from.ip() != peer.ip() {
            return Err(SocketError::Handshake("confirm from unexpected address"));
        }

        let fin = server_final_response(
            &conf,
            child.conn.sock_id(),
            self.opts.max_flow_window.max(8192),
            peer_ip_from_ipv4(listen_ip),
            now_ts(),
        );
        send_hs(&udp, peer, &fin)?;

        child.udp = Some(udp);
        child.local = self.local;
        child.conn.mark_connected()?;
        child.install_reliability()?;
        Ok((child, peer))
    }

    fn install_reliability(&mut self) -> Result<(), SocketError> {
        let mut rel = ReliabilityEngine::new(
            self.conn.sock_id(),
            self.conn.peer_sock_id(),
            self.init_seq,
            self.peer_init_seq,
        );
        rel.set_mss(self.opts.mss);
        rel.set_max_cwnd(self.opts.max_flow_window);
        self.rel = Some(rel);
        Ok(())
    }

    /// Begin orderly shutdown (Connected → Shutdown → Closed).
    pub fn shutdown(&mut self) -> Result<(), SocketError> {
        match self.conn.state() {
            ConnectionState::Connected => {
                self.conn.begin_shutdown()?;
                let _ = self.conn.close();
                Ok(())
            }
            ConnectionState::Shutdown => {
                self.conn.close()?;
                Ok(())
            }
            ConnectionState::Closed => Ok(()),
            other => Err(SocketError::State(udt_core::StateError::Illegal {
                from: other,
                to: ConnectionState::Shutdown,
            })),
        }
    }

    /// Send application bytes reliably (stream). Returns bytes accepted.
    ///
    /// May return fewer than `buf.len()` if the congestion window is full; the
    /// caller should `pump_once`/`flush` and retry the remainder. This method
    /// also pumps briefly to free window when possible.
    pub fn send(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        if self.conn.state() != ConnectionState::Connected {
            return Err(SocketError::Transfer("send requires Connected state"));
        }
        if self.sock_type != UdtSockType::Stream {
            return Err(SocketError::NotImplemented("datagram send — later phase"));
        }
        let mut offset = 0;
        let deadline = Instant::now() + Duration::from_secs(30);
        while offset < buf.len() {
            if Instant::now() >= deadline {
                break;
            }
            let now = Instant::now();
            let ts = now_ts();
            let (outgoing, consumed) = {
                let rel = self
                    .rel
                    .as_mut()
                    .ok_or(SocketError::Transfer("no reliability session"))?;
                rel.enqueue_app_data(&buf[offset..], now, ts)
            };
            if consumed == 0 && outgoing.is_empty() {
                // Window full or rate-paced — drive ACKs/rexmits then retry.
                self.pump_once()?;
                // Sleep until pacing or RTO deadline (cap 2ms).
                let now = Instant::now();
                let wait = self
                    .rel
                    .as_ref()
                    .map(|r| {
                        let send_wait = r.next_send_deadline().saturating_duration_since(now);
                        let timer_wait = r
                            .next_timer()
                            .map(|t| t.saturating_duration_since(now))
                            .unwrap_or(Duration::from_millis(2));
                        send_wait.min(timer_wait)
                    })
                    .unwrap_or(Duration::from_micros(200))
                    .min(Duration::from_millis(2));
                if !wait.is_zero() {
                    std::thread::sleep(wait);
                }
                continue;
            }
            self.dispatch(outgoing)?;
            offset += consumed;
            self.pump_once()?;
        }
        Ok(offset)
    }

    /// Current congestion window (packets) from the active CC, if connected.
    pub fn cwnd_pkts(&self) -> Option<u32> {
        self.rel.as_ref().map(|r| r.cwnd_pkts())
    }

    /// Current CC sending period (microseconds), if connected.
    pub fn sending_period_us(&self) -> Option<u64> {
        self.rel.as_ref().map(|r| r.sending_period_us())
    }

    /// Receive application bytes. Blocks/pumps until at least one byte is
    /// available or the read timeout budget is exhausted (returns `Ok(0)`).
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        if self.conn.state() != ConnectionState::Connected {
            return Err(SocketError::Transfer("recv requires Connected state"));
        }
        let deadline = Instant::now() + self.read_timeout.unwrap_or(Duration::from_secs(2));
        loop {
            {
                let rel = self.rel.as_mut().ok_or(SocketError::Transfer("no reliability session"))?;
                let n = rel.read_delivered(buf);
                if n > 0 {
                    return Ok(n);
                }
            }
            if Instant::now() >= deadline {
                return Ok(0);
            }
            self.pump_once()?;
        }
    }

    /// Drive the reliability engine until the send buffer is fully ACKed or timeout.
    ///
    /// When `timeout` is zero, uses the socket's `SndTimeo` option (default 30s).
    pub fn flush(&mut self, timeout: Duration) -> Result<(), SocketError> {
        let timeout = if timeout.is_zero() {
            self.opts.snd_timeo.unwrap_or(Duration::from_secs(30))
        } else {
            timeout
        };
        let deadline = Instant::now() + timeout;
        loop {
            let empty = self
                .rel
                .as_ref()
                .map(|r| r.send_buffer_empty())
                .unwrap_or(true);
            if empty {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(SocketError::Transfer("flush timed out with unacked data"));
            }
            self.pump_once()?;
        }
    }

    /// Process one burst of inbound UDP + timers.
    ///
    /// Drains the UDP socket in non-blocking mode so an empty queue does not
    /// burn the full `RcvTimeo` (previously a ~20 ms tax per pump when CWND was
    /// small).
    pub fn pump_once(&mut self) -> Result<(), SocketError> {
        let udp = self
            .udp
            .as_ref()
            .ok_or(SocketError::Transfer("not bound"))?
            .clone();
        let my_id = self.conn.sock_id();
        let now = Instant::now();
        let ts = now_ts();

        // Timer-driven retransmits first.
        let timed = {
            let rel = self.rel.as_mut().ok_or(SocketError::Transfer("no reliability session"))?;
            rel.on_timer_poll(now, ts)
        };
        self.dispatch(timed)?;

        // Non-blocking drain — restore prior timeout afterwards.
        let prior_timeout = self.read_timeout;
        let _ = udp.set_nonblocking(true);
        let mut buf = [0u8; 2048];
        for _ in 0..256 {
            match udp.recv_from(&mut buf) {
                Ok((n, from)) => {
                    if let Some(peer) = self.peer {
                        if from.ip() != peer.ip() {
                            continue;
                        }
                    }
                    if !packet_destined_to(&buf[..n], my_id) {
                        continue;
                    }
                    let out = {
                        let rel = self
                            .rel
                            .as_mut()
                            .ok_or(SocketError::Transfer("no reliability session"))?;
                        rel.on_datagram(&buf[..n], Instant::now(), now_ts())
                    };
                    self.dispatch(out)?;
                }
                Err(e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => {
                    let _ = udp.set_nonblocking(false);
                    let _ = udp.set_read_timeout(prior_timeout);
                    return Err(SocketError::Io(e));
                }
            }
        }
        let _ = udp.set_nonblocking(false);
        let _ = udp.set_read_timeout(prior_timeout);
        Ok(())
    }

    fn dispatch(&mut self, packets: Vec<Outgoing>) -> Result<(), SocketError> {
        let udp = self
            .udp
            .as_ref()
            .ok_or(SocketError::Transfer("not bound"))?
            .clone();
        let peer = self
            .peer
            .ok_or(SocketError::Transfer("no peer address"))?;
        let mut wire = vec![0u8; 65536];
        for pkt in packets {
            let (is_data, n) = match &pkt {
                Outgoing::Data(d) => (true, d.encode(&mut wire)?),
                Outgoing::Ack(a) => (false, a.encode(&mut wire)?),
                Outgoing::Ack2(a) => (false, a.encode(&mut wire)?),
                Outgoing::Nak(n) => (false, n.encode(&mut wire)?),
            };
            if is_data && self.should_drop_data() {
                continue; // induced loss — reliability will retransmit
            }
            udp.send_to(&wire[..n], peer)?;
        }
        Ok(())
    }

    fn should_drop_data(&mut self) -> bool {
        if self.loss_rate <= 0.0 {
            return false;
        }
        // xorshift64*
        let mut x = self.loss_rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.loss_rng = x;
        let unit = (x as f64) / (u64::MAX as f64);
        unit < self.loss_rate
    }

    fn ensure_bound_for(&mut self, peer: SocketAddr) -> Result<(), SocketError> {
        if self.udp.is_some() {
            return Ok(());
        }
        let any = match peer {
            SocketAddr::V4(_) => SocketAddr::from(([0, 0, 0, 0], 0)),
            SocketAddr::V6(_) => SocketAddr::from(([0u16; 8], 0)),
        };
        self.bind(any)
    }
}

impl Default for UdtSocket {
    fn default() -> Self {
        Self::new()
    }
}

fn packet_destined_to(buf: &[u8], sock_id: u32) -> bool {
    if buf.len() < 16 {
        return false;
    }
    // dest sock id at bytes 12..16 for data, ACK, NAK, handshake
    let dest = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
    dest == sock_id || dest == 0
}

fn next_sock_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    if id == 0 {
        NEXT.fetch_add(1, Ordering::Relaxed)
    } else {
        id
    }
}

fn now_ts() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u32)
        .unwrap_or(0)
}

fn ipv4_octets(addr: SocketAddr) -> [u8; 4] {
    match addr {
        SocketAddr::V4(v4) => v4.ip().octets(),
        SocketAddr::V6(_) => [0, 0, 0, 0],
    }
}

/// Apply `UDT_SNDBUF` to the OS UDP send buffer. Best-effort: kernels may
/// clamp or double the value; failures are ignored so option storage still wins.
fn apply_snd_buf(udp: &UdpSocket, bytes: usize) {
    let sock = socket2::SockRef::from(udp);
    let _ = sock.set_send_buffer_size(bytes.max(1024));
}

/// Apply `UDT_RCVBUF` to the OS UDP receive buffer. Best-effort (see [`apply_snd_buf`]).
fn apply_rcv_buf(udp: &UdpSocket, bytes: usize) {
    let sock = socket2::SockRef::from(udp);
    let _ = sock.set_recv_buffer_size(bytes.max(1024));
}

fn send_hs(udp: &UdpSocket, peer: SocketAddr, pkt: &HandshakePacket) -> Result<(), SocketError> {
    let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
    pkt.encode(&mut buf)?;
    udp.send_to(&buf, peer)?;
    Ok(())
}

fn recv_hs_from(
    udp: &UdpSocket,
    mut accept: impl FnMut(&HandshakePacket) -> bool,
) -> Result<(HandshakePacket, SocketAddr), SocketError> {
    // WAN / NAT paths need headroom beyond localhost; cookie HS is 4 packets.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut buf = [0u8; 2048];
    loop {
        if Instant::now() > deadline {
            return Err(SocketError::Handshake("timed out waiting for handshake"));
        }
        match udp.recv_from(&mut buf) {
            Ok((n, from)) => {
                if n < HANDSHAKE_PACKET_LEN {
                    continue;
                }
                match HandshakePacket::decode(&buf[..n]) {
                    Ok(pkt) if accept(&pkt) => return Ok((pkt, from)),
                    _ => continue,
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(e) => return Err(SocketError::Io(e)),
        }
    }
}

#[cfg(feature = "tokio-rt")]
pub mod async_socket {
    //! Tokio async facade over the sync Phase 2–5 implementation.
    //!
    //! Blocking UDP work runs via `tokio::task::spawn_blocking` for connect /
    //! accept / send / recv / flush so the async runtime is not stalled.

    use super::{SocketError, UdtOption, UdtOptionKind, UdtSocket};
    use std::net::SocketAddr;
    use std::time::Duration;

    /// Async UDT socket wrapping the sync implementation.
    #[derive(Debug)]
    pub struct AsyncUdtSocket {
        inner: UdtSocket,
    }

    impl AsyncUdtSocket {
        /// Create a new async socket.
        pub fn new() -> Self {
            Self {
                inner: UdtSocket::new(),
            }
        }

        /// Bind.
        pub async fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
            self.inner.bind(addr)
        }

        /// Listen.
        pub async fn listen(&mut self, backlog: i32) -> Result<(), SocketError> {
            self.inner.listen(backlog)
        }

        /// Connect with handshake (blocking UDP on a worker thread).
        pub async fn connect(&mut self, peer: SocketAddr) -> Result<(), SocketError> {
            let mut sock = std::mem::replace(&mut self.inner, UdtSocket::new());
            let (sock, res) = tokio::task::spawn_blocking(move || {
                let res = sock.connect(peer);
                (sock, res)
            })
            .await
            .map_err(|e| SocketError::Io(std::io::Error::other(e.to_string())))?;
            self.inner = sock;
            res
        }

        /// Accept with handshake (blocking UDP on a worker thread).
        pub async fn accept(&mut self) -> Result<(UdtSocket, SocketAddr), SocketError> {
            let mut sock = std::mem::replace(&mut self.inner, UdtSocket::new());
            let listening = sock.is_listening();
            let (sock, res) = tokio::task::spawn_blocking(move || {
                let res = sock.accept();
                (sock, res)
            })
            .await
            .map_err(|e| SocketError::Io(std::io::Error::other(e.to_string())))?;
            // Restore listener; accept returns the child separately.
            if listening {
                self.inner = sock;
            } else {
                self.inner = sock;
            }
            res
        }

        /// Send bytes (may block briefly on the worker for pump/window).
        pub async fn send(&mut self, buf: Vec<u8>) -> Result<usize, SocketError> {
            let mut sock = std::mem::replace(&mut self.inner, UdtSocket::new());
            let (sock, res) = tokio::task::spawn_blocking(move || {
                let res = sock.send(&buf);
                (sock, res)
            })
            .await
            .map_err(|e| SocketError::Io(std::io::Error::other(e.to_string())))?;
            self.inner = sock;
            res
        }

        /// Receive bytes.
        pub async fn recv(&mut self, max: usize) -> Result<Vec<u8>, SocketError> {
            let mut sock = std::mem::replace(&mut self.inner, UdtSocket::new());
            let (sock, res) = tokio::task::spawn_blocking(move || {
                let mut buf = vec![0u8; max];
                match sock.recv(&mut buf) {
                    Ok(n) => {
                        buf.truncate(n);
                        (sock, Ok(buf))
                    }
                    Err(e) => (sock, Err(e)),
                }
            })
            .await
            .map_err(|e| SocketError::Io(std::io::Error::other(e.to_string())))?;
            self.inner = sock;
            res
        }

        /// Flush unacked data.
        pub async fn flush(&mut self, timeout: Duration) -> Result<(), SocketError> {
            let mut sock = std::mem::replace(&mut self.inner, UdtSocket::new());
            let (sock, res) = tokio::task::spawn_blocking(move || {
                let res = sock.flush(timeout);
                (sock, res)
            })
            .await
            .map_err(|e| SocketError::Io(std::io::Error::other(e.to_string())))?;
            self.inner = sock;
            res
        }

        /// Set option on the inner socket.
        pub fn set_option(&mut self, opt: UdtOption) -> Result<(), SocketError> {
            self.inner.set_option(opt)
        }

        /// Get option from the inner socket.
        pub fn get_option(&self, kind: UdtOptionKind) -> UdtOption {
            self.inner.get_option(kind)
        }

        /// Access inner state.
        pub fn state(&self) -> udt_core::ConnectionState {
            self.inner.state()
        }

        /// Consume into the sync socket.
        pub fn into_inner(self) -> UdtSocket {
            self.inner
        }
    }

    impl Default for AsyncUdtSocket {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn new_is_init() {
        let s = UdtSocket::new();
        assert_eq!(s.state(), ConnectionState::Init);
        assert_eq!(s.sock_type(), UdtSockType::Stream);
    }

    #[test]
    fn socket_options_round_trip() {
        let mut s = UdtSocket::new();
        s.set_option(UdtOption::SndBuf(512 * 1024)).unwrap();
        s.set_option(UdtOption::RcvBuf(128 * 1024)).unwrap();
        s.set_option(UdtOption::Rendezvous(true)).unwrap();
        s.set_option(UdtOption::Mss(1000)).unwrap();
        s.set_option(UdtOption::MaxFlowWindow(1024)).unwrap();
        assert_eq!(s.get_option(UdtOptionKind::SndBuf), UdtOption::SndBuf(512 * 1024));
        assert_eq!(s.get_option(UdtOptionKind::RcvBuf), UdtOption::RcvBuf(128 * 1024));
        assert_eq!(
            s.get_option(UdtOptionKind::Rendezvous),
            UdtOption::Rendezvous(true)
        );
        assert_eq!(s.get_option(UdtOptionKind::Mss), UdtOption::Mss(1000));
        assert_eq!(
            s.get_option(UdtOptionKind::MaxFlowWindow),
            UdtOption::MaxFlowWindow(1024)
        );
    }

    #[test]
    fn bind_records_local() {
        let mut s = UdtSocket::new();
        s.bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        assert_ne!(s.local_addr().unwrap().port(), 0);
    }

    #[test]
    fn listen_requires_bind() {
        let mut s = UdtSocket::new();
        assert!(matches!(s.listen(128), Err(SocketError::Handshake(_))));
    }

    #[test]
    fn listen_marks_listening() {
        let mut s = UdtSocket::new();
        s.bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        s.listen(128).unwrap();
        assert!(s.is_listening());
    }

    #[test]
    fn accept_rejects_closed() {
        let mut s = UdtSocket::new();
        s.bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
        s.listen(1).unwrap();
        s.conn.close().unwrap();
        assert!(matches!(s.accept(), Err(SocketError::State(_))));
    }

    #[test]
    fn localhost_three_way_handshake() {
        let mut server = UdtSocket::new();
        server
            .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .unwrap();
        let server_addr = server.local_addr().unwrap();
        server.listen(8).unwrap();

        let accept_thread = thread::spawn(move || {
            let (child, peer) = server.accept().unwrap();
            assert_eq!(child.state(), ConnectionState::Connected);
            assert_ne!(child.peer_sock_id(), 0);
            (child.sock_id(), child.peer_sock_id(), peer)
        });

        let mut client = UdtSocket::new();
        client
            .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .unwrap();
        client.connect(server_addr).unwrap();
        assert_eq!(client.state(), ConnectionState::Connected);

        let (server_sock_id, server_sees_client, _) = accept_thread.join().unwrap();
        assert_eq!(client.peer_sock_id(), server_sock_id);
        assert_eq!(server_sees_client, client.sock_id());
    }

    #[test]
    fn small_stream_round_trip() {
        let mut server = UdtSocket::new();
        server
            .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .unwrap();
        let addr = server.local_addr().unwrap();
        server.listen(1).unwrap();
        let t = thread::spawn(move || {
            let (mut child, _) = server.accept().unwrap();
            child.set_read_timeout(Some(Duration::from_millis(20)));
            let mut buf = vec![0u8; 64];
            let mut got = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(3);
            while got.len() < 11 && Instant::now() < deadline {
                let n = child.recv(&mut buf).unwrap();
                got.extend_from_slice(&buf[..n]);
            }
            got
        });

        let mut client = UdtSocket::new();
        client.connect(addr).unwrap();
        client.set_read_timeout(Some(Duration::from_millis(20)));
        client.send(b"hello world").unwrap();
        client.flush(Duration::from_secs(3)).unwrap();
        let got = t.join().unwrap();
        assert_eq!(&got, b"hello world");
    }

    #[test]
    fn transfer_8mib_with_one_percent_loss() {
        const N: usize = 8 * 1024 * 1024;
        let payload: Vec<u8> = (0..N).map(|i| (i % 251) as u8).collect();

        let mut server = UdtSocket::new();
        server
            .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .unwrap();
        let addr = server.local_addr().unwrap();
        server.listen(1).unwrap();
        // Loss on server retransmits/ACKs path optional; induce on client send.
        let t = thread::spawn(move || {
            let (mut child, _) = server.accept().unwrap();
            child.set_read_timeout(Some(Duration::from_millis(10)));
            let mut out = vec![0u8; N];
            let mut filled = 0;
            let deadline = Instant::now() + Duration::from_secs(120);
            while filled < N && Instant::now() < deadline {
                let n = child.recv(&mut out[filled..]).unwrap();
                filled += n;
            }
            assert_eq!(filled, N, "server did not receive full 8MiB");
            out
        });

        let mut client = UdtSocket::new();
        client.connect(addr).unwrap();
        client.set_loss_rate(0.01);
        client.set_read_timeout(Some(Duration::from_millis(10)));

        let mut off = 0;
        while off < N {
            // Small chunks so ACKs/NAKs interleave with sends under loss.
            let end = (off + 16 * 1024).min(N);
            client.send(&payload[off..end]).unwrap();
            off = end;
            for _ in 0..8 {
                let _ = client.pump_once();
            }
            // Bound inflight: periodically wait for ACKs.
            if off % (256 * 1024) == 0 || off == N {
                client.flush(Duration::from_secs(30)).unwrap();
            }
        }
        client.flush(Duration::from_secs(120)).unwrap();

        let got = t.join().unwrap();
        assert_eq!(got.len(), N);
        assert_eq!(got, payload);
    }
}
