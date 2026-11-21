# Full ISO A/B: baseline UDT vs Blake3 hybrid vs SFTP (2026-11-21)

**File:** `ubuntu-26.04.1-desktop-arm64.iso`  
**Size:** 4 154 798 080 bytes (~3 962.3 MiB)  
**Path:** local → `70.88.205.138` (`/SSD2/marchon/udt-bench/`)  
**Stack:** wired Blake3 hybrid (feature `blake3-reliability`) vs ACK/NAK default; same tree otherwise.

## Results (sequential, `/usr/bin/time -p` real)

| Method | Wall time | Rate | Remote path | Size OK |
| --- | --- | --- | --- | --- |
| **UDT baseline** (ACK/NAK) | **743.22 s** (~12.4 min) | **5.331 MiB/s** | `ubuntu.iso` | yes |
| **UDT Blake3 hybrid** | **583.41 s** (~9.7 min) | **6.792 MiB/s** | `ubuntu-blake3.iso` | yes |
| **SFTP** | **606.86 s** (~10.1 min) | **6.529 MiB/s** | `ubuntu-sftp.iso` | yes |

- Blake3 vs baseline UDT: **+27.4%**
- Blake3 vs SFTP: **+4.0%** (Blake3 ahead)
- Baseline UDT vs SFTP: **81.6%** of SFTP (prior cf4874a run was 83.7% / 5.446 vs 6.509)

## Shorter probes (same day)

| Size | Baseline | Blake3 |
| --- | --- | --- |
| 8 MiB | 4.257 | 3.813 (−10%) |
| 32 MiB | 5.523 / 4.949 | 5.717 / 5.673 |
| 1 GiB | 5.376 | **7.435** (+38%) |

## Notes

- Transfers sequential so each owns the path.
- At measurement time the feature was opt-in; **`udt-cli` now defaults to Blake3**
  (`--no-default-features` for classic ACK/NAK). Both ends must match.
- Logs: `docs/benchmarks/results/blake3-iso-*.log`, `blake3-ab-*.log`.
