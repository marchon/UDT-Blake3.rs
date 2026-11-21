# `server_response`

- **Crate:** `udt-core`
- **Module path:** `udt_core::handshake`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

See rustdoc for `udt_core::handshake::server_response`.

## 2. Semantics

Compatibility helper. Prefer `server_cookie_challenge` (leg 2) and
`server_final_response` (leg 4) for the UDT4 cookie protocol. No I/O.

## 3. Preconditions / postconditions

**Preconditions**

- Field values supplied by the caller are trusted.

**Postconditions**

- Returned packet encodes with `HandshakePacket::encode`.

## 4. Errors

None (returns `HandshakePacket` by value).

## 5. Thread-safety / async cancellation

Pure / `Send` + `Sync` data.

## 6. Wire effects

None until the socket layer sends the encoded bytes.

## 7. Compatibility

Aligned with UDT4 handshake field meanings (`conn_type` 1 / −1).

## 8. Example

See `udt_core::handshake` unit test `three_way_shapes`.

## 9. See also

- [`../../protocol/handshake.md`](../../protocol/handshake.md)
