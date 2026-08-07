use axum::{extract::State, Json};
use axum::http::StatusCode;
use crate::dtos::ask_dtos::{AlfredAskRequest, AlfredAskResponse, AudioAskRequest};
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

/*
POST on ask-audio
 */
pub async fn ask_micro_handle(State(service): State<AskService>, Json(payload): Json<AudioAskRequest>, ) 
    -> Result<Json<AlfredAskResponse>, (StatusCode, String)> {
    
    if payload.audio_base64.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Aucune donnée audio envoyée.".to_string(),
        ));
    }

    match service.ask_from_audio(payload.audio_base64).await {
        Ok(response) => Ok(Json(response)),
        Err(err_msg) => {
            eprintln!("[Erreur /ask-audio] : {}", err_msg);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur lors du traitement audio : {}", err_msg),
            ))
        }
    }
}