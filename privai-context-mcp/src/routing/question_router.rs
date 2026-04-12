use crate::models::{KnowledgeLayer, RouteQuestionResponse};

pub fn route_question(question: &str) -> RouteQuestionResponse {
    let q = question.to_ascii_lowercase();
    let asks_code = ["implemented", "code", "rust", "test", "current mechanics"]
        .iter()
        .any(|needle| q.contains(needle));
    let asks_protocol = ["schema", "wire format", "exact field", "final formula"]
        .iter()
        .any(|needle| q.contains(needle));

    let layer = if asks_code {
        KnowledgeLayer::RepoVerified
    } else if asks_protocol {
        KnowledgeLayer::V0FutureSpec
    } else if q.contains("task") || q.contains("log") || q.contains("reading") {
        KnowledgeLayer::V0Control
    } else if q.contains("what is privai") || q.contains("marketplace") {
        KnowledgeLayer::V0Master
    } else {
        KnowledgeLayer::V0Direction
    };

    RouteQuestionResponse {
        layer,
        needs_code_audit: asks_code,
        needs_protocol_spec: asks_protocol,
        must_not_use_legacy: true,
        explanation: "Route through V0-only context; use code audit only for implementation facts."
            .into(),
    }
}
