# Prompt 6 Split - Repo Reconciliation

## Cel

Prompt 6 jest najtrudniejszym etapem calego pipeline.
To tutaj model przechodzi z:
- canonical truth
- i handoff reality

do:
- repo reality
- i prawdziwego current backlogu.

Zamiast jednego szerokiego promptu, lepiej rozbic ten etap na male sub-prompty
per warstwa odpowiedzialnosci.

## Zasada nadrzedna

Model nie ma robic szerokiego code tour.
Kazdy sub-prompt ma:
- maly scope,
- konkretny cel,
- jawny checkpoint,
- i obowiazkowe `Unchecked assumptions`.

## Kolejnosc sub-promptow

1. `chain`
2. `ledger`
3. `node`
4. `wallet`
5. `proof / handoff`
6. `tests / final synthesis`

W razie potrzeby:
7. `nexum-core orchestrator bridge`

## Sub-prompt 6A - Chain

### Cel
- potwierdzic formaty tx,
- `tx_signing_hash`,
- canonical action / class wiring,
- i to, co jest juz fundamentem, nie backlogiem.

### Czytaj minimalnie
- `privai-chain/*` tylko w obszarze tx / formats / ids

### Odpowiedz
- co jest `already done`
- co jest `partial`
- czego jeszcze nie sprawdziles

## Sub-prompt 6B - Ledger

### Cel
- ustalic, co ledger juz realnie egzekwuje
- i nie pomylic braku w `node` z brakiem w systemie

### Czytaj minimalnie
- `privai-ledger/*` tylko w obszarze auth / escrow / import validation

### Obowiazkowy checkpoint
Zanim oglosisz brak:
- sprawdz, czy dana regula nie siedzi w ledgerze,
- nazwij testy, jesli je widzisz,
- oddziel `confirmed by tests` od `inferred`.

## Sub-prompt 6C - Node

### Cel
- ustalic runtime boundary:
  - submit gate
  - ingress
  - mailbox
  - stage storage

### Czytaj minimalnie
- `privai-node/src/node.rs`
- `privai-node/src/mailbox_pull.rs`
- tylko najblizsze testy

### Obowiazkowy checkpoint
Jesli czegos nie ma w `node`, nie oglaszaj braku bez sprawdzenia:
- ledger,
- wallet,
- albo proof handoff.

## Sub-prompt 6D - Wallet

### Cel
- ustalic, co wallet juz sklada,
- czego nie trzeba projektowac od nowa,
- i gdzie konczy sie rola walleta.

### Czytaj minimalnie
- `privai-wallet/*` w obszarze assembly / escrow_builder / operator / proof handoff

## Sub-prompt 6E - Proof / Handoff

### Cel
- sprawdzic, czy proof path jest:
  - juz runtime-usable,
  - tylko typed,
  - czy partial operationally

### Czytaj minimalnie
- proof handoff
- artifact attach path
- import / record flow tylko w potrzebnym zakresie

## Sub-prompt 6F - Tests / Final synthesis

### Cel
- zamknac reconciliation testami,
- wyciac rzeczy juz zrobione,
- i zbudowac prawdziwy current backlog

### Czytaj minimalnie
- najblizsze test files dla obszarow, ktore model uznal za current albo partial

### Finalny output
1. `What is already done`
2. `What is partial`
3. `What is current`
4. `What is future`
5. `Unchecked assumptions`

## Obowiazkowe pytania kontrolne w Prompt 6

Dla kazdej rzeczy oznaczonej jako `absent`, `missing` albo `gap` model musi odpowiedziec:

1. W ktorym module tego szukalem?
2. Czy sprawdzilem sasiednia warstwe odpowiedzialnosci?
3. Czy test albo grep repo potwierdza brak?

Jesli nie:
- nie wolno oznaczac tego jako `absent`
- trzeba oznaczyc to jako `unchecked` albo `not yet verified`
