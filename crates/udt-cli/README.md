# udt-cli

udtcat and udtd CLIs for pure-Rust UDT stream and framed file transfers.

## Install

```sh
cargo build -p udt-cli --release
# Classic ACK/NAK only (disable measured Blake3 hybrid default):
cargo build -p udt-cli --release --no-default-features
```

## Usage

```sh
# Raw stream listen / send (udtcat)
cargo run -p udt-cli --bin udtcat -- -l 9000 -o /tmp/out.bin
cargo run -p udt-cli --bin udtcat -- 127.0.0.1:9000 -i /tmp/in.bin

# Persistent multi-file inbox (udtd)
cargo run -p udt-cli --bin udtd -- serve -l 19000 -d /tmp/udt-inbox
cargo run -p udt-cli --bin udtd -- put 127.0.0.1:19000 ./file.bin

# Tests
cargo test -p udt-cli
```

Default features enable the Blake3 64 KiB seal + WANT/HAVE hybrid (WAN ISO
~6.8 MiB/s vs ~5.3 MiB/s ACK/NAK and ~6.5 MiB/s SFTP on the 138 path). Both
ends of a transfer must use the same reliability mode.

See [`docs/usage/udtcat.md`](../../docs/usage/udtcat.md) and
[`docs/usage/udtd.md`](../../docs/usage/udtd.md).

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## License

BSD-3-Clause © UDT pure-Rust contributors
