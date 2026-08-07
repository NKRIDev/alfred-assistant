use axum::{Json, extract::State};
use crate::dtos::ask_dtos::{AlfredAskRequest, AlfredAskResponse};
use crate::services::ask_service::AskService;

/*
POST /ask, answer alfred
 */
pub async fn ask_handle(State(service): State<AskService>,
                        Json(payload): Json<AlfredAskRequest>) -> Json<AlfredAskResponse> {
    let result = service.ask(payload.text).await;
    Json(AlfredAskResponse { result })
}