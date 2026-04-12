use crate::{
    models::{KnowledgeLayer, LookupRequest, LookupResponse},
    store::{RetrievalRequest, Retriever},
    Result,
};

pub fn handle(store: &impl Retriever, request: LookupRequest) -> Result<LookupResponse> {
    let query = request.topic.unwrap_or(request.query);
    let matches = store.retrieve(RetrievalRequest {
        query,
        allowed_layers: vec![KnowledgeLayer::V0Master, KnowledgeLayer::V0Direction],
        limit: 8,
    })?;
    Ok(LookupResponse {
        matches,
        overclaim_warnings: vec![
            "Direction docs do not prove current implementation facts.".into(),
            "Exact schemas and wire formats require future protocol specs.".into(),
        ],
    })
}
