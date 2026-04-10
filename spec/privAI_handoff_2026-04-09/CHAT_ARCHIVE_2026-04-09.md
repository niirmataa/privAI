# privAI Chat Archive — 2026-04-09

Status: detailed reconstructed archive of the conversation and project state.
Important note: this is not a byte-for-byte export of the chat UI. It is an intentionally structured reconstruction of what was discussed, decided, implemented, reviewed, and left open, written so that another operator can recover the full thread of reasoning without guessing.

## 1. What This Conversation Was Really About

This conversation was not just about a few bugfixes or test repairs. It was a long-running architecture-and-execution thread focused on taking `privAI` escrow from an inconsistent, partially implied design into a first coherent v1 system shape.

The work spanned:
- docs creation and anti-drift correction,
- architectural decisions around privacy/auth/proof boundaries,
- control-plane vs execution-plane separation,
- Stage A vs Stage B separation,
- wallet final assembly,
- proof handoff,
- node staging, submit gate, and e2e validation,
- transport split between validator P2P and NXMS/mailbox control-plane,
- practical orchestration of multiple external models (Xiaomi, Gemini, Claude) plus one internal subagent (Ampere).

The central goal became:

`funded note -> Stage A approvals -> Stage B assembly -> proof handoff -> node submit gate -> block/import-ready path`

That path now exists locally and honestly for the release flow.

## 2. The Core Architectural Corrections

### 2.1. FullPrivacy / Auth Model

The project converged on the following:
- `FullPrivacy v1` uses Option B.
- All `TransferNoteTx` inputs require auth.
- `policy_opening` is mandatory.
- `policy_tag` is only a hint and must agree with `policy_opening`.

This matters because the system must never derive sensitive authorization semantics from a tag alone when the real policy opening exists.

### 2.2. Escrow Model

Escrow is modeled as `Escrow2of3` on FullPrivacy.
Roles:
- Buyer
- Merchant
- Operator

Important conceptual decision:
- Operator is a workflow machine, not a trust anchor.

### 2.3. Stage A vs Stage B

A major part of the conversation was about correcting a false model where Stage A acted as if it already knew the final canonical signing object.

Correct model:
- Stage A = proposal, approvals, quorum, authorization material, control-plane objects
- Stage B = final tx assembly, final auth insertion, final canonical signing context, final `tx_signing_hash`, proof handoff

The project repeatedly removed false assumptions that Stage A already knows or signs the final canonical `tx_signing_hash`.

### 2.4. `nexum-core` Boundary

Another repeated clarification:
- `nexum-core` is control-plane
- it is not the execution engine
- it must not pretend to compute or own final tx assembly semantics

### 2.5. Transport Split

The conversation also had to separate two transport models that were easy to conflate:

Model A:
- validator / node-to-node direct P2P
- session transport
- handshake
- encrypted frames

Model B:
- NXMS control-plane / mailbox / Tor store-and-forward
- payload envelopes
- asynchronous delivery

Final recommended v1 split:
- validators = direct P2P session transport
- NXMS control-plane = mailbox store-and-forward over Tor
- proof sidecar = mailbox path, not validator queue
- gossip = validator session transport

This was later written down in the transport freeze memo.

## 3. What Was Built or Repaired

### 3.1. Docs / Anti-Drift

Several docs were corrected so they no longer implied false semantics.
The most important docs were:
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `/home/nxms-server/privAI/spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`

A specific mini-fix corrected the description of `EscrowApprovalBundle` so it no longer claimed Stage A was carrying final signatures over a final tx hash that did not yet exist.

### 3.2. `nexum-core` Orchestrator Cleanup

The false Stage A claim about canonical `tx_signing_hash` was removed.
The orchestrator/control-plane was split so that:
- it carries proposal/auth material/final assembly inputs,
- but does not itself claim ownership of final canonical tx hash computation.

Commit already present:
- `0f6ea09` `refactor(escrow-orchestrator): split stage-a from final assembly`

### 3.3. `privai-nxms` Escrow Wire Layer

The wire layer was hardened and then cleaned up.
Delivered:
- canonical roundtrip tests,
- payload roundtrip tests,
- msg_type consistency tests,
- body_hash determinism tests.

Then the old Stage A drift was removed:
- `EscrowSpendProposal` no longer contains `tx_signing_hash`.

That was critical because proposal is a Stage A object, not a final signing object.

### 3.4. `privai-node` Stage A Runtime

The node side gradually became real and testable:
- funded/proposal/approval staging
- quorum readiness
- payload ingress via `handle_nxms_payload(...)`
- persistence of Stage A store
- export of approval bundle via `build_escrow_approval_bundle(...)`

Approval-layer hardening added:
- deterministic ordering
- duplicate signer rejection
- signer/signature count validation
- Stage A sentinel hash (`TX_SIGNING_HASH_STAGE_A`)

### 3.5. Node Escrow Submit Gate

A major milestone was the escrow-aware submit gate.
The node can now validate a final escrow tx against staged control-plane context before delegating to the ordinary submit path.

Final gate checks:
- proposal exists
- quorum exists
- transaction is `TransferNote`
- exactly one `Escrow2of3` auth entry exists
- `policy_opening` is present
- `policy_opening` decodes successfully
- decoded policy type is `SpendPolicy::Escrow2of3`
- decoded policy fields match staged `EscrowFundingDescriptor`:
  - buyer_pk_hash
  - merchant_pk_hash
  - operator_pk_hash
  - timeout_block
- `escrow_action` matches the staged proposal action
- signer set matches the Stage A approval bundle
- signatures verify against `tx_signing_hash`

The positive tests were eventually fixed so they seed ledger state correctly and require real `Ok(...)`, not just "passed the gate but failed later".

### 3.6. Wallet Final Assembly

The wallet side got a real final assembly bridge.
Important corrections along the way:
- bind to `funding_note_commit`
- enforce action consistency
- reject duplicate/conflicting signers
- stop silently deduping bad signer sets
- use the proper wallet builder path instead of an ad hoc skeleton

Then two amount narrowing bugs were fixed:
- in `escrow_builder.rs`
- in `builder.rs`

That ensured downcasts fail explicitly rather than relying on later incidental failure.

### 3.7. Typed Proof/Submit Bridge

A typed Stage B handoff was created:
- `EscrowProofReadyHandoff`
- `EscrowAttachedProof`

It bridges:
- final `TransferNoteTx`
- `tx_signing_hash`
- `TransferProvingData`
- `ProofJob`
- final proof attachment into `BatchProofArtifact`
- export to `BlockProofArtifacts`

At one point a WSL crash left only a stub placeholder in `proof_handoff.rs`, but the file was restored to a real implementation and revalidated.

### 3.8. Validator Session Regression Pack

A strong test pack was added around validator session transport.
It now covers:
- transcript mismatch
- nonce mismatch
- wrong version
- bad Falcon signature
- peer identity mismatch
- encrypted frame roundtrip
- tamper rejection
- seq/AAD rejection
- replay-ish paths
- stale/rebuild behavior
- no mailbox dependency
- gossip-adjacent transport assumptions

This reinforces the split between validator P2P and mailbox path.

### 3.9. Honest Local Escrow E2E Release Flow

This is one of the most important achievements in the conversation.
Eventually, a local e2e release test was made to pass honestly.
It now goes through:
- Stage A funding
- proposal
- approvals
- node bundle export
- wallet final assembly
- final signatures over `tx_signing_hash`
- proof handoff
- proof attach
- node submit gate
- block propose path
- block artifact import path

Honest success conditions:
- funding note becomes spent
- ledger height advances
- proof artifacts are stored and match the block

This means the system now has a real local release flow instead of only disconnected pieces.

## 4. Commits Already Present

### In `privAI`
- `06f2f32` `build(workspace): centralize rust version metadata`
- `8011197` `feat(escrow-stage): harden stage-a wire and node flow`
- `e73e1ce` `feat(wallet): add escrow assembly builder`
- `54ab473` `build(lockfile): refresh escrow dependency graph`
- `e0d5931` `feat(node): add escrow submit gate`
- `1d1ef40` `feat(wallet): add escrow proof handoff`
- `fef827f` `docs(spec): add transport runtime freeze memo`
- `90817e4` `docs(coordination): add agent task pack and orchestration context`

### In `nexum-core`
- `0f6ea09` `refactor(escrow-orchestrator): split stage-a from final assembly`

## 5. Important Coordination Files Already Written

Files already created inside the repo for handoff and coordination:
- `/home/nxms-server/privAI/zadania_12_14_opis.md`
- `/home/nxms-server/privAI/KONTEKST_ORCHESTRACJI_2026-04-09.md`
- `/home/nxms-server/privAI/XIAOMI_HANDOFF_I_SZABLON_2026-04-09.md`

These are complementary, not replacements for this archive.

## 6. Agent Behavior / Orchestration Lessons

### Xiaomi
Works well when given:
- hard write scope
- hard forbidden list
- hard DoD
- exact report format

Xiaomi delivered multiple nontrivial runtime tasks successfully once the prompts became contract-like rather than vague.

### Gemini
Worked well on:
- orchestrator boundary cleanup
- Stage B typed proof handoff
- honest escrow e2e

### Claude
Was used for architecture/docs and delivered a useful transport/runtime freeze memo.

### Ampere
Internal subagent used for reconnaissance around proof/submit bridge. Helped identify that the biggest missing piece was the typed glue between wallet assembly, proving data, proof job, and proof result attachment.

## 7. What Is Still Open

The biggest remaining runtime blocker from this whole thread is:

### Mailbox runtime loop in `privai-node`

There is already promising work in the tree:
- `/home/nxms-server/privAI/privai-node/src/config.rs`
- `/home/nxms-server/privAI/privai-node/src/lib.rs`
- `/home/nxms-server/privAI/privai-node/src/mailbox_pull.rs`
- `/home/nxms-server/privAI/privai-node/Cargo.toml`
- `/home/nxms-server/privAI/Cargo.lock`

The code appears to define:
- `MailboxPullConfig`
- `MailboxSource`
- `mailbox_ingest_tick(...)`
- `run_mailbox_pull_loop(...)`
- explicit v1 ack policy

But this task was not yet closed honestly because test verification became entangled with WSL/toolchain instability.

So the correct status is:
- not rejected
- not assumed done
- pending final verification and likely small finishing work

## 8. Environment Problems That Distorted Signals

A major practical thread in the conversation was that WSL itself became unreliable.
Two distinct issues appeared:

### 8.1. Broken user mapping
Errors like:
- `getpwnam(nxms-privAI)`
- `getpwuid(...)`

This made WSL startup inconsistent.

### 8.2. Wrong Cargo when running as root
As `root`, the system often used:
- `/usr/bin/cargo`
- `cargo 1.83.0`

But the project requires the toolchain from:
- `/home/nxms-server/privAI/rust-toolchain.toml`
- channel `1.94.1`

This caused misleading failures such as:
- workspace parse failures due to `edition2024`

So some apparent test failures or hangs were not trustworthy project signals.

Known workaround when forced to run as root:

```sh
HOME=/home/nxms-privAI PATH=/home/nxms-privAI/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin cargo ...
```

## 9. Current Reality / Honest Status Board

### Closed / verified
- orchestrator Stage A/B boundary cleanup
- docs anti-drift and bundle semantics correction
- wire cleanup in `privai-nxms`
- node staging / ingress / persistence / bundle export
- approval hardening
- node escrow submit gate
- wallet final assembly bridge
- typed proof handoff
- validator session regression pack
- honest local escrow e2e release flow
- transport/runtime freeze memo

### Still open
- mailbox runtime loop final verification and close-out
- broader runtime stabilization around mailbox path
- environment cleanup or migration away from unreliable WSL state

## 10. Recommended Next Step

Do not widen scope.
The most practical next move is:
1. finish and verify the mailbox runtime loop,
2. commit only the relevant runtime files,
3. keep the environment problem separate from application logic,
4. only then decide whether to migrate to a cleaner remote Linux setup.

## 11. If Someone New Reads Only One File After This

If a future operator wants to understand the whole thread without reading the raw chat, they should read in this order:
1. this archive
2. `/home/nxms-server/privAI/PRIVAI_PROJECT_ENTRYPOINT.md` if present
3. `/home/nxms-server/privAI/KONTEKST_ORCHESTRACJI_2026-04-09.md`
4. `/home/nxms-server/privAI/zadania_12_14_opis.md`
5. `/home/nxms-server/privAI/XIAOMI_HANDOFF_I_SZABLON_2026-04-09.md`
6. the source-of-truth specs listed earlier
