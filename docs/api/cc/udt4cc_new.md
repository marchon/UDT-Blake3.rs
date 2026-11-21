# `Udt4Cc::new`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn new() -> Self
```

## 2. Semantics

Constructs the default UDT4-style congestion-control stub. Equivalent to `Udt4Cc::default()`. Initial sending period is `1` µs and initial congestion window is `16` packets. Sync; non-blocking; returns owned `Udt4Cc`.

## 3. Preconditions / postconditions

**Preconditions**

- None.

**Postconditions**

- `sending_period_us() == 1` and `cwnd_pkts() == 16`.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: construction is safe; share behind sync if mutated
- `Send` / `Sync`: `Udt4Cc` is `Send` + `Sync`
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None until attached to a sending path that reads period / cwnd.

## 7. Compatibility

- C++ UDT4: `CCC` / `CUDTCC` default construction
- `udt-rs` / `tokio-udt`: CC lives inside linked UDT4

## 8. Example

```rust
use udt_core::cc::{CongestionControl, Udt4Cc};

let cc = Udt4Cc::new();
assert_eq!(cc.cwnd_pkts(), 16);
assert_eq!(cc.sending_period_us(), 1);
```

## 9. See also

- [`CongestionControl::on_ack`](./on_ack.md)
- [`CongestionControl::on_loss`](./on_loss.md)
- [`CongestionControl::cwnd_pkts`](./cwnd_pkts.md)
- [`CongestionControl::sending_period_us`](./sending_period_us.md)
- [API index](../README.md)
