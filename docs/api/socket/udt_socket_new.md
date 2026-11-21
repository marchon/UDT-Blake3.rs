# `UdtSocket::new`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn new() -> Self
```

## 2. Semantics

Creates an unbound synchronous UDT socket stub in `ConnectionState::Init` with no local address. Equivalent to `UdtSocket::default()`. Sync; non-blocking; returns owned `UdtSocket`.

## 3. Preconditions / postconditions

**Preconditions**

- None.

**Postconditions**

- `state() == ConnectionState::Init`.
- `local_addr()` is `None`.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| — | Never fails | — |

## 5. Thread-safety / async cancellation

- Thread-safe: construction is safe; the socket is not interior-mutable — share behind sync if needed
- `Send` / `Sync`: yes (`ConnectionState` + `Option<SocketAddr>`); prefer exclusive `&mut` for methods
- Async cancellation safety: N/A (sync). See `async_socket::AsyncUdtSocket` for Tokio stubs.

## 6. Wire effects

None.

## 7. Compatibility

- C++ UDT4: `UDT::socket`
- `udt-rs`: `UdtSocket::new`
- `tokio-udt`: async constructors wrapping UDT4

## 8. Example

```rust
use udt_core::ConnectionState;
use udt_socket::UdtSocket;

let s = UdtSocket::new();
assert_eq!(s.state(), ConnectionState::Init);
```

## 9. See also

- [`UdtSocket::bind`](./udt_socket_bind.md)
- [`UdtSocket::connect`](./udt_socket_connect.md)
- [API index](../README.md)
- Usage: [`../../usage/developer.md`](../../usage/developer.md)
