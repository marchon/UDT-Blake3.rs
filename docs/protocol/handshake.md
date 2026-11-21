# Handshake (UDT4 cookie protocol)

Compatible with C++ UDT4 regular (non-rendezvous) connect. Four UDP exchanges:

```text
Client                              Server (listening)
  |                                    |
  |--- HS request (conn_type=1) ------>|
  |    cookie=0, dest_sock_id=0        |
  |                                    |
  |<-- cookie challenge (conn_type=1) -|
  |    cookie set, dest=client_id      |
  |                                    |
  |--- confirm (conn_type=-1) -------->|
  |    cookie echoed                   |
  |                                    |
  |<-- final (conn_type=-1) -----------|
  |    server sock_id, ISN echoed      |
  |                                    |
  Connected                         Connected (accepted child)
```

Helpers in `udt_core::handshake`:

| Helper | Role |
| --- | --- |
| `client_request` | Leg 1 |
| `server_cookie_challenge` | Leg 2 |
| `client_cookie_confirm` | Leg 3 |
| `server_final_response` | Leg 4 |

Socket API: `UdtSocket::bind` / `listen` / `connect` / `accept`.

## ISN note

UDT4 echoes the **client** initial sequence number in the final response and
uses that value as the session ISN on both sides (security check). The PureRust
stack follows the same rule after Phase 6.

Stream is the default (`UdtSockType::Stream`). Datagram mode completes the same
handshake; payload messaging remains a later phase.
