//! Timer / deadline abstractions (Phase 3). Runtime-agnostic.

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// Logical timer kinds used by the reliability and keep-alive paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerKind {
    /// Retransmission timeout.
    Rto,
    /// Keep-alive probe.
    KeepAlive,
    /// ACK delay flush.
    AckDelay,
}

/// A scheduled deadline relative to an opaque clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadline {
    /// Which timer fired.
    pub kind: TimerKind,
    /// Delay from schedule time.
    pub after: Duration,
}

impl Deadline {
    /// Construct a deadline.
    pub fn new(kind: TimerKind, after: Duration) -> Self {
        Self { kind, after }
    }
}

#[derive(Debug, Clone, Copy, Eq)]
struct HeapEntry {
    at: Instant,
    kind: TimerKind,
    gen: u64,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at && self.kind == other.kind && self.gen == other.gen
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap inverted → earliest Instant pops first.
        other
            .at
            .cmp(&self.at)
            .then_with(|| self.gen.cmp(&other.gen))
    }
}

/// Minimal timer wheel: schedule by absolute [`Instant`], poll due kinds.
#[derive(Debug, Default)]
pub struct TimerWheel {
    heap: BinaryHeap<HeapEntry>,
    gen: u64,
    /// Generations invalidated per kind (cancel).
    live: [u64; 3],
}

impl TimerWheel {
    /// Create an empty wheel.
    pub fn new() -> Self {
        Self::default()
    }

    fn idx(kind: TimerKind) -> usize {
        match kind {
            TimerKind::Rto => 0,
            TimerKind::KeepAlive => 1,
            TimerKind::AckDelay => 2,
        }
    }

    /// Schedule (or replace) `kind` to fire at `at`.
    pub fn schedule(&mut self, kind: TimerKind, at: Instant) {
        self.gen = self.gen.wrapping_add(1);
        let g = self.gen;
        self.live[Self::idx(kind)] = g;
        self.heap.push(HeapEntry {
            at,
            kind,
            gen: g,
        });
    }

    /// Cancel any pending timer of `kind`.
    pub fn cancel(&mut self, kind: TimerKind) {
        self.live[Self::idx(kind)] = 0;
    }

    /// Pop all timers due at or before `now` (latest generation only).
    pub fn poll(&mut self, now: Instant) -> Vec<TimerKind> {
        let mut out = Vec::new();
        while let Some(top) = self.heap.peek().copied() {
            if top.at > now {
                break;
            }
            self.heap.pop();
            if self.live[Self::idx(top.kind)] == top.gen {
                // one-shot: clear live so duplicates of same gen don't re-fire
                self.live[Self::idx(top.kind)] = 0;
                out.push(top.kind);
            }
        }
        out
    }

    /// Next scheduled instant, if any live entry remains.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.heap
            .iter()
            .filter(|e| self.live[Self::idx(e.kind)] == e.gen)
            .map(|e| e.at)
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_in_order() {
        let mut w = TimerWheel::new();
        let t0 = Instant::now();
        w.schedule(TimerKind::AckDelay, t0 + Duration::from_millis(5));
        w.schedule(TimerKind::Rto, t0 + Duration::from_millis(10));
        assert!(w.poll(t0).is_empty());
        let due = w.poll(t0 + Duration::from_millis(6));
        assert_eq!(due, vec![TimerKind::AckDelay]);
        let due = w.poll(t0 + Duration::from_millis(11));
        assert_eq!(due, vec![TimerKind::Rto]);
    }

    #[test]
    fn cancel_prevents_fire() {
        let mut w = TimerWheel::new();
        let t0 = Instant::now();
        w.schedule(TimerKind::Rto, t0);
        w.cancel(TimerKind::Rto);
        assert!(w.poll(t0 + Duration::from_millis(1)).is_empty());
    }
}
