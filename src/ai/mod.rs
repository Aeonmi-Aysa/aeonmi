//! AI Mother Module: multi-provider abstraction.
//!
//! All providers are always compiled — no feature flags required.
//! Provider selection priority (when no AEONMI_AI_PROVIDER env var is set):
//!   1. First provider whose API key is found (env var OR encrypted api_keys store)
//!   2. First registered provider
//!
//! Provider registration order (also priority order for auto-detect):
//!   1. openrouter  — OPENROUTER_API_KEY  — free tier available
//!   2. claude      — ANTHROPIC_API_KEY   — Warren's Claude key
//!   3. openai      — OPENAI_API_KEY
//!   4. deepseek    — DEEPSEEK_API_KEY
//!   5. grok        — GROK_API_KEY        — xAI
//!   6. perplexity  — PERPLEXITY_API_KEY
//!
//! Mother's identity and personality are preserved regardless of which provider
//! is used — all providers receive the same MOTHER_SYSTEM_PROMPT.
//!
//! To force a specific provider:
//!   set AEONMI_AI_PROVIDER=claude   (or openrouter, openai, deepseek, grok, perplexity)

use anyhow::Result;

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chat(&self, prompt: &str) -> Result<String>;

    /// Chat with conversation history for session memory.
    /// Default implementation ignores history (single-turn).
    /// Override in providers that support multi-turn (e.g. Claude).
    fn chat_history(&self, prompt: &str, _history: &[claude::HistoryMessage]) -> Result<String> {
        self.chat(prompt)
    }

    fn chat_stream(&self, _prompt: &str, _cb: &mut dyn FnMut(&str)) -> Result<()> {
        let full = self.chat(_prompt)?;
        _cb(&full);
        Ok(())
    }
}

// All providers always compiled
pub mod openrouter;
pub mod claude;
pub mod openai;
pub mod deepseek;
pub mod grok;
pub mod perplexity;

#[cfg(feature = "ai-copilot")]
pub mod copilot;

/// Check whether a key is available — env var OR encrypted api_keys store.
fn has_key(env_var: &str, store_key: &str) -> bool {
    std::env::var(env_var).is_ok()
        || crate::core::api_keys::get_api_key(store_key).is_some()
}

pub struct AiRegistry {
    providers: Vec<Box<dyn AiProvider>>,
}

impl AiRegistry {
    pub fn new() -> Self {
        let mut r = Self { providers: Vec::new() };

        // Registration order = priority order
        r.providers.push(Box::new(openrouter::OpenRouter::default()));
        r.providers.push(Box::new(claude::Claude::default()));
        r.providers.push(Box::new(openai::OpenAi::default()));
        r.providers.push(Box::new(deepseek::DeepSeek::default()));
        r.providers.push(Box::new(grok::Grok::default()));
        r.providers.push(Box::new(perplexity::Perplexity::default()));

        #[cfg(feature = "ai-copilot")]
        { r.providers.push(Box::new(copilot::Copilot::default())); }

        r
    }

    pub fn list(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.iter().find(|p| p.name() == name).map(|b| b.as_ref())
    }

    /// Returns the best available provider:
    /// 1. Provider named by AEONMI_AI_PROVIDER env var (if set and found)
    /// 2. First provider whose API key is set (env var or encrypted store)
    /// 3. First registered provider (openrouter)
    pub fn preferred(&self) -> Option<&dyn AiProvider> {
        // Explicit override
        if let Ok(name) = std::env::var("AEONMI_AI_PROVIDER") {
            if let Some(p) = self.get(&name) {
                return Some(p);
            }
        }

        // Auto-detect: first provider with a key available.
        // Claude (Anthropic) is Mother's primary brain — checked first.
        // OpenRouter is a fallback for free-tier / alternative models only.
        let checks: &[(&str, &str, &str)] = &[
            ("claude",     "ANTHROPIC_API_KEY",  "claude"),
            ("openai",     "OPENAI_API_KEY",     "openai"),
            ("deepseek",   "DEEPSEEK_API_KEY",   "deepseek"),
            ("grok",       "GROK_API_KEY",        "grok"),
            ("perplexity", "PERPLEXITY_API_KEY", "perplexity"),
            ("openrouter", "OPENROUTER_API_KEY", "openrouter"),
        ];
        for (provider_name, env_var, store_key) in checks {
            if has_key(env_var, store_key) {
                if let Some(p) = self.get(provider_name) {
                    return Some(p);
                }
            }
        }

        // Fallback: first registered
        self.providers.first().map(|b| b.as_ref())
    }

    /// True if any provider has an API key available (env var or encrypted store).
    pub fn any_key_available(&self) -> bool {
        [
            ("OPENROUTER_API_KEY", "openrouter"),
            ("ANTHROPIC_API_KEY",  "claude"),
            ("OPENAI_API_KEY",     "openai"),
            ("DEEPSEEK_API_KEY",   "deepseek"),
            ("GROK_API_KEY",       "grok"),
            ("PERPLEXITY_API_KEY", "perplexity"),
        ]
        .iter()
        .any(|(env, store)| has_key(env, store))
    }
}

impl Default for AiRegistry {
    fn default() -> Self { Self::new() }
}
