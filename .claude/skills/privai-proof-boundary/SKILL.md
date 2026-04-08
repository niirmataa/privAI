---
name: privai-proof-boundary
description: Review or define privAI proof semantics, statement boundaries, public inputs, and ledger-vs-proof responsibilities. Use when tasks touch `privai-proof`, `TransferNoteTx`, `ExecutionBundle`, `ProofCertificate`, `Escrow2of3`, or proof coverage claims.
argument-hint: [scope]
---

# privAI Proof Boundary

Use this skill when working on `privai-proof`, proof semantics, statement or public-input rules, or the ledger-vs-proof boundary.

Treat `$ARGUMENTS` as the narrowed proof scope.

## Read first

1. `spec/PRIVAI_EXECUTION_SPINE.md`
2. `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
3. `spec/PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
4. `spec/PRIVAI_PROOF_BOUNDARIES.md`
5. `spec/PRIVAI_EXECUTION_BUNDLE_SEMANTICS.md`
6. `spec/PRIVAI_PROTOCOL_CORE.md`
7. `spec/PRIVAI_CANONICAL_FORMATS.md`
8. `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
9. `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
10. `spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`

## Primary code

- `privai-proof/src/transfer.rs`
- `privai-proof/src/batch.rs`
- `privai-proof/src/artifact.rs`
- `privai-proof/src/verify.rs`
- `privai-proof/src/halo2/circuits/tx_skeleton.rs`
- `privai-chain/src/consensus.rs`
- `privai-proof/tests/transfer_note_proof_semantics.rs`
- `privai-proof/tests/execution_bundle_unsupported_types.rs`

Useful repo search:

```bash
rg -n "TransferNoteTx|ExecutionBundle|ProofCertificate|public_inputs_hash|LiteTransfer|MarketplaceBatchTx|OnChainLite|Escrow2of3|policy_opening|tx_signing_hash" \
  privai-proof \
  privai-chain \
  spec
```

## Core questions

- What does the proof actually prove today?
- What are the exact current public inputs?
- Does mandatory auth for `FullPrivacy v1` change proof semantics, or does it remain ledger-side only?
- Which checks still remain ledger-side?
- Is `TransferNoteTx` the only current proof-covered tx class?
- Can escrow spend reuse the current `TransferNoteTx` proof path without a new circuit?
- Are `ExecutionBundle` and `ProofCertificate` complete semantics or only current runtime meaning?
- Is each tx class proof-covered, ledger-only, or experimental?

## Guardrails

- Do not overclaim proof semantics beyond current code.
- Do not move `FullPrivacy` mandatory auth, threshold auth, or escrow action semantics into proof unless the docs explicitly changed.
- Do not silently generalize `TransferNoteTx` semantics to `MarketplaceBatchTx` or `OnChainLite`.
- Keep `current canonical today` separate from `unresolved today`.
- Keep escrow v1 honest as a mixed model: proof for note-level correctness, ledger for auth/policy/action/timeout checks.
- Treat tests and runtime behavior as stronger evidence than aspirational docs.

## Output

- Current implemented semantics
- Ledger-vs-proof split
- Impact of auth or escrow changes on proof scope
- Unresolved gaps
- Suggested doc updates
