# `HandshakePacket::decode`

- **Crate:** `udt-core`
- **Module path:** `udt_core::packet`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl HandshakePacket {
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError>
}
```

## 2. Semantics

Parses a full UDT4 handshake control packet (TYPE 0x0) from `buf`. Validates
the control flag and type bits, then reads header and body fields as big-endian
integers (connection type as `i32` to allow −1/−2 responses).

## 3. Preconditions / postconditions

**Preconditions**

- `buf.len() >= HANDSHAKE_PACKET_LEN` (64).
- First header word must be a control packet with type 0.

**Postconditions**

- On success, returns an owned `HandshakePacket` with all fields populated.
- Does not mutate `buf` or global state.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `PacketError::TooShort` | `buf` shorter than 64 bytes | Wait for a full datagram |
| `PacketError::UnsupportedControl` | Not a handshake control type | Dispatch via `peek_type` |

## 5. Thread-safety / async cancellation

- Thread-safe: yes (pure parse)
- `Send` / `Sync`: returned value follows its fields
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None (read-only parse of a local buffer).

## 7. Compatibility

- C++ UDT4: `CHandShake::deserialize` + control header
- Must match Wireshark UDT handshake field offsets for golden pcaps

## 8. Example

```rust
use udt_core::packet::{HandshakePacket, HANDSHAKE_PACKET_LEN};

let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
// ... fill from UDP ...
let hs = HandshakePacket::decode(&buf)?;
assert_eq!(hs.version, 4);
```

## 9. See also

- [`HandshakePacket::encode`](./handshake_packet_encode.md)
- [`peek_type`](./peek_type.md)
- [Protocol notes](../../protocol/packets.md)
