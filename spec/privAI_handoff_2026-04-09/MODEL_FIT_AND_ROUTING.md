# Model Fit and Routing

## Cel

Ten dokument opisuje, ktory model AI najlepiej sprawdza sie w ktorej roli
w produkcyjnym workflow privAI.

Obserwacje pochodza z realnej pracy nad systemem — onboardingu, code-read,
budowania backlogu, korekty bledow, i porownania zachowan miedzy modelami.

To nie jest ranking "ktory model jest lepszy".
To jest routing: ktory model do jakiej roboty.

## Zasada nadrzedna

Zaden model nie jest najlepszy we wszystkim.
Najlepszy workflow to taki, w ktorym kazdy model robi to, co robi najlepiej,
a operator (czlowiek) routuje prace miedzy nimi.

```
Opus buduje task pill -> Gemini koduje -> GPT sprawdza -> operator decyduje
```

## Role w workflow

### 1. Systemowy senior / prompter / architekt

**Model: Opus 4.6 Max**

Najlepszy do:
- pelnego onboardingu systemu (Prompty 1-6),
- rozdzielania canonical / handoff / repo reality,
- budowania i porzadkowania backlogu,
- budowania task pill dla innych modeli,
- wykrywania falszywych dependency,
- re-entry po przerwie w sesji,
- znoszenia korekt bez degradacji jakosci.

Obserwacje z pracy:
- po korektach (timeout enforcement, mailbox dependency) natychmiast
  przebudowal backlog bez powtarzania starych bledow,
- poprawnie rozroznil 13 taskow w 4 kategoriach (done/partial/current/future),
- zbudowal task pille, ktore operator mogl wyslac bez dalszych pytan,
- sam zaproponowal Regule 7 ("sprawdz sasiednia warstwe") po wlasnym bledzie.

Slabsze strony:
- nie jest najtanszy (wysoki koszt tokenowy),
- nie jest najszybszym "workerem do jednego taska" — mysli za szeroko
  na waski scope, co jest zaleta w roli seniora ale wada w roli codera.

Kiedy NIE uzywac Opusa:
- do prostych, dobrze dociętych taskow implementacyjnych,
- do grepa po repo (marnowanie tokenow),
- do pracy w petli polling (szybko spali budzet).

### 2. Glowny coder / agent wykonawczy

**Model: Gemini 3.1 Pro**

Najlepszy do:
- kodowania po dobrze przygotowanym task pill,
- szybkiego lapania scope'u i wchodzenia w kod,
- pracy w jednym, dobrze ograniczonym tasku,
- konkretnych implementacji bez filozofowania.

Obserwacje z pracy:
- bardzo dobry stosunek jakosci do kosztu,
- szybko wchodzi w scope i nie odbiega od tematu,
- dobrze realizuje bounded runtime taski,
- dobrze miesci sie w formacie task pill.

Slabsze strony:
- gorzej od Opusa robi dlugie dochodzenie do prawdy projektu,
- szybciej myli:
  - stary gap ze swiezym problemem,
  - partial z absent,
  - repo reality z canonical direction,
- nie jest najlepszym modelem do re-entry po dlugiej przerwie.

Kiedy NIE uzywac Gemini:
- jako glownego systemowego seniora (brakuje mu "ostrzegawczego instynktu"),
- do budowania backlogu od zera (potrzebuje gotowego kontekstu),
- do pracy wymagajacej ciaglosci architektonicznej miedzy sesjami.

### 3. Reviewer / sanity-check / drugi mozg

**Model: GPT-4 high thinking**

Najlepszy do:
- sprawdzania czy task/prompt/backlog nie skrecil,
- sanity-checku planu przed wyslaniem do execution,
- klasyfikacji done / partial / current / future,
- porzadkowania po korektach,
- bycia "drugim glosem" przy trudnych decyzjach architektonicznych.

Obserwacje z pracy:
- po korektach zaczal dobrze porzadkowac,
- rozumie strukture pracy i pipeline uczenie -> utwardzenie -> wykonanie,
- potrafi sensownie sklasyfikowac elementy systemu,
- daje wartosc jako niezalezny reviewer planu Opusa.

Slabsze strony:
- mniej precyzyjny niz Opus w repo reconciliation,
- mniej "ostry" — czesciej zostawia rzeczy troche zbyt ogolne,
- nie jest najlepszy w glebokim code-read,
- wymaga dobrego kontekstu zeby dawac wartosc (nie jest self-starting).

Kiedy NIE uzywac GPT-4:
- jako jedynego architekta (zbyt ogolny),
- do glebokich code-read wymagajacych precyzji linia-po-linii,
- jako jedynego modelu w sesji (lepszy jako drugi glos niz pierwszy).

### 4. Worker do dalszego testowania

**Model: Xiaomi (do precyzyjnego okreslenia)**

Ostrozna ocena — mniej twardych danych niz dla pozostalych modeli.

Wstepnie wyglada sensownie do:
- bounded runtime taskow,
- hardening / negative tests,
- prostych, dobrze dociętych taskow wykonawczych.

Czego jeszcze nie wiemy:
- jak radzi sobie z re-entry po przerwie,
- jak radzi sobie z cross-module consistency,
- czy potrafi sam wykryc falszywa zaleznosc.

Kiedy uzywac:
- na bounded taskach z jasnym scope, jasnym DoD, jasnym "Do not touch",
- jako uzupelnienie Gemini, nie zamiennik.

Kiedy NIE uzywac:
- jako glownego architekta lub seniora (brak danych na poparcie),
- na taskach wymagajacych wlasnej oceny co jest done vs partial.

## Produkcyjny routing

### Faza 1: Onboarding / re-entry
```
Opus 4.6 Max
  -> czyta docs (Prompty 1-6)
  -> buduje model systemu
  -> przechodzi mini-prompty utwardzajace
  -> produkuje backlog
```

### Faza 2: Task preparation
```
Opus 4.6 Max
  -> buduje task pill (z Pre-Task Readiness Gate)
  -> task pill przechodzi sanity-check przez GPT-4
  -> operator zatwierdza
```

### Faza 3: Execution
```
Gemini 3.1 Pro (glowny coder)
  -> bierze task pill
  -> koduje
  -> raportuje wynik

Xiaomi (uzupelniajacy worker, jesli task jest bounded)
  -> bierze prostszy task pill
  -> koduje
  -> raportuje wynik
```

### Faza 4: Verification
```
GPT-4 high thinking
  -> sprawdza czy output agentow jest spojny z backlogiem
  -> sprawdza czy nie ma dryfu od spec
  -> raportuje operatorowi

Opus 4.6 Max (jesli potrzebna glebsza weryfikacja)
  -> repo reconciliation
  -> aktualizacja backlogu
```

### Faza 5: Korekta (jesli potrzebna)
```
Opus 4.6 Max
  -> Correction pill
  -> przebudowa backlogu
  -> nowe task pille
```

## Routing w tabeli

| Rola | Model | Pewnosc oceny |
|------|-------|---------------|
| Systemowy senior / architekt | Opus 4.6 Max | wysoka (zweryfikowane w sesji) |
| Glowny coder | Gemini 3.1 Pro | wysoka (wielokrotne obserwacje) |
| Reviewer / sanity-check | GPT-4 high thinking | srednia-wysoka (widoczna wartosc po korektach) |
| Bounded worker | Xiaomi | niska (do dalszego testowania) |

## Co NIE jest w tym dokumencie

Ten dokument nie mowi:
- ktory model jest "najlepszy" ogolnie,
- ze inne modele nie moga byc uzywane w tych rolach,
- ze ten routing jest permanentny.

To jest snapshot oparty na realnych obserwacjach z pracy nad privAI.
Routing powinien byc aktualizowany gdy:
- pojawia sie nowy model,
- istniejacy model dostaje znaczacy update,
- zebierzemy wiecej danych o Xiaomi lub innym modelu,
- zmieni sie charakter pracy (np. przejscie z v1 na v2).

## Relacja do pozostalych dokumentow

- `PROMPTYZACJA_SYSTEMOWEGO_SENIORA.md` — jak przygotowac model do roli seniora
- `SZABLON_REGUL_TWORZENIA_PROMPTOW_1_6.md` — sekwencja uczaca
- `PROMPTY_UTWARDZAJACE_WEJSCIE_W_KOD.md` — zabezpieczenia przed bledami code-read
- `MODEL_FIT_AND_ROUTING.md` (ten dokument) — ktory model do jakiej roboty

Razem tworza kompletny system:
- jak uczyc (Prompty 1-6),
- jak utwardzac (mini-prompty),
- kogo do czego przypisac (routing),
- jak korygowac (correction pill).
