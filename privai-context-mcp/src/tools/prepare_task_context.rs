use crate::{
    builders::task_context::prepare_task_context,
    models::{PrepareTaskContextRequest, PrepareTaskContextResponse},
};

pub fn handle(request: PrepareTaskContextRequest) -> PrepareTaskContextResponse {
    PrepareTaskContextResponse {
        pack: prepare_task_context(request),
    }
}
