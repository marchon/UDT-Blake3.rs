# Conformance corpus

Synthetic and live golden vectors for UDT handshake / packet conformance.

## Install

```sh
# From the workspace root (vectors are checked in; no extra install):
ls tests/conformance/corpus/
```

## Usage

```sh
# Inspect the Phase 0 synthetic handshake header (16 bytes, hex):
xxd tests/conformance/corpus/handshake_control_word0.bin

# Or verify via udt-core unit test:
cargo test -p udt-core golden_handshake_word0
```

## Files

| File | Size | Description |
| --- | --- | --- |
| `handshake_control_word0.bin` | 16 | Synthetic UDT common header: control bit set, type Handshake (0) |

## Byte layout: `handshake_control_word0.bin`

Big-endian fields matching `docs/protocol/packets.md` and
`udt_core::packet::encode_control_word0(0)` / `MIN_HEADER_LEN`.

| Offset | Length | Value (hex) | Field |
| --- | --- | --- | --- |
| 0 | 4 | `80 00 00 00` | Word 0: bit 31 = control (`1`); bits 16..30 = type `0` (Handshake); bits 0..15 = 0 |
| 4 | 4 | `00 00 00 00` | Additional info (zeroed for synthetic peek) |
| 8 | 4 | `00 00 00 00` | Timestamp (zeroed) |
| 12 | 4 | `00 00 00 00` | Destination socket ID (zeroed; typical for initial HS) |

Construction (equivalent Rust):

```rust
let mut buf = [0u8; 16];
let w0 = udt_core::packet::encode_control_word0(0); // 0x8000_0000
buf[0..4].copy_from_slice(&w0.to_be_bytes());
// buf[4..] left zero
```

`udt_core::packet::peek_type(&buf)` must return `PacketType::Handshake`.

See [`../notes/phase0.md`](../notes/phase0.md) for synthetic vs live-pcap status.

## Contributing

Questions: open an issue in this repository. PRs that add documented live pcaps
or additional synthetic vectors are welcome; update `notes/` with capture
conditions (MTU, loss, UDT4 version).

## License

BSD-3-Clause © UDT pure-Rust contributors
