# Phase 0 corpus notes

## Status: synthetic only

The vectors under `tests/conformance/corpus/` checked in for Phase 0 are
**synthetic** — hand-built from the draft-gg-udt-03 / UDT4 common-header layout
and `udt_core::packet::encode_control_word0`. They are **not** extracted from
live pcaps or Wireshark captures.

| Artifact | Source | Purpose |
| --- | --- | --- |
| `handshake_control_word0.bin` | Synthetic (`encode_control_word0(0)` + zero pad to 16) | `peek_type` → Handshake; header endianness smoke |

## Live pcaps still needed

Phase 0 exit prefers **≥10 handshake captures** (or equivalent). Synthetic
goldens cover codec smoke only. Still needed before treating interop corpus as
complete:

| # | Capture target | Notes |
| --- | --- | --- |
| 1–2 | Localhost client→server full handshake (both directions) | UDT4 C++ or this stack once HS works |
| 3–4 | Localhost server→client / reverse roles | Document socket IDs and cookie |
| 5–6 | Rendezvous handshake (if enabled) | Optional for early Phase 2 |
| 7–8 | Lab path with latency (WAN-ish) | Note RTT / MTU in capture notes |
| 9–10 | Lossy or reordered HS retries | Useful for reliability later |

**Target:** 10 handshake pcaps (pcapng preferred) plus optional raw `.bin`
exports of the first HS packet per capture. Document MTU, loss, and UDT4
version alongside each file under `notes/`.

## Guidelines

See parent `tests/conformance/README.md` for Wireshark / export steps.
