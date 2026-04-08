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
- mamy publicznie ujawnic `policy_tag = Escrow`,
- mamy redefiniowac caly rail tylko po to, by domknac escrow blocker.

Te pomysly padly jako robocze propozycje podczas brainstormingu.
Nie wolno ich mylic z accepted source of truth.

Doprecyzowanie:
- rozszerzenie mandatory-auth na caly `FullPrivacy` nie bylo jeszcze jawnie zamrozone w source of truth,
- ale moze byc uczciwie potraktowane jako **v1 maturity step** dla tego raila, jesli uznamy, ze optional-auth path byl tylko prototypowym dlugiem v0, a nie finalna semantyka prywatnego toru.

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
- wszystkie spends w `FullPrivacy` wygladaja z zewnatrz jednolicie,
- nie wymaga nowych pol w nocie ani nowej publicznej klasy enforcementu,
- nie wymaga zmian w canonical note payload tylko po to, by rozroznic auth-required vs open.

Wady:
- podnosi rozmiar tx i koszt walidacji dla wszystkich spends na `FullPrivacy`,
- wymaga potraktowania optional-auth path jako prototypowego dlugu v0, a nie cechy finalnej,
- wymaga aktualizacji testow i execution assumptions dla prywatnego raila.

Ocena:
- obecnie najlepsza opcja privacy-wise,
- najczystsza semantycznie, jesli `FullPrivacy` ma byc traktowane dojrzale i jednolicie.

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
- dopoki jedyna auth-required policy jest escrow, tworzy de facto distinguishability leak dla escrow-like notes.

Ocena:
- bardzo dobry fallback / bridge option,
- ale slabszy privacy-wise od Option B.

## 10. Why Option B Currently Wins

Po dodatkowej analizie privacy leak i implementation footprint Option B awansuje na miejsce pierwsze.

Powody:
- daje jeden, silny anonymity set dla spends na `FullPrivacy`,
- nie tworzy publicznie rozroznialnych podklas `open` vs `auth_required`,
- nie wymaga nowego canonical field / flagi w nocie,
- nie wymaga bindingu enforcement class do `note_commit`,
- nie zwieksza canonical surface area note modelu,
- nie tworzy ukrytego publicznego `Escrow` tagu pod neutralna nazwa.

Kluczowa obserwacja:
- jesli tylko escrow uzywa `auth_required`, to Option E publicznie odroznia escrow-like notes od reszty,
- a to jest sprzeczne z celem maksymalnej jednolitosci `FullPrivacy`.

Option B jest kosztowniejsza runtime'owo, ale ten koszt jest w praktyce kosztem prawdziwego auth w `FullPrivacy`, a nie tylko kosztem escrow.

## 11. Technical Mapping For Option B

### 11.1. Funding

Funding note pozostaje zgodny z privacy-first note modelem:
- brak nowego publicznego enforcement bitu,
- brak nowego publicznego `Escrow` tagu,
- policy body pozostaje ukryte za `spend_policy_commit`.

### 11.2. Spend

Ledger dla spends na `FullPrivacy`:
1. wymaga auth envelope dla kazdego inputu,
2. wymaga `policy_opening`,
3. hashuje `policy_opening` i sprawdza zgodnosc z `spend_policy_commit`,
4. odtwarza `tx_signing_hash`,
5. weryfikuje auth package wzgledem reconstructed policy.

### 11.3. Effect

Ledger:
- nie musi zgadywac, czy dany input wymaga auth,
- nie musi rozrozniac publicznie klasy policy,
- zawsze dostaje material do rozstrzygniecia ownership + policy constraints.

## 12. Option E As Fallback / Bridge

Option E pozostaje sensowna tylko jako:
- fallback migracyjny,
- bridge option,
- kompromis, jesli z jakiegos powodu chcemy zachowac optional-auth path na czesc prywatnych use cases.

Nie jest juz rekomendacja pierwszego wyboru.

## 13. Eventing For Operator Under Option B

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

## 14. Open Questions For Consensus

1. Czy formalnie uznajemy optional-auth path w `FullPrivacy` za prototypowy dlug v0?
2. Czy `FullPrivacy v1` ma wprost znaczyc: "all inputs require auth envelope + policy_opening"?
3. Czy aktualizujemy:
   - `PRIVAI_AUTH_SIGNING_MODEL.md`
   - `PRIVAI_PROTOCOL_CORE.md`
   - `PRIVAI_CANONICAL_FORMATS.md`
   tak, by to zapisac juz nie jako brainstorm, ale jako freeze candidate?
4. Czy Option E zostawiamy jawnie jako fallback/bridge note, czy usuwamy z dalszej rekomendacji?
5. Jak oznaczamy etap migracji testow i implementacji z optional-auth prototype do auth-required `FullPrivacy v1`?

## 15. Recommendation

Obecna rekomendacja do konsensusu:

**Option B: Auth for all `FullPrivacy` inputs**

z nastepujacym framingiem:
- to nie jest redesign calego systemu,
- to jest v1 maturity step dla `FullPrivacy`,
- optional-auth path nalezy traktowac jako prototypowy dlug v0,
- private policy body pozostaje ukryte za `spend_policy_commit`,
- escrow v1 dostaje twardy enforcement bez publicznego rozrozniania klas not.

Option E:
- pozostaje solidna opcja techniczna,
- ale jako fallback / bridge,
- nie jako rekomendacja pierwszego wyboru.

## 16. Bottom Line

Core `FullPrivacy` nie bylo bledem.
Operator-as-automation jest zgodny z zamrozonym kierunkiem.
`nexum-core` jest bardzo dobrym workflow engine dla escrow.

Realny problem pozostaje execution-level:
- jak ledger ma odrzucic escrow-governed input bez auth,
- nie wiedzac publicznie, ze to escrow.

Option B daje obecnie najlepszy wynik:
- najmocniejsza prywatnosc,
- jednolity wyglad spends na `FullPrivacy`,
- brak nowego publicznego bitu enforcementowego,
- brak publicznego distinguishability leak dla escrow-like notes,
- pelne zamkniecie empty-auth blockeru.
