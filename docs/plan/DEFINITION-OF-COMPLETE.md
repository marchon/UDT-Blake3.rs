# Definition of complete

## Program-level DoD (v1)

From research § “Definition of Done (program)”:

1. Pure Rust core builds with **zero** `libudt` link on default features
2. Stream transfer localhost + induced **1% loss** succeeds
3. At least one **C++ UDT4 interop** path green in CI
4. Public API documented **one page per function**
5. Benchmarks published for a lab path (`artifacts/bench/`; 1 Gbps-class WAN optional)

## Per-milestone DoD (every phase PR)

A milestone is complete only when **all** of the following are true:

| # | Criterion | Evidence |
| --- | --- | --- |
| 1 | Code merges with tests green | `cargo test --workspace` (and phase-specific packages) |
| 2 | Interop matrix cell filled or N/A with reason | `docs/plan/interop-matrix.md` |
| 3 | New/changed public APIs have one-pagers | `docs/api/**` paths linked from module README |
| 4 | CHANGELOG entry; ADR if behavior/semver changed | `CHANGELOG.md`, `docs/plan/ADR-*` |
| 5 | Progress console reflects completion | `progress/progress.jsonl` event with `pct=100` for the task |

## Per-agent DoD

See `ROLES-AND-TEAMS.md` agent table. An agent may mark a task complete only when
its row’s “Definition of complete” column is satisfied **and** it has appended a
progress event with `status=done`.

## Non-complete examples (do not ship)

- Codec that round-trips only happy-path handshakes (missing NAK/keep-alive)
- Socket facade that panics on drop without shutdown
- Docs that list functions without the nine-section template
- Interop “works on my machine” without CI or checked-in notes
- Progress stuck at the same `pct` for >30 minutes without a `blockers` update
