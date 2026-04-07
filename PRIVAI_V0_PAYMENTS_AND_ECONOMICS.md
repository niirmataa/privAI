# privAI v0 Payments and Economics

Status: legacy payments and economics reference in migration.
Canonicality: non-canonical. Ten dokument nie moze nadpisywac finalnych decyzji z `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Owner: privAI product and economics.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA.md` oraz odpowiednie finalne docs kanoniczne.

## 1. Cel

Ten dokument zamraza produktowo-architektoniczny kierunek dla:

- malych pakietow uslug AI,
- depozytow i sald prepaid,
- escrow marketplace,
- ekonomii on-chain versus sidecar proof,
- monitoringu oplacalnosci calego systemu.

To nie jest osobna warstwa od ledgera. To jest opis, jak `privAI v0` ma byc uzywany tak,
zeby prywatnosc i koszty pozostaly sensowne.

## 2. Twarda prawda v0

`privAI v0` jest naturalnie:

- prywatna warstwa settlement,
- prywatny rail escrow,
- warstwa finalizacji i rozrachunku,

a nie:

- rail do mikropatnosci on-chain per pojedyncze zdarzenie.

Powod:

- `ct_amt` jest kosztowny bajtowo,
- `RecipientBox` jest kosztowny bajtowo,
- prywatna nota ma wysoki koszt staly niezaleznie od tego, czy kupujesz duzy pakiet czy bardzo maly.

Najwazniejszy wniosek:

- dla malych pakietow nie robimy `1 zakup = 1 prywatna transakcja on-chain`,
- dla malych pakietow robimy `depozyt -> saldo/tab -> batch settlement`.

## 3. Co jest drogie, a co nie

### 3.1. Co jest drogie on-chain

Najwiekszy koszt on-chain robi:

- `OutputNote`,
- `LweCiphertext`,
- `RecipientBox`,
- fanout outputow i change.

To jest koszt staly i malo zalezy od wartosci samej kwoty.

### 3.2. Co nie jest glownym problemem on-chain

W obecnej architekturze pelny proof nie jest glownym payloadem bloku.

Blok niesie:

- `ExecutionBundle`,
- `ProofCertificate`,

a pelne `proof_bytes` zyja jako sidecar / artifact.

To oznacza:

- koszt proof metadata on-chain jest relatywnie maly,
- proof bytes trzeba monitorowac, ale nie one dzis dominuja rozmiar chaina.

### 3.3. Co jest drogie off-chain

Najwiekszy koszt off-chain to:

- generacja proofu,
- proof sidecar storage/relay,
- settlement orchestration,
- escrow bookkeeping.

## 4. Zamrozony kierunek v0

### DECISION-P1

`privAI v0` jest projektowany jako `deposit-first privacy settlement layer`.

### DECISION-P2

UI i wallet nie powinny promowac bezposredniego modelu:

- `maly pakiet -> osobny prywatny tx on-chain`

jako sciezki domyslnej.

### DECISION-P3

Kazdy produkt/usluga musi wybrac jeden z trzech rails:

- `PrepaidDepositRail`
- `MerchantTabRail`
- `BatchEscrowRail`

### DECISION-P4

Ekonomia systemu musi byc mierzona jako:

- `bytes on-chain`,
- `proof bytes off-chain`,
- `amortized bytes per fulfilled purchase`,
- `settlement latency`,
- `operator complexity`.

Nie wystarczy patrzec tylko na rozmiar pojedynczego proofu.

### DECISION-P5

`FullPrivacy` jest wymagane dla:

- escrow,
- payoutow o wyzszej wrazliwosci,
- settlementow o wyzszej wartosci,
- transakcji, w ktorych sama kwota jest dana wrazliwa.

### DECISION-P6

Dla drobnicy domyslny model to:

- `deposit -> off-chain service state -> batch settlement`

a nie pelna prywatna nota per zakup.

### DECISION-P7

Jesli mala platnosc trafia on-chain, domyslnym lekkim rail'em jest:

- `RecipientPrivacyLite`
- czyli `ukryty adres + jawna kwota`

a nie oslabianie warstwy PQ.

## 5. Trzy rails platnosci

## 5.1. PrepaidDepositRail

To jest domyslny rail dla malych pakietow.

### Flow

1. Uzytkownik robi prywatny depozyt on-chain do swojego walleta lub do wybranego namespace uslugi.
2. Wallet tworzy lokalne albo relay-backed saldo prepaid.
3. Kazdy maly zakup schodzi z tego salda poza chainem albo w warstwie lekkiego state tracking.
4. Chain widzi tylko:
   - top-up,
   - okresowe settlement,
   - refund / withdrawal.

### Zalety

- bardzo dobra amortyzacja kosztu not,
- dobra prywatnosc przy wielu malych zakupach,
- prosty UX po pierwszym depozycie.

### Ryzyka

- trzeba dobrze zaprojektowac lokalny stan salda,
- trzeba rozwiazac odzyskiwanie stanu po utracie klienta,
- trzeba jasno rozdzielic `funds custody` od `service entitlement`.

### Rekomendacja v0

To powinien byc domyslny rail dla retail / micro-packages.

## 5.2. MerchantTabRail

To jest rail sesyjny / relacyjny.

### Flow

1. Uzytkownik otwiera prywatny tab u merchanta.
2. Na chainie powstaje jedna nota otwierajaca finansowanie sesji lub limit wydatkow.
3. Kolejne male zakupy sa rejestrowane off-chain jako eventy:
   - usage receipt,
   - service receipt,
   - quota decrement,
   - memoized balance delta.
4. Po oknie czasu albo po osiagnieciu progu robi sie jedno settlement.

### Zalety

- bardzo dobry UX dla wielu malych interakcji z jednym sprzedawca,
- dobre dopasowanie do modelu marketplace i subskrypcji,
- naturalne miejsce na service-level policy.

### Ryzyka

- merchant tab jest stanowy, wiec trzeba dobrze zamknac:
   - timeout,
   - recovery,
   - dispute path,
   - cap na ekspozycje.

### Rekomendacja v0

To powinien byc rail dla:

- marketplace sessions,
- inference credits,
- compute tabs,
- pre-autoryzowanych zakupow seryjnych.

## 5.3. BatchEscrowRail

To jest rail dla uslug, gdzie potrzeba silniejszej logiki rozliczenia.

### Flow

1. Kupujacy blokuje srodki w nocie z polityka marketplace/escrow.
2. Sprzedawca realizuje usluge i produkuje off-chain receipts.
3. Receipts sa batchowane w oknie settlement.
4. Koniec stanu to:
   - `Release`,
   - `Refund`,
   - `Dispute`,
   - ewentualnie `PartialRelease`.

### Zalety

- logicznie pasuje do marketplace i sporow,
- pozwala laczyc prywatnosc z arbitrazem,
- dobrze wspiera drozsze lub wieloetapowe uslugi.

### Ryzyka

- najwyzsza zlozonosc orchestracji,
- duzo stanow i timeoutow do pilnowania,
- trzeba bardzo uwaznie modelowac receipts i payload commitments.

### Rekomendacja v0

To powinien byc rail dla:

- settlement marketplace,
- job escrow,
- moderowanych platnosci,
- zamowien o wyzszej wartosci lub wyzszym ryzyku.

## 5.4. DirectSettlement

Ten rail istnieje, ale nie powinien byc domyslny.

### Kiedy ma sens

- duza lub rzadka platnosc,
- payout,
- top-up,
- refund koncowy,
- pojedynczy settlement o sensownej wartosci.

### Kiedy nie ma sensu

- drobny zakup detaliczny,
- pojedynczy inference call,
- kazde klikniecie w UI,
- wielokrotne male eventy u jednego sprzedawcy.

### Zasada v0

`DirectSettlement` jest fallbackiem i sciezka power-user / operator,
nie podstawowa sciezka retail.

## 5.5. Punkty uslugi, ktore trzeba zaprojektowac od razu

Nie wystarczy powiedziec, ze system ma `deposit`, `payment` i `escrow`.
Trzeba zaprojektowac pelny lifecycle uslugi.

Kazdy produkt w marketplace powinien miec jawnie opisane:

- jak uzytkownik finansuje zakup,
- gdzie powstaje zobowiazanie uslugowe,
- kiedy srodki przechodza z `reserved` do `earned`,
- kiedy mozliwy jest `refund`,
- kiedy mozliwy jest `dispute`,
- jaki event zamyka usluge i settlement.

### 5.5.1. Punkty lifecycle

Minimalny lifecycle uslugi:

1. `Discover`
2. `Fund`
3. `Authorize`
4. `Reserve`
5. `Deliver`
6. `Accept or Timeout`
7. `Release / Refund / Dispute`
8. `Consolidate`
9. `Withdraw / Rebalance`

### 5.5.2. Co jest on-chain, a co nie

On-chain powinno istniec tylko to, co naprawde musi byc trwałe i rozstrzygalne:

- depozyt,
- otwarcie i zamkniecie escrow,
- finalny settlement,
- refund,
- payout,
- nullifiers i nowe outputy.

Off-chain powinno zyc to, co jest szybkie, drobne i czesto aktualizowane:

- usage receipts,
- quota consumption,
- merchant tab balance deltas,
- service session logs,
- intermediate accept/progress events.

### 5.5.3. Minimalny kontrakt uslugi

Kazdy listing / produkt powinien deklarowac:

- `payment_rail`
- `pricing_mode`
- `deposit_minimum`
- `reservation_mode`
- `settlement_window`
- `acceptance_rule`
- `refund_rule`
- `dispute_rule`
- `expiry_rule`
- `operator_visibility`

`pricing_mode` powinien rozroznic co najmniej:

- `FixedPrice`
- `UsageMetered`
- `TieredPackage`
- `EscrowMilestone`

`reservation_mode` powinien rozroznic co najmniej:

- `ImmediateDebit`
- `PreauthorizedHold`
- `EscrowLock`

### 5.5.4. Role systemowe

Minimalne role w v0:

- `buyer`
- `seller`
- `marketplace_operator`
- `arbiter`
- `relay/provider`

### 5.5.5. Receipts jako first-class objects

Zanim dojdziemy do pelnego ZK settlementu dla marketplace,
musimy jawnie modelowac receipts.

Minimalny receipt powinien committowac:

- `service_id`
- `settlement_id`
- `buyer side reference`
- `seller side reference`
- `usage amount or milestone`
- `unit price or price tier`
- `receipt time window`
- `result hash / payload hash`
- `operator policy context`

Settlement powinien widziec na chainie tylko `receipt_root` albo `payload_commit`,
a nie pelna tresc kazdego receiptu.

## 6. Escrow jako stan, nie tylko typ tx

Escrow w v0 nie powinno byc rozumiane jako pojedyncza transakcja.
To powinien byc maly state machine.

### Stany logiczne

- `Funded`
- `Reserved`
- `Delivering`
- `Releasable`
- `PartiallyReleased`
- `Disputed`
- `Refundable`
- `Settled`

### Minimalne osie danych

- `settlement_id`
- `marketplace_context`
- `buyer note / input set`
- `seller destination note`
- `moderator policy`
- `timeout_block`
- `service receipts root`
- `payload_commit`

### Co musi byc stale monitorowane

- ile srodkow jest zamknietych w otwartych escrow,
- sredni czas do release,
- liczba refundow,
- liczba sporow,
- jaki procent mikro-zakupow laduje niepotrzebnie w escrow.

### 6.1. Escrow state machine v0

Praktycznie chcemy miec jeden maly automat:

```text
Funded
  -> Reserved
  -> Delivering
  -> Releasable
  -> Settled

Reserved
  -> Refundable
  -> Disputed

Delivering
  -> Releasable
  -> Disputed
  -> Refundable

Releasable
  -> PartiallyReleased
  -> Settled
  -> Disputed

PartiallyReleased
  -> Settled
  -> Disputed

Disputed
  -> Settled
  -> Refundable
```

### 6.2. Mapa na fazy settlement tx

Obecny ledger ma juz dobre minimum:

- `Open`
- `Accept`
- `Release`
- `Refund`
- `Dispute`

Mapowanie robocze:

- `Open`
  - finansuje i otwiera settlement context / escrow.
- `Accept`
  - oznacza akceptacje wyniku albo potwierdzenie receipt root.
- `Release`
  - wypuszcza srodki do sprzedawcy.
- `Refund`
  - zwraca srodki kupujacemu.
- `Dispute`
  - zamraza automat i przekazuje decyzje do polityki arbitra.

### 6.3. Co trzeba doprecyzowac w escrow od razu

To sa rzeczy, ktorych nie wolno zostawic na koniec:

- czy escrow wspiera `full release` i `partial release`,
- czy `accept` jest jawne czy moze byc domniemane po timeout,
- kto moze uruchomic `refund`,
- czy `dispute` moze byc otwarte po czesciowym release,
- jak wyglada `timeout_block` dla jobow dlugich,
- czy marketplace operator jest tylko routujacy, czy ma role arbitra,
- jak receipt root laczy sie z payload commit i service result hash,
- czy settlement jest per-job, per-session czy per-batch.

## 7. Monitoring ekonomii

## 7.1. Mierniki on-chain

Dla kazdej wersji klienta i kazdego typu flow trzeba mierzyc:

- `output_note_bytes`
- `tx_bytes`
- `execution_bundle_bytes`
- `proof_certificate_bytes`
- `proof_metadata_bytes_per_tx`
- `outputs_per_tx`
- `change_outputs_per_tx`
- `block_occupancy_percent`

## 7.2. Mierniki sidecar / prover

- `proof_bytes_len`
- `proof_bytes_per_covered_tx`
- `prove_time_ms`
- `verify_time_ms`
- `artifact_count_per_block`
- `covered_tx_per_artifact`

## 7.3. Mierniki biznesowe

- `gross_purchase_value`
- `amortized_onchain_bytes_per_purchase`
- `amortized_sidecar_bytes_per_purchase`
- `purchases_per_deposit`
- `purchases_per_tab`
- `settlement_frequency`
- `escrow_open_to_release_latency`

## 7.4. Miernik decyzyjny

Najwazniejsza metryka v0:

```text
amortized_total_cost_per_purchase
  = (onchain_bytes + sidecar_bytes + proving_cost + operator_cost)
    / purchases_settled
```

Jesli ta wartosc jest zbyt wysoka dla malego pakietu, to ten produkt nie powinien isc direct on-chain.

## 8. Zasady produktowe dla malych pakietow

### Zasada 1

Maly pakiet nie powinien domyslnie otwierac nowej prywatnej noty settlementowej per zakup.

### Zasada 2

Kazdy katalog uslug powinien jawnie deklarowac:

- `rail`
- `settlement window`
- `deposit minimum`
- `escrow required or not`

### Zasada 3

Bezposredni private transfer on-chain powinien byc zarezerwowany dla:

- top-up,
- payout,
- wiekszego settlementu,
- escrow open / resolve,
- refund,

a nie dla pojedynczego drobnego klikniecia uzytkownika.

### Zasada 4

Dla najtanszych zakupow mozna rozwazyc tryb:

- `hidden recipient`
- `public amount`

ale tylko jako jawnie oznaczony tryb kompromisowy, nie jako zamiennik pelnej prywatnosci kwoty.

## 8.1. Low-cost mode: ukryty adres, jawna kwota

To jest bardzo sensowna opcja dla drobnych pakietow, jesli:

- cena i tak jest praktycznie publiczna,
- najwazniejsza jest prywatnosc odbiorcy i relacji kto-komu-placi,
- chcemy wyraznie zredukowac koszt proof layer dla amount privacy.

### Co zyskujemy

- wyrzucamy `ct_amt` i jego well-formedness / noise proofs,
- proof staje sie duzo prostszy,
- rozmiar noty spada o koszt ciphertextu LWE,
- settlement dla malych kwot staje sie bardziej realistyczny.

### Czego to nie rozwiazuje

- `RecipientBox` dalej pozostaje duzym kosztem,
- kwota staje sie jawna,
- analiza po kwocie i czasie robi sie prostsza,
- nie jest to dobre dla drozszych lub bardziej wrazliwych transakcji.

### Wniosek praktyczny

Tryb `hidden recipient / public amount` moze byc bardzo dobrym dodatkiem do:

- `PrepaidDepositRail`
- `MerchantTabRail`

ale nie powinien zastapic pelnej prywatnosci w:

- escrow wysokiego ryzyka,
- settlementach o wyzszej wartosci,
- payoutach, gdzie kwota sama w sobie jest wrazliwa.

### Jak to uczciwie pokazac produktowo

Wallet i UI powinny rozroznic co najmniej:

- `FullPrivacy`
  - ukryty adres + ukryta kwota
- `RecipientPrivacyLite`
  - ukryty adres + jawna kwota

To musi byc jasne dla uzytkownika.

### 8.2. Twardy podzial po klasie transakcji

Najzdrowszy podzial v0:

- `small payments`
  - depozyt, tab albo `RecipientPrivacyLite`
- `standard marketplace settlement`
  - zalezy od wrazliwosci kwoty i modelu uslugi
- `large value / escrow / dispute-sensitive`
  - zawsze `FullPrivacy`

To powinno byc jawnie udokumentowane w polityce uslugi, a nie zostawione jako ukryta heurystyka walleta.

### 8.3. Opaque IDs dla drobnicy

To jest bardzo dobry kierunek dla malych zakupow.
Jesli platnosc nie jest finalizowana jako osobny pelny prywatny tx, to off-chain state powinien operowac na:

- jednorazowych `purchase_id`
- jednorazowych `tab_entry_id`
- `settlement_id`
- `receipt_id`

Te identyfikatory musza byc:

- losowe albo deterministycznie wyprowadzane z tajnego kontekstu,
- jednorazowe,
- niepowiazywalne z publicznym `AccountID`,
- nieuzywane ponownie miedzy sesjami i merchantami.

### 8.4. Czego chain ma nie widziec przy drobnicy

Przy modelu depozyt + off-chain state chain nie powinien widziec:

- kazdego pojedynczego zakupu,
- kazdego session eventu,
- kazdego usage increment,
- stalego publicznego identyfikatora kupujacego.

Chain powinien widziec tylko:

- finansowanie,
- batch settlement,
- refund,
- payout,
- ewentualnie otwarcie i zamkniecie escrow.

### 8.5. Produktowa zasada ochrony kwoty

Jawnie warto zapisac:

- mala kwota nie zawsze wymaga ukrycia kwoty,
- duza kwota bardzo czesto wymaga ukrycia kwoty,
- escrow powinno domyslnie zakladac `FullPrivacy`,
- `RecipientPrivacyLite` to ekonomiczny rail dla drobnicy, a nie tryb dla sporow lub wysokiej wartosci.

## 9. Burza mozgow: optymalizacje not

To nie wszystko nadaje sie do v0, ale warto zamrozic katalog mozliwych ruchow.

### 9.1. Credit note zamiast purchase note

Zamiast tworzyc note per zakup:

- tworzymy jedna note kredytowa,
- zakupy zuzywaja credits off-chain,
- chain widzi tylko top-up i settlement.

To jest najwazniejsza optymalizacja produktowa.

### 9.2. Merchant tab note

Jedna nota otwiera limit lub sesje u sprzedawcy.
Pozwala to unikac on-chain change przy kazdym zakupie.

### 9.3. Change minimization

Wallet powinien aktywnie minimalizowac tworzenie change outputs, bo to podnosi:

- bajty,
- liczbe outputow,
- pozniejsza prace proof layer.

### 9.4. Note class split

Warto rozroznic semantycznie:

- `deposit notes`
- `service tab notes`
- `escrow notes`
- `payout notes`

Nie wszystkie noty musza byc traktowane tak samo przez UX i wallet selection.

### 9.5. Batch recipient settlement

Wiele malych service receipts moze byc settlementowane do jednego outputu albo malej liczby outputow.

### 9.6. RecipientBox optimization backlog

`RecipientBox` pozostaje jednym z glownych kosztow.
Backlog v1/v2 powinien zawierac:

- kompaktowanie boxa,
- lepsza reprezentacje route / hint,
- ewentualna deduplikacje lub ref-style delivery,

ale bez psucia samowystarczalnosci noty.

### 9.7. Proof amortization

Nie pytamy tylko:

- `jak duzy jest proof`

ale:

- `ile tx pokrywa proof`
- `ile purchase events amortyzuje settlement`

### 9.8. Change suppression windows

Wallet nie powinien automatycznie generowac change note po kazdym drobnym zuzyciu.
Lepszy jest model:

- utrzymuj lokalny bucket zuzyc,
- wypusc change dopiero przy settlement window albo przy progu wartosci.

### 9.9. Deposit partitioning

Zamiast jednej wielkiej noty i ciaglego rozbijania:

- tworz pule not o klasach:
  - `retail-small`
  - `session-medium`
  - `escrow-large`

To zmniejsza ryzyko ciaglego, nieefektywnego change fanout.

### 9.10. Rail-aware note selection

Wallet selection nie moze byc slepa.
Powinien preferowac:

- do `DepositRail` noty retailowe,
- do `MerchantTabRail` noty sesyjne,
- do `BatchEscrowRail` noty o wiekszej pojemnosci i mniejszym ryzyku konfliktu z timeout.

### 9.11. Receipt-root settlement zamiast output-per-event

Najwieksza optymalizacja dla malych pakietow to nie tyle mniejsza nota,
co brak noty per event.

Zamiast tego:

- wiele drobnych eventow produkuje jeden `receipt_root`,
- settlement wypuszcza jedna lub mala liczbe not koncowych.

### 9.12. RecipientBox profile classes

Nie kazdy output potrzebuje tak samo bogatego boxa.
Mozna od razu przewidziec profile:

- `retail_box`
- `tab_box`
- `escrow_box`
- `payout_box`

Nawet jesli v0 trzyma jeden format, backlog optymalizacyjny powinien myslec klasami.

### 9.13. Bundle lifecycle hygiene

Jednorazowe bundle musza byc rotowane i gaszone tak, aby:

- nie marnowac puli odbiorczej,
- nie dublowac kosztow box delivery,
- nie trzymac zbyt dlugo martwych prekeys.

### 9.14. Operator-side batching

Marketplace albo relay moze pomagac w:

- grupowaniu settlementow,
- oknach czasowych release,
- laczeniu refundow,
- laczeniu payoutow,

ale bez przejmowania custodian roli ponad to, co jawnie deklaruje rail.

## 10. Co powinno wejsc do implementacji od razu

### P1. Size reporting

Repo powinno dawac prosty raport:

- kanoniczny rozmiar `OutputNote`,
- kanoniczny rozmiar typowego `TransferNoteTx`,
- kanoniczny rozmiar `ExecutionBundle`,
- kanoniczny rozmiar `ProofCertificate`.

### P2. Rail selection

Warstwa wallet / marketplace powinna miec jawny enum:

```text
PaymentRail =
  PrepaidDeposit
  MerchantTab
  BatchEscrow
  DirectSettlement
```

`DirectSettlement` nie powinno byc domyslne dla malych pakietow.

### P3. Settlement policy config

Kazda usluga powinna deklarowac:

- `settlement_window`
- `deposit_minimum`
- `escrow_policy`
- `refund_policy`
- `dispute_policy`

### P4. Economics gate

Przed rolloutem produktowym trzeba umiec odpowiedziec:

- ile kosztuje 1 zakup przy `N=1`,
- ile kosztuje 1 zakup przy depozycie i `N=10`,
- ile kosztuje 1 zakup przy tab i `N=50`,
- ile kosztuje 1 zakup przy escrow batch i `N=K`.

### P5. Service policy schema

Repo powinno dostac jawny model konfiguracyjny dla uslugi:

```text
ServicePaymentPolicy {
  rail,
  pricing_mode,
  deposit_minimum,
  reservation_mode,
  settlement_window,
  acceptance_rule,
  refund_rule,
  dispute_rule,
  timeout_rule,
  batching_rule,
}
```

### P6. Escrow state schema

Repo powinno dostac jawny model runtime dla escrow:

```text
EscrowState {
  settlement_id,
  phase,
  reserved_amount,
  releasable_amount,
  refunded_amount,
  receipt_root,
  payload_commit,
  timeout_block,
}
```

## 11. Rekomendacja robocza

Najzdrowsza strategia v0:

- `DepositRail` jako domyslny retail path,
- `MerchantTabRail` dla wielokrotnego zuzycia u jednego sprzedawcy,
- `BatchEscrowRail` dla marketplace i rozstrzygania sporow,
- `DirectSettlement` tylko dla wiekszych albo rzadszych transakcji.

To pozwala zachowac:

- sensowna prywatnosc,
- sensowny UX,
- sensowna oplacalnosc on-chain,
- oraz miejsce na pozniejsze optymalizacje proof i note layoutu.
