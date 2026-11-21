# `Udt4Cc` / `CongestionControl::on_ack`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
fn on_ack(&mut self, ack_seq: u32)
```

## 2. Semantics

UDT4 native rate/window update on cumulative ACK. Slow start grows CWND;
congestion avoidance updates CWND from arrival rate and may decrease SND via
the UDT increase formula when no loss was reported since the last RC step.

## 3. Preconditions / postconditions

**Preconditions**

- Optional: RTT / recv_rate / bandwidth set via trait hooks.

**Postconditions**

- `cwnd_pkts()` / `sending_period_us()` reflect the update (SYN-gated).

## 4. Errors

None.

## 5. Thread-safety / async cancellation

Not thread-safe.

## 6. Wire effects

None directly; reliability engine reads CWND/SND for send gating.

## 7. Compatibility

C++ `CUDTCC::onACK`.

## 8. Example

See `cc` unit tests and `docs/plan/cc-reference-vectors.md`.

## 9. See also

- [`../../protocol/congestion.md`](../../protocol/congestion.md)
