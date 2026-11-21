# How to start

## 1. Read (30 minutes)

1. [`SOURCE-RESEARCH.md`](SOURCE-RESEARCH.md) — goals, scope, non-goals
2. [`ADR-0001-startup-decisions.md`](ADR-0001-startup-decisions.md) — MSRV, Tokio, CC
3. [`TECH-ORDER.md`](TECH-ORDER.md) — phase gates
4. [`ROLES-AND-TEAMS.md`](ROLES-AND-TEAMS.md) — who does what
5. [`DEFINITION-OF-COMPLETE.md`](DEFINITION-OF-COMPLETE.md) — exit rules

## 2. Toolchain

```sh
cd /Users/marchon/UDT
rustup toolchain install 1.75.0
rustup default stable
cargo test --workspace
```

## 3. Progress console

```sh
./scripts/progress_console.sh          # prints every 60s
# or
PROGRESS_INTERVAL_SEC=60 ./scripts/progress_console.sh
```

Agents append events with:

```sh
./scripts/progress_tick.sh <team> <agent> <task> <milestone> <pct> <status> <blockers> [note]
```

## 4. First engineering work (Phase 0 → 1)

| Order | Owner | Task |
| --- | --- | --- |
| 1 | C1 | Add synthetic golden handshake bytes under `tests/conformance/corpus/` |
| 2 | A1 | Extend `packet` module to encode/decode full handshake fields |
| 3 | A2 | Drive handshake over localhost UDP (after codec) |
| 4 | B1 | Keep one-pagers in sync with every new public fn |
| 5 | C2 | Pin UDT4 submodule notes (no link until Phase 6) |

## 5. Kickoff checklist

- [ ] Owner confirms ADR-0001 (or files superseding ADR)
- [ ] Protocol lead named for Team A
- [ ] Test eng named for Team C
- [ ] Progress console running in a visible terminal
- [ ] Phase 0 exit criteria tracked in `TECH-ORDER.md`
