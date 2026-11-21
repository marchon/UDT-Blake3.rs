# udt-cli

udtcat CLI for pure-Rust UDT stream file transfers.

## Install

```sh
cargo build -p udt-cli --release
```

## Usage

```sh
# Listen and write to a file
cargo run -p udt-cli -- -l 9000 -o /tmp/out.bin

# Send a file
cargo run -p udt-cli -- 127.0.0.1:9000 -i /tmp/in.bin

# Tests
cargo test -p udt-cli
```

See [`docs/usage/udtcat.md`](../../docs/usage/udtcat.md).

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## License

BSD-3-Clause © UDT pure-Rust contributors
