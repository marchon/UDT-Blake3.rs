# `CongestionControl::on_loss`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
fn on_loss(&mut self, loss_seqs: &[u32])
```

## 2. Semantics

Notifies congestion control of detected loss (NAK or timeout). Borrows `loss_seqs`; takes `&mut self`. On the `Udt4Cc` stub, halves `cwnd` (floor 2) and doubles `sending_period_us` (floor 1). The stub ignores the contents of `loss_seqs` beyond the fact that loss occurred. Sync and non-blocking.

## 3. Preconditions / postconditions

**Preconditions**

- `loss_seqs` may be empty or non-empty; stub does not require specific contents.

**Postconditions** (`Udt4Cc` stub)

- `cwnd_pkts() == max(previous / 2, 2)`.
- `sending_period_us() == max(previous * 2, 1)` (saturating multiply).

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: requires exclusive `&mut self`
- `Send` / `Sync`: depends on the concrete CC type
- Async cancellation safety: N/A (sync)

## 6. Wire effects

Updates CC state only (window / period). Does not send packets; the sender should slow down afterward.

## 7. Compatibility

- C++ UDT4: `CCC::onLoss`
- `udt-rs` / `tokio-udt`: CC callbacks inside linked UDT4

## 8. Example

```rust
use udt_core::cc::{CongestionControl, Udt4Cc};

let mut cc = Udt4Cc::new();
let before = cc.cwnd_pkts();
cc.on_loss(&[1, 2]);
assert!(cc.cwnd_pkts() <= before / 2 || cc.cwnd_pkts() == 2);
```

## 9. See also

- [`CongestionControl::on_ack`](./on_ack.md)
- [`CongestionControl::sending_period_us`](./sending_period_us.md)
- [`CongestionControl::cwnd_pkts`](./cwnd_pkts.md)
- [API index](../README.md)
