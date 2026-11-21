//! Connection state machine (Phase 2).

use thiserror::Error;

/// UDT connection lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Socket created; no handshake yet.
    Init,
    /// Handshake in progress (client or server).
    Handshaking,
    /// Data transfer allowed.
    Connected,
    /// Shutdown handshake started.
    Shutdown,
    /// Fully closed; no further I/O.
    Closed,
}

/// Illegal state transition.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    /// Transition not allowed from the current state.
    #[error("cannot transition from {from:?} to {to:?}")]
    Illegal {
        /// Current state.
        from: ConnectionState,
        /// Requested state.
        to: ConnectionState,
    },
}

impl ConnectionState {
    /// Attempt a state transition.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Illegal`] when the edge is not in the allowed graph.
    pub fn transition(self, to: ConnectionState) -> Result<ConnectionState, StateError> {
        let ok = matches!(
            (self, to),
            (Self::Init, Self::Handshaking)
                | (Self::Handshaking, Self::Connected)
                | (Self::Handshaking, Self::Closed)
                | (Self::Connected, Self::Shutdown)
                | (Self::Connected, Self::Closed)
                | (Self::Shutdown, Self::Closed)
                | (Self::Init, Self::Closed)
        );
        if ok {
            Ok(to)
        } else {
            Err(StateError::Illegal { from: self, to })
        }
    }
}

/// Connection stub holding lifecycle state and UDT socket IDs (no I/O).
///
/// Wire handshake and UDP are owned by higher layers; this type only tracks
/// [`ConnectionState`] plus local/peer socket identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    state: ConnectionState,
    sock_id: u32,
    peer_sock_id: u32,
}

impl Connection {
    /// Create a connection in [`ConnectionState::Init`].
    ///
    /// `peer_sock_id` may be `0` until the peer identity is known from handshake.
    pub fn new(sock_id: u32, peer_sock_id: u32) -> Self {
        Self {
            state: ConnectionState::Init,
            sock_id,
            peer_sock_id,
        }
    }

    /// Current lifecycle state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    /// Local UDT socket ID.
    pub fn sock_id(&self) -> u32 {
        self.sock_id
    }

    /// Peer UDT socket ID (`0` if not yet known).
    pub fn peer_sock_id(&self) -> u32 {
        self.peer_sock_id
    }

    /// Set peer socket ID once learned from handshake (no state change).
    pub fn set_peer_sock_id(&mut self, peer_sock_id: u32) {
        self.peer_sock_id = peer_sock_id;
    }

    /// Init → Handshaking.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Illegal`] when not in [`ConnectionState::Init`].
    pub fn begin_handshake(&mut self) -> Result<(), StateError> {
        self.state = self.state.transition(ConnectionState::Handshaking)?;
        Ok(())
    }

    /// Handshaking → Connected.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Illegal`] when not in [`ConnectionState::Handshaking`].
    pub fn mark_connected(&mut self) -> Result<(), StateError> {
        self.state = self.state.transition(ConnectionState::Connected)?;
        Ok(())
    }

    /// Connected → Shutdown.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Illegal`] when not in [`ConnectionState::Connected`].
    pub fn begin_shutdown(&mut self) -> Result<(), StateError> {
        self.state = self.state.transition(ConnectionState::Shutdown)?;
        Ok(())
    }

    /// Transition to [`ConnectionState::Closed`] from any allowed predecessor.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Illegal`] when already closed or the edge is disallowed.
    pub fn close(&mut self) -> Result<(), StateError> {
        self.state = self.state.transition(ConnectionState::Closed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let s = ConnectionState::Init;
        let s = s.transition(ConnectionState::Handshaking).unwrap();
        let s = s.transition(ConnectionState::Connected).unwrap();
        let s = s.transition(ConnectionState::Shutdown).unwrap();
        let s = s.transition(ConnectionState::Closed).unwrap();
        assert_eq!(s, ConnectionState::Closed);
    }

    #[test]
    fn rejects_skip() {
        assert!(ConnectionState::Init
            .transition(ConnectionState::Connected)
            .is_err());
    }

    #[test]
    fn connection_happy_path() {
        let mut c = Connection::new(1, 0);
        assert_eq!(c.state(), ConnectionState::Init);
        assert_eq!(c.sock_id(), 1);
        assert_eq!(c.peer_sock_id(), 0);

        c.begin_handshake().unwrap();
        assert_eq!(c.state(), ConnectionState::Handshaking);

        c.set_peer_sock_id(42);
        assert_eq!(c.peer_sock_id(), 42);

        c.mark_connected().unwrap();
        assert_eq!(c.state(), ConnectionState::Connected);

        c.begin_shutdown().unwrap();
        assert_eq!(c.state(), ConnectionState::Shutdown);

        c.close().unwrap();
        assert_eq!(c.state(), ConnectionState::Closed);
    }

    #[test]
    fn connection_rejects_mark_connected_from_init() {
        let mut c = Connection::new(7, 8);
        let err = c.mark_connected().unwrap_err();
        assert_eq!(
            err,
            StateError::Illegal {
                from: ConnectionState::Init,
                to: ConnectionState::Connected,
            }
        );
        assert_eq!(c.state(), ConnectionState::Init);
    }
}
