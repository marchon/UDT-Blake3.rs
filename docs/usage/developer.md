# Developer guide

## Prerequisites

- Rust **1.75+** (MSRV); develop on stable
- `cargo`, `rustfmt`, `clippy`
- Optional for Phase 6: C++ toolchain for vendored UDT4

## Build and test

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
```

## Crate map

| Crate | Allowed deps | Forbidden |
| --- | --- | --- |
| `udt-core` | `bytes`, `thiserror`, std | Tokio, `libudt` |
| `udt-socket` | `udt-core`, optional Tokio | `libudt` on default |
| `udt-cli` | `udt-socket` | — |
| `udt-interop` | `udt-core`; optional C++ | linking C++ unless `cpp-udt4` |

## Adding a public function

1. Implement with docs (`///`) and unit tests.
2. Copy `docs/api/TEMPLATE.md` to `docs/api/<module>/<fn>.md`.
3. Fill all nine sections (Signature through See also).
4. Link the page from `docs/api/README.md`.
5. Run `./scripts/check_api_docs.sh`.
6. Append a progress tick for team B.

## Phase discipline

Read `docs/plan/TECH-ORDER.md`. Start Phase N+1 only when Phase N exit criteria
are checked off in that file (PR description must quote them).

## Progress reporting

```sh
./scripts/progress_tick.sh <team> <agent> <task> <milestone> <pct> <status> <blockers> [note]
```

Statuses: `running` | `blocked` | `done` | `idle`.

## Debugging

### Packet peeks

Classify a raw UDP buffer without owning a socket. Useful when dumping
localhost captures or asserting goldens in unit tests:

```rust
use udt_core::packet::{encode_control_word0, peek_type, PacketType, MIN_HEADER_LEN};

let mut buf = [0u8; MIN_HEADER_LEN];
let w0 = encode_control_word0(0); // handshake
buf[0..4].copy_from_slice(&w0.to_be_bytes());
assert_eq!(peek_type(&buf).unwrap(), PacketType::Handshake);
```

- API one-pager: [`docs/api/packet/peek_type.md`](../api/packet/peek_type.md)
- Wire layouts: [`docs/protocol/packets.md`](../protocol/packets.md)
- Related helpers: `is_control`, `encode_control_word0`

For offline inspection, prefer Wireshark with a UDT dissector, then compare
against `peek_type` on the exported first-header bytes.

### Progress console

Agents append minute-level status; the console renders the latest event per
agent. Operators use this to see blockers without reading git history.

```sh
# Append a tick (from any agent)
./scripts/progress_tick.sh B B3-usage usage-docs phase-7 50 running none "drafting Debugging"

# One-shot board (CI / quick check)
./scripts/progress_console.sh --once

# Live board (default interval 60s; override with PROGRESS_INTERVAL_SEC)
./scripts/progress_console.sh
```

Artifacts:

| Path | Role |
| --- | --- |
| `progress/progress.jsonl` | Append-only event log |
| `progress/latest_by_agent.json` | Latest event keyed by agent |
| `progress/console.log` | Human board mirror |

### Logging (`RUST_LOG`) — future

The crates do **not** yet emit structured logs via `tracing` / `env_logger`.
`udtcat` prints status on stderr; expect a structured facade later:

```sh
# Planned once a logging facade lands (not wired today)
RUST_LOG=udt_core=debug,udt_socket=info cargo run -p udt-cli -- -h
```

Until then: use packet peeks + unit asserts for wire bugs, and the progress
console for team/agent status. Do not invent ad-hoc `eprintln!` on hot paths
in `udt-core`; prefer tests and peeks.

## Phase 5 — options and udtcat

```sh
cargo test -p udt-socket socket_options_round_trip
cargo test -p udt-cli localhost_file_round_trip
cargo run -p udt-cli -- -l 9000 -o /tmp/out.bin
```

- Options guide: [`options.md`](options.md)
- CLI guide: [`udtcat.md`](udtcat.md)
- Async: enable `udt-socket` feature `tokio-rt` → `async_socket::AsyncUdtSocket`


## Phase 2 handshake smoke

```sh
cargo test -p udt-socket localhost_three_way_handshake -- --nocapture
```

Server: `bind` → `listen` → `accept`. Client: `bind` (optional) → `connect(server_addr)`.
Both end in `ConnectionState::Connected` with peer socket IDs exchanged.
