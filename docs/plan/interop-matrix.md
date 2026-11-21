# Interop matrix

PureRust ↔ CppUdt4 cells for Phase 6. Update **Status** when a CI job
produces evidence; keep **CI job** / **Evidence path** aligned with
`crates/udt-interop` and workflow names.

| Client | Server | Status | CI job | Evidence path | Notes |
| --- | --- | --- | --- | --- | --- |
| PureRust | PureRust | **ok** | `ci` / local `run_interop_cell.sh rust-rust` | `artifacts/interop/rust-rust/` | Cookie HS + stream |
| PureRust | CppUdt4 | **ok** | `interop / rust-cpp` | `artifacts/interop/rust-cpp/` | Phase 6; vendored UDT4 `50638887` |
| CppUdt4 | PureRust | **ok** | `interop / rust-cpp` (cpp-rust step) | `artifacts/interop/cpp-rust/` | Phase 6 |
| CppUdt4 | CppUdt4 | N/A | — | — | Reference sanity only; not a product gate |

## Evidence rules

- A cell is **green** only when the named CI job has uploaded (or logged) proof
  under the evidence path (logs + `summary.txt` with `status=pass`).
- Record the job run URL or artifact SHA in the PR that flips Status.
- Print the planned in-crate matrix locally:

```sh
cargo run -p udt-interop --example print_matrix
./crates/udt-interop/scripts/build_vendor.sh
./crates/udt-interop/scripts/run_interop_cell.sh rust-cpp 65536
./crates/udt-interop/scripts/run_interop_cell.sh cpp-rust 16384
```

## Known gaps

- Cross-impl **ACK** shapes: PureRust `flush` may warn against C++ peers even
  when the peer received all bytes (byte-exact `cmp` still passes).
- Full ≥1 h soak is opt-in (`SOAK_SECONDS=3600`); CI runs a 30s smoke plus the
  `transfer_8mib_with_one_percent_loss` unit.
