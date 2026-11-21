//! Data / ACK / NAK wire codecs (Phase 3).

use crate::packet::{encode_control_word0, is_control, PacketError, MIN_HEADER_LEN};
use crate::reliability::SeqNo;
use crate::seq::{seq_norm, SEQ_MASK};

/// Control type: ACK.
pub const CTRL_ACK: u16 = 2;
/// Control type: NAK.
pub const CTRL_NAK: u16 = 3;

/// Bytes in a UDT data header before payload.
pub const DATA_HEADER_LEN: usize = 16;
/// Bytes in a minimal ACK (header only; ack seq in additional-info word).
pub const ACK_PACKET_LEN: usize = 16;
/// NAK header before the loss-list body.
pub const NAK_HEADER_LEN: usize = 16;

/// Data packet (flag bit clear).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPacket {
    /// Packet sequence number (31-bit).
    pub seq: SeqNo,
    /// Destination UDT socket ID.
    pub dest_sock_id: u32,
    /// Relative timestamp.
    pub timestamp: u32,
    /// Application payload.
    pub payload: Vec<u8>,
}

impl DataPacket {
    /// Encode into `buf`. Returns total bytes written (header + payload).
    ///
    /// # Errors
    ///
    /// [`PacketError::TooShort`] if `buf` cannot hold the packet.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        let need = DATA_HEADER_LEN + self.payload.len();
        if buf.len() < need {
            return Err(PacketError::TooShort {
                need,
                have: buf.len(),
            });
        }
        let seq = seq_norm(self.seq) & SEQ_MASK; // bit 31 clear => data
        buf[0..4].copy_from_slice(&seq.to_be_bytes());
        buf[4..8].copy_from_slice(&0u32.to_be_bytes()); // msg number (stream: 0)
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        buf[DATA_HEADER_LEN..need].copy_from_slice(&self.payload);
        Ok(need)
    }

    /// Decode a data packet from `buf`.
    ///
    /// # Errors
    ///
    /// Too short, or buffer is a control packet.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < DATA_HEADER_LEN {
            return Err(PacketError::TooShort {
                need: DATA_HEADER_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if is_control(word0) {
            return Err(PacketError::UnsupportedControl(
                ((word0 >> 16) & 0x7fff) as u16,
            ));
        }
        Ok(Self {
            seq: seq_norm(word0),
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            payload: buf[DATA_HEADER_LEN..].to_vec(),
        })
    }
}

/// Cumulative ACK: everything with seq &lt; `ack_seq` has been received in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckPacket {
    /// Next expected in-order sequence number.
    pub ack_seq: SeqNo,
    /// Destination socket ID.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
}

impl AckPacket {
    /// Encode ACK into `buf` (exactly [`ACK_PACKET_LEN`] bytes).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < ACK_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: ACK_PACKET_LEN,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_ACK).to_be_bytes());
        buf[4..8].copy_from_slice(&seq_norm(self.ack_seq).to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        Ok(ACK_PACKET_LEN)
    }

    /// Decode ACK from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < ACK_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: ACK_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_ACK {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        Ok(Self {
            ack_seq: seq_norm(u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]])),
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
        })
    }
}

/// NAK carrying an explicit loss list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NakPacket {
    /// Missing sequence numbers.
    pub lost: Vec<SeqNo>,
    /// Destination socket ID.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
}

impl NakPacket {
    /// Encode NAK; body is `u32` count then that many big-endian seqs.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        let need = NAK_HEADER_LEN + 4 + self.lost.len() * 4;
        if buf.len() < need {
            return Err(PacketError::TooShort {
                need,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_NAK).to_be_bytes());
        buf[4..8].copy_from_slice(&0u32.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        let count = self.lost.len() as u32;
        buf[16..20].copy_from_slice(&count.to_be_bytes());
        for (i, seq) in self.lost.iter().enumerate() {
            let o = 20 + i * 4;
            buf[o..o + 4].copy_from_slice(&seq_norm(*seq).to_be_bytes());
        }
        Ok(need)
    }

    /// Decode NAK from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < NAK_HEADER_LEN + 4 {
            return Err(PacketError::TooShort {
                need: NAK_HEADER_LEN + 4,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_NAK {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        let count = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]) as usize;
        let need = NAK_HEADER_LEN + 4 + count * 4;
        if buf.len() < need {
            return Err(PacketError::TooShort {
                need,
                have: buf.len(),
            });
        }
        let mut lost = Vec::with_capacity(count);
        for i in 0..count {
            let o = 20 + i * 4;
            lost.push(seq_norm(u32::from_be_bytes([
                buf[o],
                buf[o + 1],
                buf[o + 2],
                buf[o + 3],
            ])));
        }
        Ok(Self {
            lost,
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
        })
    }
}

/// Classify whether `buf` looks long enough to be a control or data header.
pub fn peek_is_control(buf: &[u8]) -> Result<bool, PacketError> {
    if buf.len() < MIN_HEADER_LEN {
        return Err(PacketError::TooShort {
            need: MIN_HEADER_LEN,
            have: buf.len(),
        });
    }
    let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    Ok(is_control(word0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_round_trip() {
        let p = DataPacket {
            seq: 42,
            dest_sock_id: 7,
            timestamp: 9,
            payload: b"hello".to_vec(),
        };
        let mut buf = vec![0u8; 64];
        let n = p.encode(&mut buf).unwrap();
        let d = DataPacket::decode(&buf[..n]).unwrap();
        assert_eq!(d, p);
        assert!(!peek_is_control(&buf[..n]).unwrap());
    }

    #[test]
    fn ack_round_trip() {
        let a = AckPacket {
            ack_seq: 100,
            dest_sock_id: 3,
            timestamp: 1,
        };
        let mut buf = [0u8; 16];
        a.encode(&mut buf).unwrap();
        assert_eq!(AckPacket::decode(&buf).unwrap(), a);
    }

    #[test]
    fn nak_round_trip() {
        let n = NakPacket {
            lost: vec![5, 6, 9],
            dest_sock_id: 2,
            timestamp: 4,
        };
        let mut buf = vec![0u8; 64];
        let len = n.encode(&mut buf).unwrap();
        assert_eq!(NakPacket::decode(&buf[..len]).unwrap(), n);
    }
}
