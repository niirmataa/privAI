# P-T050-XIAOMI — V0 Docs Dependency Graph

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-050_DOCS_DEPENDENCY_GRAPH/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.

## Cel

Zbuduj dependency graph dokumentów V0, żebyśmy wiedzieli w jakiej kolejności pisać docs i nie blokować się.

## Source Of Truth

Czytaj:

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- wszystkie `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_*_PL.md`
- wszystkie istniejące V0 direction docs w folderze

## Required Output

### 1. Current Docs Inventory

Tabela:

- filename
- role
- status
- depends on
- unlocks

### 2. Missing Docs Inventory

Tabela:

- proposed filename
- why needed
- depends on
- unlocks
- priority

### 3. Dependency Graph

Napisz graph tekstowo:

```text
A -> B -> C
A -> D
```

### 4. Parallel Tracks

Wypisz docs, które można pisać równolegle.

### 5. Critical Path To Code

Wypisz shortest path od obecnego stanu do pierwszego bezpiecznego code task.

### 6. Critical Path To MCP/RAG

Wypisz shortest path do bezpiecznego `privai-context-mcp`.

## Final Self-Check

- legacy docs: TAK/NIE
- files edited other than output: TAK/NIE
