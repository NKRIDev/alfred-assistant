use std::env;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;
use std::io::Cursor;

use rodio::stream::DeviceSinkBuilder;
use rodio::{Decoder, Player};

#[derive(Serialize)]
struct TtsRequest {
    text: String,
    language: String,
}

pub struct TtsService {
    client: Client,
    url: String,
}

impl TtsService {

    /*
    Create service by initializing the client and the URL
     */
    pub fn new() -> Self {
        let client_url = env::var("TTS_SERVER");

        Self {
            client: Client::new(),
            url: client_url.ok().expect("TTS client not found"),
        }
    }

    /*
    Play the audio track created by the TTS server.
     */
    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let payload = TtsRequest {
            text: text.to_string(),
            language: "fr".to_string(),
        };

        let response = self.client
            .post(&self.url)
            .json(&payload)
            .send()
            .await?;

        let audio_bytes = response.bytes().await?;

        if audio_bytes.len() < 1000 {
            let error_msg = String::from_utf8_lossy(&audio_bytes);
            eprintln!("[TTS Server Error] Server response : {}", error_msg);
            return Ok(());
        }

        /*
        We return a Result<Result<(), String>, JoinError>
         */
        let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
            let mut device_sink = DeviceSinkBuilder::open_default_sink()
                .map_err(|e| format!("Audio device error : {e}"))?;

            device_sink.log_on_drop(false);

            let cursor = Cursor::new(audio_bytes);
            let source = Decoder::new(cursor)
                .map_err(|e| format!("WAV decoding error : {e}"))?;

            let player = Player::connect_new(&device_sink.mixer());
            player.append(source);
            player.sleep_until_end();

            Ok(())
        }).await;

        /*
        Manage errors
         */
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err_msg)) => Err(err_msg.into()),
            Err(join_err) => Err(Box::new(join_err)),
        }
    }
}