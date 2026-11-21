# `UdtSocket::accept`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn accept(&mut self) -> Result<(UdtSocket, SocketAddr), SocketError>
```

## 2. Semantics

Blocks until a client handshake request arrives, completes the UDT4 cookie
exchange on the listening UDP socket, and returns a new `Connected` child
socket (with its own connected UDP endpoint) plus the peer address.

## 3. Preconditions / postconditions

**Preconditions**

- `listen` has been called; socket not Shutdown/Closed.

**Postconditions**

- Child is `Connected` with peer sock id from the request.
- Listener remains listening for further accepts.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `SocketError::Handshake` | Not listening / timeout | `listen` or raise timeout |
| `SocketError::State` | Shutdown/Closed | Stop accepting |

## 5. Thread-safety / async cancellation

- One accept at a time per listener recommended in Phase 2.
- Blocks on UDP recv (read timeout applies).

## 6. Wire effects

Receives HS request + confirm; sends HS response.

## 7. Compatibility

- C++ UDT4: `UDT::accept`

## 8. Example

See `localhost_three_way_handshake` in `udt-socket` tests.

## 9. See also

- [`udt_socket_listen.md`](udt_socket_listen.md)
- [`../../protocol/handshake.md`](../../protocol/handshake.md)
