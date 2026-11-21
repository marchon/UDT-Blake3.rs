# Changelog

All notable changes to this project are documented in this file.

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
