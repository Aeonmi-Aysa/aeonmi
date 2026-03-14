//! AI Mother Module: multi-provider abstraction.
use anyhow::Result;

pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chat(&self, prompt: &str) -> Result<String>;
    /// Send a multi-turn conversation. Each element is (role, content) where role is
    /// "user" or "assistant". Defaults to a single-turn call for providers that don't
    /// override this.
    fn chat_history(&self, messages: &[(&str, &str)]) -> Result<String> {
        // Default: concatenate all turns and send as single prompt
        let prompt = messages.iter()
            .map(|(role, content)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n");
        self.chat(&prompt)
    }
    fn chat_stream(&self, _prompt: &str, _cb: &mut dyn FnMut(&str)) -> Result<()> {
        // Default fallback: call non-streaming and emit once
        let full = self.chat(_prompt)?;
        _cb(&full);
        Ok(())
    }
}

#[cfg(feature = "ai-openai")]
pub mod openai;
#[cfg(feature = "ai-copilot")]
pub mod copilot;
#[cfg(feature = "ai-perplexity")]
pub mod perplexity;
#[cfg(feature = "ai-deepseek")]
pub mod deepseek;

// OpenRouter and Claude are always compiled; runtime key detection decides availability.
pub mod openrouter;
pub mod claude;

pub struct AiRegistry {
    providers: Vec<Box<dyn AiProvider>>,
}

impl AiRegistry {
    /// Alias for `from_env()` — backward-compatible constructor.
    pub fn new() -> Self {
        Self::from_env()
    }

    /// Build a registry from runtime environment detection.
    /// OpenRouter is checked first; Claude is the fallback.
    pub fn from_env() -> Self {
        let mut r = Self { providers: Vec::new() };
        if let Some(p) = openrouter::OpenRouter::from_env() {
            r.providers.push(Box::new(p));
        }
        if let Some(p) = claude::Claude::from_env() {
            r.providers.push(Box::new(p));
        }
        // Feature-gated legacy providers
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

    /// Return the preferred (first available) provider, or `None` if none configured.
    pub fn preferred(&self) -> Option<&dyn AiProvider> {
        self.providers.first().map(|b| b.as_ref())
    }

    /// Return a human-readable banner line describing the active AI provider.
    pub fn banner(&self) -> String {
        match self.preferred() {
            Some(p) => format!("AI: {} ACTIVE ✓", p.name()),
            None => "AI: No provider active — set OPENROUTER_API_KEY or ANTHROPIC_API_KEY".to_string(),
        }
    }

    pub fn list(&self) -> Vec<&'static str> { self.providers.iter().map(|p| p.name()).collect() }
    pub fn first(&self) -> Option<&Box<dyn AiProvider>> { self.providers.first() }
    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.iter().find(|p| p.name() == name).map(|b| b.as_ref())
    }
}

impl Default for AiRegistry {
    fn default() -> Self {
        Self::from_env()
    }
}


