# Connection state diagram (Phase 2)

```text
                    ┌─────────┐
                    │  Init   │
                    └────┬────┘
                         │ connect / listen accept path
                         ▼
                  ┌──────────────┐
           ┌──────│ Handshaking  │──────┐
           │      └──────┬───────┘      │ failure / cancel
           │             │ success      │
           │             ▼              ▼
           │      ┌────────────┐   ┌─────────┐
           │      │ Connected  │──►│ Closed  │
           │      └─────┬──────┘   └─────────┘
           │            │ shutdown
           │            ▼
           │      ┌────────────┐
           └─────►│ Shutdown   │──► Closed
                  └────────────┘
```

Allowed edges are enforced by `ConnectionState::transition` in `udt-core`.

## `Connection` stub

[`Connection`](../../crates/udt-core/src/state.rs) wraps `ConnectionState` plus local/peer
UDT socket IDs (`sock_id` / `peer_sock_id`, both `u32`). It performs no UDP I/O; higher
layers drive handshake bytes and call these helpers, which all go through `transition`:

| Method | Edge |
| --- | --- |
| `begin_handshake` | Init → Handshaking |
| `mark_connected` | Handshaking → Connected |
| `begin_shutdown` | Connected → Shutdown |
| `close` | any allowed predecessor → Closed |

See [`docs/api/state/begin_handshake.md`](../api/state/begin_handshake.md).


## Listening sockets

A server calls `bind` + `listen` while remaining in `Init`. The boolean
`UdtSocket::is_listening()` distinguishes listeners from idle sockets.
`accept` produces a **new** `UdtSocket` that walks `Handshaking` → `Connected`.
See [handshake.md](handshake.md).
