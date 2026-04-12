use crate::{config::Config, models::KnowledgeItem, store::RetrievalRequest, Result};

#[derive(Debug, Clone)]
pub struct VertexRagClient {
    pub project_id: String,
    pub location: String,
    pub corpus: String,
}

impl VertexRagClient {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            project_id: config.vertex_project_id.clone().ok_or_else(|| {
                crate::errors::McpError::InvalidConfig("VERTEX_PROJECT_ID missing".into())
            })?,
            location: config.vertex_location.clone().ok_or_else(|| {
                crate::errors::McpError::InvalidConfig("VERTEX_LOCATION missing".into())
            })?,
            corpus: config.vertex_rag_corpus.clone().ok_or_else(|| {
                crate::errors::McpError::InvalidConfig("VERTEX_RAG_CORPUS missing".into())
            })?,
        })
    }

    pub fn retrieve(&self, _request: RetrievalRequest) -> Result<Vec<KnowledgeItem>> {
        Err(crate::errors::McpError::Unsupported(
            "wire Vertex AI RAG RetrieveContexts API here".into(),
        ))
    }
}
