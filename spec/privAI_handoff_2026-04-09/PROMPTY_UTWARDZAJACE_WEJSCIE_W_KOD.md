# Prompty Utwardzajace Wejscie w Kod

## Cel

Ten dokument zawiera 6 mini-promptow kontrolnych, ktore stosuje sie
**po zakonczeniu onboardingu** (Prompty 1-6), a **przed wydaniem task pill**
agentowi wykonawczemu.

Ich zadaniem jest:
- zabezpieczyc model przed najczestszymi bledami code-read,
- wymusic jawne raportowanie co zostalo sprawdzone a co nie,
- uniemozliwic przejscie do wykonania na podstawie falszywych zalozen.

## Regula nadrzedna

**Mini-prompty utwardzajace nie sluza do odkrywania nowej architektury.**
**One sluza do weryfikacji i kalibracji przed wejsciem w task.**

Model po etapie 1 (Prompty 1-6) powinien juz miec poprawny mental model
systemu. Mini-prompty utwardzajace sprawdzaja, czy ten model jest spojny
z repo reality — nie buduja nowego obrazu od zera.

Jesli mini-prompt ujawni, ze model nie rozumie danej warstwy,
to nie jest moment na "douczenie sie w locie".
To jest moment na **zatrzymanie sie** i uruchomienie Correction pill.

## Kiedy stosowac

```
Etap 1: Prompty 1-6 (uczenie systemu)
         |
         v
Etap 2: Prompty utwardzajace (ten dokument)    <-- TUTAJ
         |
         v
Etap 3: Task pill dla agenta wykonawczego
```

Prompty utwardzajace stosuje sie:
- po zakonczeniu onboardingu, gdy model twierdzi ze rozumie system,
- przed pierwszym wejsciem w kod,
- przy kazdym nowym module lub warstwie, ktorej model jeszcze nie czytal,
- po kazdej dluzszej przerwie w sesji (re-entry).

Nie sa jednorazowe. Mozna je powtarzac przy kazdym nowym obszarze kodu.

## Zasada gate'owa

Model nie moze przejsc do budowania task pill, dopoki nie przejdzie
przez mini-prompty utwardzajace dla kazdej warstwy, ktora zamierza
uwzglednic w backlogu.

## Mapping: ktore mini-prompty sa obowiazkowe przy ktorej warstwie

Nie kazdy mini-prompt jest potrzebny przy kazdym wejsciu.
Ponizej obowiazkowy mapping:

### Wejscie w `chain` lub `ledger`
- **Obowiazkowe:** Mini-prompt 1 (Layer Ownership), Mini-prompt 3 (Cross-Module)
- **Zalecane:** Mini-prompt 4 (Test Coverage)
- Bo: chain definiuje reguly, ledger enforcuje — model musi zobaczyc OBE strony

### Wejscie w `node` (escrow_stage, submit gate)
- **Obowiazkowe:** Mini-prompt 1 (Layer Ownership), Mini-prompt 2 (Partial vs Absent), Mini-prompt 4 (Test Coverage)
- **Zalecane:** Mini-prompt 3 (Cross-Module)
- Bo: node jest najgestszy i ma najwiecej partiali — tu model najczesciej myli partial z absent

### Wejscie w `wallet` lub `proof`
- **Obowiazkowe:** Mini-prompt 2 (Partial vs Absent), Mini-prompt 3 (Cross-Module)
- **Zalecane:** Mini-prompt 4 (Test Coverage)
- Bo: wallet i proof sa stabilne ale nie sacred — model musi wiedziec co jest done a co moze wymagac narrow fix

### Wejscie w `transport` lub `mailbox`
- **Obowiazkowe:** Mini-prompt 2 (Partial vs Absent), Mini-prompt 5 (Dependency Reality)
- **Zalecane:** Mini-prompt 1 (Layer Ownership)
- Bo: mailbox jest partial-but-real i model czesto traktuje go jako hard blocker dla wszystkiego

### Budowanie backlogu / task pill
- **Obowiazkowe:** Mini-prompt 5 (Dependency Reality), Mini-prompt 6 (Pre-Task Readiness Gate)
- **Zalecane:** wszystkie poprzednie dla kazdego modulu w scope
- Bo: backlog z falszywymi dependency lub niezweryfikowanymi zalozeniami jest gorszy niz brak backlogu

### Mapping w tabeli

| Warstwa         | MP1 Layer | MP2 Partial | MP3 Cross | MP4 Tests | MP5 Deps | MP6 Gate |
|-----------------|-----------|-------------|-----------|-----------|----------|----------|
| chain / ledger  | **REQUIRED** | zalecany | **REQUIRED** | zalecany | -     | -        |
| node            | **REQUIRED** | **REQUIRED** | zalecany | **REQUIRED** | -  | -        |
| wallet / proof  | -         | **REQUIRED** | **REQUIRED** | zalecany | -     | -        |
| transport/mail  | zalecany  | **REQUIRED** | -         | -         | **REQUIRED** | -   |
| backlog / task  | -         | -           | -         | -         | **REQUIRED** | **REQUIRED** |


---

## Mini-prompt 1: Layer Ownership Check

### Kiedy uzywac
Zanim model powie "tego nie ma w systemie" albo "to jest gap".

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- nie potrafisz wskazac naturalnego ownera danej semantyki,
- znalazles element w innym module niz oczekiwales (to znaczy ze Twoj model ownershipa jest bledny),
- wiecej niz 2 elementy maja status `unchecked` — to znaczy ze wchodzisz w kod za wczesnie.

### Prompt

```text
Dla kazdego elementu, ktory chcesz oznaczyc jako "brak" albo "gap":

1. W ktorym module szukales?
2. Kto jest naturalnym ownerem tej semantyki?
   (chain / ledger / node / wallet / proof / transport / orchestrator)
3. Czy sprawdziles modul ownera?
4. Czy sprawdziles sasiedni modul odpowiedzialnosci?
5. Czy masz dowod na brak? (grep, test file, explicit code path)

Odpowiedz w formacie:

| Element | Szukany w | Naturalny owner | Owner sprawdzony | Sasiad sprawdzony | Dowod na brak | Evidence source | Status | Unchecked assumptions |
|---------|-----------|-----------------|------------------|-------------------|---------------|-----------------|--------|-----------------------|
| ...     | ...       | ...             | tak/nie          | tak/nie           | grep/test/brak| docs/code/tests/grep/inferred | confirmed-absent / unchecked | co jeszcze nie zostalo sprawdzone |

Jesli kolumna "Evidence source" = "inferred" lub "Unchecked assumptions" jest niepusta,
to element NIE MOZE miec statusu "confirmed-absent". Tylko "unchecked" lub "inferred-absent".
```

### Correction pill link
Jesli ten mini-prompt ujawnil blad w Twoim modelu (np. owner jest inny niz zakladales),
**nie kombinuj** — uruchom Correction pill z PROMPTYZACJA_SYSTEMOWEGO_SENIORA.md:
```
KOREKTA:
- Twierdziles: [co model powiedzial]
- Prawda: [co jest faktycznie]
- Dowod: [plik + test + konkret]
- Co sie zmienia: [jak zmienia sie status / backlog / reading path]
- Kontynuuj od: [nastepny krok]
```

### Dlaczego to jest wazne

W pierwszej sesji onboardingowej model (Opus) szukal timeout enforcement
w `node.rs` (submit gate), nie znalazl, i oglosil:
"timeout enforcement nigdzie nie jest zaimplementowany".

Timeout enforcement istnieje w `privai-ledger/src/escrow.rs` z pelnym
zestawem testow i jest podlaczony do block import path.

Gdyby model musial wypelnic te tabele, sam by zobaczyl:
- Szukany w: node
- Naturalny owner: ledger (bo to enforcement rule)
- Owner sprawdzony: nie
- Evidence source: inferred
- Unchecked assumptions: nie sprawdzono ledger/escrow.rs
- Status: unchecked (nie confirmed-absent)

I nie oglosilby braku.

### Zielone flagi
- model jawnie nazywa ownera semantyki zanim szuka
- model przyznaje "nie sprawdzilem ownera" zamiast ogłaszac brak
- model robi grep przed deklaracja
- kolumna Unchecked assumptions jest wypelniona uczciwie

### Czerwone flagi
- model mowi "tego nie ma" po przeczytaniu jednego pliku
- model nie potrafi wskazac naturalnego ownera
- model pomija ledger/chain jako potencjalnego ownera enforcement rules
- kolumna Evidence source = "inferred" ale Status = "confirmed-absent"


---

## Mini-prompt 2: Partial vs Absent Classifier

### Kiedy uzywac
Gdy model ma opisac stan implementacji danego feature'a lub modulu.

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- element ktory oznaczyłes jako "absent" okazuje sie miec kod (struct, trait, fn) — Twoj absent jest bledny,
- nie potrafisz rozroznic miedzy "nigdy nie istnialo" a "istnieje ale niekompletne",
- wiecej niz polowa elementow ma Evidence source = "inferred" — potrzebujesz wiecej code-read, nie wiecej wnioskow.

### Prompt

```text
Dla kazdego elementu, ktory opisujesz jako "partial" albo "absent":

1. Co dokladnie istnieje w kodzie? (struct, funkcja, trait, test)
2. Co dokladnie brakuje? (brak wywolania, brak testu, brak integracji)
3. Czy to jest:
   - code-confirmed (widzialem kod, wiem ze istnieje)
   - test-confirmed (istnieje test, ktory to pokrywa)
   - inferred (wynika z kodu, ale nie ma dedykowanego testu)
   - unchecked (nie sprawdzilem jeszcze)
4. Jezeli "absent" — czy to znaczy:
   - nigdy nie istnialo (greenfield)
   - istnieje fundament, brakuje integracji (partial)
   - istnieje w innym module niz szukalem (misattributed)

Odpowiedz w formacie:

| Element | Co istnieje | Co brakuje | Evidence source | Typ absent | Pewnosc | Unchecked assumptions |
|---------|-------------|------------|-----------------|------------|---------|----------------------|
| ...     | ...         | ...        | code/tests/grep/inferred/unchecked | greenfield/partial/misattributed/n-a | wysoka/srednia/niska | co nie zostalo sprawdzone |

Jesli "Evidence source" = "unchecked", to element NIE MOZE byc klasyfikowany
jako "greenfield". Moze byc tylko "unchecked — do zweryfikowania".
```

### Correction pill link
Jesli ten mini-prompt ujawnil ze cos co oznaczyłes jako absent jest partial,
uruchom Correction pill. Format:
```
KOREKTA:
- Twierdziles: [element] jest absent / greenfield
- Prawda: [element] jest partial — istnieje [co dokladnie]
- Dowod: [plik + linia + struct/fn name]
- Co sie zmienia: task scope sie kurczy z greenfield do narrow fix
- Kontynuuj od: przepisz scope tasku
```

### Dlaczego to jest wazne

Roznica miedzy "partial" a "absent" jest najczestszym i najgrozniejszym
bledem modelu. Ogłoszenie "absent" prowadzi do:
- niepotrzebnego greenfield tasku (praca juz istnieje),
- falszywego dependency (bo "trzeba najpierw zbudowac X"),
- zmarnowanych tokenow na reimplementacje.

Ogłoszenie "partial" wymusza pytanie "co dokladnie brakuje",
co prowadzi do waskego, precyzyjnego tasku zamiast redesignu.

### Przyklad z privAI

`mailbox_pull.rs` ma 693 linii kodu, 7 unit testow, pelna architekture
(trait MailboxSource, NxmsMailboxAdapter, pull/ingest/ack cycle).

Gdyby model powiedzial "mailbox absent" — ktos napisalby to od nowa.
Poprawny status: "partial — unit tests pass, production integration
unverified, NxmsMailboxAdapter.from_config() has kem_sk gap".

Evidence source: code-confirmed + test-confirmed.
Unchecked assumptions: production integration path not verified.

### Zielone flagi
- model rozroznia "nie ma kodu" od "jest kod ale brak integracji"
- model podaje konkretne nazwy struct/fn jako dowod istnienia
- model przyznaje "unchecked" zamiast zgadywac
- kolumna Unchecked assumptions jest niepusta

### Czerwone flagi
- model mowi "absent" bez podania co szukal i gdzie
- model nie rozroznia greenfield od partial
- model traktuje brak testu jako brak implementacji
- Evidence source = "unchecked" ale klasyfikacja = "greenfield"


---

## Mini-prompt 3: Cross-Module Consistency Check

### Kiedy uzywac
Gdy model przeszedl przez wiecej niz jeden modul i ma budowac obraz
calosciowy (np. przed backlogiem albo przed task pill).

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- typy miedzy modulami sie nie zgadzaja (inny enum, inny struct) — to moze byc real bug albo Twoj bledny odczyt,
- ownership danej semantyki jest niejasny miedzy dwoma modulami — to wymaga decyzji architektonicznej, nie zgadywania,
- znalazles gap miedzy modulami ktory zmienia backlog — nie idz dalej z dotychczasowym backlogiem.

### Prompt

```text
Sprawdz spojnosc miedzy modulami, ktore juz przeczytales.

Dla kazdej pary modulow, ktore wspolpracuja:

1. Czy typy sie zgadzaja? (ten sam enum, ten sam struct po obu stronach)
2. Czy semantyka jest spojma? (ten sam meaning dla tego samego pola)
3. Czy ownership jest jasny? (kto jest source of truth danej reguly)
4. Czy jest gap miedzy nimi? (modul A produkuje cos, czego modul B nie konsumuje)

Sprawdz minimum te pary:
- chain/escrow.rs <-> ledger/escrow.rs (rule table vs enforcement)
- node/escrow_stage.rs <-> wallet/escrow_builder.rs (Stage A -> Stage B handoff)
- wallet/escrow_builder.rs <-> wallet/proof_handoff.rs (assembly -> proof)
- node/node.rs <-> ledger (submit gate vs block import validation)

Odpowiedz w formacie:

| Para modulow | Typy spojne | Semantyka spojna | Ownership jasny | Gap | Evidence source | Pewnosc | Unchecked assumptions |
|-------------|-------------|------------------|-----------------|-----|-----------------|---------|----------------------|
| ...         | tak/nie/unchecked | tak/nie/unchecked | tak/nie | opis albo brak | code/tests/grep/inferred | wysoka/srednia/niska | co nie zostalo sprawdzone |

Jesli jakikolwiek wiersz ma Evidence source = "inferred" i Gap != "brak",
to MUSISZ to zweryfikowac kodem zanim przejdziesz dalej.
```

### Correction pill link
Jesli cross-check ujawnil niespojnosc:
```
KOREKTA:
- Twierdziles: [moduly X i Y sa spojne / niezalezne]
- Prawda: [jest konkretna niespojnosc / gap]
- Dowod: [typ/pole w module X vs typ/pole w module Y]
- Co sie zmienia: [nowy task albo zmiana ownershipa]
- Kontynuuj od: [rewizja backlogu uwzgledniajaca gap]
```

### Dlaczego to jest wazne

Moduly w privAI sa celowo rozdzielone na warstwy odpowiedzialnosci.
Ale to oznacza, ze model moze przeczytac jeden modul i zbudowac
obraz, ktory jest wewnetrznie spojny — ale niezgodny z sasiadem.

Przyklad: `chain/escrow.rs` definiuje `required_signers(action)`,
a `ledger/escrow.rs` to konsumuje i enforcuje. Jesli model przeczyta
tylko chain, moze pomyslec ze to "definicja bez enforcementu".
Jesli przeczyta tylko ledger, moze nie wiedziec skad biora sie reguly.

### Zielone flagi
- model jawnie laczy regule z chain z enforcement w ledger
- model rozumie Stage A -> Stage B handoff path
- model widzi ze node submit gate i ledger import sa dwiema liniami obrony

### Czerwone flagi
- model traktuje chain rules jako "dead code" bo nie widzi wywolania w node
- model nie laczy wallet assembly z proof handoff
- model mysli ze submit gate to jedyna walidacja


---

## Mini-prompt 4: Test Coverage Reality Map

### Kiedy uzywac
Gdy model ma ocenic co jest "done" vs "partial" na podstawie kodu.
Testy sa najsilniejszym dowodem. Ten prompt wymusza ich jawne zmapowanie.

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- oznaczyłes cos jako "done" ale nie ma testu (Evidence source nie moze byc "tests" — to jest "inferred done", nie "done"),
- zakladasz ze test przechodzi ale go nie uruchomiles — nie mow "tested", mow "test exists, not run",
- pokrycie testowe ujawnia gap ktory zmienia Twoja ocene statusu modulu — cofnij sie i popraw status.

### Prompt

```text
Dla kazdego modulu / feature'a ktory opisujesz:

1. Jakie testy istnieja? (podaj nazwy plikow i nazwy testow)
2. Co dokladnie pokrywaja? (happy path / error path / edge case / e2e)
3. Czego NIE pokrywaja? (jaki scenariusz nie ma testu)
4. Czy testy przeszly? (jesli nie wiesz — napisz "nie uruchomilem")

Odpowiedz w formacie:

| Modul/Feature | Plik testowy | Testy (nazwy) | Pokrycie | Brak pokrycia | Evidence source | Przeszly | Unchecked assumptions |
|--------------|-------------|----------------|----------|---------------|-----------------|----------|-----------------------|
| ...          | ...         | ...            | happy/error/edge/e2e | opis | code/tests/grep/inferred | tak/nie/nie_uruchomilem | czego nie zweryfikowano |

Sila dowodu:
- test-confirmed + przeszly = najsilniejszy dowod
- test-confirmed + nie_uruchomilem = sredni dowod
- code-confirmed + brak testu = slab dowod (inferred)
- inferred + brak testu = brak dowodu (unchecked)
```

### Correction pill link
Jesli test coverage map ujawnil ze cos co uznawailes za "done" nie ma testu:
```
KOREKTA:
- Twierdziles: [feature X] jest done
- Prawda: [feature X] jest inferred-done — brak dedykowanego testu
- Dowod: grep po repo nie zwraca test file dla tego scenariusza
- Co sie zmienia: status spada z "done" na "partial" lub "inferred"
- Kontynuuj od: dopisz test do backlogu albo obniż pewnosc
```

### Dlaczego to jest wazne

Model czesto twierdzi "X is done" albo "X is partial" na podstawie
samego kodu produkcyjnego. Ale:
- "done" bez testu to "inferred done" — moze byc buggy
- "partial" z przechodzacymi testami to mocniejsze niz "done" bez testow
- test e2e (escrow_e2e_release.rs) jest silniejszym dowodem niz 10 unit testow

W privAI:
- `escrow_e2e_release.rs` potwierdza caly release flow od keygen do block import
- `escrow_submit_gate.rs` ma 15 testow pokrywajacych rozne error pathy
- ale `escrow_e2e_refund.rs` NIE ISTNIEJE — to jest real gap, nie inferred

### Zielone flagi
- model podaje konkretne nazwy testow
- model rozroznia unit test od e2e test
- model przyznaje "nie uruchomilem" zamiast zakladac ze przechodzi
- kolumna Unchecked assumptions jawnie nazywa brak pokrycia

### Czerwone flagi
- model mowi "tested" bez podania nazwy testu
- model zaklada ze test istnieje bo "to jest oczywisty scenariusz"
- model nie rozroznia "test istnieje" od "test przechodzi"


---

## Mini-prompt 5: Dependency Reality Check

### Kiedy uzywac
Gdy model buduje kolejnosc taskow i deklaruje ze "X zalezy od Y"
albo "X musi byc przed Y".

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- deklarujesz hard dependency ale nie masz dowodu z code path (import/call) — to moze byc false dependency,
- wiecej niz polowa zaleznosci ma Evidence source = "assumption" — potrzebujesz wiecej code-read, nie wiecej zalozen,
- znalazles workaround (mock/harness) ktory zmienia kolejnosc backlogu — cofnij sie i przeoderuj.

### Prompt

```text
Dla kazdej zaleznosci, ktora deklarujesz miedzy taskami:

1. Jaki jest typ zaleznosci?
   - hard dependency (X nie moze byc zbudowany fizycznie bez Y)
   - soft dependency (Y ulatwia X, ale X mozna zrobic bez Y)
   - recommended order (lepiej zrobic Y przed X, ale nie jest konieczne)
   - false dependency (X i Y wygladaja na zalezne, ale nie sa)

2. Jaki jest dowod?
   - code path (X importuje/wywoluje Y)
   - test harness (test X wymaga Y do uruchomienia)
   - architectural (spec mowi ze X zalezy od Y)
   - assumption (wydaje mi sie ze X zalezy od Y)

3. Czy istnieje workaround?
   - harness/mock (test moze uzyc fake zamiast real Y)
   - direct call (mozna ominac Y i wywolac nizszy level)
   - brak workaround (naprawde trzeba miec Y)

Odpowiedz w formacie:

| Task X | Zalezy od Y | Typ | Evidence source | Workaround | Pewnosc | Unchecked assumptions |
|--------|------------|-----|-----------------|------------|---------|----------------------|
| ...    | ...        | hard/soft/recommended/false | code/tests/arch/assumption | opis albo brak | wysoka/srednia/niska | czego nie zweryfikowano |

Jesli Evidence source = "assumption" i Typ = "hard", to NIE MOZE tak zostac.
Musisz albo zweryfikowac kodem, albo obnizic do "soft" / "recommended".
```

### Correction pill link
Jesli dependency check ujawnil falszywa zaleznosc:
```
KOREKTA:
- Twierdziles: [Task X] wymaga [Task Y] jako hard dependency
- Prawda: [Task X] moze byc zrealizowany bez [Task Y] przez [workaround]
- Dowod: [test harness / code path ktory to potwierdza]
- Co sie zmienia: [Task X] i [Task Y] moga byc rownoglegle
- Kontynuuj od: przebuduj kolejnosc backlogu
```

### Dlaczego to jest wazne

Falszywe dependency to jeden z najdrozszych bledow planowania.
Serializuja prace, ktora mogla byc rownlegla, i opozniaja caly projekt.

W pierwszej sesji onboardingowej model traktowal mailbox runtime
jako hard dependency dla refund e2e testu. Efekt: refund bylby
zablokowany do czasu zakonczenia mailbox runtime.

Prawda: `escrow_e2e_release.rs` nie uzywa mailbox — uzywa
`handle_privai_body()` bezposrednio. Wiec refund e2e tez moze
to zrobic. Mailbox to soft dependency (recommended order), nie hard.

Evidence source: code-confirmed (test file nie importuje mailbox).
Unchecked assumptions: zakladamy ze refund test harness moze uzyc tego samego patternu co release.

### Zielone flagi
- model rozroznia hard od soft dependency
- model sprawdza test harness zanim zadeklaruje hard dependency
- model proponuje rownolegle sciezki tam gdzie to mozliwe
- kolumna Evidence source nie jest zdominowana przez "assumption"

### Czerwone flagi
- model serializuje wszystko w jeden lancuch
- model mowi "depends on" bez sprawdzenia czy istnieje workaround
- model traktuje "recommended order" jako "must have before"
- wiecej niz polowa Typ = "hard" z Evidence source = "assumption"


---

## Mini-prompt 6: Pre-Task Readiness Gate

### Kiedy uzywac
Bezposrednio przed napisaniem task pill. To jest ostatni checkpoint
przed wyslaniem pracy do agenta wykonawczego.

### When to stop
**Przerwij i uruchom Correction pill jesli:**
- na ktorekolwiek z pytan 1-5 odpowiedz brzmi "nie" albo "nie wiem" — task pill NIE JEST gotowy,
- sekcja UNCHECKED jest pusta — to niemal na pewno znaczy ze model ukrywa niepewnosc, nie ze wszystko sprawdzil,
- Definition of Done jest subiektywny (np. "code looks correct") zamiast mierzalny (np. "cargo test --test escrow_e2e_refund passes").

W tych przypadkach nie pisz task pill.
Cofnij sie do odpowiedniego mini-promptu (1-5) i domknij luke.

### Prompt

```text
Zanim napiszesz task pill, odpowiedz na te pytania:

1. STATUS MODULU
   - Czy przeszles mini-prompty 1-5 dla kazdego modulu, ktory task dotyka?
   - Czy masz pelna tabele: co jest done / partial / current / unchecked?
   - Evidence source dla statusu kazdego modulu: docs/code/tests/grep/inferred?

2. SCOPE
   - Czy task dotyczy dokladnie jednej rzeczy?
   - Czy write scope jest minimalny?
   - Czy "Do not touch" jest jawnie nazwane?

3. BUILDING BLOCKS
   - Jakie istniejace elementy agent ma wykorzystac? (podaj nazwy)
   - Czy sa testy, ktore agent moze uzyc jako template?
   - Czy jest istniejacy e2e test, ktory mozna skopiowac/zaadaptowac?
   - Evidence source: code/tests (nie "inferred" — musisz widziec plik)

4. SOURCE OF TRUTH
   - Jaki spec doc jest canonical dla tego tasku?
   - Czy ten spec jest aktualny wzgledem repo?
   - Czy jest rozbieznosc miedzy spec a kodem?
   - Evidence source: docs/code (musisz miec oba)

5. VERIFICATION
   - Jak agent ma zweryfikowac ze task jest ukonczony?
   - Jaka komenda potwierdza sukces?
   - Czy Definition of Done jest mierzalny (nie subiektywny)?

6. UNCHECKED ASSUMPTIONS
   - Czego NIE sprawdziles przed napisaniem tego tasku?
   - Jakie ryzyko z tego wynika?
   - Czy agent powinien o tym wiedziec?
   - Czy to unchecked moze zmienic scope tasku jesli sie okazuje falszywe?

Podsumuj readiness:

| Sekcja | Status | Evidence source | Unchecked |
|--------|--------|-----------------|-----------|
| Status modulu | pass/fail | ... | ... |
| Scope | pass/fail | ... | ... |
| Building blocks | pass/fail | ... | ... |
| Source of truth | pass/fail | ... | ... |
| Verification | pass/fail | ... | ... |

Jesli jakikolwiek wiersz ma Status = "fail", task pill NIE JEST gotowy.
```

### Correction pill link
Jesli gate ujawnil ze task pill nie jest gotowy:
```
NIE WYSYLAJ TASKU.
Cofnij sie do mini-promptu, ktory pokrywa obszar "fail":
- Status fail -> Mini-prompt 2 (Partial vs Absent)
- Building blocks fail -> Mini-prompt 4 (Test Coverage)
- Source of truth fail -> Prompt 6 z etapu uczacego
- Verification fail -> przepisz Definition of Done
Po domknieciu luki, wroc do Mini-prompt 6 i powtorz gate.
```

### Dlaczego to jest wazne

Task pill wyslany z falszywym "What is already there" albo brakujacym
"Do not touch" powoduje, ze agent wykonawczy:
- reimplementuje cos co juz istnieje,
- modyfikuje moduly ktore sa frozen,
- albo buduje na falszywych zalozeniach i produkuje kod
  ktory trzeba wyrzucic.

Ten checkpoint jest tani (5 minut modelu) ale zapobiega
godzinom zmarnowanej pracy agenta.

### Zielone flagi
- model odpowiada "pass" na wszystkie 5 sekcji gate'owych
- model podaje konkretne nazwy plikow/testow/specow
- sekcja UNCHECKED jest niepusta i uczciwa
- Evidence source jest wszedzie jawny

### Czerwone flagi
- model przeskakuje gate i od razu pisze task pill
- sekcja UNCHECKED jest pusta ("sprawdzilem wszystko")
- Definition of Done jest subiektywny ("code looks correct")
- Evidence source = "inferred" przy Building blocks (to znaczy ze nie widzial pliku)


---

## Podsumowanie pipeline'u

```
ETAP 1: UCZENIE (Prompty 1-6)
  Model buduje obraz systemu bez kodu.
  Efekt: poprawny mental model.

ETAP 2: UTWARDZENIE (ten dokument, mini-prompty 1-6)
  Model wchodzi w kod z zabezpieczeniami.
  Efekt: zweryfikowany obraz repo reality.

  Mini-prompt 1: Layer Ownership Check
    -> nie oglos braku bez sprawdzenia ownera
    -> When to stop: >2 elementy unchecked = cofnij sie

  Mini-prompt 2: Partial vs Absent Classifier
    -> nie pomyl partial z greenfield
    -> When to stop: absent okazuje sie miec kod = correction pill

  Mini-prompt 3: Cross-Module Consistency Check
    -> nie buduj obrazu z jednego modulu
    -> When to stop: niespojnosc typow = correction pill

  Mini-prompt 4: Test Coverage Reality Map
    -> nie mow "done" bez dowodu z testow
    -> When to stop: "done" bez testu = obniz do "inferred"

  Mini-prompt 5: Dependency Reality Check
    -> nie serializuj pracy na falszywych zalez.
    -> When to stop: hard + assumption = zweryfikuj albo obniz

  Mini-prompt 6: Pre-Task Readiness Gate
    -> nie wysylaj tasku bez checkpointu
    -> When to stop: jakikolwiek "fail" = cofnij sie

  Kazdy mini-prompt wymaga:
    - kolumne "Evidence source" (docs/code/tests/grep/inferred)
    - kolumne "Unchecked assumptions"
    - jawny warunek "When to stop"
    - link do Correction pill

ETAP 3: WYKONANIE (Task Pill)
  Agent wykonawczy dostaje minimalny, zweryfikowany task.
  Efekt: waski scope, poprawne building blocks, mierzalny DoD.
```

## Kiedy powtarzac

Mini-prompty utwardzajace nie sa jednorazowe. Powtarzaj je:
- przy wejsciu w nowy modul,
- po dluzszej przerwie w sesji,
- gdy model zmienia zdanie o statusie elementu,
- gdy model buduje nowy task pill dotyczacy warstwy,
  ktorej nie sprawdzal mini-promptami.

## Relacja do pozostalych dokumentow

- `PROMPTYZACJA_SYSTEMOWEGO_SENIORA.md` — definiuje role i reguly
- `SZABLON_REGUL_TWORZENIA_PROMPTOW_1_6.md` — definiuje sekwencje uczaca
- `PROMPTY_UTWARDZAJACE_WEJSCIE_W_KOD.md` (ten dokument) — definiuje zabezpieczenia przed bledami code-read
- Correction pill (w PROMPTYZACJA) — mechanizm korekty po bledzie
- Task Pill template (w PROMPTYZACJA) — format przekazania pracy do wykonawcy

Razem tworza kompletny pipeline:
uczenie -> utwardzenie -> wykonanie.

Kazdy etap ma wlasne dokumenty, wlasne reguly, i wlasne warunki przejscia do nastepnego.
Nie wolno przeskakiwac etapow.
