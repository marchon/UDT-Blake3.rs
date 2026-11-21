# Bench scripts

Reproduce localhost smoke tests and WAN probes for UDT-Blake3.rs.

## Install

```sh
# From repo root — build tools used by the scripts
cargo build -p udt-cli --release
cargo build -p udt-socket --example wan_probe --release --features blake3-reliability
# Or use prebuilt binaries from dist/ / GitHub Releases (see dist/README.md)
```

## Usage

```sh
# Localhost round-trip (Blake3 default udtcat)
./scripts/bench/localhost-smoke.sh

# WAN probe (server must already listen with matching udtcat)
./scripts/bench/wan-probe.sh 70.88.205.138 8388608

# Full-file timed send (ISO or any file)
./scripts/bench/wan-iso.sh 70.88.205.138 /path/to/file.bin
```

Server listener example (on Linux x86_64 host):

```sh
./udtcat -l 0.0.0.0:19000 -o /tmp/recv.bin
# pid-file pattern used in our benches:
# echo $! > udt.pid
```

Classic ACK/NAK A/B (both ends):

```sh
cargo build -p udt-cli --release --no-default-features
```

## Contributing

Questions: open a GitHub issue. PRs that add bench logs should place them under
`docs/benchmarks/results/` with a one-line note in `docs/benchmarks/WAN-RESULTS.md`.

## License

BSD-3-Clause © Marchon
