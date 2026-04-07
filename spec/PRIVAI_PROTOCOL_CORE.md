# privAI Protocol Core

Status: draft canonical protocol-core doc in migration.
Canonicality: intended canonical source of truth for protocol-core semantics. Product-level policy remains governed by `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`. Exact bytes remain governed by `spec/PRIVAI_CANONICAL_FORMATS.md`. Pliki poza `spec/` nie sa normatywnym source of truth.
Owner: privAI protocol.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Supersedes: czesciowo `PRIVAI_V0_PROTOCOL.md` oraz semantyczne fragmenty `PRIVAI_V0_FORMATS.md`.

## 1. Cel

Ten dokument opisuje finalna semantyke rdzenia protokolu `privAI`:
- model `note/UTXO`,
- obiekty odbioru i spendu,
- odpowiedzialnosci wallet/ledger/consensus,
- kanoniczne klasy transakcji na poziomie protokolowym,
- rozdzial miedzy stanem finalnym a stanem obecnej implementacji.

To nie jest dokument wire format.
To nie jest tez dokument ekonomii produktu.

Ten dokument ma odpowiedziec jasno:
- jakie byty naleza do rdzenia protokolu,
- co one znacza,
- jak maja wspolpracowac,
- co jest juz finalnym kierunkiem,
- a co pozostaje przejsciowe albo `experimental`.

## 1.1. How Devs And Agents Should Use This Doc

Ten dokument jest zrodlem prawdy dla semantyki rdzenia protokolu.

Czytaj go razem z:
- `spec/PRIVAI_SPEC_INDEX.md` jako punktem wejscia do calego frozen setu,
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md` dla polityki produktu i raili,
- `spec/PRIVAI_CANONICAL_FORMATS.md` dla exact bytes i commitments,
- `spec/PRIVAI_CONSENSUS.md` dla exact block validity i finality rules.

Zamrozona zasada pracy:
- jesli task dotyczy znaczenia `ReceiveBundle`, `RecipientBox`, `OutputNote`, `Nullifier`, tx classes albo lifecycle not, to ten dokument jest punktem wyjscia,
- sama obecnosc bytu w kodzie nie daje mu finalnej semantyki,
- referencje do kodu albo do starych docs poza `spec/` sa tylko pomocnicze i nie sa source of truth,
- jesli kod rozjezdza sie z tym dokumentem, nalezy traktowac to jako gap implementacyjny, chyba ze dokument jawnie oznacza cos jako `current non-conformity` albo `experimental`.

Jawnego freeze update wymagaja zawsze:
- zmiana semantyki core note objects,
- zmiana lifecycle bundli lub not,
- zmiana finalnych tx classes,
- zmiana odpowiedzialnosci wallet/ledger/consensus na poziomie protokolowym.

Interpretation rule for this document:
- status labels sa zdefiniowane w `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`,
- jesli jakas semantyka nie jest tu oznaczona jako `Current canonical`, `Frozen spec rule`, `Frozen future target requiring migration`, `Provisional`, `Experimental` albo `Current non-conformity`, nalezy traktowac ja jako `unresolved`,
- `unresolved` nie wolno domykac przez lokalne zgadywanie tx semantics, note semantics ani wallet behavior.

## 2. Zakres i granice

Ten dokument obejmuje:
- semantyke `ReceiveBundle`, `RecipientBox`, `RecipientBoxPlaintext`, `OutputNote`, `Nullifier`,
- semantyke kluczy walleta i hierarchii `master_seed`,
- semantyke finalnych platniczych tx classes,
- stan i lifecycle bundli oraz not,
- granice odpowiedzialnosci miedzy wallet, ledger i consensus.

Ten dokument nie obejmuje:
- exact canonical bytes i kolejnosci pol bit-po-bicie,
- finalnego opisu wszystkich hash domains,
- golden vectors,
- szczegolowej ekonomii marketplace,
- finalnej specyfikacji consensus.

## 3. Slownik kanoniczny

Finalny slownik systemu jest nastepujacy:
- projekt: `privAI`
- coin: `privAI Coin`
- ticker: `PVA`
- finalna glowna jednostka uzytkowa: `PVA`
- finalna najmniejsza jednostka ledgerowa: `aPVA`
- glowny rail prywatny: `FullPrivacy`
- lekki rail on-chain dla malych platnosci: `OnChainLite`
- marketplace-only rail: `MarketplaceSmallPaymentsRail`

Regula przejsciowa:
- `OnChainLite` jest nazwa raila / toru platnosci,
- `RecipientPrivacyLite` jest nazwa obietnicy prywatnosci, a nie nazwa osobnego finalnego raila ani finalnej nazwy tx type,
- dopoki implementacja i docs nie zostana zsynchronizowane, `RecipientPrivacyLite` nalezy czytac jako przejsciowy label dla obecnego eksperymentalnego lite path,
- `MarketplaceSmallPaymentsRail` nie jest aliasem `OnChainLite`.

## 4. Finalny model ledgera

`privAI` pozostaje systemem `note/UTXO`.

Zamrozona semantyka:
- wartosc i prawo do wydania zyja w nocie, nie w stale aktualizowanym publicznym koncie,
- output tworzy nowa note z nowym `note_commit`,
- spend zuzywa note przez `input_nullifier`,
- replay protection i anti-double-spend opieraja sie o nullifiery,
- prywatnosc odbiorcy opiera sie na jednorazowym bundle i `RecipientBox`,
- `FullPrivacy` pozostaje podstawowym finalnym modelem dla flow wrazliwych i wyzszych kwot.

## 5. Obiekty rdzenia protokolu

### 5.1. SpendPolicy

`SpendPolicy` opisuje warunek wydania noty.

Stan obecny kodu definiuje dwa podstawowe warianty:
- `Single`
- `MarketplaceSettlement`

Semantyka:
- `Single` wiaze note z jednym docelowym kluczem/autoryzacja spendera,
- `MarketplaceSettlement` opisuje bardziej zlozony warunek settlementowy typu marketplace/escrow.

Finalna zasada:
- `SpendPolicy` nalezy do rdzenia protokolu,
- `SpendPolicy.commitment()` jest czescia wiarygodnosci noty,
- otwarcie noty musi dawac mozliwosc weryfikacji, ze `spend_policy_opening` odpowiada `spend_policy_commit`.

### 5.2. ReceiveBundle

`ReceiveBundle` jest jednorazowym bytem odbiorczym.

Semantyka finalna:
- bundle identyfikuje jednorazowy punkt odbioru,
- zawiera material potrzebny do:
  - odbioru noty,
  - otwarcia `RecipientBox`,
  - zbudowania lub zweryfikowania powiazania odbiorcy z outputem,
- bundle ma lifecycle i nie powinien byc re-uzywany po konsumpcji.

Stan obecny:
- bundle niesie `bundle_id`,
- jednorazowy Falcon public key,
- jednorazowy Frodo public key,
- optional `route_hint`,
- pole `nullifier_key`.

Finalna zasada bezpieczenstwa:
- `nullifier_key` jest czescia formatu bundla,
- nadawca nie moze semantycznie narzucic arbitralnego `nullifier_key`,
- wallet odbiorcy nie moze akceptowac `nullifier_key` tylko dlatego, ze pole jest obecne w bundlu,
- wallet odbiorcy musi weryfikowac poprawnosc `nullifier_key` podczas otwierania noty.

### 5.3. RecipientBox

`RecipientBox` jest zaszyfrowanym kontenerem dla danych odbiorcy.

Semantyka finalna:
- chroni dane potrzebne do odbioru i wydania noty,
- korzysta z KEM + AEAD,
- zawiera `hint`, ktory pozwala walletowi szybko odfiltrowac potencjalnie nalezace do niego outputy,
- nie daje mozliwosci otwarcia bez lokalnych secret keys odbiorcy.

Finalna zasada:
- `hint` sluzy do scan/filter,
- ale sam `hint` nie jest dowodem wlasnosci,
- dowodem jest poprawne otwarcie boxa i weryfikacja plaintextu.

### 5.4. RecipientBoxPlaintext

`RecipientBoxPlaintext` jest kanonicznym otwarciem boxa odbiorcy.

Semantyka finalna:
- wiaze bundle z konkretnym payloadem noty,
- zawiera:
  - `bundle_id`,
  - `note_payload_commit`,
  - `amount` w reprezentacji aktualnie obslugiwanej przez amount layer, docelowo zgodnej z finalnym modelem `PVA + aPVA`,
  - `witness_seed`,
  - `nullifier_key`,
  - otwarcia `SpendPolicy` i `AuxWitness`,
  - opcjonalny `sender_memo`,
- jest tym, co wallet weryfikuje po KEM decapsulation i AEAD decrypt.

Finalna zasada:
- `note_payload_commit` jest prawidlowym bindingiem payloadu noty,
- `nullifier_key` jest czescia canonical payloadu, ale jego akceptacja wymaga pozytywnej wallet verification i expected derivation bindingu,
- nie nalezy wracac do starszej semantyki, w ktorej plaintext trzymal stare `note_commit` zamiast payload commitment.

### 5.5. AuxWitness

`AuxWitness` wiaze note z dodatkowymi danymi potrzebnymi do poprawnego spendu i dowodzenia.

Stan obecny:
- obejmuje `amount`, `witness_seed`, `noise_class`, `bundle_id`.

Finalna zasada:
- `aux_commit` jest czescia semantyki noty,
- wallet musi umiec sprawdzic zgodnosc `aux_opening` z `aux_commit`.

### 5.6. OutputNote

`OutputNote` jest kanonicznym outputem raila `FullPrivacy`.

Semantyka finalna:
- zawiera `note_commit`,
- zawiera `spend_policy_commit`,
- zawiera ukryta kwote jako `ct_amt`,
- zawiera `aux_commit`,
- zawiera pelny `RecipientBox`,
- `note_commit` binduje caly note body.

To jest glowny finalny output prywatny systemu.

### 5.7. Nullifier

`Nullifier` jest kanonicznym markerem zuzycia prawa do wydania noty.

Semantyka finalna:
- jest deterministycznie wyprowadzany z `note_commit` i `nullifier_key`,
- to on daje anti-replay oraz anti-double-spend,
- nie mozna oslabic tej zaleznosci bez nowego freeze.

### 5.8. LiteOutputNote

`LiteOutputNote` istnieje dzis w kodzie jako przejsciowy output dla obecnej implementacji lite raila.

Semantyka stanu obecnego:
- kwota jest jawna,
- `spend_policy_commit` i `aux_commit` pozostaja,
- pelny `RecipientBox` nadal jest przenoszony on-chain,
- `note_commit` binduje `hint`, a nie caly box.

Zamrozona interpretacja:
- ten byt opisuje stan obecnej implementacji,
- ale nie jest jeszcze automatycznie finalna definicja `OnChainLite`,
- finalny `OnChainLite` moze zostac uznany za zamrozony dopiero po spelnieniu acceptance criteria z freeze systemowego.

## 6. Model kluczy walleta

Model `master_seed` pozostaje finalnym kierunkiem rdzenia protokolu.

Docelowa hierarchia:
- `master_seed`
- `spend_root`
- `scan_root`
- `nullifier_root`
- `kem_root`

Semantyka finalna:
- `master_seed` pozwala odtworzyc caly stan walleta,
- `scan_root` moze byc delegowany bez przekazywania spend authority,
- system preferuje jednorazowe klucze pochodne zamiast stalej publicznej tozsamosci,
- bundle i material kluczowy maja wynikac z kontrolowanej hierarchii KDF, a nie z recznie zarzadzanej kolekcji bytow.

Stan obecny:
- deterministic derivation istnieje juz dla roots i Falcon-side bundle identity,
- KEM key path nie jest jeszcze finalnie opisana jako w pelni kanoniczna seeded spec warstwy formatow,
- to pozostaje do zamkniecia w `PRIVAI_CANONICAL_FORMATS.md`.

## 7. Lifecycle bundli i not

### 7.1. Bundle lifecycle

Stan obecny walleta definiuje:
- `Fresh`
- `Offered`
- `Used`
- `Revoked`

Finalna semantyka:
- `Fresh`: bundle gotowy do uzycia lokalnie,
- `Offered`: bundle zostal wystawiony lub przekazany odbiorcy/nadawcy,
- `Used`: bundle zostal skonsumowany przez odebrana note,
- `Revoked`: bundle nie moze byc juz bezpiecznie uzywany, np. przez expiry lub explicit revocation.

### 7.2. Owned note lifecycle

Stan obecny walleta definiuje:
- `Spendable`
- `Locked`
- `Spent`

Finalna semantyka:
- `Spendable`: note moze byc uzyta jako input,
- `Locked`: note jest czasowo wyjeta z obrotu przez lokalny flow walleta,
- `Spent`: note zostala wydana i ma przypisany nullifier.

## 8. Odbior noty przez wallet

Finalna semantyka odbioru jest nastepujaca:
- wallet filtruje outputy po `RecipientBox.hint`,
- wallet otwiera box tylko wtedy, gdy posiada lokalne secret keys,
- wallet musi zweryfikowac:
  - `note_commit`,
  - `note_payload_commit`,
  - zgodnosc `bundle_id` z `hint`,
  - zgodnosc `SpendPolicy`,
  - zgodnosc `AuxWitness`,
  - zgodnosc `nullifier_key` z derivation z KEM shared secret,
- dopiero po tej weryfikacji note moze zostac zapisana jako `Spendable`.

Finalna zasada bezpieczenstwa:
- wallet bez lokalnych secret keys nie moze otworzyc cudzego `RecipientBox`,
- `nullifier_key` niespelniajacy oczekiwanej derivation musi byc odrzucony,
- odbior noty powinien byc operacja atomowa z persystencja stanu.

## 9. Spend noty i budowa transakcji

Semantyka finalna:
- note moze byc spendowana tylko z poprawnym `SpendMaterial`,
- balance constraint pozostaje obowiazkowy,
- input jest reprezentowany przez:
  - `InputRef`,
  - odpowiadajacy mu `input_nullifier`,
  - material autoryzacyjny w `auth`,
- po poprawnym spendzie note przechodzi do stanu `Spent`.

Stan obecny:
- builder `FullPrivacy` jest juz sensownie domkniety dla `TransferNoteTx`,
- builder lite istnieje, ale nie oznacza to jeszcze finalnego zamrozenia `OnChainLite`.

## 10. Kanoniczne klasy transakcji

### 10.1. TransferNoteTx

`TransferNoteTx` jest kanoniczna finalna platnicza transakcja raila `FullPrivacy`.

Semantyka finalna:
- inputy wskazuja zuzywane note,
- outputy tworza nowe `OutputNote`,
- conservation wartosci musi byc zachowana,
- `statement_commit` reprezentuje publiczny commitment do statementu/proofu,
- `auth` reprezentuje warstwe autoryzacji inputow.

### 10.2. OnChainLite final target

Finalny `OnChainLite` jest odrebnym final targetem produktu, ale nie jest jeszcze w pelni zamrozony protokolowo.

Zamrozona semantyka targetu:
- mala kwota moze byc jawna,
- platnik nie moze miec stalej publicznej tozsamosci,
- odbiorca nie moze miec stalej publicznej tozsamosci,
- kolejne male platnosci nie powinny byc prosto linkowalne po jawnym grafie inputow.

Stan obecny:
- kod posiada `LiteTransferTx` i `LiteOutputNote`,
- ta implementacja jest przejsciowym eksperymentalnym mappingiem,
- nie moze byc jeszcze uznana za finalny `OnChainLite`.

### 10.3. MarketplaceBatchTx

`MarketplaceBatchTx` jest kanoniczna transakcja settlementowa dla `MarketplaceSmallPaymentsRail`.

Semantyka finalna:
- sluzy do batch settlementu operatora marketplace,
- niesie `SettlementBatchSummary`,
- niesie liste `ticket_nullifier`,
- jest osobnym swiatem od `OnChainLite`,
- nie wolno interpretowac jej jako zwyklej lekkiej transakcji p2p.

### 10.4. Pozostale tx klasy z kodu

Stan obecny enum `Transaction` zawiera tez:
- `SettlementTx`
- `ModelTx`
- `StakeTx`

Zamrozona interpretacja:
- te byty istnieja w kodzie,
- ale ich finalna kanoniczna rola nie jest jeszcze domknieta przez obecny freeze platnosci,
- pozostaja `provisional`,
- ich obecnosc w enum `Transaction` nie jest sama w sobie freeze decision,
- nie wolno im przypisywac wiekszej finalnosci niz wynika z przyszlych kanonicznych docs.

## 11. Odpowiedzialnosci warstw

### 11.1. Wallet

Wallet odpowiada za:
- zarzadzanie `master_seed` i hierarchia kluczy,
- generowanie i rotacje bundli,
- seal/open `RecipientBox`,
- weryfikacje otwartej noty,
- lokalny lifecycle bundli i owned notes,
- budowe transakcji zgodna z finalna semantyka raila.

### 11.2. Ledger

Ledger odpowiada za:
- shape validation transakcji,
- podstawowe sprawdzenia auth,
- wykrywanie duplicate inputs/nullifiers/outputs,
- tracking note setu i spent nullifiers,
- zastosowanie transakcji do stanu.

### 11.3. Consensus

Consensus odpowiada za:
- walidacje blokow,
- `tx_root`,
- `note_root`,
- `nullifier_root`,
- coverage i poprawny envelope proof-carrying execution,
- egzekwowanie zasad blokow i epoch parametrow.

Granica dokumentow:
- exact block validity, finality rules i consensus enforcement naleza do `spec/PRIVAI_CONSENSUS.md`.

## 12. Current non-conformities wzgledem finalnego protocol core

Najwazniejsze obecne niezgodnosci:
- model amountow pozostaje przejsciowo zwiazany z `Amount14` i nie odpowiada jeszcze finalnemu `PVA + aPVA`,
- lite rail pozostaje eksperymentalny i nie spelnia jeszcze finalnej obietnicy unlinkability,
- `Transaction::outputs()` zwraca pusto dla `LiteTransfer`, przez co lite outputs nie sa dzis traktowane jak zwykle outputs przez caly stack,
- `note_root()` opiera sie tylko o `tx.outputs()`, a wiec nie obejmuje obecnych lite outputow,
- ledger zapisuje note state tylko przez zwykly `outputs()` path,
- nazewnictwo kodu i docs nadal miesza `RecipientPrivacyLite` z finalnym `OnChainLite`,
- exact wire format nie jest jeszcze odseparowany do jednego finalnego kanonicznego pliku.

## 13. Co trzeba zrobic, aby protocol core byl pelnie finalny

- utworzyc `spec/PRIVAI_CANONICAL_FORMATS.md` i przeniesc tam exact bytes, hash domains i golden vectors,
- zamknac finalny amount model `PVA + aPVA`,
- albo dopiac finalny `OnChainLite` end-to-end, albo utrzymac go jawnie jako `experimental`,
- zsynchronizowac ledger i consensus z finalnie wybranym lite path,
- zsynchronizowac stare docs z niniejszym protocol core albo oznaczyc je jako `legacy/historical`,
- usunac resztki dwoch rownoleglych slownikow dla tych samych bytow.

## 14. Twarde zasady dla dalszych zmian

- nie wolno zmieniac semantyki obiektow rdzenia bez update freeze i protocol core,
- nie wolno zmieniac finalnych bytow przez sam kod bez jawnej decyzji spec,
- nie wolno traktowac docs spoza canonical final spec set jako rownorzednego zrodla prawdy,
- nie wolno oslabic podstawowych zasad bezpieczenstwa:
  - jednorazowosci bundli,
  - prywatnosci odbiorcy przez `RecipientBox`,
  - nullifier-based anti-double-spend,
  - hierarchii `master_seed`,
  - rozdzialu `FullPrivacy` od lekkich raili.
