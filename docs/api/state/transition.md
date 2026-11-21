# `ConnectionState::transition`

- **Crate:** `udt-core`
- **Module path:** `udt_core::state`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn transition(self, to: ConnectionState) -> Result<ConnectionState, StateError>
```

## 2. Semantics

Moves `self` along the allowed lifecycle graph by value (consumes the old state). Sync and non-blocking. Allowed edges: Init→Handshaking, Handshaking→Connected|Closed, Connected→Shutdown|Closed, Shutdown→Closed, Init→Closed.

## 3. Preconditions / postconditions

**Preconditions**

- `(self, to)` must be one of the allowed edges above.

**Postconditions**

- On `Ok(to)`, the returned value equals `to`.
- On `Err`, the old state is consumed into the error’s `from` field; caller must keep a separate copy if retry is needed.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `StateError::Illegal { from, to }` | Edge not in the allowed graph | Do not advance; log / close / retry legal path |

## 5. Thread-safety / async cancellation

- Thread-safe: enum is `Copy`; safe to share. Transition itself takes ownership of one value.
- `Send` / `Sync`: yes (`ConnectionState` is `Copy` + plain enum)
- Async cancellation safety: N/A (sync)

## 6. Wire effects

None directly; callers perform handshake / shutdown I/O around transitions.

## 7. Compatibility

- C++ UDT4: high-level `CUDT` status transitions
- `udt-rs` / `tokio-udt`: connection lifecycle inside linked UDT4

## 8. Example

```rust
use udt_core::ConnectionState;

let s = ConnectionState::Init;
let s = s.transition(ConnectionState::Handshaking).unwrap();
let s = s.transition(ConnectionState::Connected).unwrap();
assert_eq!(s, ConnectionState::Connected);
```

## 9. See also

- [`UdtSocket::connect`](../socket/udt_socket_connect.md)
- [API index](../README.md)
- Protocol notes: [`../../protocol/state-diagram.md`](../../protocol/state-diagram.md)
