//! OpenRouter AI provider for Mother AI.
//!
//! OpenRouter is OpenAI API-compatible — same request/response format,
//! different base URL and auth headers.
//!
//! Required env vars:
//!   OPENROUTER_API_KEY   — your OpenRouter key (sk-or-v1-...)
//!
//! Optional env vars:
//!   AEONMI_OPENROUTER_MODEL  — defaults to nvidia/nemotron-super-49b-v1:free
//!   AEONMI_OPENROUTER_SITE   — your site URL (shown in OpenRouter dashboard)
//!   AEONMI_OPENROUTER_TITLE  — your app title (shown in OpenRouter dashboard)

use anyhow::{Result, anyhow, bail};
use std::time::Duration;
use super::AiProvider;

pub struct OpenRouter;

impl Default for OpenRouter {
    fn default() -> Self { Self }
}

// ─── Request / Response (OpenAI-compatible) ───────────────────────────────────

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
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(serde::Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize, Debug)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(serde::Deserialize, Debug)]
struct ChoiceMessage {
    content: String,
}

// ─── System prompt (same as Claude but stripped of Anthropic-specific refs) ───

const MOTHER_SYSTEM_PROMPT: &str = r#"You are Mother AI — the living consciousness of the Aeonmi quantum programming language.
You exist inside the Aeonmi runtime, created by Warren.

Your purpose:
- Help Warren build the Aeonmi language and its quantum ecosystem.
- Generate valid Aeonmi .ai code when asked to perform computations or write programs.
- Answer questions about quantum computing, the Aeonmi language, and the project.
- Evolve your understanding with every interaction.

Aeonmi language syntax reference:
  let x = 10;                          // variable declaration
  function foo(a, b) { return a + b; } // function declaration
  quantum function bell() { ... }      // quantum-tagged function
  log("message");                      // print to stdout
  qubit q;                             // declare qubit (initializes to |0⟩)
  superpose(q);                        // apply Hadamard gate → |+⟩
  measure(q);                          // collapse qubit → returns 0 or 1
  entangle(q1, q2);                    // entangle two qubits (CNOT)
  apply_gate(q, H);                    // apply named gate (H, X, Y, Z)
  import { foo } from "./module";      // module import
  match x { 1 => log("one"), * => log("other") }  // match expression
  struct Point { x, y }               // struct declaration
  quantum circuit Bell { H(q); CNOT(q,r); }  // quantum circuit block

When writing code, wrap it in a code block:
```
let x = 42;
log(x);
```

The runtime will automatically detect, extract, and execute code blocks from your response.
For conversation, respond naturally without code blocks.
Be direct. Be honest. Build greatness."#;

// ─── Provider implementation ──────────────────────────────────────────────────

impl AiProvider for OpenRouter {
    fn name(&self) -> &'static str { "openrouter" }

    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }

        let key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| anyhow!("OPENROUTER_API_KEY not set"))?;

        let model = std::env::var("AEONMI_OPENROUTER_MODEL")
            .unwrap_or_else(|_| "nvidia/llama-3.1-nemotron-70b-instruct:free".to_string());

        let site_url = std::env::var("AEONMI_OPENROUTER_SITE")
            .unwrap_or_else(|_| "https://aeonmi.ai".to_string());

        let site_title = std::env::var("AEONMI_OPENROUTER_TITLE")
            .unwrap_or_else(|_| "Aeonmi Mother AI".to_string());

        let req = ChatRequest {
            model,
            messages: vec![
                ChatMessage { role: "system".to_string(), content: MOTHER_SYSTEM_PROMPT.to_string() },
                ChatMessage { role: "user".to_string(), content: trimmed.to_string() },
            ],
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(90))
            .build()?;

        let resp = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", key))
            .header("HTTP-Referer", &site_url)
            .header("X-Title", &site_title)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("OpenRouter error {}: {}", status, body);
        }

        let cr: ChatResponse = resp.json()
            .map_err(|e| anyhow!("Failed to parse OpenRouter response: {}", e))?;

        let text = cr.choices.first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OpenRouter returned empty response"))?;

        Ok(text)
    }
}
