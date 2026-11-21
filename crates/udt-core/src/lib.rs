//! Pure-Rust UDT protocol core.
//!
//! Runtime-agnostic: no Tokio types in the public API (see ADR-0001).
//!
//! # Modules
//!
//! - [`packet`] — encode/decode of UDT control and data headers
//! - [`handshake`] — UDT4 cookie handshake helpers (C++ compatible)
//! - [`wire`] — data / ACK / NAK codecs
//! - [`seq`] — 31-bit sequence arithmetic
//! - [`state`] — connection state machine
//! - [`reliability`] — ACK/NAK, loss lists, retransmission engine
//! - [`cc`] — congestion control trait and UDT4 native CC
//! - [`timer`] — deadline / timer wheel

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod cc;
pub mod handshake;
pub mod packet;
pub mod reliability;
pub mod seq;
pub mod state;
pub mod timer;
pub mod wire;

pub use cc::{
    CongestionControl, FixedCc, Udt4Cc, UDT_BETA, UDT_INIT_CWND, UDT_INIT_PERIOD_US, UDT_MIN_INC,
    UDT_SYN_US,
};
pub use handshake::{
    client_confirm, client_cookie_confirm, client_request, is_client_request, is_cookie_challenge,
    is_cookie_confirm, is_server_response, peer_ip_from_ipv4, server_cookie_challenge,
    server_final_response, server_response, syn_cookie, CONN_TYPE_RENDEZVOUS, CONN_TYPE_REQUEST,
    CONN_TYPE_RESPONSE, DEFAULT_FLOW_WINDOW, DEFAULT_MAX_PKT_SIZE, SOCK_TYPE_DGRAM,
    SOCK_TYPE_STREAM,
};
pub use packet::{
    HandshakePacket, PacketError, PacketType, CTRL_HANDSHAKE, HANDSHAKE_PACKET_LEN,
    MIN_HEADER_LEN,
};
pub use reliability::{
    LossList, Outgoing, ReliabilityEngine, SeqNo, DEFAULT_MSS_PAYLOAD, DEFAULT_RTO,
};
pub use state::{Connection, ConnectionState, StateError};
pub use timer::{Deadline, TimerKind, TimerWheel};
pub use wire::{
    peek_is_control, Ack2Packet, AckPacket, DataPacket, NakPacket, ACK2_PACKET_LEN,
    ACK_FULL_PACKET_LEN, ACK_PACKET_LEN, CTRL_ACK, CTRL_ACK2, CTRL_NAK, DATA_HEADER_LEN,
    NAK_HEADER_LEN,
};
