use crate::models::{ReadingMode, ReadingOrderResponse};

pub fn build_reading_order(mode: ReadingMode) -> ReadingOrderResponse {
    let mut ordered_docs = vec![
        "PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md".into(),
        "PRIVAI_V0_DIAGRAMS.md".into(),
        "PRIVAI_V0_COMPUTE_LEASE_SETTLEMENT_DIRECTION.md".into(),
        "PRIVAI_V0_DOCS_TREE.md".into(),
    ];

    if matches!(mode, ReadingMode::TaskExecution | ReadingMode::Review) {
        ordered_docs.push("PRIVAI_V0_TASK_LOG.md".into());
        ordered_docs.push("PRIVAI_V0_PROMPT_LOG.md".into());
    }

    ReadingOrderResponse {
        ordered_docs,
        why_these_docs: vec![
            "Start from V0 master truth, then diagrams, settlement direction, and docs sequencing."
                .into(),
        ],
        what_not_to_read: vec![
            "Do not read legacy marketplace docs as default V0 context.".into(),
            "Do not use handoff docs as MCP runtime truth.".into(),
        ],
        what_not_to_infer: vec![
            "Do not infer operatorless escrow or pro-rata settlement is implemented.".into(),
            "Do not infer exact protocol schemas from direction docs.".into(),
        ],
    }
}
