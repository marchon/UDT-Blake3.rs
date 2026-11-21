//! Microbench: same 16MiB transfer under different RcvTimeo values.
use std::io::Cursor;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};
use udt_socket::{UdtOption, UdtSocket};

fn run(label: &str, rcv_ms: u64, bytes: usize) {
    let mut server = UdtSocket::new();
    server
        .set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(rcv_ms))))
        .unwrap();
    server
        .set_option(UdtOption::MaxFlowWindow(65_535))
        .unwrap();
    server
        .bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .unwrap();
    let addr = server.local_addr().unwrap();
    server.listen(1).unwrap();

    let t = thread::spawn(move || {
        let (mut child, _) = server.accept().unwrap();
        child
            .set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(rcv_ms))))
            .unwrap();
        let mut out = vec![0u8; bytes];
        let mut got = 0;
        let deadline = Instant::now() + Duration::from_secs(120);
        while got < bytes && Instant::now() < deadline {
            let n = child.recv(&mut out[got..]).unwrap();
            got += n;
        }
        got
    });

    let payload = vec![0xABu8; bytes];
    let mut client = UdtSocket::new();
    client
        .set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(rcv_ms))))
        .unwrap();
    client
        .set_option(UdtOption::MaxFlowWindow(65_535))
        .unwrap();
    client
        .set_option(UdtOption::SndTimeo(Some(Duration::from_secs(120))))
        .unwrap();
    let t0 = Instant::now();
    client.connect(addr).unwrap();
    let mut off = 0;
    while off < bytes {
        let n = client.send(&payload[off..]).unwrap();
        if n == 0 {
            client.pump_once().unwrap();
            continue;
        }
        off += n;
    }
    let _ = client.flush(Duration::from_secs(30));
    let elapsed = t0.elapsed();
    let got = t.join().unwrap();
    let mibs = (bytes as f64) / elapsed.as_secs_f64() / 1024.0 / 1024.0;
    println!(
        "{label}: rcv_timeo={rcv_ms}ms bytes={got} elapsed={:.3}s rate={:.2} MiB/s cwnd={:?} period_us={:?}",
        elapsed.as_secs_f64(),
        mibs,
        client.cwnd_pkts(),
        client.sending_period_us()
    );
}

fn main() {
    let bytes = 16 * 1024 * 1024;
    for ms in [20u64, 5, 1] {
        run(&format!("t{ms}"), ms, bytes);
    }
}
