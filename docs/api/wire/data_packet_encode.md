# `DataPacket::encode`

- **Crate:** `udt-core`
- **Module path:** `udt_core::wire`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl DataPacket {
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError>
}
```

## 2. Semantics

Encode a UDT data packet header plus payload into `buf`. Returns bytes written.

## 3. Preconditions / postconditions

**Preconditions**

- `buf` long enough for header + payload.

**Postconditions**

- `peek_type` on the prefix reports `Data`.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `TooShort` | Buffer too small | Grow buffer |

## 5. Thread-safety / async cancellation

Pure encode into caller buffer.

## 6. Wire effects

Produces bytes for UDP send.

## 7. Compatibility

UDT4 data header layout (seq, msg, timestamp, dest id).

## 8. Example

See `udt_core::wire` unit test `data_round_trip`.

## 9. See also

- [`ack_packet_encode.md`](ack_packet_encode.md)
- [`../../protocol/packets.md`](../../protocol/packets.md)
