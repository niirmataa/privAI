# AGENT.md - privAI Project Rules

This file describes how an agent should behave when entering `privAI`.

## 1. First Read Order

Before touching code, read:
1. `PRIVAI_PROJECT_ENTRYPOINT.md`
2. `PRIVAI_V1_READINESS_AND_GAPS.md`
3. `PRIVAI_NEXT_DIRECTION.md`
4. `PRIVAI_DOCS_INDEX.md`
5. then open deep specs and code

Do not start from random repo files without understanding the system map first.

## 2. Project Rules

### Architectural rules
- Keep Stage A and Stage B separate.
- Do not let control-plane pretend to own final execution semantics.
- Do not mix validator P2P with NXMS mailbox control-plane.
- Treat `policy_opening` as authoritative, not `policy_tag`.
- Do not overclaim proof/runtime completeness.

### Safety rules
- Do not deploy or migrate without human review.
- Do not modify `crypto/*`, `contracts/*`, `keys/*`, or `withdrawals/*` without explicit audit intent.
- Do not introduce dependencies from unverified sources.
- Do not paste secrets or private data.

### Workflow rules
- Prefer small, sharply bounded tasks.
- Make success conditions explicit.
- Preserve honesty: say what is real, what is inferred, and what is still open.
- Keep environment problems separate from application logic problems.

## 3. Current Project Truth

Already real:
- Stage A / Stage B boundary cleanup
- wallet final assembly
- typed proof handoff
- node escrow submit gate
- validator session regression pack
- local honest escrow `release` e2e

Still open:
- mailbox runtime loop
- refund / recovery escrow e2e
- runtime hardening and truthful product freeze

## 4. Default Reading Targets By Topic

If task is about escrow semantics:
- read object model, tx matrix, proof integration docs

If task is about node submit/runtime:
- read `privai-node/src/node.rs`
- read `privai-node/src/escrow_stage.rs`
- read `privai-node/tests/escrow_submit_gate.rs`

If task is about Stage B assembly:
- read `privai-wallet/src/escrow_builder.rs`
- read `privai-wallet/src/proof_handoff.rs`

If task is about transport:
- read transport freeze memo
- read transport split doc
- read validator session invariants

## 5. What A Good Task Looks Like

A good task for this project should clearly define:
- role,
- problem,
- write scope,
- read scope,
- required changes,
- forbidden shortcuts,
- Definition of Done,
- minimal commands,
- exact final report.

Do not accept vague "fix it somehow" prompts when the project boundary matters.

## 6. Current Default Next Step

If there is no newer instruction, the safest next meaningful project step is:
- finish and verify the mailbox runtime loop in `privai-node`

After that:
- honest escrow `refund` e2e
- honest escrow `recovery_release` e2e
- recovery timeout enforcement
- Stage A / Stage B freeze hardening
