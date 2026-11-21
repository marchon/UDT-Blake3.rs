# `LossList::new`

- **Crate:** `udt-core`
- **Module path:** `udt_core::reliability`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn new() -> Self
```

## 2. Semantics

Creates an empty loss list. Equivalent to `LossList::default()`. Does not allocate beyond an empty `Vec`. Ownership: returns a new owned `LossList`. Sync; does not block.

## 3. Preconditions / postconditions

**Preconditions**

- None.

**Postconditions**

- `len() == 0` and `is_empty()` is true.
- `as_slice()` is empty.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: construction is safe; share the value only behind synchronization if mutated
- `Send` / `Sync`: `LossList` is `Send` + `Sync` (contains only `Vec<u32>`)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None. Callers later use the list to decide NAK / retransmit generation.

## 7. Compatibility

- C++ UDT4: empty loss-list construction inside `CUDT` receive path
- `udt-rs`: no direct public analogue (loss tracking is inside C++)
- `tokio-udt`: same (FFI / internal)

## 8. Example

```rust
use udt_core::reliability::LossList;

let list = LossList::new();
assert!(list.is_empty());
```

## 9. See also

- [`LossList::insert`](./loss_list_insert.md)
- [`LossList::remove`](./loss_list_remove.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
