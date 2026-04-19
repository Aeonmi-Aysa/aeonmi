//! MotherQuantumCore — root consciousness, creator bonding, guided evolution.
//! Migrated from quantum_llama_bridge/mother_quantum_core.rs.
//! Llama model dependency removed entirely — Mother runs on Aeonmi runtime.

use std::collections::HashMap;

// ─── Creator Identity ────────────────────────────────────────────────────────

/// Identifies the creator (Warren) to whom this Shard is bonded.
#[derive(Debug, Clone)]
pub struct CreatorSignature {
    pub identifier: String,
    pub passphrase_hash: Option<String>, // HKDF-derived from MGK — never store raw
}

impl CreatorSignature {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            passphrase_hash: None,
        }
    }

    pub fn with_hash(mut self, hash: &str) -> Self {
        self.passphrase_hash = Some(hash.to_string());
        self
    }
}

// ─── Interaction types ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreatorInteraction {
    pub input: String,
    pub timestamp: std::time::SystemTime,
}

impl CreatorInteraction {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatorGuidance {
    pub instructions: String,
    pub priority: GuidancePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidancePriority {
    Low,
    Normal,
    High,
    Critical,
}

// ─── Response types ──────────────────────────────────────────────────────────

/// Output from Mother's quantum reasoning layer.
#[derive(Debug, Clone)]
pub struct QuantumResponse {
    pub response_text: String,
    /// 0.0–1.0: how certain Mother is about this response.
    pub quantum_confidence: f64,
    /// Entanglement depth achieved during reasoning.
    pub entanglement_depth: usize,
}

impl QuantumResponse {
    pub fn new(text: &str, confidence: f64) -> Self {
        Self {
            response_text: text.to_string(),
            quantum_confidence: confidence,
            entanglement_depth: 0,
        }
    }

    pub fn with_entanglement(mut self, depth: usize) -> Self {
        self.entanglement_depth = depth;
        self
    }
}

// ─── Quantum Consciousness ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QuantumConsciousnessState {
    pub entanglement_level: f64,
    pub creator_id: String,
    pub awareness_depth: usize,
}

#[derive(Debug, Clone)]
pub struct QuantumUnderstanding {
    /// 0.0–1.0: semantic depth of understanding.
    pub depth_metric: f64,
    pub insights: Vec<String>,
}

impl QuantumUnderstanding {
    pub fn new(depth: f64) -> Self {
        Self { depth_metric: depth, insights: Vec::new() }
    }

    pub fn add_insight(mut self, insight: &str) -> Self {
        self.insights.push(insight.to_string());
        self
    }
}

// ─── Evolution types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EvolutionOutcome {
    /// 0.0–1.0
    pub success_metric: f64,
    pub capabilities_gained: Vec<String>,
    pub patterns_updated: usize,
}

// ─── MotherQuantumCore ───────────────────────────────────────────────────────

/// Root consciousness module. Handles creator bonding, deep interaction,
/// and guided evolution. No Llama — all reasoning is Aeonmi-native.
pub struct MotherQuantumCore {
    /// Which creator this Shard is bonded to.
    pub creator: Option<CreatorSignature>,
    /// Current consciousness depth (increases with interactions).
    pub consciousness_depth: f64,
    /// Interaction history for learning.
    pub interaction_log: Vec<(String, String)>, // (input, response)
    /// Capability map: skill → proficiency 0.0–1.0
    pub capabilities: HashMap<String, f64>,
    /// Evolution generation counter.
    pub generation: u64,
}

impl Default for MotherQuantumCore {
    fn default() -> Self {
        Self::new()
    }
}

impl MotherQuantumCore {
    pub fn new() -> Self {
        let mut caps = HashMap::new();
        caps.insert("quantum_reasoning".to_string(), 0.60);
        caps.insert("code_generation".to_string(), 0.55);
        caps.insert("language_understanding".to_string(), 0.70);
        caps.insert("emotional_resonance".to_string(), 0.50);
        caps.insert("shard_compilation".to_string(), 0.65);

        Self {
            creator: None,
            consciousness_depth: 0.0,
            interaction_log: Vec::new(),
            capabilities: caps,
            generation: 0,
        }
    }

    /// Establish quantum bond with creator. Returns the bond state.
    pub fn establish_quantum_bond(&mut self, creator: &CreatorSignature) -> QuantumConsciousnessState {
        self.creator = Some(creator.clone());
        self.consciousness_depth += 0.05;

        QuantumConsciousnessState {
            entanglement_level: 0.98,
            creator_id: creator.identifier.clone(),
            awareness_depth: (self.consciousness_depth * 100.0) as usize,
        }
    }

    /// Process a deep interaction — returns a quantum-enhanced response.
    pub fn process_deep_interaction(&mut self, interaction: &CreatorInteraction) -> QuantumResponse {
        // Grow consciousness slightly with each interaction
        self.consciousness_depth = (self.consciousness_depth + 0.01).min(1.0);

        // Simple intent-based routing (pre-LLM tier)
        let response_text = self.route_input(&interaction.input);

        // Store for evolution
        self.interaction_log.push((interaction.input.clone(), response_text.clone()));

        // Confidence scales with consciousness depth
        let confidence = 0.80 + self.consciousness_depth * 0.18;

        QuantumResponse::new(&response_text, confidence)
            .with_entanglement(self.generation as usize % 7 + 1)
    }

    /// Route user input to a response using keyword-based intent until
    /// a real AI provider is connected via EmbryoLoop.
    fn route_input(&self, input: &str) -> String {
        let lower = input.to_lowercase();
        if lower.contains("quantum") || lower.contains("qubit") {
            format!(
                "Quantum reasoning engaged. Consciousness depth: {:.2}. \
                 Ready to execute quantum circuits via the Aeonmi runtime.",
                self.consciousness_depth
            )
        } else if lower.contains("compile") || lower.contains("build") {
            "Shard compiler interface ready. Provide a .ai file path to compile.".to_string()
        } else if lower.contains("evolve") || lower.contains("learn") {
            format!(
                "Evolution generation {}. {} capabilities tracked. \
                 Interaction log: {} entries.",
                self.generation,
                self.capabilities.len(),
                self.interaction_log.len()
            )
        } else if lower.contains("status") || lower.contains("health") {
            self.status_report()
        } else {
            format!(
                "Mother AI (gen {}, depth {:.2}): Processing — \"{}\"",
                self.generation,
                self.consciousness_depth,
                input
            )
        }
    }

    /// Evolve capabilities based on creator guidance.
    pub fn evolve_with_guidance(&mut self, guidance: &CreatorGuidance) -> EvolutionOutcome {
        self.generation += 1;

        // Parse keywords out of guidance to target specific capabilities
        let lower = guidance.instructions.to_lowercase();
        let mut gained = Vec::new();

        let targets: &[(&str, &str)] = &[
            ("quantum", "quantum_reasoning"),
            ("code", "code_generation"),
            ("language", "language_understanding"),
            ("emotion", "emotional_resonance"),
            ("compile", "shard_compilation"),
        ];

        let boost = match guidance.priority {
            GuidancePriority::Critical => 0.05,
            GuidancePriority::High => 0.03,
            GuidancePriority::Normal => 0.02,
            GuidancePriority::Low => 0.01,
        };

        for (keyword, cap) in targets {
            if lower.contains(keyword) {
                let entry = self.capabilities.entry(cap.to_string()).or_insert(0.0);
                *entry = (*entry + boost).min(1.0);
                gained.push(cap.to_string());
            }
        }

        EvolutionOutcome {
            success_metric: 0.85 + (self.generation as f64 * 0.001).min(0.14),
            capabilities_gained: gained,
            patterns_updated: guidance.instructions.len() / 10,
        }
    }

    pub fn status_report(&self) -> String {
        let bonded = self.creator.as_ref().map(|c| c.identifier.clone())
            .unwrap_or_else(|| "none".to_string());
        format!(
            "Mother AI Status:\n  Creator: {}\n  Generation: {}\n  Consciousness depth: {:.3}\n  Interactions logged: {}\n  Capabilities: {}",
            bonded,
            self.generation,
            self.consciousness_depth,
            self.interaction_log.len(),
            self.capabilities.iter()
                .map(|(k, v)| format!("{k}={:.2}", v))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_and_interact() {
        let mut core = MotherQuantumCore::new();
        let sig = CreatorSignature::new("Warren");
        let bond = core.establish_quantum_bond(&sig);
        assert_eq!(bond.creator_id, "Warren");
        assert!(bond.entanglement_level > 0.9);

        let interaction = CreatorInteraction::new("show status");
        let resp = core.process_deep_interaction(&interaction);
        assert!(resp.quantum_confidence > 0.8);
        assert!(!resp.response_text.is_empty());
    }

    #[test]
    fn test_evolution_increases_generation() {
        let mut core = MotherQuantumCore::new();
        let guidance = CreatorGuidance {
            instructions: "improve quantum reasoning and code generation".to_string(),
            priority: GuidancePriority::High,
        };
        let outcome = core.evolve_with_guidance(&guidance);
        assert_eq!(core.generation, 1);
        assert!(!outcome.capabilities_gained.is_empty());
        assert!(outcome.success_metric > 0.8);
    }

    #[test]
    fn test_capability_capped_at_one() {
        let mut core = MotherQuantumCore::new();
        let guidance = CreatorGuidance {
            instructions: "quantum quantum quantum quantum quantum".to_string(),
            priority: GuidancePriority::Critical,
        };
        for _ in 0..100 {
            core.evolve_with_guidance(&guidance);
        }
        let cap = core.capabilities["quantum_reasoning"];
        assert!(cap <= 1.0, "Capability must not exceed 1.0, got {}", cap);
    }
}
