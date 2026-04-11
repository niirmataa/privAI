# privAI Docs Index

Use this as a map, not as a narrative.

Recommended read order:
1. `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md`
2. `PRIVAI_PROJECT_ENTRYPOINT.md`
3. `PRIVAI_V1_READINESS_AND_GAPS.md`
4. `PRIVAI_V1_PRODUCTION_PATH.md`
5. `PRIVAI_NEXT_DIRECTION.md`
6. `PRIVAI_CHAT_ARCHIVE_2026-04-09.md`
7. Deep specs below

## Core Escrow / Privacy Specs

### `PRIVAI_ESCROW_OBJECT_MODEL.md`
Read when you need:
- the escrow object model,
- Stage A vs Stage B semantics,
- bundle/proposal/funding relationships.

### `PRIVAI_ESCROW_TX_MATRIX.md`
Read when you need:
- action-by-action tx expectations,
- `release` / `refund` / `recovery_release` semantics,
- output/authorization matrix.

### `PRIVAI_ESCROW_PROOF_INTEGRATION.md`
Read when you need:
- proof handoff expectations,
- proving-data relationships,
- artifact flow.

### `PRIVAI_ESCROW_FULLPRIVACY_BOUNDARY_DECISION_MEMO.md`
Read when you need:
- the core privacy/auth decision,
- why Option B was chosen,
- why `policy_opening` is mandatory.

## V0 Direction Reset (2026-04-11) — READ FIRST

### `spec/PRIVAI_V0_PRIVATE_COMPUTE/PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md`
**This is the highest-authority direction document.**
Read when you need:
- the canonical system model (private compute network, not AI marketplace),
- identity model, recovery/settlement, fees/incentives, discovery, protocol versioning,
- terminology reset (user/compute lessee, compute miner, operatorless escrow).

This doc supersedes marketplace/product framing in all older docs below.

---

## Production Direction (2026-04-10) — partially superseded

### `PRIVAI_PRODUCTION_SYSTEM_DIRECTION.md`
Read when you need:
- the single production direction (22 frozen decisions),
- execution modes, pricing, operator model, sandbox network,
- rollout phases, contract model, on-chain privacy model.

### `PRIVAI_MARKETPLACE_CHAIN_BOUNDARY_FREEZE.md`
Read when you need:
- the canonical boundary between marketplace and chain,
- why marketplace is off-chain but escrow remains on-chain,
- why the canonical chain path is FullPrivacy-first,
- why private discovery is the baseline and public discovery is lower-privacy opt-in,
- why `MarketplaceBatchTx` is optional aggregate rail, not the FullPrivacy marketplace baseline.

### `PRIVAI_PRODUCTION_SYSTEM_DIAGRAMS.md`
Read when you need:
- visual companion to the direction doc (19 diagrams),
- quick scan of architecture, settlement, verification, network topology.

### `PRIVAI_STAGE_A_STAGE_B_CONTRACT_FREEZE.md`
Read when you need:
- code-confirmed escrow boundary (Stage A vs Stage B),
- 10-point submit gate validation,
- sentinel vs final tx_signing_hash.

### `PRIVAI_HALO2_PROOF_BOUNDARY_FREEZE.md`
Read when you need:
- what Halo2 circuits actually prove today (code-confirmed),
- what is NOT yet proven (explicit list),
- three-layer proof model (circuit / structural / witness consistency).

### `PRIVAI_CONTRACT_VERIFICATION_AND_SETTLEMENT_DIRECTION.md`
Read when you need:
- contract-first execution model,
- 4-level verification (mechanical → contractual → semantic → settlement),
- delivery vs quality distinction, skill pack structure.

### `PRIVAI_OPERATOR_AND_DISPUTE_QUORUM_DIRECTION.md`
Read when you need:
- operator = rule executor (not moderator),
- signer quorum vs future dispute panel,
- recovery as peer path after timeout.

### `PRIVAI_TOR_GATED_NETWORK_DIRECTION.md`
Read when you need:
- TOR-GATED sandbox network design parameters,
- frozen invariants (F1-F8) vs open design questions,
- multi-hop relay topology direction.

### `PRIVAI_OPERATOR_CHEATSHEET.md`
Read when you need:
- 4-step operator workflow,
- 6 red flags to react to,
- model routing table.

## Transport / Runtime Specs

### `PRIVAI_TRANSPORT_RUNTIME_FREEZE_MEMO.md`
Read when you need:
- current transport split,
- runtime blocker map,
- v1 recommendation.

### `PRIVAI_TRANSPORT_AND_P2P_SPLIT.md`
Read when you need:
- explicit separation between validator P2P and NXMS mailbox path.

### `PRIVAI_VALIDATOR_SESSION_INVARIANTS.md`
Read when you need:
- validator session guarantees,
- what regression tests are supposed to defend.

## Coordination / History Docs

### `PRIVAI_CHAT_ARCHIVE_2026-04-09.md`
Read when you need:
- the big-picture history of this work,
- why key decisions were made,
- what got fixed in which order.

### `PRIVAI_XIAOMI_HANDOFF_I_SZABLON_2026-04-09.md`
Read when you need:
- the Xiaomi runtime handoff,
- compact task template style,
- how the project has been prompting external agents.

## Code Landing Zones

If you want Stage A runtime:
- `/home/nxms-server/privAI/privai-node/src/node.rs`
- `/home/nxms-server/privAI/privai-node/src/escrow_stage.rs`

If you want wallet Stage B:
- `/home/nxms-server/privAI/privai-wallet/src/escrow_builder.rs`
- `/home/nxms-server/privAI/privai-wallet/src/proof_handoff.rs`

If you want proof model:
- `/home/nxms-server/privAI/privai-proof/src/transfer.rs`
- `/home/nxms-server/privAI/privai-proof/src/artifact.rs`

If you want validator transport:
- `/home/nxms-server/privAI/privai-node/src/session_impl.rs`
- `/home/nxms-server/privAI/privai-node/src/session_transport.rs`
- `/home/nxms-server/privAI/privai-node/tests/validator_session.rs`
