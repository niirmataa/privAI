# privAI System Product Foundation

This is the primary first-read document for a new agent or operator.

Read this before:
- `PRIVAI_PROJECT_ENTRYPOINT.md`
- `PRIVAI_V1_READINESS_AND_GAPS.md`
- `PRIVAI_V1_PRODUCTION_PATH.md`
- `PRIVAI_NEXT_DIRECTION.md`
- `PRIVAI_DOCS_INDEX.md`

This document explains:
- what `privAI` is as a system product,
- what its real foundations are,
- what already exists today,
- what the final direction is,
- and what still belongs to the path toward production-shape v1.

It is intentionally high-level.
It should give the correct mental model first, and then point deeper into the docs.

## 1. What `privAI` Is

`privAI` is a privacy-first system for value, settlement, and controlled execution around local AI model providers in the `privAI` marketplace.

The product is not just:
- an escrow module,
- a proof experiment,
- a wallet,
- or a blockchain repo.

It is a wider system product built around:
- a blockchain with a coin,
- a dominant privacy-first rail,
- private transfers,
- lite payments,
- marketplace settlement,
- escrow workflows,
- zero-knowledge proof plumbing,
- wallet-owned final assembly,
- node/ledger enforcement,
- NXMS control-plane transport,
- validator P2P transport,
- and a user/operator entry layer through `nexum-cli`.

The main product idea is:
- privacy first,
- safe settlement,
- private value movement,
- and controlled workflow execution for a marketplace of local AI models.

## 2. Product Foundations

The system should be understood through its main foundations.

### A. Private coin and privacy-first chain

The chain is not an auxiliary add-on.
It is one of the foundations of the system.

The product includes:
- a blockchain,
- a coin,
- and a dominant privacy-first rail for higher-value and sensitive flows.

This privacy-first rail is `FullPrivacy`.
It is the core safety rail of the system.

### B. Private transfers

The system is designed around private value movement, not only public accounting.

Already central to the system:
- hidden amounts,
- note-based transfers,
- proof-backed note semantics,
- wallet-side final assembly.

Final direction:
- privacy-preserving transfers remain a core property of the system,
- not a secondary feature.

### C. Lite payments

The product also includes lightweight payments as a separate system concern.

Current system direction:
- lite payments exist as a distinct path,
- but the `OnChainLite` rail is still experimental and must not be treated as frozen final truth.

Final product direction:
- the chain should support a mode with hidden sender and publicly visible amount on-chain up to a bounded threshold,
- but the exact final production semantics of this rail are still not frozen.

So:
- lite payments are part of the product foundation,
- but they are not yet a fully frozen final rail.

### D. Marketplace settlement

`privAI` is fundamentally a marketplace-oriented system for local AI model providers.

Marketplace is not a side story.
It is the original product direction.

The system includes:
- marketplace-side settlement flows,
- operator-trusted small marketplace payments inside the marketplace system,
- and escalation to stronger privacy-preserving rails where required.

Important distinction:
- `MarketplaceSmallPaymentsRail` is a marketplace-specific settlement rail,
- it is not the same thing as `FullPrivacy`,
- it is not the same thing as `OnChainLite`,
- and it is not the rail for escrow.

Small marketplace payments are marketplace-internal by design.

### E. Escrow

Escrow is a necessary native primitive of the system, but it is not the star of the whole product.

Escrow matters because it enables:
- safe staged settlement,
- buyer/merchant/operator workflows,
- controlled release and refund logic,
- marketplace trust minimization.

But escrow should always be understood as part of the broader product:
- private coin,
- private transfers,
- marketplace settlement,
- and privacy-preserving execution.

Important frozen rule:
- escrow `2-of-3` belongs to `FullPrivacy`,
- not to `MarketplaceSmallPaymentsRail`,
- and not to `OnChainLite`.

### F. Proof-backed validity

Proofs are a foundation of the system, not a decorative extra.

The system already has:
- real proving data structures,
- real typed handoff,
- real artifact flow,
- real import-ready proof path.

But current escrow v1 is still explicitly mixed:
- note-level semantics are proof-covered,
- auth/policy/action/threshold/timeout semantics remain ledger/auth-enforced.

That is the honest v1 state.

Final direction:
- move toward a more proof-aware and privacy-complete execution story,
- without overclaiming that this is already finished today.

### G. Wallet-owned final assembly

Wallet final assembly is one of the core structural decisions of the system.

The wallet owns:
- final tx construction,
- final auth insertion,
- final `tx_signing_hash`,
- proof-ready handoff construction.

This is one of the reasons Stage A and Stage B must stay separate.

### H. Node and ledger enforcement

Node and ledger are not passive containers.
They are the final enforcement layer of the system.

They already enforce:
- staging and quorum-related runtime rules,
- submit gate checks,
- policy reconstruction from `policy_opening`,
- signer and action validation,
- block propose/import flow.

This enforcement role is part of the system foundation.

### I. Transport split

The system has two distinct transport families:

#### Validator transport
- direct P2P,
- session-based,
- encrypted,
- synchronous,
- used for validator networking and gossip.

#### NXMS control-plane transport
- mailbox / store-and-forward,
- asynchronous,
- marketplace / escrow / workflow oriented,
- separate from validator consensus transport.

This split is one of the hardest architectural rules of the system.

### J. User and operator entry layer

`nexum-cli` is part of the intended user-facing system layer.

It is expected to serve as:
- CLI entrypoint,
- Falcon-related user flow surface,
- vault/key flow surface,
- and potentially an integration point for marketplace login / registration / challenge flows.

There is also still a manual operator flow model around escrow orchestration, but the long-term direction is to reduce or remove manual operator steering where it is only transitional workflow support.

## 3. System Roles

The system should be understood not only as a stack of modules, but also as a set of distinct roles with different trust models, powers, and responsibilities.

### A. End user

This is the widest product role.

The end user:
- uses the system as a holder and mover of private value,
- interacts through `nexum-cli` and later also through marketplace-facing surfaces,
- manages keys, vault state, and signing rights,
- may appear in more specific sub-roles such as buyer, merchant, or marketplace participant.

This role is not one protocol primitive.
It is the human using the system.

### B. Buyer

Buyer is one of the core escrow and marketplace-side financial roles.

Buyer:
- funds the escrow note,
- is the source of value in the escrow flow,
- co-authorizes `release` together with the escrow operator in normal mode,
- co-authorizes `recovery_release` together with the merchant after timeout,
- is one of the identities committed inside the `Escrow2of3` policy,
- uses the wallet to participate in final Stage B assembly and signing.

In short:
- buyer brings funds into the flow,
- participates in normal release,
- and regains agency in the recovery path.

### C. Merchant

Merchant is the service-side or seller-side participant, including the local AI model provider inside the marketplace product.

Merchant:
- is the destination of value in `release`,
- co-authorizes `refund` together with the escrow operator in normal mode,
- co-authorizes `recovery_release` together with the buyer after timeout,
- participates in marketplace settlement logic,
- creates or co-creates `Receipt` in marketplace small-payments flow.

In short:
- merchant is the service provider,
- a party in escrow policy,
- and a party in marketplace settlement.

### D. Marketplace operator

Marketplace operator is the system authority for the marketplace rail.

This role belongs to the marketplace product layer, not to the privacy core and not to validator consensus.

Marketplace operator is responsible for:
- issuing `SpendGrant`,
- enforcing `ServicePaymentPolicy`,
- intake and validation of `Receipt`,
- batching settlement,
- publishing `MarketplaceBatchTx`,
- governing scoped small marketplace payments,
- connecting login/challenge/auth flows to marketplace access and usage rights.

In the current/final marketplace rail model:
- operator is the default authority for grants and settlement,
- merchant is not the standalone settlement authority,
- operator sees more than chain,
- and the rail is explicitly operator-trusted.

Marketplace operator is therefore:
- not the same as escrow operator,
- not the ledger,
- not the validator set,
- and not just an irrelevant API server.

The most honest current interpretation is:
- marketplace operator is the backend/system service of the `privAI` marketplace itself.

### E. Escrow operator

Escrow operator is a system workflow machine.

This role must be understood as:
- deterministic,
- tightly constrained,
- automation-first,
- not a trust anchor,
- not a free-form moderator,
- not a human-in-the-loop final authority.

Escrow operator:
- is the required co-signer in normal-mode escrow,
- may attach the required normal-mode signature,
- may refuse,
- reacts to valid workflow state,
- should operate on minimal operational context only.

Escrow operator cannot:
- spend funds unilaterally,
- change policy,
- change destination constraints,
- override ledger rules,
- become the permanent lock point of the system.

This is why recovery exists:
- `Buyer + Merchant` can escape operator lock after timeout.

The intended final interpretation is:
- escrow operator is a highly restricted system machine running "like a Swiss watch", not a soft business role.

### F. Wallet

Wallet is one of the most security-critical roles in the entire system.

It is not just a user interface and not just a signer helper.

Wallet owns:
- final tx construction,
- final auth insertion,
- final `tx_signing_hash`,
- proof-ready handoff construction,
- binding of private local state to final execution artifacts.

Wallet is responsible for:
- building final `TransferNoteTx`,
- handling `policy_opening`,
- binding action and signer set correctly,
- producing final signatures,
- building proof handoff,
- protecting private local state.

Security expectations for wallet are high.

Threats include:
- key theft,
- wrong-message signing,
- replay or wrong-context signing,
- action/policy confusion,
- signer confusion,
- leakage of local private state,
- compromise of vault/seed/signing path,
- hostile integration around CLI or UI,
- leakage of receive or prekey material.

The wallet must therefore:
- sign only canonical messages,
- not trust control-plane as execution truth,
- keep strong separation between proposal input, final assembly, proof handoff, and final signature.

Wallet is a foundation of execution safety.

### G. Node

Node is the runtime boundary between:
- control-plane,
- wallet/proof path,
- and ledger execution.

Node is not the owner of Stage B assembly.
But it is the place where the final tx meets protocol enforcement.

Node is responsible for:
- Stage A staging,
- quorum runtime state,
- payload ingest,
- bundle export,
- escrow submit gate,
- block propose,
- block import,
- proof artifact handling,
- binding runtime execution to ledger semantics.

Security expectations for node are also high.

Threats include:
- accepting malformed final tx,
- accepting wrong `policy_opening`,
- accepting wrong signer set,
- accepting wrong action semantics,
- missing timeout enforcement where required,
- confusing control-plane and execution-plane inputs,
- replay or duplicate ingest,
- bad ack/no-ack behavior,
- restart/reload inconsistency,
- proof/block artifact mismatch,
- cross-contamination of validator transport and mailbox path.

Node must therefore:
- separate Stage A from Stage B,
- separate validator P2P from NXMS mailbox,
- treat `policy_opening` as source of truth,
- validate action/policy/signer consistency,
- avoid trusting control-plane or wallet blindly,
- preserve runtime state coherently.

Node is the execution firewall of the system.

### H. Ledger and validators

Ledger and validator layer are the final protocol-enforcement layer.

They:
- maintain system state,
- enforce transaction and block rules,
- verify imported blocks,
- enforce part of the escrow semantics that remain ledger-side,
- operate over validator session transport,
- remain distinct from NXMS control-plane.

Validators:
- use direct P2P session transport,
- do not use mailbox as consensus transport,
- are not marketplace operators,
- are not escrow workflow machines.

### I. NXMS control-plane

NXMS control-plane is not a human role, but it is a critical system role.

It carries:
- proposal messages,
- approvals,
- workflow payloads,
- escrow and marketplace control-plane communication,
- future workflow/inference-related messages.

It is:
- orchestration,
- not execution truth,
- not validator consensus transport.

### J. `nexum-cli`

`nexum-cli` is a core user-facing entry layer of the system.

It is already or is intended to cover:
- registration,
- login,
- challenge flows,
- PoW anti-abuse entry,
- key management,
- vault semantics,
- Falcon signing,
- FrodoKEM-based receive and transport support,
- prekeys / `ReceiveBundle`,
- payments,
- DM or secure communication support,
- later also direct marketplace flows,
- operation without a browser, over CLI and Tor only.

This is important:
- the system should be operable without a browser,
- directly through CLI,
- with local key control and local cryptography.

`nexum-cli` should therefore be understood as:
- not a helper tool,
- but a full product entrypoint.

### K. Manual operator flow

There may still be a manual operator flow model around some escrow/workflow handling.

But this should be treated as transitional support,
- not as a core final product role.

The final direction is toward stricter system automation, not manual workflow steering.

## 4. Current Honest System State

The system already has real foundations implemented today.

### Already real today

- `FullPrivacy` as the dominant privacy-first rail
- Stage A / Stage B separation
- control-plane vs execution-plane separation
- wallet final assembly
- typed proof handoff
- node escrow submit gate
- node Stage A runtime
- validator transport hardening
- honest local escrow `release` e2e
- marketplace-specific rail as a separate concept

This means `privAI` is already a real multi-layer system.

### Not fully closed yet

- mailbox runtime loop
- honest escrow `refund` e2e
- honest escrow `recovery_release` e2e
- timeout enforcement for recovery path
- stronger runtime retry/ack semantics
- fuller observability and resilience
- final truthful product-freeze docs
- final frozen semantics for lite rail
- final frozen production story for the full proof runtime/orchestration layer

## 5. Current v1 Reality vs Final Product Direction

This distinction is critical.

### Current v1 reality

Today the most honest statement is:
- `privAI` already has a real privacy-first chain foundation,
- a real escrow `release` path,
- real wallet/proof/node integration,
- and a real validator transport layer,
- but v1 is still incomplete as a full production system.

### Final product direction

The final product direction is broader than current v1.

It includes:
- privacy-first coin and value system,
- stronger privacy-preserving transfers,
- lite payments for bounded cases,
- marketplace-native settlement flows,
- escrow as a core primitive,
- wallet-owned execution,
- proof-backed validity,
- operator-safe workflow execution,
- and a coherent CLI/user layer.

In other words:
- current v1 is a real foundation,
- final product is larger than current v1 scope.

## 6. What Must Not Be Misunderstood

A new agent must not assume:

- that `privAI` is only an escrow repo,
- that marketplace is secondary,
- that privacy is just a feature layered on later,
- that wallet/proof are side modules,
- that validator P2P and NXMS mailbox are variants of the same transport,
- that `release` automatically means `refund` and `recovery_release` are done,
- that typed proof handoff means the full prover runtime is finished,
- that current mixed escrow v1 already means fully proof-native escrow.

## 7. What Is Final, What Is v1, What Is Future

### Final product vision

Final product vision is:
- a privacy marketplace for local AI models,
- built on a privacy-first value and settlement system,
- with private coin behavior,
- private transfers,
- lite bounded-payment support,
- marketplace settlement,
- escrow,
- proof-backed execution,
- and user/operator entry through CLI and related interfaces.

### Current production-shape v1 target

Current production-shape v1 target is narrower:
- close mailbox runtime,
- close escrow `refund`,
- close escrow `recovery_release`,
- enforce recovery timeout,
- harden runtime behavior,
- freeze Stage A / Stage B contract,
- write truthful final v1 docs.

### Future beyond current v1

Beyond current v1:
- fuller proof-aware threshold auth,
- fuller proof-native escrow semantics,
- final lite rail semantics,
- broader marketplace/runtime expansion,
- and whatever becomes v2/v3/v5 will be decided later with the models and actual implementation evidence.

## 8. Reading Path After This File

After reading this document, continue with:

1. `PRIVAI_PROJECT_ENTRYPOINT.md`
2. `PRIVAI_V1_READINESS_AND_GAPS.md`
3. `PRIVAI_V1_PRODUCTION_PATH.md`
4. `PRIVAI_NEXT_DIRECTION.md`
5. `PRIVAI_DOCS_INDEX.md`

Then go deeper into:
- escrow specs,
- proof boundaries,
- transport/runtime freeze docs,
- and only then into code.

## 9. One-Line Definition

`privAI` is a privacy-first marketplace and settlement system for local AI models, built around a private coin foundation, private transfers, escrow workflows, proof-backed validity, wallet-owned final assembly, and strongly separated runtime/control-plane layers.
