# Reliability (Phase 3)

Reliable stream over the connected UDP path:

1. **Data** packets carry 31-bit sequence numbers and payload (`DataPacket`).
2. Receiver delivers in order; gaps fill a **loss list** and emit **NAK**.
3. Receiver emits cumulative **ACK** (`ack_seq` = next expected).
4. Sender drops ACKed packets from the send buffer; **RTO** and NAK trigger retransmission.

Induced loss for tests: `UdtSocket::set_loss_rate(0.01)` drops outbound *data*
only (control packets still flow).

Exit proof: `transfer_8mib_with_one_percent_loss` in `udt-socket`.
