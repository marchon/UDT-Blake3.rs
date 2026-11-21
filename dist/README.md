# Prebuilt binaries for testing

Smoke-test `udtcat` / `udtd` / `wan_probe` without a full toolchain. Prefer
building from source for production (`cargo build -p udt-cli --release`).

## Install

```sh
# macOS Apple Silicon
chmod +x dist/darwin-aarch64/*
./dist/darwin-aarch64/udtcat -h

# Linux x86_64
chmod +x dist/linux-x86_64/*
./dist/linux-x86_64/udtcat -h

# Verify
shasum -a 256 -c dist/SHA256SUMS.txt
```

## Usage

```sh
# Terminal A
./dist/<triple>/udtcat -l 0.0.0.0:19000 -o /tmp/out.bin

# Terminal B
./dist/<triple>/udtcat 127.0.0.1:19000 -i /tmp/in.bin

# Short probe (needs a listening peer)
./dist/<triple>/wan_probe 127.0.0.1:19000 8388608
```

These builds include **Blake3 hybrid** (same as `udt-cli` default features).
For classic ACK/NAK, build with `--no-default-features`.

## Contents

| Triple | Binaries |
| --- | --- |
| `darwin-aarch64` | `udtcat`, `udtd`, `wan_probe` |
| `linux-x86_64` | `udtcat`, `udtd`, `wan_probe` |

Built from the commit tagged on the matching GitHub Release.

## Contributing

Report binary issues on the GitHub tracker. Do not commit unrelated large
artifacts; refresh `dist/` only with release scripts.

## License

BSD-3-Clause © UDT pure-Rust contributors
