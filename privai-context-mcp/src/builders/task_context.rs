use crate::models::{PrepareTaskContextRequest, TaskContextPack};

pub fn prepare_task_context(request: PrepareTaskContextRequest) -> TaskContextPack {
    TaskContextPack {
        task: request.task_title,
        why_now: request.task_goal,
        what_is_already_there: vec![
            "V0 master direction exists.".into(),
            "V0 settlement direction exists.".into(),
            "V0 docs tree defines sequencing.".into(),
        ],
        what_is_missing: vec!["Task-specific accepted direction/spec may still be missing.".into()],
        depends_on: vec!["PRIVAI_V0_DOCS_TREE.md".into()],
        can_run_in_parallel: vec!["Read-only review and prompt preparation.".into()],
        do_not_touch: vec![
            "Do not touch legacy docs.".into(),
            "Do not modify app/runtime code unless explicitly scoped.".into(),
            "Do not infer implementation facts from V0 direction docs.".into(),
        ],
        minimal_reading_scope: vec![
            "PRIVAI_V0_DIRECTION_RESET_PRIVATE_COMPUTE_NETWORK.md".into(),
            "PRIVAI_V0_DOCS_TREE.md".into(),
            "Task-specific V0 direction docs.".into(),
        ],
        unchecked_assumptions: vec!["Current code reality requires direct code/test audit.".into()],
        definition_of_done: vec![
            "Output stays within declared write scope.".into(),
            "No legacy docs are used as runtime source.".into(),
            "Final report lists unchecked assumptions.".into(),
        ],
        exact_final_report_format: vec![
            "WHAT WAS CREATED:".into(),
            "TRUTH SOURCE USED:".into(),
            "GUARDRAILS ENFORCED:".into(),
            "TEST RESULTS:".into(),
            "OPEN FOLLOW-UPS:".into(),
            "UNCHECKED ASSUMPTIONS:".into(),
        ],
    }
}
