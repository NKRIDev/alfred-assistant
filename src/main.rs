pub mod cli;
pub mod commands;
mod services;
mod core;
use dotenvy::dotenv;
use crate::commands::registry::init_commands;
use crate::core::alfred::Alfred;
use crate::services::application::ApplicationService;
use crate::services::stt::SttService;

fn test_transcription() {
    let stt = SttService::new("stt-models/ggml-small.bin").expect("Model not found");
    let samples = SttService::load_wav_as_f32("test_audio.wav");
    let text = stt.transcribe(&samples).expect("error");
    println!("Transcription : {}", text);
}

#[tokio::main]
async fn main() {
    /*
    Testing audio transcription using the Whisper model
    (I made sure to read and transcribe a .wav file to start with)
     */
    test_transcription();

    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    //init register command
    let registry = init_commands(app_service);

    //init alfred core
    let alfred = Alfred::new("qwen3:4b-instruct".to_string(), "skills/alfred.md".to_string(),
                             "datas/events.db".to_string(), registry);

    //Start alfred loop
    cli::start_alfred(alfred).await;
}