# `AsyncUdtSocket`

- **Crate:** `udt-socket`
- **Module path:** `udt_socket::async_socket` (feature `tokio-rt`)
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** B

## 1. Signature

```rust
pub struct AsyncUdtSocket { /* inner: UdtSocket */ }

impl AsyncUdtSocket {
    pub fn new() -> Self;
    pub async fn bind(&mut self, addr: SocketAddr) -> Result<(), SocketError>;
    pub async fn listen(&mut self, backlog: i32) -> Result<(), SocketError>;
    pub async fn connect(&mut self, peer: SocketAddr) -> Result<(), SocketError>;
    pub async fn accept(&mut self) -> Result<(UdtSocket, SocketAddr), SocketError>;
    pub async fn send(&mut self, buf: Vec<u8>) -> Result<usize, SocketError>;
    pub async fn recv(&mut self, max: usize) -> Result<Vec<u8>, SocketError>;
    pub async fn flush(&mut self, timeout: Duration) -> Result<(), SocketError>;
    pub fn set_option(&mut self, opt: UdtOption) -> Result<(), SocketError>;
    pub fn get_option(&self, kind: UdtOptionKind) -> UdtOption;
    pub fn state(&self) -> ConnectionState;
    pub fn into_inner(self) -> UdtSocket;
}
```

## 2. Semantics

Tokio-first facade over the sync `UdtSocket`. Handshake, send, recv, and flush
run on `tokio::task::spawn_blocking` so the runtime is not stalled by UDP polls.
`bind` / `listen` / options call the sync methods directly (fast path).

`accept` returns a **sync** child `UdtSocket` plus peer address (same as sync
`accept`). Wrap the child with `AsyncUdtSocket { inner }` only if you own the
struct fields; Phase 5 exposes `into_inner` the other way.

## 3. Preconditions / postconditions

Same state machine rules as the sync facade (`Init` → handshake → `Connected`).

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `Io` | `spawn_blocking` join failure | Retry / abort |
| Other | Same as sync `UdtSocket` | See sync one-pagers |

## 5. Thread-safety / async cancellation

- Not `Sync`; hold `&mut self` across `.await`.
- Cancellation of `connect` / `accept` / `send` / `recv` / `flush` may drop the
  in-flight blocking task; prefer awaiting to completion for clean shutdown.

## 6. Wire effects

Same as sync connect/accept/send/recv/flush.

## 7. Compatibility

Closest to a Tokio-friendly wrap of C++ UDT4 blocking sockets; not a native
epoll integration.

## 8. Example

```rust
// Enable feature: udt-socket = { path = "...", features = ["tokio-rt"] }
// use udt_socket::async_socket::AsyncUdtSocket;
// let mut s = AsyncUdtSocket::new();
// s.bind("127.0.0.1:0".parse()?).await?;
// s.connect(peer).await?;
// s.send(b"hello".to_vec()).await?;
// s.flush(std::time::Duration::from_secs(3)).await?;
```

## 9. See also

- [`udt_socket_connect.md`](udt_socket_connect.md)
- [`udt_socket_send.md`](udt_socket_send.md)
- [`../../usage/options.md`](../../usage/options.md)
