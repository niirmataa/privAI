# T-054-XIAOMI — Final Reviewer Brief Without Opus Gate

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md
```

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.
Nie zakładaj, że Opus jest dostępny.
Nie traktuj Opusa jako gatekeepera ani źródła canonical authority.

## Cel

Przygotuj finalny brief dla dowolnego senior reviewera/modelu (`Gemini`, `Xiaomi`, `Claude/Opus`, `Codex`) na bazie aktualnych V0 docs, audytów i task outputs.

To ma być dokument rozmowy i kontroli decyzji, nie spec.

Ma mówić reviewerowi:

- co już wiemy,
- co jest mocne,
- co jest blokujące,
- które decyzje należą do Operatora,
- które decyzje wymagają direction/spec doc,
- czego nie wolno zepsuć,
- jak nie pomylić task output z canonical doc.

## Source Of Truth

Czytaj:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/REVIEW_CODEX.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md
spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md
```

Optionalnie czytaj inne pliki w:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

Nie czytaj legacy docs.

## Required Output

### 1. Executive Summary

Maksymalnie 10 punktów.

### 2. What Is Settled

Co jest już praktycznie ustalone.

### 3. What Is Not Settled

Co nadal jest open.

### 4. Code Reality

Co kod potwierdza.

### 5. Critical Blockers

Najważniejsze blockers.

### 6. Operator Decisions

Maksymalnie 10 decyzji należących do operatora/projektu, nie do modelu.

### 7. Missing Direction / Spec Decisions

Maksymalnie 10 decyzji, które wymagają direction/spec doc albo reviewer consensus.

Nie pisz "Opus must decide".

Pisz "requires direction/spec/reviewer decision".

### 8. Recommended Next Task Order

Podaj kolejność kolejnych tasków dla Xiaomi/Gemini/Codex.

### 9. Red Lines

Czego żaden reviewer/model nie powinien robić.

### 10. Final Self-Check

- legacy docs: TAK/NIE
- task output treated as canonical: TAK/NIE
- files edited other than output: TAK/NIE
- wire formats defined: TAK/NIE
- Opus treated as blocker: TAK/NIE
