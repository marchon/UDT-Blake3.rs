# `is_control`

- **Crate:** `udt-core`
- **Module path:** `udt_core::packet`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn is_control(header_word0: u32) -> bool
```

## 2. Semantics

Returns `true` when bit 31 of the first UDT header word is set (control packet). Pure function: takes the already-decoded big-endian `u32`, borrows nothing, does not block.

## 3. Preconditions / postconditions

**Preconditions**

- `header_word0` is the first 32-bit word of a UDT header in host `u32` form (after big-endian decode).

**Postconditions**

- Return value is `true` iff `(header_word0 & 0x8000_0000) != 0`.
- No state is mutated.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: yes (pure, no shared state)
- `Send` / `Sync`: N/A (free function)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None (pure decode helper).

## 7. Compatibility

- C++ UDT4: control flag on `CPacket` header word 0
- `udt-rs` / `tokio-udt`: same wire bit via linked UDT4

## 8. Example

```rust
use udt_core::packet::is_control;

assert!(is_control(0x8000_0000));
assert!(!is_control(0x0000_0001));
```

## 9. See also

- [`peek_type`](./peek_type.md)
- [`encode_control_word0`](./encode_control_word0.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
