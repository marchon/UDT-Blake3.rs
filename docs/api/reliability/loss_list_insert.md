# `LossList::insert`

- **Crate:** `udt-core`
- **Module path:** `udt_core::reliability`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn insert(&mut self, seq: SeqNo)
```

## 2. Semantics

Records a missing sequence number. Idempotent: duplicates are ignored. After insert the internal list is sorted ascending. Takes `&mut self`; sync and non-blocking (may reallocate the inner `Vec`).

## 3. Preconditions / postconditions

**Preconditions**

- `self` is a valid `LossList` (e.g. from `LossList::new`).

**Postconditions**

- `seq` appears exactly once in `as_slice()`.
- `as_slice()` is sorted ascending.
- `len()` equals the number of unique sequences recorded.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: no shared mutation; requires exclusive `&mut self`
- `Send` / `Sync`: `LossList` is `Send` + `Sync`
- Async cancellation safety: N/A (sync)

## 6. Wire effects

May later drive NAK generation (Phase 3). This call itself does not send packets.

## 7. Compatibility

- C++ UDT4: loss-list insert analogues in the receive / NAK path
- `udt-rs` / `tokio-udt`: loss tracking inside linked UDT4

## 8. Example

```rust
use udt_core::reliability::LossList;

let mut list = LossList::new();
list.insert(10);
list.insert(5);
list.insert(10);
assert_eq!(list.as_slice(), &[5, 10]);
```

## 9. See also

- [`LossList::new`](./loss_list_new.md)
- [`LossList::remove`](./loss_list_remove.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
