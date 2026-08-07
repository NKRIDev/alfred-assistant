use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AlfredAskRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct AlfredAskResponse {
    pub result: String,
    pub audio: String,
}

#[derive(Deserialize)]
pub struct AudioAskRequest {
    pub audio_base64: String,
}