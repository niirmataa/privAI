# privAI Docs Index

Use this as a map, not as a narrative.

Recommended read order:
1. `PRIVAI_SYSTEM_PRODUCT_FOUNDATION.md`
2. `PRIVAI_PROJECT_ENTRYPOINT.md`
3. `PRIVAI_V1_READINESS_AND_GAPS.md`
4. `PRIVAI_V1_PRODUCTION_PATH.md`
5. `PRIVAI_NEXT_DIRECTION.md`
6. `CHAT_ARCHIVE_2026-04-09.md`
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

### `CHAT_ARCHIVE_2026-04-09.md`
Read when you need:
- the big-picture history of this work,
- why key decisions were made,
- what got fixed in which order.

### `XIAOMI_HANDOFF_I_SZABLON_2026-04-09.md`
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
