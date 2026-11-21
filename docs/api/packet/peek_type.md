# `peek_type`

- **Crate:** `udt-core`
- **Module path:** `udt_core::packet`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn peek_type(buf: &[u8]) -> Result<PacketType, PacketError>
```

## 2. Semantics

Classifies a raw UDP payload buffer as data or a known control type by reading the first header word. Borrows `buf`; does not consume a socket or allocate. Sync and non-blocking. Unknown control type IDs map to `PacketType::Other(id)`.

## 3. Preconditions / postconditions

**Preconditions**

- `buf` should be at least `MIN_HEADER_LEN` (16) bytes for a successful decode.

**Postconditions**

- On `Ok`, the variant matches UDT4 control type IDs (0 handshake, 1 keep-alive, 2 ACK, 3 NAK, 5 shutdown, 6 ACK2, 7 msg-drop) or `Data` when bit 31 is clear.
- `buf` is unchanged.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `PacketError::TooShort { need, have }` | `buf.len() < 16` | Wait for more bytes / drop short datagram |

## 5. Thread-safety / async cancellation

- Thread-safe: yes (read-only on caller-owned slice)
- `Send` / `Sync`: N/A (free function)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None (read-only parse of a local buffer).

## 7. Compatibility

- C++ UDT4: control type field in `CPacket`
- Must match Wireshark UDT classifications for golden pcaps
- `udt-rs` / `tokio-udt`: same type IDs on the wire

## 8. Example

```rust
use udt_core::packet::{encode_control_word0, peek_type, PacketType, MIN_HEADER_LEN};

let mut buf = [0u8; MIN_HEADER_LEN];
let w0 = encode_control_word0(0);
buf[0..4].copy_from_slice(&w0.to_be_bytes());
assert_eq!(peek_type(&buf).unwrap(), PacketType::Handshake);
```

## 9. See also

- [`is_control`](./is_control.md)
- [`encode_control_word0`](./encode_control_word0.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/packets.md`](../../protocol/packets.md)
