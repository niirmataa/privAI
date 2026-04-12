use crate::{
    builders::correction_pill::build_correction_pill,
    models::{BuildCorrectionPillRequest, BuildCorrectionPillResponse},
};

pub fn handle(request: BuildCorrectionPillRequest) -> BuildCorrectionPillResponse {
    BuildCorrectionPillResponse {
        pill: build_correction_pill(request),
    }
}
