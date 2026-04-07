# privAI Marketplace Small Payments

Status: draft canonical marketplace small-payments doc in migration.
Canonicality: intended canonical source of truth for `MarketplaceSmallPaymentsRail`. Product-wide privacy and escalation policy remain governed by `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`. Bit-level encoding remains governed by `spec/PRIVAI_CANONICAL_FORMATS.md`. Pliki poza `spec/` nie sa normatywnym source of truth.
Owner: privAI marketplace payments.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`, `spec/PRIVAI_PROTOCOL_CORE.md`, `spec/PRIVAI_CANONICAL_FORMATS.md`.
Supersedes: `spec/marketplace_small_payments_v0/README.md`, `spec/marketplace_small_payments_v0/00_PREIMPLEMENTATION_READINESS.md`, `spec/marketplace_small_payments_v0/01_SMALL_PAYMENTS_RAIL_V0.md`, `spec/marketplace_small_payments_v0/02_SERVICE_PAYMENT_POLICY.md`, `spec/marketplace_small_payments_v0/03_TICKET_ID_AND_NULLIFIER.md`, `spec/marketplace_small_payments_v0/04_RECEIPT_AND_SETTLEMENT_ROOT.md`, `spec/marketplace_small_payments_v0/05_PRIVACY_TIERS.md`, `spec/marketplace_small_payments_v0/06_IMPLEMENTATION_ROADMAP.md` na poziomie finalnej semantyki marketplace raila.

## 1. Cel

Ten dokument opisuje finalny model `MarketplaceSmallPaymentsRail` w systemie `privAI`.

Ma on odpowiedziec jasno:
- do czego ten rail sluzy,
- do czego nie sluzy,
- jakie byty sa kanoniczne,
- jaka jest rola walleta, merchanta, operatora i consensus,
- jakie sa trust assumptions,
- co jest juz semantycznie zamrozone,
- a co nadal wymaga domkniecia implementacyjnego.

To nie jest dokument dla zwyklego `OnChainLite`.
To nie jest tez dokument dla `FullPrivacy`.

## 1.1. How Devs And Agents Should Use This Doc

Ten dokument jest zrodlem prawdy dla `MarketplaceSmallPaymentsRail`.

Czytaj go razem z:
- `spec/PRIVAI_SPEC_INDEX.md` jako punktem wejscia do calego frozen setu,
- `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md` dla miejsca raila w calym systemie,
- `spec/PRIVAI_CANONICAL_FORMATS.md` dla policy/grant/receipt/summary bytes,
- `spec/PRIVAI_CONSENSUS.md` dla on-chain batch enforcement.

Zamrozona zasada pracy:
- jesli task dotyczy grantow, receiptow, ticketow, settlementu albo trust assumptions operatora, to ten dokument jest punktem wyjscia,
- nie wolno traktowac tego raila jako "po prostu lekkiego transferu",
- nie wolno dopowiadac logiki finansowej poza `policy_commit`, `grant_commit`, `receipt_commit` i `settlement_root`.
- stare docs marketplace i kod sa tylko referencja implementacyjna albo historyczna; nie wolno z nich wyprowadzac nowej semantyki obok tego dokumentu.

Jesli kod nie zgadza sie z tym dokumentem:
- nie wolno po cichu rozluzniac operator authority,
- nie wolno po cichu usuwac `purchase_commit`,
- nie wolno po cichu mieszac `ticket`, `session`, `receipt` i `settlement`.

Jawnego freeze update wymagaja zawsze:
- zmiana trust modelu operatora,
- zmiana authority modelu grantu lub batcha,
- zmiana semantyki `ticket_nullifier`,
- zmiana refund/timeout/dispute modelu,
- zmiana relacji marketplace raila do `OnChainLite` albo `FullPrivacy`.

Interpretation rule for this document:
- status labels sa zdefiniowane w `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`,
- jesli marketplace rule nie ma jawnej etykiety statusu albo jawnej reguly commitment/auth, nalezy traktowac ja jako `unresolved`,
- `unresolved` nie wolno domykac przez lokalne domysly merchanta, operatora, walleta ani agenta AI.

## 2. Miejsce tego raila w systemie

`MarketplaceSmallPaymentsRail` jest jednym z trzech kanonicznych torow systemu:
- `FullPrivacy`
- `OnChainLite`
- `MarketplaceSmallPaymentsRail`

Zamrozona zasada:
- `MarketplaceSmallPaymentsRail` jest `marketplace-only`,
- nie jest odmiana `OnChainLite`,
- nie jest zamiennikiem `FullPrivacy`,
- nie wolno opisywac go jako "lite p2p rail".

Rola tego raila:
- obsluga malych, szybkich i usage-metered rozliczen w marketplace,
- amortyzacja kosztu przez grants, receipts i batch settlement,
- brak stalego publicznego konta platnika,
- jawne i uczciwe trust assumptions operator-trusted accounting dla v0/finalnego modelu tego raila.

## 3. Fundamentalne zalozenia

Ponizsze zalozenia sa dla tego raila zamrozone:
- rail jest `marketplace-only`,
- rail zaczyna sie od prywatnego depozytu lub prywatnego funding path,
- rail sluzy drobnicy i usage-metered flows, nie duzym settlementom,
- `purchase_commit` pozostaje wymagany,
- `ticket` jest `strictly one-time`,
- `ticket` i `session/tab` to dwa rozne byty,
- marketplace operator jest domyslna authority dla grantow i settlementu,
- merchant nie jest samodzielna authority od praw z depozytu,
- `FullPrivacy` pozostaje wymagane dla escrow, flow wrazliwych i wyzszych kwot,
- nie wolno oslabiac warstwy PQ dla tego raila.

## 4. Czego ten dokument nie robi

Ten dokument nie robi nastepujacych rzeczy:
- nie redefiniuje `FullPrivacy`,
- nie definiuje finalnego `OnChainLite`,
- nie usuwa `purchase_commit`,
- nie wraca do modelu, w ktorym merchant sam interpretuje prawa z depozytu,
- nie ukrywa trust assumptions pod haslami marketingowymi,
- nie projektuje slabszego PQ stacku dla drobnicy.

## 5. Finalny model produktu

Finalny model `MarketplaceSmallPaymentsRail` jest nastepujacy:
- user ma prywatny funding path,
- marketplace operator wystawia scoped `SpendGrant`,
- wallet generuje jednorazowe lokalne tickety z `rail_seed`,
- merchant tworzy `Receipt` dla wykonanego charge lub usage eventu,
- operator agreguje receipts w settlement batch,
- on-chain trafia `MarketplaceBatchTx`,
- consensus zuzywa `ticket_nullifier` i waliduje batch na poziomie zakresu v0/finalnego modelu tego raila.

To jest rail:
- z audytowalnym batch settlementem,
- z jawna lista `ticket_nullifier` on-chain,
- z trusted accounting po stronie operatora,
- bez pelnego on-chain trust-minimized powiazania kazdego debitu z funding anchor.

## 6. Obietnica prywatnosci i granice prywatnosci

`MarketplaceSmallPaymentsRail` ma nastepujaca obietnice prywatnosci:
- nie ujawnia stalego publicznego konta platnika,
- nie ujawnia stalego publicznego rail ID dla usera,
- oddziela usage/session layer od publicznego chain state,
- publikuje settlement zagregowany przez operatora,
- nie obiecuje tej samej prywatnosci co `FullPrivacy`.

Zamrozona uczciwa interpretacja:
- ten rail ma inne trust assumptions niz `OnChainLite`,
- ten rail ma inne trust assumptions niz `FullPrivacy`,
- operator widzi wiecej niz chain,
- chain nie odtwarza calej semantyki uslugi ani calego usage logu,
- prywatnosc wynika z kombinacji scoped grants, lokalnych ticketow, receipt layer i batch settlementu, a nie z pelnego trust-minimized ukrycia wszystkiego na chainie.

## 7. Kanoniczne byty raila

### 7.1. ServicePaymentPolicy

`ServicePaymentPolicy` jest jawnym kontraktem polityki platnosci dla uslugi marketplace.

Semantyka finalna:
- policy nie moze byc ukryta w kodzie merchanta,
- wallet musi umiec ja odczytac i zastosowac,
- operator musi umiec ja egzekwowac,
- policy okresla:
  - scope merchanta/uslugi,
  - dozwolony rail,
  - pricing mode,
  - minimalny depozyt,
  - limity sesji i okna,
  - expiry rule,
  - settlement window,
  - prog wymuszajacy `FullPrivacy`.

Zamrozona zasada policy completeness:
- wszystko, co wplywa na pieniadz, spor, timeout, refund, batching i eskalacje do `FullPrivacy`, musi byc bindowane przez `policy_commit`,
- reguly te maja byc modelowane jako `tag + params`,
- nie wolno przenosic takich regul do niejawnej logiki merchanta albo operatora poza committed policy.

### 7.2. SpendGrant

`SpendGrant` jest scoped autoryzacja wystawiana przez marketplace operatora.

Semantyka finalna:
- grant wiaze sesje z prawem skorzystania z depozytu,
- grant ma twardy `spend_cap`,
- grant ma `grant_expiry`,
- grant ma `settlement_window`,
- grant jest scoped do `merchant_commit`,
- moze byc dodatkowo scoped do `service_commit`,
- `policy_commit` musi byc czescia grantu.

Zamrozona zasada:
- brak nieograniczonych grantow,
- brak niejawnych regul po stronie merchanta,
- `SpendGrant` zawsze wymaga operator authority i operator signature.

### 7.3. RailContext i rail_seed

`RailContext` i `rail_seed` sa lokalnymi bytami walleta.

Semantyka finalna:
- `rail_seed` nie trafia on-chain,
- `rail_seed` jest uzywany do deterministycznej generacji lokalnych ticketow,
- wallet utrzymuje pule ticketow per `merchant_commit`,
- recovery musi byc mozliwe z wallet state/master seed path, a nie tylko z pamieci procesu.

### 7.4. LocalTicket

`LocalTicket` jest lokalnym jednorazowym prawem uzywanym w railu marketplace.

Stan obecny kodu:
- `ticket_id`
- `ticket_nullifier`
- `ticket_auth`
- `is_used`

Semantyka finalna:
- `ticket_id` sluzy lokalnie i off-chain,
- `ticket_nullifier` jest glownym publicznym markerem jednorazowosci,
- `ticket_auth` jest lokalnym materialem autoryzacyjnym dla purchase context,
- ticket jest merchant-scoped przez pule generacyjna,
- ticket nie jest rosnacym licznikiem usage ani dlugowiecznym kontenerem calej sesji.

### 7.5. TicketId

`ticket_id`:
- nie jest glownym on-chain replay guard,
- sluzy do lokalnego i operatorskiego powiazania wydarzenia z receiptem i stanem walleta.

### 7.6. TicketNullifier

`ticket_nullifier`:
- jest glownym publicznym markerem replay protection i double-spend protection dla tego raila,
- jest publikowany on-chain w `MarketplaceBatchTx`,
- musi byc globalnie unikalny,
- nie powinien ujawniac stalej tozsamosci platnika.

### 7.7. Session / Tab

`Session` lub `tab` to oddzielny byt od ticketu.

Semantyka finalna:
- session/tab sluzy do usage tracking i lokalnej/merchantskiej semantyki uslugi,
- ticket sluzy do jednorazowego prawa debitu,
- settlement sluzy do batchowego anchorowania stanu na chainie,
- nie wolno laczyc tych trzech semantyk w jednym stalym identyfikatorze.

### 7.8. Receipt

`Receipt` jest kanonicznym obiektem settlementowym potwierdzajacym naliczenie charge lub wykonanie uslugi w ramach raila.

Semantyka finalna:
- receipt nie jest luznym logiem,
- receipt jest formalnym obiektem do:
  - batching,
  - audytu,
  - refund path,
  - dispute review,
- receipt zawiera co najmniej:
  - scope merchant/service/session,
  - `grant_commit`,
  - `purchase_commit`,
  - `ticket_nullifier`,
  - `amount`,
  - `policy_commit`,
  - `result_commit`,
  - `issued_at`,
  - signature material merchanta poza commitmentem.

### 7.9. SettlementBatchSummary

`SettlementBatchSummary` jest kanonicznym streszczeniem settlement batcha.

Semantyka finalna:
- summary zawiera scope operator/merchant/grant,
- summary zawiera `receipt_root`,
- summary zawiera liczniki receiptow i nullifierow,
- summary zawiera sumaryczne amount/fee/refund,
- summary daje `settlement_root`.

### 7.10. MarketplaceBatchTx

`MarketplaceBatchTx` jest kanoniczna on-chain transakcja settlementowa raila marketplace.

Semantyka finalna:
- publikuje settlement batch operatora,
- niesie `SettlementBatchSummary`,
- niesie jawna liste `ticket_nullifier`,
- niesie auth/signature operatora,
- sluzy do spalenia ticket nullifierow i zapisania settlement anchorow na chainie.

Zamrozona zasada:
- `MarketplaceBatchTx` zawsze wymaga operator authority,
- merchant moze podpisywac `Receipt`, ale nie jest domyslna settlement authority dla batcha.

## 8. Finalny flow raila

### 8.1. Funding i wejscie do raila

User wchodzi do raila przez prywatny funding path.

Zamrozona zasada:
- rail nie zaczyna sie od stalego publicznego konta usera,
- rail nie jest modelem "merchant sam sciaga z publicznego konta".

### 8.2. Policy read i grant issuance

Wallet odczytuje `ServicePaymentPolicy`.
Operator wystawia `SpendGrant`.

Wallet musi umiec sprawdzic:
- czy rail jest marketplace-only,
- czy limity sa akceptowalne,
- czy dany flow nie wymaga juz `FullPrivacy`,
- czy grant jest scoped do oczekiwanego merchanta/uslugi.

### 8.3. Ticket generation

Wallet generuje lokalne tickety z `rail_seed`.

Zamrozona zasada:
- ticketi sa deterministycznie odtwarzalne z wallet context,
- co najmniej merchant-scoped,
- jednorazowe.

### 8.4. Usage i receipt creation

Merchant tworzy lub wspoltworzy `Receipt` po wykonaniu uslugi albo naliczeniu charge.

Zamrozona zasada:
- `purchase_commit` pozostaje wymagany,
- sam `amount + ticket_nullifier` to za malo dla finalnego modelu audit/refund/dispute.

### 8.5. Operator batching

Operator:
- zbiera receipts,
- deduplikuje `ticket_nullifier` w batchu,
- liczy `receipt_root`,
- liczy `settlement_root`,
- tworzy `MarketplaceBatchTx`.

Rozdzial semantyczny:
- `receipt_root` commitmentuje tresc receipts w batchu,
- `settlement_root` commitmentuje summary batcha,
- te dwa rooty nie sa tym samym i nie wolno ich utozsamiac.

### 8.6. On-chain settlement

Consensus:
- weryfikuje format batcha,
- sprawdza authority/signature operatora jako warunek obowiazkowy modelu finalnego,
- sprawdza brak duplikatow `ticket_nullifier` wewnatrz batcha,
- sprawdza brak wczesniejszego globalnego zuzycia tych nullifierow,
- oznacza `ticket_nullifier` jako spent.

## 9. Trust assumptions

Zamrozona uczciwa definicja v0/finalnego modelu tego raila:
- to jest `marketplace-operator-trusted accounting`,
- operator jest domyslna authority dla grantow i settlementu,
- merchant nie jest samodzielna settlement authority,
- chain nie ma pelnego kryptograficznego dowodu powiazania kazdego `ticket_nullifier` z funding anchor.

To oznacza:
- operator moze zobaczyc i zinterpretowac wiecej niz consensus,
- correctness settlementu na poziomie pelnej ekonomicznej semantyki nie jest w 100% trust-minimized,
- to jest jawny tradeoff, a nie ukryte zalozenie.

## 10. Role i odpowiedzialnosci

### 10.1. Wallet

Wallet odpowiada za:
- odczyt `ServicePaymentPolicy`,
- decyzje czy rail marketplace jest dopuszczalny,
- odmowe wejscia w rail, jesli flow wymaga `FullPrivacy`,
- zarzadzanie `RailContext`,
- generacje ticketow,
- lokalne powiazanie purchase context i wallet state.

### 10.2. Merchant

Merchant odpowiada za:
- respektowanie polityki uslugi,
- tworzenie lub wspoltworzenie `Receipt`,
- nieprzyjmowanie dowolnych albo replayowanych debitow poza modelem raila.

### 10.3. Marketplace operator

Operator odpowiada za:
- wystawianie `SpendGrant`,
- intake `Receipt`,
- deduplikacje ticket nullifierow w batchu,
- tworzenie `SettlementBatchSummary`,
- publikacje `MarketplaceBatchTx`,
- refund/timeout/dispute path w granicach polityki raila.

Zamrozona interpretacja:
- operator jest domyslna authority dla grantow i settlementu,
- merchant nie interpretuje samodzielnie praw z depozytu usera.

### 10.4. Consensus

Consensus odpowiada za:
- walidacje shape i authority batcha,
- walidacje replay protection przez `ticket_nullifier`,
- globalne oznaczanie zuzytych `ticket_nullifier`,
- nie odpowiada za odtworzenie calej semantyki uslugi off-chain.

## 11. Refund, timeout i dispute

Zamrozona zasada:
- refund semantics nie moze byc pozostawiona "supportowi",
- timeout path i refund path musza miec anchor w policy i receipt layer,
- partial settlement path musi byc jawnie opisany przez policy/summary semantics, a nie dopowiadany operacyjnie,
- receipt retention musi istniec przez okno settlement/refund/dispute potrzebne do audytu i review,
- dispute-heavy albo high-sensitivity flows powinny eskalowac do `FullPrivacy`, a nie byc wciskane do lekkiego marketplace raila.

Finalna interpretacja:
- ten rail jest dobry dla usage-metered i drobnicy,
- nie jest dobry dla wysokowartosciowego escrow,
- nie jest dobry dla flow, w ktorych sama kwota albo spor ma wysoka wrazliwosc.

## 12. Current state vs final model

### 12.1. Co juz jest semantycznie dobre

- `ServicePaymentPolicy`, `SpendGrant`, `Receipt`, `SettlementBatchSummary` istnieja w kodzie jako jawne byty,
- wallet ma lokalny `RailContext` i deterministic ticket generation per merchant,
- operator ma prosty model issue-grant / intake-receipt / publish-batch,
- ledger umie oznaczac `ticket_nullifier` jako spent.

### 12.2. Current non-conformities

Najwazniejsze obecne niezgodnosci:
- auth model `MarketplaceBatchTx` jest jeszcze przejsciowy, bo operator signature verification jest warunkowe i zalezne od obecnosci `auth`,
- consensus/ledger nie sprawdzaja jeszcze w pelni wszystkich relacji summary counters i semantics batcha,
- `ticket_nullifier` binding do funding rights pozostaje operatorem egzekwowany off-chain, a nie trust-minimized on-chain proof,
- stare docs nadal mieszaja marketplace rail z dawnym slownikiem `RecipientPrivacyLite`,
- refund/dispute semantics sa jeszcze mocniej opisane w starych docs niz egzekwowane w kodzie,
- partial settlement i receipt retention nie sa jeszcze domkniete jako jeden finalny operational path.

## 13. Co trzeba zrobic, aby ten rail byl pelnie finalny

- dopiac finalne checks dla:
  - `receipt_count`
  - `nullifier_count`
  - `receipt_root`
  - `settlement_root`
  - operator authority
- domknac finalne signed-envelope rules dla `SpendGrant`, `Receipt` i `MarketplaceBatchTx`,
- domknac finalny model signatures/auth dla `MarketplaceBatchTx`,
- zsynchronizowac wallet, operator, ledger i docs do jednego finalnego opisu,
- dopisac vectors i przyklady dla:
  - `ServicePaymentPolicy`
  - `SpendGrant`
  - `Receipt`
  - `SettlementBatchSummary`
  - `MarketplaceBatchTx`
- wyciac semantyczne rozjazdy ze starego pakietu `marketplace_small_payments_v0`,
- utrzymac jawnie, ze jest to rail operator-trusted, a nie udawac pelnej trust minimization.

## 14. Twarde zasady dla dalszych zmian

- nie wolno uzywac tego raila poza marketplace,
- nie wolno usuwac `purchase_commit` bez jawnego freeze update,
- nie wolno zamieniac operator-trusted modelu w "niewiadomo co" bez nowej decyzji architektonicznej,
- nie wolno opisywac tego raila jako tozsamego z `OnChainLite`,
- nie wolno wciskac do tego raila escrow ani flow, ktore powinny isc przez `FullPrivacy`,
- nie wolno oslabic PQ stacku dla tego raila.
