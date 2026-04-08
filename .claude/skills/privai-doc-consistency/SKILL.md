---
name: privai-doc-consistency
description: Check whether privAI docs, trackers, and code still agree. Use when validating completed work, auditing tracker claims, checking source-of-truth drift, or asking whether something is really done.
argument-hint: [scope]
---

# privAI Doc Consistency

Use this skill when checking whether docs, trackers, and code still agree.

Treat `$ARGUMENTS` as the narrowed review scope.

## Read first

1. `spec/PRIVAI_EXECUTION_SPINE.md`
2. `spec/PRIVAI_SPEC_INDEX.md`
3. The active phase task doc
4. The active phase fix tracker
5. The relevant phase docs

Typical current active docs include:

- `spec/nxms_transport_p2p/TRANSPORT_P2P_MASTER_TASKS.md`
- `spec/nxms_transport_p2p/TRANSPORT_P2P_FIX_TRACKER.md`
- `spec/PRIVAI_PROOF_COMPLETION_PLAN.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`

## Workflow

1. Identify the likely source-of-truth doc for the area under review.
2. Classify each document as active source, frozen direction, support doc, or memory-only note.
3. Inspect the code and tests that correspond to the claim.
4. Separate three states: implemented, documented, and test-verified.

Useful repo search:

```bash
rg -n "done|fixed|implemented|experimental|current canonical|unresolved|tracker" \
  spec \
  privai-node \
  privai-proof \
  privai-ledger \
  privai-chain
```

## Core questions

- Does the tracker claim more than the code actually does?
- Is the doc describing current canonical behavior or a future target?
- Are unresolved gaps still named honestly?
- Is the document active source, frozen direction, or memory-only?
- Are there dead references to missing docs or missing code?

## Guardrails

- Prefer underclaiming over overclaiming.
- Never mark something done unless the code path is confirmed.
- Keep implemented, documented, and test-proven separate.
- If a doc is aspirational, say so plainly.

## Output

- Findings first
- Then status: `accurate`, `partially accurate`, or `overclaiming`
- Then exact docs to update
