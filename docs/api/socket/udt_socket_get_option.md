# `UdtSocket::get_option`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn get_option(&self, kind: UdtOptionKind) -> UdtOption
```

## 2. Semantics

Read one typed socket option (`getsockopt` analogue) from the stored
`SocketOptionState`. Always succeeds; returns the current value for `kind`.

## 3. Preconditions / postconditions

**Preconditions**

- None (valid on any socket state).

**Postconditions**

- Returned `UdtOption` variant matches `kind` (`opt.kind() == kind`).

## 4. Errors

None (infallible).

## 5. Thread-safety / async cancellation

Read-only borrow of option state.

## 6. Wire effects

None.

## 7. Compatibility

Maps to UDT4 `getsockopt` for the kinds listed in [`options.md`](../../usage/options.md).

## 8. Example

```rust
use udt_socket::{UdtOption, UdtOptionKind, UdtSocket};

let s = UdtSocket::new();
assert!(matches!(
    s.get_option(UdtOptionKind::SndBuf),
    UdtOption::SndBuf(_)
));
```

## 9. See also

- [`udt_socket_set_option.md`](udt_socket_set_option.md)
- [`../../usage/options.md`](../../usage/options.md)
