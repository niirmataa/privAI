# P-T048-XIAOMI — Minimal Types Freeze Candidate

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-048_MINIMAL_TYPES_FREEZE/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.
Nie proponuj patchy.
Nie definiuj finalnych field layouts jako wire format.

## Cel

Z audytów P-T040 do P-T045 wybierz minimalny zestaw typów, które można rozważyć jako pierwszą falę "build-once compatible types".

Nie chodzi o wszystkie typy.
Chodzi o minimalny set, który nie powinien wymagać refactoru później.

## Source Of Truth

Czytaj:

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md`

## Required Output

### 1. Minimal Type Set

Tabela:

- type
- why needed now
- status
- depends on
- safe to add now? yes/no
- reason
- risk if added too early
- risk if delayed

### 2. Types To Delay

Tabela:

- type
- why delay
- required prior decision
- possible later owner doc

### 3. Types To Reject

Jeśli brak, napisz `none`.

### 4. Type Dependency Graph

Napisz dependency order, na przykład:

```text
LedgerAmount before ComputeLeasePolicy
Identity role keys before ComputeOffering
```

### 5. Final Recommendation

Czy "types first" jest bezpieczne?

Jeśli tak: które 5-10 typów.

Jeśli nie: co jeszcze trzeba zamrozić.

## Final Self-Check

- legacy docs: TAK/NIE
- code read: TAK/NIE
- files edited other than output: TAK/NIE
- wire formats: TAK/NIE
