use crate::{
    models::RouteQuestionRequest, models::RouteQuestionResponse, routing::question_router,
};

pub fn handle(request: RouteQuestionRequest) -> RouteQuestionResponse {
    question_router::route_question(&request.question)
}
