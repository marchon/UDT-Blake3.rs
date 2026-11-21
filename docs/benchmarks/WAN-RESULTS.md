# WAN benchmark results (UDT-Blake3.rs)

**Date:** 2026-11-21  
**Client:** macOS arm64 laptop  
**Server:** `70.88.205.138` (Linux x86_64), UDP `19000`  
**Payload file (ISO):** `ubuntu-26.04.1-desktop-arm64.iso` — **4 154 798 080** bytes  
**Stack:** pure-Rust UDT; Blake3 hybrid = feature `blake3-reliability`  
**Timing:** `/usr/bin/time -p` `real` for ISO/SFTP; `wan_probe` DONE line for probes  
**Knobs:** MSS payload 1400, MaxFlowWindow 65535, SndBuf/RcvBuf 16 MiB  

Raw logs live under [`results/`](results/). Reproduce with [`scripts/bench/`](../../scripts/bench/).

## Summary table

| Run | Mode | Size | Wall time | Rate (MiB/s) | Log |
| --- | --- | --- | --- | --- | --- |
| Probe | baseline ACK/NAK | 8 MiB | 1.879 s | **4.257** | [`blake3-ab-baseline-8m.log`](results/blake3-ab-baseline-8m.log) |
| Probe | Blake3 | 8 MiB | 2.098 s | **3.813** | [`blake3-ab-on-8m.log`](results/blake3-ab-on-8m.log) |
| Probe | baseline | 32 MiB #1 | 5.794 s | **5.523** | [`blake3-ab-baseline-32m.log`](results/blake3-ab-baseline-32m.log) |
| Probe | baseline | 32 MiB #2 | 6.466 s | **4.949** | [`blake3-ab-baseline-32m-r2.log`](results/blake3-ab-baseline-32m-r2.log) |
| Probe | Blake3 | 32 MiB #1 | 5.597 s | **5.717** | [`blake3-ab-on-32m.log`](results/blake3-ab-on-32m.log) |
| Probe | Blake3 | 32 MiB #2 | 5.641 s | **5.673** | [`blake3-ab-on-32m-r2.log`](results/blake3-ab-on-32m-r2.log) |
| Probe | baseline | 1 GiB | 190.480 s | **5.376** | [`blake3-ab-baseline-1g.log`](results/blake3-ab-baseline-1g.log) |
| Probe | Blake3 | 1 GiB | 137.733 s | **7.435** | [`blake3-ab-on-1g.log`](results/blake3-ab-on-1g.log) |
| ISO | baseline UDT | 3 962.3 MiB | **743.22 s** | **5.331** | [`blake3-iso-baseline-udt.log`](results/blake3-iso-baseline-udt.log) |
| ISO | Blake3 UDT | 3 962.3 MiB | **583.41 s** | **6.792** | [`blake3-iso-on-udt.log`](results/blake3-iso-on-udt.log) |
| ISO | SFTP | 3 962.3 MiB | **606.86 s** | **6.529** | [`blake3-iso-sftp.log`](results/blake3-iso-sftp.log) |

Rate = `bytes / real_seconds / 1024 / 1024`.

## Full ISO A/B (headline)

| Method | Rate | vs baseline UDT | vs SFTP |
| --- | --- | --- | --- |
| UDT ACK/NAK | 5.331 MiB/s | — | 81.6% |
| **UDT Blake3** | **6.792 MiB/s** | **+27.4%** | **+4.0%** |
| SFTP | 6.529 MiB/s | +22.5% | — |

Prior ACK/NAK ISO (commit `cf4874a`, earlier same day): UDT 5.446 vs SFTP 6.509 — see [`results/COMPARISON-4GB.md`](results/COMPARISON-4GB.md).  
Blake3-day write-up: [`results/COMPARISON-4GB-blake3.md`](results/COMPARISON-4GB-blake3.md).

## Method notes

1. Transfers were **sequential** (never UDT and SFTP at once) so each owns the path.
2. Listener: `udtcat -l 0.0.0.0:19000 -o <outfile>` on the server.
3. Client: `udtcat HOST:19000 -i <infile>` or `wan_probe HOST:19000 <nbytes>`.
4. Baseline builds used `--no-default-features` / feature off; Blake3 builds used
   `--features blake3-reliability` (now the **`udt-cli` default**).
5. Remote size checked with `stat` after each ISO (exact byte match).
6. Do **not** `pkill -f "udtcat"` over SSH — it can match the SSH session. Use a
   pid file under the bench directory.

## How to reproduce

```sh
# From a clone of this repo (see scripts/bench/README.md)
./scripts/bench/localhost-smoke.sh
./scripts/bench/wan-probe.sh <server-host> 8388608          # 8 MiB
./scripts/bench/wan-iso.sh <server-host> /path/to/file.iso  # full file
```

Prebuilt binaries for smoke testing: [`dist/`](../../dist/README.md) and GitHub
Releases.
