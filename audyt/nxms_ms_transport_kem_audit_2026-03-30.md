# FrodoKEM-side audit note for NXMS transport

Data audytu: 2026-03-30

## Scope

This note continues the NXMS transport audit on the KEM side, after the prepared Falcon signer path was already accepted as the production hot path.

In scope:
- `nxms-transport/native/nexum_cli_src/pqc_kem.c`
- `nxms-transport/native/nxms_ms_transport.c` in the `ff_kem_encaps()` / `ff_kem_decaps()` path
- `nxms-transport/src/crypto.rs` KEM wrapper and decrypt entrypoints
- public transport contract around `kem_ct`, decapsulation, KDF input, and signature-first verification order

Out of scope:
- formal proof of FrodoKEM security
- internal review of liboqs / FrodoKEM implementation source beyond wrapper assumptions
- replay/state management above transport
- mailbox / relay semantics

## Current conclusion

No new P1/P2 vulnerability was found in the reviewed FrodoKEM-side transport path.

The current transport stance is:
- authenticated packets are verified signature-first before KEM decapsulation
- decapsulation enforces exact secret-key and ciphertext lengths in the C wrapper
- the shared secret is transient and explicitly cleansed on the C side after KDF use
- Rust-side public API enforces upper/lower length caps before crossing the FFI boundary

## Established guarantees

1. Signature-first verification is real, not just documented.
   - `nxms_ms_verify_decrypt()` rebuilds AAD and `sig_msg`, verifies Falcon, and only then calls `ff_kem_decaps()`.
   - This keeps unauthenticated garbage away from the FrodoKEM decapsulation hot path.

2. Exact-length decapsulation checks are enforced.
   - `ff_kem_decaps()` for `FrodoKEM-640-SHAKE` rejects any `sk_len` or `ct_len` different from the liboqs constants.
   - This is stronger than the coarse public Rust caps and is now covered by a regression test for truncated-but-still-large ciphertext.

3. Shared-secret handling is reasonably disciplined.
   - `ff_kem_encaps()` / `ff_kem_decaps()` allocate the shared secret only on success paths.
   - failure paths cleanse the secret buffer before free
   - `nxms_ms_verify_decrypt()` and `nxms_ms_encrypt_packet_impl()` cleanse the shared secret after `derive_keys()`

4. KDF binding remains consistent with the transport design.
   - `derive_keys()` binds `ss_len`, `ss`, `escrow_id_raw`, and label (`ms-ke` / `ms-km`)
   - AAD binds `sender_id`, `to_id`, `msg_type`, `escrow_id_raw`, `seq`, and `HASH(kem_ct)`

## Assumptions

1. We trust liboqs `FrodoKEM-640-SHAKE` correctness for encaps/decaps semantics.
2. We do not claim a local proof of constant-time behavior for every path inside liboqs FrodoKEM.
3. We rely on the transport contract that `seq` replay/state checks are enforced above this primitive.
4. The `HASH(kem_ct)` choice in AAD is treated as an explicit design decision, not an accidental omission.

## Residual risks

1. An authenticated peer can still force decapsulation work by sending a validly signed packet with malformed or adversarial `kem_ct`.
   - This is expected for any authenticated decrypt path; signature-first only removes unauthenticated spray.

2. The audit depth on FrodoKEM is still wrapper-centric, not implementation-formal.
   - We have strong interface evidence and negative tests, but not a formal side-channel proof for liboqs internals.

3. Transport remains an autorski system protocol.
   - The scheme is intentionally domain-separated and tested, but it is still not a drop-in AEAD theorem imported from a standard library abstraction.

## New regression evidence added in this step

Added tests in `nxms-transport/tests/crypto_negative.rs`:
- decrypt rejects a packet when the recipient uses the wrong FrodoKEM secret key
- direct `kem_decaps()` rejects truncated ciphertext even when the truncated input still stays above the coarse public minimum length

## Practical audit position after this step

Prepared Falcon signing remains the production baseline.
On the KEM side, the current NXMS transport position is now:
- no new concrete transport break found in the reviewed FrodoKEM wrapper path
- exact-length decapsulation and secret cleansing are established properties of the local wrapper
- remaining KEM risk is primarily in assumptions and residual implementation trust, not in a newly discovered protocol flaw
