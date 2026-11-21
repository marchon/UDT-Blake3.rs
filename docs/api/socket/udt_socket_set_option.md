# `UdtSocket::set_option`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn set_option(&mut self, opt: UdtOption) -> Result<(), SocketError>
```

## 2. Semantics

Apply one typed socket option (`setsockopt` analogue). Updates
`SocketOptionState` and, when relevant, live UDP timeouts or the reliability
engine (`Mss`, `MaxFlowWindow`, `RcvTimeo`).

## 3. Preconditions / postconditions

**Preconditions**

- `Rendezvous` only while `ConnectionState::Init`.

**Postconditions**

- `get_option` / `options()` reflect the new value.
- `RcvTimeo` updates the bound UDP read timeout when already bound.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `Transfer` | Rendezvous set after Init | Set before connect/listen |

## 5. Thread-safety / async cancellation

Blocking; socket is not shared across threads.

## 6. Wire effects

None immediately. `Mss` / `MaxFlowWindow` change later data-packet sizing and CWND caps.

## 7. Compatibility

Maps to UDT4 `setsockopt` for `UDT_SNDBUF`, `UDT_RCVBUF`, `UDT_RENDEZVOUS`,
`UDT_MSS`, `UDT_SNDTIMEO`, `UDT_RCVTIMEO`, `UDT_FC`.

## 8. Example

```rust
use std::time::Duration;
use udt_socket::{UdtOption, UdtSocket};

let mut s = UdtSocket::new();
s.set_option(UdtOption::RcvTimeo(Some(Duration::from_millis(20)))).unwrap();
s.set_option(UdtOption::SndBuf(512 * 1024)).unwrap();
```

## 9. See also

- [`udt_socket_get_option.md`](udt_socket_get_option.md)
- [`../../usage/options.md`](../../usage/options.md)
