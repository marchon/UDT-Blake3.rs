# `UdtSocket::flush`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn flush(&mut self, timeout: Duration) -> Result<(), SocketError>
```

## 2. Semantics

Pump the reliability engine until the send buffer is empty (all enqueued data
ACKed) or `timeout` elapses. A zero timeout uses `SndTimeo` (default 30s).

## 3. Preconditions / postconditions

**Preconditions**

- Socket is `Connected` with a reliability session.

**Postconditions**

- On `Ok`, `send_buffer_empty` is true.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `Transfer("flush timed out…")` | Deadline hit with unacked data | Retry, or accept warning against C++ peers |
| `Transfer("no reliability…")` | Not connected | Connect/accept first |

## 5. Thread-safety / async cancellation

Blocking; not shared across threads.

## 6. Wire effects

May retransmit data and emit ACK/NAK handling via `pump_once`.

## 7. Compatibility

Roughly `UDT::send` completion wait; C++ ACK layouts may leave PureRust flush
waiting even when the peer has the bytes (see interop matrix known gaps).

## 8. Example

```rust
// socket.send(buf)?;
// socket.flush(std::time::Duration::from_secs(5))?;
```

## 9. See also

- [`udt_socket_send.md`](udt_socket_send.md)
- [`../../plan/interop-matrix.md`](../../plan/interop-matrix.md)
