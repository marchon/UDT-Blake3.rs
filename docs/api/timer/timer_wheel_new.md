# `TimerWheel::new`

- **Crate:** `udt-core`
- **Module path:** `udt_core::timer`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
impl TimerWheel {
    pub fn new() -> Self
}
```

## 2. Semantics

Create an empty deadline wheel used by the reliability engine for RTO and
related timers.

## 3. Preconditions / postconditions

**Preconditions**

- None.

**Postconditions**

- Wheel has no pending deadlines.

## 4. Errors

None.

## 5. Thread-safety / async cancellation

Not `Sync`; own from one control path.

## 6. Wire effects

None.

## 7. Compatibility

Internal helper analogous to UDT timer scheduling.

## 8. Example

```rust
use udt_core::TimerWheel;
let _w = TimerWheel::new();
```

## 9. See also

- [`../../protocol/reliability.md`](../../protocol/reliability.md)
