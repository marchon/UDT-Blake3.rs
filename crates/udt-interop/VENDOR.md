# Vendoring C++ UDT4

Phase 6 interop links against a **vendored** copy of UDT4 under this crate.
System `libudt` is optional for local experiments only (ADR-0001).

**Do not enable the `cpp-udt4` feature until the submodule is present and the
Protocol lead has signed off on the pin.** Default crate builds stay pure Rust.

## Upstream

| Field | Value |
| --- | --- |
| Repository | https://github.com/eminence/udt |
| Default branch | `master` |
| Recommended pin | `50638887518f52dd6f6099050cfff75f9f7db08d` (2015-10-08, tip of `master` as of lookup; message: “First pass as some epoll exception issues”) |
| Tags | none published upstream — pin by **commit SHA**, not a tag |
| License | BSD (University of Illinois / UDT) |

Protocol lead may revise the SHA; update this table and `.gitmodules` together.

## Add the submodule (docs-only until Phase 6 CI)

From the repository root:

```sh
git submodule add https://github.com/eminence/udt crates/udt-interop/vendor/udt
cd crates/udt-interop/vendor/udt
git checkout 50638887518f52dd6f6099050cfff75f9f7db08d
cd -
git add crates/udt-interop/vendor/udt .gitmodules
git commit -m "vendor: pin eminence/udt at 50638887 for Phase 6 interop"
```

Clone / CI checkout with submodules:

```sh
git submodule update --init --recursive crates/udt-interop/vendor/udt
```

Expected layout:

```text
crates/udt-interop/vendor/udt/   # C++ UDT4 sources (submodule)
```

## Build notes (when enabling `cpp-udt4`)

1. Build the vendored tree with its own Makefile / CMake (platform-specific).
2. Point `cpp-udt4` / `build.rs` (not yet wired) at `vendor/udt` headers and the
   static/shared library artifact.
3. Keep workspace default features free of C++ so `cargo test --workspace`
   stays green without a C++ toolchain.

Until that wiring lands, treat this file as the pin + layout contract only.

## Build peers (Phase 6)

```sh
./crates/udt-interop/scripts/build_vendor.sh
# produces crates/udt-interop/cpp/stream_server and stream_client (static libudt.a)
./crates/udt-interop/scripts/run_interop_cell.sh rust-cpp 65536
./crates/udt-interop/scripts/run_interop_cell.sh cpp-rust 16384
```

On macOS Apple Silicon use `os=OSX arch=ARM_BASE` (avoid `-DAMD64` / `rdtsc`).
CI clones the pin when the submodule tree is absent (see `.github/workflows/interop.yml`).

## Out of scope

- Linking `libudt` into the Rust crates via `cxx` / `build.rs` (process-level peers are enough for matrix cells).
- System-package `libudt` as the CI source of truth.
