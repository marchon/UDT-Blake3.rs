# Congestion control (Phase 4)

Native UDT4 hybrid CC (`Udt4Cc`), following draft-gg-udt-03 / `CUDTCC`:

| Parameter | Role |
| --- | --- |
| **CWND** | Max in-flight data packets |
| **SND** | Inter-packet send period (microseconds) |

## Events

- `on_ack` — slow start grows CWND; congestion avoidance grows CWND from
  arrival rate (never shrinks on ACK) and always runs the UDT `inc` formula
  (NAK recovery does not freeze rate increases)
- `on_loss` — exit slow start; multiply SND by 1.03; soft-cut CWND by ⅞
- `on_timeout` — same gentler backoff as loss outside slow start
- NAK / RTO rexmit — up to 16 oldest outstanding packets (forward path keeps sending)

## Constants

- `UDT_SYN_US` = 10_000 (rate-control quantum)
- `UDT_BETA` = 1.5e-6
- `UDT_MIN_INC` = 0.1 (faster climb than stock 0.01)
- `UDT_LOSS_PERIOD_MULT` = 1.03 (gentler than stock 1.125 / prior 1.05)
- `UDT_LOSS_CWND_MULT` = 0.875 (⅞ soft window cut on loss / soft ACK shrink)
- `UDT_SS_EXIT_PERIOD_CAP_US` = 500 (cap seeded SND after SS exit without rate)
- `UDT_CWND_GROW_PER_SYN` = 48
- Initial CWND = 16, initial SND = 1µs

SND is clamped to `[1, SYN]` µs. Full ACKs carry RTT / recv_rate / bandwidth;
the sender applies them via `set_rtt_us` / `set_recv_rate` / `set_bandwidth`.
When `recv_rate` is still 0, congestion avoidance **keeps** the current CWND
(and grows by 1 per SYN) instead of pinning to 16. With rate feedback, CWND
grows toward the BDP target by at most +48 packets per SYN and shrinks by at
most ⅛ per SYN. While SND ≥ 200 µs (or ≥ 100 µs), `inc` is boosted ×3 (×2)
so short flows reach line rate sooner.

## Plugging in

`ReliabilityEngine::with_cc(Box::new(FixedCc::new(..)))` for tests.

## Sender pacing

`ReliabilityEngine::enqueue_app_data` advances a virtual send clock by one SND
period per packet. When SND ≥ 50µs (typical WAN after slow start), only
packets whose send time has arrived are emitted (one per call) so `dispatch`
cannot turn a lookahead batch into a UDP microburst. When SND is smaller
(localhost / early slow start), a ~2 ms lookahead burst of at most 32 packets
keeps syscall overhead low. RTO / NAK retransmission sends up to 16 oldest
outstanding packets so deep holes recover without multi-second stalls.
