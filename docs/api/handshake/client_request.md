# `client_request`

- **Crate:** `udt-core`
- **Module path:** `udt_core::handshake`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn client_request(
    sock_id: u32,
    init_seq: u32,
    sock_type: u32,
    peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket
```

## 2. Semantics

Build leg 1 of the UDT4 cookie handshake: `conn_type = 1`, `cookie = 0`,
`dest_sock_id = 0`. No I/O.

## 3. Preconditions / postconditions

**Preconditions**

- Field values supplied by the caller are trusted.

**Postconditions**

- Returned packet encodes with `HandshakePacket::encode`.

## 4. Errors

None (returns `HandshakePacket` by value).

## 5. Thread-safety / async cancellation

Pure / `Send` + `Sync` data.

## 6. Wire effects

None until the socket layer sends the encoded bytes.

## 7. Compatibility

Aligned with UDT4 initial connection request (`m_iReqType = 1`).

## 8. Example

See `udt_core::handshake` unit test `udt4_cookie_shapes`.

## 9. See also

- [`../../protocol/handshake.md`](../../protocol/handshake.md)
