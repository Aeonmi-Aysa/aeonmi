//! OpenRouter AI provider for Mother AI.
//!
//! Reads `OPENROUTER_API_KEY` from the environment.
//! Default model: `nvidia/nemotron-super-49b-v1:free` (overridable via `AEONMI_OPENROUTER_MODEL`).
//! Passes multi-turn conversation history when available.

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value as JsonValue;

use super::AiProvider;

const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "nvidia/nemotron-super-49b-v1:free";
const ENV_KEY: &str = "OPENROUTER_API_KEY";
const ENV_MODEL: &str = "AEONMI_OPENROUTER_MODEL";

pub struct OpenRouter {
    api_key: String,
    model: String,
}

impl OpenRouter {
    /// Construct from environment. Returns `None` if `OPENROUTER_API_KEY` is not set.
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var(ENV_KEY).ok().filter(|k| !k.is_empty())?;
        let model = std::env::var(ENV_MODEL)
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        Some(Self { api_key, model })
    }
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

impl AiProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "OpenRouter"
    }

    fn chat(&self, prompt: &str) -> Result<String> {
        self.chat_history(&[("user", prompt)])
    }

    fn chat_history(&self, messages: &[(&str, &str)]) -> Result<String> {
        let msgs: Vec<Message> = messages
            .iter()
            .map(|(role, content)| Message { role, content })
            .collect();

        let req = Request {
            model: &self.model,
            messages: msgs,
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://aeonmi.ai")
            .header("X-Title", "Aeonmi Mother AI")
            .json(&req)
            .send()
            .map_err(|e| anyhow!("OpenRouter request failed: {}", e))?;

        let status = response.status();
        let body: JsonValue = response
            .json()
            .map_err(|e| anyhow!("OpenRouter response parse failed: {}", e))?;

        if !status.is_success() {
            let msg = body["error"]["message"]
                .as_str()
                .unwrap_or("unknown error");
            return Err(anyhow!("OpenRouter API error {}: {}", status, msg));
        }

        let text = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("OpenRouter response missing content"))?
            .to_string();

        Ok(text)
    }
}
