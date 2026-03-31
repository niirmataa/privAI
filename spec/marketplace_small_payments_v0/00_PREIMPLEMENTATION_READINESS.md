# Marketplace Small Payments v0

## Pre-Implementation Readiness

## 1. Cel tego dokumentu

Ten dokument jest checkpointem przed wejsciem w ciezsze rzeczy:

- nowe typy tx,
- proof integration,
- receipt settlement logic,
- wallet state machine,
- marketplace batching.

Jego rola jest prosta:

- nazwac, co juz wiemy,
- nazwac, czego jeszcze nie wiemy,
- nie pozwolic wejsc w implementacje z ukrytymi zalozeniami.

## 2. Co jest juz zamrozone

### 2.1. Polityka prywatnosci

- `FullPrivacy` jest wymagane dla:
  - escrow,
  - dispute-sensitive flows,
  - settlementow o wyzszej wartosci,
  - payoutow o wyzszej wrazliwosci,
  - przypadkow, gdzie sama kwota jest dana wrazliwa.

- Dla drobnicy domyslny model to:
  - `deposit -> off-chain service state -> batch settlement`

- Jesli mala platnosc trafia on-chain, lekkim rail'em jest:
  - `RecipientPrivacyLite`
  - czyli `ukryty adres + jawna kwota`

- Nie projektujemy slabszej warstwy PQ dla drobnicy.

### 2.2. Polityka ekonomiczna

- `privAI v0` nie jest rail'em do pelnej prywatnej mikropatnosci on-chain per event.
- Najwiekszy koszt on-chain nadal robi:
  - `RecipientBox`
  - `OutputNote`
  - fanout outputow / change
- Proof bytes nie sa glownym payloadem bloku, bo zyja jako sidecar/artifact.

### 2.3. Kierunek dla marketplace

- Dla small payments chcemy rail marketplace-only.
- Nie chcemy stalego publicznego konta platnika.
- Nie chcemy stalego publicznego rail ID.
- Chcemy jednorazowe, niepowiazywalne identyfikatory / nullifiery.

## 3. Co musi byc mentalnie jasne przed implementacja

To sa rzeczy, ktore musimy miec jasno w glowie, bo inaczej implementacja szybko odpali w zla strone.

### 3.1. Nie kazda prywatnosc jest taka sama

Mamy co najmniej 3 klasy:

- `ServicePrivacy`
  - prywatnosc sesji, receipts i relacji uslugowej
- `RecipientPrivacy`
  - ukrycie odbiorcy / platnika / relacji kto-komu
- `AmountPrivacy`
  - ukrycie kwoty

W small-payments rail mozemy swiadomie miec:

- dobra `RecipientPrivacy`
- dobra `ServicePrivacy`
- slabsza albo zerowa `AmountPrivacy`

To nie jest bug. To jest jawna polityka produktu.

### 3.2. Deposit nie jest kontem

`deposit-backed rail` nie moze ewoluowac mentalnie w:

- stale konto,
- publiczny balance account,
- latwo linkowalny rail state.

Depozyt ma byc:

- anchorem finansowania,
- zrodelm ticketow / debit rights,
- punktem wejscia do later settlement,

ale nie publiczna tozsamoscia uzytkownika.

### 3.3. Ticket / ID to nie moze byc tylko ladna nazwa

Jednorazowe ID bez dobrze zaprojektowanej semantyki szybko staje sie linkowalne lub nieskuteczne.

Musimy od razu odpowiedziec:

- czy publiczny jest `ticket_id`, `ticket_nullifier`, czy oba,
- co jest replay protection,
- co jest bound do merchanta / service,
- jak wyglada lifecycle ticketu,
- co dzieje sie przy utracie urzadzenia,
- co dzieje sie przy konflikcie miedzy merchantem a userem.

### 3.4. Off-chain state nie moze byc mgla

Jesli mowimy `off-chain state`, to to nie moze znaczyc:

- wszystko dzieje sie poza chainem i kiedys to ogarniemy.

Musi byc jawnie rozpisane:

- jakie sa obiekty stanu,
- kto jest ich autorem,
- kto je podpisuje,
- kto je przechowuje,
- kto moze je odtworzyc,
- co jest rozstrzygalne on-chain, a co nie.

### 3.5. Receipt to nie jest tylko log

Receipt musi byc projektowany jak obiekt settlementowy.

To znaczy:

- ma byc komitowalny,
- ma miec sens ekonomiczny,
- ma nadawac sie do batching,
- ma nadawac sie do refund / dispute path,
- ma byc minimalny, ale wystarczajacy dla auditability.

## 4. Co trzeba domknac organizacyjnie

### 4.1. Wlasciciel decyzji

Przed implementacja trzeba jawnie wskazac:

- kto zamraza privacy tiers,
- kto zamraza service policy schema,
- kto zamraza receipt schema,
- kto zamraza settlement semantics,
- kto zatwierdza kompromisy ekonomiczne.

Bez tego decyzje beda dryfowac miedzy:

- protocol layer,
- wallet layer,
- marketplace logic,
- proof layer.

### 4.2. Wspolny slownik

Trzeba zamrozic wspolny slownik terminow, zeby nie mieszac:

- `deposit`
- `anchor`
- `ticket`
- `nullifier`
- `receipt`
- `tab`
- `settlement`
- `claim`
- `redeem`
- `refund`
- `dispute`
- `recipient privacy`
- `amount privacy`

### 4.3. Granice odpowiedzialnosci zespolow

Zanim pojdzie implementacja, musi byc jasne:

- co nalezy do `privai-chain`,
- co nalezy do `privai-proof`,
- co nalezy do walleta,
- co nalezy do marketplace layer,
- co nalezy do transport/relay.

### 4.4. Readiness do review

Potrzebujemy review z co najmniej 3 perspektyw:

- protocol
- privacy / adversarial analysis
- product / UX / economics

## 5. Co trzeba domknac technicznie przed grubsza implementacja

### 5.1. Privacy tiers jako formalny kontrakt

Musimy zapisac twardo:

- kiedy wolno uzyc `RecipientPrivacyLite`
- kiedy trzeba wymusic `FullPrivacy`
- czy decyzja nalezy do walleta, merchanta, marketplace czy polityki uslugi

### 5.2. ServicePaymentPolicy

Potrzebujemy finalnego szkicu:

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

### 5.3. Ticket semantics

Musimy domknac:

- lokalny seed
- ticket pool generation
- jednorazowosc
- replay protection
- merchant binding
- optional service binding
- expiry
- unused ticket recovery

### 5.4. Receipt schema

Musimy domknac:

- minimalne pola receiptu
- kto podpisuje receipt
- czy receipt jest jednostronny czy dwustronny
- jak wchodzi do `receipt_root`
- jak receipt laczy sie z payout / refund / dispute

### 5.5. Settlement semantics

Musimy domknac:

- kto robi redeem
- czy redeem jest per merchant, per session czy per batch
- czy chain rozlicza claim czy tylko final aggregate
- jak wyglada timeout
- jak wyglada refund sciezka

### 5.6. Accountless replay protection

Musimy miec bardzo jasna odpowiedz:

- jak chain wie, ze debit jest wazny,
- jak chain wie, ze nie jest powtorka,
- jak chain nie uczy sie stalego identyfikatora usera.

## 6. Czego nie wolno zrobic za szybko

- Nie wolno od razu projektowac 5 nowych typow tx bez zamrozenia modelu.
- Nie wolno pchac proof layer w lekki rail, jesli jeszcze nie wiemy, co jest publiczne.
- Nie wolno zakladac, ze marketplace operator bedzie zawsze uczciwy.
- Nie wolno traktowac `RecipientPrivacyLite` jako darmowego zamiennika `FullPrivacy`.
- Nie wolno mylic `anonymous ticket` z pelna anonimowoscia bez threat modelu.

## 7. Minimalny pakiet, ktory powinien byc gotowy przed implementacja

Przed wejsciem w ciezsza implementacje chcemy miec:

1. `small_payments_rail_v0.md`
2. `service_payment_policy.md`
3. `ticket_id_and_nullifier.md`
4. `receipt_and_settlement_root.md`
5. `05_PRIVACY_TIERS.md`

## 8. Werdykt readiness

Na ten moment:

- mamy juz dobry kierunek strategiczny,
- mamy dobra polityke prywatnosci,
- mamy dobry argument ekonomiczny,
- ale nie mamy jeszcze zamrozonego:
  - modelu ticketow,
  - receipt schema,
  - service policy schema,
  - claim / redeem / refund semantics.

Czyli:

- jestesmy gotowi na **spec-first phase**
- nie jestesmy jeszcze gotowi na **heavy implementation phase**

To jest dobry moment, zeby napisac spec malego raila i dopiero potem wejsc w grubsze rzeczy.
