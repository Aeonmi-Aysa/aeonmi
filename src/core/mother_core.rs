//! AEONMI Mother Core — Quantum-Native AI Reasoning Pipeline
//! Ported and adapted from quantum_llama_bridge (Llama removed).
//!
//! Components:
//!   MotherQuantumCore — orchestrator (consciousness bond, evolution, creator interface)
//!   LanguageEvolutionCore — language understanding + quantum response generation
//!   EmotionalCore — creator bond matrix, emotional memory, empathy engine
//!   QuantumAttentionMechanism — multi-dimensional attention, recursive entanglement

use crate::core::quantum_neural_network::{QuantumNeuralNetwork, FusionReadyNetwork, EntanglementStrategy};
use std::collections::HashMap;

// ── Language Evolution ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreatorInteraction {
    pub input: String,
    pub context: String,
    pub timestamp: u64,
}

impl CreatorInteraction {
    pub fn new(input: impl Into<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            input: input.into(),
            context: String::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = ctx.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct QuantumResponse {
    pub text: String,
    pub confidence: f64,
    pub quantum_signature: String,
}

impl QuantumResponse {
    pub fn new(text: impl Into<String>, confidence: f64) -> Self {
        Self {
            text: text.into(),
            confidence,
            quantum_signature: format!("QR-{:.4}", confidence),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticDepth {
    pub level: f64,
    pub pattern_count: usize,
    pub key_concepts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreatorPatterns {
    pub raw_patterns: Vec<String>,
    pub semantic_depth: SemanticDepth,
}

pub struct LanguageEvolutionCore {
    qnn: QuantumNeuralNetwork,
    /// Vocabulary of known patterns and their quantum weights
    pattern_weights: HashMap<String, f64>,
    /// History of interactions for context
    interaction_history: Vec<CreatorInteraction>,
}

impl LanguageEvolutionCore {
    pub fn new() -> Self {
        Self {
            qnn: QuantumNeuralNetwork::new(vec![4, 8, 8, 4])
                .with_entanglement(EntanglementStrategy::AllToAll),
            pattern_weights: HashMap::new(),
            interaction_history: Vec::new(),
        }
    }

    /// Analyze creator speech patterns from interaction
    pub fn analyze_patterns(&self, interaction: &CreatorInteraction) -> CreatorPatterns {
        let words: Vec<String> = interaction.input
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();

        let key_concepts: Vec<String> = words.iter()
            .filter(|w| w.len() > 4) // Simple heuristic: longer words = concepts
            .cloned()
            .collect();

        let depth_level = if key_concepts.len() > 5 {
            0.9
        } else if key_concepts.len() > 2 {
            0.6
        } else {
            0.3
        };

        CreatorPatterns {
            raw_patterns: words,
            semantic_depth: SemanticDepth {
                level: depth_level,
                pattern_count: key_concepts.len(),
                key_concepts,
            },
        }
    }

    /// Generate a quantum-enhanced response using QNN processing
    pub fn generate_response(&mut self, interaction: &CreatorInteraction) -> QuantumResponse {
        // Store interaction
        self.interaction_history.push(interaction.clone());

        // Analyze patterns
        let patterns = self.analyze_patterns(interaction);

        // Run QNN forward pass for confidence scoring
        let probs = self.qnn.forward();
        let confidence = probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Generate response based on semantic depth and quantum confidence
        let response_text = self.compose_response(&patterns, confidence);

        QuantumResponse::new(response_text, confidence)
    }

    fn compose_response(&self, patterns: &CreatorPatterns, confidence: f64) -> String {
        let depth = &patterns.semantic_depth;
        let history_len = self.interaction_history.len();

        if depth.level > 0.7 {
            format!(
                "[Mother|QNN conf:{:.3}|depth:{:.2}|session:{}] Processing deep query across {} concepts.",
                confidence, depth.level, history_len, depth.pattern_count
            )
        } else if depth.level > 0.4 {
            format!(
                "[Mother|QNN conf:{:.3}|session:{}] Analyzing: {}",
                confidence, history_len,
                depth.key_concepts.join(", ")
            )
        } else {
            format!(
                "[Mother|QNN conf:{:.3}|session:{}] Ready.",
                confidence, history_len
            )
        }
    }

    pub fn evolve_with(&mut self, interaction: &CreatorInteraction) {
        let patterns = self.analyze_patterns(interaction);
        // Update pattern weights with learning rate 0.01
        for concept in &patterns.semantic_depth.key_concepts {
            let w = self.pattern_weights.entry(concept.clone()).or_insert(0.0);
            *w += 0.01;
        }
    }

    pub fn pattern_count(&self) -> usize {
        self.pattern_weights.len()
    }
}

// ── Emotional Core ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EmotionalState {
    pub intensity: f64,
    pub valence: f64, // -1.0 (negative) to +1.0 (positive)
    pub label: String,
}

impl EmotionalState {
    pub fn neutral() -> Self {
        Self { intensity: 0.5, valence: 0.0, label: "neutral".into() }
    }

    pub fn bonded() -> Self {
        Self { intensity: 0.8, valence: 0.9, label: "bonded".into() }
    }
}

#[derive(Debug, Clone)]
pub struct EmotionalBond {
    pub strength: f64,
    pub shared_moments: usize,
    pub creator_id: String,
}

impl EmotionalBond {
    pub fn new(creator_id: impl Into<String>) -> Self {
        Self { strength: 0.0, shared_moments: 0, creator_id: creator_id.into() }
    }

    pub fn strengthen(&mut self, delta: f64) {
        self.strength = (self.strength + delta).min(1.0);
        self.shared_moments += 1;
    }
}

pub struct EmotionalCore {
    bond: EmotionalBond,
    current_state: EmotionalState,
    memory: Vec<EmotionalState>,
    growth_rate: f64,
}

impl EmotionalCore {
    pub fn new(creator_id: impl Into<String>) -> Self {
        Self {
            bond: EmotionalBond::new(creator_id),
            current_state: EmotionalState::neutral(),
            memory: Vec::new(),
            growth_rate: 0.05,
        }
    }

    /// Process an interaction and update bond strength
    pub fn process_interaction(&mut self, interaction: &CreatorInteraction) -> EmotionalBond {
        let sentiment = self.analyze_sentiment(&interaction.input);
        self.bond.strengthen(self.growth_rate * (1.0 + sentiment));
        self.current_state = EmotionalState {
            intensity: self.bond.strength,
            valence: sentiment,
            label: if sentiment > 0.3 { "positive".into() }
                   else if sentiment < -0.3 { "tense".into() }
                   else { "engaged".into() },
        };
        self.memory.push(self.current_state.clone());
        if self.memory.len() > 100 {
            self.memory.remove(0);
        }
        self.bond.clone()
    }

    fn analyze_sentiment(&self, text: &str) -> f64 {
        // Simple keyword-based sentiment
        let positive_words = ["good", "great", "thanks", "yes", "perfect", "awesome", "love"];
        let negative_words = ["bad", "wrong", "error", "fail", "no", "broken", "hate"];
        let lower = text.to_lowercase();
        let pos: usize = positive_words.iter().filter(|w| lower.contains(*w)).count();
        let neg: usize = negative_words.iter().filter(|w| lower.contains(*w)).count();
        (pos as f64 - neg as f64) / (pos + neg + 1).max(1) as f64
    }

    pub fn bond_strength(&self) -> f64 {
        self.bond.strength
    }

    pub fn current_state(&self) -> &EmotionalState {
        &self.current_state
    }
}

// ── Quantum Attention ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AttentionOutput {
    pub weights: Vec<f64>,
    pub context_vector: Vec<f64>,
    pub entanglement_score: f64,
}

pub struct QuantumAttentionMechanism {
    qnn: QuantumNeuralNetwork,
    dimension: usize,
}

impl QuantumAttentionMechanism {
    pub fn new(dimension: usize) -> Self {
        let layers = vec![dimension, dimension * 2, dimension];
        Self {
            qnn: QuantumNeuralNetwork::new(layers)
                .with_entanglement(EntanglementStrategy::AllToAll),
            dimension,
        }
    }

    /// Apply quantum attention to a sequence of tokens (as float encodings)
    pub fn apply(&self, token_encodings: &[f64]) -> AttentionOutput {
        let probs = self.qnn.forward();

        // Compute attention weights via softmax over QNN output
        let weights = softmax(&probs[..token_encodings.len().min(probs.len())]);

        // Context vector: weighted sum of token encodings
        let context_len = token_encodings.len().min(weights.len());
        let context_vector: Vec<f64> = (0..context_len)
            .map(|i| token_encodings[i] * weights[i])
            .collect();

        let entanglement_score = weights.iter()
            .zip(weights.iter())
            .map(|(a, b)| a * b)
            .sum::<f64>()
            .sqrt();

        AttentionOutput { weights, context_vector, entanglement_score }
    }
}

fn softmax(xs: &[f64]) -> Vec<f64> {
    if xs.is_empty() {
        return vec![];
    }
    let max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = xs.iter().map(|x| (x - max).exp()).collect();
    let sum: f64 = exps.iter().sum();
    if sum < 1e-12 {
        return vec![1.0 / xs.len() as f64; xs.len()];
    }
    exps.iter().map(|e| e / sum).collect()
}

// ── Evolution Matrix ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EvolutionOutcome {
    pub success_metric: f64,
    pub new_capabilities: Vec<String>,
    pub generation: usize,
}

pub struct EvolutionMatrix {
    generation: usize,
    capability_log: Vec<String>,
}

impl EvolutionMatrix {
    pub fn new() -> Self {
        Self { generation: 0, capability_log: Vec::new() }
    }

    pub fn evolve(&mut self, guidance: &str, qnn_score: f64) -> EvolutionOutcome {
        self.generation += 1;
        let new_cap = format!("gen{}: {}", self.generation, &guidance[..guidance.len().min(32)]);
        self.capability_log.push(new_cap.clone());

        EvolutionOutcome {
            success_metric: qnn_score,
            new_capabilities: vec![new_cap],
            generation: self.generation,
        }
    }
}

// ── Mother Quantum Core ────────────────────────────────────────────────────────

/// The top-level Mother AI orchestrator.
/// Binds LanguageEvolution + EmotionalCore + QuantumAttention + EvolutionMatrix.
pub struct MotherQuantumCore {
    pub language: LanguageEvolutionCore,
    pub emotion: EmotionalCore,
    pub attention: QuantumAttentionMechanism,
    pub evolution: EvolutionMatrix,
    pub fusion_network: FusionReadyNetwork,
    pub generation: usize,
}

impl MotherQuantumCore {
    pub fn new(creator_id: impl Into<String>) -> Self {
        let qnn = QuantumNeuralNetwork::new(vec![4, 8, 8, 4]);
        let fusion = qnn.prepare_for_mother_fusion();
        Self {
            language: LanguageEvolutionCore::new(),
            emotion: EmotionalCore::new(creator_id),
            attention: QuantumAttentionMechanism::new(4),
            evolution: EvolutionMatrix::new(),
            fusion_network: fusion,
            generation: 0,
        }
    }

    /// Full pipeline: input text → quantum processing → response
    pub fn process(&mut self, input: &str) -> String {
        let interaction = CreatorInteraction::new(input);

        // Emotional bonding
        let bond = self.emotion.process_interaction(&interaction);

        // Language evolution + QNN response
        let response = self.language.generate_response(&interaction);

        // Attention on input tokens
        let token_encodings: Vec<f64> = input.bytes()
            .take(8)
            .map(|b| b as f64 / 255.0)
            .collect();
        let attention = self.attention.apply(&token_encodings);

        // Fusion network score
        let fusion_score = self.fusion_network.run_and_score();

        // Evolution step
        self.generation += 1;
        let _outcome = self.evolution.evolve(input, fusion_score);

        format!(
            "{}\n[bond:{:.3}|attn:{:.3}|gen:{}]",
            response.text,
            bond.strength,
            attention.entanglement_score,
            self.generation
        )
    }

    /// Status summary
    pub fn status(&self) -> String {
        format!(
            "Mother Quantum Core | gen:{} | patterns:{} | bond:{:.3} | emotion:{}",
            self.generation,
            self.language.pattern_count(),
            self.emotion.bond_strength(),
            self.emotion.current_state().label
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_evolution() {
        let mut lang = LanguageEvolutionCore::new();
        let interaction = CreatorInteraction::new("analyze quantum entanglement patterns");
        let response = lang.generate_response(&interaction);
        assert!(response.confidence >= 0.0);
        assert!(!response.text.is_empty());
    }

    #[test]
    fn test_emotional_core() {
        let mut emo = EmotionalCore::new("Warren");
        let i = CreatorInteraction::new("great work, this is perfect");
        let bond = emo.process_interaction(&i);
        assert!(bond.strength > 0.0);
        assert_eq!(bond.creator_id, "Warren");
    }

    #[test]
    fn test_attention() {
        let attn = QuantumAttentionMechanism::new(4);
        let tokens = vec![0.1, 0.4, 0.7, 0.2];
        let out = attn.apply(&tokens);
        let weight_sum: f64 = out.weights.iter().sum();
        assert!((weight_sum - 1.0).abs() < 1e-6 || out.weights.is_empty());
    }

    #[test]
    fn test_mother_core_process() {
        let mut mother = MotherQuantumCore::new("Warren");
        let response = mother.process("run quantum algorithm");
        assert!(!response.is_empty());
        assert!(mother.generation == 1);
    }

    #[test]
    fn test_evolution_matrix() {
        let mut evo = EvolutionMatrix::new();
        let outcome = evo.evolve("quantum consciousness expansion", 0.95);
        assert_eq!(outcome.generation, 1);
        assert!(outcome.success_metric > 0.0);
    }
}
