# `CongestionControl::cwnd_pkts`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
fn cwnd_pkts(&self) -> u32
```

## 2. Semantics

Returns the congestion window size in packets. Borrow of `&self` only; sync and non-blocking. On the `Udt4Cc` stub, the initial window is `16`; `on_ack` grows it (cap 1024) and `on_loss` halves it (floor 2).

## 3. Preconditions / postconditions

**Preconditions**

- `self` is a live CC instance (typically after `Udt4Cc::new`).

**Postconditions**

- Returned value reflects the last `on_ack` / `on_loss` updates.
- Does not mutate CC state.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: read-only on `&self`; concurrent mutation needs exclusive access elsewhere
- `Send` / `Sync`: depends on the concrete CC type (`Udt4Cc` is `Send` + `Sync`)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None directly. The sender should not have more than `cwnd_pkts` unacknowledged data packets in flight.

## 7. Compatibility

- C++ UDT4: `CCC::getCWndSize` / internal congestion window
- `udt-rs` / `tokio-udt`: window owned by linked C++ CC

## 8. Example

```rust
use udt_core::cc::{CongestionControl, Udt4Cc};

let mut cc = Udt4Cc::new();
assert_eq!(cc.cwnd_pkts(), 16);
cc.on_loss(&[1]);
assert!(cc.cwnd_pkts() <= 8);
```

## 9. See also

- [`CongestionControl::sending_period_us`](./sending_period_us.md)
- [`CongestionControl::on_ack`](./on_ack.md)
- [`CongestionControl::on_loss`](./on_loss.md)
- [API index](../README.md)
