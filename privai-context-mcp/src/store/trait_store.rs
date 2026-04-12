use crate::{models::KnowledgeItem, models::KnowledgeLayer, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalRequest {
    pub query: String,
    pub allowed_layers: Vec<KnowledgeLayer>,
    pub limit: usize,
}

pub trait Retriever {
    fn retrieve(&self, request: RetrievalRequest) -> Result<Vec<KnowledgeItem>>;
    fn all_items(&self) -> Result<Vec<KnowledgeItem>>;
}
