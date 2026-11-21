# `server_cookie_challenge`

- **Crate:** `udt-core`
- **Module path:** `udt_core::handshake`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn server_cookie_challenge(
    request: &HandshakePacket,
    syn_cookie: u32,
    timestamp: u32,
) -> HandshakePacket
```

## 2. Semantics

Build leg 2 of the UDT4 cookie handshake: echo the client request fields, attach
`syn_cookie`, keep `conn_type = 1`, set `dest_sock_id` to the client sock id.

## 3. Preconditions / postconditions

**Preconditions**

- `request` is an initial client request (`is_client_request`).

**Postconditions**

- `is_cookie_challenge` is true for the result.

## 4. Errors

None (pure constructor).

## 5. Thread-safety / async cancellation

Pure; `Send` + `Sync` value.

## 6. Wire effects

None until encoded and sent.

## 7. Compatibility

Matches C++ UDT4 listen path cookie challenge (`m_iReqType` stays 1).

## 8. Example

```rust
use udt_core::{client_request, server_cookie_challenge, syn_cookie, peer_ip_from_ipv4, SOCK_TYPE_STREAM};
let req = client_request(10, 100, SOCK_TYPE_STREAM, peer_ip_from_ipv4([127,0,0,1]), 1);
let ch = server_cookie_challenge(&req, syn_cookie(10, 1), 2);
assert_ne!(ch.syn_cookie, 0);
```

## 9. See also

- [`client_confirm.md`](client_confirm.md)
- [`server_final_response.md`](server_final_response.md)
- [`../../protocol/handshake.md`](../../protocol/handshake.md)
