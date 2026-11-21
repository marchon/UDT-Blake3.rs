# `UdtSocket::send`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn send(&mut self, buf: &[u8]) -> Result<usize, SocketError>
```

## 2. Semantics

Enqueue `buf` into the reliability engine and dispatch data packets (subject to
`set_loss_rate`). Returns `buf.len()` when accepted into the send path.

## 3. Preconditions / postconditions

**Preconditions**

- `ConnectionState::Connected` stream socket.

**Postconditions**

- Bytes may still be unacked; call `flush` to wait for ACKs.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `Transfer` | Not connected | Handshake first |
| `Io` / `Packet` | Wire failures | Retry / abort |

## 5. Thread-safety / async cancellation

Blocking; not shared across threads.

## 6. Wire effects

Sends one or more UDT data packets; may drop some when loss rate &gt; 0.

## 7. Compatibility

Roughly `UDT::send` / stream write.

## 8. Example

```rust
// after connect:
// socket.send(b"hello")?;
// socket.flush(std::time::Duration::from_secs(3))?;
```

## 9. See also

- [`udt_socket_recv.md`](udt_socket_recv.md)
- [`../../protocol/reliability.md`](../../protocol/reliability.md)
