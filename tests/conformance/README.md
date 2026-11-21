# Conformance corpus (Phase 0)

## Layout

```text
tests/conformance/
  README.md
  corpus/           # pcaps + raw bin vectors (add in Phase 0)
  notes/
```

## Capture guidelines

1. Prefer Wireshark with UDT dissector enabled.
2. Capture full handshake (both directions) on localhost and one WAN-ish lab path.
3. Export as pcapng; also dump first handshake packet as raw `.bin` for unit tests.
4. Document MTU, loss, and UDT4 version in `notes/`.

Minimum for Phase 0 exit: **10 handshake captures** or equivalent synthetic goldens.
