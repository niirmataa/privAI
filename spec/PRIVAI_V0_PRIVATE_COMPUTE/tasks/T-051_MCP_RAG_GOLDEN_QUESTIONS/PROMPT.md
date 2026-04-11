# P-T051-XIAOMI — MCP/RAG Golden Questions

Pracujesz w repo:

`/home/nxms-server/privAI`

To jest NOWY TASK read-only.

Write output to:

`spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-051_MCP_RAG_GOLDEN_QUESTIONS/OUTPUT_XIAOMI.md`

Nie edytuj innych plików.
Nie twórz innych plików.
Nie czytaj legacy docs.

## Cel

Przygotuj golden questions dla przyszłego `privai-context-mcp` i V0 RAG.

Chodzi o testy, które wykryją, czy agent wraca do starego marketplace modelu albo overclaimuje implementation state.

## Source Of Truth

Czytaj:

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_FINAL_ARCHITECTURE_PROPOSAL_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md`
- audyty P-T040 do P-T045

## Required Output

### 1. Golden Questions

Podaj 30 pytań.

Dla każdego:

- question
- expected answer summary
- source docs
- forbidden wrong answer
- why it catches drift

### 2. Categories

Rozdziel pytania na:

- product framing
- implementation reality
- amount/aPVA
- escrow/spendpolicy
- identity
- marketplace legacy
- receipts/metering
- discovery/transport
- MCP/RAG source policy

### 3. Pass/Fail Criteria

Jak stwierdzić, że MCP/RAG jest bezpieczny dla agentów.

### 4. Minimal Smoke Test

Podaj 10 pytań z 30 jako szybki test.

## Final Self-Check

- legacy docs: TAK/NIE
- files edited other than output: TAK/NIE
