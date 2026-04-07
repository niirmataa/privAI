# privAI Proof Boundaries

Status: anti-inference proof status doc in migration.
Canonicality: binding cross-cutting interpretation doc for proof status, proof coverage boundaries and forbidden inference. This document does not override `spec/PRIVAI_CONSENSUS.md` or `spec/PRIVAI_CANONICAL_FORMATS.md`; it freezes what may and may not be assumed while proof semantics are still incomplete.
Owner: privAI consensus and proof governance.
Depends on:
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zablokowac dopowiadanie proof semantics tam, gdzie nie ma jeszcze jednej finalnej odpowiedzi,
- rozdzielic current canonical bytes od finalnej semantyki proof plane,
- zapisac, ktore tx classes sa juz proof-defined, a ktore nadal nie,
- zablokowac pol-finalne interpretacje `OnChainLite`.

## 2. Frozen rules

- sam fakt, ze obiekt ma `statement_commit`, nie oznacza jeszcze finalnie zamrozonego proof modelu,
- current canonical bytes `ExecutionBundle` i `ProofCertificate` moga byc frozen bez twierdzenia, ze finalna semantyka wszystkich raili jest juz kompletna,
- `TransferNoteTx` pozostaje glowna proof-covered tx class systemu,
- `MarketplaceBatchTx` nie dziedziczy automatycznie proof semantics `FullPrivacy`,
- `OnChainLite` nie moze byc uznany za finalny bez jawnie zamrozonego proof coverage modelu i pelnego spiecia z consensus/state commitments.

## 3. Proof status by tx class

| Tx class | Rail | Current canonical status | Final proof status | Forbidden inference |
|----------|------|--------------------------|--------------------|---------------------|
| `TransferNoteTx` | `FullPrivacy` | current canonical and proof-bearing | expected final proof-covered rail | Nie wolno oslabic tej klasy do "lite path". |
| `MarketplaceBatchTx` | `MarketplaceSmallPaymentsRail` | current canonical tx bytes and settlement auth/nullifier checks | final marketplace rail remains separate from `FullPrivacy` proof semantics | Nie wolno zakladac, ze batch ma ten sam proof model co `FullPrivacy`. |
| `OnChainLite` tx | `OnChainLite` | experimental only | unresolved | Nie wolno zakladac ani `proof-covered`, ani `proof-free` jako frozen truth. |
| `SettlementTx` / `ModelTx` / `StakeTx` | provisional | bytes may exist in code | unresolved | Nie wolno z samego enum wnioskowac o finalnej proof roli. |

## 4. Current canonical bytes vs current canonical semantics

### 4.1. ExecutionBundle

Current canonical facts:
- `ExecutionBundle` ma current canonical bytes,
- ma reference vector w `spec/PRIVAI_REFERENCE_VECTORS.md`,
- jest czescia block body i state/statement commitments.

Not yet frozen:
- jeden finalny meaning coverage dla wszystkich raili,
- ostateczna semantyka, jak `OnChainLite` ma byc odzwierciedlany w coverage.

### 4.2. ProofCertificate

Current canonical facts:
- `ProofCertificate` ma current canonical bytes,
- ma reference vector i `proof_cert_hash`,
- `proof_cert_root` jest current canonical block commitment.

Not yet frozen:
- pelna finalna rola `ProofCertificate` dla wszystkich raili i wszystkich execution modes,
- finalny end-to-end proof story dla lite raila.

## 5. Current non-conformities that matter for proofs

- `note_root()` obejmuje obecnie tylko `tx.outputs()`, a nie lite outputs,
- lite path nie jest end-to-end spiety z canonical state commitments,
- `build_execution_bundle_from_transactions()` traktuje `LiteTransfer` jako proof-requiring, ale obecny support dla public inputs nie jest finalnie domkniety,
- current proof coverage logic odzwierciedla glownie `TransferNoteTx`, a nie finalny stan wszystkich raili.

## 6. Closure conditions for final proof freeze

Proof status danego raila mozna nazwac `final` dopiero wtedy, gdy:
- tx class ma jawnie zamrozony status proof coverage,
- consensus doc opisuje jej relacje do `ExecutionBundle`,
- canonical formats doc opisuje jej bytes i commitments bez unresolved gaps,
- reference vectors maja bit-to-bit example,
- rail jest spiety z `note_root`, `state_root` i odpowiednimi roots/proof checks,
- nie ma juz current non-conformity, ktora podwaza ten model.

## 7. Forbidden inference

Niedozwolone jest:
- uznawanie `OnChainLite` za finalny rail proof-bearing,
- uznawanie `OnChainLite` za finalny rail proof-free,
- wyprowadzanie proof semantics z samego istnienia `statement_commit`,
- traktowanie current bytes `ExecutionBundle` lub `ProofCertificate` jako dowodu, ze semantyka wszystkich raili jest juz zamrozona,
- nadpisywanie current verifier behavior batcha marketplace przez future target proof/story.

## 8. How to use this doc

Jesli task dotyczy proof plane:
- najpierw przeczytaj ten dokument,
- potem `spec/PRIVAI_CONSENSUS.md`,
- potem `spec/PRIVAI_REFERENCE_VECTORS.md`,
- i dopiero wtedy zejdz do kodu.

Jesli task dotyczy `OnChainLite`:
- domyslny status to `experimental`,
- dopoki nie zamkniesz closure conditions, nie nazywaj go finalnym i nie dopisuj lokalnej architektury proof.
