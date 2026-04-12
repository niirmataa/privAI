use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskContextPack {
    pub task: String,
    pub why_now: String,
    pub what_is_already_there: Vec<String>,
    pub what_is_missing: Vec<String>,
    pub depends_on: Vec<String>,
    pub can_run_in_parallel: Vec<String>,
    pub do_not_touch: Vec<String>,
    pub minimal_reading_scope: Vec<String>,
    pub unchecked_assumptions: Vec<String>,
    pub definition_of_done: Vec<String>,
    pub exact_final_report_format: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectionPill {
    pub bounded_task: String,
    pub source_of_truth_docs: Vec<String>,
    pub forbidden_changes: Vec<String>,
    pub definition_of_done: Vec<String>,
    pub final_report_format: Vec<String>,
}
