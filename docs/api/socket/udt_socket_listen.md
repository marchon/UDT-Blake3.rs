# `UdtSocket::listen`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn listen(&mut self, backlog: i32) -> Result<(), SocketError>
```

## 2. Semantics

Marks a bound socket as a listening server. `backlog` is accepted for API
compatibility and ignored in Phase 2 (no accept queue depth limit yet).

## 3. Preconditions / postconditions

**Preconditions**

- Socket must be bound (`bind` first).
- State must be `ConnectionState::Init`.

**Postconditions**

- `is_listening()` is true; state remains `Init` (listener is not a data connection).

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `SocketError::Handshake` | Not bound | Call `bind` first |
| `SocketError::State` | Not in Init | Recreate socket |

## 5. Thread-safety / async cancellation

- Not thread-safe for concurrent mutation; use one owner thread.
- Async: see `AsyncUdtSocket::listen` (thin wrap).

## 6. Wire effects

None until `accept` receives a handshake.

## 7. Compatibility

- C++ UDT4: `UDT::listen`
- `udt-rs`: `listen`

## 8. Example

```rust
use std::net::SocketAddr;
use udt_socket::UdtSocket;

let mut s = UdtSocket::new();
s.bind("127.0.0.1:0".parse::<SocketAddr>().unwrap()).unwrap();
s.listen(8).unwrap();
assert!(s.is_listening());
```

## 9. See also

- [`udt_socket_accept.md`](udt_socket_accept.md)
- [`../../protocol/handshake.md`](../../protocol/handshake.md)
