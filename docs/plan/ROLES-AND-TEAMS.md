# Roles, teams, and responsibilities

## Teams

| ID | Name | Mission | Phase ownership |
| --- | --- | --- | --- |
| A | Protocol Core | Spec-faithful wire + reliability + CC in pure Rust | 0, 1, 2, 3, 4 |
| B | API & DX | Safe facades, options, one-page-per-fn docs, migration | 5, 7 (+ templates from day 1) |
| C | Interop & Perf | C++ peer matrix, soak, benches, progress gates | 6 (+ continuous from Phase 2) |

## Human RACI (from research)

| Role | Primary team | Accountable for |
| --- | --- | --- |
| Protocol lead | A | Packet formats, state machine correctness, CC math |
| Rust systems eng | A / B | Core crates, unsafe boundaries, socket facades |
| Test / interop eng | C | Corpus, C++ sidecar, soak, CI matrices |
| Perf eng | C | Benchmarks, BDP scenarios, perf regressions |
| Docs / DX | B | `docs/api/*`, usage guides, migration from `udt-rs` |
| Security reviewer | A+C (consulted) | Unsafe audit, fuzz corpus, advisory path |
| Product / owner | — | Scope, MSRV, license, stream-vs-datagram priority |

## Agent assignments (this build)

| Agent ID | Team | Role | Definition of complete |
| --- | --- | --- | --- |
| `A1-codec` | A | Packet codec + golden vectors | All header types encode/decode round-trip; property tests green |
| `A2-state` | A | Connection state machine stubs | Handshake states documented + unit-tested transitions |
| `A3-reliability` | A | ACK/NAK / loss list / timers (scaffold) | Module + tests stubs; timer trait sketched |
| `A4-cc` | A | `CongestionControl` trait + UDT4 CC stub | Trait stable; default CC compiles; unit tests for rate math stubs |
| `B1-api-docs` | B | One-page-per-fn template + seed pages | Template + ≥5 seed API pages for planned public fns |
| `B2-socket-facade` | B | `udt-socket` sync/async skeleton | Crate builds; `UdtSocket` stub types public |
| `B3-usage` | B | Usage + migration docs | `docs/usage/*` + migration outline from `udt-rs` |
| `C1-corpus` | C | Phase 0 harness + pcap notes | `tests/conformance` layout + capture README |
| `C2-interop` | C | C++ sidecar plan + vendor pin notes | `udt-interop` crate + build docs (submodule TBD) |
| `C3-progress` | C | Progress console + jsonl schema | Minute ticker running; agents can append events |
| `ORCH` | — | Orchestrator | All teams have clear DoD; console live; Phase 0 exit checklist open |

## Hand-offs

```text
C1 corpus ──► A1 codec
A1 codec  ──► A2 state ──► A3 reliability ──► A4 cc
A2 stubs  ──► B2 facade
B1 template concurrently from start
A4 + B2   ──► C2 interop (Phase 6 gate)
All public fns ──► B1 one-pagers before Phase 7 freeze
```

## Communication

- Progress: `progress/progress.jsonl` (append-only) + console every 60s
- Decisions: `docs/plan/ADR-*.md`
- Blockers: set `blockers` field in progress events; orchestrator escalates
