use std::{env, path::PathBuf};

pub const DEFAULT_V0_ROOT: &str = "/home/nxms-server/privAI/spec/PRIVAI_V0_PRIVATE_COMPUTE";
pub const DEFAULT_DATA_DIR: &str = "data";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub v0_root: PathBuf,
    pub data_dir: PathBuf,
    pub vertex_project_id: Option<String>,
    pub vertex_location: Option<String>,
    pub vertex_rag_corpus: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            v0_root: env::var("PRIVAI_V0_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(DEFAULT_V0_ROOT)),
            data_dir: env::var("PRIVAI_CONTEXT_MCP_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(DEFAULT_DATA_DIR)),
            vertex_project_id: env::var("VERTEX_PROJECT_ID").ok(),
            vertex_location: env::var("VERTEX_LOCATION").ok(),
            vertex_rag_corpus: env::var("VERTEX_RAG_CORPUS").ok(),
        }
    }

    pub fn vertex_ready(&self) -> bool {
        self.vertex_project_id.is_some()
            && self.vertex_location.is_some()
            && self.vertex_rag_corpus.is_some()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            v0_root: PathBuf::from(DEFAULT_V0_ROOT),
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            vertex_project_id: None,
            vertex_location: None,
            vertex_rag_corpus: None,
        }
    }
}
