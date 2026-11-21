# API page template (one page per function)

Copy this file to `docs/api/<crate-or-module>/<function_name>.md`.
Replace every `TODO`.

---

# `TODO_FUNCTION_NAME`

- **Crate:** `TODO_CRATE`
- **Module path:** `TODO_MODULE`
- **Stability:** experimental | unstable | stable
- **Since:** 0.1.0
- **Team owner:** A | B | C

## 1. Signature

```rust
// TODO: exact public signature
pub fn example() {}
```

## 2. Semantics

TODO: what the function does in one short paragraph. Include ownership
(moves / borrows) and whether it blocks or is async.

## 3. Preconditions / postconditions

**Preconditions**

- TODO

**Postconditions**

- TODO

## 4. Errors

| Error | When | Caller action |
| --- | --- | --- |
| TODO | TODO | TODO |

## 5. Thread-safety / async cancellation

- Thread-safe: TODO (yes/no + why)
- `Send` / `Sync`: TODO
- Async cancellation safety: TODO (if async)

## 6. Wire effects

- None | Sends packet type(s): TODO | Updates CC state: TODO

## 7. Compatibility

- C++ UDT4: TODO (equivalent call / note)
- `udt-rs`: TODO
- `tokio-udt`: TODO

## 8. Example

```rust
// TODO: compilable example (prefer `doctest` in rustdoc mirroring this)
```

## 9. See also

- [`TODO_RELATED`](./TODO_RELATED.md)
- Protocol notes: [`../../protocol/TODO.md`](../../protocol/TODO.md)
