# Gemini 3.1 Pro Task

## Cel

Masz zaprojektowac konkretny, roboczy model `marketplace-only small-payments rail` dla `privAI v0`.

Nie chodzi o ogolny brainstorming. Chodzi o architekture v0, ktora da sie zamrozic jako:

- model danych,
- model prywatnosci,
- model ekonomiczny,
- model escrow / settlement,
- lista dalszych zadan implementacyjnych.

Twoim celem jest dac 1 rekomendowany model v0 i 1 alternatywe.

## Workspace i granice pracy

Pracujesz tylko na:

- `/home/nxms-server/privAI`

Folder wyjsciowy dla rezultatow:

- `/home/nxms-server/privAI/audyt/marketplace_small_payments_v0`

Nie uzywaj sciezek Windows.
Nie pracuj na zadnych kopiach poza Alpine WSL.

## Role, ktore masz przyjac

Pracuj jednoczesnie jako:

1. `Protocol Architect`
2. `Privacy Payments Designer`
3. `Adversarial Reviewer`
4. `Marketplace Product Systems Designer`

## Najpierw przeczytaj

1. `/home/nxms-server/privAI/PRIVAI_V0_PROTOCOL.md`
2. `/home/nxms-server/privAI/PRIVAI_V0_PAYMENTS_AND_ECONOMICS.md`
3. `/home/nxms-server/privAI/privai-chain/src/params.rs`
4. `/home/nxms-server/privAI/privai-chain/src/tx.rs`
5. `/home/nxms-server/privAI/privai-chain/src/note.rs`
6. `/home/nxms-server/privAI/privai-chain/src/consensus.rs`
7. `/home/nxms-server/privAI/privai-proof/src/artifact.rs`
8. `/home/nxms-server/privAI/privai-proof/src/verify.rs`

## Zamrozone zalozenia

To nie sa juz pytania otwarte. Traktuj to jako wejscie.

### Polityka prywatnosci i ekonomii

1. `FullPrivacy` jest wymagane dla:
   - escrow,
   - payoutow o wyzszej wrazliwosci,
   - settlementow o wyzszej wartosci,
   - transakcji, w ktorych sama kwota jest dana wrazliwa.

2. Dla drobnicy domyslny model to:
   - `deposit -> off-chain service state -> batch settlement`

3. Jesli mala platnosc trafia on-chain, lekkim rail'em ma byc:
   - `RecipientPrivacyLite`
   - czyli: `ukryty adres + jawna kwota`

4. Nie oslabiamy warstwy PQ dla drobnicy.
   - Nie projektuj slabszej klasy PQ
   - Zmieniamy zakres prywatnosci i model ekonomiczny, nie fundament bezpieczenstwa

5. Nie chcemy stalego publicznego konta uzytkownika dla tego raila.

6. Nie chcemy stalego publicznego identyfikatora platnika.

7. NXMS / transport / relay to nie settlement layer.
   - nie mieszaj transportu z prawda o pieniadzu

### Kierunek produktowy

`privAI v0` jest naturalnie:

- private settlement layer,
- escrow layer,
- finalization layer,

a nie:

- per-event micropayment chain dla kazdego malego klikniecia.

### Wniosek ekonomiczny, ktory juz zmierzylismy

Przy realistycznym `RecipientBox`:

- `output_note_bytes` dla pelnej prywatnej noty: ok. `14240 B`
- `output_note_bytes_recipient_privacy_lite`: ok. `10142 B`

To daje realna ulge, ale pokazuje tez uczciwie, ze:

- `RecipientBox` nadal pozostaje glownym kosztem,
- samo usuniecie ukrytej kwoty nie rozwiazuje calej ekonomii,
- dla drobnicy nadal bardzo wazna jest amortyzacja przez depozyt, tab i batching.

Dodatkowy wniosek z raportu:

- direct settlement: ok. `30024 B / purchase`
- deposit rail przy `N=10`: ok. `3002 B / purchase`
- merchant tab przy `N=50`: ok. `600 B / purchase`
- batch escrow przy `N=20`: ok. `1501 B / purchase`

## Problem do rozwiazania

Zaprojektuj `marketplace-only small-payments rail` dla publicznego chaina `privAI`, ktory:

- zaczyna sie od prywatnego depozytu,
- umozliwia wiele malych platnosci,
- moze wykorzystywac lekki rail on-chain,
- nie ujawnia stalej tozsamosci platnika,
- nie opiera sie na stalym publicznym koncie,
- ma ochrone przed replay i double-spend,
- jest sensowny ekonomicznie dla drobnicy,
- da sie spiac z marketplace i escrow logic.

Kluczowa intuicja, ktora masz ocenic i uszczelnic:

- `deposit-backed anonymous tickets`
- albo
- `one-time debit IDs / nullifiers backed by private deposit`

Nie traktuj tego jako dogmatu.
Mozesz zaproponowac lepszy wariant v0, ale musi spelniac te same wymagania.

## Jedna bardzo wazna hipoteza robocza

Rozwaz rail, w ktorym:

1. uzytkownik robi prywatny depozyt,
2. wallet lokalnie generuje pule jednorazowych identyfikatorow z tajnego seeda,
3. male platnosci ida on-chain w lekkim formacie:
   - `one-time id` albo lepiej `nullifier-like token`
   - `public amount`
   - `merchant/service/purchase commit`
4. chain nie widzi stalego konta ani stalego rail ID,
5. merchant / marketplace pozniej rozlicza settlement i receipts.

Masz odpowiedziec, czy to jest najlepszy model v0 i jak go uszczelnic.

## Co chcemy dostac

Odpowiedz ma byc podzielona na 10 sekcji i miec jasny werdykt.

### 1. Executive summary

Podaj:

- 1 rekomendowany model v0
- 1 alternatywe
- krotkie uzasadnienie, czemu rekomendowany model wygrywa

### 2. Threat model

Przeanalizuj:

- linkowanie wielu zakupow tego samego uzytkownika
- linkowanie depozytu z malymi debitami
- replay
- double-spend
- merchant fraud
- marketplace/operator overreach
- timing/value correlation
- spam / dust / griefing

### 3. Architektura warstwowa

Rozpisz:

- co jest on-chain
- co jest off-chain
- co zyje w wallet
- co zyje po stronie merchant
- co zyje po stronie marketplace
- co jest settlement / receipt / proof material

### 4. Obiekty danych

Zaprojektuj co najmniej:

- `DepositAnchor`
- `TicketSeed`
- `TicketId`
- `TicketNullifier`
- `SmallDebitTx`
- `Receipt`
- `BatchRedeemTx`
- `SettlementRoot`
- `ServicePaymentPolicy`

Dla kazdego podaj:

- cel
- pola
- co jest publiczne
- co jest prywatne
- co jest jednorazowe
- co jest commitmentem
- co jest nullifierem

### 5. Flow end-to-end

Rozpisz dokladnie:

1. deposit open
2. local ticket pool generation
3. small payment authorization
4. on-chain lightweight debit
5. merchant acknowledgment / receipt
6. batch redeem / settlement
7. refund / expiration / unused deposit handling

Osobno:

- wariant z marketplace operator
- wariant bez marketplace operatora

### 6. Privacy model

Bardzo konkretnie odpowiedz:

- co widzi chain
- czego chain nie widzi
- co widzi merchant
- co widzi marketplace
- co mozna skorelowac
- czego nie powinno dac sie powiazac

Porownaj:

- `FullPrivacy`
- `RecipientPrivacyLite`
- `deposit-backed anonymous ticket rail`

### 7. Anti-linkability i mechanika ID/nullifierow

To jest sekcja krytyczna.

Musisz odpowiedziec:

- czy na chain powinien trafic `ticket_id`
- czy lepiej tylko `ticket_nullifier`
- czy oba
- czy potrzebny jest `merchant_commit`
- czy potrzebny jest `service_commit`
- czy potrzebny jest `purchase_commit`

Rozpisz model lokalnej generacji:

- `ticket_id_i`
- `ticket_nullifier_i`
- `auth_key_i`

Rozpisz:

- rotacje
- pule ticketow
- recovery po utracie urzadzenia
- unikanie linkowalnosci miedzy merchantami
- unikanie linkowalnosci miedzy sesjami

### 8. Economics

Porownaj ten rail do:

- `FullPrivacy`
- `RecipientPrivacyLite`
- `deposit + off-chain tab + batch settlement`
- `batch escrow`

Odpowiedz:

- kiedy ten rail ma sens
- kiedy nie ma sensu
- czy male platnosci on-chain powinny uzywac:
  - dokladnej kwoty
  - amount buckets
  - fixed denominations

### 9. Konsekwencje produktowe i UX

Rozpisz:

- kiedy wallet wybiera jaki rail
- kiedy ma wymuszac `FullPrivacy`
- kiedy ma pokazywac `RecipientPrivacyLite`
- jak wyglada onboarding do depozytu
- jak wyglada merchant tab / ticket pool z perspektywy UX
- jak komunikowac kompromis prywatnosc vs koszt

### 10. Output implementacyjny

Na koncu daj:

- liste decyzji do zamrozenia
- liste otwartych pytan
- liste zadan implementacyjnych na v0
- liste zadan na v1 / v2
- propozycje minimalnego pakietu dokumentow:
  - `small_payments_rail.md`
  - `ticket_id_and_nullifier.md`
  - `service_payment_policy.md`
  - `receipt_and_settlement_root.md`
  - `marketplace_privacy_tiers.md`

## Kolejnosc pracy, ktorej oczekujemy

Wykonaj to w tej kolejnosci:

1. `frozen assumptions memo`
2. `recommended v0 design`
3. `alternative design`
4. `threat model`
5. `data model`
6. `flows`
7. `privacy analysis`
8. `economics analysis`
9. `UX / product consequences`
10. `implementation roadmap`

Nie mieszaj tych etapow.

## Ograniczenia

1. Nie odpowiadaj: `po prostu wszystko off-chain`.
   - off-chain jest czescia rozwiazania, ale projekt ma objac takze lekki rail on-chain

2. Nie proponuj slabszej warstwy PQ dla malych platnosci

3. Nie zakladaj stalego publicznego konta platnika

4. Nie zakladaj stalego publicznego rail ID

5. Nie mieszaj transportu z settlementem

6. Traktuj to jako architekture `marketplace-only rail`, nie jako uniwersalny system wszystkich platnosci

## Forma odpowiedzi

Odpowiedz ma byc:

- konkretna
- decyzyjna
- architektoniczna
- bez lania wody
- z 1 polecanym modelem v0
- z 1 alternatywa
- z tabelami tam, gdzie pomagaja

## Finalny wymog

Na koncu dodaj sekcje:

`If I had to freeze v0 this week`

W tej sekcji podaj 1 rekomendowana sciezke bez wahania.
