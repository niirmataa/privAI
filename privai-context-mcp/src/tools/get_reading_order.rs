use crate::{
    builders::reading_order::build_reading_order,
    models::{ReadingOrderRequest, ReadingOrderResponse},
};

pub fn handle(request: ReadingOrderRequest) -> ReadingOrderResponse {
    build_reading_order(request.mode)
}
