use serde::{Deserialize, Serialize};

use super::{CorrectionPill, KnowledgeItem, KnowledgeLayer, TaskContextPack};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadingMode {
    NewAgent,
    Reentry,
    DeepSpec,
    TaskExecution,
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingOrderRequest {
    pub mode: ReadingMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadingOrderResponse {
    pub ordered_docs: Vec<String>,
    pub why_these_docs: Vec<String>,
    pub what_not_to_read: Vec<String>,
    pub what_not_to_infer: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub current_phase: String,
    pub completed_v0_docs: Vec<String>,
    pub planned_v0_docs: Vec<String>,
    pub highest_priority_gap: String,
    pub unsafe_claims: Vec<String>,
    pub next_recommended_task: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupRequest {
    pub query: String,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookupResponse {
    pub matches: Vec<KnowledgeItem>,
    pub overclaim_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailsRequest {
    pub topic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailsResponse {
    pub applicable_rules: Vec<String>,
    pub forbidden_inferences: Vec<String>,
    pub stop_conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteQuestionRequest {
    pub question: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteQuestionResponse {
    pub layer: KnowledgeLayer,
    pub needs_code_audit: bool,
    pub needs_protocol_spec: bool,
    pub must_not_use_legacy: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareTaskContextRequest {
    pub task_title: String,
    pub task_goal: String,
    pub target_model: Option<String>,
    pub write_scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareTaskContextResponse {
    pub pack: TaskContextPack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildCorrectionPillRequest {
    pub finding: String,
    pub affected_file: String,
    pub severity: String,
    pub correction_direction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildCorrectionPillResponse {
    pub pill: CorrectionPill,
}
