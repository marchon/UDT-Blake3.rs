# `Connection::begin_handshake`

- **Crate:** `udt-core`
- **Module path:** `udt_core::state`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn begin_handshake(&mut self) -> Result<(), StateError>
```

## 2. Semantics

Advances a [`Connection`](../../../crates/udt-core/src/state.rs) from
`ConnectionState::Init` to `ConnectionState::Handshaking` via
`ConnectionState::transition`. Does not send handshake packets; callers own
wire I/O.

## 3. Preconditions / postconditions

**Preconditions**

- `self.state()` is `ConnectionState::Init`.

**Postconditions**

- On `Ok(())`, `self.state()` is `ConnectionState::Handshaking`.
- Socket IDs (`sock_id`, `peer_sock_id`) are unchanged.
- On error, state is unchanged.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `StateError::Illegal` | Current state is not `Init` | Fix call order; do not start handshake twice |

## 5. Thread-safety / async cancellation

- Thread-safe: no; requires exclusive `&mut self`
- `Send` / `Sync`: `Connection` is `Send + Sync` (plain data)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None. Handshake packet exchange is the caller's responsibility.

## 7. Compatibility

- C++ UDT4: analogous to entering connecting/listening handshake in `CUDT`
- `udt-rs`: N/A (stub)
- `tokio-udt`: N/A (stub)

## 8. Example

```rust
use udt_core::{Connection, ConnectionState};

let mut c = Connection::new(1, 0);
c.begin_handshake().unwrap();
assert_eq!(c.state(), ConnectionState::Handshaking);
```

## 9. See also

- [`ConnectionState::transition`](transition.md)
- Protocol notes: [`../../protocol/state-diagram.md`](../../protocol/state-diagram.md)
