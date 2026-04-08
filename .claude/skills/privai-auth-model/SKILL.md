---
name: privai-auth-model
description: Work on privAI transaction signing, signer semantics, threshold auth, mandatory `FullPrivacy` auth, and the split between `tx_id` and `tx_signing_hash`. Use when tasks touch signer ordering, duplicate signer handling, policy binding, escrow auth, or `nexum-core` coordination.
argument-hint: [scope]
---

# privAI Auth Model

Use this skill when working on transaction authorization, signer semantics, threshold auth, mandatory auth in `FullPrivacy`, or the split between `tx_id` and `tx_signing_hash`.

Treat `$ARGUMENTS` as extra scope, files, or a narrowed question.

## Read first

1. `spec/PRIVAI_EXECUTION_SPINE.md`
2. `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
3. `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
4. `spec/PRIVAI_PROTOCOL_CORE.md`
5. `spec/PRIVAI_CANONICAL_FORMATS.md`
6. `spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`
7. `spec/PRIVAI_GAP_REGISTER.md`
8. `spec/PRIVAI_DECISION_REGISTER.md`

If present, also read:

- `spec/PRIVAI_IMPLEMENTATION_PRIORITY_TASKS.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- local `nexum-core` auth docs or auth artifact code

## Primary code

- `privai-chain/src/tx.rs`
- `privai-chain/src/canonical.rs`
- `privai-chain/src/consensus.rs`
- `privai-chain/src/note.rs`
- `privai-ledger/src/ledger.rs`
- `privai-ledger/src/escrow.rs`

If a local `nexum-core` checkout exists, inspect its auth artifact and signer coordination paths too.

Useful repo search:

```bash
rg -n "tx_signing_hash|tx_id|signer|signature|threshold|auth|policy_opening|policy_tag|Escrow2of3|FullPrivacy" \
  privai-chain \
  privai-ledger \
  spec
```

## Core questions

- Is signing cyclic because `tx_id` includes auth bytes?
- Is there a canonical signing preimage independent of final auth bytes?
- Does `FullPrivacy v1` require auth for every input in `TransferNoteTx`?
- Is `policy_opening` required and bound to `spend_policy_commit` for every `FullPrivacy` input?
- Is `policy_type` derived from `policy_opening`, with `policy_tag` only a hint that must match?
- Is signer ordering canonical?
- Can duplicate signer material count more than once?
- Is signer identity bound clearly to the auth artifact?
- Are `Single` and `Escrow2of3` validated through the same per-input auth contract, then dispatched by derived policy type?
- Is the split between `nexum-core` coordination and ledger verification explicit?

## Guardrails

- Never allow `tx_id` to remain the signing message if auth feeds back into it.
- Do not allow empty-auth `TransferNoteTx` on `FullPrivacy v1`.
- Do not treat `policy_tag` as the source of truth over `policy_opening`.
- Do not allow `policy_tag` mismatch versus the type derived from `policy_opening`.
- Do not treat threshold auth aggregation as a replacement for ledger validation.
- Do not let product or workflow logic leak into canonical signer semantics.
- Keep marketplace and lite paths separate from `FullPrivacy` auth rules.
- Keep signer uniqueness, ordering, and identity binding explicit.

## Output

- Current problem
- Required canonical rules
- Per-input validation rules
- `FullPrivacy` scope and non-scope
- Implementation checklist
- Residual risks
