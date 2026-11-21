# Blake3 hybrid wire format (experimental)

Control bit set in word0; type in bits 16–30 (same layout as UDT4 control).

| Type | Code | Direction | Purpose |
| --- | --- | --- | --- |
| BlockSeal | `0x10` | sender → receiver | Announce sealed block |
| Want | `0x11` | receiver → sender | Pull missing packet slots |
| Have | `0x12` | receiver → sender | Block verified; free store |

## BlockSeal (`0x10`) — 60 bytes

| Offset | Field |
| --- | --- |
| 0–3 | control word0 (`0x10`) |
| 4–7 | `pkt_count` (exact packets in block; additional-info) |
| 8–11 | timestamp |
| 12–15 | dest sock id |
| 16–19 | `block_id` |
| 20–23 | `first_seq` |
| 24–27 | `nbytes` (≤ 65536) |
| 28–59 | Blake3-256 digest of contiguous plaintext (truncated to `nbytes`) |

## Want (`0x11`) — 32 bytes

| Offset | Field |
| --- | --- |
| 0–15 | UDT control header |
| 16–19 | `block_id` |
| 20–23 | `first_seq` |
| 24–31 | `missing_bitmap` (bit *i* ⇒ seq `first_seq + i` missing; max 64) |

## Have (`0x12`) — 20 bytes

| Offset | Field |
| --- | --- |
| 0–15 | UDT control header |
| 16–19 | `block_id` |

## Semantics

- Block size target: **64 KiB** application payload (`BLAKE3_BLOCK_BYTES`).
- Sender seals when accumulated open-block bytes ≥ 64 KiB (last packet may
  overshoot; digest uses truncated plaintext).
- Receiver parks pre-seal data in an **orphan** map, claims orphans on seal,
  WANTs gaps, verifies Blake3, emits HAVE, delivers contiguous verified blocks
  only for the Blake3 side-channel (app delivery still follows the ACK cursor).

Constants and codecs: `crates/udt-core/src/blake3_block.rs`.
