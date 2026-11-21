# `UdtSocket::bind`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError>
```

## 2. Semantics

Binds an OS UDP socket to `addr`. Port `0` requests an ephemeral port.

## 3. Preconditions / postconditions

**Preconditions**

- Socket not already bound.

**Postconditions**

- `local_addr()` reflects the OS-assigned address.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `SocketError::Io` | OS bind failure | Pick another port |
| `SocketError::Handshake` | Already bound | Use existing local addr |

## 5. Thread-safety / async cancellation

- Sync; not shared across threads without external sync.

## 6. Wire effects

None (no UDT packets).

## 7. Compatibility

- C++ UDT4: `UDT::bind`

## 8. Example

```rust
use std::net::SocketAddr;
use udt_socket::UdtSocket;
let mut s = UdtSocket::new();
s.bind("127.0.0.1:0".parse::<SocketAddr>().unwrap()).unwrap();
assert!(s.local_addr().unwrap().port() != 0);
```

## 9. See also

- [`udt_socket_listen.md`](udt_socket_listen.md)
