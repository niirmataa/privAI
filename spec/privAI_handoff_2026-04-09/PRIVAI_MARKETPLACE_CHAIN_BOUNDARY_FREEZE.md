# privAI Marketplace / Chain Boundary Freeze

**Date:** 2026-04-11
**Status:** frozen direction
**Scope:** marketplace boundary, chain settlement boundary, escrow role, optional marketplace batch rail

---

## 1. Core Rule

The marketplace is **not** a blockchain object.

The blockchain is a generic, privacy-preserving settlement substrate.

This means:

- marketplace discovery is not a chain primitive,
- public marketplace discovery is not the privacy baseline,
- skill packs are not chain primitives,
- provider profiles are not chain primitives,
- contract negotiation is not a chain primitive,
- delivery content is not a chain primitive,
- reputation history is not a public chain primitive.

The marketplace may use the chain for escrow and settlement, but the chain must not encode marketplace business semantics.

Canonical wording:

```text
Marketplace = off-chain application/protocol layer.
Chain = blind settlement substrate.
Escrow = on-chain generic private contract primitive.
Discovery = private / encrypted / credential-gated by default.
```

---

## 2. Escrow Boundary

Escrow **must** remain on-chain.

However, escrow is not marketplace-specific. It is a generic private contract settlement primitive between contracting parties.

The correct boundary is:

```text
Marketplace is not blockchain.
Escrow is blockchain.
Escrow does not know that it came from marketplace.
```

The chain should enforce:

- escrow lock,
- private / committed amount movement,
- contract commitment binding,
- release,
- refund,
- recovery after timeout,
- nullifier / spend protection,
- proof / statement commitments,
- valid settlement authorization.

The chain should not enforce or store:

- marketplace category,
- skill name,
- public provider profile,
- prompt text,
- task conversation,
- delivery artifact contents,
- discovery route,
- review text,
- reputation score history as public plaintext.

---

## 3. Canonical Chain Policy

The canonical `privAI` chain path is **FullPrivacy-first**.

This is a product and protocol decision, not only an optimization preference.

Default settlement should assume:

- private / committed amounts,
- encrypted recipient data,
- committed policy / contract data,
- PQ-safe authorization,
- generic private escrow,
- no marketplace business metadata on-chain.

The system may be slower, heavier, or more complex because of this. That is the
accepted cost of long-horizon privacy and post-quantum security.

Visible or aggregate payment rails may exist only as explicit opt-in compromises.
They must not become the default path and must not redefine the privacy promise of
the system.

Canonical wording:

```text
privAI chain default = FullPrivacy.
privAI escrow default = FullPrivacy PQ private contract escrow.
Lower-privacy rails = explicit opt-in compromise, not baseline.
```

---

## 4. Discovery Privacy Policy

The canonical FullPrivacy marketplace path does **not** use public discovery as
the baseline.

Public directories, public provider profiles, public ranking pages, public service
history, and public marketplace search indexes create correlation surfaces. Even
if they appear harmless at launch, they may become linkable over time through
timing, payment, transport, reputation, leaked logs, or external datasets.

Default discovery should be:

- private,
- encrypted,
- transport-protected,
- scoped to the querying buyer / session / epoch where possible,
- backed by selective credential proofs instead of public identity history.

Provider reputation should be proven selectively:

- prove threshold membership without exposing full history,
- prove capability / score class without exposing the provider root identity,
- avoid stable public IDs as the default discovery handle,
- prefer scoped offering IDs and scoped provider IDs.

Canonical wording:

```text
Public discovery is not the privAI privacy baseline.
Private discovery + private credential proofs are the baseline.
Any public listing is a lower-privacy opt-in surface.
```

---

## 5. Transport / Mailbox Priority

Because marketplace semantics stay off-chain, transport and mailbox privacy become
core protocol priorities.

Current NXMS transport encrypts payloads, but envelope metadata can still be
visible to relays/mailboxes. This must be treated as a known privacy gap, not as
an acceptable final state.

Future strengthening should prioritize:

- scoped / one-time sender identifiers,
- scoped / one-time receiver identifiers,
- opaque context IDs,
- mailbox queues that do not reveal stable buyer/provider relationships,
- relay-visible metadata minimization,
- private discovery queries,
- private offering delivery,
- ZK / credential proofs for provider quality without public identity linkage.

The FullPrivacy marketplace depends on hardening transport and mailbox metadata.
The chain alone cannot provide marketplace privacy if off-chain discovery and
communication leak stable relationships.

---

## 6. Layer Model

The production marketplace path has three layers.

### 6.1 Marketplace Layer

`off-chain application/protocol layer`

Responsible for:

- private provider discovery,
- signed discoverable offerings,
- skill pack distribution,
- contract negotiation,
- buyer/provider communication,
- delivery transport,
- evidence packet exchange,
- reputation credential updates.

This layer may use NXMS / mailbox / Tor-gated transport. It must not assume that marketplace state is published to the ledger.

### 6.2 Contract Layer

`off-chain agreement, on-chain commitment`

Responsible for:

- accepted contract terms,
- accepted price,
- delivery expectations,
- verification rules,
- timeout rules,
- settlement policy,
- contract commitment.

The contract is negotiated and accepted off-chain. The chain only needs the commitment and settlement-relevant bindings.

### 6.3 Chain Settlement Layer

`on-chain generic private escrow`

Responsible for:

- locking value,
- enforcing release / refund / recovery,
- validating signatures / authorization,
- enforcing timeout gates,
- protecting against double spend,
- binding settlement to committed statements.

The chain should be unable to tell whether a generic escrow came from:

- AI marketplace work,
- direct peer contract,
- private service purchase,
- another application using the same escrow primitive.

---

## 7. FullPrivacy Marketplace Path

The canonical FullPrivacy marketplace path is:

1. Provider publishes a discoverable offering off-chain.
2. Buyer discovers the offering through a private / encrypted / credential-gated discovery path.
3. Buyer and provider negotiate / accept a contract over encrypted transport.
4. Buyer locks funds in a generic FullPrivacy escrow on-chain.
5. Provider delivers the artifact off-chain over encrypted transport.
6. Settlement occurs on-chain through generic release / refund / recovery.
7. Reputation or score updates accrue to the provider identity / credential layer, not to public marketplace history on-chain.

The chain sees generic settlement data. It does not see marketplace semantics.

---

## 8. MarketplaceBatchTx Boundary

`MarketplaceBatchTx` exists in the codebase as an aggregate payment / settlement rail.

It must not be treated as the canonical FullPrivacy marketplace path.

Frozen classification:

```text
MarketplaceBatchTx = optional low-cost aggregate rail.
MarketplaceBatchTx != FullPrivacy marketplace baseline.
MarketplaceBatchTx != proof that marketplace must leave marketplace traces on-chain.
```

It is not the default `privAI` chain policy.

Use cases where `MarketplaceBatchTx` may be acceptable:

- cheap frequent payments,
- batched receipts,
- aggregate settlement windows,
- lower-cost merchant/operator accounting,
- flows that explicitly accept aggregate metadata leakage.

It may expose or imply:

- merchant/operator commitments,
- receipt roots,
- receipt counts,
- nullifier counts,
- total gross / fee / refund amounts,
- settlement windows,
- ticket nullifiers.

Therefore it is a privacy/cost compromise, not the FullPrivacy product definition.

---

## 9. Required Vocabulary

Use these names consistently:

- **FullPrivacy Marketplace Path** — off-chain marketplace protocol using generic on-chain private escrow.
- **Private Discovery** — encrypted / scoped / credential-gated discovery, the canonical privacy baseline.
- **Generic FullPrivacy Escrow** — on-chain private contract escrow primitive.
- **Private Contract Settlement** — release / refund / recovery against a committed contract, without marketplace semantics.
- **Batch Marketplace Rail** — explicit opt-in aggregate settlement path with weaker privacy and lower cost.
- **RecipientPrivacyLite** — visible amount / private recipient lightweight transfer rail.
- **Public Discovery** — lower-privacy opt-in surface, not baseline.

Do not use `MarketplaceBatchTx` as shorthand for marketplace.

---

## 10. Visibility Rules

### 10.1 Marketplace Layer

Marketplace-specific data should remain off-chain:

- skill pack,
- offer details,
- task text,
- artifact content,
- delivery conversation,
- semantic review notes,
- reputation details.

Even off-chain, these data should not default to public discovery or stable public
profiles.

### 10.2 Chain Layer

The chain may see only generic settlement material:

- commitments,
- nullifiers,
- encrypted recipient data,
- encrypted / committed amounts,
- fees,
- timeouts,
- statement commitments,
- proof envelopes / proof certificates,
- settlement authorization.

### 10.3 Transport Layer

Current NXMS transport encrypts payloads, but envelope metadata may still be visible to relays/mailboxes.

Therefore FullPrivacy marketplace must not rely on current transport envelope fields being private.

Future strengthening must address:

- scoped / one-time sender identifiers,
- scoped / one-time receiver identifiers,
- opaque context IDs,
- metadata-minimized routing,
- private provider identity proofs.

---

## 11. Do Not Infer

This document does **not** claim:

- that FullPrivacy cryptographic proof enforcement is complete today,
- that current NXMS transport is metadata-free,
- that `MarketplaceBatchTx` should be removed immediately,
- that all marketplace payments must use batch settlement,
- that lower-privacy rails are acceptable defaults,
- that public marketplace discovery is acceptable as the default,
- that delivery artifacts must be public or on-chain,
- that skill packs must be written into ledger state,
- that provider reputation must be public on-chain history.

This document does claim:

- marketplace semantics belong off-chain,
- private discovery is the canonical marketplace discovery baseline,
- transport / mailbox metadata privacy is a core FullPrivacy priority,
- escrow belongs on-chain,
- escrow must remain generic,
- FullPrivacy is the canonical chain default,
- FullPrivacy marketplace must not be defined by `MarketplaceBatchTx`,
- any future task must state which rail it targets.

---

## 12. Open Follow-Up Decisions

The following remain intentionally open:

- exact `DiscoverableOffering` format,
- exact provider identity / credential format,
- scoped identity rotation rules,
- whether delivery commitment is mandatory on-chain or optional per contract,
- exact `contract_commit` preimage format,
- whether current `MarketplaceBatchTx` naming should be retained, deprecated, or renamed,
- how reputation updates bind to hidden provider credentials,
- how transport envelope metadata is minimized for FullPrivacy.

---

## 13. Implementation Guardrail

Before implementing any marketplace task, the task prompt must explicitly choose one target:

1. **FullPrivacy Marketplace Path**
2. **Batch Marketplace Rail**
3. **RecipientPrivacyLite**
4. **Generic FullPrivacy Escrow**

If the task does not name the target rail, it is underspecified.

No agent should add new CLI commands, transaction types, ledger semantics, or marketplace-visible chain fields merely because `MarketplaceBatchTx` exists.

No agent should implement public marketplace discovery, public provider profiles,
or public reputation history unless the task explicitly marks it as a lower-privacy
opt-in feature.

---

*This freeze captures the 2026-04-11 architecture decision that marketplace is an off-chain protocol layer, while escrow is an on-chain generic private contract primitive. It complements `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`, `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md`, and `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md`.*
