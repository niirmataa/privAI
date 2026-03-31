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

- [ ] Wskazac wlasciciela decyzji dla:
  - privacy tiers
  - service payment policy
  - ticket/nullifier semantics
  - receipt/settlement semantics
  - economics and rollout gate
- [ ] Potwierdzic, ze marketplace small-payments rail jest `marketplace-only`
- [ ] Potwierdzic, ze zwykla drobnica poza marketplace idzie osobnym swiatem
- [ ] Potwierdzic, ze `FullPrivacy` pozostaje wymagane dla:
  - escrow
  - dispute-sensitive flows
  - duzych kwot
  - amount-sensitive flows
- [ ] Zamrozic wspolny slownik:
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

- [ ] Przepuscic Gemini przez:
  - [02_SERVICE_PAYMENT_POLICY.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/02_SERVICE_PAYMENT_POLICY.md)
  - [03_TICKET_ID_AND_NULLIFIER.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/03_TICKET_ID_AND_NULLIFIER.md)
  - [04_RECEIPT_AND_SETTLEMENT_ROOT.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md)
  - [05_PRIVACY_TIERS.md](/home/nxms-server/privAI/spec/marketplace_small_payments_v0/05_PRIVACY_TIERS.md)
- [ ] Zrobic ludzki review odpowiedzi Gemini z 3 perspektyw:
  - protocol
  - adversarial/privacy
  - product/economics
- [ ] Spisac diff:
  - co Gemini potwierdzilo
  - co wymaga poprawki
  - co nadal pozostaje otwarte
- [ ] Zamknac ostatnie niejednoznacznosci w dokumentach `02-05`

## 6. Phase 2 - Final spec freeze

- [ ] Zamrozic finalna definicje `ServicePaymentPolicy`
- [ ] Zamrozic finalna definicje `SpendGrant`
- [ ] Zamrozic finalna definicje `TicketId`
- [ ] Zamrozic finalna definicje `TicketNullifier`
- [ ] Zamrozic finalna definicje `Receipt`
- [ ] Zamrozic finalna definicje `ReceiptRoot`
- [ ] Zamrozic finalna definicje `SettlementRoot`
- [ ] Zamrozic finalna definicje `SettlementTx`
- [ ] Zamrozic finalna macierz privacy-tier selection
- [ ] Zamrozic finalne zasady:
  - refund
  - timeout
  - partial settlement
  - escalation to `FullPrivacy`

## 7. Phase 3 - Canonical serialization and commits

- [ ] Zdefiniowac canonical byte encoding dla:
  - `ServicePaymentPolicy`
  - `SpendGrant`
  - `Receipt`
  - `SettlementBatchSummary`
  - `SettlementTx`
- [ ] Zdefiniowac commitment/hash domains dla:
  - `policy_commit`
  - `grant_commit`
  - `purchase_commit`
  - `receipt_commit`
  - `receipt_root`
  - `settlement_root`
- [ ] Zdefiniowac stable hashing rules:
  - field order
  - length-prefixing
  - versioning
- [ ] Zrobic test vectors dla encoding i commitow

## 8. Phase 4 - Chain surface design

- [ ] Okreslic, co jest on-chain typem, a co tylko off-chain objectem
- [ ] Rozpisac minimalne on-chain pola dla `SettlementTx`
- [ ] Rozpisac model globalnego `ticket_nullifier` set
- [ ] Rozpisac model authority:
  - kto moze publikowac settlement
  - jak jest identyfikowany operator
  - czy delegated actor jest dopuszczalny
- [ ] Rozpisac walidacje konsensusu:
  - uniqueness
  - authority
  - totals
  - roots/header consistency
- [ ] Rozpisac state transitions:
  - accepted settlement
  - duplicate nullifier rejection
  - expired settlement rejection
  - malformed batch rejection

## 9. Phase 5 - Wallet architecture

- [ ] Zdefiniowac wallet rail selector:
  - `SmallPaymentsRail`
  - `RecipientPrivacyLite`
  - `FullPrivacy`
- [ ] Zdefiniowac local rail context po prywatnym depozycie
- [ ] Zdefiniowac storage dla:
  - ticket seed
  - counters
  - session state
  - grant state
  - receipt state
- [ ] Zdefiniowac recovery model:
  - single device
  - multi-device
  - stale ticket handling
- [ ] Zdefiniowac wallet checks:
  - policy compatibility
  - grant validity
  - spend cap safety
  - timeout safety
- [ ] Zdefiniowac wallet UX decyzji:
  - kiedy pokazuje lekki rail
  - kiedy wymusza `FullPrivacy`
  - kiedy pokazuje jawna kwote jako tradeoff

## 10. Phase 6 - Marketplace/operator architecture

- [ ] Zdefiniowac service, ktory wystawia `SessionGrant/SpendGrant`
- [ ] Zdefiniowac merchant/operator handshake dla session open
- [ ] Zdefiniowac receipt ingestion path
- [ ] Zdefiniowac receipt retention policy
- [ ] Zdefiniowac settlement batch builder
- [ ] Zdefiniowac refund batch / timeout handler
- [ ] Zdefiniowac audit export:
  - receipts by batch
  - settlement summary
  - refund summary
- [ ] Zdefiniowac operator controls:
  - max batch size
  - batching windows
  - stale session cleanup
  - duplicate receipt protection

## 11. Phase 7 - Merchant integration

- [ ] Zdefiniowac merchant-visible session API
- [ ] Zdefiniowac merchant-visible debit acceptance API
- [ ] Zdefiniowac merchant receipt schema and signing flow
- [ ] Zdefiniowac merchant-side replay cache rules
- [ ] Zdefiniowac merchant-side failure handling:
  - duplicate ticket attempt
  - expired grant
  - service timeout
  - partial delivery
- [ ] Zdefiniowac minimalne wymagania retencji po stronie merchanta

## 12. Phase 8 - Protocol implementation order

### 8.1. Safe first implementation slice

- [ ] Dac do repo tylko typy/spec helpers bez aktywacji konsensusowej
- [ ] Dodac canonical structs i serialization tests
- [ ] Dodac policy/grant validation helpers
- [ ] Dodac receipt and settlement batch builders jako local/off-chain modules

### 8.2. Chain implementation slice

- [ ] Dodac `SettlementTx` type
- [ ] Dodac nullifier uniqueness validation
- [ ] Dodac operator authority validation
- [ ] Dodac root/header/totals validation
- [ ] Dodac timeout/refund state handling

### 8.3. Wallet implementation slice

- [ ] Dodac rail selector
- [ ] Dodac local grant/session/ticket state
- [ ] Dodac ticket generation
- [ ] Dodac receipt tracking
- [ ] Dodac recovery path

### 8.4. Operator implementation slice

- [ ] Dodac grant issuance
- [ ] Dodac receipt intake
- [ ] Dodac batch settlement publisher
- [ ] Dodac refund/timeout processor

## 13. Phase 9 - Testing and adversarial validation

- [ ] Unit tests dla:
  - canonical encoding
  - commit derivation
  - policy checks
  - grant checks
  - nullifier uniqueness
- [ ] Integration tests dla:
  - deposit -> grant -> session -> receipt -> settlement
  - duplicate nullifier rejection
  - expired grant rejection
  - timeout refund path
  - partial settlement path
- [ ] Adversarial tests dla:
  - replay attempts
  - merchant overcharge attempts
  - operator malformed batch
  - stale receipt replay
  - invalid delegated publisher
- [ ] Economics tests:
  - bytes per batch
  - bytes per debit
  - operator batch overhead
  - refund path overhead

## 14. Phase 10 - Rollout gate

- [ ] Zrobic finalny architecture review
- [ ] Zrobic privacy/adversarial review
- [ ] Zrobic product/economics review
- [ ] Potwierdzic, ze:
  - lekki rail nie wycieka stalego konta
  - `FullPrivacy` escalation dziala
  - settlement assumptions sa nazwane po imieniu
  - operator trust model jest zaakceptowany
- [ ] Potwierdzic rollout constraints:
  - marketplace-only
  - no public generalization yet
  - no hidden-amount promise for this rail

## 15. Definition of Done for spec-first phase

Spec-first phase jest skonczona dopiero wtedy, gdy:

- [ ] dokumenty `02-05` sa po review i freeze
- [ ] roadmap ma przypisanych wlascicieli etapow
- [ ] serialization i commit rules sa zamrozone
- [ ] chain surface jest rozpisany
- [ ] wallet/operator/merchant responsibilities sa rozpisane
- [ ] test plan jest gotowy
- [ ] rollout gate jest jawny

## 16. Minimalny nastepny ruch

Jesli chcemy isc bez chaosu, nastepna kolejnosc jest taka:

1. [ ] Przepuscic Gemini przez `02-05`
2. [ ] Zrobic ludzki review odpowiedzi
3. [ ] Domknac ostatnie otwarte decyzje
4. [ ] Zamrozic canonical serialization i commit rules
5. [ ] Dopiero potem wejsc w implementacje chain/wallet/operator
