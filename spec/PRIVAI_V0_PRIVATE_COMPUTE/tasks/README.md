# privAI V0 Tasks Workspace

**Status:** non-canonical task workspace
**Scope:** prompt files, model outputs, review notes, task status
**Canonical docs live in:** `spec/PRIVAI_V0_PRIVATE_COMPUTE/`

This folder is the working area for model tasks.

It is not the canonical V0 source of truth.
It is where prompts, model outputs, and reviews are collected before anything is promoted into the main V0 docs.

---

## Core Rule

```text
tasks/ = workspace
main V0 folder = accepted docs
```

Do not treat raw model output as final architecture.

Every output must be reviewed before any accepted V0 doc is created from it.

---

## Folder Pattern

Each task gets one folder:

```text
tasks/T-0XX_SHORT_NAME/
├── PROMPT.md
├── OUTPUT_XIAOMI.md
├── REVIEW_CODEX.md
└── STATUS.md
```

Minimum required files:

- `PROMPT.md`
- `OUTPUT_<MODEL>.md`
- `STATUS.md`

Use model-specific output names when the task is not for Xiaomi, for example:

- `OUTPUT_XIAOMI.md`
- `OUTPUT_GEMINI.md`
- `OUTPUT_CLAUDE.md`

`REVIEW_CODEX.md` is added after the output is reviewed.

---

## How To Use

Give the model only the prompt path, for example:

```text
Execute:
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/PROMPT.md

Write output to:
spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/OUTPUT_XIAOMI.md
```

The prompt itself must also contain its required output path.

---

## Status Values

Use only:

- `prompt_ready`
- `issued`
- `output_received`
- `reviewed`
- `accepted_to_canonical`
- `rejected`
- `superseded`

---

## Source Policy

V0 tasks must use:

```text
spec/PRIVAI_V0_PRIVATE_COMPUTE/
```

V0 tasks must not use:

- legacy docs,
- old marketplace docs,
- old root `TASK_LOG.md`,
- old root `PROMPT_LOG.md`.

If implementation truth is needed, read current code and tests directly.

---

## Pre-Tasks Audit Map

Some Xiaomi audits were created before this `tasks/` workspace existed.

Those tasks do not have `tasks/T-040...` folders. Their accepted working outputs are stored as V0 audit docs:

| Task Ref | Saved Output |
|---|---|
| `P-T040-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md` |
| `P-T041-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md` |
| `P-T042-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md` |
| `P-T043-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md` |
| `P-T044-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md` |
| `P-T045-XIAOMI` | `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md` |

If a model cites `P-T040` through `P-T045`, it must cite the saved audit doc above, not a non-existent task folder.

---

## Reviewer Policy

No model is the canonical authority by itself.

Opus is not a blocking dependency.

If Opus/Claude is available later, its output is an additional reviewer artifact, not a required gate.

Current review flow:

```text
saved V0 docs/audits -> Xiaomi synthesis -> Codex sanity review -> Gemini adversarial review -> Operator decision -> canonical doc
```

Use `blocked by spec/reviewer decision` instead of `blocked by Opus` unless a task is explicitly assigned to Opus/Claude.

---

## Promotion Rule

Raw task output is not canonical.

Promotion flow:

```text
PROMPT.md -> OUTPUT_XIAOMI.md -> REVIEW_CODEX.md -> accepted V0 doc
```

Only accepted V0 docs should be used as stable source material by future tasks.
