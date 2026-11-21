//! Short WAN probe: send ~8MB, print cwnd/period samples.
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use udt_socket::{UdtOption, UdtSocket};

fn main() {
    let peer: SocketAddr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "70.88.205.138:19000".into())
        .parse()
        .expect("addr");
    let bytes: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8 * 1024 * 1024);

    let mut client = UdtSocket::new();
    client
        .set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20))))
        .unwrap();
    client
        .set_option(UdtOption::MaxFlowWindow(65_535))
        .unwrap();
    client
        .set_option(UdtOption::SndTimeo(Some(Duration::from_secs(120))))
        .unwrap();
    eprintln!("probe: connect {peer}");
    client.connect(peer).unwrap();
    eprintln!(
        "probe: connected cwnd={:?} period={:?}",
        client.cwnd_pkts(),
        client.sending_period_us()
    );

    let payload = vec![0x5Au8; 64 * 1024];
    let mut sent = 0usize;
    let t0 = Instant::now();
    let mut last_log = t0;
    while sent < bytes {
        let n = client.send(&payload[..payload.len().min(bytes - sent)]).unwrap();
        if n == 0 {
            client.pump_once().unwrap();
            continue;
        }
        sent += n;
        let now = Instant::now();
        if now.duration_since(last_log) >= Duration::from_millis(200) {
            let elapsed = now.duration_since(t0).as_secs_f64().max(1e-6);
            let mibs = sent as f64 / elapsed / 1024.0 / 1024.0;
            eprintln!(
                "t={:.2}s sent={sent} rate={mibs:.3} MiB/s cwnd={:?} period_us={:?} inflight_hint=cwnd",
                elapsed,
                client.cwnd_pkts(),
                client.sending_period_us()
            );
            last_log = now;
        }
    }
    let _ = client.flush(Duration::from_secs(30));
    let elapsed = t0.elapsed().as_secs_f64();
    eprintln!(
        "DONE sent={sent} elapsed={elapsed:.3}s rate={:.3} MiB/s final_cwnd={:?} final_period={:?}",
        sent as f64 / elapsed / 1024.0 / 1024.0,
        client.cwnd_pkts(),
        client.sending_period_us()
    );
}
