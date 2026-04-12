# privAI V0 Xiaomi 5 Context Prompts

**Status:** ready-to-use prompt pack
**Date:** 2026-04-11
**Scope:** read-only V0 context onboarding and focused discussion

Use these prompts one by one.

Do not give all five at once.

After each result, update:

```text
PRIVAI_V0_TASK_LOG.md
PRIVAI_V0_PROMPT_LOG.md
```

---

## Prompt 1 — V0 Context Lock

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest NOWY TASK.
Nie dotykasz kodu.
Nie edytujesz plików.
Nie tworzysz nowych docs.
To jest read-only context lock.

# P-T034-XIAOMI-01 — V0 Context Lock

## Rola

Masz zablokować sobie właściwy mental model V0 `privAI`.

Nie masz proponować implementacji.
Nie masz tworzyć nowych koncepcji.
Masz zrozumieć, co jest canonical, co jest rejected, a co jest future follow-up.

## Source of truth

Przeczytaj tylko:

- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

Nie czytaj kodu.
Nie czytaj legacy docs.
Nie grepaj repo.

## Core rule

```text
privAI is not an AI model marketplace.
privAI is a post-quantum FullPrivacy private AI compute network.
```

```text
Privacy is the product.
Compute is the supply.
PVA is the incentive.
Chain is the settlement.
Transport is the shield.
```

## Required response

Odpowiedz dokładnie w sekcjach:

### 1. Canonical V0 Model

W 10 punktach opisz, czym jest V0.

### 2. Rejected Old Model

W 10 punktach opisz, czego V0 odrzuca.

### 3. Current Status

Wypisz, które V0 docs już istnieją i do czego służą.

### 4. Next Work

Wypisz, jaki dokument jest następny według docs tree i dlaczego.

### 5. Anti-Hallucination Rules

Wypisz 10 reguł, które mają Cię powstrzymać przed wymyślaniem.

### 6. Czego nie sprawdziłem

Wypisz uczciwie.

## Forbidden

Nie wolno:
1. proponować kodu,
2. proponować wire formatów,
3. twierdzić, że operatorless escrow jest implemented,
4. twierdzić, że pro-rata jest implemented,
5. twierdzić, że proof system jest complete,
6. przywracać AI marketplace jako baseline,
7. mieszać compute miner i validator jako jedną rolę,
8. mówić o public discovery jako default,
9. mówić o public provider profile jako default,
10. mówić o quality-of-answer settlement.

## Wynik końcowy

Na końcu napisz:

- czy edytowałeś pliki,
- które pliki przeczytałeś,
- czy masz pytania blokujące,
- czy jesteś gotowy na kolejny prompt.
```

---

## Prompt 2 — Compute Lease Settlement Understanding

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest KONTYNUACJA V0.
Nie dotykasz kodu.
Nie edytujesz plików.
Nie tworzysz docs.
To jest read-only settlement understanding task.

# P-T034-XIAOMI-02 — Compute Lease Settlement Understanding

## Rola

Masz zrozumieć settlement V0 bez wymyślania implementacji.

Masz rozdzielić:
- V0 direction,
- current code reality,
- future protocol follow-up.

## Source of truth

Przeczytaj tylko:

- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`

Nie czytaj kodu.
Nie czytaj legacy docs.

## Required response

Odpowiedz dokładnie w sekcjach:

### 1. Settlement Core Rule

Jednym akapitem wyjaśnij, co escrow rozlicza w V0.

### 2. Old vs V0

Tabela:

| Old | V0 |
|-----|----|

Uwzględnij:
- task contract,
- artifact delivery,
- proof of delivery,
- semantic review,
- provider,
- buyer,
- settlement primitive.

### 3. Settlement Outcomes

Opisz:
- full release,
- full refund,
- pro-rata split,
- penalty/slash direction,
- recovery after timeout.

Przy każdym oznacz:
`direction`, `current reality`, albo `future protocol follow-up`.

### 4. Three Legs Of Operatorless Settlement

Rozwiń:

```text
receipt validation
lease policy binding
escrow note splitting
```

Dla każdego napisz:
- po co jest,
- czego jeszcze brakuje,
- jaki błąd agent może popełnić.

### 5. Open Follow-Ups

Wypisz minimum 7 rzeczy, których ten dokument nie zamyka.

### 6. Czego nie sprawdziłem

Wypisz uczciwie.

## Forbidden

Nie wolno:
1. definiować receipt schema,
2. definiować Rust structs,
3. definiować new tx types,
4. twierdzić, że pro-rata działa dziś,
5. twierdzić, że operatorless działa dziś,
6. twierdzić, że ledger waliduje receipts dziś,
7. robić planu implementacji,
8. dotykać kodu.

## Wynik końcowy

Na końcu napisz:

- czy edytowałeś pliki,
- które docs przeczytałeś,
- czy rozumiesz różnicę direction/current/future,
- czy masz pytania blokujące.
```

---

## Prompt 3 — Operatorless Bridge Reasoning

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest KONTYNUACJA V0.
Nie dotykasz kodu.
Nie edytujesz plików.
Nie tworzysz docs.
To jest focused architecture discussion.

# P-T034-XIAOMI-03 — Operatorless Bridge Reasoning

## Rola

Masz rozwinąć myślenie o przejściu:

```text
Phase 0 -> Phase 1 -> Phase 2
```

Nie masz projektować kodu.
Nie masz pisać final speca.
Masz pomóc uchwycić ryzyka i kolejność.

## Source of truth

Przeczytaj tylko:

- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`

## Working assumptions

Przyjmij:

```text
Phase 0: current 2-of-3 operator path remains current code reality.
Phase 1: automated operator validates receipts mechanically and co-signs.
Phase 2: protocol-level receipt validation removes operator from normal settlement.
```

Nie proponuj Phase 0 -> Phase 2 skip jako default.

## Required response

Odpowiedz dokładnie w sekcjach:

### 1. Why Phase 1 Is Required

5-8 punktów.

### 2. What Automated Operator Validates

Lista walidacji direction-level, bez wire format.

### 3. How To Prevent Phase 1 Becoming Permanent Centralization

5-8 punktów.

### 4. Phase 2 Entry Criteria

Co musi być prawdą, żeby operatorless settlement można było wprowadzić.

### 5. Agent Failure Modes

Jak agenci mogą to źle zrozumieć.

### 6. Questions For Future Opus Spec

Maksymalnie 5 pytań, które warto dać Opusowi 15 kwietnia.

### 7. Czego nie sprawdziłem

Wypisz uczciwie.

## Forbidden

Nie wolno:
1. twierdzić, że Phase 1 istnieje w kodzie,
2. twierdzić, że Phase 2 istnieje w kodzie,
3. zmieniać current escrow mechanics,
4. projektować konkretnego operator service API,
5. definiować receipt schema,
6. definiować ledger validation logic,
7. używać operatora jako final trust anchor.

## Wynik końcowy

Na końcu napisz:

- czy edytowałeś pliki,
- które docs przeczytałeś,
- czy odpowiedź jest direction-level,
- czy masz pytania blokujące.
```

---

## Prompt 4 — Identity And Private Discovery Reasoning

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest KONTYNUACJA V0.
Nie dotykasz kodu.
Nie edytujesz plików.
Nie tworzysz docs.
To jest focused architecture discussion.

# P-T034-XIAOMI-04 — Identity And Private Discovery Reasoning

## Rola

Masz rozwinąć myślenie o identity i private discovery w V0.

Nie masz definiować credential schema.
Nie masz wybierać final discovery architecture.
Masz wskazać ryzyka, minimalne dane i antywzorce.

## Source of truth

Przeczytaj tylko:

- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`

## Core assumptions

```text
Identity = hidden root credential + scoped role/session/epoch identities.
Falcon is a signing tool, not the whole identity.
Discovery is private/encrypted/credential-gated by default.
Public discovery is lower-privacy opt-in, not baseline.
```

## Required response

Odpowiedz dokładnie w sekcjach:

### 1. Identity Layers

Opisz:
- hidden root,
- role identity,
- epoch identity,
- session identity,
- scoped offering ID.

### 2. What Falcon Is And Is Not

Wyjaśnij granicę:
- Falcon jako podpis,
- Falcon nie jako public marketplace identity.

### 3. Minimal ComputeOffering Discovery Data

Wypisz minimalne dane potrzebne do discovery.

### 4. Data That Must Not Be Public

Wypisz dane, których discovery nie może ujawniać jako baseline.

### 5. Discovery Architecture Tradeoffs

Porównaj direction-level:
- encrypted registry,
- mailbox-based query,
- local trusted bootstrap list,
- gossip,
- DHT.

Nie wybieraj final architecture.

### 6. Risks For Agents

5 punktów.

### 7. Czego nie sprawdziłem

Wypisz uczciwie.

## Forbidden

Nie wolno:
1. definiować credential wire format,
2. definiować ComputeOffering struct,
3. wybierać final discovery architecture,
4. przywracać public provider profile,
5. przywracać public marketplace search,
6. przywracać public reputation leaderboard,
7. twierdzić, że current code ma hidden root identity.

## Wynik końcowy

Na końcu napisz:

- czy edytowałeś pliki,
- które docs przeczytałeś,
- czy odpowiedź jest direction-level,
- czy masz pytania blokujące.
```

---

## Prompt 5 — V0 Production Phase Plan Sanity

```text
Pracujesz tylko w WSL.

Repo:
`/home/nxms-server/privAI`

Nie pracuj na ścieżkach Windows.
Nie używaj `C:\...`.

To jest KONTYNUACJA V0.
Nie dotykasz kodu.
Nie edytujesz plików.
Nie tworzysz docs.
To jest sanity-check faz produkcyjnych.

# P-T034-XIAOMI-05 — V0 Production Phase Plan Sanity

## Rola

Masz ocenić fazy produkcyjne V0 z punktu widzenia spójności i ryzyka.

Nie masz pisać planu implementacji.
Nie masz zmieniać docs.
Masz wskazać, czy kolejność faz ma sens i gdzie są największe ryzyka.

## Source of truth

Przeczytaj tylko:

- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`
- `/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`

## Required response

Odpowiedz dokładnie w sekcjach:

### 1. Phase Summary

Streść fazy:
- Phase 0,
- Phase 1,
- Phase 2,
- Phase 3,
- Phase 4,
- Phase 5.

### 2. Does The Order Make Sense

Oceń kolejność. Jeśli coś wymaga przesunięcia, powiedz konkretnie co i dlaczego.

### 3. Biggest Phase Risks

Dla każdej fazy podaj 1-2 ryzyka.

### 4. What Must Not Be Done Too Early

Wypisz rzeczy, których nie wolno robić przed czasem.

### 5. What Can Be Discussed Safely Before Opus Returns

Wypisz tematy bezpieczne do rozmowy do 15 kwietnia.

### 6. Suggested Questions For Opus After 2026-04-15

Maksymalnie 7 pytań.

### 7. Czego nie sprawdziłem

Wypisz uczciwie.

## Forbidden

Nie wolno:
1. zmieniać faz,
2. pisać code tasks,
3. pisać protocol specs,
4. twierdzić, że docs tree jest final production plan,
5. twierdzić, że current code już odpowiada V0,
6. rekomendować implementacji przed docs/spec freeze,
7. ignorować guardrail: no old AI marketplace baseline.

## Wynik końcowy

Na końcu napisz:

- czy edytowałeś pliki,
- które docs przeczytałeś,
- czy masz pytania blokujące,
- czy fazy są gotowe do Opus review.
```

---

## Usage Order

```text
1. P-T034-XIAOMI-01 — Context Lock
2. P-T034-XIAOMI-02 — Settlement Understanding
3. P-T034-XIAOMI-03 — Operatorless Bridge
4. P-T034-XIAOMI-04 — Identity And Private Discovery
5. P-T034-XIAOMI-05 — Production Phase Plan Sanity
```
