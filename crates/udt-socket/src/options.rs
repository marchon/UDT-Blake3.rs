//! Socket options (Phase 5) — UDT `setsockopt` / `getsockopt` analogues.

use std::time::Duration;

/// Identifiers for [`crate::UdtSocket::get_option`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdtOptionKind {
    /// Send buffer size in bytes (`UDT_SNDBUF`).
    SndBuf,
    /// Receive buffer size in bytes (`UDT_RCVBUF`).
    RcvBuf,
    /// Rendezvous connection mode (`UDT_RENDEZVOUS`).
    Rendezvous,
    /// Maximum segment / packet payload hint (`UDT_MSS`).
    Mss,
    /// Send timeout (`UDT_SNDTIMEO`).
    SndTimeo,
    /// Receive timeout (`UDT_RCVTIMEO`).
    RcvTimeo,
    /// Maximum flow / flight window in packets (`UDT_FC`).
    MaxFlowWindow,
}

/// Option values for [`crate::UdtSocket::set_option`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdtOption {
    /// Send buffer (bytes). Advisory in Phase 5; stored for getsockopt parity.
    SndBuf(usize),
    /// Receive buffer (bytes). Advisory.
    RcvBuf(usize),
    /// When true, future connect uses rendezvous (Phase 5 stores flag; HS path TBD).
    Rendezvous(bool),
    /// Max payload bytes per data packet (applied to new reliability sessions).
    Mss(usize),
    /// Timeout for send/flush style waits.
    SndTimeo(Option<Duration>),
    /// Timeout for recv pumps.
    RcvTimeo(Option<Duration>),
    /// Max flow window in packets (feeds CC max CWND after connect).
    MaxFlowWindow(u32),
}

impl UdtOption {
    /// Kind discriminator.
    pub fn kind(&self) -> UdtOptionKind {
        match self {
            Self::SndBuf(_) => UdtOptionKind::SndBuf,
            Self::RcvBuf(_) => UdtOptionKind::RcvBuf,
            Self::Rendezvous(_) => UdtOptionKind::Rendezvous,
            Self::Mss(_) => UdtOptionKind::Mss,
            Self::SndTimeo(_) => UdtOptionKind::SndTimeo,
            Self::RcvTimeo(_) => UdtOptionKind::RcvTimeo,
            Self::MaxFlowWindow(_) => UdtOptionKind::MaxFlowWindow,
        }
    }
}

/// Stored option state on a socket.
#[derive(Debug, Clone)]
pub struct SocketOptionState {
    /// `UDT_SNDBUF`.
    pub snd_buf: usize,
    /// `UDT_RCVBUF`.
    pub rcv_buf: usize,
    /// `UDT_RENDEZVOUS`.
    pub rendezvous: bool,
    /// Payload MSS.
    pub mss: usize,
    /// Send/flush timeout.
    pub snd_timeo: Option<Duration>,
    /// Recv timeout.
    pub rcv_timeo: Option<Duration>,
    /// Max flow window (packets).
    pub max_flow_window: u32,
}

impl Default for SocketOptionState {
    fn default() -> Self {
        Self {
            snd_buf: 256 * 1024,
            rcv_buf: 256 * 1024,
            rendezvous: false,
            mss: udt_core::DEFAULT_MSS_PAYLOAD,
            snd_timeo: Some(Duration::from_secs(30)),
            rcv_timeo: Some(Duration::from_millis(50)),
            max_flow_window: 16_384,
        }
    }
}

impl SocketOptionState {
    /// Apply one option.
    pub fn set(&mut self, opt: UdtOption) {
        match opt {
            UdtOption::SndBuf(v) => self.snd_buf = v.max(1024),
            UdtOption::RcvBuf(v) => self.rcv_buf = v.max(1024),
            UdtOption::Rendezvous(v) => self.rendezvous = v,
            UdtOption::Mss(v) => self.mss = v.clamp(64, 64 * 1024),
            UdtOption::SndTimeo(v) => self.snd_timeo = v,
            UdtOption::RcvTimeo(v) => self.rcv_timeo = v,
            UdtOption::MaxFlowWindow(v) => self.max_flow_window = v.max(2),
        }
    }

    /// Read one option.
    pub fn get(&self, kind: UdtOptionKind) -> UdtOption {
        match kind {
            UdtOptionKind::SndBuf => UdtOption::SndBuf(self.snd_buf),
            UdtOptionKind::RcvBuf => UdtOption::RcvBuf(self.rcv_buf),
            UdtOptionKind::Rendezvous => UdtOption::Rendezvous(self.rendezvous),
            UdtOptionKind::Mss => UdtOption::Mss(self.mss),
            UdtOptionKind::SndTimeo => UdtOption::SndTimeo(self.snd_timeo),
            UdtOptionKind::RcvTimeo => UdtOption::RcvTimeo(self.rcv_timeo),
            UdtOptionKind::MaxFlowWindow => UdtOption::MaxFlowWindow(self.max_flow_window),
        }
    }
}
