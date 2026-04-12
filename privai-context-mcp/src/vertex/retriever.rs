use crate::{models::KnowledgeLayer, store::RetrievalRequest};

pub fn bounded_request(
    query: impl Into<String>,
    allowed_layers: Vec<KnowledgeLayer>,
    limit: usize,
) -> RetrievalRequest {
    RetrievalRequest {
        query: query.into(),
        allowed_layers,
        limit,
    }
}
