//! Anthropic Claude AI provider for Mother AI.
//!
//! Reads `ANTHROPIC_API_KEY` from the environment.
//! Default model: `claude-sonnet-4-20250514` (overridable via `AEONMI_CLAUDE_MODEL`).
//! Passes multi-turn conversation history when available.

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value as JsonValue;

use super::AiProvider;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const ENV_KEY: &str = "ANTHROPIC_API_KEY";
const ENV_MODEL: &str = "AEONMI_CLAUDE_MODEL";

pub struct Claude {
    api_key: String,
    model: String,
}

impl Claude {
    /// Construct from environment. Returns `None` if `ANTHROPIC_API_KEY` is not set.
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
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

impl AiProvider for Claude {
    fn name(&self) -> &'static str {
        "Claude"
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
            max_tokens: 4096,
            messages: msgs,
        };

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .map_err(|e| anyhow!("Claude request failed: {}", e))?;

        let status = response.status();
        let body: JsonValue = response
            .json()
            .map_err(|e| anyhow!("Claude response parse failed: {}", e))?;

        if !status.is_success() {
            let msg = body["error"]["message"]
                .as_str()
                .unwrap_or("unknown error");
            return Err(anyhow!("Claude API error {}: {}", status, msg));
        }

        // Claude returns content as an array of content blocks
        let text = body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Claude response missing content text"))?
            .to_string();

        Ok(text)
    }
}
