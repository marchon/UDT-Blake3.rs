# UDT-Blake3.rs

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](LICENSE)
[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Release](https://img.shields.io/github/v/release/marchon/UDT-Blake3.rs)](https://github.com/marchon/UDT-Blake3.rs/releases)

Pure-Rust UDT4 rewrite with Blake3 hybrid reliability for high-BDP WAN transfers.

**This is not an FFI wrapper.** `udtcat`, `udtd`, and the library crates are
implemented in Rust (`udt-core` / `udt-socket` / `udt-cli`). Default builds do
**not** link `libudt` or any C++ UDT library. (That is what
[udt-rs](https://github.com/eminence/udt-rs) does; this repo does the opposite.)

The Blake3 hybrid (64 KiB seals + WANT/HAVE) is the measured WAN default in the
CLI. On a ~4 GB ISO path it reached **6.79 MiB/s** vs **5.33 MiB/s** classic
ACK/NAK UDT and **6.53 MiB/s** SFTP.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Pure Rust (no libudt)](#pure-rust-no-libudt)
- [Why Blake3](#why-blake3)
- [Benchmarks](#benchmarks)
- [Prebuilt binaries](#prebuilt-binaries)
- [Project layout](#project-layout)
- [Documentation](#documentation)
- [Reference peer (optional)](#reference-peer-optional)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Background

[UDT](https://udt.sourceforge.io/) (UDP-based Data Transfer) is a reliable
UDP application protocol for high bandwidth-delay-product networks, designed by
Yunhong Gu and Robert L. Grossman. This project reimplements UDT4-class
behavior in pure Rust, then adds an experimental Blake3 reliability path for
long WAN flows where classic ACK/NAK asymptoted under SFTP.

See [`docs/references/papers-and-implementations.md`](docs/references/papers-and-implementations.md)
for papers and other implementations.

## Install

```sh
# Requires Rust 1.75+ (MSRV)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/marchon/UDT-Blake3.rs.git
cd UDT-Blake3.rs
cargo build -p udt-cli --release
./scripts/bench/localhost-smoke.sh
```

Classic ACK/NAK only (no Blake3 extension):

```sh
cargo build -p udt-cli --release --no-default-features
```

## Usage

```sh
# File listen / send (Blake3 hybrid is on by default in udt-cli)
./target/release/udtcat -l 9000 -o /tmp/out.bin
./target/release/udtcat 127.0.0.1:9000 -i /tmp/in.bin

# Multi-file inbox daemon
./target/release/udtd serve -l 19000 -d /tmp/udt-inbox
./target/release/udtd put 127.0.0.1:19000 ./file.bin

# Short WAN probe
cargo run -p udt-socket --release --example wan_probe --features blake3-reliability -- \
  HOST:19000 8388608
```

Both ends of a transfer must use the same reliability mode (Blake3 default, or
both built with `--no-default-features`).

## Pure Rust (no libudt)

| Crate | Role | Links C++ UDT? |
| --- | --- | --- |
| `udt-core` | Packets, handshake, ACK/NAK, UDT4 CC, Blake3 module | No |
| `udt-socket` | Sync + Tokio socket API | No |
| `udt-cli` | `udtcat` / `udtd` | No |

Default workspace members are those three crates only. Verify a release binary:

```sh
# macOS
otool -L target/release/udtcat
# Expect system libs only — no libudt
```

## Why Blake3

On the measured WAN path, classic ACK/NAK UDT settled near **5.3 MiB/s** while
SFTP held **~6.5 MiB/s**. NAK storms and reverse-path ACK load fought pacing.
The hybrid keeps light ACK for congestion control, seals every 64 KiB with
Blake3, and uses WANT/HAVE for gap pull and integrity.

Full write-up: [`docs/blake3/why-blake3.md`](docs/blake3/why-blake3.md).

## Benchmarks

Sequential runs, local macOS → `70.88.205.138:19000`, MSS 1400, ~4 GB Ubuntu ISO:

| Method | Wall time | Rate |
| --- | --- | --- |
| UDT ACK/NAK (pure Rust) | 743 s | 5.331 MiB/s |
| **UDT Blake3 (pure Rust)** | **583 s** | **6.792 MiB/s** |
| SFTP | 607 s | 6.529 MiB/s |

Tables, method, and raw logs:
[`docs/benchmarks/WAN-RESULTS.md`](docs/benchmarks/WAN-RESULTS.md).  
Reproduce: [`scripts/bench/`](scripts/bench/README.md).

## Prebuilt binaries

| Platform | Location |
| --- | --- |
| macOS Apple Silicon | [`dist/darwin-aarch64/`](dist/darwin-aarch64/) |
| Linux x86_64 | [`dist/linux-x86_64/`](dist/linux-x86_64/) |

Checksums: [`dist/SHA256SUMS.txt`](dist/SHA256SUMS.txt). Same files on the
[v0.2.0 release](https://github.com/marchon/UDT-Blake3.rs/releases/tag/v0.2.0).

## Project layout

| Path | Purpose |
| --- | --- |
| `crates/udt-core` | Pure-Rust protocol core |
| `crates/udt-socket` | Socket API + `wan_probe` |
| `crates/udt-cli` | `udtcat` + `udtd` |
| `docs/blake3` | Blake3 design + wire format |
| `docs/benchmarks` | WAN results + logs |
| `scripts/bench` | Smoke / probe / ISO helpers |
| `dist/` | Prebuilt test binaries |

## Documentation

- [Why Blake3](docs/blake3/why-blake3.md)
- [WAN results](docs/benchmarks/WAN-RESULTS.md)
- [Papers & implementations](docs/references/papers-and-implementations.md)
- [udtcat](docs/usage/udtcat.md) · [udtd](docs/usage/udtd.md) · [protocol](docs/protocol/)

## Reference peer (optional)

`crates/udt-interop` is an **optional** harness that can build vendored C++ UDT4
peers to test wire compatibility (PureRust ↔ CppUdt4). It is **not** a
dependency of `udt-cli`, is **not** a default workspace member, and is **not**
required to build, test, or run transfers.

```sh
# Only if you want interop matrix cells:
cargo build -p udt-interop
./crates/udt-interop/scripts/build_vendor.sh
```

## Maintainers

- [Marchon](https://github.com/marchon) — Marchon@gmail.com

## Contributing

Questions and bugs: [GitHub Issues](https://github.com/marchon/UDT-Blake3.rs/issues).
PRs are welcome when they include tests for the behavior you change. Protocol
work should follow [`docs/plan/TECH-ORDER.md`](docs/plan/TECH-ORDER.md).

## License

[BSD-3-Clause](LICENSE) © Marchon

Optional vendored UDT4 (interop only): see [`NOTICE`](NOTICE).
