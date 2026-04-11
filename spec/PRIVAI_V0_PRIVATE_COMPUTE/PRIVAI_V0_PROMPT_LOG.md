# privAI V0 Prompt Log

**Status:** canonical prompt log for the V0 private compute direction
**Started:** 2026-04-11
**Scope:** `spec/PRIVAI_V0_PRIVATE_COMPUTE`

This log starts at the V0 reset.

Do not continue the old root `PROMPT_LOG.md` for V0 work. The old root log belongs to the pre-V0 escrow/mailbox track.

---

## Prompt Log

### V0-000 | 2026-04-11 | Operator + Codex | V0 Direction Reset

**Goal:** Reset product/business framing from AI marketplace to private AI compute network.

**Output:**

- `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- superseded / index / start updates in legacy handoff docs
- checkpoint commit `ea38d4b`

**Status:** done

---

### V0-004 | 2026-04-11 | Codex | Legacy Zero-Use Tightening

**Goal:** Enforce the operator decision that V0 code work must not use legacy docs at all.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Result:**

- Legacy docs are now historical archive only for V0.
- V0 code-touching tasks must read code/tests and V0 docs, not legacy docs.
- RAG/MCP must not ingest legacy docs.
- New direction/spec/planning/code-landing docs must be written under `spec/PRIVAI_V0_PRIVATE_COMPUTE/`.

**Status:** done

---

### V0-005 | 2026-04-11 | Codex | privai-context-mcp Direction

**Goal:** Capture the V0-only direction for a future `privai-context-mcp` server before any implementation task.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Result:**

- Working name frozen directionally as `privai-context-mcp`.
- v1 purpose limited to onboarding, task-context packs, and truth-layer guardrails.
- v1 source policy is V0-only.
- v1 has eight read-only tools and no legacy lookup.
- implementation is explicitly deferred until a dedicated task.

**Status:** done

---

### P-T023-OPUS | 2026-04-11 | Opus | Senior Review Memo + Rewrite Plan

**Goal:** Review V0 master doc, answer whether it closes the six architecture gaps at direction level, classify legacy docs, and define rewrite order.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md`

**Status:** done

---

### T-030-OPUS | 2026-04-11 | Opus | V0-Aligned Diagrams

**Goal:** Create V0 visual companion without touching legacy diagrams.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIAGRAMS.md`

**Result:** 15 diagrams, 14 canonical V0 direction, 1 future strengthening.

**Status:** done

---

### T-031-OPUS | 2026-04-11 | Opus | Compute Lease Settlement Direction

**Goal:** Replace old contract/quality/artifact settlement mental model with compute lease policy, receipts, metering, timeout rules, pro-rata direction, and operatorless target.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`

**Guardrails:**

- no code changes,
- no legacy docs changes,
- no wire format definitions,
- no claim that pro-rata/operatorless settlement is implemented.

**Status:** done

---

### P-T032-XIAOMI | 2026-04-11 | Xiaomi | V0 Onboarding + Focused Discussion

**Goal:** Read the V0 folder and prepare for a focused architecture discussion from a fresh model perspective.

**Write scope:** none.

**Read-only scope:**

- `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `PRIVAI_V0_LEGACY_DOCSET_REWRITE_PLAN.md`
- `PRIVAI_V0_DIAGRAMS.md`
- `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- `TASK_031_OPUS_WORKING_CONTEXT.md`

**Status:** prompt-ready, not yet reported as completed.

---

### P-T033-XIAOMI | 2026-04-11 | Xiaomi | V0 Deep Discussion

**Goal:** Develop Xiaomi's V0 read-only perspective on three hard topics:

- Phase 0 -> Phase 1 -> Phase 2 operatorless bridge,
- aPVA precision and integer settlement,
- private discovery architecture tradeoffs.

**Result:**

- Phase 1 automated operator should be treated as required lab, not optional shortcut.
- Operatorless settlement depends on three legs:
  - receipt validation,
  - lease policy binding,
  - escrow note splitting.
- `1 PVA = 10^12 aPVA` is a strong candidate but requires supply/type decision.
- Integer pro-rata should use `miner_share = total_aPVA * delivered_units / committed_units`, with remainder to user.
- Receipt availability and metering trust model are near-term hard risks.
- Private discovery likely starts with encrypted bootstrap/registry, then mailbox/encrypted query, later stronger decentralized discovery.

**Status:** done

---

### V0-002 | 2026-04-11 | Codex | V0 Docs Tree

**Goal:** Create a single map of all V0 docs needed from direction reset to production planning, and establish that only V0 logs are updated for the new track.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Status:** done

---

### P-T034-XIAOMI | 2026-04-11 | Codex | Five-Step V0 Context Prompt Pack

**Goal:** Save five strict read-only Xiaomi prompts that progressively lock V0 context without allowing code changes, wire-format invention, or legacy marketplace drift.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_XIAOMI_5_CONTEXT_PROMPTS.md`

**Prompt sequence:**

1. V0 Context Lock
2. Compute Lease Settlement Understanding
3. Operatorless Bridge Reasoning
4. Identity And Private Discovery Reasoning
5. V0 Production Phase Plan Sanity

**Status:** done; prompts 01-05 completed read-only.

---

### P-T034-XIAOMI-01 | 2026-04-11 | Xiaomi | V0 Context Lock

**Goal:** Force a fresh model to lock the canonical V0 context before any deeper reasoning.

**Read-only scope:**

- `PRIVAI_V0_DOCS_TREE.md`
- `PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
- `PRIVAI_V0_TASK_LOG.md`
- `PRIVAI_V0_PROMPT_LOG.md`

**Result:**

- Correctly restated privAI as a post-quantum FullPrivacy private AI compute network, not an AI marketplace.
- Correctly rejected public provider profiles, public marketplace discovery, quality-based settlement, operator-as-canonical, skill-pack marketplace core, artifact-delivery settlement, public reputation, and MarketplaceBatchTx as product definition.
- Correctly identified `PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md` as the next planned doc.
- Correctly stated that no code, legacy docs, git history, or implementation claims were checked.

**Status:** done; no files edited by Xiaomi.

---

### P-T034-XIAOMI-02 | 2026-04-11 | Xiaomi | Compute Lease Settlement Understanding

**Goal:** Verify that the model understands V0 settlement as compute lease / receipts / metering, not artifact quality.

**Read-only scope:**

- `PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md`
- selected V0 diagrams covering lease lifecycle, on-chain/off-chain boundary, escrow, and discovery

**Result:**

- Correctly restated settlement as proof of delivered runtime resources, not proof of AI answer quality.
- Correctly separated old superseded model from V0: task contract/artifact/semantic quality becomes lease policy/private compute session/receipt validity.
- Correctly separated current reality from V0 direction and future follow-ups:
  - current: 2-of-3 Release/Refund and operatorless RecoveryRelease after timeout,
  - direction: receipt-validated settlement,
  - future: pro-rata note splitting, receipt schema, lease policy struct, penalty/slash, automated operator, operatorless settlement.
- Correctly identified the three legs of operatorless settlement: receipt validation, lease policy binding, and escrow note splitting.
- Correctly warned that signed self-reported receipts are not enough to solve metering truth.

**Status:** done; no files edited by Xiaomi.

---

### P-T034-XIAOMI-03 | 2026-04-11 | Xiaomi | Operatorless Bridge Reasoning

**Goal:** Verify whether Xiaomi understands why Phase 1 automated operator exists before protocol-level operatorless settlement.

**Result:**

- Correctly framed Phase 1 as required receipt-validation lab before consensus/protocol commitment.
- Correctly preserved current 2-of-3 escrow mechanics during Phase 1.
- Correctly identified automated operator validation areas: receipt signature, session binding, resource class, duration coverage, duplicates, meter version, timeout, lease policy commitment, full/partial/no-delivery decision.
- Correctly warned that Phase 1 needs determinism, open-source reference implementation, auditability, multiple independent instances, and explicit kill criteria.
- Correctly listed Phase 2 entry criteria: frozen receipt schema, on-chain lease policy commitment, pro-rata-capable escrow mechanics, proven validation logic, timeout claim path, migration path, settlement formula, and sufficient metering trust model.

**Status:** done; no files edited by Xiaomi.

---

### P-T034-XIAOMI-04 | 2026-04-11 | Xiaomi | Identity And Private Discovery Reasoning

**Goal:** Verify whether Xiaomi understands hidden-root identity, scoped IDs, Falcon boundaries, and private discovery tradeoffs.

**Result:**

- Correctly separated hidden root credential, role identity, session identity, epoch identity, and scoped offering ID.
- Correctly stated that Falcon is a post-quantum signing tool, not public identity or public marketplace profile.
- Correctly identified minimal ComputeOffering discovery fields at direction level without defining a wire format.
- Correctly rejected public miner identity, lease history, public reliability leaderboard, workload contents, provider profile, public Falcon lookup key, default geo exposure, and public stake/bond amount.
- Correctly compared encrypted registry, mailbox-based query, local bootstrap list, gossip, and DHT as tradeoffs rather than final decisions.

**Status:** done; no files edited by Xiaomi.

---

### P-T034-XIAOMI-05 | 2026-04-11 | Xiaomi | V0 Production Phase Plan Sanity

**Goal:** Verify whether Xiaomi understands the V0 production phase order and can identify phase risks without writing specs.

**Result:**

- Correctly summarized Phase 0 through Phase 5.
- Confirmed the general order: direction, compatibility bridge, receipt-aware escrow, operatorless settlement, identity/discovery/transport hardening, production readiness.
- Identified useful risks: Phase 4 may bundle too many domains, Phase 2-to-3 dependencies may bottleneck, existing escrow migration is a major Phase 3 risk, and production phase naming may collide with escrow transition phases.
- Correctly warned not to write Tier 3 protocol specs too early, not to change escrow code too early, not to pick final discovery architecture too early, and not to declare Phase 0 complete before operatorless direction is addressed.
- Produced concrete questions for future Opus review.

**Status:** done; no files edited by Xiaomi.

---

### V0-003 | 2026-04-11 | Codex | Single Source Of Truth Context Plan

**Goal:** Make the future RAG/MCP/team context boundary explicit before any ingestion work.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Result:**

- V0 folder is now explicitly the active source of truth.
- Legacy docs are quarantined, not default V0 context.
- RAG/MCP must start V0-only and read-only.
- Gemini/Xiaomi, Claude/Opus, and Codex must share the same source policy and differ only by role.

**Status:** done

---

### P-T035-XIAOMI | 2026-04-11 | Codex | Whole-System Synthesis And Direction Proposals

**Goal:** Ask Xiaomi to summarize how it now sees the complete V0 system and propose direction-level solutions, without editing files or inventing protocol wire formats.

**Write scope:** none.

**Result:**

- Correctly described privAI V0 as a post-quantum FullPrivacy private AI compute network.
- Correctly explained the system in layers: product, roles, identity, compute lease/session, settlement/escrow, metering/receipts, transport, discovery, versioning, and agent/RAG/MCP context.
- Proposed direction-level solutions for operatorless bridge, receipt availability, metering trust, pro-rata settlement, aPVA precision, hidden-root identity, private discovery, runtime privacy classes, transport/mailbox privacy, exit nodes, and `privai-context-mcp`.
- Correctly separated docs/direction/protocol/code/devnet/production phases.
- Identified major risks: receipt validation, receipt availability, metering fraud, escrow note splitting, identity correlation, discovery metadata leakage, permanent operator bridge, legacy drift, and RAG contamination.
- Recommended next docs, led by operatorless escrow, metering, identity, node roles, aPVA, private discovery, runtime privacy classes, and protocol versioning.

**Status:** done; no files edited by Xiaomi.

---

### P-T036-XIAOMI | 2026-04-11 | Codex | Deep Architecture Conversation

**Goal:** Push Xiaomi from synthesis into deeper architectural tradeoff discussion while keeping the task read-only and direction-level.

**Write scope:** none.

**Result:**

- Xiaomi gave a positive architecture verdict: V0 is coherent at direction level and does not need a major direction change.
- The primary system risk was identified as `receipt truth`: signed miner self-reporting proves what the miner claimed, not what the miner actually delivered.
- Strongest design choices identified:
  - settlement based on resource delivery, not AI output quality,
  - operatorless escrow by design,
  - hidden-root plus scoped identities,
  - private discovery by default,
  - no quality disputes,
  - V0-only context layer with legacy quarantine.
- Weakest areas identified:
  - receipt truth,
  - receipt availability,
  - pro-rata note splitting,
  - identity correlation,
  - discovery metadata leakage,
  - transport metadata,
  - Phase 1 centralization,
  - RAG/MCP contamination.
- Direction-level decisions proposed for Opus/operator review:
  - challenge/response for Phase 2 receipt truth,
  - dual receipt storage plus on-chain commitment,
  - max PVA supply before aPVA type choice,
  - explicit Phase 1 kill criteria,
  - phase naming clarification,
  - discovery metadata threat model,
  - runtime privacy class hierarchy,
  - transport metadata hardening,
  - isolated pro-rata note split spec,
  - MCP golden tests before agent use,
  - identity before discovery,
  - aPVA before settlement formula freeze.
- Next Xiaomi discussions proposed:
  - Receipt Truth Under Adversarial Conditions,
  - The Economics of Privacy,
  - 10-Year Attack Surface Review.

**Status:** done; no files edited by Xiaomi.

---

### P-T037-XIAOMI | 2026-04-11 | Codex | Final V0 Architecture Proposal

**Goal:** Ask Xiaomi to propose the target V0 architecture after identifying what can kill the system, while staying read-only and direction-level.

**Write scope:** none.

**Result:**

- Xiaomi proposed an 11-layer final V0 architecture.
- Main system rule: chain sees commitments and receipts, not workloads, outputs, or people.
- Receipt truth was proposed as a three-layer model:
  - miner signed receipts,
  - optional user acknowledgments,
  - protocol-level challenge/response for final operatorless settlement.
- The answer separated on-chain, off-chain, transport, and RAG/MCP boundaries.
- The answer proposed final phasing, decisions to freeze before code, decisions to keep open, next docs, red lines, and blocking questions.

**Status:** done; no files edited by Xiaomi.

---

### V0-006 | 2026-04-11 | Codex | Polish Final Architecture Proposal

**Goal:** Save the P-T037-XIAOMI final architecture proposal as a Polish V0 direction document.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_FINAL_ARCHITECTURE_PROPOSAL_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Status:** done

---

### P-T038-XIAOMI | 2026-04-11 | Xiaomi | V0 Code Reality Gap Review

**Goal:** Compare V0 private compute direction against current code reality without using legacy docs or editing files.

**Result:**

- Xiaomi concluded that V0 direction is correct but far from current implementation.
- Current code strengths identified:
  - solid escrow 2-of-3,
  - Stage A/B boundary,
  - Falcon usage,
  - NXMS mailbox,
  - Halo2 scaffold,
  - escrow orchestrator as Phase 1 candidate.
- Major gaps identified:
  - no compute lease metering receipts,
  - no pro-rata note split,
  - Falcon public key currently functions as identity,
  - marketplace types still exist in code,
  - operator remains canonical for Release/Refund,
  - `Amount14` may fundamentally conflict with `aPVA` precision,
  - no private discovery,
  - no runtime privacy classes,
  - no challenge/response.
- Blocking decisions identified:
  - `Amount14` / amount type,
  - `MarketplaceBatchTx` fate,
  - `ComputeLeaseEscrow` vs extending `Escrow2of3`,
  - Falcon identity migration,
  - small payments receipt reuse vs separate compute lease receipt,
  - orchestrator as automated operator base.

**Status:** done; no files edited by Xiaomi.

---

### V0-007 | 2026-04-11 | Codex | Polish Code Reality Gap Review

**Goal:** Save the P-T038-XIAOMI code-reality review as a Polish V0 gap document.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Status:** done

---

### P-T039-XIAOMI | 2026-04-11 | Xiaomi | V0 Migration Architecture

**Goal:** Describe how to migrate from current code to V0 private compute without editing files.

**Result:**

- Xiaomi recommended an `add new alongside old` migration strategy.
- Existing mechanics should be kept or bridged:
  - `Escrow2of3`,
  - `RecoveryRelease`,
  - `EscrowApprovalBundle`,
  - Falcon signatures,
  - NXMS mailbox,
  - Halo2 scaffold,
  - escrow orchestrator.
- Legacy marketplace types should be deprecated/isolated, not immediately removed.
- New V0 primitives are needed:
  - `ComputeLeaseEscrow`,
  - compute lease receipts,
  - receipt/metering layer,
  - hidden-root identity,
  - private discovery,
  - runtime privacy classes,
  - pro-rata note splitting.
- `Amount14` should likely remain for proof/plaintext lane, while V0 needs a larger `LedgerAmount` directionally.
- `nxms-escrow-orchestrator` is the strongest candidate for Phase 1 automated operator bridge.

**Status:** done; no files edited by Xiaomi.

---

### V0-008 | 2026-04-11 | Codex | Polish Migration Architecture

**Goal:** Save the P-T039-XIAOMI migration architecture as a Polish V0 strategy document.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_TASK_LOG.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_PROMPT_LOG.md`

**Status:** done

---

### P-T040-XIAOMI | 2026-04-11 | Xiaomi | Domain Model Candidate Classification

**Goal:** Classify the candidate V0 domain model as `FROZEN_CANDIDATE`, `CANDIDATE`, `OPEN`, `BLOCKED_BY_CODE_AUDIT`, or `REJECTED`.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOMAIN_MODEL_CLASSIFICATION_PL.md`

**Status:** done

---

### P-T041-XIAOMI | 2026-04-11 | Xiaomi | Amount14 / LedgerAmount Audit

**Goal:** Audit `Amount14` usage against V0 `aPVA` / `LedgerAmount` requirements.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_AMOUNT14_AUDIT_PL.md`

**Key result:**

- `Amount14` appears to belong to the proof/plaintext lane, while V0 compute lease economics likely need a separate ledger amount representation.

**Status:** done

---

### P-T042-XIAOMI | 2026-04-11 | Xiaomi | SpendPolicy / Escrow Compatibility Audit

**Goal:** Audit whether V0 compute lease escrow should extend `Escrow2of3`, add a new `SpendPolicy`, add a new transaction, or use a separate settlement layer.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md`

**Status:** done

---

### P-T043-XIAOMI | 2026-04-11 | Xiaomi | Identity Migration Audit

**Goal:** Audit how current code uses Falcon identity and how it can migrate toward hidden root + scoped keys.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md`

**Key result:**

- Current code treats Falcon PK / `node_pk_hash` as the core system identifier.
- `falcon_pk_hash()` with domain `privai:falcon-pk:v0` is frozen for compatibility and must not be changed.
- `node_pk_hash` should remain unchanged for consensus and be reframed semantically as `ValidatorRoleKey` hash.
- Hidden root can be introduced later as an additive vault extension.
- Role separation should be added gradually without breaking consensus identity.
- The first safe step is documentation/comment-level clarification, not behavior change.

**Status:** done

---

### P-T044-XIAOMI | 2026-04-11 | Xiaomi | Marketplace Types Fate Audit

**Goal:** Audit what to do with marketplace-era types so they do not contaminate V0 private compute.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md`

**Status:** done

---

### P-T045-XIAOMI | 2026-04-11 | Xiaomi | Build-Once Domain Types Review

**Goal:** Review the build-once candidate types for necessity, timing, overengineering risk, and code-audit blockers.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md`

**Status:** done

---

### V0-009 | 2026-04-11 | Codex | Task Workspace Setup

**Goal:** Create a stable task-prompt workspace so future model tasks are referenced by path instead of pasted manually through chat.

**Write scope:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-047_DOMAIN_BOUNDARIES_FREEZE/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-048_MINIMAL_TYPES_FREEZE/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-049_IMPLEMENTATION_BLOCKERS/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-050_DOCS_DEPENDENCY_GRAPH/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-051_MCP_RAG_GOLDEN_QUESTIONS/`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-052_PRE_OPUS_BRIEF/`

**Result:**

- Each task folder contains `PROMPT.md`, `OUTPUT_XIAOMI.md`, and `STATUS.md`.
- Each prompt explicitly tells Xiaomi where to write `OUTPUT_XIAOMI.md`.
- `tasks/` is marked as non-canonical workspace.
- Main V0 docs remain the accepted/canonical doc area.

**Status:** done

---

### P-T053-GEMINI | 2026-04-11 | Gemini 3.1 Pro | Decision Matrix Cross-Review

**Goal:** Independently review Xiaomi T-046 and Codex review for overclaims, premature freezes, missing blockers, and legacy/marketplace drift.

**Prompt:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-053_GEMINI_DECISION_MATRIX_REVIEW/PROMPT.md`

**Output path:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-053_GEMINI_DECISION_MATRIX_REVIEW/OUTPUT_GEMINI.md`

**Status:** prompt_ready

---

### T-054-XIAOMI | 2026-04-11 | Xiaomi | Final Reviewer Brief Without Opus Gate

**Goal:** Replace the old pre-Opus brief with a model-neutral reviewer brief that treats Opus/Claude as optional future reviewer, not a blocking authority.

**Prompt:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/PROMPT.md`

**Output path:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/OUTPUT_XIAOMI.md`

**Status:** prompt_ready

---

### T-055-GEMINI | 2026-04-11 | Gemini 3.1 Pro | privai-context-mcp Sprint 1

**Goal:** Implement Sprint 1 of `privai-context-mcp` as a real local Rust stdio MCP server with 8 read-only V0 tools and file-backed V0-only indexes.

**Prompt:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-055_GEMINI_CONTEXT_MCP_SPRINT1/PROMPT.md`

**Output path:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-055_GEMINI_CONTEXT_MCP_SPRINT1/OUTPUT_GEMINI.md`

**Code write scope:**

- `privai-context-mcp/**`

**Status:** prompt_ready

---

### T-056-GEMINI | 2026-04-11 | Gemini 3.1 Pro | Vertex RAG Bridge Design

**Goal:** Design the future Vertex RAG backend boundary for `privai-context-mcp` without making Vertex a Sprint 1 implementation requirement.

**Prompt:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-056_VERTEX_RAG_BRIDGE_DESIGN/PROMPT.md`

**Output path:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-056_VERTEX_RAG_BRIDGE_DESIGN/OUTPUT_GEMINI.md`

**Status:** prompt_ready

---

### V0-010 | 2026-04-11 | Codex | Task Workflow Correction

**Goal:** Remove Opus as a blocking dependency from the task workflow and document where pre-task Xiaomi audits P-T040 through P-T045 are stored.

**Output:**

- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/README.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-046_DECISION_MATRIX_DRAFT/REVIEW_CODEX.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-053_GEMINI_DECISION_MATRIX_REVIEW/PROMPT.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/tasks/T-054_FINAL_REVIEWER_BRIEF/PROMPT.md`
- `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DOCS_TREE.md`

**Status:** done

---

## Tracking Rule

For V0 prompts, update this file when a prompt is issued or a result is received.

Do not update the old root `PROMPT_LOG.md` for V0 work unless explicitly requested.
