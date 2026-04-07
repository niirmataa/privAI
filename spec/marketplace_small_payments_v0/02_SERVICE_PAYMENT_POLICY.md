# Marketplace Small Payments v0

Status: migration source for final marketplace payment policy.
Canonicality: non-canonical. Ten dokument jest wazny tylko jako material do zlozenia finalnej spec.
Owner: privAI marketplace payments.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: planowany `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`.

## Service Payment Policy

## 1. Cel

Ten dokument ma zamknac warstwe polityki platnosci dla lekkiego raila marketplace-only:

- kto i kiedy moze obciazyc depozyt,
- czy debit jest prepaid, reserve-based czy postpaid,
- jak wyglada `SpendGrant`,
- jakie sa limity, expiry i batching windows,
- kiedy mozna settlementowac,
- kiedy trzeba refundowac,
- kiedy trzeba wymusic ciezszy rail.

Ten dokument ma wymusic jedna spojna semantyke polityki uslugi.
Nie ma zostawic "to ogarniemy w aplikacji".

## 2. Co jest juz zamrozone i nie podlega renegocjacji w tym dokumencie

1. Rail jest `marketplace-only`.
2. Rail zaczyna sie od prywatnego depozytu.
3. Rail sluzy drobnicy, nie duzym settlementom.
4. `RecipientPrivacyLite` jest dopuszczalna dla lekkiego raila.
5. `FullPrivacy` pozostaje wymagana dla escrow, duzych kwot i dispute-sensitive flows.
6. `ticket` jest `strictly one-time`.
7. `ticket` i `tab/session` sa rozdzielone.
8. `purchase_commit` jest wymagany.
9. Domyslny model v0 to `marketplace-operator-trusted accounting`.
10. Marketplace operator jest domyslna authority dla `SessionGrant/SpendGrant` i settlementu.

## 3. Czego ten dokument ma nie robic

Ten dokument nie ma:

- redefiniowac modelu ticketow,
- projektowac finalnego `SettlementTx`,
- usuwac `purchase_commit`,
- wprowadzac slabszej kryptografii PQ,
- przenosic odpowiedzialnosci polityki do niejawnego kodu merchanta.

Ten dokument ma zamknac tylko:

- schemat `ServicePaymentPolicy`,
- semantyke `SessionGrant/SpendGrant`,
- pricing mode,
- reservation mode,
- settlement window,
- refund/dispute/timeout rules,
- moment przelaczenia na `FullPrivacy`.

## 4. Twardy problem do rozwiazania

Musimy umiec odpowiedziec, jak marketplace ma jawnie decydowac:

- czy dana usluga moze uzyc lekkiego raila,
- jaki jest maksymalny debit w sesji,
- czy debit jest natychmiastowy, rezerwowany czy rozliczany po wykonaniu,
- kto moze wystawic charge,
- w jakim oknie operator moze zrobic settlement,
- kiedy user ma prawo do refundu,
- kiedy lekki rail przestaje byc dozwolony i trzeba wejsc w `FullPrivacy`.

Po tej iteracji ma byc jasne:

- jakie pola ma `ServicePaymentPolicy`,
- jakie pola ma `SpendGrant`,
- jaki jest domyslny model v0,
- co wallet, merchant i operator musza respektowac.

## 5. Definicje robocze

### 5.1. ServicePaymentPolicy

`ServicePaymentPolicy` to kanoniczna deklaracja ekonomiki i dopuszczalnych zachowan platniczych dla danej uslugi marketplace.

To nie jest komentarz w kodzie.
To nie jest tylko UI hint.
To jest jawny kontrakt, ktory musi byc respektowany przez:

- wallet,
- merchanta,
- operatora,
- settlement layer.

### 5.2. PaymentRail

`PaymentRail` to wybrany tryb platnosci dla uslugi lub transakcji.

Dla marketplace v0 rozpatrujemy co najmniej:

- `SmallPaymentsRail`
- `RecipientPrivacyLite`
- `FullPrivacy`

`FullPrivacy` pozostaje obowiazkowe dla przypadkow poza zakresem lekkiego raila.

### 5.3. PricingMode

`PricingMode` opisuje, jak naliczana jest naleznosc.

Warianty do oceny:

- `ExactAmount`
- `BucketedAmount`
- `FixedDenomination`
- `UsageMetered`
- `ReservationThenSettle`

### 5.4. ReservationMode

`ReservationMode` opisuje, czy srodki sa:

- obciazane od razu,
- rezerwowane przed wykonaniem,
- czy settlementowane dopiero po wykonaniu.

### 5.5. SessionGrant

`SessionGrant` to operator-scoped autoryzacja otwierajaca sesje malego raila.

Rola:

- scope do `merchant_commit`,
- optional scope do `service_commit`,
- limit czasu,
- limit wartosci lub usage,
- warunki akceptacji dla session/tab.

### 5.6. SpendGrant

`SpendGrant` to bardziej konkretny byt autoryzacyjny spinajacy sesje z prawem wydania.

Rola:

- `spend_cap`,
- `expiry`,
- allowed policy mode,
- allowed merchant/service scope,
- optional refund/dispute profile,
- authority operatora.

### 5.7. SettlementWindow

`SettlementWindow` to czasowe okno, w ktorym operator moze agregowac receipts i publikowac settlement.

To nie jest to samo co expiry ticketu czy expiry grantu.

### 5.8. RefundRule

`RefundRule` opisuje, w jakich warunkach user moze odzyskac srodki lub anulowac charge.

### 5.9. DisputeRule

`DisputeRule` opisuje, kiedy lekki rail przestaje byc wystarczajacy i trzeba wejsc w bardziej formalny albo bardziej prywatny path.

## 6. Domyslne decyzje robocze do przyjecia, jesli nie ma lepszej kontrpropozycji

### D1. Kazda usluga musi miec jawna `ServicePaymentPolicy`

Domyslna decyzja:

- polityka uslugi nie moze byc ukryta w kodzie merchanta,
- wallet musi umiec ja odczytac i zastosowac,
- operator musi umiec ja egzekwowac.

### D2. v0 default dla marketplace drobnicy to `ReservationThenSettle`

Domyslna decyzja:

- dla usage-metered AI marketplace najzdrowszy v0 default to:
  - grant + session + receipts + batch settlement,
- a nie:
  - czysty postpaid bez limitu,
  - ani natychmiastowy heavy on-chain debit per event.

To oznacza:

- srodki sa logicznie zarezerwowane lub scoped przez `SpendGrant`,
- realny transfer ekonomiczny zamyka sie przez settlement batch.

### D3. `SpendGrant` musi miec twardy `spend_cap`

Domyslna decyzja:

- brak nieograniczonych grantow,
- `SpendGrant` zawsze niesie co najmniej:
  - `grant_commit`
  - `merchant_commit`
  - optional `service_commit`
  - `spend_cap`
  - `expiry`
  - `pricing_mode`
  - `reservation_mode`

### D4. `SettlementWindow` musi byc jawny

Domyslna decyzja:

- usluga nie moze miec "kiedys settlementujemy",
- policy musi okreslic:
  - jak dlugo receipts moga czekac,
  - kiedy operator musi settlementowac,
  - kiedy user moze uznac brak settlementu za failure path.

### D5. Lekkie rail nie jest dla escrow i sporow z wysoka stawka

Domyslna decyzja:

- jesli usluga ma:
  - istotne ryzyko sporu,
  - wysokie wartosci,
  - potrzebe ukrycia kwoty,
  - payout sensitivity,
  to policy musi wymusic `FullPrivacy`.

### D6. Refund rule musi byc okreslona upfront

Domyslna decyzja:

- refund semantics nie moze byc pozostawiona "supportowi",
- policy musi jawnie okreslic:
  - `no_refund`
  - `timeout_refund`
  - `merchant_no_delivery_refund`
  - `partial_refund_allowed`
  - `operator_discretionary_refund`

### D7. `UsageMetered` bez granic jest zabronione

Domyslna decyzja:

- jesli usluga jest usage-metered, to musi miec:
  - granice czasu,
  - granice wartosci,
  - granice usage units,
  - granice settlement window.

Nie wolno zostawiac otwartego, nieskonczonego taba bez sztywnej polityki.

### D8. Wallet musi umiec odrzucic policy niezgodna z privacy tier

Domyslna decyzja:

- wallet nie tylko wyswietla policy,
- wallet musi umiec odrzucic flow, jesli:
  - rail jest za slaby dla wartosci,
  - usluga wymaga escrow, a policy probuje uzyc lekkiego raila,
  - settlement window jest nieakceptowalnie dlugie,
  - spend cap lub refund semantics sa niezgodne z profilem usera.

## 7. Wymagany model `ServicePaymentPolicy`

Masz pracowac z nastepujacym szkicem:

```text
ServicePaymentPolicy
  = service identity
  + allowed rail(s)
  + pricing mode
  + reservation mode
  + grant constraints
  + settlement constraints
  + refund/dispute constraints
  + batching constraints
```

Na starcie przyjmij, ze policy powinna umiec niesc co najmniej:

- `policy_version`
- `merchant_commit`
- `service_commit`
- `allowed_rail`
- `pricing_mode`
- `reservation_mode`
- `min_deposit_required`
- `max_spend_per_session`
- `max_spend_per_window`
- `max_usage_units`
- `grant_expiry_rule`
- `settlement_window_rule`
- `acceptance_rule`
- `timeout_rule`
- `refund_rule`
- `dispute_rule`
- `batching_rule`
- `requires_full_privacy_if`
- `policy_commit`

Masz odpowiedziec:

1. ktore pola sa obowiazkowe,
2. ktore pola maja byc publiczne,
3. ktore moga byc tylko off-chain,
4. jak policy laczy sie z `grant_commit`,
5. jak wallet weryfikuje zgodnosc flow z policy.

## 8. Wymagany model `SessionGrant` i `SpendGrant`

Masz rozstrzygnac:

### 8.1. Kto wystawia grant

Domyslny v0:

- marketplace operator

Musisz opisac:

- czy merchant moze prosic o grant,
- czy wallet moze zainicjowac grant,
- czy grant jest wydawany per sesja, per zakup, czy per okno czasu.

### 8.2. Jak wyglada minimalny `SpendGrant`

Na starcie przyjmij, ze powinien zawierac co najmniej:

- `grant_commit`
- `merchant_commit`
- optional `service_commit`
- `session_scope`
- `spend_cap`
- `currency_or_unit_mode`
- `grant_expiry`
- `settlement_window`
- `policy_commit`
- `operator_sig`

### 8.3. Cap model

Musisz porownac:

- cap per session
- cap per service window
- cap per purchase
- cap per merchant

Domyslny kierunek v0:

- `cap per session`
- plus optional `cap per settlement window`

## 9. Wymagany model pricing i reservation

To jest sekcja krytyczna.

Masz ocenic i rozstrzygnac co najmniej te warianty:

### Wariant A. Exact per debit

- kazdy ticket/debit ma dokladna kwote
- proste settlement semantics
- slabsza privacy kwoty

### Wariant B. Bucketed

- debit wchodzi do koszyka
- lepsza privacy kwoty
- wiecej zlozonosci billingowej

### Wariant C. Fixed denomination

- tylko stale wartosci
- najprostsze rozliczenie
- najslabsza ergonomia uslug usage-metered

### Wariant D. Reservation then exact settle

- grant rezerwuje scope/limit,
- finalna kwota wynika z receipts,
- najlepsze dopasowanie do AI marketplace,
- wymaga najmocniejszej warstwy receiptow i settlementu.

Domyslny wybor v0:

- `ReservationThenSettle` jako default dla usage-metered marketplace,
- `Exact per debit` jako opcjonalny prostszy profil dla jednorazowych uslug.

Masz tez odpowiedziec:

1. czy `amount` ma byc dokladny czy bucketed w domyslnym railu v0,
2. czy ma to byc decyzja policy, czy globalna decyzja protokolu,
3. kiedy wallet powinien odmowic wejscia w usage-metered flow.

## 10. Wymagany model refund, dispute i timeout

Musisz opisac, jak policy definiuje:

- brak wykonania uslugi,
- opozniony settlement,
- partial delivery,
- anulowanie przez usera,
- anulowanie przez merchanta,
- expiry grantu,
- expiry sesji.

Musisz rozstrzygnac:

### 10.1. Refund path

Czy refund:

- jest automatyczny po timeout,
- wymaga receipt proof,
- wymaga operator decision,
- moze byc partial.

### 10.2. Dispute threshold

Masz odpowiedziec:

- kiedy lekki rail jeszcze wystarcza,
- kiedy usluga musi wymusic `FullPrivacy`,
- czy istnieje wartosciowy lub semantyczny prog, po ktorym lekki rail jest niedozwolony.

### 10.3. Timeout rule

Masz odpowiedziec:

- kiedy grant wygasa,
- kiedy session wygasa,
- kiedy settlement window wygasa,
- co dzieje sie z nierozliczonymi receipts po timeout.

## 11. Wymagany model batching i redeem authority

Masz odpowiedziec:

1. czy settlement jest per merchant, per service czy per grant,
2. czy operator publikuje zawsze sam,
3. czy merchant moze publikowac tylko jako delegated actor,
4. jak policy ogranicza batching window,
5. czy policy ogranicza maksymalny rozmiar batcha,
6. jak policy laczy sie z `receipt_root` i `settlement_root`.

Domyslny v0:

- settlement publikuje operator,
- batching jest co najmniej merchant-window scoped,
- service-scoped batching jest opcjonalny dla wrazliwych uslug.

## 12. Pytania, ktore musza dostac odpowiedz

Nie wolno zostawic tych pytan bez odpowiedzi:

1. Jak wyglada minimalny `ServicePaymentPolicy`?
2. Jakie pola policy sa obowiazkowe?
3. Czy policy jest publiczne, czy tylko marketplace-visible?
4. Jak wyglada minimalny `SpendGrant`?
5. Kto wystawia grant?
6. Czy grant jest per session, per purchase czy per time window?
7. Jaki jest domyslny `pricing_mode` v0?
8. Jaki jest domyslny `reservation_mode` v0?
9. Czy amount jest exact, bucketed czy denominated?
10. Jak wyglada refund rule?
11. Jak wyglada timeout rule?
12. Kiedy policy musi wymusic `FullPrivacy`?
13. Kto publikuje settlement w v0?
14. Jak policy ogranicza batching?
15. Jak wallet ma zdecydowac, czy wejsc w dany rail?

## 13. Czego nie wolno proponowac

Nie proponuj:

- policy ukrytej tylko w kodzie merchanta,
- grantow bez `spend_cap`,
- grantow bez expiry,
- usage-metered flow bez settlement window,
- lekkiego raila dla escrow i wysokowartosciowych sporow,
- modelu, w ktorym merchant sam bez operatora ma authority do wydania depozytu,
- polityki bez refund/timeout semantics,
- polityki, ktora nie mowi walletowi, kiedy ma przelaczyc sie na `FullPrivacy`.

## 14. Wymagany format odpowiedzi

Odpowiedz ma miec dokladnie te sekcje:

1. `Recommended v0 design`
2. `Alternative design`
3. `ServicePaymentPolicy data model`
4. `SpendGrant data model`
5. `Pricing and reservation model`
6. `Refund and timeout model`
7. `Batching and settlement authority model`
8. `Wallet decision model`
9. `Frozen decisions`
10. `Open questions`

## 15. Oczekiwany wynik

Po przeczytaniu tego dokumentu i wykonaniu zadania mamy dostac:

- gotowa definicje `ServicePaymentPolicy`,
- gotowa definicje `SpendGrant`,
- gotowy domyslny `pricing_mode`,
- gotowy domyslny `reservation_mode`,
- gotowa odpowiedz, kiedy lekki rail jest dozwolony,
- gotowa odpowiedz, kiedy trzeba wymusic `FullPrivacy`,
- gotowa odpowiedz, kto publikuje settlement,
- gotowa odpowiedz, jak wyglada refund i timeout,
- minimalna liste pytan otwartych, jesli cos naprawde musi zostac otwarte.

Nie chcemy po tej iteracji dostac kolejnego brainstormingu.
Chcemy dostac material, z ktorego da sie robic spec i implementacje.
