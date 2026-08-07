use std::sync::Arc;
use tokio::sync::Mutex;
use alfred_core::core::alfred::Alfred;
use alfred_core::core::orchestrator::Orchestrator;
use alfred_core::services::tts::TtsService;
use crate::dtos::ask_dtos::AlfredAskResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/*
Business service: holds Alfred (shared, thread-safe)
and exposes the logic, modifying the HTTP protocol
 */
#[derive(Clone)]
pub struct AskService {
    alfred: Arc<Mutex<Alfred>>,
    tts: TtsService,
}

impl AskService {
    pub fn new(alfred: Arc<Mutex<Alfred>>, tts: TtsService) -> Self {
        Self { alfred, tts }
    }

    /*
    Returns Alfred's response
     */
    pub async fn ask(&self, text: String) -> Result<AlfredAskResponse, String> {
        //Ask alfred and return response
        let mut alfred = self.alfred.lock().await;
        let response = Orchestrator::ask_alfred(&text, &mut alfred).await;

        //Generate audio
        let audio_bytes = self.tts.generate_audio(&response).await
            .map_err(|e| format!("Error TTS : {}", e))?;
        let audio_base64 = STANDARD.encode(audio_bytes);

        //Create JSON response
        let response_json = AlfredAskResponse {
            result: response,
            audio: audio_base64
        };

        Ok(response_json)
    }
}