# `server_final_response`

- **Crate:** `udt-core`
- **Module path:** `udt_core::handshake`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn server_final_response(
    confirm: &HandshakePacket,
    server_sock_id: u32,
    server_flow_window: u32,
    server_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket
```

## 2. Semantics

Build leg 4 of the UDT4 cookie handshake: `conn_type = -1`, assign
`server_sock_id`, echo the client's ISN (security check), advertise the server
flow window.

## 3. Preconditions / postconditions

**Preconditions**

- `confirm` is a cookie confirm (`is_cookie_confirm`).

**Postconditions**

- `is_server_response` is true; `init_seq == confirm.init_seq`.

## 4. Errors

None (pure constructor).

## 5. Thread-safety / async cancellation

Pure; `Send` + `Sync` value.

## 6. Wire effects

None until encoded and sent.

## 7. Compatibility

Matches C++ UDT4 final response after `newConnection` / `connect(peer, hs)`.

## 8. Example

See `udt_core::handshake` unit test `udt4_cookie_shapes`.

## 9. See also

- [`server_cookie_challenge.md`](server_cookie_challenge.md)
- [`../../protocol/handshake.md`](../../protocol/handshake.md)
