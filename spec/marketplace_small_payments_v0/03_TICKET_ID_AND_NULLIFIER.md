# Marketplace Small Payments v0

Status: migration source for final marketplace ticket semantics.
Canonicality: non-canonical. Ten dokument jest wazny tylko jako material do zlozenia finalnej spec.
Owner: privAI marketplace payments.
Depends on: `spec/PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`.
Superseded by: planowany `spec/PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`.

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
- nie zaakceptowal losowego nullifiera bez prawa wydania z depozytu,
- merchant nie mogl latwo reuzyc obcego ticketu,
- dwa zakupy tego samego usera nie byly latwo linkowalne.

## 5. Definicje robocze

### 5.1. Ticket

`Ticket` to lokalne, jednorazowe prawo do wykonania malego debitu w ramach lekkiego raila.

Ticket nie jest publicznym kontem.
Ticket nie jest globalnym identity tokenem.
Ticket nie jest dlugowiecznym session ID.

Ticket jest scisle `one-time`.
Nie wolno interpretowac go jako stalego kontenera do inkrementowanego usage billingu.

### 5.2. TicketSeed

`TicketSeed` to lokalny sekret raila, wyprowadzony po stronie walleta z:

- prywatnego depozytu / jego lokalnego contextu,
- tajnego materialu walleta,
- opcjonalnie dodatkowego rail salt.

`TicketSeed` nie trafia on-chain.

`TicketSeed` sam w sobie nie rozwiazuje jeszcze problemu konsensusowego.
W modelu finalnym musi byc jasno pokazane, skad bierze sie prawo do generowania ticketow i jak settlement odroznia prawdziwy debit od losowego nullifiera.

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

Sama unikalnosc nie wystarcza.
Model musi jeszcze odpowiedziec, skad bierze sie `deposit rights binding`.

### 5.5. TabSession / UsageContext

To jest osobny byt od ticketu.

Rola:

- wielokrotne eventy usage-metered,
- merchant tab,
- sesje wielu malych wywolan,
- lokalna lub marketplace'owa agregacja stanu.

Domyslna zasada:

- `ticket` sluzy do jednorazowego debitu,
- `tab/session` sluzy do wielokrotnego usage tracking,
- nie wolno mieszac tych dwoch semantyk w jednym identyfikatorze.

### 5.6. SessionGrant / SpendGrant

To jest jawny byt autoryzacyjny v0 po stronie marketplace operatora.

Rola:

- przydziela konkretnej sesji prawo do korzystania z depozytu,
- ustawia `spend_cap`,
- ustawia `expiry`,
- wiąze rail z `merchant_commit`,
- opcjonalnie wiąze rail z `service_commit`.

To nie jest publiczne konto usera.
To jest scoped grant wystawiany przez trusted marketplace authority dla konkretnej sesji.

### 5.7. SessionId

`SessionId` nie powinien byc budowany wprost z jawnej semantyki `DepositAnchor`.

Domyslny kierunek:

- `session_nonce`
- `merchant_commit`
- `grant_commit`
- `payer_ephemeral_pub`

Czyli sesja jest zwiazana z grantem i merchantem, ale nie daje merchantowi surowego uchwytu do depozytu.

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

### D6. Deposit binding musi byc jawny

Domyslna decyzja:

- nie wystarczy sam `ticket_nullifier`,
- model musi jawnie opisac, jak debit jest powiazany z prawem wydania z depozytu.

Domyslny model v0:

- `marketplace-operator-trusted accounting`

To znaczy:

- nie kazdy merchant samodzielnie interpretuje prawa z depozytu,
- marketplace operator wystawia `SessionGrant/SpendGrant`,
- merchant konsumuje grant w ramach sesji,
- operator publikuje finalny settlement albo jest jednoznaczna authority dla redeem path.

Alternatywa:

- `anchor-bound cryptographic model`
  - debit / settlement daje chainowi kryptograficznie sprawdzalny zwiazek z `DepositAnchor`
  - to jest mocniejsza sciezka trust-minimized, ale nie default v0.

W odpowiedzi wolno odrzucic model operator-trusted tylko jesli podana zostanie lepsza, spojna i wykonalna architektura v0.

### D7. `purchase_commit` pozostaje obowiazkowy

Domyslna decyzja:

- `purchase_commit` zostaje,
- nie wolno go wyrzucac tylko dlatego, ze mamy jednorazowy ticket.

Powod:

- porzadkuje refund path,
- porzadkuje dispute path,
- porzadkuje receipt linkage,
- rozdziela sam debit od biznesowego sensu zakupu.

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

Masz tez jawnie odpowiedziec:

7. jak to wyprowadzenie laczy sie z prawem z depozytu,
8. jak `SessionGrant/SpendGrant` wchodzi w model generacji lub autoryzacji,
9. czy ten model wymaga trusted marketplace accounting, czy daje chainowi cryptographic anchor binding.

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

Masz tez rozdzielic:

- scope ticketu
- scope tab/session context

To nie musza byc identyczne zakresy.

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

`purchase_commit` traktuj jako obowiazkowy element analizy, nie opcjonalny detal.

## 10. Wymagany model replay protection

## 10.1. Wymagany model deposit binding

To jest sekcja krytyczna.

Masz opisac, jak chain lub settlement layer odroznia:

- prawdziwy debit wynikajacy z depozytu
- od losowego, samowolnie wymyslonego `ticket_nullifier`.

Musisz wybrac i uzasadnic:

### Model A. Marketplace-operator-trusted accounting

- marketplace operator prowadzi accounting off-chain,
- merchant nie jest samodzielna authority dla praw z depozytu,
- operator wystawia `SessionGrant/SpendGrant`,
- chain ufa settlementowi / redeemowi publikowanemu przez operatora lub przez scisle zdefiniowana delegated authority,
- `ticket_nullifier` zabezpiecza jednorazowosc,
- depozyt i saldo sa pilnowane w trusted accounting layer.

### Model B. Anchor-bound cryptographic settlement

- settlement daje chainowi kryptograficzny zwiazek z `DepositAnchor`,
- chain nie ufa tylko accounting layer,
- jest mocniej trust-minimized,
- ale moze byc ciezsze i bardziej zlozone dla v0.

Domyslny wybor dla v0:

- `Marketplace-operator-trusted accounting`

Nie wolno pominac tego wyboru ani rozmywac go do hasla "merchant sprawdza depozyt".

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

Masz tez jawnie opisac:

- dlaczego replay protection nie moze byc jednoczesnie modelem incremental billing,
- gdzie konczy sie odpowiedzialnosc `ticket`,
- gdzie zaczyna sie odpowiedzialnosc `tab/session`.

Masz tez odpowiedziec:

- kto publikuje finalny settlement w v0,
- czy merchant publikuje go sam,
- czy publikuje go marketplace operator,
- czy merchant moze publikowac go tylko jako delegated actor w ramach operator grant model.

## 11. Wymagany model recovery i lifecycle

Masz opisac:

- jak wyglada pula ticketow,
- kiedy sa generowane,
- kiedy sa uznawane za zuzyte,
- kiedy sa uznawane za stale,
- co robi wallet po timeout,
- jak odzyskac stan po utracie urzadzenia,
- jak uniknac lokalnych kolizji przy wielu urzadzeniach.

Masz tez jawnie odpowiedziec:

- czy wiele urzadzen moze legalnie korzystac z jednego `DepositAnchor`,
- czy potrzebny jest osobny device branch / sub-seed,
- jak uniknac kolizji licznikow.

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
9. Jak dokladnie debit jest powiazany z prawem z depozytu?
10. Czy v0 wybiera `marketplace-operator-trusted accounting`, czy `anchor-bound cryptographic settlement`?
11. Gdzie konczy sie `ticket`, a gdzie zaczyna `tab/session`?
12. Jak wyglada `SessionGrant/SpendGrant` i kto jest jego authority?
13. Kto publikuje finalny settlement w v0?

## 14. Czego nie wolno proponowac

Nie proponuj:

- stalego publicznego `payer_id`
- stalego publicznego `account_id`
- stalego `rail_id` widocznego on-chain
- prostego numerowanego session ID, ktory da sie linkowac przez miesiac
- replay protection opartego tylko na merchant-side cache
- modelu, w ktorym chain nie ma zadnego twardego jednorazowego markera
- modelu, w ktorym ten sam ticket reprezentuje rosnacy rachunek usage-metered
- modelu, w ktorym `purchase_commit` znika i zostaje tylko `amount + nullifier`

## 15. Wymagany format odpowiedzi

Odpowiedz ma miec dokladnie te sekcje:

1. `Recommended v0 design`
2. `Alternative design`
3. `Data model`
4. `Public vs private fields`
5. `Deposit binding model`
6. `Replay protection model`
7. `Scope decision`
8. `Ticket vs tab separation`
9. `Recovery and lifecycle`
10. `Threat analysis`
11. `Frozen decisions`
12. `Open questions`

## 16. Oczekiwany wynik

Po przeczytaniu tego dokumentu i wykonaniu zadania mamy dostac:

- gotowa odpowiedz, czym jest `ticket`,
- gotowa odpowiedz, czym jest `ticket_nullifier`,
- gotowa decyzje o scope ticketu,
- gotowa odpowiedz, jak `ticket_nullifier` jest zwiazany z prawem z depozytu,
- gotowa odpowiedz, co trafia on-chain,
- gotowa odpowiedz, jak dziala replay protection,
- gotowa odpowiedz, jak odroznic `ticket` od `tab/session`,
- minimalna lista otwartych pytan, jesli cos naprawde musi zostac otwarte.

Nie chcemy po tej iteracji dostac kolejnego brainstormingu.
Chcemy dostac material, z ktorego da sie robic spec i implementacje.
