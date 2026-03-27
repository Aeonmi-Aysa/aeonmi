//! AI Mother Module: multi-provider abstraction.
//!
//! Provider priority (first registered = preferred when no AEONMI_AI_PROVIDER set):
//!   1. openrouter  — OpenRouter (OPENROUTER_API_KEY)   — always compiled
//!   2. claude      — Anthropic  (ANTHROPIC_API_KEY)    — always compiled
//!   3. openai      — OpenAI     (OPENAI_API_KEY)        — feature: ai-openai
//!   4. copilot     — Copilot                            — feature: ai-copilot
//!   5. perplexity  — Perplexity (PERPLEXITY_API_KEY)   — feature: ai-perplexity
//!   6. deepseek    — DeepSeek   (DEEPSEEK_API_KEY)      — feature: ai-deepseek
//!
//! The preferred() method checks AEONMI_AI_PROVIDER env var first,
//! then falls back to the first provider whose API key is set,
//! then falls back to the first registered.

use anyhow::Result;

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chat(&self, prompt: &str) -> Result<String>;
    fn chat_stream(&self, _prompt: &str, _cb: &mut dyn FnMut(&str)) -> Result<()> {
        let full = self.chat(_prompt)?;
        _cb(&full);
        Ok(())
    }
}

// Always-compiled providers
pub mod openrouter;
pub mod claude;

// Feature-gated providers
#[cfg(feature = "ai-openai")]
pub mod openai;
#[cfg(feature = "ai-copilot")]
pub mod copilot;
#[cfg(feature = "ai-perplexity")]
pub mod perplexity;
#[cfg(feature = "ai-deepseek")]
pub mod deepseek;

pub struct AiRegistry {
    providers: Vec<Box<dyn AiProvider>>,
}

impl AiRegistry {
    pub fn new() -> Self {
        let mut r = Self { providers: Vec::new() };

        // OpenRouter first — free tier available, Warren's current setup
        r.providers.push(Box::new(openrouter::OpenRouter::default()));

        // Claude second — when ANTHROPIC_API_KEY is set
        r.providers.push(Box::new(claude::Claude::default()));

        #[cfg(feature = "ai-openai")]
        { r.providers.push(Box::new(openai::OpenAi::default())); }
        #[cfg(feature = "ai-copilot")]
        { r.providers.push(Box::new(copilot::Copilot::default())); }
        #[cfg(feature = "ai-perplexity")]
        { r.providers.push(Box::new(perplexity::Perplexity::default())); }
        #[cfg(feature = "ai-deepseek")]
        { r.providers.push(Box::new(deepseek::DeepSeek::default())); }

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
    /// 2. First provider whose API key env var is set
    /// 3. First registered provider
    pub fn preferred(&self) -> Option<&dyn AiProvider> {
        // Explicit override
        if let Ok(name) = std::env::var("AEONMI_AI_PROVIDER") {
            if let Some(p) = self.get(&name) {
                return Some(p);
            }
        }

        // Auto-detect: first provider with a key set
        let key_vars = [
            ("openrouter", "OPENROUTER_API_KEY"),
            ("claude",     "ANTHROPIC_API_KEY"),
            ("openai",     "OPENAI_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("deepseek",   "DEEPSEEK_API_KEY"),
        ];
        for (provider_name, env_var) in &key_vars {
            if std::env::var(env_var).is_ok() {
                if let Some(p) = self.get(provider_name) {
                    return Some(p);
                }
            }
        }

        // Fallback: first registered
        self.providers.first().map(|b| b.as_ref())
    }
}

impl Default for AiRegistry {
    fn default() -> Self { Self::new() }
}
