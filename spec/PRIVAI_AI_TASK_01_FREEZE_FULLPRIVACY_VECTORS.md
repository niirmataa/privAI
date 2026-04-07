# privAI AI Task 01 — Freeze Current Canonical FullPrivacy Vectors

Status: active implementation task.
Canonicality: execution task derived from the canonical `spec/` set. This document is not a new source of truth; it tells an agent what to implement under the existing frozen docs.
Owner: privAI spec-to-code alignment.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`

## 1. Cel zadania

Celem tego zadania jest zamrozenie pierwszego realnego pakietu bit-to-bit reference vectors dla rdzenia `FullPrivacy`.

Ten task ma:
- wypelnic konkretne placeholdery w `spec/PRIVAI_REFERENCE_VECTORS.md`,
- dodac testy w kodzie potwierdzajace te same bytes i commitmenty,
- zamknac pierwszy wycinek `docs == code` bez ruszania rzeczy przejsciowych.

To jest zadanie na:
- `Current canonical`
- `Frozen spec rule`

To nie jest zadanie na:
- migracje formatu,
- zmiane architektury,
- doprojektowanie lite raila,
- doprojektowanie marketplace future target.

## 2. Twardy scope

Agent ma pracowac tylko nad `FullPrivacy` core i bezpiecznymi vectors dla aktualnego canonical body.

### 2.1. W scope

W scope sa tylko:
- `ReceiveBundle`
- `SpendPolicy::Single`
- `SpendPolicy::MarketplaceSettlement`
- `RecipientBox`
- `RecipientBoxPlaintext`
- `AuxWitness`
- `OutputNote`
- `Nullifier`
- `TransferNoteTx`
- `tx_id`
- `tx_root` example
- boundary vectors:
  - `None` vs `Some`
  - one-field mutation changes commit
  - odd-leaf duplication

### 2.2. Poza scope

Poza scope sa bez wyjatkow:
- `ServicePaymentPolicy`
- `SpendGrant`
- `Receipt`
- `SettlementBatchSummary`
- `MarketplaceBatchTx`
- `receipt_root`
- `ExecutionBundle`
- `ProofCertificate`
- wszystko z `OnChainLite`
- `LiteOutputNote`
- `LiteTransferTx`
- migracje `PVA + aPVA`
- zmiany signed-envelope semantics

Jesli agent wejdzie w cos spoza tej listy, to jest to blad scope.

## 3. Jedyny dozwolony model interpretacji

Agent musi czytac statusy dokladnie tak, jak sa zdefiniowane w:
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`

Zamrozona zasada:
- jesli cos nie jest jawnie `Current canonical` albo `Frozen spec rule`, agent nie moze tego uzupelniac przez zgadywanie,
- jesli cos okazuje sie `unresolved`, agent ma to zostawic nietkniete i zglosic jako blocker,
- kod poza `spec/` sluzy tylko do odczytania current behavior i wygenerowania referencyjnych wartosci dla rzeczy juz dozwolonych w scope.

## 4. Pliki do edycji

Agent ma prawo edytowac tylko:
- `spec/PRIVAI_REFERENCE_VECTORS.md`
- `privai-chain/src/note.rs`
- `privai-chain/src/tx.rs`
- `privai-chain/src/consensus.rs`
- `privai-chain/src/primitives.rs`
- `privai-chain/src/canonical.rs`
- `privai-chain/tests/reference_vectors_fullprivacy.rs`

Jesli do wykonania testow wygodniej bedzie dodac helper w innym pliku `privai-chain`, agent moze to zrobic tylko wtedy, gdy:
- helper nie zmienia semantyki produkcyjnej,
- helper sluzy tylko do jednoznacznego odtworzenia current canonical bytes lub merkle rules,
- agent jasno to opisze w raporcie.

Nie wolno edytowac:
- `privai-ledger/*`
- `privai-wallet/*`
- `privai-proof/*`
- marketplace docs
- consensus docs poza ewentualnym odczytem referencyjnym

## 5. Co dokladnie zrobic

### Krok 1. Ustal jeden deterministyczny zestaw sample objects

Agent ma przygotowac jeden stabilny zestaw przykladowych danych wejsciowych dla wszystkich obiektow w scope.

Wymagania:
- zadnych losowych danych,
- zadnych zaleznosci od czasu systemowego,
- zadnych zaleznosci od zewnetrznych kluczy,
- bajty musza byc stale i powtarzalne.

Jesli jakis byt wymaga pola opcjonalnego:
- przygotuj przynajmniej jeden case z `None`,
- i jeden boundary case z `Some(...)`, jesli sekcja vectors tego wymaga.

### Krok 2. Wyciagnij current canonical bytes i expected commitments z kodu

Agent ma wyliczyc z implementacji:
- canonical bytes,
- payload bytes tam, gdzie dotyczy,
- payload commitment tam, gdzie dotyczy,
- final commitment / hash / tx_id / root.

Agent nie moze:
- przepisywac przyblizen,
- wpisywac placeholderow udajacych finalna wartosc,
- zmieniac formuly po to, zeby "ladniej pasowaly" do docs.

### Krok 3. Wypelnij tylko dozwolone sekcje w `PRIVAI_REFERENCE_VECTORS.md`

Agent ma wypelnic konkretne wartosci w nastepujacych sekcjach:

- `4.1. ReceiveBundle`
- `4.2. SpendPolicy::Single`
- `4.3. SpendPolicy::MarketplaceSettlement`
- `4.4. RecipientBox`
- `4.5. RecipientBoxPlaintext`
- `4.6. AuxWitness`
- `4.7. OutputNote`
- `4.8. Nullifier derivation`
- `5.1. TransferNoteTx`
- `5.2. tx_root example`
- `8.1. Option encoding: None vs Some`
- `8.2. One-field mutation changes commit`
- `8.3. Merkle odd-leaf duplication`

W tych sekcjach agent ma:
- zastapic placeholdery realnymi wartosciami,
- zostawic lowercase hex bez `0x`,
- zachowac format dokumentu,
- nie dopisywac nowych finalnych obiektow poza scope.

### Krok 4. Dodaj testy reference vectors w `privai-chain`

Agent ma dodac plik:
- `privai-chain/tests/reference_vectors_fullprivacy.rs`

Testy maja sprawdzac co najmniej:
- canonical bytes dla obiektow w scope,
- `bundle_commit`,
- `spend_policy_commit`,
- `note_payload_commit`,
- `aux_commit`,
- `note_commit`,
- `nullifier`,
- `tx_id`,
- `tx_root`,
- `None` vs `Some` daje inne bytes i inny commit tam, gdzie dotyczy,
- zmiana jednego pola zmienia commit,
- merkle odd-leaf duplication daje oczekiwany root.

W testach:
- nie wolno polegac na println jako zrodle prawdy,
- expected values musza byc jawnie wpisane jako stale referencyjne,
- nazwy testow maja byc opisowe, np. `test_reference_vector_receive_bundle_commit`.

### Krok 5. Nie ruszaj production semantics

To zadanie ma zamrozic vectors, a nie przebudowywac system.

Dlatego:
- nie wolno zmieniac kolejnosci pol,
- nie wolno zmieniac domain strings,
- nie wolno zmieniac formula commitments,
- nie wolno "naprawiac" `Amount14`,
- nie wolno dotykac `OnChainLite`,
- nie wolno wprowadzac marketplace migration.

Jesli agent znajdzie rozjazd:
- ma go opisac w raporcie,
- ale nie wolno mu samemu zmienic spec scope.

## 6. Kryteria akceptacji

Zadanie jest uznane za wykonane tylko wtedy, gdy wszystkie ponizsze warunki sa spelnione:

- placeholdery w dozwolonych sekcjach `PRIVAI_REFERENCE_VECTORS.md` zostaly zastapione realnymi wartosciami,
- wartosci odpowiadaja obecnemu `CanonicalEncode` i current formulas,
- `privai-chain/tests/reference_vectors_fullprivacy.rs` istnieje i przechodzi,
- testy potwierdzaja te same wartosci, ktore sa wpisane do docs,
- agent nie zmienil scope zadania,
- agent nie dotknal `OnChainLite`, marketplace migration ani provisional consensus objects.

## 7. Weryfikacja

Agent ma uruchomic:

```bash
cd /home/nxms-server/privAI/privai-chain
cargo test
```

Jesli testy w calej paczce sa zbyt szerokie przez niezalezne problemy:
- agent ma przynajmniej uruchomic test file reference vectors,
- ale ma to jawnie napisac w raporcie.

## 8. Format raportu koncowego

Agent ma oddac raport w tej formie:

1. `Co zostalo zrobione`
2. `Jakie sekcje vectors zostaly wypelnione`
3. `Jakie testy dodano`
4. `Czy wszystko przechodzi`
5. `Jakie blockery lub current non-conformities zostaly znalezione`

Agent ma tez podac liste zmienionych plikow.

## 9. Twarde zakazy

Niedozwolone jest:
- zmienianie canonical docs poza zakresem taska,
- dopisywanie future target values do current vectors,
- wypelnianie placeholderow przez zgadywanie,
- mieszanie finalnych vectors z `experimental appendix`,
- zmiana semantyki systemu pod pretekstem "łatwiejszych testów",
- ruszanie lite raila,
- ruszanie marketplace migration,
- ruszanie `ExecutionBundle` lub `ProofCertificate`.

## 10. Jednozdaniowy cel dla agenta

Zamroz pierwszy realny pakiet `current canonical` vectors dla rdzenia `FullPrivacy` w `privai-chain` i `spec/PRIVAI_REFERENCE_VECTORS.md`, bez zmiany architektury, bez lite raila i bez marketplace migration.
