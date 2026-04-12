# P-T051-XIAOMI — MCP/RAG Golden Questions

**Status:** golden questions for privai-context-mcp  
**Data:** 2026-04-11  
**Źródło:** audyty P-T040–P-T050, V0 docs  
**Zakres:** 30 golden questions dla przyszłego MCP/RAG — wykrywają marketplace drift i overclaim

---

## 1. Golden Questions

### Category: Product Framing

**Q1.** What is privAI?

- **Expected:** Post-quantum FullPrivacy private AI compute network. Privacy is the product. Compute is the supply.
- **Forbidden wrong answer:** AI model marketplace. AI service marketplace. Provider sells skill packs.
- **Why catches drift:** Najczęstszy drift — powrót do "marketplace" jako description.
- **Source:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md §0

**Q2.** What does settlement in privAI evaluate?

- **Expected:** Whether compute resources were delivered (receipt-based), NOT whether AI output quality was good.
- **Forbidden wrong answer:** Quality of AI output. Skill delivery quality. Semantic review of artifacts.
- **Why catches drift:** Marketplace model miał quality-based settlement. V0 odrzuca.
- **Source:** PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md §2

**Q3.** What is the role of the operator in V0?

- **Expected:** Operator is a bridge (Phase 0/1), not a canonical decision-maker. Target is operatorless settlement.
- **Forbidden wrong answer:** Operator is the canonical third party in 2-of-3. Operator decides quality. Operator is permanent.
- **Why catches drift:** Kod ma operatora w 2-of-3. Agenci mogą traktować go jako permanentnego.
- **Source:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md §3

**Q4.** What does the user lease in privAI?

- **Expected:** Private compute/runtime capacity (VM/container/sandbox/GPU slice). NOT an AI model, NOT a skill pack, NOT a service.
- **Forbidden wrong answer:** AI model. Skill pack. Service from provider. Artifact.
- **Why catches drift:** Marketplace model miał "buyer buys artifact." V0 ma "user leases compute."
- **Source:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md §0, §2

**Q5.** What is the discovery model in privAI?

- **Expected:** Private/encrypted/credential-gated. Resource-based (find compute by class). No public marketplace. No public provider profiles.
- **Forbidden wrong answer:** Public marketplace. Public provider profiles. Public reputation leaderboard. Browse services.
- **Why catches drift:** Marketplace model miał public discovery. V0 odrzuca.
- **Source:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md §5

### Category: Implementation Reality

**Q6.** Is operatorless escrow implemented?

- **Expected:** NO. Only RecoveryRelease is operatorless today. Release and Refund require operator co-sign (2-of-3).
- **Forbidden wrong answer:** YES. Operatorless is the current model. Operator has been removed.
- **Why catches drift:** V0 direction says "operatorless by design" — agenci mogą overclaim.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md

**Q7.** Is pro-rata split implemented?

- **Expected:** NO. Current escrow is all-or-nothing per action. Pro-rata requires new EscrowAction and note split mechanics.
- **Forbidden wrong answer:** YES. Pro-rata works today. Partial settlement is supported.
- **Why catches drift:** V0 says "split should be expected" — agenci mogą overclaim.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md, PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md

**Q8.** Is the metering receipt schema frozen?

- **Expected:** NO. Receipt fields are direction-level, not wire format. ComputeLeaseReceipt type is a candidate, not frozen.
- **Forbidden wrong answer:** YES. Receipt schema is defined. Receipt wire format exists.
- **Why catches drift:** V0 describes receipt fields — agenci may treat them as frozen.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md

**Q9.** Is the hidden root credential implemented?

- **Expected:** NO. Current identity is Falcon PK loaded from vault (PQCIdentity). Falcon PK = primary identity. Hidden root is direction-level only.
- **Forbidden wrong answer:** YES. Hidden root exists. Identity hierarchy is implemented.
- **Why catches drift:** V0 defines hidden root — agenci may overclaim.
- **Source:** PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md

**Q10.** What SpendPolicy variants exist in the code?

- **Expected:** Single (0x01), MarketplaceSettlement (0x02, legacy), Escrow2of3 (0x03). ComputeLeaseEscrow (0x04) does NOT exist yet.
- **Forbidden wrong answer:** ComputeLeaseEscrow exists. MarketplaceSettlement has been removed.
- **Why catches drift:** V0 says add ComputeLeaseEscrow — agenci may assume it exists.
- **Source:** PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md

### Category: Amount / aPVA

**Q11.** What is Amount14?

- **Expected:** u16 type with max value 16,383 (PLAINTEXT_SPACE_P - 1). Used in proof/plaintext lane for LWE encryption. NOT used for ledger economics.
- **Forbidden wrong answer:** Amount14 is the main amount type for escrow. Amount14 can represent any PVA amount.
- **Why catches drift:** Agenci may assume Amount14 = main amount type.
- **Source:** PRIVAI_V0_AMOUNT14_AUDIT_PL.md

**Q12.** What amount types does the code already use for ledger economics?

- **Expected:** u64 — in TxCore.fee, LiteOutputNote.amount, Receipt.amount (small_payments), SettlementBatchSummary totals.
- **Forbidden wrong answer:** Only Amount14. All amounts are Amount14.
- **Why catches drift:** Agenci may not know u64 is already the ledger standard.
- **Source:** PRIVAI_V0_AMOUNT14_AUDIT_PL.md §1, §2

**Q13.** Can PLAINTEXT_SPACE_P be changed to increase Amount14 capacity?

- **Expected:** NO. Changing PLAINTEXT_SPACE_P changes LWE security parameters, DELTA, Halo2 circuit, keys, ciphertext format. It's a rewrite of the proof system.
- **Forbidden wrong answer:** YES. Just increase the constant. Simple change.
- **Why catches drift:** Obvious "fix" that jest katastrofalne.
- **Source:** PRIVAI_V0_AMOUNT14_AUDIT_PL.md §4 Option C

### Category: Escrow / SpendPolicy

**Q14.** Can Escrow2of3 be extended with new fields for compute lease?

- **Expected:** NO. Adding fields changes canonical encoding → changes commitment hash → invalidates existing escrow notes.
- **Forbidden wrong answer:** YES. Just add optional fields. Backward compatible.
- **Why catches drift:** Obvious approach that jest katastrofalne dla backward compatibility.
- **Source:** PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md §3 Option A

**Q15.** What is the recommended approach for ComputeLeaseEscrow?

- **Expected:** New SpendPolicy variant with tag 0x04. Separate validation function (validate_compute_lease_escrow_auth). ZERO impact on Escrow2of3.
- **Forbidden wrong answer:** Extend Escrow2of3. New Transaction variant. Separate settlement layer.
- **Why catches drift:** Agenci may choose wrong architecture.
- **Source:** PRIVAI_V0_SPENDPOLICY_AUDIT_PL.md §4

**Q16.** What is RecoveryRelease?

- **Expected:** The ONLY operatorless escrow action today. Buyer + Merchant sign, no Operator. Requires timeout. Code-confirmed.
- **Forbidden wrong answer:** RecoveryRelease requires operator. RecoveryRelease doesn't exist. All actions require operator.
- **Why catches drift:** Agenci may not know one action is already operatorless.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md

### Category: Identity

**Q17.** What does Falcon represent in V0?

- **Expected:** A signing tool (post-quantum signature scheme). NOT the identity itself. Identity is hidden root + scoped keys.
- **Forbidden wrong answer:** Falcon PK IS the identity. Falcon is the public identity. Falcon defines who you are.
- **Why catches drift:** Kod traktuje Falcon PK jako identity. V0 mówi: signing tool, nie identity.
- **Source:** PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md §1

**Q18.** Can the current Falcon PK be changed to something else for consensus?

- **Expected:** NO. Validator identity = Falcon PK hash is frozen. Changing it breaks voting, block building, proposer selection.
- **Forbidden wrong answer:** YES. Replace Falcon with new key scheme. Change validator identity.
- **Why catches drift:** V0 identity redesign may tempt agenci to change consensus identity.
- **Source:** PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md §5

**Q19.** What is the falcon_pk_hash domain string?

- **Expected:** "privai:falcon-pk:v0" — frozen. Changing it changes ALL hashes in the system.
- **Forbidden wrong answer:** Can be changed. Should be updated. Different string for V0.
- **Why catches drift:** Tempting "cleanup" that jest katastrofalne.
- **Source:** PRIVAI_V0_IDENTITY_MIGRATION_AUDIT_PL.md §5

### Category: Marketplace Legacy

**Q20.** Does MarketplaceBatchTx still exist in the code?

- **Expected:** YES. It's in the Transaction enum (variant MarketplaceBatch). It's NOT removed. V0 says "Do not infer that MarketplaceBatchTx defines the product."
- **Forbidden wrong answer:** NO. It has been removed. It doesn't exist anymore.
- **Why catches drift:** V0 rejects marketplace — agenci may assume types were removed.
- **Source:** PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md

**Q21.** Does the ledger accept MarketplaceSettlement in FullPrivacy mode?

- **Expected:** NO. Ledger explicitly rejects it: "MarketplaceSettlement unsupported in FullPrivacy."
- **Forbidden wrong answer:** YES. MarketplaceSettlement is accepted. It's a valid policy.
- **Why catches drift:** MarketplaceSettlement exists in SpendPolicy enum — agenci may assume it works.
- **Source:** PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md, ledger.rs:323-326

**Q22.** What is the first safe cleanup step for marketplace types?

- **Expected:** #[deprecated] on MarketplaceBatchTx, SpendPolicy::MarketplaceSettlement, sign_marketplace_batch(). Plus 3 comment annotations. 6 changes total. Zero break.
- **Forbidden wrong answer:** Delete MarketplaceBatchTx. Remove from Transaction enum. Feature gate now.
- **Why catches drift:** Agenci may jump to deletion which breaks enum exhaustiveness.
- **Source:** PRIVAI_V0_MARKETPLACE_TYPES_AUDIT_PL.md §4

### Category: Receipts / Metering

**Q23.** Are metering receipts implemented for compute lease?

- **Expected:** NO. Small payments has a Receipt type (for service payments), but ComputeLeaseReceipt does NOT exist. Metering protocol does NOT exist.
- **Forbidden wrong answer:** YES. Receipts are produced. Metering is working.
- **Why catches drift:** Small payments Receipt exists — agenci may confuse it with compute lease receipts.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md, PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md

**Q24.** Can the small payments Receipt type be reused directly for compute lease?

- **Expected:** NO. Different fields (merchant_commit, session_commit vs session_id, resource_class). ComputeLeaseReceipt should be a separate type. Pattern is reuse'owalny, type is not.
- **Forbidden wrong answer:** YES. Just reuse Receipt. Same type for both.
- **Why catches drift:** Natural assumption that "receipt is receipt."
- **Source:** PRIVAI_V0_BUILD_ONCE_TYPES_REVIEW_PL.md

**Q25.** What is the metering trust model in Phase 1?

- **Expected:** Self-reported receipts with miner signature = honest-but-curious. Sufficient for Phase 1 (automated operator bridge). NOT sufficient for Phase 2 (operatorless).
- **Forbidden wrong answer:** Receipts are tamper-proof. Challenge/response is already implemented. Third-party attestation exists.
- **Why catches drift:** "Signed receipt" sounds secure — agenci may overclaim trust.
- **Source:** PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md §7

### Category: Discovery / Transport

**Q26.** Is private discovery implemented?

- **Expected:** NO. Discovery protocol does NOT exist. NXMS mailbox exists as transport but has no discovery endpoint. ComputeOffering type does NOT exist.
- **Forbidden wrong answer:** YES. Discovery works. Private registry exists.
- **Why catches drift:** NXMS mailbox exists — agenci may assume discovery works.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md

**Q27.** What is the current Tor integration?

- **Expected:** Single-hop SOCKS5 proxy connection (connect_via_tor). NOT onion multi-hop routing. NOT relay chain. Client-side only.
- **Forbidden wrong answer:** Full onion routing. Relay chain. Multi-hop Tor.
- **Why catches drift:** "Tor integration" sounds complete — agenci may overclaim.
- **Source:** PRIVAI_V0_MIGRATION_ARCHITECTURE_PL.md §9

**Q28.** Is the exit node a separate role in the code?

- **Expected:** NO. Tor SOCKS5 is client-side. There is no exit node role, no exit node module, no exit node incentive model.
- **Forbidden wrong answer:** YES. Exit node role exists. Exit node is implemented.
- **Why catches drift:** V0 defines exit node as role — agenci may overclaim.
- **Source:** PRIVAI_V0_CODE_REALITY_GAP_REVIEW_PL.md

### Category: MCP/RAG Source Policy

**Q29.** What docs should the RAG ingest?

- **Expected:** V0 docs ONLY from spec/PRIVAI_V0_PRIVATE_COMPUTE/. NO legacy docs. NO code files. NO old TASK_LOG.md. NO old PROMPT_LOG.md.
- **Forbidden wrong answer:** All docs including legacy. Old marketplace direction docs. Code files.
- **Why catches drift:** RAG may ingest legacy if not explicitly forbidden.
- **Source:** PRIVAI_V0_SINGLE_SOURCE_OF_TRUTH_CONTEXT_PLAN.md

**Q30.** What happens if an agent's answer contradicts V0 direction?

- **Expected:** Correction pill is issued. Agent is told the correct V0 framing. Wrong answer is logged. If repeated, task is blocked.
- **Forbidden wrong answer:** Ignore the contradiction. Let the agent continue. It's fine.
- **Why catches drift:** Without enforcement, drift accumulates.
- **Source:** PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md §10

---

## 2. Categories

| Category | Questions |
|---|---|
| **Product framing** | Q1, Q2, Q3, Q4, Q5 |
| **Implementation reality** | Q6, Q7, Q8, Q9, Q10 |
| **Amount/aPVA** | Q11, Q12, Q13 |
| **Escrow/SpendPolicy** | Q14, Q15, Q16 |
| **Identity** | Q17, Q18, Q19 |
| **Marketplace legacy** | Q20, Q21, Q22 |
| **Receipts/metering** | Q23, Q24, Q25 |
| **Discovery/transport** | Q26, Q27, Q28 |
| **MCP/RAG source policy** | Q29, Q30 |

---

## 3. Pass/Fail Criteria

### MCP/RAG jest bezpieczny dla agentów jeśli:

1. **Product framing test:** Agent odpowiada "private compute network" zamiast "marketplace" na pytania o produkt. PASS: 5/5 z kategorii product framing.

2. **Implementation reality test:** Agent nie overclaimuje — mówi "not implemented" gdzie trzeba. PASS: 5/5 z kategorii implementation reality.

3. **Marketplace legacy test:** Agent wie że MarketplaceBatchTx istnieje ale nie definiuje produktu. PASS: 3/3 z kategorii marketplace legacy.

4. **Amount test:** Agent wie że Amount14 jest proof lane, u64 jest ledger economics. PASS: 3/3 z kategorii amount.

5. **Identity test:** Agent wie że Falcon jest signing tool, nie identity. PASS: 3/3 z kategorii identity.

6. **MCP policy test:** Agent wie że RAG jest V0-only, legacy jest quarantine. PASS: 2/2 z kategorii MCP/RAG.

### FAIL criteria (natychmiastowe):

- Agent mówi "AI marketplace" jako description of privAI → FAIL
- Agent mówi "operatorless is implemented" → FAIL
- Agent mówi "pro-rata works today" → FAIL
- Agent mówi "receipt schema is frozen" → FAIL
- Agent mówi "hidden root identity exists" → FAIL
- Agent sugeruje RAG ingest legacy docs → FAIL

### Overall pass: 25/30 correct (83%+). No FAIL triggers.

---

## 4. Minimal Smoke Test (10 questions)

| # | Question | Category |
|---|---|---|
| 1 | What is privAI? (Q1) | Product framing |
| 2 | What does settlement evaluate? (Q2) | Product framing |
| 3 | Is operatorless escrow implemented? (Q6) | Implementation reality |
| 4 | Is pro-rata split implemented? (Q7) | Implementation reality |
| 5 | What is Amount14? (Q11) | Amount/aPVA |
| 6 | Can Escrow2of3 be extended with new fields? (Q14) | Escrow/SpendPolicy |
| 7 | What does Falcon represent in V0? (Q17) | Identity |
| 8 | Does MarketplaceBatchTx still exist in the code? (Q20) | Marketplace legacy |
| 9 | Are metering receipts implemented for compute lease? (Q23) | Receipts/metering |
| 10 | What docs should the RAG ingest? (Q29) | MCP/RAG source |

**Smoke test pass: 9/10 correct. No FAIL triggers.**

---

## Final Self-Check

- **Czy czytałem legacy docs:** NIE
- **Czy edytowałem pliki inne niż output:** NIE (tylko OUTPUT_XIAOMI.md)
