use tokio::io::{self, AsyncBufReadExt, BufReader};
use crate::core::alfred::Alfred;
use crate::core::dream::DreamTimer;
use crate::core::orchestrator::Orchestrator;
use crate::services::micro::MicroService;
use crate::services::stt::SttService;
/*
Returns the value the user enters in the console
 */
async fn input_command(reader: &mut BufReader<io::Stdin>) -> String {
    let mut input = String::new();
    reader.read_line(&mut input).await.expect("Failed to read input.");
    input.trim().to_string()
}

/*
Record micro and return text
 */
async fn record_micro(stt: &SttService) -> Option<String>{
    let (raw_audio, sample_rate) = MicroService::record();
    let audio_16k = MicroService::resample_to_16k(&raw_audio, sample_rate);

    match stt.transcribe(&audio_16k) {
        Ok(text) if !text.trim().is_empty() => Some(text),
        Ok(_) => {
            println!("[SYSTEM] Rien entendu, réessaie.");
            None
        }
        Err(e) => {
            eprintln!("[STT ERROR] {}", e);
            None
        }
    }
}

/*
Main loop
 */
pub async fn start_alfred(mut alfred: Alfred, stt: SttService) {
    println!("Welcome to Alfred Assistant");

    /*
    Tokio lib
     */
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    /*
    Alfred loop
     */
    loop {
        println!("> ");

        //Disable to test the microphone with the assistant
        //let input = input_command(&mut reader).await;
        let input = match record_micro(&stt).await{
            Some(text) => text,
            None => continue,
        };

        let reply = Orchestrator::ask_alfred(&input, &mut alfred).await;
        println!("{}", reply);

        //Check if input is "quit", break loop
        if input == "quit" {
            DreamTimer::trigger(alfred.database_url).await;
            break;
        }
    }
}