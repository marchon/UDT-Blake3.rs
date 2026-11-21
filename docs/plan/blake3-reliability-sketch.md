# Blake3 realtime validation vs ACK/NAK — design sketch

**Status:** wired + WAN A/B measured; **`udt-cli` default-on**, core/socket opt-in  
**Context:** WAN ISO path local→70.88.205.138; UDT **5.45 MiB/s**, SFTP **6.51 MiB/s** (commit `cf4874a`).  
**Question:** Can Blake3 realtime validation replace ACK/NAK, with missed units re-requested and rebuilt?

## Prototype (this tree)

- Crate feature: `udt-core/blake3-reliability` → module [`blake3_block`](../../crates/udt-core/src/blake3_block.rs); also `udt-socket` / `udt-cli` features of the same name
- Wire: `BlockSeal` `0x10`, `Want` `0x11`, `Have` `0x12` (experimental; not UDT4)
- APIs: `Blake3BlockSender` / `Blake3BlockReceiver` — seal every 64 KiB, WANT bitmap gaps, HAVE after Blake3 verify
- **Wired** into `ReliabilityEngine` + socket `dispatch` when the feature is on: light ACK kept for CC/delivery; NAK suppressed; WANT after seal for gap pull; HAVE frees sender block store; orphan buffer avoids full-block re-WANT for data that arrived before the seal
- Enable tests: `cargo test -p udt-core --features blake3-reliability`
- Build probe path: `cargo build -p udt-cli --release --features blake3-reliability` (both ends must match)

## WAN A/B (local → 70.88.205.138:19000, `wan_probe`, MSS 1400, 16 MiB buffers)

| Build | Size | Run | Rate (MiB/s) | Final period (µs) | Elapsed |
| --- | --- | --- | --- | --- | --- |
| baseline (ACK/NAK) | 8 MiB | 1 | **4.257** | 161 | 1.88 s |
| blake3 hybrid | 8 MiB | 1 | **3.813** | 155 | 2.10 s |
| baseline | 32 MiB | 1 | **5.523** | 152 | 5.79 s |
| baseline | 32 MiB | 2 | **4.949** | 180 | 6.47 s |
| blake3 hybrid | 32 MiB | 1 | **5.717** | 111 | 5.60 s |
| blake3 hybrid | 32 MiB | 2 | **5.673** | 108 | 5.64 s |
| baseline | **1 GiB** | 1 | **5.376** | 173 | 190.5 s |
| blake3 hybrid | **1 GiB** | 1 | **7.435** | 112 | 137.7 s |
| baseline | **~4 GB ISO** | 1 | **5.331** | — | 743.2 s |
| blake3 hybrid | **~4 GB ISO** | 1 | **6.792** | — | 583.4 s |
| SFTP (same day) | **~4 GB ISO** | 1 | **6.529** | — | 606.9 s |

Logs: `artifacts/bench-iso/blake3-ab-*.log`, `blake3-iso-*.log`, `COMPARISON-4GB-blake3.md`.

**Read:**

- Short (8 MiB): Blake3 ~10% slower (seal/WANT startup tax).
- Medium (32 MiB): flat-to-slight Blake3 edge; noisy.
- **Long (1 GiB): Blake3 +38%** (7.435 vs 5.376 MiB/s).
- **Full ISO: Blake3 6.792 MiB/s vs baseline 5.331 (+27%) and vs SFTP 6.529 (+4%)** — first UDT win over SFTP on this path.
- Hypothesis “cleaner WANT / fewer NAK storms helps long flow” is **supported**.
- **`udt-cli` default features now enable `blake3-reliability`** (both ends must match). `udt-core` / `udt-socket` keep the feature opt-in; build CLI with `--no-default-features` for classic ACK/NAK.

## 1. What “Blake3 instead of ACK/NAK” can mean

Three distinct designs are easy to conflate:

| Mode | Delivery proof | Loss signal | Integrity |
| --- | --- | --- | --- |
| **A. Today (UDT4)** | Cumulative ACK | NAK + RTO | UDP checksum only |
| **B. Hybrid** | Light ACK / block ACK | NAK or hash-miss pull | Blake3 per block |
| **C. Pull-only content-addressed** | Implicit when block hash matches | Receiver `WANT(block_id)` | Blake3 is the contract |

The interesting ask is closest to **C** (or **B** as a stepping stone): the receiver continuously validates chunks; anything missing or wrong is **re-requested by id/hash** and spliced back into the stream (“rebuilt”), rather than relying on seq-based NAK lists and cumulative ACK clocks.

## 2. Proposed shape (hybrid → pull)

### Wire unit

- Stream split into **blocks** of fixed payload size (e.g. 64–256 KiB), each an integer number of MSS packets.
- Each block carries:
  - `block_id` (monotonic)
  - `blake3_32` (32-byte digest of plaintext payload)
  - optional packet-level `seq` for ordering inside the block (or encode offset in header)

Blake3 of a 64 KiB block is cheap on modern CPUs (typically multi-GB/s), so **realtime** hashing on send and receive is realistic at 5–10 MiB/s and well beyond.

### Receiver

1. Reassemble packets into block `i`.
2. If a packet gap remains past a short timer → emit **`WANT(block_id, bitmap|ranges)`** (pull), not a classic NAK storm.
3. When the block is complete → Blake3; on mismatch → `WANT(block_id)` full block (corruption / bit-rot path).
4. Deliver contiguous prefix of verified blocks to the app (same stream semantics as today).
5. Optionally send **`HAVE(block_id)`** or a sparse bitmap infrequently (e.g. every N blocks) so the sender can release buffer — this is a **coarse ACK**, not per-packet ACK.

### Sender

1. Keep a **block store** (payload + digest) until `HAVE` advances.
2. Pace new blocks with existing CC (CWND/SND) — CC still needs *some* delivery/loss signal.
3. On `WANT`, retransmit only the requested ranges (rebuild). Do **not** stall new-block transmit solely because a WANT is in flight (same lesson as “don’t wait for NAKs”).

## 3. What Blake3 does *not* replace by itself

- **Congestion control** still needs rate/loss feedback (recv_rate, delay, or explicit loss). Hash OK ≠ “path is not congested.”
- **Ordering / stream API** still needs a contiguous delivery cursor; hashes alone do not order bytes.
- **Liveness** still needs a timer if the reverse path is silent (WANT/HAVE loss).

So Blake3 is a strong **integrity + naming** layer for selective retransmit. It is a weak standalone replacement for the entire ACK clock unless paired with pull timers and coarse HAVE.

## 4. Why it might beat ACK/NAK on this path

Current long-flow UDT asymptote ≈ **5.45 MiB/s** (~245 µs SND at MSS 1400). Reverse-path cost today:

- Light ACK essentially every data packet
- NAK bursts on gaps; RTO prefix rexmit

Blake3/pull benefits:

1. **Fewer reverse packets** — HAVE every 64–256 KiB instead of ACK every ~1.4 KiB (~50–180× less ACK traffic).
2. **Cleaner loss API** — `WANT` names durable blocks; avoids NAK list truncation and HOL stalls from “oldest-only” vs “blast all.”
3. **Corruption coverage** — detects silent payload damage UDP checksums miss.
4. **Rebuild semantics** — receiver-driven repair matches “re-requested and rebuilt” and parallelizes with forward send.

Risks:

1. **Block size vs latency** — large blocks delay app delivery and inflate rexmit cost on a single loss.
2. **Protocol break** — not UDT4 wire-compatible; needs a version bit / new sock type.
3. **CPU** — negligible at 10 MiB/s; matters if we later aim at multi-hundred MiB/s.
4. **CC coupling** — must feed loss events from WANT timeouts into `on_loss` or delay-based CC.

## 5. Feasibility vs current stack

| Piece | Effort | Notes |
| --- | --- | --- |
| `blake3` crate on send/recv block seal | S | Pure Rust, easy |
| Block framing in `udt-core::wire` | M | New control types: HAVE / WANT |
| Receiver block reassembly + pull timer | M | Parallel to `LossList` |
| Sender block store + selective rexmit | M | Replaces much of seq NAK path |
| Keep Udt4Cc with WANT→`on_loss` | S | Reuse pacing wins |
| Interop with stock UDT4 | — | **Not compatible** — feature-flag / new mode |

Suggested first prototype (**mode B**): keep packet seq + light ACK for CC, add Blake3 over 64 KiB blocks, replace NAK with WANT-bitmap for gaps inside the open block. Measure reverse-path pps and 32 MiB / 4 GB rates vs `cf4874a`.

## 6. Expected results (hypothesis)

- **Integrity:** catch corruption the current path cannot.
- **Throughput:** possible **partial** close of the 1.06 MiB/s SFTP gap if ACK PPS / interrupt overhead is material; **unlikely** to win alone if the limiter is SND equilibrium from loss×1.02 and path capacity.
- **Tail latency / stalls:** better than aggressive NAK/RTO interactions if WANT is capped and forward send continues (already learned on this codebase).

## 7. Recommended next step

1. Keep shipping ACK/NAK CC wins (persistent non-blocking pump, gentler loss).
2. If prototyping: **64 KiB Blake3 blocks + WANT bitmap + coarse HAVE**, feature `blake3-reliability`, off by default.
3. Bench: 32 MiB ×3 and one 1 GB slice before another full 4 GB ISO.

## 8. Non-goals for v1 sketch

- FEC / fountain codes (orthogonal “rebuild” strategy)
- Replacing UDP with QUIC
- Dropping sequence numbers entirely in the first prototype
