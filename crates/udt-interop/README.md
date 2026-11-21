# udt-interop

C++ UDT4 interop harness (Phase 6): matrix helpers, vendored UDT4 peers, soak/bench scripts.

## Install

```sh
cargo build -p udt-interop
./crates/udt-interop/scripts/build_vendor.sh
```

## Usage

```sh
cargo test -p udt-interop
cargo run -p udt-interop --example print_matrix
./crates/udt-interop/scripts/run_interop_cell.sh rust-cpp 65536
./crates/udt-interop/scripts/run_interop_cell.sh cpp-rust 16384
SOAK_SECONDS=30 ./crates/udt-interop/scripts/run_soak_smoke.sh
./crates/udt-interop/scripts/run_bench.sh
```

Default features do **not** link `libudt`. Thin C++ `stream_server` / `stream_client`
binaries link the vendored static library (see [`VENDOR.md`](VENDOR.md)). Upstream UDT
remains BSD (University of Illinois).

## Contributing

Questions: open an issue. PRs welcome for matrix cells with CI evidence under
`artifacts/interop/`.

## License

BSD-3-Clause © UDT pure-Rust contributors. See [`LICENSE`](../../LICENSE) and
[`NOTICE`](../../NOTICE) for vendored UDT4 attribution.
