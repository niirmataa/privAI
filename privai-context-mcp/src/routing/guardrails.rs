use crate::{config::DEFAULT_V0_ROOT, errors::McpError, models::KnowledgeItem, Result};

pub const SOURCE_SCOPE_V0_ONLY: &str = "v0_only";

pub fn validate_v0_item(item: &KnowledgeItem) -> Result<()> {
    if item.source_scope != SOURCE_SCOPE_V0_ONLY {
        return Err(McpError::GuardrailViolation(format!(
            "{} has source_scope={}",
            item.id, item.source_scope
        )));
    }
    if item.legacy_allowed {
        return Err(McpError::GuardrailViolation(format!(
            "{} allows legacy context",
            item.id
        )));
    }
    if !item.layer.is_v0_runtime_layer() {
        return Err(McpError::GuardrailViolation(format!(
            "{} has disallowed layer {}",
            item.id,
            item.layer.as_str()
        )));
    }
    if !item.source_path.starts_with(DEFAULT_V0_ROOT) {
        return Err(McpError::GuardrailViolation(format!(
            "{} source_path outside V0 root: {}",
            item.id, item.source_path
        )));
    }
    Ok(())
}

pub fn default_guardrails() -> Vec<String> {
    vec![
        "No public AI marketplace as baseline.".into(),
        "No public discovery as baseline.".into(),
        "No public provider profile as baseline.".into(),
        "No subjective AI quality settlement.".into(),
        "No operator as canonical escrow decision-maker.".into(),
        "No silent downgrade from FullPrivacy.".into(),
        "No legacy docs in V0 RAG/MCP retrieval.".into(),
        "No code facts inferred from V0 direction docs.".into(),
    ]
}

pub fn forbidden_inferences() -> Vec<String> {
    vec![
        "Do not infer operatorless escrow is implemented.".into(),
        "Do not infer pro-rata settlement is implemented.".into(),
        "Do not infer receipt schemas are frozen.".into(),
        "Do not infer current code reality from product direction docs.".into(),
        "Do not infer MarketplaceBatchTx defines the V0 product.".into(),
    ]
}
