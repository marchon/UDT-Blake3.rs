# API reference (one page per function)

Each public function or method has its own page. Template:
[`TEMPLATE.md`](TEMPLATE.md). Lint: `./scripts/check_api_docs.sh`.

## Index

### `udt-core` — `packet`

| Function | Page |
| --- | --- |
| `is_control` | [`packet/is_control.md`](packet/is_control.md) |
| `peek_type` | [`packet/peek_type.md`](packet/peek_type.md) |
| `encode_control_word0` | [`packet/encode_control_word0.md`](packet/encode_control_word0.md) |
| `HandshakePacket::encode` | [`packet/handshake_packet_encode.md`](packet/handshake_packet_encode.md) |
| `HandshakePacket::decode` | [`packet/handshake_packet_decode.md`](packet/handshake_packet_decode.md) |

### `udt-core` — `handshake`

| Function | Page |
| --- | --- |
| `client_request` | [`handshake/client_request.md`](handshake/client_request.md) |
| `server_cookie_challenge` | [`handshake/server_cookie_challenge.md`](handshake/server_cookie_challenge.md) |
| `client_cookie_confirm` / `client_confirm` | [`handshake/client_confirm.md`](handshake/client_confirm.md) |
| `server_final_response` | [`handshake/server_final_response.md`](handshake/server_final_response.md) |
| `server_response` (compat) | [`handshake/server_response.md`](handshake/server_response.md) |

### `udt-core` — `wire`

| Function | Page |
| --- | --- |
| `DataPacket::encode` | [`wire/data_packet_encode.md`](wire/data_packet_encode.md) |
| `AckPacket::encode` | [`wire/ack_packet_encode.md`](wire/ack_packet_encode.md) |

### `udt-core` — `state`

| Function | Page |
| --- | --- |
| `ConnectionState::transition` | [`state/transition.md`](state/transition.md) |
| `Connection::begin_handshake` | [`state/begin_handshake.md`](state/begin_handshake.md) |

### `udt-core` — `reliability`

| Function | Page |
| --- | --- |
| `ReliabilityEngine::enqueue_app_data` | [`reliability/engine_enqueue.md`](reliability/engine_enqueue.md) |
| `LossList::new` | [`reliability/loss_list_new.md`](reliability/loss_list_new.md) |
| `LossList::insert` | [`reliability/loss_list_insert.md`](reliability/loss_list_insert.md) |
| `LossList::remove` | [`reliability/loss_list_remove.md`](reliability/loss_list_remove.md) |

### `udt-core` — `cc`

| Function | Page |
| --- | --- |
| `Udt4Cc::new` | [`cc/udt4cc_new.md`](cc/udt4cc_new.md) |
| `FixedCc::new` | [`cc/fixed_cc_new.md`](cc/fixed_cc_new.md) |
| `CongestionControl::on_ack` | [`cc/on_ack.md`](cc/on_ack.md) |
| `CongestionControl::on_loss` | [`cc/on_loss.md`](cc/on_loss.md) |
| `CongestionControl::sending_period_us` | [`cc/sending_period_us.md`](cc/sending_period_us.md) |
| `CongestionControl::cwnd_pkts` | [`cc/cwnd_pkts.md`](cc/cwnd_pkts.md) |

### `udt-core` — `timer`

| Function | Page |
| --- | --- |
| `TimerWheel::new` | [`timer/timer_wheel_new.md`](timer/timer_wheel_new.md) |

### `udt-socket`

| Function | Page |
| --- | --- |
| `UdtSocket::new` | [`socket/udt_socket_new.md`](socket/udt_socket_new.md) |
| `UdtSocket::bind` | [`socket/udt_socket_bind.md`](socket/udt_socket_bind.md) |
| `UdtSocket::listen` | [`socket/udt_socket_listen.md`](socket/udt_socket_listen.md) |
| `UdtSocket::connect` | [`socket/udt_socket_connect.md`](socket/udt_socket_connect.md) |
| `UdtSocket::accept` | [`socket/udt_socket_accept.md`](socket/udt_socket_accept.md) |
| `UdtSocket::send` | [`socket/udt_socket_send.md`](socket/udt_socket_send.md) |
| `UdtSocket::recv` | [`socket/udt_socket_recv.md`](socket/udt_socket_recv.md) |
| `UdtSocket::flush` | [`socket/udt_socket_flush.md`](socket/udt_socket_flush.md) |
| `UdtSocket::shutdown` | [`socket/udt_socket_shutdown.md`](socket/udt_socket_shutdown.md) |
| `UdtSocket::set_option` | [`socket/udt_socket_set_option.md`](socket/udt_socket_set_option.md) |
| `UdtSocket::get_option` | [`socket/udt_socket_get_option.md`](socket/udt_socket_get_option.md) |
| `AsyncUdtSocket` | [`socket/async_udt_socket.md`](socket/async_udt_socket.md) |
