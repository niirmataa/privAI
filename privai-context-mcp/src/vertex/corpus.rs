use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VertexCorpusConfig {
    pub project_id: String,
    pub location: String,
    pub corpus: String,
}
