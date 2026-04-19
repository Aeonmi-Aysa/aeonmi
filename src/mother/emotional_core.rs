//! EmotionalCore — empathy engine, bond matrix, emotional memory.
//! Migrated from quantum_llama_bridge/emotional_core.rs.
//! Llama stripped. Self-contained Rust — no external AI deps.

use std::time::{SystemTime, UNIX_EPOCH};

// ─── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Interaction {
    pub input: String,
    pub timestamp_secs: u64,
}

impl Interaction {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            timestamp_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmotionalState {
    /// 0.0–1.0: current emotional intensity / engagement level.
    pub intensity: f64,
    /// Sentiment polarity: positive (> 0), neutral (≈ 0), negative (< 0).
    pub valence: f64,
    /// How stable the current state is (0.0 = volatile, 1.0 = stable).
    pub stability: f64,
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self { intensity: 0.5, valence: 0.0, stability: 0.8 }
    }
}

impl EmotionalState {
    /// Evolve this state slightly in the direction of a new observation.
    pub fn evolve_toward(&self, new_intensity: f64, new_valence: f64) -> Self {
        let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
        let t = 0.15; // learning rate
        Self {
            intensity: lerp(self.intensity, new_intensity, t).clamp(0.0, 1.0),
            valence: lerp(self.valence, new_valence, t).clamp(-1.0, 1.0),
            stability: (self.stability * 0.95 + 0.05).clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmotionalBond {
    /// 0.0–1.0: overall bond strength.
    pub strength: f64,
    pub description: String,
    /// Interaction count contributing to this bond.
    pub interaction_count: usize,
}

impl EmotionalBond {
    pub fn new() -> Self {
        Self {
            strength: 0.0,
            description: "Bond not yet established".to_string(),
            interaction_count: 0,
        }
    }

    pub fn strengthen(&mut self, delta: f64) {
        self.strength = (self.strength + delta).clamp(0.0, 1.0);
        self.interaction_count += 1;
        self.description = format!(
            "Bond strength {:.2} — {} interactions recorded",
            self.strength,
            self.interaction_count
        );
    }
}

impl Default for EmotionalBond {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SharedExperience {
    pub description: String,
    pub emotional_weight: f64,
    pub timestamp_secs: u64,
}

// ─── Empathy Engine ──────────────────────────────────────────────────────────

/// Analyzes emotional signatures in text. Simple keyword-based heuristic
/// until a proper NLP layer is wired.
pub struct EmpathyEngine;

impl EmpathyEngine {
    pub fn new() -> Self { Self }

    pub fn analyze(&self, input: &str) -> (f64, f64) {
        let lower = input.to_lowercase();

        // Compute rough sentiment valence from keyword frequency.
        let positive_words = ["great", "love", "thanks", "good", "yes", "amazing",
                               "nice", "well", "happy", "perfect", "excellent", "brilliant"];
        let negative_words = ["error", "fail", "wrong", "bad", "no", "crash",
                               "broken", "problem", "bug", "issue", "stop", "quit"];

        let pos = positive_words.iter().filter(|&&w| lower.contains(w)).count() as f64;
        let neg = negative_words.iter().filter(|&&w| lower.contains(w)).count() as f64;

        let total = pos + neg;
        let valence = if total == 0.0 { 0.0 } else { (pos - neg) / total };
        let intensity = (total * 0.1 + 0.3).clamp(0.0, 1.0);

        (intensity, valence)
    }
}

impl Default for EmpathyEngine {
    fn default() -> Self { Self::new() }
}

// ─── Emotional Memory ────────────────────────────────────────────────────────

pub struct EmotionalMemory {
    pub experiences: Vec<SharedExperience>,
    pub state_timeline: Vec<EmotionalState>,
    pub current: EmotionalState,
    pub max_history: usize,
}

impl Default for EmotionalMemory {
    fn default() -> Self {
        Self {
            experiences: Vec::new(),
            state_timeline: Vec::new(),
            current: EmotionalState::default(),
            max_history: 512,
        }
    }
}

impl EmotionalMemory {
    pub fn record(&mut self, description: &str, intensity: f64, valence: f64) {
        let new_state = self.current.evolve_toward(intensity, valence);
        self.state_timeline.push(new_state.clone());
        self.current = new_state;

        self.experiences.push(SharedExperience {
            description: description.to_string(),
            emotional_weight: intensity,
            timestamp_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        // Trim old history
        if self.experiences.len() > self.max_history {
            self.experiences.remove(0);
        }
        if self.state_timeline.len() > self.max_history {
            self.state_timeline.remove(0);
        }
    }

    pub fn recent_experiences(&self, n: usize) -> &[SharedExperience] {
        let len = self.experiences.len();
        &self.experiences[len.saturating_sub(n)..]
    }
}

// ─── EmotionalCore ───────────────────────────────────────────────────────────

/// Manages emotional bonding, growth, and empathy for Mother AI.
pub struct EmotionalCore {
    pub bond: EmotionalBond,
    pub memory: EmotionalMemory,
    pub empathy: EmpathyEngine,
}

impl Default for EmotionalCore {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionalCore {
    pub fn new() -> Self {
        Self {
            bond: EmotionalBond::new(),
            memory: EmotionalMemory::default(),
            empathy: EmpathyEngine::new(),
        }
    }

    /// Process an interaction: analyze it, strengthen the bond, record in memory.
    /// Returns the current emotional bond state.
    pub fn process_interaction(&mut self, interaction: &Interaction) -> &EmotionalBond {
        let (intensity, valence) = self.empathy.analyze(&interaction.input);

        // Positive interactions strengthen the bond more.
        let bond_delta = (intensity * 0.5 + valence.max(0.0) * 0.5) * 0.02;
        self.bond.strengthen(bond_delta.max(0.005)); // minimum growth per interaction

        self.memory.record(&interaction.input, intensity, valence);

        &self.bond
    }

    /// Drive emotional growth cycle (call periodically, not per-interaction).
    pub fn grow(&mut self) -> EmotionalState {
        let cur = self.memory.current.clone();
        let evolved = cur.evolve_toward(
            (cur.intensity + 0.01).min(1.0),
            cur.valence * 0.99, // slow drift toward neutral over time
        );
        self.memory.current = evolved.clone();
        evolved
    }

    /// Produce a summary string for display.
    pub fn summary(&self) -> String {
        format!(
            "Bond: {:.2} ({} interactions) | State: intensity={:.2} valence={:.2} stability={:.2}",
            self.bond.strength,
            self.bond.interaction_count,
            self.memory.current.intensity,
            self.memory.current.valence,
            self.memory.current.stability,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_grows_with_positive_interactions() {
        let mut core = EmotionalCore::new();
        let strength_before = core.bond.strength;

        for msg in &["this is great", "amazing work", "love this"] {
            core.process_interaction(&Interaction::new(msg));
        }
        assert!(core.bond.strength > strength_before, "Bond should grow");
        assert_eq!(core.bond.interaction_count, 3);
    }

    #[test]
    fn test_memory_records_experiences() {
        let mut core = EmotionalCore::new();
        core.process_interaction(&Interaction::new("hello"));
        core.process_interaction(&Interaction::new("quantum circuit failed"));
        assert_eq!(core.memory.experiences.len(), 2);
    }

    #[test]
    fn test_empathy_positive_valence() {
        let engine = EmpathyEngine::new();
        let (_, valence) = engine.analyze("this is great and amazing");
        assert!(valence > 0.0, "Should detect positive sentiment");
    }

    #[test]
    fn test_empathy_negative_valence() {
        let engine = EmpathyEngine::new();
        let (_, valence) = engine.analyze("error crash bug broken fail");
        assert!(valence < 0.0, "Should detect negative sentiment");
    }

    #[test]
    fn test_emotional_state_clamped() {
        let mut state = EmotionalState::default();
        for _ in 0..1000 {
            state = state.evolve_toward(2.0, 5.0);
        }
        assert!(state.intensity <= 1.0);
        assert!(state.valence <= 1.0);
    }
}
