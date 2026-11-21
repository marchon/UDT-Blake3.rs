# Packet layouts (Phase 0–1)

Reference: [draft-gg-udt-03](https://datatracker.ietf.org/doc/html/draft-gg-udt-03), UDT4 `CPacket`.

## Common header

Multi-byte fields are **big-endian**.

| Bits | Field |
| --- | --- |
| 1 | Flag: `1` = control, `0` = data |
| 15 | Control type (if control) or unused |
| 16 | Reserved / additional type info |
| 32 | Additional info / ACK seq / etc. (type-specific) |
| 32 | Timestamp |
| 32 | Destination socket ID |

Minimum decoded length in this codebase: **16 bytes** (`udt_core::packet::MIN_HEADER_LEN`).

## Control types (UDT4)

| Type ID | Name | `PacketType` |
| --- | --- | --- |
| 0 | Handshake | `Handshake` |
| 1 | Keep-alive | `KeepAlive` |
| 2 | ACK | `Ack` |
| 3 | NAK | `Nak` |
| 5 | Shutdown | `Shutdown` |
| 6 | ACK2 | `Ack2` |
| 7 | Message drop request | `MsgDropReq` |

## Handshake (TYPE 0x0)

Total length: **64 bytes** (`HANDSHAKE_PACKET_LEN`).

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | Control word0 (`0x8000_0000`, type 0) |
| 4 | 4 | Additional info (undefined; encode as 0) |
| 8 | 4 | Timestamp |
| 12 | 4 | Destination socket ID |
| 16 | 4 | UDT version |
| 20 | 4 | Socket type (STREAM / DGRAM) |
| 24 | 4 | Initial sequence number |
| 28 | 4 | Maximum packet size |
| 32 | 4 | Maximum flow window size |
| 36 | 4 | Connection type (`i32`: 1 request, 0 rendezvous, −1/−2 response) |
| 40 | 4 | Socket ID |
| 44 | 4 | SYN cookie |
| 48 | 16 | Peer IP (16-octet placeholder) |

## Golden corpus

Place pcaps and byte vectors under `tests/conformance/corpus/`. See
`tests/conformance/README.md`.
