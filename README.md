# UDT-Blake3.rs

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](LICENSE)

Pure-Rust UDT4 with Blake3 block seal + WANT/HAVE hybrid for faster WAN transfers.

Maps C++ UDT4 responsibilities onto Rust crates without linking `libudt` on the
default hot path, then adds an experimental Blake3 reliability hybrid that beat
both classic ACK/NAK UDT and SFTP on a measured ~4 GB WAN ISO path
(**6.79 MiB/s** vs **5.33** / **6.53**). Repository name *UDT-Blake3.rs*; workspace
crates remain `udt-core` / `udt-socket` / `udt-cli`.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Why Blake3](#why-blake3)
- [Benchmarks](#benchmarks)
- [Prebuilt binaries](#prebuilt-binaries)
- [Project layout](#project-layout)
- [Documentation](#documentation)
- [References](#references)
- [Contributing](#contributing)
- [License](#license)

## Background

[UDT](https://udt.sourceforge.io/) (UDP-based Data Transfer) is a reliable
application-level protocol for high bandwidth-delay-product WAN paths, designed
by Yunhong Gu and Robert L. Grossman. Existing Rust options either FFI to C++
(`udt-rs`) or partially reimplement the stack (`tokio-udt`).

This workspace is a **pure-Rust** UDT4-class core plus tools (`udtcat`, `udtd`).
On long WAN flows, classic ACK/NAK recovery left us under SFTP. The **Blake3
hybrid** keeps light ACK for congestion control, seals every 64 KiB, and uses
WANT/HAVE for gap pull and integrity — see [Why Blake3](#why-blake3).

## Install

```sh
# Rust 1.75+ (MSRV)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/marchon/UDT-Blake3.rs.git
cd UDT-Blake3.rs
cargo build -p udt-cli --release
cargo test -p udt-core --features blake3-reliability
./scripts/bench/localhost-smoke.sh
```

Classic ACK/NAK only (UDT4-oriented interop):

```sh
cargo build -p udt-cli --release --no-default-features
```

## Usage

```sh
# Listen / send a file (Blake3 hybrid is the udt-cli default)
cargo run -p udt-cli --release --bin udtcat -- -l 9000 -o /tmp/out.bin
cargo run -p udt-cli --release --bin udtcat -- 127.0.0.1:9000 -i /tmp/in.bin

# Persistent multi-file inbox
cargo run -p udt-cli --release --bin udtd -- serve -l 19000 -d /tmp/udt-inbox
cargo run -p udt-cli --release --bin udtd -- put 127.0.0.1:19000 ./file.bin

# Short WAN probe (build example with Blake3)
cargo run -p udt-socket --release --example wan_probe --features blake3-reliability -- \
  HOST:19000 8388608
```

Both ends of a transfer must use the same reliability mode.

## Why Blake3

Classic UDT ACK/NAK on our WAN path asymptoted near **5.3 MiB/s** while SFTP
held **~6.5 MiB/s**. NAK storms and reverse-path ACK load interacted badly with
pacing and capped rexmit. The hybrid keeps ACK for UDT4 CC, replaces NAK gap
pull with sealed-block WANT bitmaps, and verifies each 64 KiB with Blake3.

Full rationale: [`docs/blake3/why-blake3.md`](docs/blake3/why-blake3.md).  
Wire format: [`docs/blake3/wire-format.md`](docs/blake3/wire-format.md).  
Design sketch: [`docs/plan/blake3-reliability-sketch.md`](docs/plan/blake3-reliability-sketch.md).

## Benchmarks

Same-day sequential runs, local macOS → `70.88.205.138:19000`, MSS 1400:

| Method | ~4 GB ISO | Rate |
| --- | --- | --- |
| UDT ACK/NAK | 743.22 s | 5.331 MiB/s |
| **UDT Blake3** | **583.41 s** | **6.792 MiB/s** |
| SFTP | 606.86 s | 6.529 MiB/s |

1 GiB probe: Blake3 **7.435** vs baseline **5.376 MiB/s**.  
Complete tables, method, and raw logs:
[`docs/benchmarks/WAN-RESULTS.md`](docs/benchmarks/WAN-RESULTS.md).  
Reproduce: [`scripts/bench/`](scripts/bench/README.md).

## Prebuilt binaries

| Platform | Path |
| --- | --- |
| macOS Apple Silicon | [`dist/darwin-aarch64/`](dist/darwin-aarch64/) |
| Linux x86_64 | [`dist/linux-x86_64/`](dist/linux-x86_64/) |

Checksums: [`dist/SHA256SUMS.txt`](dist/SHA256SUMS.txt). Also attached to GitHub
Releases. Details: [`dist/README.md`](dist/README.md).

## Project layout

| Path | Purpose |
| --- | --- |
| `crates/udt-core` | Packets, state, reliability, CC, Blake3 module |
| `crates/udt-socket` | Sync + Tokio facades; `wan_probe` example |
| `crates/udt-cli` | `udtcat` + `udtd` (Blake3 default) |
| `crates/udt-interop` | C++ UDT4 matrix harness |
| `docs/blake3` | Why Blake3 + wire format |
| `docs/benchmarks` | WAN results + raw logs |
| `docs/references` | Papers and other implementations |
| `scripts/bench` | Smoke / probe / ISO helpers |
| `dist/` | Prebuilt test binaries |

## Documentation

- Why Blake3: [`docs/blake3/why-blake3.md`](docs/blake3/why-blake3.md)
- WAN results: [`docs/benchmarks/WAN-RESULTS.md`](docs/benchmarks/WAN-RESULTS.md)
- Papers & implementations: [`docs/references/papers-and-implementations.md`](docs/references/papers-and-implementations.md)
- Research plan archive: [`docs/plan/SOURCE-RESEARCH.md`](docs/plan/SOURCE-RESEARCH.md)
- Protocol: [`docs/protocol/`](docs/protocol/)
- Usage: [`docs/usage/udtcat.md`](docs/usage/udtcat.md), [`docs/usage/udtd.md`](docs/usage/udtd.md)
- Testing: [`docs/usage/testing.md`](docs/usage/testing.md)

## References

Primary: Gu & Grossman, *Computer Networks* 2007; `draft-gg-udt-03`; UDT4 C++ at
[eminence/udt](https://github.com/eminence/udt). Full list with links:
[`docs/references/papers-and-implementations.md`](docs/references/papers-and-implementations.md).

## Contributing

Questions: open an issue on
[github.com/marchon/UDT-Blake3.rs](https://github.com/marchon/UDT-Blake3.rs).
PRs welcome when they include tests for touched behavior and, for public APIs, a
page under `docs/api/`. Follow `docs/plan/TECH-ORDER.md` for protocol phases.
Both ends of Blake3 transfers must enable the feature (CLI default).

## License

[BSD-3-Clause](LICENSE) © UDT pure-Rust contributors

Third-party UDT4 attribution (optional vendor tree): see [`NOTICE`](NOTICE) and
[`crates/udt-interop/VENDOR.md`](crates/udt-interop/VENDOR.md).
