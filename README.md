# UDT Pure Rust

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

Pure-Rust UDT (UDP-based Data Transfer) with C++ UDT4 class-compatible behavior.

Maps C++ UDT4 responsibilities (`CUDT`, queues, `CCC`, packets) onto Rust modules
and traits without linking `libudt` on the default hot path. See
`docs/plan/SOURCE-RESEARCH.md` for the full research and build plan.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Project layout](#project-layout)
- [Documentation](#documentation)
- [Progress console](#progress-console)
- [Contributing](#contributing)
- [License](#license)

## Background

UDT is a reliable UDP application-level protocol for high-BDP WAN transfers.
Existing Rust options either FFI to C++ (`udt-rs`) or partially reimplement the
stack (`tokio-udt`). This workspace aims for a **pure-Rust** core with interop
tests against vendored UDT4.

## Install

```sh
# Requires Rust 1.75+ (MSRV) and a recent stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cd /Users/marchon/UDT
cargo build --workspace
cargo test --workspace
```

## Usage

```sh
# Run unit tests
cargo test -p udt-core

# udtcat: listen then send a file over localhost UDT
cargo run -p udt-cli -- -l 9000 -o /tmp/out.bin
# other terminal:
cargo run -p udt-cli -- 127.0.0.1:9000 -i /tmp/in.bin

# Phase 6 interop (needs vendored UDT4 — see crates/udt-interop/VENDOR.md)
./crates/udt-interop/scripts/build_vendor.sh
./crates/udt-interop/scripts/run_interop_cell.sh rust-cpp 65536

# Minute-level progress board (teams/agents)
./scripts/progress_console.sh --once
```

## Project layout

| Path | Purpose |
| --- | --- |
| `crates/udt-core` | Packets, state, reliability, CC (no Tokio) |
| `crates/udt-socket` | Sync + Tokio async facades |
| `crates/udt-cli` | `udtcat` tool |
| `crates/udt-interop` | C++ UDT4 matrix harness |
| `docs/plan` | Research, ADR, roles, tech order, DoD |
| `docs/api` | One page per public function |
| `progress/` | Agent/team console jsonl + logs |

## Documentation

- Plan: [`docs/plan/SOURCE-RESEARCH.md`](docs/plan/SOURCE-RESEARCH.md)
- Tech order: [`docs/plan/TECH-ORDER.md`](docs/plan/TECH-ORDER.md)
- Roles: [`docs/plan/ROLES-AND-TEAMS.md`](docs/plan/ROLES-AND-TEAMS.md)
- DoD: [`docs/plan/DEFINITION-OF-COMPLETE.md`](docs/plan/DEFINITION-OF-COMPLETE.md)
- API template: [`docs/api/TEMPLATE.md`](docs/api/TEMPLATE.md)
- Developer guide: [`docs/usage/developer.md`](docs/usage/developer.md)
- Handshake: [`docs/protocol/handshake.md`](docs/protocol/handshake.md)
- Reliability: [`docs/protocol/reliability.md`](docs/protocol/reliability.md)
- Congestion: [`docs/protocol/congestion.md`](docs/protocol/congestion.md)
- Testing guide: [`docs/usage/testing.md`](docs/usage/testing.md)
- udtcat: [`docs/usage/udtcat.md`](docs/usage/udtcat.md)
- Socket options: [`docs/usage/options.md`](docs/usage/options.md)
- Interop matrix: [`docs/plan/interop-matrix.md`](docs/plan/interop-matrix.md)
- Migration from udt-rs: [`docs/usage/migration-udt-rs.md`](docs/usage/migration-udt-rs.md)

## Progress console

```sh
chmod +x scripts/*.sh
./scripts/progress_tick.sh A A1-codec handshake-header-codec phase-1 10 running none "started"
./scripts/progress_console.sh
```

## Contributing

Questions: open an issue in this repository. PRs are accepted when they include
tests for the touched phase exit criteria and, for public APIs, a one-page doc
under `docs/api/` using the template. Follow `docs/plan/TECH-ORDER.md` — do not
skip phase exit criteria.

## License

[BSD-3-Clause](LICENSE) © UDT pure-Rust contributors

Third-party UDT4 attribution (optional vendor tree): see [`NOTICE`](NOTICE) and
[`crates/udt-interop/VENDOR.md`](crates/udt-interop/VENDOR.md).
