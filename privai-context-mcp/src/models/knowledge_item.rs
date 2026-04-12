use serde::{Deserialize, Serialize};

use super::KnowledgeLayer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub layer: KnowledgeLayer,
    pub title: String,
    pub topic: String,
    pub summary: String,
    pub content: String,
    pub tags: Vec<String>,
    pub status: String,
    pub recommended_for: Vec<String>,
    pub do_not_overclaim: Vec<String>,
    pub source_path: String,
    pub source_scope: String,
    pub legacy_allowed: bool,
}

impl KnowledgeItem {
    pub fn is_v0_only(&self) -> bool {
        self.source_scope == "v0_only" && !self.legacy_allowed && self.layer.is_v0_runtime_layer()
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.to_ascii_lowercase();
        self.title.to_ascii_lowercase().contains(&query)
            || self.topic.to_ascii_lowercase().contains(&query)
            || self.summary.to_ascii_lowercase().contains(&query)
            || self.content.to_ascii_lowercase().contains(&query)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(&query))
    }
}
