# `client_cookie_confirm` (alias `client_confirm`)

- **Crate:** `udt-core`
- **Module path:** `udt_core::handshake`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn client_cookie_confirm(
    challenge: &HandshakePacket,
    client_sock_id: u32,
    client_init_seq: u32,
    client_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket
```

## 2. Semantics

Build leg 3 of the UDT4 cookie handshake: `conn_type = -1` with the challenge
cookie echoed. (`client_confirm` is a compatibility alias.)

## 3. Preconditions / postconditions

**Preconditions**

- `challenge` is a cookie challenge (`is_cookie_challenge`).

**Postconditions**

- `is_cookie_confirm` / `is_server_response` is true for the result.

## 4. Errors

None (pure constructor).

## 5. Thread-safety / async cancellation

Pure; `Send` + `Sync` value.

## 6. Wire effects

None until encoded and sent.

## 7. Compatibility

Matches C++ UDT4 client after receiving a cookie challenge (`m_iReqType = -1`).

## 8. Example

```rust
use udt_core::{client_cookie_confirm, client_request, server_cookie_challenge, syn_cookie, SOCK_TYPE_STREAM, peer_ip_from_ipv4};
let req = client_request(10, 100, SOCK_TYPE_STREAM, peer_ip_from_ipv4([127,0,0,1]), 1);
let ch = server_cookie_challenge(&req, syn_cookie(10, 1), 2);
let conf = client_cookie_confirm(&ch, 10, 100, peer_ip_from_ipv4([127,0,0,1]), 3);
```

## 9. See also

- [`server_response.md`](server_response.md)
- [`../../protocol/handshake.md`](../../protocol/handshake.md)
