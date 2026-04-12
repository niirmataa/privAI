use crate::{builders::status_snapshot::current_status, models::StatusResponse};

pub fn handle() -> StatusResponse {
    current_status()
}
