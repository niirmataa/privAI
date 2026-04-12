pub mod build_correction_pill;
pub mod get_current_status;
pub mod get_guardrails;
pub mod get_reading_order;
pub mod lookup_control;
pub mod lookup_direction;
pub mod prepare_task_context;
pub mod route_question;

pub const TOOL_NAMES: &[&str] = &[
    "privai_v0_get_reading_order",
    "privai_v0_get_current_status",
    "privai_v0_lookup_direction",
    "privai_v0_lookup_control",
    "privai_v0_get_guardrails",
    "privai_v0_route_question",
    "privai_v0_prepare_task_context",
    "privai_v0_build_correction_pill",
];
