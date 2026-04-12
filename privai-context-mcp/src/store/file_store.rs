use std::{fs, path::PathBuf};

use crate::{
    models::{KnowledgeItem, KnowledgeLayer},
    routing::guardrails::validate_v0_item,
    store::{RetrievalRequest, Retriever},
    Result,
};

#[derive(Debug, Clone)]
pub struct LocalManifestStore {
    manifest_paths: Vec<PathBuf>,
}

impl LocalManifestStore {
    pub fn new(manifest_paths: Vec<PathBuf>) -> Self {
        Self { manifest_paths }
    }

    pub fn default_paths(data_dir: impl Into<PathBuf>) -> Vec<PathBuf> {
        let data_dir = data_dir.into();
        [
            "v0_master.json",
            "v0_direction.json",
            "v0_control.json",
            "v0_future_spec.json",
        ]
        .into_iter()
        .map(|name| data_dir.join(name))
        .collect()
    }

    fn load_path(path: &PathBuf) -> Result<Vec<KnowledgeItem>> {
        let raw = fs::read_to_string(path)?;
        let items: Vec<KnowledgeItem> = serde_json::from_str(&raw)?;
        Ok(items)
    }
}

impl Retriever for LocalManifestStore {
    fn retrieve(&self, request: RetrievalRequest) -> Result<Vec<KnowledgeItem>> {
        let mut results = Vec::new();
        for item in self.all_items()? {
            if !request.allowed_layers.contains(&item.layer) {
                continue;
            }
            if !item.matches_query(&request.query) {
                continue;
            }
            validate_v0_item(&item)?;
            results.push(item);
            if results.len() >= request.limit {
                break;
            }
        }
        Ok(results)
    }

    fn all_items(&self) -> Result<Vec<KnowledgeItem>> {
        let mut items = Vec::new();
        for path in &self.manifest_paths {
            if !path.exists() {
                continue;
            }
            for item in Self::load_path(path)? {
                validate_v0_item(&item)?;
                items.push(item);
            }
        }
        items.sort_by_key(|item| match item.layer {
            KnowledgeLayer::V0Master => 0,
            KnowledgeLayer::V0Direction => 1,
            KnowledgeLayer::V0Control => 2,
            KnowledgeLayer::V0FutureSpec => 3,
            KnowledgeLayer::RepoVerified => 4,
            KnowledgeLayer::Unknown => 5,
        });
        Ok(items)
    }
}
