# Marketplace Small Payments v0

## Small Payments Rail v0

## 1. Cel

Ten dokument opisuje rekomendowany roboczy model v0 dla malych platnosci w marketplace.

Nie opisuje on pelnego `FullPrivacy` raila dla duzych kwot i escrow.
Opisuje lekki rail dla drobnicy, ktory:

- zaczyna sie od prywatnego depozytu,
- pozwala rozliczac male platnosci,
- nie ujawnia stalego konta platnika,
- moze uzyc jawnej kwoty,
- ma byc sensowny ekonomicznie dla retail / micro-packages.

## 2. Werdykt v0

Rekomendowany model v0:

- `private deposit`
- `off-chain service state`
- `anonymous one-time debit rights`
- `public amount`
- `batch settlement by receipts`

To nie jest rail, w ktorym kazdy maly zakup robi pelna prywatna note.
To jest rail, w ktorym:

- finansowanie jest prywatne,
- sesja i relacja user-service sa odsloniete tylko tyle, ile trzeba,
- pojedynczy maly zakup moze byc lekki,
- finalny settlement jest amortyzowany.

## 3. Zakres uzycia

Ten rail jest dobry dla:

- retail AI packages,
- inference credits,
- usage-metered marketplace sessions,
- niewielkich jednorazowych zakupow,
- subskrypcyjnych lub powtarzalnych wydatkow u jednego merchanta.

Ten rail nie jest dobry dla:

- escrow,
- flows z dispute-heavy semantics,
- settlementow o wyzszej wartosci,
- payoutow o wysokiej wrazliwosci,
- przypadkow, gdzie sama kwota powinna byc ukryta.

## 4. Privacy tier dla tego raila

Ten rail dziala pod polityka:

- `RecipientPrivacyLite`

Czyli:

- platnik nie ma stalego publicznego konta,
- odbiorca / merchant nie musi poznawac globalnej tozsamosci usera,
- jednorazowe debity sa niepowiazywalne przez stale ID,
- kwota malej platnosci moze byc jawna.

To jest swiadomy kompromis:

- dobra prywatnosc relacji,
- mniejsza prywatnosc kwoty,
- wyraznie lepsza ekonomia niz `FullPrivacy` per purchase.

## 5. Podstawowe byty

## 5.1. DepositAnchor

`DepositAnchor` to nie jest publiczne konto.
To jest finansowy punkt wejscia do raila.

Minimalna semantyka:

- powstaje z prywatnego depozytu,
- definiuje pule srodkow lub limitu dla small-payments rail,
- jest zrodlem prawa do generowania debit tickets,
- moze miec ograniczony lifecycle:
  - amount cap
  - service scope
  - merchant scope
  - expiry

Chain nie powinien dostawac stalego identyfikatora usera powiazanego z tym anchorem.

## 5.2. TicketSeed

Lokalny sekret walleta.
Nie trafia on-chain.

Z niego wyprowadzamy:

- `ticket_id_i`
- `ticket_nullifier_i`
- `ticket_auth_i`
- ewentualnie `merchant-bound tag`

Seed musi dawac:

- jednorazowosc
- lokalna odtwarzalnosc
- brak stalego publicznego rail ID

## 5.3. TicketId

`TicketId` jest obiektem lokalnym lub pol-publicznym.

Rola:

- identyfikator biznesowy pojedynczego debitu,
- zwiazanie z receipt / service event / purchase context.

Wersja rekomendowana:

- nie traktowac `ticket_id` jako glownego on-chain identyfikatora anty-replay.
- On-chain lepiej oprzec sie o `ticket_nullifier`.

## 5.4. TicketNullifier

To jest najwazniejszy publiczny marker jednorazowosci.

Rola:

- replay protection
- double-spend protection
- brak stalego publicznego konta

Chain powinien pilnowac globalnej unikalnosci `ticket_nullifier`.

## 5.5. SmallDebitTx

Lekki typ platnosci dla drobnicy.

Minimalne publiczne pola:

- `version`
- `payment_mode = RecipientPrivacyLite`
- `ticket_nullifier`
- `merchant_commit`
- `service_commit`
- `purchase_commit`
- `amount`
- `fee`
- `statement_commit`

Minimalne prywatne / off-chain korelaty:

- lokalny `ticket_id`
- auth material
- receipt context
- policy context

`amount` moze byc:

- dokladna jawna kwota,
- bucket,
- denomination

Na v0 rekomendacja:

- zaczac od jawnej kwoty lub prostych bucketow,
- nie komplikowac od razu denomination engine, jesli nie ma twardej potrzeby.

## 5.6. Receipt

Receipt jest minimalnym dowodem uslugowym.

Minimalne pola:

- `receipt_id`
- `purchase_commit`
- `merchant_commit`
- `service_commit`
- `amount`
- `time_window`
- `result_commit`
- `policy_commit`

Receipt musi nadawac sie do:

- batching,
- refund path,
- dispute path,
- settlement audit.

## 5.7. BatchRedeem / SettlementRoot

Merchant albo marketplace nie musi settlementowac kazdego eventu osobno.

Rekomendowany model:

- wiele receipts tworzy `receipt_root`
- settlement layer redeemuje batch
- chain widzi finalna operacje settlementowa, nie kazdy detal sesji

## 5.8. ServicePaymentPolicy

Kazda usluga musi zadeklarowac:

- `rail`
- `pricing_mode`
- `reservation_mode`
- `deposit_minimum`
- `settlement_window`
- `acceptance_rule`
- `refund_rule`
- `dispute_rule`
- `timeout_rule`
- `batching_rule`

To nie moze byc ukryte w kodzie aplikacji.

## 6. Model on-chain vs off-chain

## 6.1. On-chain

Chain powinien widziec tylko minimum:

- prywatny depozyt / jego wynikowy anchor semantics
- `ticket_nullifier`
- `amount`
- `merchant_commit`
- `service_commit`
- `purchase_commit`
- final settlement / redeem
- refund / expiry outcome

## 6.2. Off-chain

Poza chainem powinno zyc:

- lokalna pula ticketow
- `ticket_id`
- merchant tab state
- detailed receipts
- intermediate service events
- batching orchestration
- customer-visible session state

## 7. Flow v0

## 7.1. Deposit open

1. Uzytkownik finansuje prywatny depozyt.
2. Wallet tworzy lokalny rail context.
3. Rail context ma:
   - `deposit anchor semantics`
   - `ticket seed`
   - policy scope
   - optional merchant scope
   - expiry

## 7.2. Ticket pool generation

Wallet lokalnie generuje pule jednorazowych ticketow:

- `ticket_id_i`
- `ticket_nullifier_i`
- `ticket_auth_i`

Opcjonalnie:

- merchant-bound tickets
- session-bound tickets
- scope-bound tickets

## 7.3. Small payment authorization

Przy zakupie wallet:

1. wybiera rail,
2. wybiera ticket,
3. tworzy `purchase_commit`,
4. wiąze purchase z merchant / service context,
5. podpisuje lub autoryzuje debit.

## 7.4. On-chain lightweight debit

Na chain idzie `SmallDebitTx`.

Chain sprawdza:

- poprawny format
- poprawny nullifier uniqueness
- poprawny policy scope
- poprawny amount semantics
- poprawny statement / auth semantics

Chain nie uczy sie:

- stalego konta usera
- stalego publicznego rail ID
- globalnej historii ticket pool

## 7.5. Merchant acknowledgment

Merchant lub marketplace generuje receipt:

- potwierdza wykonanie uslugi
- wiąże wynik z `purchase_commit`
- przygotowuje dane do settlementu

## 7.6. Batch settlement

Po oknie czasu:

1. receipty sa agregowane do `receipt_root`
2. merchant / marketplace robi `BatchRedeem`
3. settlement rozlicza:
   - naleznosc merchanta
   - refund delta
   - ewentualny fee split

## 7.7. Refund / expiry / unused funds

Trzeba przewidziec:

- niewykorzystane ticketi
- wygasly rail context
- merchant non-delivery
- timeout without completion

To moze isc przez:

- `refund batch`
- `close rail`
- `expiry reclaim`

## 8. Anti-linkability zasady v0

## 8.1. Czego nie robimy

- nie publikujemy stalego `payer_account`
- nie publikujemy stalego `rail_id`
- nie publikujemy jednego jawnego `session_id`, ktory trwa miesiacami

## 8.2. Co publikujemy

Preferowany publiczny marker:

- `ticket_nullifier`

Opcjonalny dodatkowy marker:

- `purchase_commit`

`ticket_id` nie powinien byc glownym on-chain identyfikatorem replay protection.

## 8.3. Merchant / service binding

Każdy debit powinien byc zwiazany co najmniej z:

- `merchant_commit`
- `service_commit`
- `purchase_commit`

To zmniejsza ryzyko:

- reuzycia ticketu poza zakresem
- merchant-side confusion
- dispute ambiguity

## 8.4. Recovery

Rail musi miec przynajmniej jedna z dwoch sciezek:

- deterministic recovery z lokalnego seeda
- backup rail context encrypted w wallet backup

Nie wolno opierac recovery tylko na pamieci aplikacji.

## 9. Czy male platnosci powinny byc zawsze on-chain

Nie.

Najzdrowszy model v0:

- retail default:
  - deposit + off-chain state
- on-chain lite:
  - gdy potrzebna jest trwala finalnosc per debit
  - albo gdy merchant chce prostszego modelu rozliczenia
- escrow / sensitive:
  - tylko `FullPrivacy`

Czyli `SmallDebitTx` ma byc narzedziem, nie domyslna odpowiedzia na kazdy event.

## 10. Otwarta lista decyzji do domkniecia

1. Czy `amount` na v0 ma byc:
   - dokladny
   - bucketed
   - denominated

2. Czy `SmallDebitTx` rozlicza:
   - pojedynczy purchase
   - czy tylko rezerwacje pod purchase

3. Czy redeem jest:
   - per merchant
   - per service
   - per time window
   - per session

4. Czy merchant sam sklada redeem, czy robi to marketplace

5. Czy ticketi maja byc:
   - globalne
   - merchant-scoped
   - service-scoped

6. Jak wyglada dispute path dla lekkiego raila:
   - czy tylko refund/no-refund
   - czy tez partial settlement

## 11. Rekomendacja na teraz

Jesli mielibysmy zamrozic v0 teraz, to:

- `DepositRail` pozostaje domyslnym retail rail'em
- `SmallDebitTx` jest opcjonalnym lekkim on-chain debit rail'em
- `ticket_nullifier` jest glownym publicznym replay guard
- `amount` jest jawny
- `merchant_commit + service_commit + purchase_commit` sa jawne commitmenty kontekstowe
- `BatchRedeem` settlementuje receipts
- `Escrow` i duze kwoty pozostaja w `FullPrivacy`

To jest najmocniejszy, najbardziej realistyczny kompromis miedzy:

- prywatnoscia
- ekonomia
- prostota v0
- i mozliwoscia rozwoju do mocniejszego modelu w v1/v2.
