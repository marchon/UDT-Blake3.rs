# udtcat

Stream file transfer over pure-Rust UDT (Phase 5). Binary from crate `udt-cli`.

## Install

```sh
cargo build -p udt-cli --release
# binary: target/release/udtcat
```

## Usage

```sh
# Listener: write received bytes to a file (or stdout if -o omitted)
udtcat -l [host:]port [-o outfile]

# Sender: send a file (or stdin if -i omitted)
udtcat host:port [-i infile]

udtcat -h | --help
```

Address forms: `9000`, `:9000`, `127.0.0.1:9000`, or any resolvable `host:port`.
Bare port numbers bind/connect on `127.0.0.1`.

### Localhost round-trip

```sh
# Terminal A
cargo run -p udt-cli -- -l 9000 -o /tmp/out.bin

# Terminal B
dd if=/dev/urandom of=/tmp/in.bin bs=1024 count=256
cargo run -p udt-cli -- 127.0.0.1:9000 -i /tmp/in.bin
cmp /tmp/in.bin /tmp/out.bin && echo OK
```

Listener stops after ~5 s of idle after the last received byte (sender `flush` +
`shutdown`). Stderr prints listen/accept/connect and byte counts.

### Library helpers

`udt_cli` also exposes `parse_addr`, `listen_and_recv`, `connect_and_send`,
`open_input`, and `open_output` for embedding the same path in tests or tools.

```sh
cargo test -p udt-cli localhost_file_round_trip
```

## Options used by the CLI

| Option | Value | Why |
| --- | --- | --- |
| `RcvTimeo` | 20 ms | Short UDP poll while pumping ACK/NAK |
| `SndTimeo` | 120 s (sender) | Bound `flush` wait on large files |

See [`options.md`](options.md) for the full `setsockopt` / `getsockopt` map.

## Limits (Phase 5)

- One accepted client per listen invocation
- Stream mode only (no datagram messaging CLI)
- Rendezvous connect flag is stored but not wired on the handshake path yet
- Idle end-of-stream detection (no explicit FIN packet yet)

## See also

- Options: [`options.md`](options.md)
- Socket API: [`../api/socket/`](../api/socket/)
- Tech order Phase 5: [`../plan/TECH-ORDER.md`](../plan/TECH-ORDER.md)
