# Socket options (Phase 5)

Typed `setsockopt` / `getsockopt` analogues on `udt_socket::UdtSocket` and
`AsyncUdtSocket` (`tokio-rt` feature).

## API

```rust
use std::time::Duration;
use udt_socket::{UdtOption, UdtOptionKind, UdtSocket};

let mut s = UdtSocket::new();
s.set_option(UdtOption::SndBuf(512 * 1024))?;
s.set_option(UdtOption::Mss(1400))?;
s.set_option(UdtOption::Rendezvous(false))?;
assert_eq!(
    s.get_option(UdtOptionKind::SndBuf),
    UdtOption::SndBuf(512 * 1024)
);
let _ = s.options(); // borrow full SocketOptionState
```

Defaults (`SocketOptionState::default`):

| Field | Default |
| --- | --- |
| `snd_buf` / `rcv_buf` | 256 KiB |
| `rendezvous` | `false` |
| `mss` | `udt_core::DEFAULT_MSS_PAYLOAD` |
| `snd_timeo` | 30 s |
| `rcv_timeo` | 50 ms |
| `max_flow_window` | 16_384 packets |

## Option map

| Kind | C++ / UDT name | Effect in Phase 5 |
| --- | --- | --- |
| `SndBuf` | `UDT_SNDBUF` | Stored; applied to OS `SO_SNDBUF` on bind / set. Clamped ≥ 1024. |
| `RcvBuf` | `UDT_RCVBUF` | Stored; applied to OS `SO_RCVBUF` on bind / set. Clamped ≥ 1024. |
| `Rendezvous` | `UDT_RENDEZVOUS` | Stored; must be set in `Init`. Handshake path TBD. |
| `Mss` | `UDT_MSS` | Payload size hint; applied to live `ReliabilityEngine` via `set_mss`. Clamped 64…64 KiB. |
| `SndTimeo` | `UDT_SNDTIMEO` | Stored for send/flush-style waits. |
| `RcvTimeo` | `UDT_RCVTIMEO` | Drives UDP `set_read_timeout` and pump polling. |
| `MaxFlowWindow` | `UDT_FC` | Max CWND packets; applied via `set_max_cwnd` after connect. ≥ 2. |

## Errors

- `Rendezvous` after `Init` → `SocketError::Transfer("rendezvous must be set before connect/listen")`

## Async

With feature `tokio-rt`, `udt_socket::async_socket::AsyncUdtSocket` forwards
`set_option` / `get_option` to the inner sync socket (no spawn needed).

## See also

- [`../api/socket/udt_socket_set_option.md`](../api/socket/udt_socket_set_option.md)
- [`../api/socket/udt_socket_get_option.md`](../api/socket/udt_socket_get_option.md)
- [`migration-udt-rs.md`](migration-udt-rs.md)
