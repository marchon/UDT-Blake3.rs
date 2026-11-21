//! Reliability: ACK/NAK, loss lists, send/recv buffers, retransmission (Phase 3).
//! Congestion window / rate hooks from Phase 4 [`CongestionControl`].

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use crate::cc::{CongestionControl, Udt4Cc};
use crate::packet::PacketType;
use crate::seq::{seq_add, seq_before, seq_norm};
use crate::timer::{TimerKind, TimerWheel};
use crate::wire::{peek_is_control, AckPacket, DataPacket, NakPacket, DATA_HEADER_LEN};

/// Sequence number used on the UDT data path.
pub type SeqNo = u32;

/// Default payload size per data packet (keeps UDP under typical 1500 MTU).
pub const DEFAULT_MSS_PAYLOAD: usize = 1200;
/// Default retransmission timeout.
pub const DEFAULT_RTO: Duration = Duration::from_millis(100);

/// Tracks packets that appear lost and need retransmission or NAK.
#[derive(Debug, Default, Clone)]
pub struct LossList {
    seqs: Vec<SeqNo>,
}

impl LossList {
    /// Create an empty loss list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a missing sequence number (idempotent).
    pub fn insert(&mut self, seq: SeqNo) {
        let seq = seq_norm(seq);
        if !self.seqs.contains(&seq) {
            self.seqs.push(seq);
            self.seqs.sort_unstable();
        }
    }

    /// Insert every seq in `[start, end)` (end exclusive) into the loss list.
    ///
    /// Stops after `max` insertions to bound NAK size under heavy reordering.
    pub fn insert_range(&mut self, start: SeqNo, end: SeqNo, max: usize) {
        let mut s = seq_norm(start);
        let mut n = 0;
        while seq_before(s, end) && n < max {
            self.insert(s);
            s = seq_add(s, 1);
            n += 1;
        }
    }

    /// Remove a sequence after ACK or successful repair.
    pub fn remove(&mut self, seq: SeqNo) {
        let seq = seq_norm(seq);
        self.seqs.retain(|x| *x != seq);
    }

    /// Drop all seqs strictly before `ack_seq`.
    pub fn remove_before(&mut self, ack_seq: SeqNo) {
        self.seqs.retain(|s| !seq_before(*s, ack_seq));
    }

    /// Current loss list (sorted).
    pub fn as_slice(&self) -> &[SeqNo] {
        &self.seqs
    }

    /// Number of outstanding losses.
    pub fn len(&self) -> usize {
        self.seqs.len()
    }

    /// Returns true when no losses are recorded.
    pub fn is_empty(&self) -> bool {
        self.seqs.is_empty()
    }
}

#[derive(Debug, Clone)]
struct SentPacket {
    payload: Vec<u8>,
    last_sent: Instant,
}

/// Packets the reliability layer wants put on the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    /// Data packet.
    Data(DataPacket),
    /// Cumulative ACK.
    Ack(AckPacket),
    /// Loss report.
    Nak(NakPacket),
}

/// Reliable byte-stream engine (no I/O).
pub struct ReliabilityEngine {
    /// Next seq to assign for new data.
    send_next: SeqNo,
    /// Unacknowledged data.
    unacked: BTreeMap<SeqNo, SentPacket>,
    /// Next in-order seq expected from peer.
    recv_next: SeqNo,
    /// Out-of-order payloads.
    reorder: BTreeMap<SeqNo, Vec<u8>>,
    /// Gaps for NAK.
    loss_list: LossList,
    /// Delivered byte stream for the application.
    delivered: VecDeque<u8>,
    /// Peer UDT socket id (dest on outbound).
    peer_sock_id: u32,
    /// Local socket id (unused on wire except identity).
    #[allow(dead_code)]
    local_sock_id: u32,
    /// Payload MSS.
    mss: usize,
    /// Retransmission timeout.
    rto: Duration,
    timers: TimerWheel,
    /// Need to emit ACK soon.
    ack_pending: bool,
    /// Need to emit NAK soon.
    nak_pending: bool,
    /// Pluggable congestion control (default [`Udt4Cc`]).
    cc: Box<dyn CongestionControl>,
    /// Earliest Instant the next *new* data packet may leave (rate pacing).
    next_send_at: Instant,
}

impl std::fmt::Debug for ReliabilityEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReliabilityEngine")
            .field("send_next", &self.send_next)
            .field("unacked", &self.unacked.len())
            .field("recv_next", &self.recv_next)
            .field("cwnd", &self.cc.cwnd_pkts())
            .field("period_us", &self.cc.sending_period_us())
            .finish()
    }
}

impl ReliabilityEngine {
    /// Create an engine after handshake with default [`Udt4Cc`].
    ///
    /// `local_init_seq` is this side's first data seq; `peer_init_seq` is the
    /// first data seq expected from the peer.
    pub fn new(
        local_sock_id: u32,
        peer_sock_id: u32,
        local_init_seq: SeqNo,
        peer_init_seq: SeqNo,
    ) -> Self {
        Self::with_cc(
            local_sock_id,
            peer_sock_id,
            local_init_seq,
            peer_init_seq,
            Box::new(Udt4Cc::new()),
        )
    }

    /// Create an engine with a custom congestion controller (tests / alt CC).
    pub fn with_cc(
        local_sock_id: u32,
        peer_sock_id: u32,
        local_init_seq: SeqNo,
        peer_init_seq: SeqNo,
        mut cc: Box<dyn CongestionControl>,
    ) -> Self {
        cc.set_snd_curr_seq(seq_norm(local_init_seq));
        cc.set_mss(DEFAULT_MSS_PAYLOAD as u32 + DATA_HEADER_LEN as u32);
        Self {
            send_next: seq_norm(local_init_seq),
            unacked: BTreeMap::new(),
            recv_next: seq_norm(peer_init_seq),
            reorder: BTreeMap::new(),
            loss_list: LossList::new(),
            delivered: VecDeque::new(),
            peer_sock_id,
            local_sock_id,
            mss: DEFAULT_MSS_PAYLOAD,
            rto: DEFAULT_RTO,
            timers: TimerWheel::new(),
            ack_pending: false,
            nak_pending: false,
            cc,
            next_send_at: Instant::now(),
        }
    }

    /// Current congestion window (packets).
    pub fn cwnd_pkts(&self) -> u32 {
        self.cc.cwnd_pkts()
    }

    /// Current sending period (microseconds).
    pub fn sending_period_us(&self) -> u64 {
        self.cc.sending_period_us()
    }

    /// Replace the congestion controller.
    pub fn set_cc(&mut self, cc: Box<dyn CongestionControl>) {
        self.cc = cc;
    }

    /// Set payload MSS (bytes) for future segments.
    pub fn set_mss(&mut self, mss: usize) {
        self.mss = mss.clamp(64, 64 * 1024);
        self.cc
            .set_mss((self.mss + DATA_HEADER_LEN) as u32);
    }

    /// Set maximum congestion / flow window (packets).
    pub fn set_max_cwnd(&mut self, pkts: u32) {
        self.cc.set_max_cwnd(pkts);
    }

    /// Mutable access to the congestion controller.
    pub fn cc_mut(&mut self) -> &mut dyn CongestionControl {
        self.cc.as_mut()
    }

    /// Bytes still buffered for the application.
    pub fn delivered_len(&self) -> usize {
        self.delivered.len()
    }

    /// True when all sent data has been cumulatively acknowledged.
    pub fn send_buffer_empty(&self) -> bool {
        self.unacked.is_empty()
    }

    /// Number of unacked packets.
    pub fn inflight_packets(&self) -> usize {
        self.unacked.len()
    }

    /// Copy delivered bytes into `out`; returns bytes copied.
    pub fn read_delivered(&mut self, out: &mut [u8]) -> usize {
        let n = out.len().min(self.delivered.len());
        for slot in out.iter_mut().take(n) {
            *slot = self.delivered.pop_front().expect("len checked");
        }
        n
    }

    /// Segment `data` into packets and return wire messages to send now.
    ///
    /// Stops when the congestion window is full (`inflight >= cwnd`). Remaining
    /// bytes are **not** queued — the caller should retry after ACKs free window.
    /// Returns `(outgoing, bytes_consumed_from_data)`.
    pub fn enqueue_app_data(
        &mut self,
        data: &[u8],
        now: Instant,
        timestamp: u32,
    ) -> (Vec<Outgoing>, usize) {
        let mut out = Vec::new();
        let mut offset = 0;
        let cwnd = self.cc.cwnd_pkts() as usize;
        // If the rate gate is still closed, emit nothing this call.
        if now < self.next_send_at {
            return (out, 0);
        }
        let period = Duration::from_micros(self.cc.sending_period_us());
        // Fill up to cwnd in this call; virtual pacing advances next_send_at for
        // subsequent calls (socket layer may also sleep on period).
        while offset < data.len() && self.unacked.len() < cwnd {
            let end = (offset + self.mss).min(data.len());
            let payload = data[offset..end].to_vec();
            let seq = self.send_next;
            self.send_next = seq_add(self.send_next, 1);
            self.cc.set_snd_curr_seq(seq);
            self.unacked.insert(
                seq,
                SentPacket {
                    payload: payload.clone(),
                    last_sent: now,
                },
            );
            out.push(Outgoing::Data(DataPacket {
                seq,
                dest_sock_id: self.peer_sock_id,
                timestamp,
                payload,
            }));
            offset = end;
        }
        if !out.is_empty() {
            // Gate the *next* enqueue burst; do not multiply by packet count
            // (wire pacing is approximate until a dedicated sender thread exists).
            self.next_send_at = now + period;
            self.arm_rto(now);
        }
        (out, offset)
    }

    /// Legacy helper: enqueue and ignore the consumed-length (tests).
    pub fn enqueue_app_data_all(
        &mut self,
        data: &[u8],
        now: Instant,
        timestamp: u32,
    ) -> Vec<Outgoing> {
        self.enqueue_app_data(data, now, timestamp).0
    }

    /// Handle a raw UDP datagram; returns packets to send (ACK/NAK/rexmit).
    pub fn on_datagram(&mut self, buf: &[u8], now: Instant, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        let is_ctrl = match peek_is_control(buf) {
            Ok(v) => v,
            Err(_) => return out,
        };
        if is_ctrl {
            match crate::packet::peek_type(buf) {
                Ok(PacketType::Ack) => {
                    if let Ok(ack) = AckPacket::decode(buf) {
                        out.extend(self.on_ack(ack.ack_seq, now));
                    }
                }
                Ok(PacketType::Nak) => {
                    if let Ok(nak) = NakPacket::decode(buf) {
                        out.extend(self.on_nak(&nak.lost, now, timestamp));
                    }
                }
                _ => {}
            }
        } else if let Ok(data) = DataPacket::decode(buf) {
            out.extend(self.on_data(data, now, timestamp));
        }
        out.extend(self.flush_control(timestamp));
        out
    }

    /// Poll timers (RTO → retransmit).
    pub fn on_timer_poll(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        for kind in self.timers.poll(now) {
            if kind == TimerKind::Rto {
                self.cc.on_timeout();
                out.extend(self.retransmit_all_unacked(now, timestamp));
                if !self.unacked.is_empty() {
                    self.arm_rto(now);
                }
            }
        }
        out.extend(self.flush_control(timestamp));
        out
    }

    fn on_ack(&mut self, ack_seq: SeqNo, now: Instant) -> Vec<Outgoing> {
        let ack_seq = seq_norm(ack_seq);
        self.unacked.retain(|seq, _| !seq_before(*seq, ack_seq));
        self.cc.on_ack(ack_seq);
        if self.unacked.is_empty() {
            self.timers.cancel(TimerKind::Rto);
        } else {
            self.arm_rto(now);
        }
        Vec::new()
    }

    fn on_nak(&mut self, lost: &[SeqNo], now: Instant, timestamp: u32) -> Vec<Outgoing> {
        self.cc.set_snd_curr_seq(self.send_next);
        self.cc.on_loss(lost);
        let mut out = Vec::new();
        for seq in lost {
            if let Some(pkt) = self.unacked.get_mut(seq) {
                pkt.last_sent = now;
                out.push(Outgoing::Data(DataPacket {
                    seq: *seq,
                    dest_sock_id: self.peer_sock_id,
                    timestamp,
                    payload: pkt.payload.clone(),
                }));
            }
        }
        if !out.is_empty() {
            self.arm_rto(now);
        }
        out
    }

    fn on_data(&mut self, data: DataPacket, _now: Instant, _timestamp: u32) -> Vec<Outgoing> {
        let seq = seq_norm(data.seq);
        if seq_before(seq, self.recv_next) {
            // duplicate of already delivered — still ACK
            self.ack_pending = true;
            return Vec::new();
        }
        if seq == self.recv_next {
            self.delivered.extend(data.payload);
            self.recv_next = seq_add(self.recv_next, 1);
            self.loss_list.remove(seq);
            // drain reorder
            while let Some(payload) = self.reorder.remove(&self.recv_next) {
                self.delivered.extend(payload);
                self.loss_list.remove(self.recv_next);
                self.recv_next = seq_add(self.recv_next, 1);
            }
        } else {
            // gap — record a bounded prefix of missing seqs for NAK
            self.loss_list.insert_range(self.recv_next, seq, 64);
            self.reorder.entry(seq).or_insert(data.payload);
            self.nak_pending = true;
        }
        self.ack_pending = true;
        Vec::new()
    }

    fn flush_control(&mut self, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        // Cap NAK size so the datagram stays under a typical 1500–8K UDP payload.
        const MAX_NAK_SEQS: usize = 64;
        if self.nak_pending && !self.loss_list.is_empty() {
            let lost: Vec<SeqNo> = self
                .loss_list
                .as_slice()
                .iter()
                .copied()
                .take(MAX_NAK_SEQS)
                .collect();
            out.push(Outgoing::Nak(NakPacket {
                lost,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
            self.nak_pending = false;
        }
        if self.ack_pending {
            out.push(Outgoing::Ack(AckPacket {
                ack_seq: self.recv_next,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
            self.ack_pending = false;
        }
        out
    }

    fn retransmit_all_unacked(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        for (seq, pkt) in self.unacked.iter_mut() {
            pkt.last_sent = now;
            out.push(Outgoing::Data(DataPacket {
                seq: *seq,
                dest_sock_id: self.peer_sock_id,
                timestamp,
                payload: pkt.payload.clone(),
            }));
        }
        out
    }

    fn arm_rto(&mut self, now: Instant) {
        self.timers.schedule(TimerKind::Rto, now + self.rto);
    }

    /// Next timer deadline for integrating with socket poll timeouts.
    pub fn next_timer(&self) -> Option<Instant> {
        self.timers.next_deadline()
    }

    /// Encode helper size hint for data.
    pub fn data_overhead() -> usize {
        DATA_HEADER_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_sorted_unique() {
        let mut l = LossList::new();
        l.insert(10);
        l.insert(5);
        l.insert(10);
        assert_eq!(l.as_slice(), &[5, 10]);
    }

    #[test]
    fn in_order_delivery_and_ack() {
        let mut a = ReliabilityEngine::new(1, 2, 0, 0);
        let mut b = ReliabilityEngine::new(2, 1, 0, 0);
        let now = Instant::now();
        let (pkts, ncons) = a.enqueue_app_data(b"abcdef", now, 1);
        assert_eq!(ncons, 6);
        assert_eq!(pkts.len(), 1);
        let Outgoing::Data(d) = &pkts[0] else {
            panic!("expected data");
        };
        let mut buf = vec![0u8; 2048];
        let n = d.encode(&mut buf).unwrap();
        let resp = b.on_datagram(&buf[..n], now, 2);
        assert!(resp.iter().any(|o| matches!(o, Outgoing::Ack(_))));
        let mut out = [0u8; 16];
        assert_eq!(b.read_delivered(&mut out), 6);
        assert_eq!(&out[..6], b"abcdef");
    }

    #[test]
    fn gap_triggers_nak_and_recovers() {
        let mut sender = ReliabilityEngine::new(1, 2, 0, 0);
        let mut recv = ReliabilityEngine::new(2, 1, 0, 0);
        let now = Instant::now();
        sender.mss = 3;
        sender.next_send_at = now;
        let (pkts, _) = sender.enqueue_app_data(b"abcdef", now, 1);
        assert_eq!(pkts.len(), 2);
        // Deliver only second packet first
        let Outgoing::Data(d1) = &pkts[1] else {
            panic!()
        };
        let mut buf = vec![0u8; 64];
        let n = d1.encode(&mut buf).unwrap();
        let out = recv.on_datagram(&buf[..n], now, 2);
        assert!(out.iter().any(|o| matches!(o, Outgoing::Nak(_))));
        // Deliver first
        let Outgoing::Data(d0) = &pkts[0] else {
            panic!()
        };
        let n = d0.encode(&mut buf).unwrap();
        let _ = recv.on_datagram(&buf[..n], now, 3);
        let mut outb = [0u8; 16];
        assert_eq!(recv.read_delivered(&mut outb), 6);
        assert_eq!(&outb[..6], b"abcdef");
    }

    #[test]
    fn ack_frees_send_buffer() {
        let mut a = ReliabilityEngine::new(1, 2, 0, 0);
        let now = Instant::now();
        let (pkts, _) = a.enqueue_app_data(b"hi", now, 1);
        assert_eq!(a.inflight_packets(), 1);
        let Outgoing::Data(d) = &pkts[0] else {
            panic!()
        };
        let ack = AckPacket {
            ack_seq: seq_add(d.seq, 1),
            dest_sock_id: 1,
            timestamp: 2,
        };
        let mut buf = [0u8; 16];
        ack.encode(&mut buf).unwrap();
        let _ = a.on_datagram(&buf, now, 2);
        assert!(a.send_buffer_empty());
    }

    #[test]
    fn cwnd_limits_inflight() {
        use crate::cc::FixedCc;
        let mut eng = ReliabilityEngine::with_cc(1, 2, 0, 0, Box::new(FixedCc::new(1, 2)));
        let now = Instant::now();
        eng.mss = 1;
        eng.next_send_at = now;
        let data = b"abcdef"; // 6 packets at mss=1
        let (pkts, consumed) = eng.enqueue_app_data(data, now, 1);
        assert_eq!(pkts.len(), 2);
        assert_eq!(consumed, 2);
        assert_eq!(eng.inflight_packets(), 2);
    }
}
