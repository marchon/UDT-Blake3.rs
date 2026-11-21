# `UdtSocket::connect`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn connect(&mut self, peer: SocketAddr) -> Result<(), SocketError>
```

## 2. Semantics

Runs the UDT4 cookie handshake as a client. Auto-binds an ephemeral UDP port
when unbound. On success the socket is `Connected` and the UDP socket is
`connect()`ed to the peer.

## 3. Preconditions / postconditions

**Preconditions**

- State `Init`; not a listening socket.

**Postconditions**

- State `Connected`; `peer_sock_id()` and `peer_addr()` set.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `SocketError::Handshake` | Timeout / unexpected peer / listening socket | Retry / check server |
| `SocketError::Io` | UDP failures | Check network |
| `SocketError::Packet` | Malformed HS | Interop bug |

## 5. Thread-safety / async cancellation

- Blocks up to ~3s waiting for the server response.
- Not cancellation-safe mid-handshake if interrupted externally.

## 6. Wire effects

Sends HS request + HS confirm; receives HS response.

## 7. Compatibility

- C++ UDT4: `UDT::connect` (simplified cookie path)
- Wire types use UDT4 stream/dgram constants

## 8. Example

See unit test `localhost_three_way_handshake` in `udt-socket`.

## 9. See also

- [`../../protocol/handshake.md`](../../protocol/handshake.md)
- [`udt_socket_accept.md`](udt_socket_accept.md)
