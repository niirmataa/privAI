use std::path::Path;

use crate::models::KnowledgeLayer;

pub fn infer_layer(path: &Path) -> KnowledgeLayer {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name == "PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md" {
        KnowledgeLayer::V0Master
    } else if name.contains("TASK_LOG")
        || name.contains("PROMPT_LOG")
        || name.contains("DOCS_TREE")
        || name.contains("CONTEXT")
    {
        KnowledgeLayer::V0Control
    } else if name.contains("SPEC") || name.contains("SCHEMA") || name.contains("FREEZE") {
        KnowledgeLayer::V0FutureSpec
    } else {
        KnowledgeLayer::V0Direction
    }
}

pub fn status_for_layer(layer: KnowledgeLayer) -> &'static str {
    match layer {
        KnowledgeLayer::V0Master => "canonical",
        KnowledgeLayer::V0Direction => "direction",
        KnowledgeLayer::V0Control => "planning",
        KnowledgeLayer::V0FutureSpec => "future_spec",
        KnowledgeLayer::RepoVerified => "repo_verified",
        KnowledgeLayer::Unknown => "unknown",
    }
}
