# privAI Escrow V1 FullPrivacy Boundary Decision Memo

Status: discussion memo for consensus-building around escrow v1 and the `FullPrivacy` enforcement boundary.
Canonicality: non-canonical decision-support document. This file does not override protocol, formats, consensus, proof or product semantics. It exists to clarify the current escrow v1 blocker, preserve alignment with accepted source-of-truth docs, and compare execution-level options before any canonical freeze update.
Owner: shared architecture / ledger / proof / integration review.
Audience: human owner, implementation chats, Claude, Gemini.
Depends on:
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_AUTH_SIGNING_MODEL.md`
- `spec/PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`
- `spec/PRIVAI_ESCROW_FINAL_MODEL.md`
- `spec/PRIVAI_ESCROW_TX_MATRIX.md`
- `spec/PRIVAI_ESCROW_PROOF_INTEGRATION.md`
- `spec/PRIVAI_ESCROW_2OF3_ADAPTATION.md`
- `spec/NEXUM_CLI_INTEGRATION_PROPOSAL.md`

## 1. Purpose

Ten dokument istnieje po to, zeby:
- odciac drift architektoniczny wokol escrow v1,
- rozdzielic frozen source of truth od roboczych propozycji,
- nazwac realny blocker odkryty w kodzie,
- porownac mozliwe rozwiazania,
- wskazac obecnego faworyta do konsensusu bez psucia filaru `FullPrivacy`.

To nie jest:
- nowa canonical spec,
- freeze update,
- binary format spec,
- nowy proof system.

## 2. Frozen Facts

Ponizsze punkty sa juz zamrozone lub wystarczajaco stabilne, by traktowac je jako source of truth:

- `FullPrivacy` pozostaje glownym finalnym prywatnym railem dla flow wrazliwych i wyzszych kwot.
- Escrow 2-of-3 nalezy do `FullPrivacy`.
- Escrow 2-of-3 nie nalezy do `MarketplaceSmallPaymentsRail`.
- Escrow v1 jest modelem **mixed**:
  - proof coveruje note-level semantics,
  - ledger/auth coveruje policy/action/threshold/timeout semantics.
- Podpisy i approvals ida po `tx_signing_hash`, nie po `tx_id`.
- `policy_opening` pozostaje zrodlem rekonstrukcji policy.
- High-level escrow intent jest zamrozony:
  - `Buyer + Operator` => `release` do Merchanta,
  - `Merchant + Operator` => `refund` do Buyera,
  - `Buyer + Merchant` => `recovery_release`,
  - recovery po timeout.

## 3. What Core `FullPrivacy` Already Solved Well

Core `FullPrivacy` nie bylo bledem. Privacy-first foundation jest sensowna:

- nota niesie `spend_policy_commit`, a nie jawny typ policy,
- szczegoly policy sa ukryte za commitmentem,
- `policy_opening` sluzy do rekonstrukcji przy spendzie,
- proof i ledger sa rozdzielone,
- proof nie udaje auth/policy enforcement.

Model docelowy jest zdrowy:
- jawne reguly systemu,
- prywatne wnetrze uczestnictwa.

## 4. The Real Blocker

Problem pojawia sie na granicy:
- opaque `spend_policy_commit`,
- optional auth w obecnym execution path,
- escrow, ktore wymaga twardego auth enforcementu.

Jesli:
- `spend_policy_commit` jest tylko opaque hashem,
- a transakcja moze przyjsc z pustym `auth`,

to ledger nie umie rozroznic:
- zwyklego private spend bez auth,
- od escrow-governed inputu bez auth.

To jest prawdziwy blocker:
- ledger nie potrafi twardo wymusic escrow auth w empty-auth case,
- jesli nie ma dodatkowej informacji o klasie enforcementu albo innego execution contract.

## 5. Important Scope Correction

Nie jest zamrozone, ze:
- caly `FullPrivacy` musi nagle wymagac auth dla kazdego inputu,
- mamy publicznie ujawnic `policy_tag = Escrow`,
- mamy redefiniowac caly rail tylko po to, by domknac escrow blocker.

Te pomysly padly jako robocze propozycje podczas brainstormingu.
Nie wolno ich mylic z accepted source of truth.

## 6. Operator Model

Operator w escrow v1:
- nie jest autorytetem systemowym,
- nie jest zewnetrznym trust anchorem,
- nie jest Monero-style execution engine,
- nie jest zrodlem prawdy protokolu.

Operator jest:
- zautomatyzowanym programem systemowym,
- deterministycznym egzekutorem regul flow,
- obowiazkowym wspolsygnatariuszem normal mode,
- elementem liveness i kontroli wykonania.

Rozdzial odpowiedzialnosci:
- kryptografia odpowiada za bezpieczenstwo matematyczne,
- ledger odpowiada za finalna walidacje protokolowa,
- operator odpowiada za wykonanie workflow zgodnie z regulami i zabezpieczeniami operacyjnymi.

Twarde zasady:
- operator nie moze jednostronnie wydac srodkow,
- operator nie moze zmienic policy ani destination constraints,
- operator nie moze nadac waznosci transakcji sprzecznej z ledger rules,
- operator moze dolaczyc wymagany podpis normal mode albo odmowic,
- recovery path po timeout istnieje po to, by operator nie byl single point of permanent lock.

## 7. Minimal-Knowledge Operator Flow

`nexum-core` i operator pasuja jako workflow/control-plane:
- operator dostaje przez `nxms-transport` / mailbox minimalny event operacyjny,
- np. ze funding note zostala wykryta i srodki sa zablokowane,
- operator nie potrzebuje dostepu do prywatnego wnetrza uczestnikow ani do danych, ktore mozna wykorzystac do profilowania.

Operator moze widziec tylko minimum potrzebne do wykonania flow, np.:
- `escrow_id`,
- techniczny identyfikator funding note,
- status workflow,
- moment / height potrzebny do dalszej orkiestracji.

Ledger pozostaje zrodlem prawdy o locku i istnieniu noty.
Mailbox i `nexum-core` sa kanalem workflow, nie zrodlem ledger truth.

## 8. Openness vs Privacy

privAI nie opiera bezpieczenstwa na ukrywaniu kodu ani regul systemu.

Kod moze byc otwarty.
Reguly protokolu moga byc jawne.
Audyty, testy i ograniczenia systemu moga byc publiczne.

To, co ma pozostac chronione, to:
- tozsamosci uzytkownikow,
- relacje miedzy stronami,
- szczegoly przeplywu srodkow,
- prywatne dane auth i policy tam, gdzie nie musza byc publiczne,
- prywatne wnetrze uczestnictwa.

Model docelowy:
- jawny system,
- nieobserwowalne prywatne wnetrze ponad to, co protokol jawnie ujawnia.

## 9. Design Options

### Option A. Public `policy_tag/version`

Opis:
- nota lub publiczny stan niesie jawnie klase policy,
- np. `Single`, `Escrow2of3`, `MarketplaceSettlement`.

Zalety:
- najprostszy enforcement,
- najmniejszy koszt runtime.

Wady:
- publiczny leak klasy policy,
- obserwator widzi, ze nota jest escrow.

Ocena:
- mocne wydajnosciowo,
- slabe privacy-wise,
- slabo pasuje do ducha core `FullPrivacy`.

### Option B. Auth for all `FullPrivacy` inputs

Opis:
- kazdy spend input na railu `FullPrivacy` musi miec auth envelope.

Zalety:
- bardzo mocne privacy,
- brak publicznego leak klasy policy,
- ledger zawsze ma material do rozstrzygniecia.

Wady:
- zmienia semantyke calego `FullPrivacy`,
- podnosi rozmiar tx i koszt walidacji,
- wychodzi poza frozen scope escrow v1.

Ocena:
- mocne privacy-wise,
- ale za szerokie scope.

### Option C. Escrow-specific auth-bound spend path/version

Opis:
- escrow nie idzie przez stary optional-auth transfer path,
- tylko przez auth-bound private spend path / tx version / envelope contract,
- auth jest wymagany per input,
- typ policy nadal nie jest ujawniany publicznie w nocie.

Zalety:
- zachowuje privacy core note modelu,
- nie wymaga publicznego `Escrow` tagu,
- nie redefiniuje calego `FullPrivacy`.

Wady:
- wymaga nowego execution contract / tx path / versioning decision,
- nie jest juz tylko mala poprawka w ledgerze.

Ocena:
- bardzo dobry kompromis,
- ale wymaga wiekszej zmiany execution modelu.

### Option D. Wallet/runtime-only enforcement

Opis:
- zakladamy, ze runtime zawsze dolacza auth dla escrow,
- ledger nie ma niezaleznego sposobu wykrycia empty-auth escrow input.

Ocena:
- nieakceptowalne jako final closure.

### Option E. Generic Auth-Required Enforcement Class

Opis:
- nota publicznie nie ujawnia biznesowej klasy policy (`Escrow`, `Single`, `MarketplaceSettlement`),
- ale ujawnia minimalna klase enforcementu, np.:
  - `open`
  - `auth_required`
- ledger wie tylko, ze dany input jest policy-gated i wymaga auth envelope przy spendzie,
- dopiero `policy_opening` rozstrzyga, czy to dokladnie escrow 2-of-3, czy inna policy auth-bound.

Zalety:
- zamyka empty-auth blocker,
- nie ujawnia publicznie, ze nota jest escrow,
- nie redefiniuje calego `FullPrivacy`,
- daje ledgerowi minimalny, deterministyczny sygnal enforcementowy,
- zachowuje private policy body.

Wady:
- nadal ujawnia, ze nota jest auth-required / policy-gated,
- wymaga rozszerzenia note payload / canonical formats / state semantics,
- jest to jawny freeze candidate, a nie tylko lokalna poprawka.

Ocena:
- najlepszy obecny kompromis privacy vs wydajnosc vs frozen scope.

## 10. Why Option E Currently Wins

Option E eliminuje dwa toksyczne skrajnosci:
- nie robi publicznego `Escrow` tagu,
- nie rozlewa mandatory-auth na caly `FullPrivacy`.

Publicznie ujawnia tylko:
- "ten UTXO ma nalozony zamek behawioralny",
a nie:
- jaki to dokladnie zamek,
- czy to escrow,
- kto jest signerem,
- jaki jest policy body.

To daje ledgerowi dokladnie tyle informacji, ile potrzebuje do twardego odrzucenia empty-auth case.

## 11. Technical Mapping For Option E

### 11.1. Funding

Przy tworzeniu note:
- ustawiamy minimalna klase enforcementu, np. `auth_required`,
- ta klasa wchodzi do canonical payloadu i do `note_commit`,
- zmiana klasy po fakcie uniewaznia binding.

### 11.2. Spend

Ledger:
1. podnosi note ze stanu,
2. odczytuje enforcement class,
3. jesli klasa to `auth_required`, wymaga:
   - auth envelope,
   - `policy_opening`,
4. hashuje `policy_opening` i sprawdza zgodnosc z `spend_policy_commit`,
5. dopiero potem rozstrzyga:
   - jaka jest policy,
   - jaki signer set,
   - jaki threshold,
   - jaka action semantics.

### 11.3. Effect

Ledger dostaje:
- deterministyczny sygnal "auth required",
- bez publicznego leak "this is escrow".

## 12. Eventing For Operator Under Option E

Minimalny flow dla operatora:
1. Buyer funduje note on-chain.
2. Rownolegle lub po potwierdzeniu funding tx idzie event przez `nxms-transport` / mailbox.
3. Operator dostaje zaszyfrowany `EscrowFundingDescriptor`.
4. `nexum-core` nie ufa eventowi w ciemno:
   - sprawdza na swoim `LedgerSnapshot`, czy funding note istnieje,
   - czy jest unspent,
   - czy `spend_policy_commit` pasuje do deskryptora.
5. Dopiero wtedy workflow przechodzi do "Funding Confirmed".

To odcina:
- falszywe eventy,
- replay trigger,
- mailbox-only fake orchestration.

## 13. Open Questions For Consensus

1. Czy akceptujemy publiczna klase enforcementu jako minimalny leak?
2. Czy nazewnictwo ma byc:
   - `open` / `auth_required`,
   - czy inne neutralne sformulowanie?
3. Czy enforcement class jest:
   - polem payloadu,
   - flaga,
   - czy osobnym canonical enum?
4. Czy Option E ma byc:
   - explicit update do `PRIVAI_CANONICAL_FORMATS.md`,
   - explicit update do `PRIVAI_PROTOCOL_CORE.md`,
   - explicit ledger freeze rule?
5. Czy Option E w v1 obejmuje tylko escrow, czy otwiera droge dla innych auth-bound private policies?

## 14. Recommendation

Obecna rekomendacja do konsensusu:

**Option E: Generic Auth-Required Enforcement Class**

z nastepujacym framingiem:
- to nie jest biznesowy `Escrow` tag,
- to nie jest publiczna klasyfikacja policy body,
- to jest minimalna klasa enforcementu potrzebna ledgerowi,
- private policy body pozostaje ukryte za `spend_policy_commit`,
- escrow v1 zachowuje `FullPrivacy` foundation i dostaje twardy enforcement.

## 15. Bottom Line

Core `FullPrivacy` nie bylo bledem.
Operator-as-automation jest zgodny z zamrozonym kierunkiem.
`nexum-core` jest bardzo dobrym workflow engine dla escrow.

Realny problem pozostaje execution-level:
- jak ledger ma odrzucic escrow-governed input bez auth,
- nie wiedzac publicznie, ze to escrow.

Option E daje obecnie najlepszy kompromis:
- minimalny publiczny sygnal enforcementowy,
- bez publicznego leak klasy escrow,
- bez przedefiniowania calego `FullPrivacy`,
- bez udawania, ze current optional-auth path juz wystarcza.
