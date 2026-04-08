# Transport / P2P Master Tasks

Status: execution board for validator transport and P2P hardening.
Canonicality: non-canonical implementation tracker. This file does not override protocol/spec docs. It exists to coordinate parallel agent work without task collisions.
Scope:
- `privai-node/src/session_impl.rs`
- `privai-node/src/session_transport.rs`
- `privai-node/src/identity_provider.rs`
- `privai-node/src/config.rs`
- `privai-node/src/node.rs`
- `privai-node/src/main.rs`
- `privai-node/tests/validator_session.rs`
- `nxms-transport/src/tor_net.rs`
- `nexum-cli/src/vault.c`

## 1. Current Baseline

### Already fixed locally
- [x] Incoming decrypt gap in `session_impl.rs`
- [x] Handshake transcript hardening (`Challenge -> Init -> Response`)
- [x] Shared frame defaults reduced in `nxms-transport/src/tor_net.rs`
- [x] Ban poisoning removed from unauthenticated handshake branches
- [x] Plaintext fallback removed from `ConnectionPool::send_message()`
- [x] Short server-side write timeouts for `Challenge` / `Response`
- [x] Tracker/comment drift cleanup

### Still open
- [ ] `validator_session` tests still target old handshake shape
- [ ] `identity_provider.rs` TLV parser is inconsistent with `nexum-cli/src/vault.c`
- [ ] transport KEM keys are not wired end-to-end into runtime startup
- [ ] session transport does not fail fast on placeholder / zero transport keys
- [ ] rate limiter semantics are stronger in comments/docs than in reality
- [ ] frame-level replay / ordering semantics are still missing
- [ ] failed-handshake cooldown / anti-spam policy is still missing
- [ ] outgoing path still lacks ban/cooldown policy review

## 2. Coordination Rules

- Tasks marked `parallel-safe` may run at the same time.
- Tasks marked `depends on` must wait for the listed prerequisite.
- Agents must not widen scope beyond listed files.
- No task may revert or redesign the current handshake v2 model.
- No task may touch `crypto/*`, `contracts/*`, `keys/*`, `withdrawals/*`.
- Runtime correctness beats docs cleanup. Docs should only be updated when the code change is already done.

## 3. Parallel Work Lanes

### Lane A: Tests and Validation
- focus: test harness and regression pack
- primary files:
  - `privai-node/tests/validator_session.rs`

### Lane B: Identity / Vault / Startup Keys
- focus: TLV parser correctness and transport key wiring
- primary files:
  - `privai-node/src/identity_provider.rs`
  - `privai-node/src/node.rs`
  - `privai-node/src/config.rs`
  - `privai-node/src/main.rs`
  - `nexum-cli/src/vault.c`

### Lane C: Session Hardening
- focus: remaining `session_impl.rs` transport/runtime hardening
- primary files:
  - `privai-node/src/session_impl.rs`
  - `privai-node/src/session_transport.rs`

### Lane D: Docs / Tracking
- focus: trackers, checklists, invariants sync
- primary files:
  - `spec/nxms_transport_p2p/*.md`
  - `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
  - `spec/PRIVAI_VALIDATOR_SESSION_TEST_PLAN.md`

## 4. Ready Tasks

### Task 01
Status: ready
Parallel-safe: yes
Lane: A
Title: Update `validator_session` tests to handshake v2

Files:
- `privai-node/tests/validator_session.rs`

Goal:
- migrate test helpers and handshake tests from old `HandshakeMsg` struct assumptions to the current `Challenge -> Init -> Response` flow

Context:
- production `HandshakeMsg` is now an enum in `privai-node/src/session_impl.rs`
- tests still build old-style `HandshakeMsg { version, kem_pk_b64, ... }`
- runtime is already on handshake v2; tests must catch up without changing runtime code

Required work:
- [ ] rewrite handshake helpers in `validator_session.rs`
- [ ] read `Challenge` first, then build signed `Init`
- [ ] interpret `Response` as success path
- [ ] preserve wrong-version test semantics
- [ ] preserve bad-signature test semantics
- [ ] keep non-handshake tests stable unless compilation requires minimal edits
- [ ] run `cargo check --test validator_session` or `cargo test --test validator_session --no-run`

Must not:
- modify `privai-node/src/session_impl.rs`
- revert handshake v2
- export private production helpers just to satisfy tests

Acceptance:
- [ ] `validator_session.rs` no longer assumes old `HandshakeMsg` struct
- [ ] test target compiles

### Task 02
Status: ready
Parallel-safe: yes
Lane: B
Title: Reconcile `identity_provider.rs` TLV parser with `vault.c`

Files:
- `privai-node/src/identity_provider.rs`
- `nexum-cli/src/vault.c`

Goal:
- make Rust parser match the actual current vault TLV format before any further key integration

Context:
- current parser has wrong tag constants
- current parser reads TLV header as `u16 type + u16 len`
- `vault.c` writes TLV as `u16 type + u32 len`

Required work:
- [ ] align TLV tag constants with `vault.c`
- [ ] change parser header layout to `u16 type + u32 len`
- [ ] parse `T_KEM_PK`
- [ ] parse `T_KEM_SK`
- [ ] preserve Falcon parsing
- [ ] add a narrow unit test or fixture-driven parser test if practical
- [ ] run `cargo check`

Must not:
- wire keys into startup/runtime yet
- guess undocumented TLV semantics
- touch session transport code

Acceptance:
- [ ] parser layout matches `vault.c`
- [ ] parser can read Falcon + KEM keys from the current format

### Task 03
Status: done
Parallel-safe: yes
Lane: C
Title: Add short server-side write timeouts for handshake writes

Files:
- `privai-node/src/session_impl.rs`

Goal:
- prevent incoming listener slots from hanging on challenge/response writes

Context:
- outgoing path already uses short handshake timeouts
- server-side `write_frame()` for `Challenge` and `Response` is still not wrapped in similar short write timeouts

Required work:
- [x] wrap `Challenge` write in a short timeout
- [x] wrap `Response` write in a short timeout
- [x] keep current handshake semantics unchanged
- [x] run `cargo check`

Must not:
- redesign handshake
- change message format
- combine with BanList / plaintext fallback changes

Acceptance:
- [x] both server-side handshake writes have bounded timeout behavior
- [x] handshake v2 remains semantically identical

### Task 04
Status: done
Parallel-safe: yes
Lane: D
Title: Sync session docs with handshake v2 and completed fixes

Files:
- `spec/PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `spec/PRIVAI_VALIDATOR_SESSION_TEST_PLAN.md`
- `spec/nxms_transport_p2p/TRANSPORT_P2P_FIX_TRACKER.md`

Goal:
- align docs with current code after decrypt-gap fix, handshake v2, ban-policy fix, and plaintext-fallback removal

Context:
- docs and tests still mention old handshake model in places
- tracker is current enough to serve as source material

Required work:
- [x] update invariants doc to reflect `Challenge -> Init -> Response`
- [ ] update test plan doc to reflect handshake v2 helpers and expectations
- [x] keep tracker consistent with current status

Must not:
- invent new protocol behavior not already in code
- block on unrelated transport hardening

Acceptance:
- [x] docs no longer describe old handshake shape as current

## 5. Dependent Tasks

### Task 05
Status: done
Depends on: Task 02
Parallel-safe: no
Lane: B
Title: Extend `PQCIdentity` with transport KEM keys

Files:
- `privai-node/src/identity_provider.rs`

Goal:
- carry full runtime identity material, not just Falcon vote keys

Required work:
- [x] add `kem_pk`
- [x] add `kem_sk`
- [x] keep secrets in `Zeroizing<Vec<u8>>`
- [x] preserve existing Falcon behavior
- [x] run `cargo check`

Acceptance:
- [x] `PQCIdentity` exposes Falcon + KEM material

### Task 06
Status: done
Depends on: Task 05
Parallel-safe: no
Lane: B
Title: Wire transport keys into node runtime and startup

Files:
- `privai-node/src/node.rs`
- `privai-node/src/config.rs`
- `privai-node/src/main.rs`
- `privai-node/src/session_transport.rs`

Goal:
- ensure session transport uses real loaded transport keys instead of placeholders

Required work:
- [x] load KEM keys into runtime config/state
- [x] ensure transport-facing code gets real `node_kem_pk/node_kem_sk`
- [x] preserve existing Falcon vote-key behavior unless explicitly improved as part of the same path
- [x] run `cargo check`

Acceptance:
- [x] runtime path no longer relies on placeholder transport keys

### Task 07
Status: blocked
Depends on: Task 06
Parallel-safe: no
Lane: B
Title: Add fail-fast guard for placeholder / zero transport keys

Files:
- `privai-node/src/main.rs`
- `privai-node/src/session_transport.rs`
- `privai-node/src/config.rs`

Goal:
- validator session transport must not start on empty / zero transport key material

Required work:
- [ ] detect empty/zero placeholder transport keys
- [ ] fail fast before listener/pool startup
- [ ] keep the error explicit and operator-readable
- [ ] run `cargo check`

Acceptance:
- [ ] no real startup path can silently proceed on placeholder transport keys

## 6. Later Hardening Tasks

### Task 08
Status: ready
Parallel-safe: yes
Lane: C
Title: Clarify or rework rate limiter semantics

Files:
- `privai-node/src/session_impl.rs`
- `spec/nxms_transport_p2p/TRANSPORT_P2P_FIX_TRACKER.md`
- optional docs under `spec/`

Goal:
- either make limiter semantics honest in docs/comments or improve implementation so it matches intended claims

Context:
- current limiter is keyed by `addr.to_string()`
- that is a listener pressure guard, not a true authenticated peer-level limiter

Required work:
- [ ] audit comments and assumptions
- [ ] either rename/document honestly or minimally improve semantics without redesign
- [ ] run `cargo check`

Acceptance:
- [ ] comments/docs no longer overclaim protection level

### Task 09
Status: ready
Parallel-safe: yes
Lane: C
Title: Add failed-handshake counters / cooldown policy

Files:
- `privai-node/src/session_impl.rs`
- optional tracker/docs updates

Goal:
- add cheap anti-spam pressure after handshake v2 without changing the core protocol

Required work:
- [ ] count repeated failed handshake attempts in a local transport-safe way
- [ ] add cooldown / refusal policy
- [ ] avoid `BanList(peer_id)` before identity proof
- [ ] run `cargo check`

Acceptance:
- [ ] repeated failed attempts are throttled without reintroducing ban poisoning

### Task 10
Status: ready
Parallel-safe: yes
Lane: C
Title: Add frame-level replay / ordering semantics

Files:
- `privai-node/src/session_impl.rs`
- tests if added locally

Goal:
- bind per-session sequencing into encrypted frames

Context:
- current XChaCha20-Poly1305 framing protects per-frame confidentiality/integrity
- it does not yet authenticate ordering or provide explicit anti-replay semantics

Required work:
- [ ] design minimal frame sequence model
- [ ] bind sequence into AAD
- [ ] reject duplicates / out-of-order frames according to the chosen model
- [ ] update tests/docs if the implementation is changed

Must not:
- redesign higher-level consensus semantics
- require wire changes outside validator session framing without explicit note

Acceptance:
- [ ] session framing has explicit sequence semantics

### Task 11
Status: ready
Parallel-safe: yes
Lane: C
Title: Review outgoing path policy symmetry

Files:
- `privai-node/src/session_impl.rs`
- `privai-node/src/session_transport.rs`
- tracker/docs if needed

Goal:
- review whether outgoing path should consult ban/cooldown policy and make it explicit

Required work:
- [ ] inspect current outgoing connect / establish path
- [ ] decide and implement whether outgoing path should consult `BanList` or cooldown state
- [ ] document the chosen policy

Acceptance:
- [ ] outgoing policy is explicit rather than accidental

## 7. Suggested Parallel Batches

### Batch 1
Can run together:
- Task 01
- Task 02
- Task 03
- Task 04

Why safe:
- they touch different files or only docs
- no direct code collision

### Batch 2
Can run after Batch 1:
- Task 05
- Task 08
- Task 09

Why safe:
- Task 05 depends on parser reconciliation
- Task 08 and Task 09 stay in session/runtime policy and do not need startup key wiring

### Batch 3
Can run after Task 05:
- Task 06

### Batch 4
Can run after Task 06:
- Task 07

### Batch 5
Can run later, after the above stabilizes:
- Task 10
- Task 11

## 8. Single-Page Operator Checklist

- [ ] Task 01 done
- [ ] Task 02 done
- [x] Task 03 done
- [x] Task 04 done
- [x] Task 05 done
- [x] Task 06 done
- [ ] Task 07 done
- [ ] Task 08 done
- [ ] Task 09 done
- [ ] Task 10 done
- [ ] Task 11 done

## 9. Notes

- Keep tasks small and file-local.
- Do not mix vault/parser work with session runtime work in the same agent task.
- Do not mix test migrations with protocol redesign.
- Do not treat placeholder transport keys as acceptable runtime state.
- If a task changes handshake/session semantics, update docs only after code and compilation succeed.
