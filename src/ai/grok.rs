//! xAI Grok AI provider for Mother AI.
//!
//! Grok uses an OpenAI-compatible API.
//! Base URL: https://api.x.ai/v1
//!
//! Required env var:
//!   GROK_API_KEY   — your xAI API key (xai-...)
//!
//! Optional:
//!   AEONMI_GROK_MODEL  — defaults to grok-beta

use anyhow::{Result, anyhow, bail};
use super::AiProvider;
use std::time::Duration;

pub struct Grok;

impl Default for Grok {
    fn default() -> Self { Self }
}

#[derive(serde::Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(serde::Serialize)]
struct ChatMessage { role: String, content: String }

#[derive(serde::Deserialize, Debug)]
struct ChatResponse { choices: Vec<Choice> }

#[derive(serde::Deserialize, Debug)]
struct Choice { message: ChoiceMessage }

#[derive(serde::Deserialize, Debug)]
struct ChoiceMessage { content: String }

impl AiProvider for Grok {
    fn name(&self) -> &'static str { "grok" }

    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }

        let key = std::env::var("GROK_API_KEY")
            .ok()
            .or_else(|| crate::core::api_keys::get_api_key("grok"))
            .or_else(|| crate::core::api_keys::get_api_key("xai"))
            .ok_or_else(|| anyhow!("GROK_API_KEY not set — run: set GROK_API_KEY=xai-..."))?;

        let model = std::env::var("AEONMI_GROK_MODEL")
            .unwrap_or_else(|_| "grok-beta".to_string());

        let req = ChatRequest {
            model,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: crate::ai::openrouter::MOTHER_SYSTEM_PROMPT.to_string(),
                },
                ChatMessage { role: "user".to_string(), content: trimmed.to_string() },
            ],
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let resp = client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Grok API error {}: {}", status, body);
        }

        let cr: ChatResponse = resp.json()
            .map_err(|e| anyhow!("Failed to parse Grok response: {}", e))?;

        let text = cr.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("Grok returned empty response"))?;

        Ok(text)
    }
}
