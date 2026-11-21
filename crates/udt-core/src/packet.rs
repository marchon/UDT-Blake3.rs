//! UDT packet encode/decode (Phase 1).
//!
//! Wire layouts follow draft-gg-udt-03 / UDT4. Multi-byte fields are big-endian.

use thiserror::Error;

/// High-level UDT packet classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    /// Data packet (bit 0 of first word clear in UDT4).
    Data,
    /// Control: handshake.
    Handshake,
    /// Control: keep-alive.
    KeepAlive,
    /// Control: ACK.
    Ack,
    /// Control: NAK (loss report).
    Nak,
    /// Control: shutdown.
    Shutdown,
    /// Control: ACK2.
    Ack2,
    /// Control: message drop request.
    MsgDropReq,
    /// Unknown / reserved control type.
    Other(u16),
}

/// Packet codec errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PacketError {
    /// Buffer shorter than the minimum header.
    #[error("buffer too short: need at least {need} bytes, have {have}")]
    TooShort {
        /// Bytes required.
        need: usize,
        /// Bytes available.
        have: usize,
    },
    /// Control type not recognized for the attempted decode path.
    #[error("unsupported control type {0}")]
    UnsupportedControl(u16),
}

/// Minimum UDT header size in bytes (two 32-bit words before type-specific fields).
pub const MIN_HEADER_LEN: usize = 16;

/// Full handshake control packet length on the wire (header + control info).
///
/// Layout: 16-byte common header + 48-byte handshake body (draft-gg-udt-03 TYPE 0x0).
pub const HANDSHAKE_PACKET_LEN: usize = 64;

/// Control type ID for handshake packets.
pub const CTRL_HANDSHAKE: u16 = 0;

/// UDT4 handshake control packet (TYPE 0x0).
///
/// Field order and sizes match draft-gg-udt-03 / UDT4 `CHandShake`. Multi-byte
/// integers are encoded big-endian. `peer_ip` is a 16-byte placeholder (IPv4
/// mapped into the first 4 bytes, or a full IPv6 address).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakePacket {
    /// Relative send timestamp (header word 2).
    pub timestamp: u32,
    /// Destination socket ID (0 while the peer ID is unknown).
    pub dest_sock_id: u32,
    /// UDT protocol version (4 for UDT4).
    pub version: u32,
    /// Socket type: draft STREAM = 0, DGRAM = 1 (UDT4 reference may use 1/2).
    pub sock_type: u32,
    /// Initial packet sequence number for this side's data stream.
    pub init_seq: u32,
    /// Maximum packet size including UDP/IP headers (MSS / MTU).
    pub max_pkt_size: u32,
    /// Maximum flow / flight window size in packets.
    pub max_flow_window: u32,
    /// Connection type: 1 = regular request, 0 = rendezvous, −1/−2 = response.
    pub conn_type: i32,
    /// This side's UDT socket ID.
    pub sock_id: u32,
    /// SYN cookie (anti-flood); echoed by the client after the first response.
    pub syn_cookie: u32,
    /// Peer's UDP bind address as 16 raw octets (network order).
    pub peer_ip: [u8; 16],
}

impl HandshakePacket {
    /// Encode this handshake into `buf`.
    ///
    /// Writes exactly [`HANDSHAKE_PACKET_LEN`] bytes. Returns that length on
    /// success.
    ///
    /// # Errors
    ///
    /// Returns [`PacketError::TooShort`] if `buf` is shorter than
    /// [`HANDSHAKE_PACKET_LEN`].
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, PacketError> {
        if buf.len() < HANDSHAKE_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: HANDSHAKE_PACKET_LEN,
                have: buf.len(),
            });
        }
        // Common control header: type 0, additional info undefined (0).
        buf[0..4].copy_from_slice(&encode_control_word0(CTRL_HANDSHAKE).to_be_bytes());
        buf[4..8].copy_from_slice(&0u32.to_be_bytes());
        buf[8..12].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[12..16].copy_from_slice(&self.dest_sock_id.to_be_bytes());
        // Handshake body (48 bytes).
        buf[16..20].copy_from_slice(&self.version.to_be_bytes());
        buf[20..24].copy_from_slice(&self.sock_type.to_be_bytes());
        buf[24..28].copy_from_slice(&self.init_seq.to_be_bytes());
        buf[28..32].copy_from_slice(&self.max_pkt_size.to_be_bytes());
        buf[32..36].copy_from_slice(&self.max_flow_window.to_be_bytes());
        buf[36..40].copy_from_slice(&self.conn_type.to_be_bytes());
        buf[40..44].copy_from_slice(&self.sock_id.to_be_bytes());
        buf[44..48].copy_from_slice(&self.syn_cookie.to_be_bytes());
        buf[48..64].copy_from_slice(&self.peer_ip);
        Ok(HANDSHAKE_PACKET_LEN)
    }

    /// Decode a handshake control packet from `buf`.
    ///
    /// # Errors
    ///
    /// - [`PacketError::TooShort`] if `buf` is shorter than
    ///   [`HANDSHAKE_PACKET_LEN`].
    /// - [`PacketError::UnsupportedControl`] if the control type is not
    ///   handshake (0).
    pub fn decode(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < HANDSHAKE_PACKET_LEN {
            return Err(PacketError::TooShort {
                need: HANDSHAKE_PACKET_LEN,
                have: buf.len(),
            });
        }
        let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if !is_control(word0) {
            return Err(PacketError::UnsupportedControl(0xffff));
        }
        let ctrl = ((word0 >> 16) & 0x7fff) as u16;
        if ctrl != CTRL_HANDSHAKE {
            return Err(PacketError::UnsupportedControl(ctrl));
        }
        Ok(Self {
            timestamp: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
            dest_sock_id: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            version: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            sock_type: u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]),
            init_seq: u32::from_be_bytes([buf[24], buf[25], buf[26], buf[27]]),
            max_pkt_size: u32::from_be_bytes([buf[28], buf[29], buf[30], buf[31]]),
            max_flow_window: u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]),
            conn_type: i32::from_be_bytes([buf[36], buf[37], buf[38], buf[39]]),
            sock_id: u32::from_be_bytes([buf[40], buf[41], buf[42], buf[43]]),
            syn_cookie: u32::from_be_bytes([buf[44], buf[45], buf[46], buf[47]]),
            peer_ip: {
                let mut ip = [0u8; 16];
                ip.copy_from_slice(&buf[48..64]);
                ip
            },
        })
    }
}

/// Returns whether the first bit of the header marks a control packet.
pub fn is_control(header_word0: u32) -> bool {
    (header_word0 & 0x8000_0000) != 0
}

/// Decode the packet type from a raw header buffer.
///
/// # Errors
///
/// Returns [`PacketError::TooShort`] if `buf` is shorter than [`MIN_HEADER_LEN`].
pub fn peek_type(buf: &[u8]) -> Result<PacketType, PacketError> {
    if buf.len() < MIN_HEADER_LEN {
        return Err(PacketError::TooShort {
            need: MIN_HEADER_LEN,
            have: buf.len(),
        });
    }
    let word0 = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    if !is_control(word0) {
        return Ok(PacketType::Data);
    }
    let ctrl = ((word0 >> 16) & 0x7fff) as u16;
    Ok(match ctrl {
        0 => PacketType::Handshake,
        1 => PacketType::KeepAlive,
        2 => PacketType::Ack,
        3 => PacketType::Nak,
        5 => PacketType::Shutdown,
        6 => PacketType::Ack2,
        7 => PacketType::MsgDropReq,
        other => PacketType::Other(other),
    })
}

/// Encode a control type into the first header word (type bits only; seq left 0).
pub fn encode_control_word0(ctrl_type: u16) -> u32 {
    0x8000_0000 | ((u32::from(ctrl_type) & 0x7fff) << 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_handshake() -> HandshakePacket {
        HandshakePacket {
            timestamp: 0x0102_0304,
            dest_sock_id: 0,
            version: 4,
            sock_type: 0, // STREAM per draft
            init_seq: 0x0a0b_0c0d,
            max_pkt_size: 1500,
            max_flow_window: 8192,
            conn_type: 1, // regular client request
            sock_id: 0x1122_3344,
            syn_cookie: 0x5566_7788,
            peer_ip: [
                192, 0, 2, 1, // 192.0.2.1 (TEST-NET-1)
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        }
    }

    #[test]
    fn peek_type_too_short() {
        assert!(matches!(
            peek_type(&[0u8; 4]),
            Err(PacketError::TooShort { need: 16, have: 4 })
        ));
    }

    #[test]
    fn handshake_round_trip_type() {
        let mut buf = [0u8; MIN_HEADER_LEN];
        let w0 = encode_control_word0(0);
        buf[0..4].copy_from_slice(&w0.to_be_bytes());
        assert_eq!(peek_type(&buf).unwrap(), PacketType::Handshake);
    }

    #[test]
    fn data_packet_type() {
        let mut buf = [0u8; MIN_HEADER_LEN];
        // bit 31 clear => data
        buf[0..4].copy_from_slice(&0x0000_0001u32.to_be_bytes());
        assert_eq!(peek_type(&buf).unwrap(), PacketType::Data);
    }

    #[test]
    fn handshake_encode_decode_round_trip() {
        let hs = sample_handshake();
        let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
        assert_eq!(hs.encode(&mut buf).unwrap(), HANDSHAKE_PACKET_LEN);
        assert_eq!(peek_type(&buf).unwrap(), PacketType::Handshake);
        let decoded = HandshakePacket::decode(&buf).unwrap();
        assert_eq!(decoded, hs);
    }

    #[test]
    fn handshake_encode_too_short() {
        let hs = sample_handshake();
        let mut buf = [0u8; 32];
        assert_eq!(
            hs.encode(&mut buf),
            Err(PacketError::TooShort {
                need: HANDSHAKE_PACKET_LEN,
                have: 32
            })
        );
    }

    #[test]
    fn handshake_decode_too_short() {
        assert_eq!(
            HandshakePacket::decode(&[0u8; 16]),
            Err(PacketError::TooShort {
                need: HANDSHAKE_PACKET_LEN,
                have: 16
            })
        );
    }

    #[test]
    fn handshake_decode_wrong_type() {
        let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
        // ACK control type (2)
        buf[0..4].copy_from_slice(&encode_control_word0(2).to_be_bytes());
        assert_eq!(
            HandshakePacket::decode(&buf),
            Err(PacketError::UnsupportedControl(2))
        );
    }

    #[test]
    fn handshake_response_conn_type_negative() {
        let mut hs = sample_handshake();
        hs.conn_type = -1;
        hs.syn_cookie = 0xdead_beef;
        let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
        hs.encode(&mut buf).unwrap();
        // conn_type at offset 36 must be big-endian i32 -1
        assert_eq!(&buf[36..40], &(-1i32).to_be_bytes());
        let decoded = HandshakePacket::decode(&buf).unwrap();
        assert_eq!(decoded.conn_type, -1);
        assert_eq!(decoded.syn_cookie, 0xdead_beef);
    }

    #[test]
    fn handshake_wire_field_offsets() {
        let hs = sample_handshake();
        let mut buf = [0u8; HANDSHAKE_PACKET_LEN];
        hs.encode(&mut buf).unwrap();
        assert_eq!(&buf[0..4], &encode_control_word0(0).to_be_bytes());
        assert_eq!(&buf[8..12], &0x0102_0304u32.to_be_bytes());
        assert_eq!(&buf[16..20], &4u32.to_be_bytes());
        assert_eq!(&buf[24..28], &0x0a0b_0c0du32.to_be_bytes());
        assert_eq!(&buf[28..32], &1500u32.to_be_bytes());
        assert_eq!(&buf[40..44], &0x1122_3344u32.to_be_bytes());
        assert_eq!(&buf[48..52], &[192, 0, 2, 1]);
    }

    /// Synthetic golden: same bytes as `tests/conformance/corpus/handshake_control_word0.bin`.
    #[test]
    fn golden_handshake_word0() {
        const INLINE: [u8; MIN_HEADER_LEN] = [
            0x80, 0x00, 0x00, 0x00, // word0: control + Handshake (type 0)
            0x00, 0x00, 0x00, 0x00, // additional info
            0x00, 0x00, 0x00, 0x00, // timestamp
            0x00, 0x00, 0x00, 0x00, // destination socket ID
        ];
        let from_file =
            include_bytes!("../../../tests/conformance/corpus/handshake_control_word0.bin");
        assert_eq!(
            from_file.as_slice(),
            INLINE.as_slice(),
            "corpus bin must match inline golden bytes"
        );
        assert_eq!(peek_type(&INLINE).unwrap(), PacketType::Handshake);
        assert_eq!(peek_type(from_file).unwrap(), PacketType::Handshake);
        assert_eq!(encode_control_word0(0).to_be_bytes(), INLINE[0..4]);
    }
}
