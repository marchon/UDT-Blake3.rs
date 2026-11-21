# UDT papers and other implementations

References for the original UDT protocol and related stacks. This repository
(*UDT-Blake3.rs*) is a **pure-Rust** UDT4-class implementation with an optional
Blake3 block seal + WANT/HAVE hybrid reliability path.

## Primary papers

| Work | Citation | Link |
| --- | --- | --- |
| UDT journal paper | Yunhong Gu, Robert L. Grossman. *UDT: UDP-based data transfer for high-speed wide area networks.* Computer Networks 51(7), 1777–1799, May 2007. DOI [10.1016/j.comnet.2006.11.009](https://doi.org/10.1016/j.comnet.2006.11.009) | [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1389128606003057) |
| SC 2004 design | Yunhong Gu, Xinwei Hong, Robert L. Grossman. *Experiences in Design and Implementation of a High Performance Transport Protocol.* SC 2004. DOI [10.1109/SC.2004.24](https://doi.org/10.1109/SC.2004.24) | [IEEE](https://www.computer.org/csdl/proceedings-article/sc/2004/21530022/12OmNwCJOVO) |
| UDTv4 | Yunhong Gu, Robert L. Grossman. *UDTv4: Improvements in Performance and Usability.* GridNets 2008. | [udt.sourceforge.io docs](https://udt.sourceforge.io/doc.html) |
| Configurable CC | Yunhong Gu, Robert L. Grossman. *Supporting Configurable Congestion Control in Data Transport Services.* SC 2005. | [udt.sourceforge.io docs](https://udt.sourceforge.io/doc.html) |
| PFLDnet 2004 | Yunhong Gu, Robert L. Grossman. *UDT: An Application Level Transport Protocol for Grid Computing.* PFLDnet 2004. | [archived PDF](https://web.archive.org/web/20110902183704if_/http:/www-didc.lbl.gov/PFLDNet2004/papers/Grossman.pdf) |

## Protocol specification

| Spec | Notes | Link |
| --- | --- | --- |
| `draft-gg-udt-03` | IETF Internet-Draft (expired). Wire formats used by UDT4 and this crate. | [datatracker](https://datatracker.ietf.org/doc/html/draft-gg-udt-03) |
| Project site | Manuals, papers index, downloads | [udt.sourceforge.io](https://udt.sourceforge.io/) / [udt.sourceforge.net](https://udt.sourceforge.net/) |

## Reference and alternate implementations

| Project | Language | Nature | Link |
| --- | --- | --- | --- |
| **UDT4 (eminence)** | C++ | Canonical UDT4 library (BSD) | [github.com/eminence/udt](https://github.com/eminence/udt) |
| **dorkbox/UDT** | C++ | Maintained UDT packaging / docs mirror | [github.com/dorkbox/UDT](https://github.com/dorkbox/UDT) |
| **udt-rs** | Rust | FFI bindings to C++ UDT4 (`libudt`) | [github.com/eminence/udt-rs](https://github.com/eminence/udt-rs) · [docs.rs/udt](https://docs.rs/udt) |
| **tokio-udt** | Rust | Tokio-based UDT reimplementation | [github.com/distributed-epfl/tokio-udt](https://github.com/Distributed-EPFL/tokio-udt) |
| **Wireshark UDT** | C | Dissector (handshake heuristic) | Wireshark source tree (`epan/dissectors`) |
| **This repo** | Rust | Pure-Rust UDT4-class + Blake3 hybrid | [github.com/marchon/UDT-Blake3.rs](https://github.com/marchon/UDT-Blake3.rs) |

## Related Blake3 / content-addressed transfer work

Not UDT, but relevant to the hybrid design (hash-named blocks + pull repair):

| Project | Notes | Link |
| --- | --- | --- |
| **blake3** | Official Rust Blake3 crate used here | [github.com/BLAKE3-team/BLAKE3](https://github.com/BLAKE3-team/BLAKE3) |
| **iroh-blobs** | Blake3 verified streaming over QUIC | [crates.io/crates/iroh-blobs](https://crates.io/crates/iroh-blobs) |

## How this repo relates

- **Wire-compatible goal:** data/ACK/NAK/handshake follow `draft-gg-udt-03` / UDT4 so classic peers can interoperate when Blake3 mode is off (`udt-cli --no-default-features`).
- **Experimental extension:** control types `0x10` BlockSeal, `0x11` WANT, `0x12` HAVE are **not** in the IETF draft. Both ends must enable `blake3-reliability` (default in `udt-cli`).
- **CC retained:** UDT4-style CWND + SND period control stays; Blake3 does not replace congestion control.

See also [`docs/blake3/why-blake3.md`](../blake3/why-blake3.md) and [`docs/plan/blake3-reliability-sketch.md`](../plan/blake3-reliability-sketch.md).
