# udtd

Persistent multi-file UDT delivery daemon and client. Binary from crate `udt-cli`.

Each connection carries **one framed file** (the client opens a new connection
per file). The server listens forever and writes each file under an inbox
directory (basename only; collisions get a numeric suffix).

## Install

```sh
cargo build -p udt-cli --release
# binaries: target/release/udtd  target/release/udtcat
```

## Usage

```sh
# Terminal A — wait for deliveries
udtd serve -l 19000 -d /tmp/udt-inbox

# Terminal B — deliver one or more files
udtd put 127.0.0.1:19000 ./report.pdf ./data.bin
```

### Frame format (big-endian)

| Field | Size | Meaning |
| --- | --- | --- |
| `name_len` | u32 | Byte length of the file name |
| `name` | `name_len` | UTF-8 basename (path components stripped) |
| `file_len` | u64 | Exact byte length of the body |
| body | `file_len` | File contents |

## Contributing

Questions: open an issue. PRs welcome when tests and docs match `docs/plan/TECH-ORDER.md`.

## See also

- Raw stream CLI: [`udtcat.md`](udtcat.md)
- Options: [`options.md`](options.md)

## License

BSD-3-Clause © UDT pure-Rust contributors
