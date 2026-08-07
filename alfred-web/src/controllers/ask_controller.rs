use axum::{Json, extract::State};
use axum::http::StatusCode;
use crate::dtos::ask_dtos::{AlfredAskRequest, AlfredAskResponse};
use crate::services::ask_service::AskService;

/*
POST /ask, answer alfred
 */
pub async fn ask_handle(State(service): State<AskService>, Json(payload): Json<AlfredAskRequest>)
    -> Result<Json<AlfredAskResponse>, (StatusCode, String)>{
    
    match service.ask(payload.text).await {
        Ok(response) => Ok(Json(response)),
        Err(err_msg) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur lors du traitement : {}", err_msg),
        )),
    }
}