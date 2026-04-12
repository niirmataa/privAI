use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeLayer {
    V0Master,
    V0Direction,
    V0Control,
    V0FutureSpec,
    RepoVerified,
    Unknown,
}

impl KnowledgeLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V0Master => "v0_master",
            Self::V0Direction => "v0_direction",
            Self::V0Control => "v0_control",
            Self::V0FutureSpec => "v0_future_spec",
            Self::RepoVerified => "repo_verified",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_v0_runtime_layer(self) -> bool {
        matches!(
            self,
            Self::V0Master | Self::V0Direction | Self::V0Control | Self::V0FutureSpec
        )
    }
}
