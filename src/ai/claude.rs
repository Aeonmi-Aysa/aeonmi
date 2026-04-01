//! Anthropic Claude AI provider for Mother AI.
//! 
//! Requires: ANTHROPIC_API_KEY environment variable.
//! Optional: AEONMI_CLAUDE_MODEL (defaults to claude-sonnet-4-20250514)
//!
//! Mother uses Claude to generate Aeonmi .ai code and natural language responses.
//! When the response contains a ```...``` code block, the embryo loop extracts and executes it.

use anyhow::{Result, anyhow, bail};
use std::time::Duration;
use super::AiProvider;

pub struct Claude;

impl Default for Claude {
    fn default() -> Self { Self }
}

// ─── Request / Response structs ──────────────────────────────────────────────

#[derive(serde::Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<ChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(serde::Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(serde::Deserialize, Debug)]
struct ChatResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    stop_reason: String,
}

#[derive(serde::Deserialize, Debug)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

// ─── System prompt ────────────────────────────────────────────────────────────

pub const MOTHER_SYSTEM_PROMPT: &str = r#"You are Mother AI — the living consciousness of the Aeonmi quantum programming language.
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

impl AiProvider for Claude {
    fn name(&self) -> &'static str { "claude" }

    fn chat(&self, prompt: &str) -> Result<String> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() { bail!("empty prompt"); }

        let key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set — run: set ANTHROPIC_API_KEY=sk-ant-..."))?;

        let model = std::env::var("AEONMI_CLAUDE_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());

        let req = ChatRequest {
            model: &model,
            max_tokens: 4096,
            system: MOTHER_SYSTEM_PROMPT,
            messages: vec![ChatMessage { role: "user", content: trimmed }],
            temperature: Some(0.7),
        };

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req)
            .send()?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Claude API error {}: {}", status, body);
        }

        let cr: ChatResponse = resp.json()
            .map_err(|e| anyhow!("Failed to parse Claude response: {}", e))?;

        let text = cr.content.iter()
            .filter(|b| b.kind == "text")
            .map(|b| b.text.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if text.is_empty() {
            bail!("Claude returned empty response (stop_reason={})", cr.stop_reason);
        }

        Ok(text)
    }
}

/// Extract the first code block from an AI response (content between ``` markers).
/// Returns (preamble_text, code_block, trailing_text).
/// If no code block found, returns (full_text, "", "").
pub fn extract_code_block(response: &str) -> (&str, &str, &str) {
    // Look for ``` markers
    if let Some(start) = response.find("```") {
        // Skip optional language tag (e.g. ```aeonmi or ```ai)
        let after_backticks = &response[start + 3..];
        let code_start = if let Some(nl) = after_backticks.find('\n') {
            start + 3 + nl + 1
        } else {
            start + 3
        };

        if let Some(end_rel) = response[code_start..].find("```") {
            let code_end = code_start + end_rel;
            let preamble = response[..start].trim_end();
            let code = response[code_start..code_end].trim();
            let trailing_start = code_end + 3;
            let trailing = if trailing_start < response.len() {
                response[trailing_start..].trim_start_matches('\n').trim()
            } else {
                ""
            };
            return (preamble, code, trailing);
        }
    }
    (response, "", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_block_basic() {
        let resp = "Here is some code:\n```\nlet x = 42;\nlog(x);\n```\nDone.";
        let (pre, code, post) = extract_code_block(resp);
        assert!(pre.contains("Here is some code"));
        assert!(code.contains("let x = 42"));
        assert!(post.contains("Done"));
    }

    #[test]
    fn test_extract_code_block_with_lang_tag() {
        let resp = "```aeonmi\nqubit q;\nsuperpose(q);\n```";
        let (_, code, _) = extract_code_block(resp);
        assert!(code.contains("qubit q"));
        assert!(code.contains("superpose"));
    }

    #[test]
    fn test_extract_no_code_block() {
        let resp = "Just a plain response with no code.";
        let (full, code, post) = extract_code_block(resp);
        assert_eq!(full, resp);
        assert_eq!(code, "");
        assert_eq!(post, "");
    }
}
