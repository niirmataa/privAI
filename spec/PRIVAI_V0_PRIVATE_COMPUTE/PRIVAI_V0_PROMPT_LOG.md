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

**Status:** prompt-ready

---

## Tracking Rule

For V0 prompts, update this file when a prompt is issued or a result is received.

Do not update the old root `PROMPT_LOG.md` for V0 work unless explicitly requested.
