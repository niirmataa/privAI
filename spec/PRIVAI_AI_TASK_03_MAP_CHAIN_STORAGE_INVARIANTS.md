# privAI AI Task 03 — Map Current Chain Storage Invariants

Status: active implementation-mapping task.
Canonicality: execution task derived from the canonical `spec/` set. This document is not a new source of truth; it tells an agent what to map and document under the existing frozen docs.
Owner: privAI storage and runtime alignment.
Depends on:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_REFERENCE_VECTORS.md`
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`

## 1. Cel zadania

Celem tego zadania jest spisanie jednego uczciwego mapowania:
- co jest current durable state,
- co jest rebuildable,
- co jest ephemeral,
- gdzie dzis lezy granica miedzy ledger state a node-local auxiliary state.

To nie jest zadanie na:
- zmiane backendu storage,
- refactor storage layer,
- nowy format blokow,
- nowy model consensus,
- "naprawianie" kodu przez architektoniczne zgadywanie.

## 2. Twarda zasada interpretacji

Agent nie tworzy nowego source of truth.
Agent nie wymysla nowej architektury storage.

Agent ma:
- przeczytac canonical docs,
- zmapowac current code behavior,
- zapisac go w nowym support doc,
- nazwac miejsca `Unresolved` albo `Current non-conformity`, jesli kod nie daje jednej czystej odpowiedzi.

Jesli czegos nie da sie wywnioskowac jednoznacznie z kodu i docs:
- nie wolno tego domykac "najbardziej sensowna" odpowiedzia,
- trzeba to oznaczyc jako luka.

## 3. Gdzie agent ma czytac

Najpierw:
- `spec/PRIVAI_SPEC_INDEX.md`
- `spec/PRIVAI_REENTRY_GUIDE.md`
- `spec/PRIVAI_PROTOCOL_CORE.md`
- `spec/PRIVAI_CANONICAL_FORMATS.md`
- `spec/PRIVAI_CONSENSUS.md`
- `spec/PRIVAI_CHAIN_STORAGE_AND_RECOVERY_NEXT_STEPS.md`

Potem kod:
- `privai-ledger/*`
- `privai-chain/*`
- `privai-node/*`

Tylko tam, gdzie to jest potrzebne do:
- ustalenia current persistent state,
- ustalenia restart/recovery contract,
- ustalenia DB layout i ownership granic.

## 4. Wynik zadania

Agent ma utworzyc nowy plik:
- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`

Ten plik ma byc support doc anti-drift, nie nowym canonical core doc.

## 5. Co dokument ma zawierac

### 5.1. Scope i boundary

Dokument ma jasno opisac:
- co obejmuje chain storage,
- czego nie obejmuje,
- co nalezy do `privai-ledger`,
- co nalezy do `privai-node`,
- co nalezy do runtime pomocniczego.

### 5.2. Durable / rebuildable / ephemeral split

Dokument ma rozdzielic:
- `Current canonical durable state`
- `Rebuildable derived state`
- `Ephemeral runtime state`

### 5.3. Source-of-truth mapping

Dokument ma nazwac source of truth dla:
- note status,
- note commitments / note set,
- spent note nullifiers,
- spent ticket nullifiers,
- block store,
- tip / head,
- finalized head,
- QC / safety state,
- last persisted but not fully indexed block.

### 5.4. Atomicity groups

Dokument ma opisac:
- ktore write groups musza byc atomowe,
- ktore dane moga byc zapisane pozniej,
- co jest correctness-critical, a co tylko convenience index.

### 5.5. Current non-conformities / unresolved

Jesli dzis:
- cos jest zapisane niejawnie,
- cos jest tylko in-memory,
- cos nie ma jednego zrodla prawdy,
- cos nie ma jasnego restart contract,

to trzeba to nazwac wprost jako:
- `Current non-conformity`
- albo `Unresolved`

## 6. Twardy scope

W scope:
- nowy plik `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md`
- ewentualne bardzo male dopiecie linkow w `spec/PRIVAI_SPEC_INDEX.md`
- ewentualne bardzo male dopiecie linkow w `spec/PRIVAI_REENTRY_GUIDE.md`

Poza scope:
- zmiany produkcyjnego kodu storage
- migracje DB
- testy storage/restart
- performance tuning
- compaction work
- nowe column families tylko "bo tak bedzie ladniej"

## 7. Acceptance criteria

Zadanie jest uznane za wykonane tylko wtedy, gdy:
- `spec/PRIVAI_CHAIN_STORAGE_INVARIANTS.md` istnieje,
- dokument nie tworzy nowego source of truth,
- dokument uczciwie rozdziela:
  - `Current canonical`
  - `Current non-conformity`
  - `Unresolved`
  - `Future target requiring migration`
- nie ma zgadywania tam, gdzie kod nie daje jednej odpowiedzi,
- dokument daje devowi i agentowi praktyczny storage map bez czytania calego repo od zera.

## 8. Raport koncowy

Agent ma oddac raport w tej formie:

1. `Ktore pliki przejrzano`
2. `Ktore pliki zmieniono`
3. `Co zostalo uznane za Current canonical durable state`
4. `Co zostalo uznane za rebuildable`
5. `Co zostalo uznane za ephemeral`
6. `Jakie Current non-conformities wykryto`
7. `Jakie rzeczy pozostaly Unresolved`
