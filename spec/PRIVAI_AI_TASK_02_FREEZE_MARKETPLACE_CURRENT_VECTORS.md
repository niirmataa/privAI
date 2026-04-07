# privAI AI Task 02: Freeze Marketplace Current Vectors

Status: active task draft.  
Canonicality: execution task for filling `Current canonical` marketplace vectors in `spec/PRIVAI_REFERENCE_VECTORS.md`.  
Owner: privAI protocol docs and vectors.  
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel

To zadanie ma zamrozic tylko obecne, juz dozwolone marketplace vectors dla:
- `ServicePaymentPolicy` w obecnej implementowanej postaci,
- `SpendGrantBody`,
- signed payload dla `SpendGrant`,
- `ReceiptBody`,
- signed payload dla `Receipt`,
- `SettlementBatchSummary`,
- `receipt_root`,
- `MarketplaceBatchTx` w obecnej implementowanej postaci,
- `tx_id` dla `MarketplaceBatchTx`.

To nie jest zadanie migracyjne.
To nie jest zadanie do rozszerzania policy format.
To nie jest zadanie do zmiany signing semantics.

## 2. Co jest w scope

W scope sa tylko rzeczy oznaczone jako:
- `Current canonical`
- `Frozen spec rule`

W praktyce oznacza to:
- obecny `ServicePaymentPolicy` z kodu,
- obecny `SpendGrant` / `Receipt` body-commit-signature split,
- obecny batch signing over `settlement_root`,
- obecny `MarketplaceBatchTx.operator_sig` jako `current frozen legacy quirk`,
- `receipt_root` jako frozen spec rule.

## 3. Czego nie wolno robic

Nie wolno:
- wdrazac rozszerzonego `ServicePaymentPolicy`,
- zmieniac `MarketplaceBatchTx.operator_sig` z `vec-of-one` na `bytes`,
- zmieniac signed payload formulas,
- ruszac `OnChainLite`,
- ruszac `ExecutionBundle` ani `ProofCertificate`,
- robic marketplace format migration,
- zmieniac wallet/ledger semantics tylko po to, zeby vectors "ladniej wygladaly".

Jesli agent znajdzie rozjazd:
- ma go opisac,
- ale nie moze samodzielnie zmienic spec scope.

## 4. Pliki do edycji

Agent ma prawo edytowac tylko:
- `spec/PRIVAI_REFERENCE_VECTORS.md`
- `privai-chain/src/small_payments.rs`
- `privai-chain/src/tx.rs`
- `privai-chain/src/canonical.rs`
- `privai-chain/src/hash.rs`
- `privai-chain/tests/reference_vectors_marketplace.rs`

Helper w innym pliku `privai-chain` wolno dodac tylko wtedy, gdy:
- nie zmienia production semantics,
- sluzy wylacznie do jednoznacznego odtworzenia current canonical bytes, commits albo merkle rule,
- agent jawnie to opisze w raporcie.

Nie wolno edytowac:
- `privai-ledger/*`
- `privai-wallet/*`
- `privai-proof/*`
- innych docs poza `spec/PRIVAI_REFERENCE_VECTORS.md`

## 5. Co dokladnie zrobic

### Krok 1. Ustal jeden deterministyczny zestaw sample objects

Agent ma przygotowac jeden stabilny zestaw przykladowych obiektow dla:
- `ServicePaymentPolicy`
- `SpendGrantBody`
- `ReceiptBody`
- `SettlementBatchSummary`
- `MarketplaceBatchTx`

Wszystkie sample values musza byc:
- deterministyczne,
- proste do review,
- zgodne z current code semantics.

### Krok 2. Wypelnij vectors w docs

W `spec/PRIVAI_REFERENCE_VECTORS.md` agent ma wypelnic tylko sekcje marketplace nalezace do `Current canonical` albo `Frozen spec rule`:
- `6.1. ServicePaymentPolicy` current code section
- `6.2. SpendGrantBody`
- `6.3. SpendGrantEnvelope`
- `6.4. ReceiptBody`
- `6.5. ReceiptEnvelope`
- `6.6. SettlementBatchSummary`
- `6.7. receipt_root`
- `6.8. MarketplaceBatchTx`

W tych sekcjach placeholdery maja zostac zastapione realnymi wartosciami.

### Krok 3. Dodaj testy

Agent ma dodac test file:
- `privai-chain/tests/reference_vectors_marketplace.rs`

Testy maja:
- sprawdzac exact canonical bytes,
- sprawdzac commits,
- sprawdzac signed payload bytes,
- sprawdzac `receipt_root`,
- sprawdzac `MarketplaceBatchTx.tx_id`,
- sprawdzac zgodnosc `docs == code`.

W testach:
- nie wolno polegac na `println!` jako zrodle prawdy,
- expected values maja byc wpisane jawnie jako stale,
- nazwy testow maja byc opisowe.

### Krok 4. Nie ruszaj production semantics

To zadanie ma zamrozic current vectors, a nie przebudowywac marketplace rail.

Dlatego:
- nie wolno zmieniac field order,
- nie wolno zmieniac signing target,
- nie wolno zmieniac `policy_commit`,
- nie wolno zmieniac `operator_sig` representation,
- nie wolno robic migration do future target policy fields.

## 6. Kryteria akceptacji

Zadanie jest uznane za wykonane tylko wtedy, gdy wszystkie ponizsze warunki sa spelnione:

- placeholdery w marketplace sections objetych scope zostaly zastapione realnymi wartosciami,
- wartosci odpowiadaja obecnemu `CanonicalEncode` i current formulas,
- `privai-chain/tests/reference_vectors_marketplace.rs` istnieje i przechodzi,
- testy potwierdzaja te same wartosci, ktore sa wpisane do docs,
- agent nie zmienil scope zadania,
- agent nie ruszyl future migration targets, `OnChainLite` ani provisional consensus objects.

## 7. Weryfikacja

Agent ma uruchomic:

```bash
cd /home/nxms-server/privAI/privai-chain
cargo test --test reference_vectors_marketplace
```

Jesli pelne `cargo test` przechodzi, moze to dopisac jako plus.
Jesli nie, ma przynajmniej uruchomic test file reference vectors i jawnie to napisac.

## 8. Format raportu koncowego

Agent ma oddac raport w tej formie:

1. `Co zostalo zrobione`
2. `Jakie sekcje vectors zostaly wypelnione`
3. `Jakie testy dodano`
4. `Czy byly blockery`
5. `Czy scope zostal utrzymany`
