# Soak tests (Phase 6)

Long-running PureRust transfers. Default CI runs a **smoke** (~30s) plus the
`transfer_8mib_with_one_percent_loss` unit test. Full hour:

```sh
SOAK_SECONDS=3600 ./crates/udt-interop/scripts/run_soak_smoke.sh
```

Evidence lands in `artifacts/soak/summary.txt`.
