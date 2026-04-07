# privAI Reentry Guide

Status: operational reentry doc for humans and agents resuming work after a pause.
Canonicality: non-overriding anti-drift guide. This document does not define bytes or semantics by itself; it tells readers where to re-enter the frozen context without rediscovering the architecture.
Owner: privAI spec governance.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_DECISION_REGISTER.md`
- `spec/PRIVAI_GAP_REGISTER.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
- `spec/PRIVAI_PROOF_BOUNDARIES.md`

## 1. Cel

Ten dokument istnieje po to, zeby po kilku dniach przerwy:
- nie czytac calego systemu od zera,
- nie mieszac starych i nowych docs,
- nie zgadywac architektury tam, gdzie jest jeszcze luka,
- szybko odrozniac:
  - co jest zamrozone,
  - co jest current canonical,
  - co jest targetem migracji,
  - co pozostaje experimental albo unresolved.

## 2. Minimalny powrot do kontekstu

Jesli wracasz po przerwie, czytaj w tej kolejnosci:

1. `spec/PRIVAI_SPEC_INDEX.md`
2. ten dokument
3. `spec/PRIVAI_DECISION_REGISTER.md`
4. `spec/PRIVAI_GAP_REGISTER.md`
5. `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
6. `spec/PRIVAI_PROTOCOL_CORE.md`
7. `spec/PRIVAI_CANONICAL_FORMATS.md`
8. `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
9. `spec/PRIVAI_CONSENSUS.md`
10. `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
11. `spec/PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
12. `spec/PRIVAI_PROOF_BOUNDARIES.md`
13. `spec/PRIVAI_REFERENCE_VECTORS.md`

## 3. Current system snapshot

Na dzis kierunek systemu jest finalny na poziomie architektury:
- trzy raile sa ustalone,
- `FullPrivacy` jest glownym prywatnym torem,
- marketplace rail ma jawny model `operator-trusted accounting`,
- `note/UTXO`, `ReceiveBundle`, `RecipientBox`, `Nullifier`, `master_seed` pozostaja rdzeniem,
- `PVA` i `aPVA` pozostaja docelowym modelem jednostek.

To, co nie jest jeszcze w 100% domkniete, to nie jest kierunek, tylko:
- wybrane migration gaps,
- proof boundaries,
- coin-side escrow multisig architecture do dobudowania,
- obszary z realnym potencjalem na wklad kryptograficzny,
- finalizacja lite raila,
- finalizacja amount layer.

## 4. Co jest zamrozone

- canonical source of truth lives under `spec/`
- trzy raile i ich rozdzial semantyczny
- `FullPrivacy` above-threshold / sensitive-flow rule
- marketplace as separate settlement world
- escrow `2 z 3` on `FullPrivacy` with `nexum-core` as control-plane adaptation target
- current marketplace signed semantics for `SpendGrant`, `Receipt` and batch verification over `settlement_root`
- current `MarketplaceBatchTx.operator_sig` legacy bytes quirk
- `receipt_root` formula
- current canonical bytes for `ExecutionBundle` and `ProofCertificate`

## 5. Co jest jeszcze otwarte

- rozszerzona `ServicePaymentPolicy`
- final mandatory operator auth path w ledgerze dla marketplace batcha
- coin-side `2 z 3` escrow auth model and escrow object model
- final proof model `OnChainLite`
- final amount encoding for `PVA + aPVA`
- ewentualna przyszla migracja batch signed payload i `operator_sig` bytes layout

## 6. Czego nie wolno robic

- nie wolno traktowac starych `PRIVAI_V0*` docs jako source of truth
- nie wolno uzupelniac unresolved miejsc "najbardziej prawdopodobna" interpretacja
- nie wolno wyciagac finalnej architektury z samego kodu, jesli canonical docs mowia inaczej
- nie wolno traktowac `OnChainLite` jako finalnego tylko dlatego, ze istnieje w kodzie
- nie wolno podmieniac current canonical verifier behavior opisem future target migration

## 7. Fast routing by task

Jesli task dotyczy:
- produktu i raili: czytaj `PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- core note semantics: czytaj `PRIVAI_PROTOCOL_CORE.md`
- bytes / domains / commitments / envelopes: czytaj `PRIVAI_CANONICAL_FORMATS.md`
- marketplace rail: czytaj `PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- consensus / roots / rail enforcement: czytaj `PRIVAI_CONSENSUS.md`
- escrow `2 z 3` / `nexum-core` adaptation: czytaj `PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- potencjalny wklad kryptograficzny / research direction: czytaj `PRIVAI_CRYPTOGRAPHIC_BREAKTHROUGH_CANDIDATES.md`
- bit-to-bit outputs: czytaj `PRIVAI_REFERENCE_VECTORS.md`
- statusu proof i czego nie wolno dopowiadac: czytaj `PRIVAI_PROOF_BOUNDARIES.md`
- zatwierdzonych decyzji: czytaj `PRIVAI_DECISION_REGISTER.md`
- otwartych luk: czytaj `PRIVAI_GAP_REGISTER.md`

## 8. Before changing code or docs

Przed zmiana:
- ustal, czy dotykasz `Current canonical`, `Frozen`, `Future target requiring migration`, `Experimental` czy `Unresolved`,
- sprawdz odpowiedni wpis w decision register i gap register,
- jesli obszar jest `Unresolved`, nie projektuj go lokalnie,
- jesli obszar wymaga migracji, opisz migracje jawnie zamiast udawac zgodnosc current behavior i final targetu.
