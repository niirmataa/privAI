# privAI v1 Production Path

This document explains what is still missing before `privAI` can be honestly described as a production-shape v1 system, and what path leads there.

Read it after:
- `PRIVAI_PROJECT_ENTRYPOINT.md`
- `PRIVAI_V1_READINESS_AND_GAPS.md`

## 1. What Already Exists As Real Foundation

`privAI` already has real system foundations:
- a real `FullPrivacy` model built around Option B,
- a real split between control-plane and execution-plane,
- a real split between Stage A and Stage B,
- real node-side Stage A runtime,
- real wallet-side final assembly,
- real typed proof handoff,
- real escrow-aware submit gate,
- real validator session transport hardening,
- a real honest local escrow `release` e2e.

This means the project is no longer just an architecture sketch.

## 2. What Foundations Are Still Missing For Production-Shape v1

To reach production-shape v1, five foundation areas are still incomplete.

### A. Runtime control-plane delivery

The NXMS mailbox control-plane path is not fully closed yet.

What is still needed:
- mailbox pull,
- payload decode,
- node ingest through `handle_nxms_payload(...)`,
- ack / no-ack policy,
- retry / duplicate / delayed-ack semantics.

This is currently the biggest runtime gap.

### B. Full escrow action coverage

`release` is already verified locally, but production-shape escrow needs all major action paths to be equally real:
- `release`
- `refund`
- `recovery_release`

Still needed:
- honest `refund` e2e,
- honest `recovery_release` e2e,
- explicit timeout enforcement verification,
- action-specific output/auth validation.

### C. Proof/runtime honesty

The proof layer is real, but still needs more operational closure.

Already real:
- typed proof handoff,
- proof attachment,
- artifact shape,
- import-ready flow.

Still needed:
- clearer operational story around prover runtime / sidecar behavior,
- stronger mismatch and artifact integrity tests,
- honest documentation of what is already runtime-ready and what is still partial.

The critical rule here is:
- do not overclaim full production prover runtime if what is real today is mainly architecture plus integration path.

### D. Runtime resilience and observability

Production-shape v1 also needs:
- stronger error taxonomy,
- retry semantics,
- restart/reload confidence,
- runtime logs and metrics,
- diagnosable submit / proof / mailbox failures.

Without this, the system may work functionally but still lack operational confidence.

### E. Truthful product freeze

The last missing foundation is documentary truthfulness:
- what is production-shape,
- what is still evolving,
- what is still scaffolding,
- what must not be marketed or described as "done" yet.

This matters because `privAI` is already substantial enough that overclaiming is now a real risk.

## 3. Minimal Production Baseline For v1

The most honest minimum baseline for production-shape v1 is:

### 1. Escrow action paths

All three major escrow paths must be honestly validated:
- `release`
- `refund`
- `recovery_release`

### 2. Runtime delivery

The NXMS mailbox path must be:
- implemented,
- verified,
- semantically documented,
- clearly separated from validator P2P.

### 3. Stage A / Stage B contract

The contract between control-plane and final assembly must be:
- explicit,
- tested,
- frozen for v1.

### 4. Proof path

The proof path must be:
- typed,
- integration-ready,
- consistent,
- truthfully described in terms of what runtime already exists and what does not.

### 5. Operational confidence

There must be:
- retry/error semantics,
- restart/reload confidence,
- minimum observability,
- enough diagnostics to debug runtime failures without guesswork.

Without these five things, the system is real but not yet fully production-shape.

## 4. Recommended Path To Production-Shape v1

The most sensible path forward is:

### Step 1
- finish and verify the mailbox runtime loop

### Step 2
- deliver honest escrow `refund` e2e

### Step 3
- deliver honest escrow `recovery_release` e2e

### Step 4
- verify timeout enforcement and action-specific hardening

### Step 5
- freeze the Stage A / Stage B contract after the real paths exist

### Step 6
- deliver retry/ack/resilience/observability hardening

### Step 7
- write final truthful production-freeze docs

This is the path a new agent should treat as the main direction of the project.

## 5. What Should Not Be Treated As Absolute Blockers

Not everything must be strictly sequential.

For example:
- mailbox runtime is the biggest runtime gap,
- but local escrow `refund` e2e is not necessarily a hard dependency on mailbox,
- and some hardening work can happen in parallel.

So an agent must distinguish between:
- hard dependency,
- recommended order,
- parallelizable work.

False dependencies are one of the easiest ways to waste time in this project.

## 6. What A New Agent Must Not Guess

A new agent should not assume:
- mailbox runtime is already done just because files exist,
- proof runtime is fully production-ready just because typed handoff exists,
- `release` automatically implies `refund` and `recovery_release`,
- `nexum-core` may own final execution semantics,
- validator P2P and mailbox are just two variants of the same runtime layer.

Those distinctions are part of the system foundation and must remain explicit.

## 7. One-Line Definition Of Current State

`privAI` already has real production foundations, but honest production-shape v1 still requires mailbox runtime closure, full escrow action coverage, timeout/retry hardening, proof/runtime honesty, and final truth-doc freeze.
