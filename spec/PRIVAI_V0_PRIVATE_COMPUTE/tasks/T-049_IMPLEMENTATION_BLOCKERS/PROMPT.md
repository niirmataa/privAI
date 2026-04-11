# P-T049-XIAOMI — Implementation-Blocking Decisions

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-049_IMPLEMENTATION_BLOCKERS/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.
Nie proponuj patchy.

## Cel

Zidentyfikuj wszystkie decyzje, które blokują jakikolwiek sensowny kod V0.

Nie chodzi o pełną architekturę.
Chodzi o listę: jeśli tego nie zdecydujemy, kod będzie zły.

## Source Of Truth

Czytaj wszystkie V0 docs/audity w:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/`

Nie czytaj legacy.

## Required Output

### 1. Blocking Decisions Ranked

Tabela:

- rank
- decision
- why blocking
- affected modules
- blocked future docs
- blocked code tasks
- owner
- minimum evidence needed

### 2. Critical Path

Wypisz kolejność, w której te decyzje muszą zapaść.

### 3. Parallelizable Decisions

Wypisz decyzje, które można rozwiązywać równolegle.

### 4. Decisions Not Blocking Yet

Wypisz rzeczy ważne, ale nieblokujące teraz.

### 5. Immediate Next 3 Decisions

Wskaż tylko 3 najbliższe decyzje.

## Final Self-Check

- legacy docs: TAK/NIE
- files edited other than output: TAK/NIE
- code read: TAK/NIE
- wire formats: TAK/NIE
