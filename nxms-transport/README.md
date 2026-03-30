# nxms-transport

Library crate containing the NXMS transport primitives:

- `wire`: legacy escrow wire types (`NxmsEnvelope`, `NxmsPayload`) plus generic `NXMS/2` types (`NxmsEnvelopeV2`, `NxmsPayloadV2`)
- `crypto`: FFI wrapper for NXMS packet encryption/decryption (FrodoKEM-640-SHAKE + Falcon-1024-CT)
- `peers`: allowlist types (`Peer`, `PeerBook`)
- `tor_net`: optional framed TCP helpers (direct P2P over SOCKS5h)

## Notes

- The native crypto build links against `liboqs` (`-loqs`).
- The mailbox relay does **not** need `crypto`; it can store/forward either envelope version without decryption.
- `seq` is part of the cryptographic binding (anti-replay/idempotency). Keep it monotonic per `(context_id, from)` in `NXMS/2`, and per `(escrow_id, from)` in the legacy escrow flow.
- The current native transport core uses length-prefixed canonical encoding for AAD and signature inputs, explicit domain separation for stream/tag/KDF/hash contexts, and signature verification before KEM decapsulation.
- This hardened transport core is not wire-compatible with older packet encodings that used a different canonicalization order.
- `NXMS/1` remains the escrow-specific compatibility layer. New protocols should prefer `NXMS/2` with explicit `app_proto`, string `msg_type`, and generic `context_id_hex`.
- On Alpine/musl, `.cargo/config.toml` disables `crt-static` for this crate because the PQ native stack links against shared system libraries such as `liboqs.so` and `libcrypto.so.3`.

## Features

- Default features include `crypto`.
- To depend on `wire` only (no native build, no `liboqs`/`libsodium`): use `default-features = false`.
