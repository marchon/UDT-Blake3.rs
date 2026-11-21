# `AckPacket::encode`

- **Crate:** `udt-core`
- **Module path:** `udt_core::wire`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl AckPacket {
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError>
}
```

## 2. Semantics

Encode an ACK control packet (type 2). Light ACKs are 16 bytes (cumulative ack
in the additional-info word). Full ACKs are 40 bytes and append data ack, RTT,
RTTVar, flow window, recv_rate, and bandwidth for CC feedback.

## 3. Preconditions / postconditions

**Preconditions**

- `buf.len() >= ACK_PACKET_LEN` (light) or `ACK_FULL_PACKET_LEN` (full).

**Postconditions**

- Control bit set; type ACK.

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| `TooShort` | Buffer too small | Grow buffer |

## 5. Thread-safety / async cancellation

Pure encode.

## 6. Wire effects

Produces ACK bytes for UDP send.

## 7. Compatibility

Subset of UDT4 ACK (optional RTT/rate fields not always present).

## 8. Example

See `udt_core::wire` unit test `ack_round_trip`.

## 9. See also

- [`data_packet_encode.md`](data_packet_encode.md)
- [`../../protocol/reliability.md`](../../protocol/reliability.md)
