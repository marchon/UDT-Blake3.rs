# `FixedCc::new`

- **Crate:** `udt-core`
- **Module path:** `udt_core::cc`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl FixedCc {
    pub fn new(cwnd_pkts: u32, period_us: u64) -> Self
}
```

## 2. Semantics

Construct a test congestion controller with a fixed CWND and send period.
Ignores ACK/loss/timeout for pacing math; used via
`ReliabilityEngine::with_cc`.

## 3. Preconditions / postconditions

**Preconditions**

- `cwnd_pkts >= 1` recommended.

**Postconditions**

- `cwnd_pkts()` / `sending_period_us()` return the configured values.

## 4. Errors

None.

## 5. Thread-safety / async cancellation

`Send`; not shared mutably across threads without sync.

## 6. Wire effects

None directly; gates enqueue/dispatch when installed on an engine.

## 7. Compatibility

Test double; no C++ equivalent required.

## 8. Example

```rust
use udt_core::{FixedCc, ReliabilityEngine};
// let eng = ReliabilityEngine::with_cc(..., Box::new(FixedCc::new(32, 10)));
```

## 9. See also

- [`udt4cc_new.md`](udt4cc_new.md)
- [`../../protocol/congestion.md`](../../protocol/congestion.md)
