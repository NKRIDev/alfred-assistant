use std::sync::Arc;
use tokio::sync::Mutex;
use alfred_core::core::alfred::Alfred;
use alfred_core::core::orchestrator::Orchestrator;
use alfred_core::services::tts::TtsService;
use crate::dtos::ask_dtos::AlfredAskResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use alfred_core::services::micro::MicroService;
use alfred_core::services::stt::SttService;
/*
Business service: holds Alfred (shared, thread-safe)
and exposes the logic, modifying the HTTP protocol
 */
#[derive(Clone)]
pub struct AskService {
    alfred: Arc<Mutex<Alfred>>,
    tts: TtsService,
    stt: SttService,
}

impl AskService {
    pub fn new(alfred: Arc<Mutex<Alfred>>, tts: TtsService, stt: SttService) -> Self {
        Self { alfred, tts, stt }
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

    /*
    Ask alfred with audio response
     */
    pub async fn ask_from_audio(&self, audio_base64: String) -> Result<AlfredAskResponse, String> {
        //Base64 decoding of data sent by the JS
        let audio_bytes = STANDARD.decode(audio_base64)
            .map_err(|e| format!("Erreur décodage Base64 : {}", e))?;

        //Convert WAV bytes to 32-bit float samples
        let (samples, sample_rate) = MicroService::decode_webm_to_pcm(&audio_bytes)?;

        //Resample to 16kHz for Whisper via your microservice
        let samples_16k = MicroService::resample_to_16k(&samples, sample_rate);

        //Transcription via Whisper (SttService)
        let transcribed_text = self.stt.transcribe(&samples_16k)?;

        if transcribed_text.trim().is_empty() {
            return Err("Aucun texte n'a été détecté dans l'enregistrement.".into());
        }

        println!("[Micro Web] Message transcrit : {}", transcribed_text);
        self.ask(transcribed_text).await
    }
}