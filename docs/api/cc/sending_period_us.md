# `CongestionControl::sending_period_us`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
fn sending_period_us(&self) -> u64
```

## 2. Semantics

Returns the current inter-packet sending period in microseconds (UDT rate interval). Borrow of `&self` only; sync and non-blocking. On the `Udt4Cc` stub, the default period is `1` µs and doubles on each `on_loss` (saturating).

## 3. Preconditions / postconditions

**Preconditions**

- `self` is a live CC instance (typically after `Udt4Cc::new`).

**Postconditions**

- Returned value is ≥ 1 for the default stub after any `on_loss` path; initial stub value is `1`.
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

None directly. The sender pacing loop should sleep / schedule using this period before emitting the next data packet.

## 7. Compatibility

- C++ UDT4: `CCC::getSndPeriod` / internal `m_dPktSndPeriod`
- `udt-rs` / `tokio-udt`: period owned by linked C++ CC

## 8. Example

```rust
use udt_core::cc::{CongestionControl, Udt4Cc};

let cc = Udt4Cc::new();
assert_eq!(cc.sending_period_us(), 1);
```

## 9. See also

- [`CongestionControl::cwnd_pkts`](./cwnd_pkts.md)
- [`CongestionControl::on_loss`](./on_loss.md)
- [`Udt4Cc::new`](./udt4cc_new.md)
- [API index](../README.md)
