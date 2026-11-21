# Technology implementation order

Ordered for **key development and testing**. Later phases consume artifacts from earlier ones. Do not skip exit criteria.

## Dependency graph

```text
Phase 0 Spec & harness
   │
   ▼
Phase 1 Packet codec ◄── golden pcaps / vectors
   │
   ▼
Phase 2 Socket state machine ◄── codec
   │
   ├──────────────┐
   ▼              ▼
Phase 3        Phase 5 (partial stubs OK)
Reliability    API facades / options
   │              │
   ▼              │
Phase 4 CC ◄──────┘
   │
   ▼
Phase 6 Interop & soak
   │
   ▼
Phase 7 Docs freeze
```

## Phase exit criteria

### Phase 0 — Spec & harness

- [ ] Draft-gg-udt-03 + UDT4 tag pinned in docs
- [x] Synthetic golden handshake vectors checked in (`tests/conformance/corpus/`)
- [ ] ≥10 live handshake pcaps (still needed; see `tests/conformance/notes/phase0.md`)
- [ ] CI builds workspace on stable + MSRV
- [ ] Progress console live

**Complete when:** corpus README exists and `cargo test -p udt-core` runs (even if empty).

### Phase 1 — Packet codec

- [ ] Control + data header encode/decode
- [ ] Endianness tests
- [ ] Property / fuzz smoke on codec
- [ ] One-pagers for `encode_*` / `decode_*` public fns

**Complete when:** round-trip tests cover handshake, ACK, NAK, data, keep-alive.

### Phase 2 — Socket state machine

- [x] States: Init, Handshaking, Connected, Shutdown, Closed
- [x] Client + server handshake on localhost UDP (three-way)
- [x] Stream default; datagram type handshake stub (`UdtSockType`)
- [x] State diagram in `docs/protocol/` (+ `handshake.md`)

**Complete when:** two processes complete handshake without data payload. ✅ (`localhost_three_way_handshake` test)

### Phase 3 — Reliability

- [x] ACK/NAK generation and handling (`wire` + `ReliabilityEngine`)
- [x] Loss list + retransmission (RTO + NAK-driven)
- [x] Timer wheel / deadline API (`TimerWheel`)
- [x] Unit tests with induced drop/reorder + `set_loss_rate(0.01)`

**Complete when:** 1% loss localhost stream transfer of 8 MiB succeeds. ✅ (`transfer_8mib_with_one_percent_loss`)

### Phase 4 — Congestion control

- [x] `CongestionControl` trait (ACK/loss/timeout + rate/RTT hooks)
- [x] Default UDT4 CC implementation (rate + window) — `Udt4Cc`
- [x] Pluggable injection for tests (`FixedCc`, `ReliabilityEngine::with_cc`)
- [x] CC math unit tests vs reference vectors (`docs/plan/cc-reference-vectors.md`)

**Complete when:** CC responds to ACK/NAK in sim; no panics under loss. ✅ (`sim_ack_nak_no_panic` + wired into send path)

### Phase 5 — API polish & options

- [x] Sync facade (`UdtSocket` send/recv/flush + options)
- [x] Async Tokio facade (`async_socket::AsyncUdtSocket`, feature `tokio-rt`)
- [x] Core socket options mapped (`SNDBUF`, `RCVBUF`, rendezvous, MSS, timeouts, FC)
- [x] Examples in `udt-cli` (`udtcat` listen/send file)

**Complete when:** `udt-cli` can send a file localhost; options documented. ✅ (`localhost_file_round_trip` + `docs/usage/options.md` / `udtcat.md`)

### Phase 6 — Interop & soak

- [x] Pure-Rust ↔ C++ UDT4 matrix (client/server both ways) — cookie HS + stream peers
- [x] Soak smoke with loss profile (30s localhost rounds + 8MiB@1% unit; 1h via `SOAK_SECONDS=3600`)
- [x] Bench numbers for lab localhost path (`artifacts/bench/summary.txt`)

**Complete when:** CI job green for at least one C++ interop cell + soak smoke. ✅ (`interop.yml` + `artifacts/interop/rust-cpp/` / `cpp-rust/` + soak smoke)

### Phase 7 — Docs freeze

- [x] Every public item has `docs/api/...` page (index + one-pagers; lint enforces headings/links)
- [x] Migration guide from `udt-rs` (`docs/usage/migration-udt-rs.md`)
- [x] CHANGELOG for v0.1 (`CHANGELOG.md` dated 0.1.0)
- [x] License + attribution complete (`LICENSE`, `NOTICE`)

**Complete when:** docs lint script passes (`scripts/check_api_docs.sh`). ✅

## Testing strategy by phase

| Phase | Unit | Property/fuzz | Interop | Perf |
| --- | --- | --- | --- | --- |
| 0 | harness smoke | — | capture only | — |
| 1 | codec round-trip | yes | pcap decode | — |
| 2 | state transitions | — | localhost HS | — |
| 3 | loss injection | — | localhost stream | micro |
| 4 | CC math | — | sim | micro |
| 5 | options | — | cli | — |
| 6 | — | — | C++ matrix | bench + soak |
| 7 | doc tests | — | — | — |
