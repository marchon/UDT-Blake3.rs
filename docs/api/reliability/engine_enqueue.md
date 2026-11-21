# `ReliabilityEngine::enqueue_app_data`

- **Crate:** `udt-core`
- **Module path:** `udt_core::reliability`
- **Stability:** experimental
- **Since:** 0.1.0
- **Team owner:** A

## 1. Signature

```rust
pub fn enqueue_app_data(&mut self, data: &[u8], now: Instant, timestamp: u32) -> Vec<Outgoing>
```

## 2. Semantics

Segments application bytes into MSS-sized data packets, stores them in the
unacked send buffer, and returns `Outgoing::Data` messages to put on the wire.

## 3. Preconditions / postconditions

**Preconditions**

- Engine created after handshake with matching init seqs.

**Postconditions**

- `inflight_packets()` increases; RTO armed.

## 4. Errors

None (returns packets by value).

## 5. Thread-safety / async cancellation

Not thread-safe; single owner.

## 6. Wire effects

Caller must send returned `Outgoing` datagrams.

## 7. Compatibility

Analogous to UDT4 sender buffer enqueue.

## 8. Example

See `reliability` unit tests and `UdtSocket::send`.

## 9. See also

- [`../../protocol/reliability.md`](../../protocol/reliability.md)
