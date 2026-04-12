use crate::models::{BuildCorrectionPillRequest, CorrectionPill};

pub fn build_correction_pill(request: BuildCorrectionPillRequest) -> CorrectionPill {
    CorrectionPill {
        bounded_task: format!(
            "Correct {} finding in {}",
            request.severity, request.affected_file
        ),
        source_of_truth_docs: vec![
            "PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md".into(),
            "PRIVAI_V0_DOCS_TREE.md".into(),
        ],
        forbidden_changes: vec![
            "Do not use legacy docs as source truth.".into(),
            "Do not broaden the task beyond the affected file/scope.".into(),
            "Do not add write/execution tools to the MCP server.".into(),
        ],
        definition_of_done: vec![
            request.finding,
            request
                .correction_direction
                .unwrap_or_else(|| "Apply the smallest V0-consistent correction.".into()),
        ],
        final_report_format: vec![
            "WHAT WAS CHANGED:".into(),
            "SOURCE USED:".into(),
            "TEST RESULTS:".into(),
            "UNCHECKED ASSUMPTIONS:".into(),
        ],
    }
}
