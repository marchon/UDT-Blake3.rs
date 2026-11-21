# Changelog

All notable changes to this project are documented in this file.

## Unreleased

## 0.2.0 — 2026-11-21

Blake3 hybrid reliability + public docs cut as **UDT-Blake3.rs**.

### Added

- Feature `blake3-reliability`: 64 KiB Blake3 BlockSeal + WANT/HAVE (experimental control types `0x10`/`0x11`/`0x12`)
- Wired into `ReliabilityEngine` and socket dispatch; **`udt-cli` defaults to Blake3**
- Docs: why-Blake3, wire format, papers/implementations, full WAN result tables + raw logs
- `scripts/bench/` smoke/probe/ISO helpers; `dist/` prebuilt darwin-aarch64 + linux-x86_64 binaries

### Measured (WAN local → 70.88.205.138, ~4 GB ISO)

- UDT ACK/NAK **5.331 MiB/s** · UDT Blake3 **6.792 MiB/s** · SFTP **6.529 MiB/s**

### Fixed

- Congestion avoidance no longer pins CWND at 16 when `recv_rate` feedback is missing (WAN was stuck ~0.55 MiB/s)
- Full ACK carries RTT / RTTVar / flow window / recv_rate / bandwidth; sender applies them to CC
- Local RTT sampling from send timestamps + ACK2; RTO derived from `RTT + 4·RTTVar`
- `pump_once` drains UDP in non-blocking mode (removed ~20 ms empty-recv tax)

### Added

- Per-packet SND pacing (virtual clock) to avoid WAN UDP microbursts
- RTO retransmits oldest unacked packet only (not entire flight)
- 1 MiB application read/write chunks in `udtcat` / `udtd`
- `udtd` persistent multi-file delivery daemon (`serve -l … -d inbox`, `put host:port files…`)
- Framed file protocol: `u32 name_len | name | u64 file_len | body`

## 0.1.0 — 2026-11-21

First docs-freeze cut of the pure-Rust UDT4-class stack.

### Added

- Phase 7: API one-pager index freeze, migration guide, `LICENSE` + `NOTICE`, docs lint gate
- Phase 6: UDT4 cookie handshake (C++ compatible), vendored UDT4 peers, PureRust↔CppUdt4 interop cells, soak smoke, localhost bench
- Phase 5: socket options (`SNDBUF`/`RCVBUF`/rendezvous/MSS/timeouts/FC), Tokio `AsyncUdtSocket`, `udtcat` file listen/send
- Phase 4: Udt4Cc native congestion control (rate+window), FixedCc, engine CWND gating
- Phase 3: reliable stream send/recv, ACK/NAK, RTO, 8MiB@1% loss test
- Phase 2: localhost UDP handshake (`connect`/`accept`)
- `udt_core::handshake` helpers + protocol docs
- Workspace scaffold: `udt-core`, `udt-socket`, `udt-cli`, `udt-interop`
- Progress console (`scripts/progress_console.sh`)

### Known limitations

- Stream-first (datagram messaging deferred; ADR-0001)
- Rendezvous flag stored; HS path not wired
- Cross-impl ACK optional fields may make `flush` warn when bytes already arrived
- Lab localhost bench is not a 1 Gbps WAN claim
