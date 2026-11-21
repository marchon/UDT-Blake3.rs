# UDT4 CC reference vectors

Deterministic checks implemented in `udt_core::cc` unit tests.

| Vector | Setup | Action | Expect |
| --- | --- | --- | --- |
| init | `Udt4Cc::new()` | — | CWND=16, SND=1µs, slow-start |
| ss-ack | last_ack=0 | `on_ack(5)` | CWND=21, still slow-start |
| loss-1.125 | SS off, SND=1000 | `on_loss([40])` with curr=50 | SND=1125 |
| timeout | SS off, SND=800 | `on_timeout` | SND=900 |
| inc-min | B<=C | `compute_inc` | `UDT_MIN_INC` |
| inc-spare | B>>C | `compute_inc` | > `UDT_MIN_INC` |
| sim | 200 ACK/NAK mix | loop | no panic; CWND≥2 |

Clock is pinned via `set_clock` / `advance_clock` where SYN gating matters.
