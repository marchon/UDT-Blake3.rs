# `HandshakePacket::encode`

- **Crate:** `udt-core`
- **Module path:** `udt_core::packet`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl HandshakePacket {
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError>
}
```

## 2. Semantics

Serializes a full UDT4 handshake control packet (TYPE 0x0) into `buf` as
big-endian bytes. Writes exactly [`HANDSHAKE_PACKET_LEN`](../../protocol/packets.md)
(64) bytes: 16-byte common header plus 48-byte handshake body.

## 3. Preconditions / postconditions

**Preconditions**

- `buf.len() >= HANDSHAKE_PACKET_LEN` (64).

**Postconditions**

- On success, `buf[0..64]` holds the encoded packet and the returned length is 64.
- No global state is mutated.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `PacketError::TooShort` | `buf` shorter than 64 bytes | Grow the buffer |

## 5. Thread-safety / async cancellation

- Thread-safe: yes (pure encode into caller buffer)
- `Send` / `Sync`: follows `HandshakePacket` fields
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None locally; caller is responsible for sending the encoded UDP datagram.

## 7. Compatibility

- C++ UDT4: `CHandShake::serialize` + control header
- Must match Wireshark UDT handshake field offsets for golden pcaps

## 8. Example

```rust
use udt_core::packet::{HandshakePacket, HANDSHAKE_PACKET_LEN};

let hs = HandshakePacket {
    timestamp: 0,
    dest_sock_id: 0,
    version: 4,
    sock_type: 0,
    init_seq: 1,
    max_pkt_size: 1500,
    max_flow_window: 8192,
    conn_type: 1,
    sock_id: 42,
    syn_cookie: 0,
    peer_ip: [0; 16],
};
let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
assert_eq!(hs.encode(&mut buf).unwrap(), HANDSHAKE_PACKET_LEN);
```

## 9. See also

- [`HandshakePacket::decode`](./handshake_packet_decode.md)
- [`peek_type`](./peek_type.md)
- [Protocol notes](../../protocol/packets.md)
