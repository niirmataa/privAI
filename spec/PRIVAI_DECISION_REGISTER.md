# privAI Decision Register

Status: anti-drift governance doc for frozen and near-frozen architectural decisions.
Canonicality: binding decision log for reading intent and approved direction. This document does not override the canonical spec set; it records the final interpretation of decisions already reflected in canonical docs.
Owner: privAI spec governance.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

Ten dokument istnieje po to, zeby:
- zatrzymac powtarzanie tych samych decyzji architektonicznych,
- zapisac finalna interpretacje kierunku,
- rozrozniac decyzje zamrozone od targetow wymagajacych migracji,
- utrudnic dopowiadanie brakujacej semantyki "z pamieci".

Zamrozona zasada:
- jesli decyzji nie ma tutaj albo w canonical docs, nie wolno jej traktowac jako zatwierdzonej tylko dlatego, ze "tak brzmi sensownie",
- jesli wpis ma status `Future target requiring migration`, nie wolno udawac, ze jest juz current canonical behavior,
- jesli wpis ma status `Unresolved`, nie wolno z niego wyprowadzac architektury ani bytes.

## 2. Status vocabulary

- `Frozen`:
  decyzja jest zatwierdzona i obowiazuje jako kierunek systemu.
- `Current canonical`:
  decyzja opisuje aktualny normatywny stan bytes / verifier behavior / helper rule.
- `Future target requiring migration`:
  decyzja jest zatwierdzonym targetem, ale wymaga jawnej migracji formatu, kodu albo vectors.
- `Experimental`:
  rzecz istnieje, ale nie jest finalna i nie wolno jej sprzedawac jako zamrozonej.
- `Unresolved`:
  nie ma jeszcze jednej finalnej odpowiedzi i nie wolno jej dopowiadac.

## 3. Decision register

| ID | Status | Decision | Why it exists | Affects | Migration required |
|----|--------|----------|---------------|---------|--------------------|
| `DEC-001` | `Frozen` | Canonical source of truth lives under `spec/` and starts from `PRIVAI_SPEC_INDEX.md`. | Zatrzymuje mieszanie starych docs, kodu i handoffow jako rownorzednych source of truth. | all specs | no |
| `DEC-002` | `Frozen` | System has exactly 3 rails: `FullPrivacy`, `OnChainLite`, `MarketplaceSmallPaymentsRail`. | Blokuje mieszanie semantyki lite p2p i marketplace settlement. | freeze, protocol, consensus | no |
| `DEC-003` | `Frozen` | `FullPrivacy` remains mandatory for higher-value and sensitive flows and always allowed below the lite threshold. | Zachowuje glowny prywatny rail jako domyslny tor bezpieczny. | freeze, protocol, consensus | no |
| `DEC-004` | `Frozen` | Product unit is `PVA`, ledger atomic unit is `aPVA`, with `1 PVA = 10^12 aPVA`. | Rozdziela UX od ledger accounting i zamraza model jednostek. | freeze, formats, future amount migration | yes |
| `DEC-005` | `Frozen` | `MAX_LITE_TX_AMOUNT_PVA = 500 PVA` is a system validation rule, not a wire-format field. | Blokuje ukrywanie polityki w bytes i stabilizuje threshold semantics. | freeze, consensus | no |
| `DEC-006` | `Frozen` | `OnChainLite` is the rail name; `RecipientPrivacyLite` is a privacy claim, not a separate rail or tx type. | Usuwa dryf nazewniczy. | freeze, protocol | no |
| `DEC-007` | `Experimental` | `OnChainLite` remains experimental until it is fully tied into `note_root`, `state_root`, `ExecutionBundle`, proof coverage and threshold enforcement. | Blokuje pol-finalny rail. | protocol, consensus, vectors | yes |
| `DEC-008` | `Frozen` | `MarketplaceSmallPaymentsRail` is operator-trusted accounting, separate from lite p2p. | Zachowuje uczciwy trust model v0. | freeze, marketplace, consensus | no |
| `DEC-009` | `Current canonical` | Current `ServicePaymentPolicy` is the narrow code-defined form from `small_payments.rs`. | Zgrywa spec z implementacja bez udawania, ze rozszerzona policy juz istnieje. | formats, marketplace, vectors | no |
| `DEC-010` | `Future target requiring migration` | `ServicePaymentPolicy` will be extended to bind reservation / acceptance / refund / dispute / timeout / batching rules as `tag + params`. | Zamraza kierunek bez falszowania stanu obecnego. | formats, marketplace | yes |
| `DEC-011` | `Current canonical` | `SpendGrant` and `Receipt` signatures stay outside `grant_commit` and `receipt_commit`. | Rozdziela body, commitment i auth. | formats, marketplace, vectors | no |
| `DEC-012` | `Current canonical` | `MarketplaceBatchTx` verifier message is currently `settlement_root`. | To jest realny current code behavior i nie wolno go nadpisywac opisem targetu. | ledger, formats, vectors | no |
| `DEC-013` | `Future target requiring migration` | Richer batch signed payload may later become `canonical(core || summary || ticket_nullifiers)` under a dedicated domain. | To jest target, nie dzisiejszy verifier rule. | formats, consensus, vectors | yes |
| `DEC-014` | `Current canonical` | `MarketplaceBatchTx.operator_sig` remains encoded as a historical `vec-of-one` quirk. | Zabezpiecza current tx bytes i `tx_id` before any format migration. | tx, formats, vectors | yes |
| `DEC-015` | `Frozen` | `receipt_root` is defined as `H_receipt_root(merkle_root(receipt_commits))` with standard odd-leaf duplication. | Domyka receipt settlement anchor bit-to-bit. | formats, marketplace, vectors | no |
| `DEC-016` | `Current canonical` | `ExecutionBundle` and `ProofCertificate` have current canonical bytes and vectors. | Pozwala zamrozic bytes bez udawania, ze pelna proof semantics wszystkich raili jest juz domknieta. | consensus, vectors | no |
| `DEC-017` | `Unresolved` | Final proof coverage model for `OnChainLite` is not frozen. | To jest glowna przestrzen do dopowiadania i trzeba ja jawnie zamknac statusem. | consensus, proof boundaries | yes |
| `DEC-018` | `Frozen` | Escrow `2 z 3` belongs to `FullPrivacy`, not to `OnChainLite` and not to `MarketplaceSmallPaymentsRail`. | Utrzymuje escrow w prawidlowym trust i privacy modelu. | freeze, protocol, consensus, escrow adaptation | no |
| `DEC-019` | `Frozen` | `nexum-core` is adapted as workflow/control-plane only; Monero multisig execution is not part of the target runtime. | Rozdziela portable orchestration od Monero-specific custody and execution. | escrow adaptation, future integration | no |
| `DEC-020` | `Frozen` | Escrow `2 z 3` v1 uses separate Falcon signatures checked by ledger threshold rules, without requiring a Monero-style multisig address or aggregated threshold signature. | Stabilizuje techniczny model multisig dla `privAI` v1. | protocol, ledger, escrow adaptation | yes |
| `DEC-021` | `Frozen` | `nxms-transport` is not the final validator consensus wire protocol; it remains escrow/control-plane transport plus shared primitives. | Zatrzymuje mieszanie packet transportu escrow z validator networking. | transport, p2p, refactor planning | no |
| `DEC-022` | `Frozen` | Validator P2P requires a separate session transport layer between shared primitives and `privai-node` consensus overlay. | Zamyka docelowy split warstw i zmniejsza ryzyko dalszej duplikacji session logic. | transport, p2p, privai-node | yes |

## 4. Rules for adding entries

- nowy wpis dodajemy tylko wtedy, gdy:
  - decyzja jest juz zatwierdzona,
  - albo luka jest tak istotna, ze wymaga jawnego statusu `Unresolved`,
- wpis nie moze opisywac "pomyslu roboczego" bez statusu,
- wpis nie moze byc bardziej normatywny niz canonical docs, ktore wspiera,
- jesli decyzja zmienia bytes, domains, roots albo rail semantics, musi miec jawny link do migracji.

## 5. Forbidden inference

Z tego dokumentu nie wolno:
- dopisywac brakujacych enum tags,
- zgadywac signed payload formulas,
- zgadywac future bytes layout,
- zgadywac proof coverage modelu `OnChainLite`,
- traktowac targetu wymagajacego migracji jako current canonical behavior.
