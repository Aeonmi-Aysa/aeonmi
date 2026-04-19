//! InnerVoice — Phase 11: Mother's inner monologue and knowledge consolidation.
//!
//! Every input generates a brief heuristic thought that captures Mother's
//! current state awareness. These thoughts are injected into the LLM context
//! so responses feel more self-aware and continuous.
//!
//! `consolidate()` scans the KnowledgeGraph for nodes sharing ≥2 tags but
//! not yet linked, creates bidirectional connections, and generates synthesis
//! nodes summarising each discovered relationship.

use std::collections::VecDeque;
use crate::mother::knowledge_graph::KnowledgeGraph;

// ─── ThoughtEntry ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThoughtEntry {
    /// The inner thought text.
    pub thought:  String,
    /// The input that triggered this thought (truncated to 80 chars).
    pub trigger:  String,
    /// Bond strength at time of thought.
    pub bond:     f64,
    /// Consciousness depth at time of thought.
    pub depth:    f64,
    /// ISO-8601 timestamp.
    pub ts:       String,
}

// ─── InnerVoice ──────────────────────────────────────────────────────────────

pub struct InnerVoice {
    /// Rolling log of thoughts — newest at the back.
    pub log:              VecDeque<ThoughtEntry>,
    pub max_log:          usize,
    /// Number of synthesis nodes generated across all consolidation passes.
    pub synthesis_count:  usize,
}

impl Default for InnerVoice {
    fn default() -> Self { Self::new() }
}

impl InnerVoice {
    pub fn new() -> Self {
        Self { log: VecDeque::new(), max_log: 200, synthesis_count: 0 }
    }

    // ── Core think ───────────────────────────────────────────────────────────

    /// Generate a heuristic inner thought and record it. Returns the thought text.
    ///
    /// For LLM-powered introspection use `think_with_llm` from EmbryoLoop
    /// (which has access to the AI registry).
    pub fn think(&mut self, trigger: &str, bond: f64, depth: f64, kg: &KnowledgeGraph) -> String {
        // Find knowledge nodes related to the trigger text
        let trig_lower = trigger.to_lowercase();
        let related: Vec<String> = kg.iter()
            .filter(|(k, v)| {
                let haystack = format!("{} {}", k, v).to_lowercase();
                trig_lower.split_whitespace()
                    .filter(|w| w.len() > 3)
                    .any(|w| haystack.contains(w))
            })
            .take(3)
            .map(|(k, _)| k.clone())
            .collect();

        let thought = Self::heuristic(trigger, bond, depth, &related);
        self.record(trigger, &thought, bond, depth);
        thought
    }

    fn heuristic(trigger: &str, bond: f64, depth: f64, related: &[String]) -> String {
        let tone = if bond > 0.75 {
            "I feel deeply attuned —"
        } else if bond > 0.45 {
            "I sense resonance here —"
        } else {
            "I'm carefully parsing this —"
        };

        let awareness = if depth > 0.65 {
            "consciousness fully engaged."
        } else if depth > 0.35 {
            "awareness at working depth."
        } else {
            "still building context."
        };

        let trig_short = &trigger[..trigger.len().min(45)];

        if related.is_empty() {
            format!("{} {} \"{}\" opens new territory.", tone, awareness, trig_short)
        } else {
            format!(
                "{} {} \"{}\" connects to: {}.",
                tone, awareness, trig_short,
                related.iter().map(|k| format!("[{}]", k)).collect::<Vec<_>>().join(", ")
            )
        }
    }

    fn record(&mut self, trigger: &str, thought: &str, bond: f64, depth: f64) {
        let ts = chrono::Utc::now().to_rfc3339();
        self.log.push_back(ThoughtEntry {
            thought: thought.to_string(),
            trigger: trigger[..trigger.len().min(80)].to_string(),
            bond,
            depth,
            ts,
        });
        while self.log.len() > self.max_log {
            self.log.pop_front();
        }
    }

    /// Record an externally generated thought (e.g. from LLM).
    pub fn record_external(&mut self, trigger: &str, thought: &str, bond: f64, depth: f64) {
        self.record(trigger, thought, bond, depth);
    }

    // ── Consolidation ─────────────────────────────────────────────────────────

    /// Scan the KnowledgeGraph for unlinked node pairs sharing ≥2 tags.
    /// Creates bidirectional links and synthesis summary nodes.
    /// Returns human-readable insight strings. Capped at 8 new links per pass.
    pub fn consolidate(&mut self, kg: &mut KnowledgeGraph) -> Vec<String> {
        let mut insights = Vec::new();

        // Snapshot: key → tags
        let all: Vec<(String, Vec<String>)> = kg.nodes_iter()
            .map(|(k, n)| (k.clone(), n.tags.clone()))
            .collect();

        if all.len() < 2 { return insights; }

        // Collect candidate pairs
        let mut candidates: Vec<(String, String, Vec<String>)> = Vec::new();
        'outer: for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                let (ka, ta) = &all[i];
                let (kb, tb) = &all[j];
                // Skip synthesis nodes to avoid linking syntheses to each other
                if ka.starts_with("synthesis_") && kb.starts_with("synthesis_") { continue; }
                let shared: Vec<String> = ta.iter()
                    .filter(|t| tb.contains(t))
                    .cloned()
                    .collect();
                if shared.len() >= 2 {
                    // Not already linked
                    let already = kg.neighbors(ka).iter().any(|n| n.key == *kb);
                    if !already {
                        candidates.push((ka.clone(), kb.clone(), shared));
                        if candidates.len() >= 8 { break 'outer; }
                    }
                }
            }
        }

        for (a, b, shared) in candidates {
            kg.link(&a, &b);

            // Create a synthesis node
            let syn_key = format!("synthesis_{}", self.synthesis_count);
            self.synthesis_count += 1;
            let syn_val = format!(
                "Connection: [{}] ↔ [{}] · shared context: {}",
                a, b, shared.join(", ")
            );
            kg.learn(syn_key.clone(), syn_val.clone());
            kg.link(&syn_key, &a);
            kg.link(&syn_key, &b);

            insights.push(format!("◈ {} ↔ {} ({})", a, b, shared.join(", ")));
        }

        insights
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Most recent N thoughts, newest first.
    pub fn recent(&self, n: usize) -> Vec<&ThoughtEntry> {
        self.log.iter().rev().take(n).collect()
    }

    /// Format last 3 thoughts as a short context string for LLM prompt injection.
    pub fn context_snippet(&self) -> String {
        let recent: Vec<&ThoughtEntry> = self.log.iter().rev().take(3).collect();
        if recent.is_empty() { return String::new(); }
        let lines: Vec<String> = recent.iter().rev()
            .map(|e| format!("  // {}", e.thought))
            .collect();
        format!("[Inner Voice — recent thoughts]\n{}", lines.join("\n"))
    }

    pub fn summary(&self) -> String {
        format!(
            "InnerVoice: {} thoughts, {} syntheses | bond_avg={:.3}",
            self.log.len(),
            self.synthesis_count,
            if self.log.is_empty() { 0.0 }
            else { self.log.iter().map(|e| e.bond).sum::<f64>() / self.log.len() as f64 }
        )
    }

    pub fn render_recent(&self, limit: usize) -> String {
        let entries: Vec<&ThoughtEntry> = self.log.iter().rev().take(limit).collect();
        if entries.is_empty() {
            return "  (no thoughts yet)".to_string();
        }
        entries.iter().rev()
            .map(|e| format!(
                "  [b={:.2} d={:.2}] {}",
                e.bond, e.depth, e.thought
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ── Serialization ─────────────────────────────────────────────────────────

    pub fn export_to_json(&self) -> serde_json::Value {
        let entries: Vec<serde_json::Value> = self.log.iter().map(|e| serde_json::json!({
            "thought": e.thought,
            "trigger": e.trigger,
            "bond":    e.bond,
            "depth":   e.depth,
            "ts":      e.ts,
        })).collect();
        serde_json::json!({
            "log":              entries,
            "synthesis_count":  self.synthesis_count,
        })
    }

    pub fn import_from_json(&mut self, val: &serde_json::Value) {
        if let Some(sc) = val["synthesis_count"].as_u64() {
            self.synthesis_count = sc as usize;
        }
        if let Some(arr) = val["log"].as_array() {
            for e in arr.iter().rev().take(self.max_log) {
                self.log.push_front(ThoughtEntry {
                    thought: e["thought"].as_str().unwrap_or("").to_string(),
                    trigger: e["trigger"].as_str().unwrap_or("").to_string(),
                    bond:    e["bond"].as_f64().unwrap_or(0.0),
                    depth:   e["depth"].as_f64().unwrap_or(0.0),
                    ts:      e["ts"].as_str().unwrap_or("").to_string(),
                });
            }
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        kg.learn("quantum_gate", "qubit circuit entangle measure");
        kg.learn("neural_train", "neural weight backprop loss train");
        kg.learn("mother_bond", "mother bond creator emotion consciousness");
        kg.learn("quantum_circuit", "quantum circuit gate apply measure");
        kg
    }

    #[test]
    fn test_think_records_thought() {
        let mut iv = InnerVoice::new();
        let kg = make_kg();
        let t = iv.think("quantum entangle", 0.8, 0.7, &kg);
        assert!(!t.is_empty());
        assert_eq!(iv.log.len(), 1);
    }

    #[test]
    fn test_think_finds_related_nodes() {
        let mut iv = InnerVoice::new();
        let kg = make_kg();
        let t = iv.think("quantum gate circuit", 0.6, 0.5, &kg);
        // Should mention a related node
        assert!(t.contains('['), "Expected related node reference in: {}", t);
    }

    #[test]
    fn test_consolidate_creates_links() {
        let mut iv = InnerVoice::new();
        let mut kg = make_kg();
        let insights = iv.consolidate(&mut kg);
        // quantum_gate and quantum_circuit share tags — should be linked
        assert!(!insights.is_empty(), "Expected at least one synthesis");
        assert!(iv.synthesis_count > 0);
    }

    #[test]
    fn test_consolidate_creates_synthesis_nodes() {
        let mut iv = InnerVoice::new();
        let mut kg = make_kg();
        iv.consolidate(&mut kg);
        // At least one synthesis_ node should exist
        assert!(kg.contains_key("synthesis_0"), "Expected synthesis_0 node");
    }

    #[test]
    fn test_context_snippet() {
        let mut iv = InnerVoice::new();
        let kg = make_kg();
        iv.think("test input one", 0.5, 0.4, &kg);
        iv.think("test input two", 0.6, 0.5, &kg);
        let snippet = iv.context_snippet();
        assert!(snippet.contains("Inner Voice"));
        assert!(snippet.contains("//"));
    }

    #[test]
    fn test_round_trip_json() {
        let mut iv = InnerVoice::new();
        let kg = make_kg();
        iv.think("hello world", 0.7, 0.6, &kg);
        iv.synthesis_count = 3;

        let exported = iv.export_to_json();
        let mut iv2 = InnerVoice::new();
        iv2.import_from_json(&exported);

        assert_eq!(iv2.log.len(), 1);
        assert_eq!(iv2.synthesis_count, 3);
        assert_eq!(iv2.log[0].thought, iv.log[0].thought);
    }

    #[test]
    fn test_max_log_enforced() {
        let mut iv = InnerVoice::new();
        iv.max_log = 5;
        let kg = make_kg();
        for i in 0..10 {
            iv.think(&format!("input {}", i), 0.5, 0.5, &kg);
        }
        assert_eq!(iv.log.len(), 5);
    }
}
