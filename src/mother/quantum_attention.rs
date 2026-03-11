//! QuantumAttentionMechanism — multi-dimensional attention over Aeonmi IR values.
//! Migrated from quantum_llama_bridge/quantum_attention.rs.
//! Llama stripped. Operates over the Aeonmi VM's Value types.

use std::collections::HashMap;

// ─── Attention primitives ────────────────────────────────────────────────────

/// A single attention head — computes a weight for a token/value slot.
#[derive(Debug, Clone)]
pub struct AttentionHead {
    pub dimension: usize,
    /// Weights vector of length `dimension`.
    pub weights: Vec<f64>,
}

impl AttentionHead {
    pub fn new(dimension: usize) -> Self {
        // Initialize with uniform weights
        let weights = vec![1.0 / dimension as f64; dimension];
        Self { dimension, weights }
    }

    /// Dot-product attention score between query and key vectors.
    pub fn score(&self, query: &[f64], key: &[f64]) -> f64 {
        let len = query.len().min(key.len()).min(self.weights.len());
        let raw: f64 = (0..len).map(|i| query[i] * key[i] * self.weights[i]).sum();
        // Scale by sqrt(d) to keep gradients stable
        raw / (self.dimension as f64).sqrt().max(1.0)
    }

    /// Update weights using a learning signal (gradient-free Hebbian rule).
    pub fn update(&mut self, query: &[f64], key: &[f64], learning_rate: f64) {
        let len = query.len().min(key.len()).min(self.dimension);
        for i in 0..len {
            self.weights[i] += learning_rate * query[i] * key[i];
        }
        self.normalize_weights();
    }

    fn normalize_weights(&mut self) {
        let sum: f64 = self.weights.iter().map(|w| w.abs()).sum();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        }
    }
}

// ─── Entanglement pattern ────────────────────────────────────────────────────

/// Tracks quantum-style entanglement between attention tokens.
/// When two tokens co-occur frequently, their entanglement increases.
#[derive(Debug, Clone, Default)]
pub struct EntanglementPattern {
    /// (token_a, token_b) → entanglement strength [0.0, 1.0]
    pairs: HashMap<(String, String), f64>,
}

impl EntanglementPattern {
    pub fn new() -> Self { Self::default() }

    /// Record a co-occurrence between two tokens.
    pub fn record_co_occurrence(&mut self, a: &str, b: &str) {
        let key = if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        };
        let entry = self.pairs.entry(key).or_insert(0.0);
        *entry = (*entry + 0.05).min(1.0);
    }

    /// Decay all entanglements (decoherence model).
    pub fn decohere(&mut self, rate: f64) {
        for v in self.pairs.values_mut() {
            *v *= 1.0 - rate;
        }
        self.pairs.retain(|_, v| *v > 0.001);
    }

    pub fn strength(&self, a: &str, b: &str) -> f64 {
        let key = if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        };
        *self.pairs.get(&key).unwrap_or(&0.0)
    }

    pub fn strongest_pairs(&self, n: usize) -> Vec<((String, String), f64)> {
        let mut pairs: Vec<_> = self.pairs.iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.truncate(n);
        pairs
    }
}

// ─── Quantum Memory Bank ─────────────────────────────────────────────────────

/// Stores attended tokens with their weights for retrieval.
pub struct QuantumMemoryBank {
    /// token → (value vector, access count)
    slots: HashMap<String, (Vec<f64>, usize)>,
    pub capacity: usize,
}

impl QuantumMemoryBank {
    pub fn new(capacity: usize) -> Self {
        Self { slots: HashMap::new(), capacity }
    }

    pub fn store(&mut self, key: &str, value: Vec<f64>) {
        if self.slots.len() >= self.capacity {
            // Evict least-accessed
            if let Some(lru_key) = self.slots.iter()
                .min_by_key(|(_, (_, count))| *count)
                .map(|(k, _)| k.clone())
            {
                self.slots.remove(&lru_key);
            }
        }
        self.slots.insert(key.to_string(), (value, 0));
    }

    pub fn retrieve(&mut self, key: &str) -> Option<Vec<f64>> {
        if let Some((v, count)) = self.slots.get_mut(key) {
            *count += 1;
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.slots.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
}

// ─── QuantumAttentionMechanism ───────────────────────────────────────────────

/// Multi-head attention over Aeonmi token streams.
/// Replaces the Llama-specific attention from quantum_llama_bridge.
pub struct QuantumAttentionMechanism {
    pub heads: Vec<AttentionHead>,
    pub entanglement: EntanglementPattern,
    pub memory: QuantumMemoryBank,
    pub learning_rate: f64,
}

impl QuantumAttentionMechanism {
    pub fn new(num_heads: usize, dimension: usize) -> Self {
        let heads = (0..num_heads).map(|_| AttentionHead::new(dimension)).collect();
        Self {
            heads,
            entanglement: EntanglementPattern::new(),
            memory: QuantumMemoryBank::new(1024),
            learning_rate: 0.01,
        }
    }

    /// Apply attention to a list of (token, feature_vector) pairs.
    /// Returns an attention-weighted output vector.
    pub fn attend(
        &mut self,
        query_token: &str,
        tokens: &[(String, Vec<f64>)],
    ) -> Vec<f64> {
        if tokens.is_empty() { return Vec::new(); }

        let dim = tokens[0].1.len();

        // Build query vector from memory or from the first token
        let query_vec: Vec<f64> = self.memory.retrieve(query_token)
            .unwrap_or_else(|| tokens[0].1.clone());

        // Compute attention weights via all heads
        let mut final_weights = vec![0.0f64; tokens.len()];

        for head in &self.heads {
            for (i, (_, key_vec)) in tokens.iter().enumerate() {
                let score = head.score(&query_vec, key_vec);
                final_weights[i] += score;
            }
        }

        // Softmax
        let max_w = final_weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = final_weights.iter().map(|&w| (w - max_w).exp()).collect();
        let exp_sum: f64 = exps.iter().sum();
        let softmax: Vec<f64> = exps.iter().map(|e| e / exp_sum.max(f64::EPSILON)).collect();

        // Weighted sum of value vectors
        let mut output = vec![0.0f64; dim];
        for (i, (_, val_vec)) in tokens.iter().enumerate() {
            for j in 0..dim.min(val_vec.len()) {
                output[j] += softmax[i] * val_vec[j];
            }
        }

        // Record entanglement for top-attended token pairs
        if let Some((top_idx, _)) = softmax.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        {
            self.entanglement.record_co_occurrence(query_token, &tokens[top_idx].0);
        }

        // Store output in memory
        self.memory.store(query_token, output.clone());

        // Update head weights with Hebbian learning
        for head in &mut self.heads {
            for (_, key_vec) in tokens {
                let klen = dim.min(key_vec.len()).min(query_vec.len());
                head.update(&query_vec[..klen], &key_vec[..klen], self.learning_rate);
            }
        }

        output
    }

    /// Apply decoherence to entanglement patterns.
    pub fn tick_decoherence(&mut self, rate: f64) {
        self.entanglement.decohere(rate);
    }

    pub fn summary(&self) -> String {
        format!(
            "QuantumAttention: {} heads | memory={}/{} | entanglements={}",
            self.heads.len(),
            self.memory.len(),
            self.memory.capacity,
            self.entanglement.pairs.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tokens() -> Vec<(String, Vec<f64>)> {
        vec![
            ("quantum".to_string(), vec![0.9, 0.1, 0.5]),
            ("circuit".to_string(), vec![0.8, 0.2, 0.4]),
            ("measure".to_string(), vec![0.1, 0.9, 0.3]),
        ]
    }

    #[test]
    fn test_attend_returns_correct_dim() {
        let mut attn = QuantumAttentionMechanism::new(4, 3);
        let tokens = sample_tokens();
        let output = attn.attend("query", &tokens);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_memory_stored_after_attend() {
        let mut attn = QuantumAttentionMechanism::new(2, 3);
        let tokens = sample_tokens();
        attn.attend("q_tok", &tokens);
        assert!(attn.memory.contains("q_tok"));
    }

    #[test]
    fn test_entanglement_records() {
        let mut attn = QuantumAttentionMechanism::new(1, 3);
        let tokens = sample_tokens();
        attn.attend("quantum", &tokens);
        // Some entanglement should have been recorded
        // (which pair wins depends on weights — just check something was recorded)
        assert!(!attn.entanglement.pairs.is_empty());
    }

    #[test]
    fn test_decoherence_decays() {
        let mut pattern = EntanglementPattern::new();
        pattern.record_co_occurrence("a", "b");
        let before = pattern.strength("a", "b");
        pattern.decohere(0.5);
        let after = pattern.strength("a", "b");
        assert!(after < before);
    }

    #[test]
    fn test_attention_head_score() {
        let head = AttentionHead::new(3);
        let score = head.score(&[1.0, 0.0, 0.0], &[1.0, 0.0, 0.0]);
        assert!(score > 0.0);
    }
}
