use dotenvy::dotenv;
use alfred_core::commands::registry::init_commands;
use alfred_core::core::alfred::Alfred;
use alfred_core::services::application::ApplicationService;
use alfred_core::services::micro::MicroService;
use alfred_core::services::stt::SttService;

mod cli;

fn test_transcription() {
    let stt = SttService::new("stt-models/ggml-small.bin").expect("Model not found");
    let samples = SttService::load_wav_as_f32("test_audio.wav");
    let text = stt.transcribe(&samples).expect("error");
    println!("Transcription : {}", text);
}

fn listen_micro(){
    let stt = SttService::new("stt-models/ggml-small.bin").expect("Model not found");
    let (raw_audio, sample_rate) = MicroService::record();
    let audio_16k = MicroService::resample_to_16k(&raw_audio, sample_rate);
    let text = stt.transcribe(&audio_16k).expect("error");
    println!("Transcription : {}", text);
}

#[tokio::main]
async fn main() {
    //Init .env
    dotenv().ok();

    //init application register
    let app_service = ApplicationService::new("apps.toml");

    //init register command
    let registry = init_commands(app_service);

    //STT service
    let stt_service = SttService::new("stt-models/ggml-small.bin").expect("Model not found");

    //init alfred core
    let alfred = Alfred::new("qwen3:4b-instruct".to_string(), "skills/alfred.md".to_string(),
                             "datas/events.db".to_string(), registry);

    //Start alfred loop
    cli::start_alfred(alfred, stt_service).await;
}