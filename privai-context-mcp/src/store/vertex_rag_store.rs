use crate::{
    models::KnowledgeItem,
    routing::guardrails::validate_v0_item,
    store::{RetrievalRequest, Retriever},
    vertex::client::VertexRagClient,
    Result,
};

#[derive(Debug, Clone)]
pub struct VertexRagStore {
    client: VertexRagClient,
}

impl VertexRagStore {
    pub fn new(client: VertexRagClient) -> Self {
        Self { client }
    }
}

impl Retriever for VertexRagStore {
    fn retrieve(&self, request: RetrievalRequest) -> Result<Vec<KnowledgeItem>> {
        let mut items = self.client.retrieve(request)?;
        items.retain(|item| validate_v0_item(item).is_ok());
        Ok(items)
    }

    fn all_items(&self) -> Result<Vec<KnowledgeItem>> {
        Err(crate::errors::McpError::Unsupported(
            "Vertex RAG all_items is intentionally unavailable; query through bounded tools".into(),
        ))
    }
}
