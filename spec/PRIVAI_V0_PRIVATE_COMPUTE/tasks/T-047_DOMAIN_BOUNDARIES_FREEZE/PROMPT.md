# P-T047-XIAOMI — Final Domain Boundaries Freeze Candidate

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-047_DOMAIN_BOUNDARIES_FREEZE/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.
Nie definiuj Rust structs.
Nie definiuj wire formatów.

## Cel

Na podstawie V0 docs i audytów zaproponuj finalne granice domenowe, które mogą zostać zamrożone przed pisaniem kodu.

Chodzi o boundaries, nie fields.

## Source Of Truth

Czytaj:

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_FINAL_ARCHITECTURE_PROPOSAL_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md`

## Required Output

### 1. Boundary List

Dla każdej granicy:

- name
- responsibility
- explicitly not responsible for
- input
- output
- owner module candidate
- status
- blockers

Uwzględnij:

- Chain / Consensus
- Escrow / Settlement
- Metering / Receipts
- Lease Policy
- Identity / Credentials
- Discovery
- Transport / Mailbox / Relay
- Runtime / Compute
- Wallet / Client
- RAG / MCP / Agent Context

### 2. Boundary Invariants

Wypisz invariants dla każdej granicy.

### 3. Boundary Anti-Patterns

Wypisz czego nie wolno mieszać.

Przykłady:

- metering does not settle
- transport does not understand lease policy
- discovery does not expose public identity
- chain does not see workload
- identity does not decide trust

### 4. Freeze Recommendation

Które boundaries można zamrozić teraz?

Które muszą czekać?

## Final Self-Check

- legacy docs: TAK/NIE
- code read: TAK/NIE
- files edited other than output: TAK/NIE
- wire formats: TAK/NIE
