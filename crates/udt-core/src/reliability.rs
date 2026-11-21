//! Reliability: ACK/NAK, loss lists, send/recv buffers, retransmission (Phase 3).
//! Congestion window / rate hooks from Phase 4 [`CongestionControl`].

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use crate::cc::{CongestionControl, Udt4Cc};
use crate::packet::PacketType;
use crate::seq::{seq_add, seq_before, seq_norm};
use crate::timer::{TimerKind, TimerWheel};
use crate::cc::UDT_SYN_US;
use crate::wire::{
    peek_is_control, Ack2Packet, AckPacket, DataPacket, NakPacket, DATA_HEADER_LEN,
};

/// Sequence number used on the UDT data path.
pub type SeqNo = u32;

/// Default payload size per data packet (keeps UDP under typical 1500 MTU).
pub const DEFAULT_MSS_PAYLOAD: usize = 1200;
/// Default retransmission timeout (until RTT is measured).
pub const DEFAULT_RTO: Duration = Duration::from_millis(100);
/// Floor for RTO.
pub const MIN_RTO: Duration = Duration::from_millis(50);
/// Ceiling for RTO.
pub const MAX_RTO: Duration = Duration::from_secs(3);

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
    /// ACK of ACK (RTT measurement).
    Ack2(Ack2Packet),
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
    /// Smoothed RTT (microseconds).
    rtt_us: u64,
    /// RTT variance (microseconds).
    rtt_var_us: u64,
    timers: TimerWheel,
    /// Need to emit ACK soon.
    ack_pending: bool,
    /// Need to emit NAK soon.
    nak_pending: bool,
    /// Pluggable congestion control (default [`Udt4Cc`]).
    cc: Box<dyn CongestionControl>,
    /// Earliest Instant the next *new* data packet may leave (rate pacing).
    next_send_at: Instant,
    /// Next ACK sub-sequence number (for ACK2).
    next_ack_number: u32,
    /// When we last sent a full ACK with rate feedback.
    last_full_ack_at: Option<Instant>,
    /// Packets received since `recv_rate_window_start`.
    recv_pkts_in_window: u32,
    /// Start of the current recv-rate measurement window.
    recv_rate_window_start: Instant,
    /// EWMA packet arrival rate (packets/sec).
    recv_rate_pps: u32,
    /// EWMA bandwidth estimate (packets/sec).
    bandwidth_pps: u32,
    /// Advertised flow window (packets) included in full ACKs.
    flow_window_pkts: u32,
    /// Pending ACK2 to send (ack_number).
    pending_ack2: Option<u32>,
    /// ACK numbers we sent → (data ack seq, send time) awaiting ACK2.
    ack_flight: BTreeMap<u32, (SeqNo, Instant)>,
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
        let now = Instant::now();
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
            rtt_us: 100_000,
            rtt_var_us: 50_000,
            timers: TimerWheel::new(),
            ack_pending: false,
            nak_pending: false,
            cc,
            next_send_at: now,
            next_ack_number: 1,
            last_full_ack_at: None,
            recv_pkts_in_window: 0,
            recv_rate_window_start: now,
            recv_rate_pps: 0,
            bandwidth_pps: 0,
            flow_window_pkts: 16_384,
            pending_ack2: None,
            ack_flight: BTreeMap::new(),
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
        self.flow_window_pkts = pkts.max(2);
    }

    /// Current smoothed RTT in microseconds.
    pub fn rtt_us(&self) -> u64 {
        self.rtt_us
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
    /// Stops when the congestion window is full (`inflight >= cwnd`) or the
    /// rate-pacing budget for this call is exhausted. Remaining bytes are
    /// **not** queued — the caller should retry after ACKs / pacing allow more.
    /// Returns `(outgoing, bytes_consumed_from_data)`.
    ///
    /// Packets are paced one SND period apart (virtual clock). Emitting a full
    /// CWND as a single UDP microburst caused heavy WAN loss and pinned rate.
    pub fn enqueue_app_data(
        &mut self,
        data: &[u8],
        now: Instant,
        timestamp: u32,
    ) -> (Vec<Outgoing>, usize) {
        let mut out = Vec::new();
        let mut offset = 0;
        let cwnd = self.cc.cwnd_pkts() as usize;
        let period_us = self.cc.sending_period_us().max(1);
        let period = Duration::from_micros(period_us);
        // Fast path (period < 50µs, typical localhost / early SS): allow a short
        // lookahead burst so one enqueue call can emit many packets. Cap at 16
        // (was 32) — smaller SS bursts cut the size of the first WAN loss hole.
        //
        // WAN path (period ≥ 50µs): emit only packets whose virtual send time
        // has arrived. Lookahead batches were dispatched as UDP microbursts
        // (dispatch sends the whole Vec back-to-back), which caused loss and
        // stalled SND growth on the 138 path.
        const STRICT_PACE_US: u64 = 50;
        const MAX_SS_BURST: usize = 16;
        let strict_pace = period_us >= STRICT_PACE_US;
        let horizon = if strict_pace {
            Duration::ZERO
        } else {
            Duration::from_millis(2).max(period)
        };
        let max_batch = if strict_pace {
            1
        } else {
            ((horizon.as_micros() as u64) / period_us.max(1))
                .max(1)
                .min(MAX_SS_BURST as u64) as usize
        };
        let mut slot = self.next_send_at.max(now);
        let mut batch = 0usize;
        while offset < data.len() && self.unacked.len() < cwnd && batch < max_batch {
            if strict_pace {
                if slot > now {
                    break;
                }
            } else if slot > now + horizon {
                break;
            }
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
            slot += period;
            batch += 1;
        }
        if !out.is_empty() {
            self.next_send_at = slot;
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
                        out.extend(self.on_ack_packet(&ack, now, timestamp));
                    }
                }
                Ok(PacketType::Ack2) => {
                    if let Ok(ack2) = Ack2Packet::decode(buf) {
                        self.on_ack2(&ack2, now);
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
        out.extend(self.flush_control(now, timestamp));
        out
    }

    /// Poll timers (RTO → retransmit oldest outstanding packet).
    pub fn on_timer_poll(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        for kind in self.timers.poll(now) {
            if kind == TimerKind::Rto {
                self.cc.on_timeout();
                // UDT EXP path retransmits the oldest unacked packet — not the
                // entire flight (that collapsed WAN throughput after any RTO).
                out.extend(self.retransmit_oldest_unacked(now, timestamp));
                if !self.unacked.is_empty() {
                    self.arm_rto(now);
                }
            }
        }
        out.extend(self.flush_control(now, timestamp));
        out
    }

    fn on_ack_packet(
        &mut self,
        ack: &AckPacket,
        now: Instant,
        timestamp: u32,
    ) -> Vec<Outgoing> {
        let ack_seq = seq_norm(ack.ack_seq);

        // Local RTT from the newest packet freed by this ACK.
        if let Some((_, sent)) = self
            .unacked
            .iter()
            .rev()
            .find(|(seq, _)| seq_before(**seq, ack_seq))
        {
            let sample = now.saturating_duration_since(sent.last_sent);
            self.update_rtt(sample);
        }

        if let Some(rtt) = ack.rtt_us {
            self.cc.set_rtt_us(u64::from(rtt));
        }
        if let Some(rate) = ack.recv_rate {
            self.cc.set_recv_rate(rate);
        }
        if let Some(bw) = ack.bandwidth {
            self.cc.set_bandwidth(bw);
        }
        if let Some(fw) = ack.flow_window {
            self.cc.set_max_cwnd(fw.max(2));
        }

        self.unacked.retain(|seq, _| !seq_before(*seq, ack_seq));
        self.cc.on_ack(ack_seq);
        if self.unacked.is_empty() {
            self.timers.cancel(TimerKind::Rto);
        } else {
            self.arm_rto(now);
        }

        // Reply ACK2 so the peer can refine RTT (UDT4).
        let mut out = Vec::new();
        if ack.ack_number != 0 {
            out.push(Outgoing::Ack2(Ack2Packet {
                ack_number: ack.ack_number,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
        }
        out
    }

    fn on_ack2(&mut self, ack2: &Ack2Packet, now: Instant) {
        if let Some((_ack_seq, sent_at)) = self.ack_flight.remove(&ack2.ack_number) {
            let sample = now.saturating_duration_since(sent_at);
            self.update_rtt(sample);
        }
    }

    fn update_rtt(&mut self, sample: Duration) {
        let sample_us = sample.as_micros().min(u128::from(u64::MAX)) as u64;
        if sample_us == 0 {
            return;
        }
        // EWMA as in UDT4: RTT = (7*RTT + sample)/8 ; VAR = (3*VAR + |d|)/4
        let diff = sample_us.abs_diff(self.rtt_us);
        self.rtt_var_us = (self.rtt_var_us * 3 + diff) / 4;
        self.rtt_us = (self.rtt_us * 7 + sample_us) / 8;
        self.cc.set_rtt_us(self.rtt_us);
        self.rto = Duration::from_micros(
            (self.rtt_us + 4 * self.rtt_var_us).max(MIN_RTO.as_micros() as u64),
        )
        .min(MAX_RTO);
    }

    fn on_nak(&mut self, lost: &[SeqNo], now: Instant, timestamp: u32) -> Vec<Outgoing> {
        self.cc.set_snd_curr_seq(self.send_next);
        self.cc.on_loss(lost);
        // Cap rexmits per NAK. A full-list blast is a UDP microburst; a single
        // packet leaves large SS holes recovering for many RTTs (seen as multi-
        // second stalls on WAN). Cap keeps recovery moving while the forward
        // path continues to enqueue new data (CWND is not collapsed on ACK).
        const MAX_NAK_REXMIT: usize = 16;
        let mut out = Vec::new();
        let mut n = 0usize;
        for &seq in lost {
            if n >= MAX_NAK_REXMIT {
                break;
            }
            if let Some(pkt) = self.unacked.get_mut(&seq) {
                pkt.last_sent = now;
                out.push(Outgoing::Data(DataPacket {
                    seq,
                    dest_sock_id: self.peer_sock_id,
                    timestamp,
                    payload: pkt.payload.clone(),
                }));
                n += 1;
            }
        }
        if !out.is_empty() {
            self.arm_rto(now);
        }
        out
    }

    fn on_data(&mut self, data: DataPacket, now: Instant, _timestamp: u32) -> Vec<Outgoing> {
        let seq = seq_norm(data.seq);
        self.note_recv_packet(now);
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

    fn note_recv_packet(&mut self, now: Instant) {
        self.recv_pkts_in_window = self.recv_pkts_in_window.saturating_add(1);
        let elapsed = now.saturating_duration_since(self.recv_rate_window_start);
        if elapsed >= Duration::from_micros(UDT_SYN_US) {
            let secs = elapsed.as_secs_f64().max(1e-6);
            let instant = (f64::from(self.recv_pkts_in_window) / secs) as u32;
            if self.recv_rate_pps == 0 {
                self.recv_rate_pps = instant.max(1);
            } else {
                self.recv_rate_pps = (self.recv_rate_pps * 7 + instant) / 8;
            }
            // Bandwidth floor: at least the observed arrival rate.
            if self.bandwidth_pps == 0 {
                self.bandwidth_pps = self.recv_rate_pps.saturating_mul(2).max(16);
            } else {
                let peak = instant.saturating_mul(2).max(self.recv_rate_pps);
                self.bandwidth_pps = (self.bandwidth_pps * 7 + peak) / 8;
            }
            self.recv_pkts_in_window = 0;
            self.recv_rate_window_start = now;
        }
    }

    fn flush_control(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
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
            let send_full = match self.last_full_ack_at {
                None => true,
                Some(t) => now.saturating_duration_since(t)
                    >= Duration::from_micros(UDT_SYN_US),
            };
            let ack_number = self.next_ack_number;
            self.next_ack_number = self.next_ack_number.wrapping_add(1).max(1);
            let ack = if send_full {
                self.last_full_ack_at = Some(now);
                self.ack_flight.insert(ack_number, (self.recv_next, now));
                while self.ack_flight.len() > 1024 {
                    if let Some(k) = self.ack_flight.keys().next().copied() {
                        self.ack_flight.remove(&k);
                    } else {
                        break;
                    }
                }
                AckPacket {
                    ack_seq: self.recv_next,
                    dest_sock_id: self.peer_sock_id,
                    timestamp,
                    ack_number,
                    rtt_us: Some(self.rtt_us.min(u64::from(u32::MAX)) as u32),
                    rtt_var_us: Some(self.rtt_var_us.min(u64::from(u32::MAX)) as u32),
                    flow_window: Some(self.flow_window_pkts.max(2)),
                    // Until the SYN window has samples, advertise a high rate so
                    // the peer CC can open CWND instead of stalling at 16.
                    recv_rate: Some(self.recv_rate_pps.max(1_000)),
                    bandwidth: Some(
                        self.bandwidth_pps
                            .max(self.recv_rate_pps)
                            .max(10_000),
                    ),
                }
            } else {
                // Light ACK keeps data ack_seq in header additional-info (no ack_number).
                AckPacket::light(self.recv_next, self.peer_sock_id, timestamp)
            };
            out.push(Outgoing::Ack(ack));
            self.ack_pending = false;
        }
        if let Some(n) = self.pending_ack2.take() {
            out.push(Outgoing::Ack2(Ack2Packet {
                ack_number: n,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
        }
        out
    }

    fn retransmit_oldest_unacked(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
        let mut out = Vec::new();
        if let Some((&seq, pkt)) = self.unacked.iter_mut().next() {
            pkt.last_sent = now;
            out.push(Outgoing::Data(DataPacket {
                seq,
                dest_sock_id: self.peer_sock_id,
                timestamp,
                payload: pkt.payload.clone(),
            }));
        }
        out
    }

    /// Retransmit every unacked packet (tests / recovery).
    pub fn retransmit_all_unacked(&mut self, now: Instant, timestamp: u32) -> Vec<Outgoing> {
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

    /// Earliest Instant the next *new* data packet may be enqueued (rate pacing).
    pub fn next_send_deadline(&self) -> Instant {
        self.next_send_at
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
        let ack = AckPacket::light(seq_add(d.seq, 1), 1, 2);
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
