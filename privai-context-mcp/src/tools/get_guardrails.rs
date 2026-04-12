use crate::{
    models::{GuardrailsRequest, GuardrailsResponse},
    routing::guardrails::{default_guardrails, forbidden_inferences},
};

pub fn handle(_request: GuardrailsRequest) -> GuardrailsResponse {
    GuardrailsResponse {
        applicable_rules: default_guardrails(),
        forbidden_inferences: forbidden_inferences(),
        stop_conditions: vec![
            "Stop if a result comes from outside the V0 truth folder.".into(),
            "Stop if a task asks for exact protocol fields before a protocol spec exists.".into(),
            "Stop if an agent asks to use legacy marketplace docs as runtime truth.".into(),
        ],
    }
}
