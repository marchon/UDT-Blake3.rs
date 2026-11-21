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

SND is clamped to `[1, SYN]` µs until receiver bandwidth/rate feedback is fully
wired on the ACK path (Phase 6+).

## Plugging in

`ReliabilityEngine::with_cc(Box::new(FixedCc::new(..)))` for tests.
