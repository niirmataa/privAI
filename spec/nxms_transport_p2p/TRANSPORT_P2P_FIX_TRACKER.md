# Transport / P2P Fix Tracker

Status: local working tracker for transport and validator P2P hardening.
Canonicality: non-canonical execution tracker. This file does not override protocol/spec docs. It exists to keep transport/P2P fixes out of the main spec root while we drive implementation and review.
Scope:
- `privai-node/src/session_impl.rs`
- `privai-node/src/session_transport.rs`
- `privai-node/src/identity_provider.rs`
- `privai-node/src/config.rs`
- `privai-node/src/node.rs`
- `nxms-transport/src/tor_net.rs`
- `nexum-cli/src/vault.c`

## 1. Current State

### Already fixed locally
- [x] Incoming decrypt gap in `session_impl.rs`
  - listener now keeps the FrodoKEM-derived shared secret
  - incoming frames are decrypted before `ConsensusMsg` deserialization
- [x] Handshake transcript hardening in `session_impl.rs`
  - challenge-response flow
  - signed `Response`
  - verification against `PeerBook` / dialed peer keys instead of blindly trusting keys from the handshake message
- [x] Ban poisoning through unauthenticated `peer_id`
  - Removed `ban_list.ban(&peer_id)` calls before identity proof (e.g. unknown peer, key mismatch, invalid signature) to prevent DoS by claiming someone else's ID.
- [x] Plaintext fallback in `send_message()`
  - Removed plaintext fallback in `ConnectionPool::send_message()`. It now returns a hard error `NetError::Transport` if `shared_secret` is missing.
- [x] Short server-side write timeouts
  - Added short (10s) timeouts around server-side handshake writes (`Challenge` and `Response`) in `run_listener`.

### Still open
- [ ] Transport key integration from vault into `NodeConfig`
- [x] Rate limiter semantics for Tor / hidden-service reality
- [x] Frame-level replay / ordering semantics after session establishment

## 2. Confirmed Issues

### 2.1. Ban poisoning on incoming handshake

Problem:
- current incoming handshake path can still call `ban_list.ban(&peer_id)` before the claimed `peer_id` is cryptographically authenticated
- this allows griefing / denial of service against a legitimate validator identity

Rule:
- `BanList(peer_id)` only after cryptographic confirmation of identity

Immediate policy:
- unknown `peer_id` -> drop only
- key mismatch with `PeerBook` -> drop only
- invalid Falcon signature during handshake -> drop only
- optional: increment local attempt counters keyed by transport source / local socket context

### 2.2. Plaintext fallback in `send_message()`

Problem:
- `ConnectionPool::send_message()` still has:
  - encrypt if `shared_secret` exists
  - otherwise send plaintext

Why this is bad:
- in the current validator session model, missing `shared_secret` is not a fallback state
- it is a broken session state machine

Rule:
- no plaintext post-handshake validator traffic

Immediate policy:
- if `shared_secret` is missing when sending over a validator session, return a hard session-state error

### 2.3. Identity / vault integration is not finished

Problem:
- `NodeConfig` still contains placeholder transport keys in example/default flows
- `identity_provider.rs` currently loads Falcon material, but transport KEM integration is not finished end-to-end

Critical new finding:
- `privai-node/src/identity_provider.rs` claims tag mapping “as per `nexum-cli/src/vault.c`”
- current constants there do not match `nexum-cli/src/vault.c`

Observed mismatch:
- `identity_provider.rs`
  - `T_KEM_SK = 0x0004`
  - `T_FALCON_SK = 0x0006`
  - `T_FALCON_PK = 0x0007`
- `nexum-cli/src/vault.c`
  - `T_FALCON_SK = 3`
  - `T_FALCON_PK = 4`
  - `T_KEM_SK = 5`
  - `T_KEM_PK = 6`

Implication:
- before extending key integration, we must first reconcile the TLV parser with the actual current vault format

Rule:
- `ValidatorSessionTransport` must not start on placeholder transport keys

### 2.4. Rate limiter semantics are weaker than comments suggest (FIXED)

Problem:
- limiter is keyed by `addr.to_string()`
- comments suggest per-peer / per-onion protection
- that is stronger than what the code can actually guarantee

Current reality (Fixed):
- renamed `RateLimiter` to `ListenerPressureGuard` to honestly reflect its semantics as a listener pressure guard
- removed misleading comments about authenticated peer-level limits

### 2.5. Session frames still lack replay / ordering semantics (FIXED)

Problem:
- `encrypt_frame()` / `decrypt_frame()` give confidentiality and integrity per frame
- they do not yet bind a per-session frame sequence into AAD

Current consequence (Fixed):
- AAD now strictly binds a `tx_seq` / `rx_seq` counter
- explicitly provides authenticated ordering and anti-replay per session.

## 3. Priority Order

### Immediate
- [x] Remove all `BanList(peer_id)` updates on unauthenticated handshake failures
- [x] Remove plaintext fallback from `ConnectionPool::send_message()`
- [ ] Reconcile `identity_provider.rs` TLV tags with `nexum-cli/src/vault.c`

### Next
- [ ] Extend `PQCIdentity` to carry transport KEM keys
- [ ] Wire real `kem_pk` / `kem_sk` into `NodeConfig` before transport startup
- [ ] Add guard: no session transport startup on placeholder / zero transport keys

### Later hardening
- [x] Rework limiter semantics or rename/document them honestly
- [x] Add failed-handshake counters / cooldown
- [x] Add per-frame sequence/AAD anti-replay model

## 4. Agent-Safe Task Suggestions

### Task A: Ban policy + plaintext fallback
Safe, local, no architecture change:
- remove `ban_list.ban(&peer_id)` from unauthenticated handshake failure branches
- convert plaintext fallback in `send_message()` into a hard error
- run `cargo check`

### Task B: TLV parser reconciliation
Safe if done carefully:
- compare `identity_provider.rs` tag constants against `nexum-cli/src/vault.c`
- fix tag constants
- add parsing for `T_KEM_PK`
- report any ambiguity instead of guessing

### Task C: Transport key wiring
Depends on Task B:
- extend `PQCIdentity`
- wire transport keys into `NodeConfig`
- fail fast if transport keys are missing / placeholder

## 5. Notes

- Do not treat unknown `peer_id` as authenticated identity.
- Do not use `BanList` as a generic handshake error sink.
- Do not keep “fallback plaintext” in validator transport; it hides broken state.
- Do not assume `identity_provider.rs` matches vault format until tag mapping is verified against current `nexum-cli/src/vault.c`.
