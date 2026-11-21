# udt-core

Pure-Rust UDT protocol core.

## Install

```sh
cargo build -p udt-core
```

## Usage

```sh
cargo test -p udt-core
# Hybrid Blake3 64KiB seal + WANT/HAVE (wired into ReliabilityEngine when on):
cargo test -p udt-core --features blake3-reliability
```

`udt-cli` enables `blake3-reliability` by default; `udt-core` / `udt-socket`
keep it opt-in.

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## License

BSD-3-Clause © UDT pure-Rust contributors
