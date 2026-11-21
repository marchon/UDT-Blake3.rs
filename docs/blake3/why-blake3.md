# Why Blake3 hybrid reliability improved WAN throughput

## The problem we measured

On a real WAN path (laptop → `70.88.205.138:19000`), a pure-Rust UDT4-class
stack with classic **ACK + NAK** reliability reached:

| Milestone | ~4 GB ISO rate | vs SFTP |
| --- | --- | --- |
| Early (CWND pinned / microbursts) | ~0.54 MiB/s | far behind |
| After pacing, OS buffers, gentler CC (`cf4874a`) | **5.45 MiB/s** | 83.7% of SFTP 6.51 |
| Same-day baseline recheck (ACK/NAK) | **5.33 MiB/s** | 81.6% of SFTP 6.53 |

Short probes (8–32 MiB) looked healthy; the long-flow asymptote still sat under
SFTP. SND period settled around ~150–180 µs. Further CWND tricks either stalled
or regressed. The remaining gap looked like **loss-recovery / reverse-path
cost**, not “open the window harder.”

## What classic UDT does on loss

Per Gu & Grossman and `draft-gg-udt-03`:

1. Receiver emits **NAK** lists of missing sequence numbers (often promptly).
2. Sender retransmits; **RTO** covers silent loss.
3. Receiver emits frequent **ACKs** (light + periodic full) for delivery and CC.

On our path that produced:

- High reverse-path packet rate (ACK nearly per data packet).
- NAK bursts that either **microburst** the forward path (full-list rexmit) or
  **stall** under head-of-line caps (oldest-only / small NAK rexmit caps).
- CC `on_loss` reactions that raise SND period even when the hole was already
  being repaired.

## What we changed (Blake3 hybrid)

Feature `blake3-reliability` (default in `udt-cli`):

| Piece | Role |
| --- | --- |
| **Light / full ACK** | Still drives delivery cursor + UDT4 CC (recv_rate, RTT, CWND/SND). |
| **64 KiB Blake3 BlockSeal** (`0x10`) | Sender announces digest + exact packet count after each block. |
| **WANT bitmap** (`0x11`) | Receiver pulls missing slots **after** the seal (and orphan buffer so pre-seal data is not re-WANTed wholesale). |
| **HAVE** (`0x12`) | Receiver confirms Blake3; sender frees the block store. |
| **NAK suppressed** | Gap pull is WANT-centric once seals exist; RTO remains a backstop. |

Design sketch: [`docs/plan/blake3-reliability-sketch.md`](../plan/blake3-reliability-sketch.md).  
Wire helpers: `crates/udt-core/src/blake3_block.rs`.

## Why that helps throughput (causal story)

1. **Cleaner loss API.** WANT names a durable block + bitmap instead of racing
   growing NAK lists against encode-size caps and HOL rexmit policy. Forward
   send continues while a capped WANT rexmit (16 packets) runs — same lesson as
   “don’t stall new data waiting for NAKs,” with less storm behavior.

2. **Fewer pathological CC kicks.** NAK storms fed `on_loss` repeatedly on the
   same hole. WANT batches missing slots per sealed block; loss signals are
   coarser and aligned with repair. Measured long-flow SND period improved
   (1 GiB: final ~112 µs Blake3 vs ~173 µs baseline).

3. **Integrity that UDP checksums miss.** Blake3 over 64 KiB plaintext catches
   silent corruption; mismatch triggers a full-block WANT rebuild.

4. **What Blake3 does *not* replace.** Congestion control still needs ACK-side
   rate/RTT feedback. Hashes alone do not order the byte stream or prove the
   path is uncongested. Hybrid keeps ACK for CC; Blake3 owns block integrity +
   selective pull.

## Evidence (same path, sequential runs)

Full tables and raw logs: [`docs/benchmarks/WAN-RESULTS.md`](../benchmarks/WAN-RESULTS.md).

| Transfer | ACK/NAK | Blake3 hybrid | SFTP |
| --- | --- | --- | --- |
| 8 MiB probe | 4.26 MiB/s | 3.81 MiB/s (−10%, startup tax) | — |
| 1 GiB probe | 5.38 MiB/s | **7.44 MiB/s** (+38%) | — |
| ~4 GB Ubuntu ISO | 5.33 MiB/s | **6.79 MiB/s** (+27%) | 6.53 MiB/s |

Blake3 is slightly slower on very short flows (seal/WANT warm-up) and **faster
than both baseline UDT and SFTP** on multi-gigabyte transfers on this path.

## Non-goals and compatibility

- Experimental control types are **not** UDT4 / draft-gg-udt-03.
- Both ends must use the same mode (`udt-cli` defaults to Blake3; use
  `--no-default-features` for classic ACK/NAK interop with stock UDT4).
- Not a claim that Blake3 always beats TCP/SFTP on every path — results are for
  the documented bench host and date.
