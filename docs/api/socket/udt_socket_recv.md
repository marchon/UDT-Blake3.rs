# `UdtSocket::recv`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, SocketError>
```

## 2. Semantics

Pumps the UDP/reliability loop until delivered bytes are available or the read
timeout budget expires (`Ok(0)`).

## 3. Preconditions / postconditions

**Preconditions**

- Connected stream socket.

**Postconditions**

- Returns in-order stream bytes; may be partial.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `Transfer` | Not connected | Handshake first |

## 5. Thread-safety / async cancellation

Blocking pump loop.

## 6. Wire effects

May send ACK/NAK while pumping.

## 7. Compatibility

Roughly `UDT::recv`.

## 8. Example

See `small_stream_round_trip` test.

## 9. See also

- [`udt_socket_send.md`](udt_socket_send.md)
