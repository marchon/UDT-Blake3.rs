# ADR-0001: Startup decisions

- **Status:** Accepted
- **Date:** 2026-11-21
- **Owner:** Product / owner (George Lambert) with Protocol lead

## Context

Phase 2 must not complete until these five questions are recorded. Defaults below
unblocked scaffolding and agent assignment; change them only with a superseding ADR.

## Decisions

| # | Question | Decision |
| --- | --- | --- |
| 1 | Async runtime | **Tokio-first** for `udt-socket` async facade. `udt-core` stays runtime-agnostic (no Tokio types in public core API). |
| 2 | Stream vs datagram for v1 GA | **Stream first** (SOCK_STREAM equivalent). Datagram stubs land in Phase 2; full datagram GA is post-v1 unless interop forces earlier. |
| 3 | UDT4 CC bit-compat | **Goal: bit-compatible UDT4 CC behavior** on the default profile. Extra CC profiles may exist behind explicit opt-in IDs later. |
| 4 | MSRV / edition | **Edition 2021**, **MSRV 1.75**. CI tests stable + MSRV. |
| 5 | C++ UDT4 in CI | **Vendored git submodule** under `crates/udt-interop/vendor/udt` (tag pinned). System `libudt` is optional for local dev only. |

## Consequences

- Agents must not add Tokio to `udt-core` dependencies.
- Phase 6 CI image builds vendored UDT4; first interop PR includes submodule pin.
- Datagram-only users get an explicit non-goal until post-v1 roadmap.

## Alternatives considered

- Runtime-agnostic async (embassy/`async-std`): deferred; raises DX cost for little v1 gain.
- Rely on system libudt only: fragile CI; rejected.
