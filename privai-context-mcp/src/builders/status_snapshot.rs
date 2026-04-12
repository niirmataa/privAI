use crate::models::StatusResponse;

pub fn current_status() -> StatusResponse {
    StatusResponse {
        current_phase: "V0 direction freeze / context tooling preparation".into(),
        completed_v0_docs: vec![
            "PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md".into(),
            "PRIVAI_V0_DIAGRAMS.md".into(),
            "PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md".into(),
            "PRIVAI_V0_DOCS_TREE.md".into(),
            "PRIVAI_V0_CONTEXT_MCP_SERVER_DIRECTION.md".into(),
        ],
        planned_v0_docs: vec![
            "PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md".into(),
            "PRIVAI_V0_METERING_PROTOCOL_DIRECTION.md".into(),
            "PRIVAI_V0_PRIVATE_DISCOVERY_DIRECTION.md".into(),
            "PRIVAI_V0_APVA_DENOMINATION_DIRECTION.md".into(),
        ],
        highest_priority_gap:
            "Operatorless escrow bridge and metering direction remain before protocol specs.".into(),
        unsafe_claims: vec![
            "operatorless escrow is implemented".into(),
            "pro-rata split is implemented".into(),
            "receipt schema is frozen".into(),
            "legacy marketplace docs are valid V0 source".into(),
        ],
        next_recommended_task:
            "Write or review PRIVAI_V0_OPERATORLESS_ESCROW_DIRECTION.md before protocol work."
                .into(),
    }
}
