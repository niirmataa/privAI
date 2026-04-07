# privAI Gap Register

Status: anti-drift governance doc for unresolved items and known migration gaps.
Canonicality: binding gap ledger for what is still open, what is merely current behavior, and what is forbidden to infer. This document does not override canonical specs; it narrows interpretation where the canonical set is intentionally incomplete.
Owner: privAI spec governance.
Depends on:
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby kazda istotna luka miala:
- jeden identyfikator,
- current state,
- final target,
- blokujacy status,
- zabronione dopowiedzenia,
- nastepny ruch zamykajacy temat.

Zamrozona zasada:
- jesli cos jest wpisane jako gap, nie wolno zachowywac sie tak, jakby "na pewno bylo juz rozstrzygniete",
- jesli `Forbidden inference` czegos zabrania, nie wolno implementowac lokalnej interpretacji.

## 2. Gap register

| ID | Area | Current state | Final target | Blocking? | Forbidden inference | Next action |
|----|------|---------------|--------------|-----------|---------------------|-------------|
| `GAP-001` | `ServicePaymentPolicy` | Current canonical policy is the narrow format from `small_payments.rs`. | Extended `tag + params` policy binding reservation / acceptance / refund / dispute / timeout / batching. | `yes` for full marketplace freeze; `no` for current-compatible freeze | Nie wolno udawac, ze rozszerzone pola juz sa w `policy_commit`. | Prepare explicit format migration. |
| `GAP-002` | `MarketplaceBatchTx auth` | Ledger checks operator signature only if `auth` is present. | Operator auth mandatory for final marketplace settlement. | `yes` for full final consensus freeze | Nie wolno nazywac obecnego warunkowego checku finalnym stanem. | Tighten ledger validation. |
| `GAP-003` | `MarketplaceBatchTx signed payload` | Current verifier message is `settlement_root`. | Richer signed payload may later include `core + summary + ticket_nullifiers` under dedicated domain. | `no` for current-compatible freeze | Nie wolno opisywac richer batch signed payload jako dzisiejszego verifier rule. | Keep current rule frozen; decide migration later. |
| `GAP-004` | `operator_sig bytes layout` | `operator_sig` is encoded as `vec-of-one`, affecting `tx_id`. | Future migration to plain `bytes` is possible but not approved. | `no` | Nie wolno lokalnie upraszczac bytes layout bez jawnej migracji. | Preserve current quirk until explicit migration. |
| `GAP-005` | `receipt_root helper` | Spec rule and vectors exist; one public helper in crate is still missing. | One canonical helper for `receipt_root` may be added to code. | `no` | Nie wolno zmieniac formula `receipt_root` pod pretekstem helpera. | Optional helper implementation. |
| `GAP-006` | `ExecutionBundle / ProofCertificate semantics` | Current canonical bytes and vectors exist. Full multi-rail semantics remain incomplete. | Final semantics across all rails and proof modes. | `no` for bytes freeze; `yes` for complete proof semantics freeze | Nie wolno z current bytes wnioskowac, ze proof semantics wszystkich raili sa finalne. | Keep bytes frozen, semantics tracked in proof boundaries. |
| `GAP-007` | `OnChainLite proof model` | Rail is experimental and not fully tied into proofs and state commitments. | One frozen proof coverage model, tied into `note_root`, `state_root`, `ExecutionBundle`, threshold rules. | `yes` for final lite freeze | Nie wolno zakladac ani "proof-covered", ani "proof-free" jako frozen truth. | Resolve proof model first. |
| `GAP-008` | `OnChainLite state commitments` | `note_root()` currently excludes lite outputs via `tx.outputs()`. | Final lite rail must be fully reflected in canonical state commitments. | `yes` for final lite freeze | Nie wolno nazywac obecnego lite path finalnym rail state model. | Align tx/ledger/consensus path. |
| `GAP-009` | `OnChainLite threshold enforcement` | Product freeze says threshold is a system validation rule, but full implementation alignment is incomplete. | Clear consensus/ledger enforcement for lite threshold. | `yes` for final lite freeze | Nie wolno zgadywac enforcement policy from UX intent alone. | Add explicit enforcement or keep rail experimental. |
| `GAP-010` | `Amount layer` | Current witnesses still depend on `Amount14`-era constraints. | Final canonical integer amount encoding for `PVA + aPVA`. | `yes` for full unit freeze in code | Nie wolno traktowac current witness limitu jako finalnej granicy produktu. | Finalize amount migration path. |
| `GAP-011` | `Coin signing hash` | Current auth is verified against `tx_id`, while `tx_id` includes `auth`. | One stable signing hash that excludes auth self-reference. | `yes` for real escrow multisig | Nie wolno traktowac obecnego `tx_id` auth jako finalnego modelu podpisu. | Add `tx_signing_hash` / `input_auth_message`. |
| `GAP-012` | `Policy-aware threshold auth` | Ledger verifies signatures structurally, but does not reconstruct policy opening nor enforce `1 z 1` / `2 z 3` semantics. | Ledger reconstructs policy from opening and enforces threshold rules per policy kind. | `yes` for real escrow multisig | Nie wolno twierdzic, ze sam poprawny Falcon signature oznacza poprawny spend policy auth. | Add `policy_opening` and threshold validation. |
| `GAP-013` | `Escrow object model` | There is no canonical `EscrowFundingDescriptor`, `EscrowSnapshot`, `EscrowSpendProposal` or `EscrowApprovalBundle` for `privAI`. | One frozen off-chain control-plane object model for escrow orchestration. | `yes` for clean `nexum-core` adaptation | Nie wolno improwizowac lokalnych escrow payloads bez jednego frozen modelu. | Define canonical escrow objects and integration flow. |
| `GAP-014` | `Transport/P2P split` | `nxms-transport` mixes escrow wire, packet crypto and shared transport helpers, while `privai-node::net` contains a growing validator session layer. | One explicit split: shared primitives, escrow packet transport, validator session transport, consensus overlay. | `yes` for clean networking refactor | Nie wolno traktowac obecnego zmieszanego ukladu jako finalnej architektury warstw. | Freeze split, then extract session layer behind a narrow API. |

## 3. Blocking rule

Zamrozona zasada:
- `Blocking? = yes` oznacza, ze obszar nie moze zostac nazwany `final` tylko dlatego, ze istnieje w kodzie albo ma draft docs,
- `Blocking? = no` nie oznacza, ze problem jest nieistotny; oznacza tylko, ze nie blokuje current-compatible freeze.

## 4. Forbidden inference rule

Kazdy wpis ma pole `Forbidden inference`.

Interpretacja:
- jesli dana inferencja jest zabroniona, agent albo dev nie moze:
  - dopisac helpera "zgodnego z intuicja",
  - zaprojektowac bytes na podstawie targetu,
  - uznac eksperymentalnego raila za finalny,
  - odczytac niejawnej polityki z samego istnienia pola w kodzie.

## 5. Closure rule

Gap mozna zamknac tylko wtedy, gdy:
- canonical doc ma juz finalna formule albo status,
- kod i vectors sa zgodne z finalna regula, jesli gap dotyczy current canonical behavior,
- albo gap zostal przeksztalcony w jawny `Future target requiring migration` z zatwierdzona sciezka migracji.
