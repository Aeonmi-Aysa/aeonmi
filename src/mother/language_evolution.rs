//! LanguageEvolutionCore — speech pattern analysis, semantic depth, evolved responses.
//! Migrated from quantum_llama_bridge/language_evolution.rs.
//! Llama stripped. Runs on pattern analysis + Aeonmi runtime.

use std::collections::HashMap;

// ─── Data types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreatorLanguagePattern {
    pub keyword: String,
    pub frequency: usize,
    pub semantic_weight: f64,
}

#[derive(Debug, Clone)]
pub struct SemanticDepth {
    /// 0.0–1.0
    pub level: f64,
    pub dominant_topic: Option<String>,
    pub complexity_score: f64,
}

impl SemanticDepth {
    fn compute(input: &str, history: &[String]) -> Self {
        let words: Vec<&str> = input.split_whitespace().collect();
        let unique: std::collections::HashSet<&str> = words.iter().cloned().collect();

        // Lexical richness
        let richness = if words.is_empty() {
            0.0
        } else {
            unique.len() as f64 / words.len() as f64
        };

        // Topic detection via keyword density
        let topics = [
            ("quantum", &["qubit", "superpose", "entangle", "measure", "gate", "circuit", "quantum"] as &[&str]),
            ("code", &["function", "let", "return", "loop", "compile", "debug", "impl"]),
            ("system", &["status", "health", "init", "boot", "vault", "seal", "glyph"]),
            ("ai", &["mother", "learn", "evolve", "neural", "consciousness", "emotion"]),
        ];

        let lower = input.to_lowercase();
        let dominant = topics
            .iter()
            .map(|(name, kws)| {
                let score: usize = kws.iter().filter(|&&k| lower.contains(k)).count();
                (*name, score)
            })
            .max_by_key(|&(_, s)| s)
            .and_then(|(name, score)| if score > 0 { Some(name.to_string()) } else { None });

        // Context bonus from history
        let context_bonus = if history.is_empty() {
            0.0
        } else {
            let hist_text = history.join(" ").to_lowercase();
            let overlap: usize = unique.iter().filter(|&&w| hist_text.contains(w)).count();
            (overlap as f64 / unique.len().max(1) as f64) * 0.2
        };

        let level = (richness * 0.6 + context_bonus + 0.1).clamp(0.0, 1.0);
        let complexity = (words.len() as f64 / 20.0).clamp(0.0, 1.0);

        Self { level, dominant_topic: dominant, complexity_score: complexity }
    }
}

#[derive(Debug, Clone)]
pub struct EvolvedLanguage {
    pub evolved_text: String,
    pub evolution_applied: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub message: String,
    pub history: Vec<String>,
}

impl ConversationContext {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string(), history: Vec::new() }
    }

    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }
}

// ─── LanguageEvolutionCore ───────────────────────────────────────────────────

/// Analyzes creator language patterns and evolves Mother's response style.
pub struct LanguageEvolutionCore {
    /// Keyword → frequency across all interactions.
    keyword_freq: HashMap<String, usize>,
    /// How many interactions have been processed.
    pub interaction_count: usize,
    /// Evolution generation.
    pub generation: u64,
    /// Moving average of semantic depth (for trend tracking).
    semantic_depth_avg: f64,
    /// History of recent inputs (for context analysis).
    history: Vec<String>,
    pub max_history: usize,
}

impl Default for LanguageEvolutionCore {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageEvolutionCore {
    pub fn new() -> Self {
        Self {
            keyword_freq: HashMap::new(),
            interaction_count: 0,
            generation: 0,
            semantic_depth_avg: 0.0,
            history: Vec::new(),
            max_history: 200,
        }
    }

    /// Analyze an interaction and evolve language understanding.
    pub fn evolve_with_creator(&mut self, message: &str) -> EvolvedLanguage {
        // Update keyword frequencies
        for word in message.split_whitespace() {
            let clean = word.to_lowercase()
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if clean.len() > 2 {
                *self.keyword_freq.entry(clean).or_insert(0) += 1;
            }
        }

        let depth = SemanticDepth::compute(message, &self.history);

        // Update rolling average
        self.semantic_depth_avg = self.semantic_depth_avg * 0.9 + depth.level * 0.1;

        // Record history
        self.history.push(message.to_string());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        self.interaction_count += 1;

        // Compute which evolutions were applied
        let mut evolutions = Vec::new();
        if depth.level > 0.6 { evolutions.push("deep_semantic_analysis".to_string()); }
        if depth.complexity_score > 0.5 { evolutions.push("high_complexity_processing".to_string()); }
        if let Some(ref topic) = depth.dominant_topic {
            evolutions.push(format!("topic_focus:{}", topic));
        }

        EvolvedLanguage {
            evolved_text: format!(
                "[evolved] semantic_depth={:.2} topic={} avg_depth={:.2}",
                depth.level,
                depth.dominant_topic.as_deref().unwrap_or("general"),
                self.semantic_depth_avg
            ),
            evolution_applied: evolutions,
        }
    }

    /// Generate an evolved response for a given context.
    pub fn generate_evolved_response(&self, ctx: &ConversationContext) -> String {
        let depth = SemanticDepth::compute(&ctx.message, &ctx.history);

        // Build a context-aware prefix based on semantic depth
        let prefix = match depth.level {
            d if d > 0.7 => "[ high-depth quantum context ]",
            d if d > 0.4 => "[ standard context ]",
            _ => "[ surface context ]",
        };

        // Use top keywords to guide response tone
        let top_kw = self.top_keywords(3);
        let kw_str = if top_kw.is_empty() {
            "—".to_string()
        } else {
            top_kw.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ")
        };

        format!(
            "{} Responding to: \"{}\" | Dominant vocabulary: [{}] | Evolution gen {}",
            prefix,
            ctx.message,
            kw_str,
            self.generation
        )
    }

    /// Top N most-frequent keywords.
    pub fn top_keywords(&self, n: usize) -> Vec<(String, usize)> {
        let mut pairs: Vec<(String, usize)> = self.keyword_freq
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }

    /// Trigger evolution: increment generation and prune low-frequency vocab.
    pub fn trigger_evolution(&mut self) {
        self.generation += 1;
        // Prune keywords seen only once (noise reduction)
        self.keyword_freq.retain(|_, &mut v| v > 1);
    }

    pub fn summary(&self) -> String {
        format!(
            "LanguageEvolution: gen={} interactions={} vocab_size={} avg_depth={:.3}",
            self.generation,
            self.interaction_count,
            self.keyword_freq.len(),
            self.semantic_depth_avg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolve_builds_vocabulary() {
        let mut evo = LanguageEvolutionCore::new();
        evo.evolve_with_creator("quantum circuit entangle measure qubit");
        evo.evolve_with_creator("quantum superpose apply gate");
        assert!(evo.keyword_freq.contains_key("quantum"));
        assert!(*evo.keyword_freq.get("quantum").unwrap() >= 2);
    }

    #[test]
    fn test_top_keywords_sorted() {
        let mut evo = LanguageEvolutionCore::new();
        for _ in 0..5 { evo.evolve_with_creator("quantum quantum quantum"); }
        evo.evolve_with_creator("code code");
        let top = evo.top_keywords(2);
        assert!(!top.is_empty());
        assert_eq!(top[0].0, "quantum");
    }

    #[test]
    fn test_generation_increments() {
        let mut evo = LanguageEvolutionCore::new();
        evo.trigger_evolution();
        evo.trigger_evolution();
        assert_eq!(evo.generation, 2);
    }

    #[test]
    fn test_semantic_depth_quantum_topic() {
        let depth = SemanticDepth::compute("qubit entangle superpose circuit gate measure", &[]);
        assert!(depth.level > 0.0);
        assert_eq!(depth.dominant_topic.as_deref(), Some("quantum"));
    }
}
