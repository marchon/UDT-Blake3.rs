# Migration from udt-rs

`udt-rs` links C++ UDT4 via FFI. This workspace replaces that with a **pure Rust**
core (`udt-core`) and facades (`udt-socket`). Default features never link
`libudt`.

## API map

| udt-rs | udt-pure |
| --- | --- |
| `UdtSocket::new(AFInet, Stream)` | `udt_socket::UdtSocket::new` / `new_with_type(UdtSockType::Stream)` |
| `bind` / `listen` / `accept` / `connect` | Same names on `UdtSocket`; async via `async_socket::AsyncUdtSocket` (`tokio-rt`) |
| `send` / `recv` | `UdtSocket::send` / `recv`; call `flush` after large sends |
| `getsockopt` / `setsockopt` | `get_option` / `set_option` with `UdtOption` / `UdtOptionKind` |
| Epoll / select helpers | Not yet; use Tokio tasks or blocking pumps |
| FFI build of `libudt` | Not required for default features |
| File transfer apps | `udtcat` (`cargo run -p udt-cli`) |

## Options

See [`options.md`](options.md). Mapped today: `SndBuf`, `RcvBuf`, `Rendezvous`
(stored), `Mss`, `SndTimeo`, `RcvTimeo`, `MaxFlowWindow`.

## Handshake

Wire handshake follows the **UDT4 cookie protocol** (compatible with C++ UDT4).
PureRust peers and C++ peers can complete connect/accept on localhost; see
[`../protocol/handshake.md`](../protocol/handshake.md) and
[`../plan/interop-matrix.md`](../plan/interop-matrix.md).

## Interop during migration

1. Keep C++ UDT4 servers running.
2. Point PureRust clients at them (`run_interop_cell.sh rust-cpp`).
3. Flip servers to PureRust when the `cpp-rust` cell is green in your environment.

## CLI

```sh
cargo run -p udt-cli -- -l 9000 -o out.bin
cargo run -p udt-cli -- 127.0.0.1:9000 -i in.bin
```

Details: [`udtcat.md`](udtcat.md).

## Gaps vs udt-rs / UDT4

- No datagram messaging GA yet (stream first; ADR-0001).
- No rendezvous connect path yet (flag stored only).
- No epoll multiplex API.
- Shutdown control packet not emitted; `udtcat` uses idle end-of-stream.
- Cross-impl ACK optional fields may make `flush` warn even when bytes arrived.
