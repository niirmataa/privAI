use crate::{
    models::{KnowledgeLayer, LookupRequest, LookupResponse},
    store::{RetrievalRequest, Retriever},
    Result,
};

pub fn handle(store: &impl Retriever, request: LookupRequest) -> Result<LookupResponse> {
    let matches = store.retrieve(RetrievalRequest {
        query: request.query,
        allowed_layers: vec![KnowledgeLayer::V0Control],
        limit: 8,
    })?;
    Ok(LookupResponse {
        matches,
        overclaim_warnings: vec![
            "Control docs guide sequencing; they are not protocol specs.".into(),
        ],
    })
}
