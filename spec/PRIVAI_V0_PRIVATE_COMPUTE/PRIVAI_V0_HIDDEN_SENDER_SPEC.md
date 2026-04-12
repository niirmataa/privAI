# privAI V0 — Hidden Sender (NXMS Envelope V3)

**Status:** canonical V0 protocol spec
**Date:** 2026-04-12
**Scope:** Encrypts `from` field in NxmsEnvelope so mailbox never sees sender identity.
**Supersedes:** NxmsEnvelope (NXMS/1) and NxmsEnvelopeV2 (NXMS/2) `from` field semantics.
**Companion to:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md, PRIVAI_V0_ARCHITECTURE_SPEC.md

---

## 1. Problem

Current `NxmsEnvelope` exposes `from` as plaintext:

```rust
// nxms-transport/src/wire.rs — current
pub struct NxmsEnvelope {
    pub from: String,      // "alice" — VISIBLE to mailbox
    pub to: String,        // "bob"   — VISIBLE to mailbox
    pub seq: u64,          // VISIBLE to mailbox
    pub kem_ct_b64: String,
    pub ciphertext_b64: String,
    // ...
}
```

Mailbox sees the full social graph: who talks to whom, how often, at what times.

**Goal:** Mailbox sees `to` (required for delivery), but never sees `from`, `seq`, or message content.

---

## 2. Solution: One KEM, Two AEADs

### 2.1 Core idea

Alice encrypts `from` + `seq` + Falcon signature using the same FrodoKEM shared secret she already needs for the payload. One encapsulation, two derived nonces, two XChaCha20Poly1305 ciphertexts. **No signature on envelope exterior** — mailbox cannot link envelope to sender's public key.

```rust
// ONE FrodoKEM encapsulation:
let (shared_secret, kem_ct) = frodokem_encapsulate(bob_kem_pk);

// Two nonces from the same shared_secret (NO seq — shared_secret is unique per message):
let nonce_from    = blake3("privai:nonce:from"    || shared_secret);
let nonce_payload = blake3("privai:nonce:payload"  || shared_secret);

// Alice pads from to fixed length BEFORE encryption:
let from_padded = pad_from("alice", 64); // 64 bytes always

// Falcon signature INSIDE the encrypted block:
let payload_hash = blake3(&payload_bytes);
let sig_inner = falcon_sign(alice_sk, blake3("privai:envelope:v1" || from_padded || payload_hash));
let from_block = from_padded || sig_inner; // 64 + ~666 bytes = padded to FIXED_FROM_BLOCK_LEN

let encrypted_from    = xchacha20poly1305_encrypt(&shared_secret, &nonce_from, &from_block);
let encrypted_payload = xchacha20poly1305_encrypt(&shared_secret, &nonce_payload, &payload_bytes);
```

### 2.2 Why one KEM, not two

FrodoKEM-640 encapsulation costs ~1-3ms CPU. Doing it twice to the same public key is wasteful. XChaCha20Poly1305 with different nonces from the same key is cryptographically sound — nonce reuse is the danger, and we derive unique nonces per domain.

### 2.3 Domain separation

```
nonce_from    = blake3("privai:nonce:from"    || shared_secret)
nonce_payload = blake3("privai:nonce:payload"  || shared_secret)
```

Domain separator strings prevent cross-domain nonce collision. `shared_secret` is unique per FrodoKEM encapsulation (fresh per message), so nonce reuse is impossible by construction.

### 2.4 Why signature is INSIDE encrypted block

Falcon signature on envelope exterior is verifiable against known public keys. If mailbox knows Alice's Falcon PK (and it might, for other protocol reasons), it can try `falcon_verify(pk, envelope, sig)` for every known key. If verification succeeds → mailbox knows Alice sent this envelope. **Hidden sender is defeated.**

By putting the signature inside the `encrypted_from` block:
- Mailbox sees only ciphertext — cannot verify against any key
- Bob decrypts, extracts `from` + `sig_inner`, verifies authenticity
- Signature proves: Alice sent this envelope, payload is not tampered
- Mailbox learns: nothing about sender

---

## 3. Wire Format: NxmsEnvelopeV3

```rust
pub struct NxmsEnvelopeV3 {
    pub proto: String,              // "NXMS/3"
    pub kem_id: String,             // "FrodoKEM-640-SHAKE"
    pub to: String,                 // JAWNE — mailbox needs this for routing

    pub kem_ct_b64: String,         // JEDEN KEM ciphertext

    // Encrypted sender block (from + seq + falcon_sig_inner, padded to fixed length):
    pub encrypted_from_b64: String, // xchacha20poly1305 ciphertext
    pub nonce_from_b64: String,     // 24 bytes
    pub tag_from_b64: String,       // 16 bytes

    // Encrypted payload:
    pub nonce_payload_b64: String,  // 24 bytes
    pub ciphertext_b64: String,     // xchacha20poly1305 ciphertext
    pub tag_payload_b64: String,    // 16 bytes

    // PoW anti-spam:
    pub pow_nonce: u64,

    // NO sig_b64 — signature is INSIDE encrypted_from block
    // NO pow_difficulty — mailbox uses its own configured difficulty
}
```

Note: `sig_id` field removed. Falcon algorithm is implied by the protocol version. If algorithm negotiation is needed in future, add it to `encrypted_from` block.
Note: `pow_difficulty` removed from envelope. Client knows difficulty from `/info` endpoint. Mailbox verifies against its own config — client cannot game it by setting `pow_difficulty: 0`.

### 3.1 What mailbox sees

| Field | Visible | Content |
|-------|---------|---------|
| `to` | YES | Recipient peer_id |
| `kem_ct_b64` | YES | Fixed-size ciphertext (~10,880 bytes for FrodoKEM-640) |
| `encrypted_from_b64` | YES | Fixed-size ciphertext (padded, always same size) |
| `ciphertext_b64` | YES | Variable-size payload ciphertext |
| `pow_nonce` | YES | PoW solution |

| Field | Hidden |
|-------|--------|
| `from` | Encrypted inside `encrypted_from_b64` |
| `seq` | Encrypted inside `encrypted_from_b64` |
| `falcon_sig` | Encrypted inside `encrypted_from_b64` |
| `payload` | Encrypted inside `ciphertext_b64` |

### 3.2 What Bob decrypts

```rust
// Bob receives envelope, has his KEM secret key:
let shared_secret = frodokem_decapsulate(bob_kem_sk, &kem_ct);

let nonce_from = blake3("privai:nonce:from" || shared_secret);
let from_block = xchacha20poly1305_decrypt(&shared_secret, &nonce_from, &encrypted_from, &tag_from);

// Parse from_block: [from_padded (64 bytes)] || [falcon_sig_inner (variable)]
let (from_padded, sig_inner_bytes) = split_at(&from_block, FIXED_FROM_LEN);
let (from, seq) = unpad_from(&from_padded);

let nonce_payload = blake3("privai:nonce:payload" || shared_secret);
let payload = xchacha20poly1305_decrypt(&shared_secret, &nonce_payload, &ciphertext, &tag_payload);

// Verify Falcon signature INSIDE the decrypted block:
let payload_hash = blake3(&payload);
let expected_sig_msg = blake3("privai:envelope:v1" || from_padded || payload_hash);
let alice_pk = lookup_known_pk(&from); // Bob knows Alice's PK from prior interaction
falcon_verify(alice_pk, &expected_sig_msg, &sig_inner_bytes)
    .expect("signature verification failed — not from Alice or tampered");
```

Bob derives both nonces from `shared_secret` alone (no `seq` needed). `seq` is extracted from the decrypted `from_padded` block for application-level dedup. Falcon signature proves authenticity — Bob verifies it after decryption. Mailbox never sees the signature.

---

## 4. Encrypted `from` Block Layout

`encrypted_from_b64` contains: `from` + `seq` + `falcon_sig_inner`, all padded to a fixed total length. Mailbox sees one fixed-size ciphertext regardless of actual content.

```rust
const FIXED_FROM_LEN: usize = 64;   // from + seq + padding
const SIG_LEN_PREFIX: usize = 2;    // u16 length prefix for sig
const FIXED_SIG_LEN: usize = 700;   // 2-byte prefix + max ~690 bytes Falcon-1024 sig + padding
const FIXED_FROM_BLOCK_LEN: usize = FIXED_FROM_LEN + FIXED_SIG_LEN; // 764 bytes total

fn build_from_block(from: &str, seq: u64, sig_inner: &[u8]) -> Vec<u8> {
    let mut block = vec![0u8; FIXED_FROM_BLOCK_LEN];

    // from + seq (64 bytes):
    let from_bytes = from.as_bytes();
    assert!(from_bytes.len() <= FIXED_FROM_LEN - 8, "from too long");
    block[..from_bytes.len()].copy_from_slice(from_bytes);
    block[FIXED_FROM_LEN - 8..FIXED_FROM_LEN].copy_from_slice(&seq.to_le_bytes());

    // sig length prefix (2 bytes LE u16) + sig (variable) + zero padding:
    let sig_len = sig_inner.len() as u16;
    assert!((sig_len as usize) <= FIXED_SIG_LEN - SIG_LEN_PREFIX, "sig too long");
    block[FIXED_FROM_LEN..FIXED_FROM_LEN + 2].copy_from_slice(&sig_len.to_le_bytes());
    block[FIXED_FROM_LEN + 2..FIXED_FROM_LEN + 2 + sig_inner.len()].copy_from_slice(sig_inner);

    block
}

fn parse_from_block(block: &[u8]) -> (String, u64, &[u8]) {
    // from + seq:
    let from_padded: [u8; FIXED_FROM_LEN] = block[..FIXED_FROM_LEN].try_into().unwrap();
    let seq = u64::from_le_bytes(from_padded[FIXED_FROM_LEN - 8..].try_into().unwrap());
    let from_end = from_padded[..FIXED_FROM_LEN - 8]
        .iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
    let from = String::from_utf8_lossy(&from_padded[..from_end]).to_string();

    // sig via length prefix (NOT trimming zeros — Falcon sig can legally end with 0x00):
    let sig_len = u16::from_le_bytes(
        block[FIXED_FROM_LEN..FIXED_FROM_LEN + 2].try_into().unwrap()
    ) as usize;
    let sig_inner = &block[FIXED_FROM_LEN + 2..FIXED_FROM_LEN + 2 + sig_len];

    (from, seq, sig_inner)
}
```

Note: The 2-byte length prefix is critical. Zero-trimming (`rposition(|&b| b != 0)`) would silently corrupt Falcon signatures that legally end with `0x00` bytes. The length prefix makes parsing deterministic regardless of signature content.

`encrypted_from_b64` is always the same size (764 bytes plaintext → fixed ciphertext size). Mailbox cannot distinguish sender by length, and cannot see the signature.

---

## 5. PoW Anti-Spam

### 5.1 PoW verification (mailbox side)

```rust
fn verify_pow(envelope: &NxmsEnvelopeV3, required_difficulty: u8) -> bool {
    let challenge = blake3(
        "privai:pow:v1"
        || envelope.kem_ct_b64.as_bytes()
        || envelope.encrypted_from_b64.as_bytes()
        || envelope.ciphertext_b64.as_bytes()
        || envelope.pow_nonce.to_le_bytes()
    );
    count_leading_zero_bits(&challenge) >= required_difficulty
}
```

`required_difficulty` is from mailbox's own config, NOT from the envelope. Client cannot bypass by setting low difficulty. Verification cost: ~1μs (one blake3 hash). Generation cost at 16 bits: ~100ms-1s on CPU, ~10ms on GPU.

### 5.2 Difficulty

**V0: Static.** Mailbox admin sets difficulty. Exposed via `/info` endpoint:

```json
{
  "ok": true,
  "proto": "NXMS/3",
  "pow_difficulty": 16,
  "max_body_bytes": 16777216,
  "default_ttl_secs": 300,
  "max_ttl_secs": 3600
}
```

Clients query `/info` on startup, use returned `pow_difficulty` for envelope generation. No hardcoding.

**Future: Dynamic.** Mailbox measures queue depth, adjusts difficulty. More spam → higher difficulty. Not in V0 scope.

### 5.3 PoW and dedup

The `pow_nonce` is part of the envelope hash (see §6). Replaying an identical envelope with the same `pow_nonce` is caught by dedup. Retrying requires a new `pow_nonce` → new PoW → new hash → passes dedup.

---

## 6. Dedup: Hash-Based at Mailbox

Mailbox deduplicates by hash of the full envelope:

```rust
fn envelope_dedup_hash(env: &NxmsEnvelopeV3) -> [u8; 32] {
    blake3(
        "privai:dedup:v1"
        || env.to.as_bytes()
        || env.kem_ct_b64.as_bytes()
        || env.encrypted_from_b64.as_bytes()
        || env.nonce_from_b64.as_bytes()
        || env.tag_from_b64.as_bytes()
        || env.nonce_payload_b64.as_bytes()
        || env.ciphertext_b64.as_bytes()
        || env.tag_payload_b64.as_bytes()
        || env.pow_nonce.to_le_bytes()
    )
}
```

Note: No `sig_b64` in dedup hash — signature is inside encrypted block, not on envelope exterior.

**Why this works:**
- Mailbox doesn't know `from` — privacy preserved
- Identical envelope (same `pow_nonce`) → same hash → rejected as duplicate
- Retry with new `pow_nonce` → new hash → passes dedup
- Different sender, same content → different `kem_ct` (fresh FrodoKEM) → different hash → passes

**Storage:** Mailbox keeps a HashSet/bloom filter of recent envelope hashes. Entries expire with TTL. Zero knowledge of content or sender.

**Migration from current dedup:** Current dedup is per `(to, from, escrow_id, seq)` in `db.rs:238`. New dedup replaces this with hash-based. The `from_id` column in the `messages` table becomes unused — store `envelope_dedup_hash` instead.

---

## 7. Threat Model

### 7.1 What mailbox sees after this change

| Observable | Privacy impact |
|------------|---------------|
| `to` | Who receives messages. Required for routing. |
| `kem_ct` size | Fixed (~10,880 bytes). No sender info. |
| `encrypted_from` size | Fixed (padded to 64 bytes). No sender info. |
| `ciphertext` size | Payload size. May leak message type. |
| Source IP | Network-level. Mitigated by Tor/proxy on client side. |
| Timing | When messages arrive. Mitigated by traffic padding (future). |
| Volume | How many messages per inbox. Visible aggregate. |

### 7.2 What mailbox CANNOT see

- Sender identity (`from`)
- Sequence number (`seq`)
- Falcon signature (inside `encrypted_from` — mailbox cannot verify against any key)
- Message content (`payload`)
- Whether two messages are from the same sender (different FrodoKEM encapsulations per message)

### 7.3 Known residual risks

| Risk | Mitigation | Status |
|------|-----------|--------|
| Traffic analysis (timing/volume) | Tor on client side, traffic padding | V0: Tor. Future: padding. |
| IP correlation | Client uses Tor/proxy | Client responsibility |
| `to` field visibility | Required for routing. Future: PIR or onion delivery | Not in V0 |
| GPU-accelerated PoW | Dynamic difficulty adjustment | Future |
| Spam from botnet (many IPs) | IP-based rate limiting still applies | Existing |

---

## 8. Impact on Existing Code

### 8.1 Files to modify

| File | Change |
|------|--------|
| `nxms-transport/src/wire.rs` | Add `NxmsEnvelopeV3` struct. Remove `sig_b64` and `sig_id` from V3. Keep V1/V2 for backward compat. |
| `nxms-mailbox/src/db.rs` | Replace `from_id`-based dedup with hash-based dedup. Add `envelope_hash` column. |
| `nxms-mailbox/src/api.rs` | Add PoW verification in `push()`. Add `/info` endpoint with `pow_difficulty`. |
| `nxms-mailbox-client/src/lib.rs` | Support V3 envelope construction. PoW generation. Build `from_block` with inner sig. |
| `nxms-signer/src/agent/transport.rs` | Decrypt V3 envelopes: extract `from` + `seq` + verify inner Falcon sig. |
| `privai-nxms/src/discovery.rs` | `DiscoveryQuery` wrapped in V3 envelope. |

### 8.2 Backward compatibility

- V1 and V2 envelopes continue to work (plaintext `from`)
- Mailbox accepts both V2 and V3
- Clients negotiate version via `/info` endpoint
- V3 clients can send to V2 recipients (V2 recipient's mailbox sees plaintext `from`)
- Migration: V3-only mode is a mailbox config flag

### 8.3 Database migration

```sql
-- Add dedup hash column, keep from_id for backward compat during migration
ALTER TABLE messages ADD COLUMN envelope_hash_hex TEXT;
CREATE INDEX idx_envelope_hash ON messages(envelope_hash_hex);
```

---

## 9. Non-goals

1. **No PIR (Private Information Retrieval).** `to` remains visible. PIR is heavy research, not V0.
2. **No traffic padding.** Timing/volume analysis remains possible. Client-side Tor mitigates.
3. **No dynamic PoW difficulty.** Static at V0. Dynamic is future.
4. **No anonymous credential for mailbox access.** Push token auth remains. Future: blind tokens.
5. **No onion routing for mailbox delivery.** Single mailbox hop. Relay onion routing is separate concern.

---

## 10. Open Items

1. ~~**Falcon signature scope.**~~ **RESOLVED.** Signature is inside `encrypted_from` block. Mailbox never sees it. Bob verifies after decryption. Signed message: `blake3("privai:envelope:v1" || from_padded || payload_hash)`.
2. ~~**`pow_difficulty` in envelope.**~~ **RESOLVED.** Removed from envelope. Mailbox verifies against its own config. Client cannot game by setting low value. Difficulty exposed via `/info` endpoint for client consumption.
3. **PoW difficulty value.** 16 bits is proposal. Benchmark on target hardware before freeze.
4. **Bloom filter vs HashSet for dedup.** Bloom filter is memory-efficient but has false positives (may reject valid unique envelopes). HashSet is exact but grows with messages. Decision: HashSet with TTL-based eviction for V0.
5. **V2→V3 migration timeline.** How long to support both? Proposal: V2 deprecated at devnet launch, removed at testnet.

---

## 11. Test Plan

| Test | Description |
|------|-------------|
| `v3_roundtrip` | Alice encrypts, Bob decrypts. `from` and `payload` match. |
| `v3_mailbox_cannot_see_from` | Mailbox receives V3 envelope. `encrypted_from_b64` is opaque. Cannot extract `from`. |
| `v3_sig_not_linkable` | Mailbox has Alice's Falcon PK. Cannot determine Alice is sender — `sig` is inside encrypted block, not on envelope exterior. `falcon_verify(alice_pk, envelope_bytes, any_field)` fails for all visible fields. |
| `v3_fixed_size_encrypted_from` | Different peer_id lengths produce same-size `encrypted_from_b64`. |
| `v3_dedup_rejects_replay` | Same envelope hash → second push rejected. |
| `v3_dedup_allows_retry` | Same content, different `pow_nonce` → different hash → accepted. |
| `v3_pow_verification` | Valid PoW → accepted. Invalid PoW → rejected. |
| `v3_backward_compat_v2` | V2 envelope still works alongside V3. |
| `v3_cross_version` | V3 sender to V2 recipient. V2 mailbox sees plaintext `from` (expected). |
| `v3_seq_in_encrypted_from` | Bob decrypts, extracts correct `seq`. |
| `v3_bob_verifies_inner_sig` | Bob decrypts, extracts Falcon sig from `from_block`, verifies against Alice's PK. Tampered payload → sig verification fails. |
| `v3_sig_trailing_zero` | Falcon signature ending with `0x00` bytes is correctly preserved after build/parse round-trip (length prefix, not zero-trimming). |

---

*Wersja: 2026-04-12. Hidden sender spec for NXMS Envelope V3. One KEM, two nonces, padded from, PoW anti-spam, hash-based dedup.*
