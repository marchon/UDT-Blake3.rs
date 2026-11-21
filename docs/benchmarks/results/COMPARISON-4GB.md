# Full ISO UDT vs SFTP (2026-11-21, post-opt9b)

**File:** `ubuntu-26.04.1-desktop-arm64.iso`  
**Size:** 4 154 798 080 bytes (~3 962.3 MiB)  
**Path:** local → `70.88.205.138` (`/SSD2/marchon/udt-bench/`)  
**Stack:** commit `cf4874a` (early climb + loss×1.03 + RTO/NAK×16)

## Results

| Method | Wall time (`time` real) | Rate | Remote path | Size OK |
| --- | --- | --- | --- | --- |
| **UDT** (`udtcat`) | **727.53 s** (~12.1 min) | **5.446 MiB/s** | `ubuntu.iso` | yes (exact) |
| **SFTP** | **608.70 s** (~10.1 min) | **6.509 MiB/s** | `ubuntu-sftp.iso` | yes (exact) |

UDT reached **83.7%** of SFTP rate on the same path (gap **1.06 MiB/s**).

## Prior baseline (same ISO / path, before CC fixes)

| Method | Rate | Notes |
| --- | --- | --- |
| UDT | ~0.54 MiB/s | CWND pinned at 16; transfer aborted mid-way |
| SFTP | ~4.5 MiB/s | early sample only |

## Notes

- Transfers run **sequentially** (UDT then SFTP) so each gets the full path.
- UDT sustained ~5.3–5.7 MiB/s for the whole ISO; no multi-minute stalls.
- Sustained UDT ≈ period ~245 µs at MSS 1400 — short 32 MiB probes reached ~140 µs / 5+ MiB/s peaks, so long-flow equilibrium still has headroom.
- `/usr/bin/time -p` `real` is the authoritative wall clock for rate.
