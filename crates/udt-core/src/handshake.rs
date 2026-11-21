//! Handshake helpers — UDT4 cookie protocol (compatible with C++ UDT4).
//!
//! Regular (non-rendezvous) exchange:
//! 1. Client request (`conn_type = 1`, cookie = 0)
//! 2. Server cookie challenge (`conn_type = 1`, cookie set; echoes client id)
//! 3. Client confirm (`conn_type = -1`, echoes cookie)
//! 4. Server final (`conn_type = -1`, server sock id; ISN echoed for security)

use crate::packet::HandshakePacket;

/// UDT4 stream socket type value (matches reference `UDT_STREAM`).
pub const SOCK_TYPE_STREAM: u32 = 1;
/// UDT4 datagram socket type value (matches reference `UDT_DGRAM`).
pub const SOCK_TYPE_DGRAM: u32 = 2;

/// Regular client→server connection request / cookie-challenge echo.
pub const CONN_TYPE_REQUEST: i32 = 1;
/// Cookie confirm (client) and final response (server).
pub const CONN_TYPE_RESPONSE: i32 = -1;
/// Rendezvous mode (Phase 5+; not used on the regular path).
pub const CONN_TYPE_RENDEZVOUS: i32 = 0;

/// Default MSS-ish max packet size for lab localhost.
pub const DEFAULT_MAX_PKT_SIZE: u32 = 1500;
/// Default flow window in packets (UDT4 listen path often advertises larger).
pub const DEFAULT_FLOW_WINDOW: u32 = 256;

/// Encode an IPv4 address into the 16-byte `peer_ip` handshake field (raw octets).
pub fn peer_ip_from_ipv4(octets: [u8; 4]) -> [u8; 16] {
    let mut ip = [0u8; 16];
    ip[0..4].copy_from_slice(&octets);
    ip
}

/// Build the client's initial handshake request.
pub fn client_request(
    sock_id: u32,
    init_seq: u32,
    sock_type: u32,
    peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket {
    HandshakePacket {
        timestamp,
        dest_sock_id: 0,
        version: 4,
        sock_type,
        init_seq,
        max_pkt_size: DEFAULT_MAX_PKT_SIZE,
        max_flow_window: DEFAULT_FLOW_WINDOW,
        conn_type: CONN_TYPE_REQUEST,
        sock_id,
        syn_cookie: 0,
        peer_ip,
    }
}

/// Server cookie challenge: echo client fields, attach cookie, keep `conn_type = 1`.
pub fn server_cookie_challenge(
    request: &HandshakePacket,
    syn_cookie: u32,
    timestamp: u32,
) -> HandshakePacket {
    HandshakePacket {
        timestamp,
        dest_sock_id: request.sock_id,
        version: 4,
        sock_type: request.sock_type,
        init_seq: request.init_seq,
        max_pkt_size: request.max_pkt_size.min(DEFAULT_MAX_PKT_SIZE),
        max_flow_window: request.max_flow_window,
        conn_type: CONN_TYPE_REQUEST,
        sock_id: request.sock_id,
        syn_cookie,
        peer_ip: request.peer_ip,
    }
}

/// Client confirm after cookie challenge (`conn_type = -1`).
pub fn client_cookie_confirm(
    challenge: &HandshakePacket,
    client_sock_id: u32,
    client_init_seq: u32,
    client_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket {
    HandshakePacket {
        timestamp,
        dest_sock_id: 0,
        version: 4,
        sock_type: challenge.sock_type,
        init_seq: client_init_seq,
        max_pkt_size: challenge.max_pkt_size,
        max_flow_window: challenge.max_flow_window,
        conn_type: CONN_TYPE_RESPONSE,
        sock_id: client_sock_id,
        syn_cookie: challenge.syn_cookie,
        peer_ip: client_peer_ip,
    }
}

/// Server final response after a valid cookie confirm (`conn_type = -1`).
///
/// UDT4 echoes the client's ISN in `init_seq` (security check) and assigns
/// `server_sock_id`. Flow window is the server's advertised receive capacity.
pub fn server_final_response(
    confirm: &HandshakePacket,
    server_sock_id: u32,
    server_flow_window: u32,
    server_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket {
    HandshakePacket {
        timestamp,
        dest_sock_id: confirm.sock_id,
        version: 4,
        sock_type: confirm.sock_type,
        init_seq: confirm.init_seq,
        max_pkt_size: confirm.max_pkt_size.min(DEFAULT_MAX_PKT_SIZE),
        max_flow_window: server_flow_window.max(confirm.max_flow_window),
        conn_type: CONN_TYPE_RESPONSE,
        sock_id: server_sock_id,
        syn_cookie: confirm.syn_cookie,
        peer_ip: server_peer_ip,
    }
}

/// Derive a deterministic SYN cookie from peer addressing material (PureRust).
///
/// C++ UDT4 uses MD5(`host:port:minute`); clients talking to C++ only need to
/// **echo** the challenge cookie and do not call this helper.
pub fn syn_cookie(peer_sock_hint: u32, salt: u32) -> u32 {
    peer_sock_hint
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(salt)
        ^ 0xA5A5_5A5A
}

/// True when `pkt` looks like an initial client request (cookie unset).
pub fn is_client_request(pkt: &HandshakePacket) -> bool {
    pkt.conn_type == CONN_TYPE_REQUEST && pkt.version == 4 && pkt.syn_cookie == 0
}

/// True when `pkt` is a cookie challenge (request type with cookie set).
pub fn is_cookie_challenge(pkt: &HandshakePacket) -> bool {
    pkt.conn_type == CONN_TYPE_REQUEST && pkt.version == 4 && pkt.syn_cookie != 0
}

/// True when `pkt` is a cookie confirm or final server response (`conn_type = -1`).
pub fn is_server_response(pkt: &HandshakePacket) -> bool {
    pkt.conn_type == CONN_TYPE_RESPONSE && pkt.version == 4 && pkt.sock_id != 0
}

/// Alias: client cookie confirm uses `conn_type = -1` with the client's sock id.
pub fn is_cookie_confirm(pkt: &HandshakePacket) -> bool {
    is_server_response(pkt)
}

// --- Compatibility aliases (older call sites / docs) ---

/// Deprecated name for [`server_final_response`] shape used in early Phase 2 docs.
pub fn server_response(
    request: &HandshakePacket,
    server_sock_id: u32,
    server_init_seq: u32,
    syn_cookie: u32,
    server_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket {
    let _ = server_init_seq;
    HandshakePacket {
        timestamp,
        dest_sock_id: request.sock_id,
        version: 4,
        sock_type: request.sock_type,
        init_seq: request.init_seq,
        max_pkt_size: request.max_pkt_size.min(DEFAULT_MAX_PKT_SIZE),
        max_flow_window: request.max_flow_window.min(DEFAULT_FLOW_WINDOW),
        conn_type: CONN_TYPE_RESPONSE,
        sock_id: server_sock_id,
        syn_cookie,
        peer_ip: server_peer_ip,
    }
}

/// Deprecated: prefer [`client_cookie_confirm`] (`conn_type = -1`).
pub fn client_confirm(
    response: &HandshakePacket,
    client_sock_id: u32,
    client_init_seq: u32,
    client_peer_ip: [u8; 16],
    timestamp: u32,
) -> HandshakePacket {
    client_cookie_confirm(
        response,
        client_sock_id,
        client_init_seq,
        client_peer_ip,
        timestamp,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udt4_cookie_shapes() {
        let req = client_request(10, 100, SOCK_TYPE_STREAM, peer_ip_from_ipv4([127, 0, 0, 1]), 1);
        assert!(is_client_request(&req));
        let cookie = syn_cookie(req.sock_id, 99);
        let challenge = server_cookie_challenge(&req, cookie, 2);
        assert!(is_cookie_challenge(&challenge));
        assert_eq!(challenge.dest_sock_id, 10);
        assert_eq!(challenge.syn_cookie, cookie);

        let conf = client_cookie_confirm(&challenge, 10, 100, peer_ip_from_ipv4([127, 0, 0, 1]), 3);
        assert!(is_cookie_confirm(&conf));
        assert_eq!(conf.conn_type, CONN_TYPE_RESPONSE);
        assert_eq!(conf.syn_cookie, cookie);

        let fin = server_final_response(
            &conf,
            20,
            8192,
            peer_ip_from_ipv4([127, 0, 0, 1]),
            4,
        );
        assert!(is_server_response(&fin));
        assert_eq!(fin.sock_id, 20);
        assert_eq!(fin.init_seq, 100);
        assert_eq!(fin.dest_sock_id, 10);
    }
}
