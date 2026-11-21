# Congestion control (Phase 4)

Native UDT4 hybrid CC (`Udt4Cc`), following draft-gg-udt-03 / `CUDTCC`:

| Parameter | Role |
| --- | --- |
| **CWND** | Max in-flight data packets |
| **SND** | Inter-packet send period (microseconds) |

## Events

- `on_ack` — slow start grows CWND; congestion avoidance updates CWND from
  arrival rate and increases rate via the UDT `inc` formula when no recent loss
- `on_loss` — exit slow start; multiply SND by 1.125 with UDT decrease bookkeeping
- `on_timeout` — exit slow start; multiply SND by 1.125

## Constants

- `UDT_SYN_US` = 10_000 (rate-control quantum)
- `UDT_BETA` = 1.5e-6
- `UDT_MIN_INC` = 0.01
- Initial CWND = 16, initial SND = 1µs

SND is clamped to `[1, SYN]` µs. Full ACKs carry RTT / recv_rate / bandwidth;
the sender applies them via `set_rtt_us` / `set_recv_rate` / `set_bandwidth`.
When `recv_rate` is still 0, congestion avoidance **keeps** the current CWND
(and grows by 1 per SYN) instead of pinning to 16.

## Plugging in

`ReliabilityEngine::with_cc(Box::new(FixedCc::new(..)))` for tests.

## Sender pacing

`ReliabilityEngine::enqueue_app_data` advances a virtual send clock by one SND
period per packet. When SND ≥ 50µs (typical WAN after slow start), only
packets whose send time has arrived are emitted (one per call) so `dispatch`
cannot turn a lookahead batch into a UDP microburst. When SND is smaller
(localhost / early slow start), a ~2 ms lookahead burst of at most 32 packets
keeps syscall overhead low. RTO retransmission sends the oldest outstanding
packet only.
