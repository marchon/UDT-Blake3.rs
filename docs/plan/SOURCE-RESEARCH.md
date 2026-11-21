# UDT-class transfer in pure Rust: research and build plans

> Archived from Grok Bot research PDF (2026-11-21). Source file:
> `UDT-class transfer in pure Rust_ research and build plans - Grok Bot.pdf`

## Goal

Build a **pure-Rust** implementation of **UDT** (UDP-based Data Transfer) that is
**class-compatible** with the original C++ UDT4/`libudt` and with existing Rust
wrappers (`udt-rs`, `tokio-udt`), so applications can migrate without rewriting
wire formats or control-plane semantics.

## Scope

- Wire protocol (handshake, data, ACK/NAK, keep-alive)
- Congestion control (UDT CC / UDT4)
- Socket API surface (stream + datagram, rendezvous, options)
- Interop with C++ UDT4 and with `udt-rs` / `tokio-udt` where feasible
- Performance targets: high BDP WAN paths; **avoid C++ FFI for the core path**

## Non-goals (v1)

- Full replacement of every historical UDT fork
- Kernel bypass (DPDK / io_uring) in v1
- Multipath or QUIC coexistence in v1

## Approach

1. Spec-first from RFCs/drafts + C++ reference
2. Pure Rust core (no `libudt` linkage on hot path)
3. Incremental milestones with interop tests
4. Document every public function (**one page per function**)

## Research findings

### 1. Protocol and references

- IETF draft: [draft-gg-udt-03](https://datatracker.ietf.org/doc/html/draft-gg-udt-03)
- Project site: [udt.sourceforge.net](https://udt.sourceforge.net/)
- C++ reference: [github.com/eminence/udt](https://github.com/eminence/udt) (UDT4)
- Wireshark UDT dissector (heuristic on handshake)

### 2. Existing Rust landscape

| Crate | Nature | Notes |
| --- | --- | --- |
| `udt-rs` | FFI to C++ UDT4 | Not pure Rust; inherits C++ build/ABI |
| `tokio-udt` | Pure-ish Tokio implementation | Useful reference; incomplete vs full class map |

**Gaps:** incomplete option parity, weak class/API mapping docs, sparse interop
test suites vs C++ UDT4.

### 3. What "class transfer" means

Map C++ UDT classes onto Rust modules/types with **equivalent responsibilities
and wire behavior**:

| C++ | Rust responsibility |
| --- | --- |
| `CUDTUnited` | Global/runtime registry |
| `CUDT` | Connection / socket instance |
| `CSndQueue` / `CRcvQueue` | Send/receive queues + mux |
| `CCC` | Congestion control trait + UDT4 CC |
| Packet types | Codec modules |

This is **not** an OOP inheritance copy. Prefer ownership + traits.

### 4. Risks

- Congestion control timing sensitivity
- Endianness / packing of control packets
- Rendezvous + firewall traversal
- API surface sprawl vs thin safe wrappers

## Technology implementation order

| Phase | Name | Deliverable |
| --- | --- | --- |
| 0 | Spec & harness | Packet corpus, CI, golden vectors from Wireshark/C++ |
| 1 | Packet codec | Encode/decode all control + data headers; property tests |
| 2 | Socket state machine | Handshake, connected, shutdown; stream + datagram stubs |
| 3 | Reliability | ACK/NAK, loss lists, retransmission, timers |
| 4 | Congestion control | UDT CC (rate + window); pluggable `CongestionControl` trait |
| 5 | API polish & options | `setsockopt`-class options; sync + async facades |
| 6 | Interop & soak | C++ peer matrix; long transfers; loss/reorder/latency |
| 7 | Docs freeze | One page per public function; migration guide from `udt-rs` |

**Rule:** Do not start Phase *N+1* until Phase *N* exit criteria pass
(tests + interop notes).

## Who needs to be involved (RACI)

| Role | Responsibilities | R/A/C/I |
| --- | --- | --- |
| Protocol lead | Spec fidelity, packet formats, CC math | A (protocol) |
| Rust systems eng | Core crates, unsafe boundaries, perf | R (implementation) |
| Test / interop eng | C++ sidecar, corpus, soak, CI matrices | R (quality) |
| Perf eng | Benches, flamegraphs, BDP scenarios | C / R (perf gates) |
| Docs / DX | One-page-per-fn, examples, migration | R (docs) |
| Security reviewer | Unsafe audit, fuzzing, advisory process | C |
| Product / owner | Prioritize stream vs datagram, MSRV, license | A (scope) |

Minimum viable team: Protocol lead + Rust eng + Test eng (Docs can be shared early).

## How to start (first 2 weeks)

### Week 1

- **Day 1–2:** Freeze references (draft + UDT4 tag). Capture 10 pcap handshakes. Create workspace.
- **Day 3–5:** Implement packet codec + golden tests. Stub `UdtSocket` state enum. Wire CI.

### Week 2

- **Day 6–8:** Handshake client/server localhost. Add lossy unit tests (drop/reorder).
- **Day 9–10:** First C++ interop (pure-Rust client ↔ UDT4 server). Publish API doc template.

### Program Definition of Done

- Pure Rust core builds with **zero** `libudt` link on default features
- Stream transfer localhost + induced 1% loss succeeds
- At least one C++ UDT4 interop path green in CI
- Public API documented one page per function
- Benchmarks published for 1 Gbps-class path (lab)

## Suggested repo layout

```text
udt-pure/   (this repository root: /Users/marchon/UDT)
  Cargo.toml          # workspace
  crates/
    udt-core/         # packets, timers, CC, state machine
    udt-socket/       # async + sync facades
    udt-cli/          # udtcat-style tools
    udt-interop/      # C++ sidecar tests
  docs/
    plan/             # this document + ADRs
    api/              # one page per public function
    protocol/         # packet layouts, state diagrams
    usage/            # end-user guides
  tests/
    conformance/
    soak/
  scripts/
  progress/           # minute-level agent/team console
```

**License note:** UDT C++ is BSD. Keep pure-Rust code clearly licensed; document
any copied constants/algorithms with attribution.

## Documentation standard (one page per function)

Each public `fn` / method gets `docs/api/<module>/<fn_name>.md`:

1. Signature
2. Semantics
3. Preconditions / postconditions
4. Errors
5. Thread-safety / async cancellation
6. Wire effects (if any)
7. Compatibility (C++ / `udt-rs` notes)
8. Example
9. See also

Codebase sections that must be covered: packet encode/decode, connection
lifecycle, send/recv paths, CCC hooks, socket options, interop helpers.

## Agent / team workload

| Team | Focus | Owns phases |
| --- | --- | --- |
| **A — Protocol Core** | Codec, state, reliability, CC | 0–4 |
| **B — API & DX** | Facades, options, one-pagers, migration | 5, 7 |
| **C — Interop & Perf** | C++ matrix, soak, benches | 6 + continuous |

### Definition of complete (per milestone)

- Code merges with tests green
- Interop matrix cell filled (or N/A with reason)
- Public APIs have one-pager in `docs/api/`
- CHANGELOG + ADR if behavior changed
- Progress console updated

## Console progress logging

Every minute emit:

```text
[HH:MM:SS] team=<A|B|C> agent=<id> task=<slug> milestone=<phase> pct=<0-100> blockers=<none|...>
```

Store append-only in `progress/progress.jsonl`; render via `scripts/progress_console.sh`.

## Open questions (answered in ADR-0001)

1. Async runtime: tokio-only vs runtime-agnostic?
2. Stream vs datagram priority for v1 GA?
3. Must bit-compat with UDT4 CC, or allow documented CC profile ID?
4. Minimum Rust edition / MSRV?
5. Vendored C++ UDT4 for CI vs system-installed?

## Immediate next actions

1. Owner picks MSRV + runtime (see ADR-0001)
2. Protocol lead opens Phase 0 (corpus + CI)
3. Docs creates `docs/api/` template
4. Kickoff with roles from RACI table
5. Start minute-level progress console

## References

- https://datatracker.ietf.org/doc/html/draft-gg-udt-03
- https://udt.sourceforge.net/
- https://github.com/eminence/udt
- https://github.com/eminence/udt-rs
- https://github.com/async-storm/tokio-udt (and crates.io `tokio-udt`)
- https://wiki.wireshark.org/UDT
