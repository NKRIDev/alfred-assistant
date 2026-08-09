use std::collections::HashMap;
use serde_json::{json, Value};
use crate::commands::command::CommandHandler;
use tokio::runtime::Handle;
use crate::services::spotify::SpotifyService;

pub struct ManageMusicCommand {
    spotify: SpotifyService,
}

impl ManageMusicCommand {
    pub fn new(spotify: SpotifyService) -> Self {
        Self { spotify }
    }
}

impl CommandHandler for ManageMusicCommand {
    fn execute(&self, args: &HashMap<String, String>) -> String {
        let action = match args.get("action") {
            Some(v) => v.as_str(),
            None => return "Missing argument: action".into(),
        };

        let result = tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                match action {
                    "play" => self.spotify.play().await,
                    "pause" => self.spotify.pause().await,
                    "next" => self.spotify.next_track().await,
                    "search_and_play" => {
                        let query = args.get("query")
                            .ok_or_else(|| "Missing argument: query for search_and_play".to_string())?;
                        self.spotify.search_and_play(query).await
                    }
                    _ =>  {
                        eprint!("Unknown action '{}'", action);
                        Err(format!("Action inconnue : {}", action))
                    }
                }
            })
        });

        match &result {
            Err(e) => eprintln!("[SPOTIFY ERROR] {}", e),
            _ => {}
        }

        result.unwrap_or_else(|e| format!("Erreur : {}", e))
    }

    fn description(&self) -> Value {
        json!({
        "type": "function",
        "function": {
            "name": "manage_music",
            "description": "Control Spotify music playback: play a specific song or artist, pause, resume, \
            or skip tracks. Use this tool whenever the user asks to play, launch, start, or listen to a song, \
            artist, or music — even if they say 'lance' or 'ouvre' a song. Do NOT use the 'open' tool for playing music.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["play", "pause", "next", "search_and_play"],
                        "description": "Action to perform"
                    },
                    "query": {
                        "type": "string",
                        "description": "Song title and/or artist to search, required only for search_and_play"
                    }
                },
                "required": ["action"]
            }
        }
    })
    }
}