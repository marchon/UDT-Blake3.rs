# udt-socket

Sync and Tokio async UDT socket facades.

## Install

```sh
cargo build -p udt-socket
# Optional Blake3 hybrid reliability (also default in udt-cli):
cargo build -p udt-socket --features blake3-reliability
```

## Usage

```sh
cargo test -p udt-socket
cargo test -p udt-socket --features blake3-reliability
```

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## License

BSD-3-Clause © Marchon
