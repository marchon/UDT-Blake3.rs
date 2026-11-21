//! Blake3 64 KiB block seal + WANT/HAVE pull recovery (feature `blake3-reliability`).
//!
//! Hybrid mode: packet sequence + light ACK remain the CC path. This module
//! adds content-addressed blocks so the receiver can **WANT** missing slots and
//! **HAVE** verified blocks instead of relying only on NAK lists.
//!
//! Wire control types (experimental, not UDT4):
//! - `0x10` [`BlockSealPacket`] — sender announces digest after a block is sent
//! - `0x11` [`WantPacket`] — receiver pulls missing packet slots (bitmap)
//! - `0x12` [`HavePacket`] — receiver confirmed Blake3; sender may free the block

use crate::packet::{encode_control_word0, is_control, PacketError, MIN_HEADER_LEN};
use crate::reliability::SeqNo;
use crate::seq::{seq_add, seq_norm};
use std::collections::BTreeMap;

/// Fixed application payload per sealed block.
pub const BLAKE3_BLOCK_BYTES: usize = 64 * 1024;
/// Experimental control type: block seal.
pub const CTRL_BLOCK_SEAL: u16 = 0x10;
/// Experimental control type: WANT.
pub const CTRL_WANT: u16 = 0x11;
/// Experimental control type: HAVE.
pub const CTRL_HAVE: u16 = 0x12;

/// Header + block_id + first_seq + nbytes + digest[32].
pub const BLOCK_SEAL_PACKET_LEN: usize = MIN_HEADER_LEN + 4 + 4 + 4 + 32;
/// Header + block_id + first_seq + bitmap_u64.
pub const WANT_PACKET_LEN: usize = MIN_HEADER_LEN + 4 + 4 + 8;
/// Header + block_id.
pub const HAVE_PACKET_LEN: usize = MIN_HEADER_LEN + 4;

/// Max packets tracked per block bitmap (u64).
pub const WANT_BITMAP_BITS: usize = 64;

/// Sender → receiver: sealed block metadata + Blake3-256 digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSealPacket {
    /// Monotonic block id.
    pub block_id: u32,
    /// Sequence number of the first data packet in the block.
    pub first_seq: SeqNo,
    /// Payload bytes covered (≤ [`BLAKE3_BLOCK_BYTES`]; last block may be short).
    pub nbytes: u32,
    /// Exact packet count in the block (stored in header additional-info).
    pub pkt_count: u32,
    /// Destination socket id.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
    /// Blake3-256 of the contiguous plaintext payload.
    pub digest: [u8; 32],
}

impl BlockSealPacket {
    /// Encode into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < BLOCK_SEAL_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: BLOCK_SEAL_PACKET_LEN,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_BLOCK_SEAL).to_be_bytes());
        buf[4..8].copy_from_slice(&self.pkt_count.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        buf[16..20].copy_from_slice(&self.block_id.to_be_bytes());
        buf[20..24].copy_from_slice(&seq_norm(self.first_seq).to_be_bytes());
        buf[24..28].copy_from_slice(&self.nbytes.to_be_bytes());
        buf[28..60].copy_from_slice(&self.digest);
        Ok(BLOCK_SEAL_PACKET_LEN)
    }

    /// Decode from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < BLOCK_SEAL_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: BLOCK_SEAL_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_BLOCK_SEAL {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&buf[28..60]);
        Ok(Self {
            pkt_count: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            block_id: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            first_seq: seq_norm(u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]])),
            nbytes: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            digest,
        })
    }
}

/// Receiver → sender: pull missing packet slots inside a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WantPacket {
    /// Block id being repaired.
    pub block_id: u32,
    /// First seq of the block (bitmap bit 0).
    pub first_seq: SeqNo,
    /// Bit `i` set ⇒ packet `first_seq + i` is missing (up to 64 packets).
    pub missing_bitmap: u64,
    /// Destination socket id.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
}

impl WantPacket {
    /// Encode into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < WANT_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: WANT_PACKET_LEN,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_WANT).to_be_bytes());
        buf[4..8].copy_from_slice(&0u32.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        buf[16..20].copy_from_slice(&self.block_id.to_be_bytes());
        buf[20..24].copy_from_slice(&seq_norm(self.first_seq).to_be_bytes());
        buf[24..32].copy_from_slice(&self.missing_bitmap.to_be_bytes());
        Ok(WANT_PACKET_LEN)
    }

    /// Decode from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < WANT_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: WANT_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_WANT {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        Ok(Self {
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            block_id: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            first_seq: seq_norm(u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]])),
            missing_bitmap: u64::from_be_bytes([
                buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31],
            ]),
        })
    }

    /// Seq numbers marked missing in the bitmap.
    pub fn missing_seqs(&self) -> Vec<SeqNo> {
        let mut out = Vec::new();
        for i in 0..WANT_BITMAP_BITS {
            if (self.missing_bitmap >> i) & 1 == 1 {
                out.push(seq_add(self.first_seq, i as u32));
            }
        }
        out
    }
}

/// Receiver → sender: block verified; safe to free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HavePacket {
    /// Verified block id.
    pub block_id: u32,
    /// Destination socket id.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
}

impl HavePacket {
    /// Encode into `buf`.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < HAVE_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: HAVE_PACKET_LEN,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_HAVE).to_be_bytes());
        buf[4..8].copy_from_slice(&0u32.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        buf[16..20].copy_from_slice(&self.block_id.to_be_bytes());
        Ok(HAVE_PACKET_LEN)
    }

    /// Decode from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < HAVE_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: HAVE_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_HAVE {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        Ok(Self {
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            block_id: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
        })
    }
}

/// Hash plaintext with Blake3-256.
pub fn blake3_digest(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// One sealed block retained by the sender until HAVE.
#[derive(Debug, Clone)]
struct StoredBlock {
    first_seq: SeqNo,
    /// Packets in send order (index 0 = first_seq).
    packets: Vec<Vec<u8>>,
    #[allow(dead_code)]
    digest: [u8; 32],
    #[allow(dead_code)]
    nbytes: usize,
}

/// Sender-side accumulator: seals every [`BLAKE3_BLOCK_BYTES`] and serves WANTs.
#[derive(Debug, Default)]
pub struct Blake3BlockSender {
    next_block_id: u32,
    open_first_seq: Option<SeqNo>,
    open_packets: Vec<Vec<u8>>,
    open_bytes: usize,
    store: BTreeMap<u32, StoredBlock>,
    peer_sock_id: u32,
}

impl Blake3BlockSender {
    /// Create a sender for `peer_sock_id`.
    pub fn new(peer_sock_id: u32) -> Self {
        Self {
            peer_sock_id,
            ..Self::default()
        }
    }

    /// Record a just-sent data packet. Returns a seal when a block fills.
    pub fn on_data_sent(
        &mut self,
        seq: SeqNo,
        payload: &[u8],
        timestamp: u32,
    ) -> Option<BlockSealPacket> {
        if self.open_first_seq.is_none() {
            self.open_first_seq = Some(seq_norm(seq));
        }
        self.open_packets.push(payload.to_vec());
        self.open_bytes = self.open_bytes.saturating_add(payload.len());
        if self.open_bytes >= BLAKE3_BLOCK_BYTES {
            return Some(self.seal_open(timestamp));
        }
        None
    }

    /// Seal a short final block (end of stream).
    pub fn flush_final(&mut self, timestamp: u32) -> Option<BlockSealPacket> {
        if self.open_bytes == 0 {
            return None;
        }
        Some(self.seal_open(timestamp))
    }

    fn seal_open(&mut self, timestamp: u32) -> BlockSealPacket {
        let first = self.open_first_seq.expect("open block");
        let mut plain = Vec::with_capacity(self.open_bytes);
        for p in &self.open_packets {
            plain.extend_from_slice(p);
        }
        if plain.len() > BLAKE3_BLOCK_BYTES {
            plain.truncate(BLAKE3_BLOCK_BYTES);
        }
        let digest = blake3_digest(&plain);
        let block_id = self.next_block_id;
        self.next_block_id = self.next_block_id.wrapping_add(1);
        let nbytes = plain.len();
        let pkt_count = self.open_packets.len() as u32;
        self.store.insert(
            block_id,
            StoredBlock {
                first_seq: first,
                packets: std::mem::take(&mut self.open_packets),
                digest,
                nbytes,
            },
        );
        self.open_first_seq = None;
        self.open_bytes = 0;
        BlockSealPacket {
            block_id,
            first_seq: first,
            nbytes: nbytes as u32,
            pkt_count,
            dest_sock_id: self.peer_sock_id,
            timestamp,
            digest,
        }
    }

    /// Serve a WANT: return `(seq, payload)` pairs to retransmit (capped).
    pub fn on_want(&self, want: &WantPacket, max_rexmit: usize) -> Vec<(SeqNo, Vec<u8>)> {
        let Some(block) = self.store.get(&want.block_id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for seq in want.missing_seqs() {
            if out.len() >= max_rexmit {
                break;
            }
            let off = seq_norm(seq).wrapping_sub(seq_norm(block.first_seq)) as usize;
            if let Some(payload) = block.packets.get(off) {
                out.push((seq, payload.clone()));
            }
        }
        out
    }

    /// Free a block after receiver HAVE.
    pub fn on_have(&mut self, have: &HavePacket) -> bool {
        self.store.remove(&have.block_id).is_some()
    }

    /// Number of sealed blocks still held.
    pub fn stored_blocks(&self) -> usize {
        self.store.len()
    }

    /// Bytes accumulated in the open (unsealed) block.
    pub fn open_bytes(&self) -> usize {
        self.open_bytes
    }
}

#[derive(Debug)]
struct RecvBlock {
    first_seq: SeqNo,
    nbytes: usize,
    digest: [u8; 32],
    /// Packet index → payload.
    slots: BTreeMap<u32, Vec<u8>>,
    expected_pkts: u32,
}

/// Control messages the receiver wants the socket to send.
#[derive(Debug, Clone)]
pub enum Blake3RecvOut {
    /// Pull missing slots.
    Want(WantPacket),
    /// Block verified.
    Have(HavePacket),
}

/// Receiver-side: assemble, WANT gaps, verify Blake3, emit HAVE.
#[derive(Debug)]
pub struct Blake3BlockReceiver {
    peer_sock_id: u32,
    blocks: BTreeMap<u32, RecvBlock>,
    /// Data that arrived before its block seal (seq → payload).
    orphans: BTreeMap<SeqNo, Vec<u8>>,
    /// Verified payloads waiting for in-order delivery.
    verified: BTreeMap<u32, Vec<u8>>,
    deliver_next: u32,
    delivered: Vec<u8>,
}

impl Blake3BlockReceiver {
    /// Create a receiver addressing `peer_sock_id`.
    pub fn new(peer_sock_id: u32) -> Self {
        Self {
            peer_sock_id,
            blocks: BTreeMap::new(),
            orphans: BTreeMap::new(),
            verified: BTreeMap::new(),
            deliver_next: 0,
            delivered: Vec::new(),
        }
    }

    /// Ingest a seal; drain orphans into the new block, then WANT or HAVE.
    pub fn on_seal(&mut self, seal: &BlockSealPacket, timestamp: u32) -> Vec<Blake3RecvOut> {
        let expected_pkts = seal.pkt_count.clamp(1, WANT_BITMAP_BITS as u32);
        let mut slots = BTreeMap::new();
        let first = seq_norm(seal.first_seq);
        // Claim matching orphans so we do not WANT packets already received.
        let mut claim: Vec<SeqNo> = Vec::new();
        for i in 0..expected_pkts {
            let seq = seq_add(first, i);
            if self.orphans.contains_key(&seq) {
                claim.push(seq);
            }
        }
        for seq in claim {
            if let Some(payload) = self.orphans.remove(&seq) {
                let idx = seq_norm(seq).wrapping_sub(first);
                slots.insert(idx, payload);
            }
        }
        // Bound orphan map growth if seals are lost.
        while self.orphans.len() > 256 {
            if let Some(k) = self.orphans.keys().next().copied() {
                self.orphans.remove(&k);
            } else {
                break;
            }
        }
        self.blocks.insert(
            seal.block_id,
            RecvBlock {
                first_seq: first,
                nbytes: seal.nbytes as usize,
                digest: seal.digest,
                slots,
                expected_pkts,
            },
        );
        self.poll_block(seal.block_id, timestamp)
    }

    /// Ingest data for a known sealed block, or park as orphan until seal.
    pub fn on_data(
        &mut self,
        seq: SeqNo,
        payload: &[u8],
        timestamp: u32,
    ) -> Vec<Blake3RecvOut> {
        let seq = seq_norm(seq);
        let mut hit = None;
        for (bid, block) in self.blocks.iter_mut() {
            let idx = seq.wrapping_sub(seq_norm(block.first_seq));
            if idx < block.expected_pkts {
                block.slots.insert(idx, payload.to_vec());
                hit = Some(*bid);
                break;
            }
        }
        match hit {
            Some(bid) => self.poll_block(bid, timestamp),
            None => {
                self.orphans.insert(seq, payload.to_vec());
                while self.orphans.len() > 256 {
                    if let Some(k) = self.orphans.keys().next().copied() {
                        self.orphans.remove(&k);
                    } else {
                        break;
                    }
                }
                Vec::new()
            }
        }
    }

    /// Copy verified contiguous bytes into `out`; returns bytes copied.
    pub fn read_delivered(&mut self, out: &mut [u8]) -> usize {
        let n = out.len().min(self.delivered.len());
        out[..n].copy_from_slice(&self.delivered[..n]);
        self.delivered.drain(..n);
        n
    }

    /// Bytes waiting in the verified delivery buffer.
    pub fn delivered_len(&self) -> usize {
        self.delivered.len()
    }

    fn poll_block(&mut self, block_id: u32, timestamp: u32) -> Vec<Blake3RecvOut> {
        let mut out = Vec::new();
        let Some(block) = self.blocks.get(&block_id) else {
            return out;
        };
        // WANT if any gap among expected slots.
        let mut bitmap = 0u64;
        let mut missing = false;
        for i in 0..block.expected_pkts as usize {
            if !block.slots.contains_key(&(i as u32)) {
                bitmap |= 1u64 << i;
                missing = true;
            }
        }
        if missing {
            out.push(Blake3RecvOut::Want(WantPacket {
                block_id,
                first_seq: block.first_seq,
                missing_bitmap: bitmap,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
            return out;
        }
        // All slots present — assemble and verify.
        let nbytes = block.nbytes;
        let digest = block.digest;
        let first_seq = block.first_seq;
        let expected_pkts = block.expected_pkts;
        let mut plain = Vec::with_capacity(nbytes);
        for i in 0..expected_pkts {
            plain.extend_from_slice(&block.slots[&i]);
        }
        plain.truncate(nbytes);
        if blake3_digest(&plain) != digest {
            // Corruption: drop slots so a later WANT rebuilds the whole block.
            let mut full = 0u64;
            for i in 0..expected_pkts as usize {
                full |= 1u64 << i;
            }
            if let Some(b) = self.blocks.get_mut(&block_id) {
                b.slots.clear();
            }
            out.push(Blake3RecvOut::Want(WantPacket {
                block_id,
                first_seq,
                missing_bitmap: full,
                dest_sock_id: self.peer_sock_id,
                timestamp,
            }));
            return out;
        }
        self.blocks.remove(&block_id);
        self.verified.insert(block_id, plain);
        self.drain_verified();
        out.push(Blake3RecvOut::Have(HavePacket {
            block_id,
            dest_sock_id: self.peer_sock_id,
            timestamp,
        }));
        out
    }

    fn drain_verified(&mut self) {
        while let Some(payload) = self.verified.remove(&self.deliver_next) {
            self.delivered.extend_from_slice(&payload);
            self.deliver_next = self.deliver_next.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_want_have_wire_round_trip() {
        let seal = BlockSealPacket {
            block_id: 3,
            first_seq: 100,
            nbytes: 64000,
            pkt_count: 46,
            dest_sock_id: 9,
            timestamp: 1,
            digest: [0xAB; 32],
        };
        let mut buf = [0u8; 128];
        let n = seal.encode(&mut buf).unwrap();
        assert_eq!(n, BLOCK_SEAL_PACKET_LEN);
        assert_eq!(BlockSealPacket::decode(&buf[..n]).unwrap(), seal);

        let want = WantPacket {
            block_id: 3,
            first_seq: 100,
            missing_bitmap: 0b1011,
            dest_sock_id: 9,
            timestamp: 2,
        };
        let n = want.encode(&mut buf).unwrap();
        assert_eq!(WantPacket::decode(&buf[..n]).unwrap(), want);
        assert_eq!(want.missing_seqs(), vec![100, 101, 103]);

        let have = HavePacket {
            block_id: 3,
            dest_sock_id: 9,
            timestamp: 3,
        };
        let n = have.encode(&mut buf).unwrap();
        assert_eq!(HavePacket::decode(&buf[..n]).unwrap(), have);
    }

    #[test]
    fn sender_seals_64kib_and_serves_want() {
        let mut tx = Blake3BlockSender::new(2);
        let mss = 1400;
        let mut seq = 0u32;
        let mut sealed = None;
        // Fill exactly one block.
        while sealed.is_none() {
            let remaining = BLAKE3_BLOCK_BYTES.saturating_sub(tx.open_bytes());
            let chunk = remaining.min(mss).max(1);
            let payload = vec![0x5Au8; chunk];
            sealed = tx.on_data_sent(seq, &payload, 1);
            seq = seq_add(seq, 1);
        }
        let seal = sealed.unwrap();
        assert_eq!(seal.block_id, 0);
        assert_eq!(seal.nbytes as usize, BLAKE3_BLOCK_BYTES);
        assert_eq!(tx.stored_blocks(), 1);

        // Drop packet index 2 → WANT bit 2.
        let want = WantPacket {
            block_id: 0,
            first_seq: seal.first_seq,
            missing_bitmap: 1u64 << 2,
            dest_sock_id: 2,
            timestamp: 2,
        };
        let rexmit = tx.on_want(&want, 16);
        assert_eq!(rexmit.len(), 1);
        assert_eq!(rexmit[0].0, seq_add(seal.first_seq, 2));

        assert!(tx.on_have(&HavePacket {
            block_id: 0,
            dest_sock_id: 2,
            timestamp: 3,
        }));
        assert_eq!(tx.stored_blocks(), 0);
    }

    #[test]
    fn receiver_wants_gap_then_verifies_and_haves() {
        let mut tx = Blake3BlockSender::new(2);
        let mut rx = Blake3BlockReceiver::new(1);
        let mss = 1400usize;
        let mut seq = 10u32;
        let mut packets = Vec::new();
        let mut seal = None;
        while seal.is_none() {
            let remaining = BLAKE3_BLOCK_BYTES.saturating_sub(tx.open_bytes());
            let chunk = remaining.min(mss).max(1);
            let payload = vec![(seq & 0xff) as u8; chunk];
            packets.push((seq, payload.clone()));
            seal = tx.on_data_sent(seq, &payload, 1);
            seq = seq_add(seq, 1);
        }
        let seal = seal.unwrap();

        // Deliver seal first, skip packet index 1, then fill.
        let outs = rx.on_seal(&seal, 1);
        assert!(matches!(outs[0], Blake3RecvOut::Want(_)));

        for (i, (s, p)) in packets.iter().enumerate() {
            if i == 1 {
                continue; // gap
            }
            let _ = rx.on_data(*s, p, 2);
        }
        // Still missing index 1.
        let outs = rx.on_data(packets[0].0, &packets[0].1, 3); // duplicate ok
        let want = outs
            .iter()
            .find_map(|o| match o {
                Blake3RecvOut::Want(w) => Some(w.clone()),
                _ => None,
            })
            .expect("want gap");
        assert_ne!(want.missing_bitmap & (1 << 1), 0);

        // Repair via WANT path.
        let rexmit = tx.on_want(&want, 16);
        assert!(!rexmit.is_empty());
        let mut have = None;
        for (s, p) in rexmit {
            for o in rx.on_data(s, &p, 4) {
                if let Blake3RecvOut::Have(h) = o {
                    have = Some(h);
                }
            }
        }
        let have = have.expect("have after repair");
        assert_eq!(have.block_id, 0);
        assert!(tx.on_have(&have));

        let mut buf = vec![0u8; BLAKE3_BLOCK_BYTES];
        assert_eq!(rx.read_delivered(&mut buf), BLAKE3_BLOCK_BYTES);
        assert_eq!(blake3_digest(&buf), seal.digest);
    }

    #[test]
    fn corruption_clears_and_wants_full_block() {
        let mut rx = Blake3BlockReceiver::new(1);
        let payload = vec![1u8; 1400];
        let mut plain = Vec::new();
        for _ in 0..47 {
            plain.extend_from_slice(&payload);
        }
        plain.truncate(BLAKE3_BLOCK_BYTES);
        let good = blake3_digest(&plain);
        let mut bad = good;
        bad[0] ^= 0xff;
        let expected = ((BLAKE3_BLOCK_BYTES as u32 + 1399) / 1400).clamp(1, 64);
        let seal = BlockSealPacket {
            block_id: 0,
            first_seq: 0,
            nbytes: BLAKE3_BLOCK_BYTES as u32,
            pkt_count: expected,
            dest_sock_id: 1,
            timestamp: 1,
            digest: bad,
        };
        let _ = rx.on_seal(&seal, 1);
        // Feed enough packets to satisfy expected_pkts count.
        for i in 0..expected {
            let _ = rx.on_data(i, &payload, 2);
        }
        // Last on_data should detect mismatch and WANT full bitmap.
        let outs = rx.on_data(0, &payload, 3);
        let want = outs.iter().find_map(|o| match o {
            Blake3RecvOut::Want(w) => Some(w),
            _ => None,
        });
        assert!(want.is_some());
        assert_eq!(rx.delivered_len(), 0);
    }
}
