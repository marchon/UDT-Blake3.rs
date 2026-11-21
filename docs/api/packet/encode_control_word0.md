# `encode_control_word0`

- **Crate:** `udt-core`
- **Module path:** `udt_core::packet`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn encode_control_word0(ctrl_type: u16) -> u32
```

## 2. Semantics

Builds the first 32-bit header word for a control packet: sets bit 31 and places `ctrl_type & 0x7fff` in bits 16..30. Lower 16 bits (subtype / additional info) are left zero. Pure; sync; non-blocking.

## 3. Preconditions / postconditions

**Preconditions**

- `ctrl_type` should be a UDT4 control type (0–7 for known types); higher bits beyond 15 are masked off.

**Postconditions**

- Result has bit 31 set (`is_control` returns true).
- No heap allocation or global mutation.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | Validate `ctrl_type` at a higher layer if needed |

## 5. Thread-safety / async cancellation

- Thread-safe: yes (pure)
- `Send` / `Sync`: N/A (free function)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

Produces host `u32` intended to be written big-endian as UDP payload bytes 0..4. Does not send by itself.

## 7. Compatibility

- C++ UDT4: control bit set + type in bits 16..30 of header word 0
- `udt-rs` / `tokio-udt`: same layout via C++ `CPacket`

## 8. Example

```rust
use udt_core::packet::{encode_control_word0, is_control};

let w0 = encode_control_word0(2); // ACK
assert!(is_control(w0));
assert_eq!((w0 >> 16) & 0x7fff, 2);
```

## 9. See also

- [`is_control`](./is_control.md)
- [`peek_type`](./peek_type.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
