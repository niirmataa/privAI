# P-T046-XIAOMI — Decision Matrix Draft

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie rób commitów.
Nie czytaj legacy docs.
Nie używaj marketplace framingu jako product direction.
Nie proponuj patchy.
Nie definiuj finalnych Rust structs.
Nie definiuj wire formatów.

## Cel

Zbuduj pierwszą macierz decyzji V0 na podstawie wszystkich dotychczasowych audytów.

Nie wymyślaj nowego systemu.
Nie rób kolejnej ogólnej syntezy.
Masz sklasyfikować decyzje.

## Source Of Truth

Czytaj tylko:

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_FINAL_ARCHITECTURE_PROPOSAL_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

## Status Labels

Używaj tylko:

- `FROZEN_CANDIDATE`
- `STRONG_CANDIDATE`
- `CANDIDATE`
- `OPEN`
- `BLOCKED_BY_OPERATOR`
- `BLOCKED_BY_OPUS`
- `BLOCKED_BY_CODE_AUDIT`
- `REJECTED`

## Required Output

### 1. Decision Matrix

Tabela:

- decision
- recommended status
- reason
- supporting docs
- blockers
- risk if frozen too early
- risk if delayed
- owner

Uwzględnij minimum:

- V0 product model
- add-new-alongside-old strategy
- Escrow2of3 as bridge
- RecoveryRelease as operatorless anchor
- Amount14 proof lane only
- LedgerAmount for economics
- u64 vs u128
- ComputeLeaseEscrow new SpendPolicy
- Pro-rata as future path
- MarketplaceBatchTx fate
- MarketplaceSettlement fate
- Falcon PK as ValidatorRoleKey
- HiddenRootCredential additive later
- small payments Receipt not reused directly
- nxms-escrow-orchestrator as automated operator bridge
- NXMS mailbox as discovery transport base
- VM as default privacy class
- exit node opt-in only
- V0-only MCP/RAG context

### 2. Freeze-Ready Decisions

Wypisz decyzje, które mogą być prawie zamrożone.

### 3. Decisions That Must Wait

Wypisz decyzje, których nie wolno jeszcze zamrażać.

### 4. Operator Decisions

Wypisz decyzje, które wymagają operatora, nie modelu.

### 5. Opus Decisions

Wypisz decyzje, które powinien zatwierdzić Opus.

### 6. Code Audit Decisions

Wypisz decyzje, które wymagają dalszego code audit.

### 7. Final Recommendation

Co powinno trafić do `PRIVAI_V0_FINAL_DOMAIN_AND_MIGRATION_DECISIONS_PL.md`.

## Final Self-Check

Na końcu odpowiedz:

- Czy czytałeś legacy docs: TAK/NIE
- Czy czytałeś kod: TAK/NIE
- Czy edytowałeś pliki inne niż output: TAK/NIE
- Czy definiowałeś wire formaty: TAK/NIE
- Czy to jest decision matrix, a nie implementation spec: TAK/NIE
