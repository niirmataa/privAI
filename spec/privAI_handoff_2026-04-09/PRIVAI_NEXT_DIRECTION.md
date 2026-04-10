# privAI Next Direction

This document answers one question:

What should we do next, in what order, and what should we avoid touching by accident?

## 1. Immediate Priority Order

### Priority 1
- finish and verify mailbox runtime loop in `privai-node`

### Priority 2
- add honest escrow `refund` e2e

### Priority 3
- add honest escrow `recovery_release` e2e

### Priority 4
- enforce and verify recovery timeout behavior

### Priority 5
- freeze post-handoff Stage A / Stage B contract more tightly

## 2. Why This Order

### Mailbox first
Because it is the biggest open runtime gap and directly follows the transport/runtime freeze memo.

### Refund / recovery next
Because `release` is already real, so the next clean expansion is coverage of the other escrow action families.

### Contract freeze after that
Because it is best done after the real paths exist, not before.

## 3. Do Next

### Track A - runtime
- mailbox pull/ingest/ack loop
- retry/no-ack semantics
- duplicate delivery behavior
- delayed-ack behavior

### Track B - escrow coverage
- `refund` e2e
- `recovery_release` e2e
- timeout path verification
- output target validation per action

### Track C - hardening
- `EscrowApprovalBundle` post-handoff review
- sentinel `TX_SIGNING_HASH_STAGE_A` review
- error taxonomy
- golden vectors
- restart/reload tests

## 4. Do Not Do Right Now

- do not redesign escrow from scratch
- do not mix validator P2P with NXMS control-plane
- do not touch `crypto/*`, `contracts/*`, `keys/*`, `withdrawals/*` without audit
- do not overclaim full prover runtime if it is still partially staged
- do not treat WSL instability as automatic proof that the app code is wrong

## 5. Suggested Agent Split

### Xiaomi
- mailbox runtime loop
- runtime retry/ack behavior
- recovery and reload tests
- bounded node/runtime hardening tasks

### Gemini
- `refund` e2e
- `recovery_release` e2e
- cross-layer escrow consistency tasks

### Claude
- truth docs
- product freeze docs
- readiness / gap memos
- drift cleanup memos

### Ampere / strongest model
- risky glue-layer work
- difficult boundary review
- cross-module consistency surgery

## 6. Exit Condition For This Phase

This phase is truly done when:
- mailbox runtime path is verified,
- `release`, `refund`, and `recovery_release` all have honest e2e,
- Stage A / Stage B contract is frozen cleanly,
- docs truthfully describe what is production-shape and what is still scaffolding.
