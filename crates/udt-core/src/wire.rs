//! Data / ACK / NAK wire codecs (Phase 3).

use crate::packet::{encode_control_word0, is_control, PacketError, MIN_HEADER_LEN};
use crate::reliability::SeqNo;
use crate::seq::{seq_norm, SEQ_MASK};

/// Control type: ACK.
pub const CTRL_ACK: u16 = 2;
/// Control type: NAK.
pub const CTRL_NAK: u16 = 3;
/// Control type: ACK2 (acknowledgement of acknowledgement).
pub const CTRL_ACK2: u16 = 6;

/// Bytes in a UDT data header before payload.
pub const DATA_HEADER_LEN: usize = 16;
/// Bytes in a minimal (light) ACK — header only; data ack seq in additional-info word.
pub const ACK_PACKET_LEN: usize = 16;
/// Full ACK: light header + data ack + RTT, RTTVar, flow, recv rate, bandwidth (6 × u32).
pub const ACK_FULL_PACKET_LEN: usize = 40;
/// ACK2 packet length (header only; ACK seq in additional-info word).
pub const ACK2_PACKET_LEN: usize = 16;
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
///
/// Light ACK is 16 bytes (header). Full ACK appends optional feedback fields used
/// by UDT4 CC: RTT, RTT variance, flow window, packet arrival rate, bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AckPacket {
    /// Next expected in-order sequence number.
    pub ack_seq: SeqNo,
    /// Destination socket ID.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
    /// ACK sub-sequence number (for ACK2 RTT measurement); 0 when unused.
    pub ack_number: u32,
    /// Optional: peer-reported RTT sample (microseconds).
    pub rtt_us: Option<u32>,
    /// Optional: peer-reported RTT variance (microseconds).
    pub rtt_var_us: Option<u32>,
    /// Optional: advertised flow / receive window (packets).
    pub flow_window: Option<u32>,
    /// Optional: packet arrival rate at receiver (packets/sec).
    pub recv_rate: Option<u32>,
    /// Optional: estimated link capacity (packets/sec).
    pub bandwidth: Option<u32>,
}

impl AckPacket {
    /// Construct a light ACK (no rate/RTT extras).
    pub fn light(ack_seq: SeqNo, dest_sock_id: u32, timestamp: u32) -> Self {
        Self {
            ack_seq,
            dest_sock_id,
            timestamp,
            ack_number: 0,
            rtt_us: None,
            rtt_var_us: None,
            flow_window: None,
            recv_rate: None,
            bandwidth: None,
        }
    }

    /// True when this ACK carries full CC feedback fields.
    pub fn is_full(&self) -> bool {
        self.recv_rate.is_some() || self.bandwidth.is_some() || self.rtt_us.is_some()
    }

    /// Encode ACK into `buf`. Returns bytes written (16 light or [`ACK_FULL_PACKET_LEN`] full).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        let full = self.is_full();
        let need = if full {
            ACK_FULL_PACKET_LEN
        } else {
            ACK_PACKET_LEN
        };
        if buf.len() < need {
            return Err(PacketError::TooShort {
                need,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_ACK).to_be_bytes());
        // Additional-info: prefer ack_number when set (UDT4 ACK seq); else data ack.
        let add_info = if self.ack_number != 0 {
            self.ack_number
        } else {
            seq_norm(self.ack_seq)
        };
        buf[4..8].copy_from_slice(&add_info.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        if full {
            buf[16..20].copy_from_slice(&seq_norm(self.ack_seq).to_be_bytes());
            buf[20..24].copy_from_slice(&self.rtt_us.unwrap_or(0).to_be_bytes());
            buf[24..28].copy_from_slice(&self.rtt_var_us.unwrap_or(0).to_be_bytes());
            buf[28..32].copy_from_slice(&self.flow_window.unwrap_or(2).to_be_bytes());
            buf[32..36].copy_from_slice(&self.recv_rate.unwrap_or(0).to_be_bytes());
            buf[36..40].copy_from_slice(&self.bandwidth.unwrap_or(0).to_be_bytes());
        }
        Ok(need)
    }

    /// Decode ACK from `buf` (light 16 or full [`ACK_FULL_PACKET_LEN`]).
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
        let add_info = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let timestamp = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let dest_sock_id = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);

        if buf.len() >= ACK_FULL_PACKET_LEN {
            let ack_seq = seq_norm(u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]));
            let rtt = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
            let rtt_var = u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]);
            let flow = u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]);
            let rate = u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]);
            let bw = u32::from_be_bytes([buf[36], buf[37], buf[38], buf[39]]);
            return Ok(Self {
                ack_seq,
                dest_sock_id,
                timestamp,
                ack_number: add_info,
                rtt_us: Some(rtt).filter(|&v| v > 0),
                rtt_var_us: Some(rtt_var).filter(|&v| v > 0),
                flow_window: Some(flow).filter(|&v| v > 0),
                recv_rate: Some(rate).filter(|&v| v > 0),
                bandwidth: Some(bw).filter(|&v| v > 0),
            });
        }

        // Light ACK: additional-info holds the data ack seq (historic layout).
        Ok(Self {
            ack_seq: seq_norm(add_info),
            dest_sock_id,
            timestamp,
            ack_number: 0,
            rtt_us: None,
            rtt_var_us: None,
            flow_window: None,
            recv_rate: None,
            bandwidth: None,
        })
    }
}

/// ACK2: acknowledges a prior ACK's `ack_number` so the peer can measure RTT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack2Packet {
    /// ACK sub-sequence number being acknowledged.
    pub ack_number: u32,
    /// Destination socket ID.
    pub dest_sock_id: u32,
    /// Timestamp.
    pub timestamp: u32,
}

impl Ack2Packet {
    /// Encode ACK2 into `buf` (exactly [`ACK2_PACKET_LEN`] bytes).
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < ACK2_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: ACK2_PACKET_LEN,
                have: buf.len(),
            });
        }
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_ACK2).to_be_bytes());
        buf[4..8].copy_from_slice(&self.ack_number.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        Ok(ACK2_PACKET_LEN)
    }

    /// Decode ACK2 from `buf`.
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < ACK2_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: ACK2_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_ACK2 {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        Ok(Self {
            ack_number: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
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
            let off = 20 + i * 4;
            buf[off..off + 4].copy_from_slice(&seq_norm(*seq).to_be_bytes());
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
        let timestamp = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let dest_sock_id = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
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
            let off = 20 + i * 4;
            lost.push(seq_norm(u32::from_be_bytes([
                buf[off],
                buf[off + 1],
                buf[off + 2],
                buf[off + 3],
            ])));
        }
        Ok(Self {
            lost,
            dest_sock_id,
            timestamp,
        })
    }
}

/// Peek whether `buf` starts with a control packet.
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
    fn ack_light_round_trip() {
        let a = AckPacket::light(100, 3, 1);
        let mut buf = [0u8; 16];
        a.encode(&mut buf).unwrap();
        let d = AckPacket::decode(&buf).unwrap();
        assert_eq!(d.ack_seq, 100);
        assert_eq!(d.dest_sock_id, 3);
        assert!(!d.is_full());
    }

    #[test]
    fn ack_full_round_trip() {
        let a = AckPacket {
            ack_seq: 200,
            dest_sock_id: 5,
            timestamp: 9,
            ack_number: 7,
            rtt_us: Some(6_500),
            rtt_var_us: Some(500),
            flow_window: Some(8192),
            recv_rate: Some(50_000),
            bandwidth: Some(80_000),
        };
        let mut buf = [0u8; 40];
        let n = a.encode(&mut buf).unwrap();
        assert_eq!(n, 40);
        let d = AckPacket::decode(&buf[..n]).unwrap();
        assert_eq!(d.ack_seq, 200);
        assert_eq!(d.ack_number, 7);
        assert_eq!(d.rtt_us, Some(6_500));
        assert_eq!(d.recv_rate, Some(50_000));
        assert_eq!(d.bandwidth, Some(80_000));
    }

    #[test]
    fn ack2_round_trip() {
        let a = Ack2Packet {
            ack_number: 42,
            dest_sock_id: 1,
            timestamp: 3,
        };
        let mut buf = [0u8; 16];
        a.encode(&mut buf).unwrap();
        assert_eq!(Ack2Packet::decode(&buf).unwrap(), a);
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
