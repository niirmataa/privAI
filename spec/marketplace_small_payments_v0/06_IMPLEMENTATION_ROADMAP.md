# Marketplace Small Payments v0

## Implementation Roadmap

## 1. Cel

Ten dokument zamienia pakiet spec-first dla marketplace small-payments v0
w kolejnosc konkretnych zadan do odhaczania.

To nie jest roadmap "na kiedys".
To jest checklist wdrozeniowy:

- co trzeba domknac najpierw,
- czego nie wolno robic za wczesnie,
- kiedy mozna wejsc w ciezsza implementacje,
- jakie sa checkpointy review i freeze.

## 2. Zasada ogolna

Nie skaczemy od razu do:

- nowych typow tx,
- proof integration,
- wallet state machine,
- operator batching implementation,

zanim nie przejdziemy przez checkpoints spec i review.

Kolejnosc ma znaczenie.
Jesli etap wyzej nie jest zamkniety, etap nizej nie powinien byc uznany za "done".

## 3. Artefakty wejsciowe, ktore juz mamy

- [00_PREIMPLEMENTATION_READINESS.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/00_PREIMPLEMENTATION_READINESS.md)
- [01_SMALL_PAYMENTS_RAIL_V0.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/01_SMALL_PAYMENTS_RAIL_V0.md)
- [02_SERVICE_PAYMENT_POLICY.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/02_SERVICE_PAYMENT_POLICY.md)
- [03_TICKET_ID_AND_NULLIFIER.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/03_TICKET_ID_AND_NULLIFIER.md)
- [04_RECEIPT_AND_SETTLEMENT_ROOT.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md)
- [05_PRIVACY_TIERS.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/05_PRIVACY_TIERS.md)

## 4. Phase 0 - Freeze package and ownership

- [x] Wskazac wlasciciela decyzji dla:
  - privacy tiers
  - service payment policy
  - ticket/nullifier semantics
  - receipt/settlement semantics
  - economics and rollout gate
- [x] Potwierdzic, ze marketplace small-payments rail jest `marketplace-only`
- [x] Potwierdzic, ze zwykla drobnica poza marketplace idzie osobnym swiatem
- [x] Potwierdzic, ze `FullPrivacy` pozostaje wymagane dla:
  - escrow
  - dispute-sensitive flows
  - duzych kwot
  - amount-sensitive flows
- [x] Zamrozic wspolny slownik:
  - deposit
  - grant
  - ticket
  - nullifier
  - session/tab
  - receipt
  - settlement
  - refund
  - dispute

## 5. Phase 1 - Gemini / architecture pass

- [x] Przepuscic Gemini przez:
  - [02_SERVICE_PAYMENT_POLICY.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/02_SERVICE_PAYMENT_POLICY.md)
  - [03_TICKET_ID_AND_NULLIFIER.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/03_TICKET_ID_AND_NULLIFIER.md)
  - [04_RECEIPT_AND_SETTLEMENT_ROOT.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md)
  - [05_PRIVACY_TIERS.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/05_PRIVACY_TIERS.md)
- [x] Zrobic ludzki review odpowiedzi Gemini z 3 perspektyw:
  - protocol
  - adversarial/privacy
  - product/economics
- [x] Spisac diff:
  - co Gemini potwierdzilo
  - co wymaga poprawki
  - co nadal pozostaje otwarte
- [x] Zamknac ostatnie niejednoznacznosci w dokumentach `02-05`

## 6. Phase 2 - Final spec freeze

- [x] Zamrozic finalna definicje `ServicePaymentPolicy`
- [x] Zamrozic finalna definicje `SpendGrant`
- [x] Zamrozic finalna definicje `TicketId`
- [x] Zamrozic finalna definicje `TicketNullifier`
- [x] Zamrozic finalna definicje `Receipt`
- [x] Zamrozic finalna definicje `ReceiptRoot`
- [x] Zamrozic finalna definicje `SettlementRoot`
- [x] Zamrozic finalna definicje `SettlementTx`
- [x] Zamrozic finalna macierz privacy-tier selection
- [x] Zamrozic finalne zasady:
  - refund
  - timeout
  - partial settlement
  - escalation to `FullPrivacy`

## 7. Phase 3 - Canonical serialization and commits

- [x] Zdefiniowac canonical byte encoding dla:
  - `ServicePaymentPolicy`
  - `SpendGrant`
  - `Receipt`
  - `SettlementBatchSummary`
  - `SettlementTx`
- [x] Zdefiniowac commitment/hash domains dla:
  - `policy_commit`
  - `grant_commit`
  - `purchase_commit`
  - `receipt_commit`
  - `receipt_root`
  - `settlement_root`
- [x] Zdefiniowac stable hashing rules:
  - field order
  - length-prefixing
  - versioning
- [x] Zrobic test vectors dla encoding i commitow

## 8. Phase 4 - Chain surface design

- [x] Okreslic, co jest on-chain typem, a co tylko off-chain objectem
- [x] Rozpisac minimalne on-chain pola dla `SettlementTx`
- [x] Rozpisac model globalnego `ticket_nullifier` set
- [x] Rozpisac model authority:
  - kto moze publikowac settlement
  - jak jest identyfikowany operator
  - czy delegated actor jest dopuszczalny
- [x] Rozpisac walidacje konsensusu:
  - uniqueness
  - authority
  - totals
  - roots/header consistency
- [x] Rozpisac state transitions:
  - accepted settlement
  - duplicate nullifier rejection
  - expired settlement rejection
  - malformed batch rejection

## 9. Phase 5 - Wallet architecture

- [x] Zdefiniowac wallet rail selector:
  - `SmallPaymentsRail`
  - `RecipientPrivacyLite`
  - `FullPrivacy`
- [x] Zdefiniowac local rail context po prywatnym depozycie
- [x] Zdefiniowac storage dla:
  - ticket seed
  - counters
  - session state
  - grant state
  - receipt state
- [x] Zdefiniowac recovery model:
  - single device
  - multi-device
  - stale ticket handling
- [x] Zdefiniowac wallet checks:
  - policy compatibility
  - grant validity
  - spend cap safety
  - timeout safety
- [x] Zdefiniowac wallet UX decyzji:
  - kiedy pokazuje lekki rail
  - kiedy wymusza `FullPrivacy`
  - kiedy pokazuje jawna kwote jako tradeoff

## 10. Phase 6 - Marketplace/operator architecture

- [x] Zdefiniowac service, ktory wystawia `SessionGrant/SpendGrant`
- [x] Zdefiniowac merchant/operator handshake dla session open
- [x] Zdefiniowac receipt ingestion path
- [x] Zdefiniowac receipt retention policy
- [x] Zdefiniowac settlement batch builder
- [x] Zdefiniowac refund batch / timeout handler
- [x] Zdefiniowac audit export:
  - receipts by batch
  - settlement summary
  - refund summary
- [x] Zdefiniowac operator controls:
  - max batch size
  - batching windows
  - stale session cleanup
  - duplicate receipt protection

## 11. Phase 7 - Merchant integration

- [x] Zdefiniowac merchant-visible session API
- [x] Zdefiniowac merchant-visible debit acceptance API
- [x] Zdefiniowac merchant receipt schema and signing flow
- [x] Zdefiniowac merchant-side replay cache rules
- [x] Zdefiniowac merchant-side failure handling:
  - duplicate ticket attempt
  - expired grant
  - service timeout
  - partial delivery
- [x] Zdefiniowac minimalne wymagania retencji po stronie merchanta

## 12. Phase 8 - Protocol implementation order

### 8.1. Safe first implementation slice

- [x] Dac do repo tylko typy/spec helpers bez aktywacji konsensusowej
- [x] Dodac canonical structs i serialization tests
- [x] Dodac policy/grant validation helpers
- [x] Dodac receipt and settlement batch builders jako local/off-chain modules

### 8.2. Chain implementation slice

- [x] Dodac `SettlementTx` type (MarketplaceBatchTx z jawna lista nullifierow)
- [x] Dodac nullifier uniqueness validation (w privai-ledger/src/ledger.rs)
- [x] Dodac operator authority validation (jako struktury na on-chain)
- [x] Dodac root/header/totals validation (Merkle root dla receiptow i settlement batch)
- [x] Dodac timeout/refund state handling (w SettlementBatchSummary i operator.rs)

### 8.3. Wallet implementation slice

- [x] Dodac rail selector (AllowedRail enum)
- [x] Dodac local grant/session/ticket state (RailContext, WalletSnapshot, ManagedBundle)
- [x] Dodac ticket generation (generate_next_ticket() przez blake3 KDF)
- [x] Dodac receipt tracking (struktura Receipt i logiki)
- [x] Dodac recovery path (re-generacja biletow z rail_seed)

### 8.4. Operator implementation slice

- [x] Dodac grant issuance (w privai-wallet/src/operator.rs)
- [x] Dodac receipt intake (weryfikacja capa, tracking pending_receipts)
- [x] Dodac batch settlement publisher (zlozenie on-chain struktury MarketplaceBatchTx)
- [x] Dodac refund/timeout processor (auto-refunds z pozostalosci budzetu)

## 13. Phase 9 - Testing and adversarial validation

- [x] Unit tests dla:
  - canonical encoding
  - commit derivation
  - policy checks
  - grant checks
  - nullifier uniqueness
- [x] Integration tests dla:
  - deposit -> grant -> session -> receipt -> settlement
  - duplicate nullifier rejection
  - expired grant rejection
  - timeout refund path
  - partial settlement path
- [x] Adversarial tests dla:
  - replay attempts
  - merchant overcharge attempts
  - operator malformed batch
  - stale receipt replay
  - invalid delegated publisher
- [x] Economics tests:
  - bytes per batch
  - bytes per debit
  - operator batch overhead
  - refund path overhead

## 14. Phase 10 - Rollout gate

- [x] Zrobic finalny architecture review
- [x] Zrobic privacy/adversarial review
- [x] Zrobic product/economics review
- [x] Potwierdzic, ze:
  - lekki rail nie wycieka stalego konta
  - `FullPrivacy` escalation dziala
  - settlement assumptions sa nazwane po imieniu
  - operator trust model jest zaakceptowany
- [x] Potwierdzic rollout constraints:
  - marketplace-only
  - no public generalization yet
  - no hidden-amount promise for this rail

## 15. Definition of Done for spec-first phase

Spec-first phase jest skonczona dopiero wtedy, gdy:

- [x] dokumenty `02-05` sa po review i freeze
- [x] roadmap ma przypisanych wlascicieli etapow
- [x] serialization i commit rules sa zamrozone
- [x] chain surface jest rozpisany
- [x] wallet/operator/merchant responsibilities sa rozpisane
- [x] test plan jest gotowy
- [x] rollout gate jest jawny

## 16. Minimalny nastepny ruch

Jesli chcemy isc bez chaosu, nastepna kolejnosc jest taka:

1. [x] Przepuscic Gemini przez `02-05`
2. [x] Zrobic ludzki review odpowiedzi
3. [x] Domknac ostatnie otwarte decyzje
4. [x] Zamrozic canonical serialization i commit rules
5. [x] Dopiero potem wejsc w implementacje chain/wallet/operator
