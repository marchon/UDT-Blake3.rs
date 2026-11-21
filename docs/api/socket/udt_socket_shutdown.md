# `UdtSocket::shutdown`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub fn shutdown(&mut self) -> Result<(), SocketError>
```

## 2. Semantics

Begin orderly shutdown from `Connected` (`Connected` → `Shutdown` → `Closed`).
Does not yet emit a UDT shutdown control packet on the wire (Phase 5/6 idle
end-of-stream is used by `udtcat`).

## 3. Preconditions / postconditions

**Preconditions**

- Prefer `Connected` (other states may return `StateError`).

**Postconditions**

- State is `Closed` on success from `Connected`.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `State` | Illegal transition | Check `state()` |

## 5. Thread-safety / async cancellation

Blocking; not shared across threads.

## 6. Wire effects

None today (no shutdown control packet yet).

## 7. Compatibility

Partial analogue of `UDT::close`; full FIN/shutdown packet TBD.

## 8. Example

```rust
// let _ = socket.shutdown();
```

## 9. See also

- [`udt_socket_flush.md`](udt_socket_flush.md)
- [`../state/transition.md`](../state/transition.md)
