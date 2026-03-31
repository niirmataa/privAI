# Marketplace Small Payments v0

## Ticket ID And Nullifier

## 1. Cel

Ten dokument ma zamknac najwazniejszy filar lekkiego raila:

- jak modelujemy jednorazowe prawa pobrania,
- jak wyglada replay protection,
- jak zachowujemy accountless privacy,
- co jest lokalne, co publiczne, a co tylko commitmentem.

Ten dokument jest napisany tak, aby kolejny model lub wykonawca nie musial sam wymyslac brakujacych zalozen.

## 2. Co jest juz zamrozone i nie podlega renegocjacji w tym dokumencie

1. Rail jest `marketplace-only`, nie uniwersalny dla calego chaina.
2. Rail zaczyna sie od prywatnego depozytu.
3. Rail ma sluzyc drobnicy.
4. Dla drobnicy dopuszczalny jest `RecipientPrivacyLite`.
5. Nie wolno wprowadzac stalego publicznego konta platnika.
6. Nie wolno wprowadzac stalego publicznego rail ID.
7. Nie wolno projektowac slabszej warstwy PQ.
8. Escrow i duze kwoty pozostaja poza tym dokumentem i dalej wymagaja `FullPrivacy`.

## 3. Czego ten dokument ma nie robic

Ten dokument nie ma:

- projektowac pelnego receipt root,
- projektowac settlement root,
- projektowac pricing policy,
- projektowac finalnego UX.

Ma zamknac tylko:

- ticket,
- ticket_id,
- ticket_nullifier,
- scope,
- replay model,
- recovery model,
- binding do merchant/service/purchase context.

## 4. Twardy problem do rozwiazania

Musimy umiec odpowiedziec, jak chain ma zaakceptowac lekki debit tak, zeby:

- nie widzial stalego konta usera,
- nie widzial stalego rail ID,
- nie zaakceptowal replay,
- nie zaakceptowal double-spend,
- merchant nie mogl latwo reuzyc obcego ticketu,
- dwa zakupy tego samego usera nie byly latwo linkowalne.

## 5. Definicje robocze

### 5.1. Ticket

`Ticket` to lokalne, jednorazowe prawo do wykonania malego debitu w ramach lekkiego raila.

Ticket nie jest publicznym kontem.
Ticket nie jest globalnym identity tokenem.
Ticket nie jest dlugowiecznym session ID.

### 5.2. TicketSeed

`TicketSeed` to lokalny sekret raila, wyprowadzony po stronie walleta z:

- prywatnego depozytu / jego lokalnego contextu,
- tajnego materialu walleta,
- opcjonalnie dodatkowego rail salt.

`TicketSeed` nie trafia on-chain.

### 5.3. TicketId

`TicketId` to lokalny albo pol-publiczny identyfikator biznesowy ticketu.

Rola:

- laczenie debitu z receipt,
- laczenie debitu z local wallet state,
- korelacja biznesowa po stronie merchanta / marketplace tam, gdzie to konieczne.

Domyslna zasada:

- `TicketId` nie jest glownym publicznym replay guard.

### 5.4. TicketNullifier

`TicketNullifier` to glowny publiczny marker jednorazowosci.

Rola:

- replay protection,
- double-spend protection,
- brak stalego publicznego identyfikatora platnika.

Domyslna zasada:

- chain pilnuje globalnej unikalnosci `TicketNullifier`.

## 6. Domyslne decyzje robocze do przyjecia, jesli nie ma lepszej kontrpropozycji

Te decyzje sa domyslne. Jesli proponujesz inna droge, musisz jawnie wykazac, ze jest lepsza.

### D1. Ticket nie pochodzi z ReceiveBundle

Ticket nie ma byc generowany z `ReceiveBundle`.

Powod:

- `ReceiveBundle` dotyczy ukrytego odbioru not,
- small-payments rail ma inna semantyke,
- mieszanie ticketow z bundle receipt path zbyt mocno scala dwie warstwy.

Domyslna decyzja:

- `TicketSeed` pochodzi z lokalnego rail context zwiazanego z depozytem,
- nie z one-time receiver bundle.

### D2. On-chain glownym markerem jest `ticket_nullifier`

Domyslna decyzja:

- on-chain glownym markerem replay protection jest `ticket_nullifier`,
- nie `ticket_id`.

### D3. Ticket ma byc co najmniej merchant-scoped

Domyslna decyzja:

- ticket nie jest marketplace-global bez scope,
- ticket jest co najmniej zwiazany z `merchant_commit`,
- opcjonalnie dodatkowo z `service_commit`.

### D4. Debit ma byc wiazany z purchase context

Domyslna decyzja:

- lekki debit powinien miec `purchase_commit`,
- sam `amount + nullifier` to za malo.

### D5. Lokalny seed musi umiec recovery

Domyslna decyzja:

- ticketi musza byc odtwarzalne z lokalnego seeda albo z backupu walleta,
- nie wolno opierac recovery tylko na pamieci procesu.

## 7. Wymagany model generacji

Masz pracowac z nastepujacym szkicem:

```text
rail_seed
  -> ticket_id_i
  -> ticket_nullifier_i
  -> ticket_auth_i
  -> optional merchant/service scoped derivation
```

Masz odpowiedziec:

1. jak najlepiej wyprowadzac te wartosci,
2. ktore z nich sa lokalne,
3. ktore z nich sa publikowane,
4. ktore z nich maja byc podpisywane,
5. jak uniknac linkowalnosci miedzy merchantami,
6. jak uniknac linkowalnosci miedzy sesjami.

## 8. Wymagany model scope

Masz ocenic i rozstrzygnac 3 warianty:

### Wariant A. Marketplace-wide ticket

Ticket dziala globalnie w calym marketplace.

Zalety:

- prostszy wallet UX,
- prostsza pula ticketow.

Ryzyka:

- silniejsza linkowalnosc,
- slabsze ograniczenie blast radius przy wycieku lub bledzie.

### Wariant B. Merchant-scoped ticket

Ticket dziala tylko dla jednego `merchant_commit`.

Zalety:

- lepsza izolacja,
- slabsza linkowalnosc miedzy merchantami,
- lepsza semantyka dla merchant tabs.

Ryzyka:

- wiecej lokalnego stanu,
- trudniejszy wallet selection.

### Wariant C. Service-scoped ticket

Ticket dziala tylko dla jednego `merchant_commit + service_commit`.

Zalety:

- najmniejszy blast radius,
- najlepsza precyzja polityki.

Ryzyka:

- jeszcze wiecej lokalnego stanu,
- najwyzsza zlozonosc.

Domyslna propozycja, od ktorej masz startowac:

- `merchant-scoped` jako v0 default,
- `service-scoped` jako opcja dla wrazliwych uslug,
- `marketplace-wide` tylko jesli istnieje bardzo mocny argument ekonomiczny i prywatnosciowy.

## 9. Wymagany model publicznych danych

Masz odpowiedziec, co konkretnie trafia on-chain przy lekkim debicie.

Na starcie przyjmij, ze publiczne sa co najmniej:

- `ticket_nullifier`
- `merchant_commit`
- `service_commit`
- `purchase_commit`
- `amount`

Masz ocenic:

- czy `ticket_id` powinien byc publiczny
- czy powinien byc tylko lokalny
- czy powinien byc widoczny merchantowi, ale nie chainowi

## 10. Wymagany model replay protection

Masz opisac konkretny mechanizm:

1. co sprawdza chain,
2. co sprawdza merchant,
3. co sprawdza wallet,
4. gdzie moze dojsc do replay,
5. gdzie moze dojsc do race condition,
6. jak unikamy podwojnego wykorzystania ticketu.

Przyjmij jako baseline:

- chain dba o globalna unikalnosc `ticket_nullifier`,
- merchant dba o zgodnosc `purchase_commit` i receipt context,
- wallet nie powinien reuzywac ticketow po sukcesie ani po niejednoznacznym timeout.

## 11. Wymagany model recovery i lifecycle

Masz opisac:

- jak wyglada pula ticketow,
- kiedy sa generowane,
- kiedy sa uznawane za zuzyte,
- kiedy sa uznawane za stale,
- co robi wallet po timeout,
- jak odzyskac stan po utracie urzadzenia,
- jak uniknac lokalnych kolizji przy wielu urzadzeniach.

## 12. Wymagany model auth

Ticket nie moze byc tylko numerem.

Masz zaproponowac:

- jak wallet autoryzuje debit,
- czy potrzebny jest `ticket_auth_i`,
- czy auth jest merchant-bound,
- czy auth jest purchase-bound,
- czy auth powinien byc sprawdzalny przez chain, czy tylko przez merchant / settlement layer.

## 13. Pytania, ktore musza dostac odpowiedz

Nie wolno zostawic tych pytan bez odpowiedzi:

1. Czy `ticket_id` ma byc publiczny, czy tylko lokalny?
2. Czy `ticket_nullifier` jest samowystarczalny jako replay guard?
3. Czy ticket ma byc merchant-scoped czy service-scoped w v0?
4. Czy potrzebny jest `purchase_commit` w kazdym debicie?
5. Czy ticket ma miec expiry?
6. Czy ticketi niewykorzystane po expiry wracaja do puli, czy sa spalane logicznie?
7. Czy wallet moze generowac ticketi dla wielu merchantow z jednego rail seed?
8. Jak wyglada recovery, jesli user ma dwa urzadzenia?

## 14. Czego nie wolno proponowac

Nie proponuj:

- stalego publicznego `payer_id`
- stalego publicznego `account_id`
- stalego `rail_id` widocznego on-chain
- prostego numerowanego session ID, ktory da sie linkowac przez miesiac
- replay protection opartego tylko na merchant-side cache
- modelu, w ktorym chain nie ma zadnego twardego jednorazowego markera

## 15. Wymagany format odpowiedzi

Odpowiedz ma miec dokladnie te sekcje:

1. `Recommended v0 design`
2. `Alternative design`
3. `Data model`
4. `Public vs private fields`
5. `Replay protection model`
6. `Scope decision`
7. `Recovery and lifecycle`
8. `Threat analysis`
9. `Frozen decisions`
10. `Open questions`

## 16. Oczekiwany wynik

Po przeczytaniu tego dokumentu i wykonaniu zadania mamy dostac:

- gotowa odpowiedz, czym jest `ticket`,
- gotowa odpowiedz, czym jest `ticket_nullifier`,
- gotowa decyzje o scope ticketu,
- gotowa odpowiedz, co trafia on-chain,
- gotowa odpowiedz, jak dziala replay protection,
- minimalna lista otwartych pytan, jesli cos naprawde musi zostac otwarte.

Nie chcemy po tej iteracji dostac kolejnego brainstormingu.
Chcemy dostac material, z ktorego da sie robic spec i implementacje.
