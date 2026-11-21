# `LossList::remove`

- **Crate:** `udt-core`
- **Module path:** `udt_core::reliability`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn remove(&mut self, seq: SeqNo)
```

## 2. Semantics

Drops `seq` from the loss list after ACK or successful retransmit. No-op if `seq` was not present. Takes `&mut self`; sync and non-blocking.

## 3. Preconditions / postconditions

**Preconditions**

- `self` is a valid `LossList`.

**Postconditions**

- `seq` is absent from `as_slice()`.
- Relative order of remaining sequences is preserved (sorted).

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: no shared mutation; requires exclusive `&mut self`
- `Send` / `Sync`: `LossList` is `Send` + `Sync`
- Async cancellation safety: N/A (sync)

## 6. Wire effects

Clears pending NAK / retransmit interest for `seq`. Does not send packets itself.

## 7. Compatibility

- C++ UDT4: loss-list remove on ACK / repair
- `udt-rs` / `tokio-udt`: loss tracking inside linked UDT4

## 8. Example

```rust
use udt_core::reliability::LossList;

let mut list = LossList::new();
list.insert(7);
list.remove(7);
assert!(list.is_empty());
```

## 9. See also

- [`LossList::new`](./loss_list_new.md)
- [`LossList::insert`](./loss_list_insert.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
