---
name: privai-escrow-design
description: Draft or review the final privAI escrow model around note-based value, `Escrow2of3`, operator automation, mandatory `FullPrivacy` auth, recovery paths, and proof integration. Use when tasks touch escrow roles, `nexum-core`, recovery semantics, policy-constrained auth, or marketplace trust assumptions.
argument-hint: [scope]
---

# privAI Escrow Design

Use this skill when drafting or reviewing the final escrow model, especially around note-based value, threshold auth, operator gating, recovery mode, mandatory auth in `FullPrivacy`, and proof integration.

Treat `$ARGUMENTS` as the narrowed escrow scope.

## Read first

1. `spec/PRIVAI_EXECUTION_SPINE.md`
2. `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
3. `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
4. `spec/PRIVAI_PROTOCOL_CORE.md`
5. `spec/PRIVAI_CANONICAL_FORMATS.md`
6. `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
7. `spec/PRIVAI_ESCROW_TX_MATRIX.md`
8. `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
9. `spec/PRIVAI_ESCROW_OBJECT_MODEL.md`
10. `spec/PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`
11. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`

## Primary code

- `privai-chain/src/tx.rs`
- `privai-chain/src/note.rs`
- `privai-ledger/src/ledger.rs`
- `privai-ledger/src/escrow.rs`
- `privai-nxms/src/lib.rs`
- `privai-proof/src/transfer.rs`

If a local `nexum-core` checkout exists, inspect the control-plane objects and approval flow there as well.

Useful repo search:

```bash
rg -n "escrow|Escrow2of3|2-of-3|buyer|merchant|operator|recovery|policy_opening|policy_tag|tx_signing_hash|nexum-core" \
  spec \
  privai-chain \
  privai-ledger \
  privai-nxms
```

## Core model assumptions

- Escrow is note-based, not operator-balance-based.
- Escrow is `FullPrivacy`, not marketplace convenience rail.
- `Escrow2of3` is policy-constrained, not unrestricted any-2-of-3.
- Roles are buyer, merchant, and operator.
- Normal mode requires operator participation.
- Recovery mode allows buyer and merchant only under explicit recovery conditions.
- `FullPrivacy v1` requires auth for every input and `policy_opening` is mandatory.
- `policy_opening` is the source of truth for policy type; `policy_tag` is only a hint and must match the derived type.
- Signatures authorize canonical actions, not arbitrary fund movement.
- Operator is a deterministic workflow machine, not a trust anchor.
- `nexum-core` is control-plane/orchestration, not Monero execution engine.
- Escrow v1 stays mixed: proof covers note-level correctness, ledger covers auth/policy/action/timeout semantics.

## Core questions

- What exactly is the escrow note policy?
- Which action types are legal?
- How does `Escrow2of3` map Buyer, Merchant, and Operator into a frozen action table?
- What is public versus private in the escrow auth path?
- What must ledger check outside the proof?
- What does `nexum-core` generate and what does ledger verify?
- What does the operator see via mailbox / `nxms-transport`, and what must stay opaque?
- What is normal mode versus recovery mode?
- What can operator do, and what is operator forbidden from doing?

## Guardrails

- Do not describe final escrow as operator-only.
- Do not describe escrow as unrestricted any-2-of-3.
- Do not merge the marketplace convenience rail into final escrow semantics.
- Do not propose public escrow tagging or a public `auth_required` classification as the default answer if `FullPrivacy v1` already settled on mandatory auth for all inputs.
- Do not claim full PQ privacy unless the proof layer actually supports that claim.
- Keep current v0 shortcuts separate from final escrow design.
- Do not treat mailbox events or operator-local state as stronger than on-chain note state.
- Do not describe `nexum-core` as the execution engine for coin spends; it is the workflow/control-plane layer.

## Output

- Roles
- Trust model
- Action model
- Auth model
- Ledger/proof split
- Operator / `nexum-core` flow
- Recovery semantics
- Open risks or unresolved boundaries
