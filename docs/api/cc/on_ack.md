# `CongestionControl::on_ack`

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

Notifies congestion control that cumulative delivery advanced to `ack_seq`. Takes `&mut self`. On the `Udt4Cc` stub, increases `cwnd` by 1 up to a cap of 1024; `ack_seq` is currently unused by the stub. Sync and non-blocking.

## 3. Preconditions / postconditions

**Preconditions**

- Caller should invoke after a valid ACK advances the cumulative sequence.

**Postconditions** (`Udt4Cc` stub)

- `cwnd_pkts()` is `min(previous + 1, 1024)`.
- Sending period is unchanged.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: requires exclusive `&mut self`
- `Send` / `Sync`: depends on the concrete CC type
- Async cancellation safety: N/A (sync)

## 6. Wire effects

Updates CC state only. Does not send packets; the sender may open the window afterward.

## 7. Compatibility

- C++ UDT4: `CCC::onACK`
- `udt-rs` / `tokio-udt`: CC callbacks inside linked UDT4

## 8. Example

```rust
use udt_core::cc::{CongestionControl, Udt4Cc};

let mut cc = Udt4Cc::new();
let before = cc.cwnd_pkts();
cc.on_ack(42);
assert_eq!(cc.cwnd_pkts(), before + 1);
```

## 9. See also

- [`CongestionControl::on_loss`](./on_loss.md)
- [`CongestionControl::cwnd_pkts`](./cwnd_pkts.md)
- [`Udt4Cc::new`](./udt4cc_new.md)
- [API index](../README.md)
