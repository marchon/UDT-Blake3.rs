# udt-core

Pure-Rust UDT protocol core.

## Install

```sh
cargo build -p udt-core
```

## Usage

```sh
cargo test -p udt-core
# Optional hybrid Blake3 64KiB seal + WANT/HAVE pull recovery:
cargo test -p udt-core --features blake3-reliability
```

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## License

BSD-3-Clause © UDT pure-Rust contributors
