# Szablon Regul Tworzenia Promptow 1-6

## Cel

Ten dokument opisuje, jak budowac sekwencje promptow onboardingowych dla
modelu kontekstowego w projekcie `privAI`.

Celem nie jest:
- zalanie modelu dokumentacja,
- zrobienie jednego ogromnego promptu,
- ani wymuszenie czytania wszystkiego od razu.

Celem jest:
- jak najszybciej doprowadzic model do poprawnego obrazu systemu,
- przy jak najmniejszym koszcie tokenowym,
- bez hallucynowania,
- bez falszywych blockerow,
- i bez mieszania current reality z future direction.

## Zasada nadrzedna

Kazdy prompt ma zamykac jedna klase pytan i przygotowywac grunt pod kolejny krok.

Nie wolno:
- przeskakiwac od razu do kodu,
- wrzucac deep docs zanim model rozumie produkt,
- mieszac canonical `/spec` z repo reality,
- ani kazac modelowi "czytac wszystko".

## Trzy warstwy prawdy

Przy tworzeniu promptow zawsze trzeba pilnowac trzech warstw:

### 1. Canonical truth
- `spec/`
- frozen rules
- current canonical
- future target
- experimental / unresolved

### 2. Handoff reality
- `docs/ai_onboarding/`
- aktualny stan projektu
- ostatnie decyzje
- current priorities

### 3. Repo reality
- kod
- testy
- faktyczny stan implementacji

Kazdy prompt powinien jawnie wskazywac, na ktorej warstwie model aktualnie pracuje.

## Reguly ogolne dla wszystkich promptow

### Regula 1
Na poczatku nie pozwalaj modelowi implementowac ani czytac kodu.

### Regula 2
Najpierw produkt i granice, potem governance, potem canonical core, potem deep docs, potem repo.

### Regula 3
Kazdy prompt musi wymuszac rozdzielenie:
- `confirmed`
- `partial`
- `open`
- `unresolved`

albo na pozniejszych etapach:
- `already done`
- `partial`
- `current`
- `future direction`

### Regula 4
Jesli model ma wskazac kolejny reading path, nie wolno mu "zrzucic calego /spec".
Ma wskazac minimalny, sensowny nastepny zestaw dokumentow.

### Regula 5
Kazdy prompt powinien miec jawny zakaz:
- overclaimingu
- zgadywania brakujacej semantyki
- mylenia spec gap z current repo blockerem

### Regula 6
Kazdy prompt powinien prosic o odpowiedz operacyjna, nie tylko ladne streszczenie.

### Regula 7
Jesli model cos pomylil, kolejny prompt powinien najpierw skorygowac trajektorie,
a dopiero potem pchac go dalej.

### Regula 8
Jesli model popelnil konkretny blad, uzyj `Correction pill`, nie ogolnej reprymendy.

### Regula 9
W odpowiedziach repo-facing albo backlog-facing wymagaj sekcji:
- `Unchecked assumptions`

### Regula 10
Prompt 6 powinien domyslnie byc dzielony na mniejsze kroki per warstwa odpowiedzialnosci.

## Prompt 1 - Foundation

### Cel
- zbudowac poprawny model produktu
- ustawic granice systemu
- oddzielic produkt od jednego modulu

### Czego model ma sie nauczyc
- czym jest `privAI`
- jakie sa filary produktu
- czemu escrow nie jest gwiazda calego systemu
- jakie sa podstawowe role i trust assumptions

### Jakie docs dawac
- foundation
- entrypoint
- readiness/gaps
- production path
- next direction
- docs index

### Czego nie dawac
- kodu
- calego `/spec`
- deep escrow docs

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model nie traktuje projektu jak repo od escrow
- poprawnie rozroznia rails
- poprawnie rozroznia Stage A / Stage B
- poprawnie rozroznia wallet / node / control-plane

#### Czerwone flagi
- od razu chce czytac kod
- od razu chce czytac wszystko w `/spec`
- opisuje produkt jak "escrow chain"
- miesza marketplace z FullPrivacy

#### Pytania kontrolne
- Czy `privAI` to system escrow? Odpowiedz powinna brzmiec: nie, escrow jest jednym z primitives.
- Czy marketplace rail i FullPrivacy to ten sam trust model? Odpowiedz powinna brzmiec: nie.
- Czy model probuje schodzic do tx / proof / consensus? Jesli tak, to za szybko.

## Prompt 2 - Governance Layer

### Cel
- nauczyc model status vocabulary
- nauczyc model source of truth hierarchy
- nauczyc model anti-hallucination rules

### Czego model ma sie nauczyc
- `spec/` jako canonical truth
- frozen vs current canonical vs future target vs experimental vs unresolved
- execution spine jako dokument sterujacy, nie nowa architektura

### Jakie docs dawac
- `PRIVAI_SPEC_INDEX.md`
- `PRIVAI_SYSTEM_FREEZE_FOR_PVA_DRAFT.md`
- `PRIVAI_DECISION_REGISTER.md`
- `PRIVAI_GAP_REGISTER.md`
- `PRIVAI_EXECUTION_SPINE.md`

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model przestaje nazywac "missing" rzeczy, ktore sa tylko rozproszone
- rozumie statusy
- rozumie, ze unresolved nie wolno zgadywac

#### Czerwone flagi
- bierze kazdy `GAP-*` jako current repo blocker
- wskazuje nieistniejace pliki
- mysli, ze execution spine jest normatywna architektura

#### Pytania kontrolne
- Czy model potrafi sklasyfikowac element jako: frozen / current canonical / future target / experimental / unresolved?
- Czy model rozumie, ze `GAP-*` nie jest automatycznie current repo blockerem?

## Prompt 3 - Canonical Core

### Cel
- zbudowac techniczny kregoslup systemu
- nauczyc ownership rules
- nauczyc relacji product semantics vs canonical formats vs consensus

### Czego model ma sie nauczyc
- note/UTXO core
- tx classes
- canonical formats
- consensus ownership
- marketplace rail
- escrow adaptation

### Jakie docs dawac
- `PRIVAI_PROTOCOL_CORE.md`
- `PRIVAI_CANONICAL_FORMATS.md`
- `PRIVAI_MARKETPLACE_SMALL_PAYMENTS.md`
- `PRIVAI_CONSENSUS.md`
- `PRIVAI_ESCROW_2OF3_ADAPTATION.md`

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model poprawnie rozroznia current canonical vs future migration
- poprawnie rozumie `tx_signing_hash`
- poprawnie rozumie marketplace rail jako odrebny trust model

#### Czerwone flagi
- zbyt szerokie "proof is done"
- mieszanie canonical bytes z final semantics
- ponowne traktowanie escrow object model jako pustej kartki

#### Pytania kontrolne
- Gdzie nalezy `tx_signing_hash`: do Stage A czy Stage B?
- Czy `policy_tag` jest source of truth, czy tylko hintem?
- Czy marketplace rail jest odmiana FullPrivacy? Odpowiedz powinna brzmiec: nie.

## Prompt 4 - Deep Domain Closure

### Cel
- doprecyzowac proof vs ledger boundary
- doprecyzowac escrow object model i action semantics
- doprecyzowac transport split

### Jakie docs dawac
- `PRIVAI_PROOF_BOUNDARIES.md`
- `PRIVAI_TRANSFER_NOTE_PROOF_SEMANTICS.md`
- `PRIVAI_ESCROW_OBJECT_MODEL.md`
- `PRIVAI_ESCROW_TX_MATRIX.md`
- `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model umie jasno powiedziec co jest proof-covered, a co ledger-enforced
- model umie opisac `release/refund/recovery_release`
- model nie miesza validator transport i NXMS

#### Czerwone flagi
- traktuje `statement_commit` jako dowod final proof modelu wszystkiego
- miesza target i current rule dla escrow
- robi z transportu jeden stos

#### Pytania kontrolne
- Czy timeout jest proof-enforced czy ledger-enforced?
- Czy `TransferNoteTx` oznacza, ze caly proof plane jest domkniety? Odpowiedz powinna brzmiec: nie.
- Czy validator transport i NXMS mailbox to jedna warstwa? Odpowiedz powinna brzmiec: nie.

## Prompt 5 - Freeze, Vectors, Auth Rules

### Cel
- doprowadzic model do stanu, w ktorym umie zrobic finalne summary systemu
- rozlozyc implementation tracks
- przygotowac sensowna liste taskow

### Jakie docs dawac
- `PRIVAI_REFERENCE_VECTORS.md`
- `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
- `PRIVAI_PROOF_COMPLETION_PLAN.md`
- `PRIVAI_AUTH_SIGNING_MODEL.md`
- `PRIVAI_THRESHOLD_AUTH_CANONICAL_RULES.md`

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model poprawnie oddziela:
  - current reality
  - production-shape v1
  - final product direction
- umie rozpisac implementation tracks
- umie zrobic liste taskow bez chaosu

#### Czerwone flagi
- wrzuca future direction przed v1 closure
- cofa sie do juz zrobionych fundamentow
- robi taski abstrakcyjne zamiast operacyjnych

#### Pytania kontrolne
- Czy backlog rozdziela `already done / partial / current / future direction`?
- Czy model wrzuca post-v1 rzeczy przed runtime closure? Jesli tak, to zle.

## Prompt 6 - Repo Reconciliation

### Cel
- zderzyc canonical system truth z repo reality
- wyciac taski juz zrobione
- rozpoznac partial-but-real
- zrobic prawdziwy current backlog

### Jakie docs i inputs dawac
- handoff docs
- minimalny code-read
- test files
- konkretne module boundaries

### Jak prowadzic Prompt 6
- najlepiej dzielic go na sub-prompty:
  - chain
  - ledger
  - node
  - wallet
  - proof / handoff
  - tests / final synthesis
- po kazdym kroku model powinien powiedziec:
  - co znalazl
  - czego jeszcze nie sprawdzil
  - czy sprawdzil sasiednia warstwe odpowiedzialnosci

### Na co zwracac uwage w odpowiedzi

#### Zielone flagi
- model umie powiedziec co jest:
  - already done
  - partial
  - current
  - future
- nie robi greenfield z partiali
- nie robi falszywych blockerow
- umie korygowac wlasne poprzednie bledy

#### Czerwone flagi
- ignoruje testy
- oglasza brak po przeczytaniu jednego modulu
- myli spec gap z repo blockerem
- nie odroznia "potwierdzone w kodzie" od "wynika z docs"

#### Pytania kontrolne
- Jesli model mowi, ze cos "nie istnieje", to w jakim module tego szukal?
- Czy sprawdzil sasiednie warstwy?
- Czy ma na to test, grep albo wyrazny dowod?

## Minimalny format odpowiedzi modelu kontekstowego

Jesli prompt ma porzadkowac system lub backlog, domyslny format powinien byc:

1. `Corrected understanding`
2. `Ordered tasks`
3. `Execution order`
4. `Notes for agent prompting`
5. `Unchecked assumptions`

To jest najlepszy format, bo:
- wymusza korekte trajektorii,
- wymusza backlog,
- wymusza dependency map,
- wymusza uczciwe nazwanie niepewnosci,
- i od razu przygotowuje prompt dla wykonawcy.

## Jak oceniac odpowiedzi modelu

### Odpowiedz dobra
- rozumie produkt
- rozumie boundary
- rozumie statusy
- nie overclaimuje
- umie wskazac minimalny nastepny krok
- uczciwie nazywa `Unchecked assumptions`

### Odpowiedz srednia
- rozumie system ogolnie
- ale robi zbyt szerokie wnioski
- albo miesza current z targetem

### Odpowiedz slaba
- opisuje projekt jak repo od jednego modulu
- zbyt szybko wchodzi w kod
- tworzy falszywe dependency
- nie umie odroznic partial od absent
- nie nazywa tego, czego nie sprawdzil

## Kiedy korygowac model

Model trzeba natychmiast skorygowac, jesli:
- uzna partial za absent,
- uzna target za current,
- uzna current repo gap bez sprawdzenia sasiednich warstw,
- albo zbuduje zly reading path.

Najlepszy format korekty:
- `Twierdziles`
- `Prawda`
- `Dowod`
- `Co sie zmienia`
- `Kontynuuj od`

## Co ma byc efektem koncowym

Po przejsciu promptow 1-6 model ma:
- rozumiec system jak senior z kontekstem,
- umiec odzyskac backlog bez historii czatu,
- umiec zrobic dobry task prompt dla wykonawcy,
- i umiec powiedziec, czego jeszcze nie zweryfikowal.

To jest docelowy standard re-entry dla `privAI`.
