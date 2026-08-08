use std::env;
use std::time::Duration;
use serde_json::{json, Value};

pub struct MistralService;

impl MistralService {
    pub async fn chat(messages: &Value, tools: &Value) -> Result<Value, String> {
        // 1. Récupération de la clé d'API Mistral
        let api_key = env::var("MISTRAL_API_KEY")
            .map_err(|_| "MISTRAL_API_KEY est manquante dans les variables d'environnement.".to_string())?;

        // 2. Construction du payload selon les spécifications de Mistral AI
        let mut body = json!({
            "model": "mistral-large-latest", // Tu peux aussi utiliser "mistral-large-latest"
            "messages": messages,
            "temperature": 0.4
        });

        // Ajouter les tools uniquement si le tableau n'est pas vide
        if let Some(tools_arr) = tools.as_array() {
            if !tools_arr.is_empty() {
                body["tools"] = tools.clone();
                body["tool_choice"] = json!("auto");
            }
        }

        // 3. Configuration du client HTTP
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| format!("Impossible de créer le client HTTP : {}", e))?;

        // 4. Envoi de la requête HTTP POST avec l'en-tête Bearer Token
        let response = client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Erreur lors de la requête Mistral : {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("L'API Mistral a renvoyé une erreur : {}", error_text));
        }

        // 5. Lecture de la réponse JSON
        let raw_res: Value = response
            .json()
            .await
            .map_err(|e| format!("Erreur de parsing JSON Mistral : {}", e))?;

        // 6. Normalisation de la réponse pour conserver le format attendu par ton Orchestrator
        // L'API Mistral renvoie la réponse sous : choices[0].message
        if let Some(choice_message) = raw_res["choices"][0]["message"].as_object() {
            Ok(json!({ "message": choice_message }))
        } else {
            Err("Format de réponse Mistral invalide.".to_string())
        }
    }
}