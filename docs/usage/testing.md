# Testing guide

Four layers of tests. Prefer the cheapest layer that can catch the bug.
Phase mapping lives in [`docs/plan/TECH-ORDER.md`](../plan/TECH-ORDER.md).

## Summary

| Layer | What it proves | Default CI? | Where |
| --- | --- | --- | --- |
| **Unit** | Pure functions, state transitions, induced loss in-process | Yes | `crates/*/src/**` `#[cfg(test)]`, crate tests |
| **Conformance** | Wire bytes match UDT4 / draft-gg-udt-03 goldens | Yes (when corpus present) | `tests/conformance/` |
| **Soak** | Long transfers under loss/latency stay healthy | No (opt-in / ignored) | `tests/soak/`, `udt-interop` soak feature |
| **Interop** | Pure-Rust peers with C++ UDT4 (and self) | Phase 6+ job | `crates/udt-interop/`, matrix doc |

## Unit

Fast, no C++ link, no multi-hour runs. Cover codecs, state machine, loss list,
CC math stubs, and socket facade stubs.

```sh
cargo test --workspace
cargo test -p udt-core
cargo test -p udt-socket
```

Guidelines:

- Prefer table-driven cases and induced drop/reorder inside the process.
- Public helpers like `peek_type` should have round-trip and too-short cases.
- Do not require network privileges for unit tests.

## Conformance

Golden pcaps and raw `.bin` vectors under `tests/conformance/corpus/`. Decode
with the pure-Rust codec; classifications must match Wireshark UDT where
applicable.

```sh
# Layout and capture notes
cat tests/conformance/README.md

# Unit/codec tests that load goldens (as they are added)
cargo test -p udt-core
```

Capture guidelines (from the corpus README):

1. Wireshark with UDT dissector when possible.
2. Full handshake both directions (localhost + one lab path).
3. Export pcapng plus first-header `.bin` for cheap unit loads.
4. Record MTU, loss, and UDT4 version in `tests/conformance/notes/`.

Phase 0 exit expects ≥10 handshake captures or equivalent synthetic goldens.

## Soak

Long-running transfers with induced loss/latency. **Not** part of default
`cargo test`.

```sh
# Planned (Phase 6); ignored by default
cargo test -p udt-interop --features soak -- --ignored
```

Use soak for:

- ≥1 h stream under a documented loss/latency profile
- Memory / fd leak checks after many connect/shutdown cycles
- Regression after CC or reliability changes

See [`tests/soak/README.md`](../../tests/soak/README.md).

## Interop

Prove class-compatible behavior against vendored C++ UDT4 and pure-Rust peers.
Matrix: [`docs/plan/interop-matrix.md`](../plan/interop-matrix.md).

```sh
# Default features: no libudt link
cargo test -p udt-interop

# After submodule pin (ADR-0001):
# git submodule update --init crates/udt-interop/vendor/udt
# cargo test -p udt-interop --features cpp-udt4
```

Cells to fill (client → server):

| Client | Server | When |
| --- | --- | --- |
| PureRust | PureRust | From Phase 2 handshake onward |
| PureRust | CppUdt4 | Phase 6 |
| CppUdt4 | PureRust | Phase 6 |

Update the matrix with CI evidence or an N/A reason. “Works on my machine”
without a checked-in note does not satisfy DoD.

## Choosing a layer

| Symptom | Start with |
| --- | --- |
| Wrong header bits / endianness | Unit + conformance goldens |
| Handshake stuck / bad state | Unit state tests, then localhost interop |
| Throughput collapse under loss | Unit CC/reliability, then soak |
| Peer (C++ or other) rejects us | Interop matrix cell + pcap |

## Related docs

- Developer debugging (peeks, progress console): [`developer.md`](developer.md)
- Protocol packets: [`../protocol/packets.md`](../protocol/packets.md)
- Program DoD: [`../plan/DEFINITION-OF-COMPLETE.md`](../plan/DEFINITION-OF-COMPLETE.md)
