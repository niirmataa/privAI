# Agent Skills and Best Use

This file is not about formal permissions. It is about practical strengths.

## Xiaomi

Best at:
- bounded runtime tasks,
- node integration points,
- retry/ack/reload/persistence work,
- strong negative-test coverage,
- cleanup passes after a good architectural prompt.

Prompt Xiaomi with:
- strict write scope,
- strict forbidden list,
- strict Definition of Done,
- exact final report format.

Do not prompt Xiaomi with:
- vague "explore the repo" requests,
- huge multi-module redesigns without boundaries.

## Gemini

Best at:
- cross-layer integration,
- Stage A / Stage B boundary work,
- proof/submit/import glue,
- honest e2e tests,
- architectural consistency inside a bounded feature path.

Prompt Gemini with:
- clear system goal,
- deep but bounded reading list,
- explicit honesty constraints ("do not overclaim"),
- explicit success condition.

## Claude

Best at:
- architecture memos,
- readiness docs,
- freeze docs,
- drift analysis,
- product-state documentation.

Prompt Claude with:
- strong document goal,
- exact required sections,
- concrete source-of-truth docs,
- explicit write scope limited to docs when possible.

## Ampere / strongest model

Best at:
- hardest glue-layer decisions,
- cross-module boundary review,
- difficult consistency problems,
- turning partial implementation into coherent system shape.

Use when:
- the next move is technically risky,
- multiple modules interact,
- or the exact contract between layers is still muddy.

## Shared Prompting Rules

Every good task should include:
- role,
- problem,
- write scope,
- source of truth,
- code to read,
- required changes,
- forbidden shortcuts,
- Definition of Done,
- minimal commands,
- exact final report format.

Good prompts maximize:
- context,
- constraints,
- honesty,
- ownership.

Good prompts do **not**:
- hand the whole implementation to the model,
- leave scope fuzzy,
- or let the model guess what "done" means.
