//! Congestion control trait and UDT4 native CC (Phase 4).
//!
//! Default algorithm follows draft-gg-udt-03 / UDT4 `CUDTCC`: hybrid rate
//! (packet send period SND) + congestion window (CWND).

use std::time::{Duration, Instant};

/// SYN interval used by native UDT rate control (10 ms), in microseconds.
pub const UDT_SYN_US: u64 = 10_000;
/// Minimum rate-increase factor per SYN (packets).
pub const UDT_MIN_INC: f64 = 0.1;
/// Beta constant from the UDT rate-increase formula.
pub const UDT_BETA: f64 = 0.000_001_5;
/// Initial congestion window (packets).
pub const UDT_INIT_CWND: f64 = 16.0;
/// Initial packet send period (microseconds). `1` ≈ “as fast as possible”.
pub const UDT_INIT_PERIOD_US: f64 = 1.0;
/// SND multiply on loss / timeout (UDT4 uses 1.125; gentler keeps WAN climbing).
pub const UDT_LOSS_PERIOD_MULT: f64 = 1.03;
/// CWND factor applied once per loss event (⅞ ≈ small reduction, not a collapse).
pub const UDT_LOSS_CWND_MULT: f64 = 0.875;
/// After slow-start exit without recv_rate, cap seeded SND (µs) so a default
/// 100 ms RTT does not start CA at multi-millisecond pacing.
pub const UDT_SS_EXIT_PERIOD_CAP_US: f64 = 500.0;
/// Per-SYN CWND growth cap toward the rate-based target.
pub const UDT_CWND_GROW_PER_SYN: f64 = 48.0;

/// Pluggable congestion control (maps to C++ `CCC`).
pub trait CongestionControl: Send {
    /// Called when an ACK advances the cumulative sequence.
    fn on_ack(&mut self, ack_seq: u32);
    /// Called when loss is detected (NAK or timeout path).
    fn on_loss(&mut self, loss_seqs: &[u32]);
    /// Called on retransmission timeout.
    fn on_timeout(&mut self) {
        self.on_loss(&[]);
    }
    /// Current sending period in microseconds (UDT SND).
    fn sending_period_us(&self) -> u64;
    /// Congestion window in packets (ceil).
    fn cwnd_pkts(&self) -> u32;
    /// Update smoothed RTT sample (microseconds).
    fn set_rtt_us(&mut self, _rtt_us: u64) {}
    /// Update receiver-reported packet arrival rate (packets/sec).
    fn set_recv_rate(&mut self, _pkts_per_sec: u32) {}
    /// Update estimated link capacity (packets/sec).
    fn set_bandwidth(&mut self, _pkts_per_sec: u32) {}
    /// Update sender's current data sequence (for loss decrease bookkeeping).
    fn set_snd_curr_seq(&mut self, _seq: u32) {}
    /// Maximum congestion window (flow window).
    fn set_max_cwnd(&mut self, _pkts: u32) {}
    /// Maximum segment size in bytes (for rate-increase formula).
    fn set_mss(&mut self, _mss: u32) {}
}

/// Native UDT4 congestion control (`CUDTCC`-compatible behavior).
#[derive(Debug, Clone)]
pub struct Udt4Cc {
    /// Packet sending period SND (microseconds), stored as float for formula fidelity.
    period_us: f64,
    /// Congestion window size (packets, float as in reference).
    cwnd: f64,
    /// Flow / max window.
    max_cwnd: f64,
    /// MSS in bytes.
    mss: u32,
    /// RTT sample (microseconds).
    rtt_us: u64,
    /// Packet arrival rate from receiver (packets/sec).
    recv_rate: u32,
    /// Estimated bandwidth (packets/sec).
    bandwidth: u32,
    /// Slow-start flag.
    slow_start: bool,
    /// Last ACK seq seen in slow start.
    last_ack_seq: u32,
    /// Sender current seq (for LastDecSeq bookkeeping).
    snd_curr_seq: u32,
    /// AvgNAKNum.
    avg_nak_num: f64,
    /// NAKCount.
    nak_count: u32,
    /// DecCount.
    dec_count: u32,
    /// LastDecSeq.
    last_dec_seq: u32,
    /// LastDecPeriod.
    last_dec_period: f64,
    /// DecRandom.
    dec_random: u32,
    /// Loss reported since last rate-increase ACK.
    loss_since_ack: bool,
    /// Last time rate control ran.
    last_rc_time: Option<Instant>,
    /// Test clock override (when set, used instead of Instant::now).
    clock: Option<Instant>,
}

impl Default for Udt4Cc {
    fn default() -> Self {
        Self::new()
    }
}

impl Udt4Cc {
    /// Create CC with UDT4 defaults (CWND=16, SND≈1µs, slow start on).
    pub fn new() -> Self {
        Self {
            period_us: UDT_INIT_PERIOD_US,
            cwnd: UDT_INIT_CWND,
            max_cwnd: 16_384.0,
            mss: 1500,
            rtt_us: 100_000, // 100 ms default until measured
            recv_rate: 0,
            bandwidth: 0,
            slow_start: true,
            last_ack_seq: 0,
            snd_curr_seq: 0,
            avg_nak_num: 1.0,
            nak_count: 1,
            dec_count: 1,
            last_dec_seq: u32::MAX, // "init_seq - 1" stand-in until set
            last_dec_period: UDT_INIT_PERIOD_US,
            dec_random: 1,
            loss_since_ack: false,
            last_rc_time: None,
            clock: None,
        }
    }

    /// Pin the CC clock for deterministic unit tests.
    pub fn set_clock(&mut self, now: Instant) {
        self.clock = Some(now);
    }

    /// Advance the pinned clock (tests).
    pub fn advance_clock(&mut self, by: Duration) {
        if let Some(c) = &mut self.clock {
            *c += by;
        }
    }

    fn now(&self) -> Instant {
        self.clock.unwrap_or_else(Instant::now)
    }

    fn set_period(&mut self, period: f64) {
        // Floor 1µs; ceiling one packet per SYN (10ms). Without receiver
        // rate/bandwidth feedback, unbounded 1.125× decreases would stall localhost.
        self.period_us = period.clamp(1.0, UDT_SYN_US as f64);
    }

    fn set_cwnd(&mut self, cwnd: f64) {
        self.cwnd = cwnd.clamp(2.0, self.max_cwnd);
    }

    /// Reference rate-increase `inc` (packets per SYN).
    pub fn compute_inc(bandwidth: u32, period_us: f64, mss: u32) -> f64 {
        let c = if period_us <= 0.0 {
            0.0
        } else {
            1_000_000.0 / period_us
        };
        let b = bandwidth as f64;
        if b <= c {
            return UDT_MIN_INC;
        }
        let mss = mss.max(1) as f64;
        let spare_bits = (b - c) * mss * 8.0;
        if spare_bits <= 1.0 {
            return UDT_MIN_INC;
        }
        let exp = spare_bits.log10().ceil();
        let inc = 10f64.powf(exp) * UDT_BETA / mss;
        inc.max(UDT_MIN_INC)
    }

    fn seed_period_after_ss_exit(&mut self) {
        if self.recv_rate > 0 {
            self.set_period(1_000_000.0 / f64::from(self.recv_rate));
        } else {
            let denom = self.cwnd.max(1.0);
            let seeded = (self.rtt_us as f64 + UDT_SYN_US as f64) / denom;
            self.set_period(seeded.min(UDT_SS_EXIT_PERIOD_CAP_US));
        }
    }

    fn update_snd_on_ack(&mut self) {
        let mut inc = Self::compute_inc(self.bandwidth, self.period_us, self.mss);
        // Short flows spent most of an 8 MiB transfer climbing from ~700 µs.
        // Boost inc while SND is still coarse so asymptote arrives sooner.
        if self.period_us >= 200.0 {
            inc *= 3.0;
        } else if self.period_us >= 100.0 {
            inc *= 2.0;
        }
        let inc = inc.max(UDT_MIN_INC);
        // SND = (SND * SYN) / (SND * inc + SYN)
        let syn = UDT_SYN_US as f64;
        let snd = self.period_us;
        self.set_period((snd * syn) / (snd * inc + syn));
    }
}

impl CongestionControl for Udt4Cc {
    fn on_ack(&mut self, ack_seq: u32) {
        let now = self.now();
        // Rate-control at most once per SYN.
        if let Some(last) = self.last_rc_time {
            if now.duration_since(last) < Duration::from_micros(UDT_SYN_US) {
                // Still track slow-start ack high-water inside the same SYN.
                if self.slow_start && ack_seq > self.last_ack_seq {
                    let delta = ack_seq.wrapping_sub(self.last_ack_seq) as f64;
                    self.set_cwnd(self.cwnd + delta);
                    self.last_ack_seq = ack_seq;
                }
                return;
            }
        }
        self.last_rc_time = Some(now);

        if self.slow_start {
            if ack_seq > self.last_ack_seq {
                let delta = ack_seq.wrapping_sub(self.last_ack_seq) as f64;
                self.set_cwnd(self.cwnd + delta);
                self.last_ack_seq = ack_seq;
            }
            if self.cwnd >= self.max_cwnd {
                self.slow_start = false;
                self.seed_period_after_ss_exit();
            }
            return;
        }

        // Congestion avoidance: target CWND = A*(RTT+SYN)+16 when rate is known.
        // Grow toward target with a per-SYN cap (avoids ballooning to hundreds of
        // in-flight packets ahead of a loss hole). Shrink at most ⅞ per SYN so a
        // transient recv_rate dip does not stall new transmits during NAK recovery.
        if self.recv_rate > 0 {
            let a = f64::from(self.recv_rate);
            let target =
                a / 1_000_000.0 * (self.rtt_us as f64 + UDT_SYN_US as f64) + 16.0;
            if target > self.cwnd {
                self.set_cwnd((self.cwnd + UDT_CWND_GROW_PER_SYN).min(target));
            } else {
                let floor = self.cwnd * UDT_LOSS_CWND_MULT;
                self.set_cwnd(target.max(floor));
            }
        } else if self.cwnd < self.max_cwnd {
            // Gentle growth without rate feedback so a post-loss session can recover.
            self.set_cwnd(self.cwnd + 1.0);
        }

        // Keep increasing SND even after a recent NAK — do not freeze the forward
        // rate while recovery retransmits run in parallel.
        self.loss_since_ack = false;
        self.update_snd_on_ack();
    }

    fn on_loss(&mut self, loss_seqs: &[u32]) {
        if self.slow_start {
            self.slow_start = false;
            self.seed_period_after_ss_exit();
        }

        self.loss_since_ack = true;
        // Small window reduction — enough to react, small enough that new data
        // can still be in flight alongside retransmits.
        self.set_cwnd(self.cwnd * UDT_LOSS_CWND_MULT);

        let first = loss_seqs.first().copied().unwrap_or(self.snd_curr_seq);
        // Sequence compare in 31-bit space: treat wrapping distance.
        let loss_after_last_dec = first != self.last_dec_seq
            && first.wrapping_sub(self.last_dec_seq) < (1 << 30);

        if loss_after_last_dec || self.last_dec_seq == u32::MAX {
            self.last_dec_period = self.period_us;
            self.set_period(self.period_us * UDT_LOSS_PERIOD_MULT);
            self.avg_nak_num = self.avg_nak_num * 0.875 + f64::from(self.nak_count) * 0.125;
            self.nak_count = 1;
            self.dec_count = 1;
            self.last_dec_seq = self.snd_curr_seq;
            let mut dr = (self.avg_nak_num
                * pseudo_random_unit(self.last_dec_seq))
            .ceil() as u32;
            if dr < 1 {
                dr = 1;
            }
            self.dec_random = dr;
            return;
        }

        let prev = self.dec_count;
        self.dec_count = self.dec_count.saturating_add(1);
        if prev < 5 {
            self.nak_count = self.nak_count.saturating_add(1);
            if self.dec_random > 0 && self.nak_count % self.dec_random == 0 {
                self.set_period(self.period_us * UDT_LOSS_PERIOD_MULT);
                self.last_dec_seq = self.snd_curr_seq;
            }
        }
    }

    fn on_timeout(&mut self) {
        if self.slow_start {
            self.slow_start = false;
            self.seed_period_after_ss_exit();
        } else {
            self.set_period(self.period_us * UDT_LOSS_PERIOD_MULT);
            self.set_cwnd(self.cwnd * UDT_LOSS_CWND_MULT);
        }
        self.loss_since_ack = true;
    }

    fn sending_period_us(&self) -> u64 {
        self.period_us.round().max(1.0) as u64
    }

    fn cwnd_pkts(&self) -> u32 {
        self.cwnd.round().max(2.0) as u32
    }

    fn set_rtt_us(&mut self, rtt_us: u64) {
        self.rtt_us = rtt_us.max(1);
    }

    fn set_recv_rate(&mut self, pkts_per_sec: u32) {
        self.recv_rate = pkts_per_sec;
    }

    fn set_bandwidth(&mut self, pkts_per_sec: u32) {
        self.bandwidth = pkts_per_sec;
    }

    fn set_snd_curr_seq(&mut self, seq: u32) {
        self.snd_curr_seq = seq;
    }

    fn set_max_cwnd(&mut self, pkts: u32) {
        self.max_cwnd = f64::from(pkts.max(2));
        if self.cwnd > self.max_cwnd {
            self.cwnd = self.max_cwnd;
        }
    }

    fn set_mss(&mut self, mss: u32) {
        self.mss = mss.max(1);
    }
}

/// Deterministic stand-in for the C++ decrease random factor in `[0, 1)`.
fn pseudo_random_unit(seed: u32) -> f64 {
    let x = seed
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(0xA5A5_5A5A);
    f64::from(x % 10_000) / 10_000.0
}

/// Fixed CC for tests: constant window and period.
#[derive(Debug, Clone)]
pub struct FixedCc {
    /// Send period (µs).
    pub period_us: u64,
    /// Window (packets).
    pub cwnd: u32,
}

impl FixedCc {
    /// Construct a fixed CC profile.
    pub fn new(period_us: u64, cwnd: u32) -> Self {
        Self {
            period_us: period_us.max(1),
            cwnd: cwnd.max(1),
        }
    }
}

impl CongestionControl for FixedCc {
    fn on_ack(&mut self, _ack_seq: u32) {}
    fn on_loss(&mut self, _loss_seqs: &[u32]) {}
    fn sending_period_us(&self) -> u64 {
        self.period_us
    }
    fn cwnd_pkts(&self) -> u32 {
        self.cwnd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_defaults() {
        let cc = Udt4Cc::new();
        assert_eq!(cc.cwnd_pkts(), 16);
        assert_eq!(cc.sending_period_us(), 1);
        assert!(cc.slow_start);
    }

    #[test]
    fn slow_start_grows_cwnd_on_ack() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.last_ack_seq = 0;
        cc.on_ack(5); // +5
        assert_eq!(cc.cwnd_pkts(), 21);
        assert!(cc.slow_start);
    }

    #[test]
    fn loss_exits_slow_start_and_raises_period() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.set_rtt_us(50_000);
        cc.set_snd_curr_seq(100);
        let before = cc.sending_period_us();
        cc.on_loss(&[90]);
        assert!(!cc.slow_start);
        assert!(
            cc.sending_period_us() as f64 >= (before as f64) * (UDT_LOSS_PERIOD_MULT - 0.001)
        );
    }

    #[test]
    fn loss_decrease_vector_gentler_mult() {
        let mut cc = Udt4Cc::new();
        cc.slow_start = false;
        cc.set_period(1000.0);
        cc.set_cwnd(80.0);
        cc.set_snd_curr_seq(50);
        cc.last_dec_seq = 10;
        cc.on_loss(&[40]);
        // 1000 * 1.03 = 1030; CWND 80 * 0.875 = 70
        assert_eq!(cc.sending_period_us(), 1030);
        assert_eq!(cc.cwnd_pkts(), 70);
    }

    #[test]
    fn compute_inc_min_when_no_spare() {
        let inc = Udt4Cc::compute_inc(100, 10_000.0, 1500); // C = 100 pps, B = 100
        assert!((inc - UDT_MIN_INC).abs() < 1e-9);
    }

    #[test]
    fn compute_inc_grows_with_spare_bandwidth() {
        let inc = Udt4Cc::compute_inc(10_000, 10_000.0, 1500); // C=100, B=10000
        assert!(inc > UDT_MIN_INC);
    }

    #[test]
    fn ack_rate_increase_updates_period() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.slow_start = false;
        cc.loss_since_ack = false;
        cc.set_period(1000.0);
        cc.set_bandwidth(50_000);
        cc.set_recv_rate(1_000);
        cc.set_rtt_us(50_000);
        cc.on_ack(10);
        // Period should decrease (send faster) when spare bandwidth exists.
        assert!(cc.sending_period_us() < 1000);
    }

    #[test]
    fn fixed_cc_pluggable() {
        let mut cc = FixedCc::new(50, 8);
        cc.on_ack(1);
        cc.on_loss(&[1]);
        assert_eq!(cc.sending_period_us(), 50);
        assert_eq!(cc.cwnd_pkts(), 8);
    }

    #[test]
    fn timeout_raises_period() {
        let mut cc = Udt4Cc::new();
        cc.slow_start = false;
        cc.set_period(800.0);
        cc.set_cwnd(64.0);
        cc.on_timeout();
        assert_eq!(cc.sending_period_us(), 824); // 800 * 1.03
        assert_eq!(cc.cwnd_pkts(), 56); // 64 * 0.875
    }

    #[test]
    fn timeout_in_slow_start_seeds_period_from_rtt() {
        let mut cc = Udt4Cc::new();
        cc.set_rtt_us(40_000);
        cc.set_cwnd(16.0);
        assert!(cc.slow_start);
        cc.on_timeout();
        assert!(!cc.slow_start);
        // (40000+10000)/16 = 3125, capped at UDT_SS_EXIT_PERIOD_CAP_US = 500
        assert_eq!(cc.sending_period_us(), 500);
    }

    #[test]
    fn early_climb_boost_reduces_coarse_period() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.slow_start = false;
        cc.set_period(700.0);
        cc.set_bandwidth(50_000);
        cc.set_recv_rate(2_000);
        cc.set_rtt_us(6_500);
        cc.on_ack(100);
        // With early-climb boost, 700µs should drop meaningfully in one SYN.
        assert!(
            cc.sending_period_us() < 600,
            "period={}",
            cc.sending_period_us()
        );
    }

    #[test]
    fn ca_without_recv_rate_does_not_pin_cwnd_at_16() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.slow_start = false;
        cc.loss_since_ack = false;
        cc.set_cwnd(64.0);
        cc.set_recv_rate(0);
        cc.set_bandwidth(0);
        cc.set_rtt_us(50_000);
        cc.on_ack(100);
        // Must not collapse back to 16 when feedback is missing.
        assert!(cc.cwnd_pkts() >= 64, "cwnd={}", cc.cwnd_pkts());
    }

    #[test]
    fn ca_soft_shrinks_cwnd_when_recv_rate_dips() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.slow_start = false;
        cc.set_cwnd(200.0);
        cc.set_recv_rate(500); // target << 200 for typical RTT
        cc.set_rtt_us(6_500);
        cc.set_bandwidth(20_000);
        cc.on_ack(1000);
        let w = cc.cwnd_pkts();
        // At most ⅛ reduction per SYN (200 * 0.875 = 175).
        assert!(w >= 175, "cwnd shrank too hard: {w}");
        assert!(w <= 200, "cwnd grew on rate dip: {w}");
    }

    #[test]
    fn sim_ack_nak_no_panic() {
        let mut cc = Udt4Cc::new();
        let t0 = Instant::now();
        cc.set_clock(t0);
        cc.set_rtt_us(40_000);
        cc.set_bandwidth(20_000);
        cc.set_recv_rate(5_000);
        cc.set_mss(1500);
        cc.set_snd_curr_seq(0);
        for i in 0..200u32 {
            cc.advance_clock(Duration::from_millis(11));
            cc.set_snd_curr_seq(i * 10);
            if i % 7 == 0 {
                cc.on_loss(&[i * 10 + 1, i * 10 + 2]);
            } else {
                cc.on_ack(i * 10);
            }
            let _ = cc.cwnd_pkts();
            let _ = cc.sending_period_us();
        }
        assert!(cc.cwnd_pkts() >= 2);
        assert!(cc.sending_period_us() >= 1);
    }
}
