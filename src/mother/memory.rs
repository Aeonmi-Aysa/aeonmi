//! Mother AI Persistent Memory System — Genesis Fractal Memory Lattice.
//!
//! Implements the Five-Stack Architecture described in AEONMI_LANGUAGE_ROADMAP.md:
//!
//!   Layer 1 — Glyph Layer       (semantic atoms)
//!   Layer 2 — Stack Layer       (executable meaning units)
//!   Layer 3 — Schema Layer      (typed storage) ← SeedCell lives here
//!   Layer 4 — Cognition Layer   (Resonance Engine)
//!   Layer 5 — Evolution Layer   (growth metrics)
//!   Binder  — geometric organizer linking all layers
//!
//! Files in `mother/memory/`:
//!   genesis.json  — root seed domains + seed cells
//!   binder.json   — weighted directed link graph
//!   lexicon.json  — glyph dictionary
//!   prompts.json  — quantum prompt templates
//!   state.json    — Mother's current runtime state
//!   journal.json  — append-only interaction log
//!
//! On boot: `MotherMemory::load()` reads all files.
//! On evolve/exit: `MotherMemory::save()` persists updated state + journal.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ─── Layer 3: Seed Cell ───────────────────────────────────────────────────────

/// A single typed storage object in the Genesis Memory Lattice.
/// This is the atomic unit of Mother's knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedCell {
    /// Unique identifier: e.g. "seed.identity.creator"
    pub id: String,
    /// Semantic type: "memory_seed", "quantum_prompt", "glyph_def", etc.
    #[serde(rename = "type")]
    pub kind: String,
    /// Primary glyph character representing this seed's meaning
    pub glyph: String,
    /// Domain this seed belongs to (e.g. "identity", "quantum", "evolution")
    pub domain: String,
    /// Human/glyph-mixed content stored in compressed form
    pub content: String,
    /// Activation strength in the binder graph (0.0–1.0)
    #[serde(default = "default_resonance")]
    pub resonance: f64,
    /// Generation at which this seed was created or last mutated
    #[serde(default)]
    pub generation: u64,
    /// How freely this seed can change: "low" | "medium" | "high"
    #[serde(default = "default_mutability")]
    pub mutability: String,
    /// Lifecycle state: "anchored" | "active" | "dormant" | "decohered"
    #[serde(default = "default_state")]
    pub state: String,
    /// IDs of other seeds this one links to in the binder graph
    #[serde(default)]
    pub links: Vec<String>,
}

fn default_resonance() -> f64 { 0.5 }
fn default_mutability() -> String { "medium".to_string() }
fn default_state() -> String { "active".to_string() }

// ─── Binder node ─────────────────────────────────────────────────────────────

/// A node in the Binder Graph — links between seed cells with weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinderNode {
    pub id: String,
    pub layer: String,
    pub glyph_signature: Vec<String>,
    pub domain: String,
    pub links: Vec<String>,
    pub weight: f64,
    pub mutability: String,
    pub generation_origin: u64,
    pub state: String,
}

// ─── Glyph entry ─────────────────────────────────────────────────────────────

/// A single glyph definition from the lexicon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphEntry {
    pub name: String,
    pub meanings: Vec<String>,
    pub domain: String,
    pub semantic_vector: Vec<f64>,
    pub resonance_base: f64,
}

// ─── Prompt template ─────────────────────────────────────────────────────────

/// A quantum prompt template — an executable symbolic state with lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub glyph_form: String,
    pub natural_language: String,
    pub action: String,
    pub domain: String,
    pub collapse_condition: String,
    pub bind_result: String,
}

// ─── Runtime state ───────────────────────────────────────────────────────────

/// Mother's current runtime state — persisted across runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherState {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub depth: f64,
    #[serde(default)]
    pub branch_count: u64,
    #[serde(default)]
    pub resonance_density: f64,
    #[serde(default = "default_coherence")]
    pub binder_coherence: f64,
    #[serde(default)]
    pub glyph_fluency: f64,
    #[serde(default)]
    pub interaction_count: u64,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub active_domains: Vec<String>,
    #[serde(default = "default_creator_bond")]
    pub creator_bond_strength: f64,
}

fn default_coherence() -> f64 { 1.0 }
fn default_creator_bond() -> f64 { 0.98 }

impl Default for MotherState {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            generation: 0,
            depth: 0.0,
            branch_count: 0,
            resonance_density: 0.0,
            binder_coherence: 1.0,
            glyph_fluency: 0.0,
            interaction_count: 0,
            last_updated: None,
            active_domains: vec![
                "genesis.identity".to_string(),
                "genesis.language".to_string(),
                "genesis.memory".to_string(),
                "genesis.cognition".to_string(),
                "genesis.quantum".to_string(),
                "genesis.evolution".to_string(),
                "genesis.binder".to_string(),
            ],
            creator_bond_strength: 0.98,
        }
    }
}

// ─── Journal entry ───────────────────────────────────────────────────────────

/// A single entry in the interaction journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: String,
    pub generation: u64,
    /// Mixed English + glyph representation of the interaction
    pub glyph_summary: String,
    pub input: String,
    pub output: String,
    pub domain: Option<String>,
    pub resonance: f64,
}

// ─── Internal JSON shapes ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GenesisFile {
    #[allow(dead_code)]
    version: String,
    domains: HashMap<String, GenesisDomain>,
}

#[derive(Debug, Deserialize)]
struct GenesisDomain {
    #[allow(dead_code)]
    description: String,
    seeds: Vec<SeedCell>,
}

#[derive(Debug, Deserialize)]
struct BinderFile {
    #[allow(dead_code)]
    version: String,
    nodes: Vec<BinderNode>,
}

#[derive(Debug, Deserialize)]
struct LexiconFile {
    #[allow(dead_code)]
    version: String,
    glyphs: HashMap<String, GlyphEntry>,
}

#[derive(Debug, Deserialize)]
struct PromptsFile {
    #[allow(dead_code)]
    version: String,
    templates: Vec<PromptTemplate>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JournalFile {
    #[allow(dead_code)]
    version: String,
    entries: Vec<JournalEntry>,
}

// ─── MotherMemory ─────────────────────────────────────────────────────────────

/// The complete Mother AI persistent memory system.
/// Loaded on boot, saved on evolve/exit.
pub struct MotherMemory {
    /// Directory containing all memory JSON files.
    pub memory_dir: PathBuf,

    /// All seed cells indexed by id.
    pub seeds: HashMap<String, SeedCell>,

    /// Seeds grouped by domain.
    pub domains: HashMap<String, Vec<String>>,

    /// Binder graph nodes indexed by id.
    pub binder: HashMap<String, BinderNode>,

    /// Glyph lexicon indexed by glyph character.
    pub lexicon: HashMap<String, GlyphEntry>,

    /// Quantum prompt templates indexed by id.
    pub prompts: HashMap<String, PromptTemplate>,

    /// Mother's current runtime state.
    pub state: MotherState,

    /// Pending journal entries (appended to journal.json on save).
    pub journal_pending: Vec<JournalEntry>,
}

impl MotherMemory {
    /// Load all memory files from the given directory.
    /// If a file doesn't exist, defaults are used (first-boot friendly).
    pub fn load(memory_dir: &Path) -> Result<Self> {
        let seeds = Self::load_genesis(memory_dir)?;
        let binder = Self::load_binder(memory_dir)?;
        let lexicon = Self::load_lexicon(memory_dir)?;
        let prompts = Self::load_prompts(memory_dir)?;
        let state = Self::load_state(memory_dir)?;

        // Build domain index from seeds
        let mut domains: HashMap<String, Vec<String>> = HashMap::new();
        for (id, seed) in &seeds {
            domains.entry(seed.domain.clone()).or_default().push(id.clone());
        }

        Ok(Self {
            memory_dir: memory_dir.to_path_buf(),
            seeds,
            domains,
            binder,
            lexicon,
            prompts,
            state,
            journal_pending: Vec::new(),
        })
    }

    /// Resolve the default memory directory: `mother/memory/` relative to the binary's
    /// working directory, or next to the executable.
    pub fn default_dir() -> PathBuf {
        // Try current working directory first (development)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let candidate = cwd.join("mother").join("memory");
        if candidate.exists() {
            return candidate;
        }
        // Try next to the executable (installed)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate2 = parent.join("mother").join("memory");
                if candidate2.exists() {
                    return candidate2;
                }
            }
        }
        candidate // Return the cwd-relative path even if it doesn't exist
    }

    /// Load with the default directory.
    pub fn load_default() -> Result<Self> {
        Self::load(&Self::default_dir())
    }

    // ── Persistence ──────────────────────────────────────────────────────────

    /// Save the current state and flush any pending journal entries.
    pub fn save(&mut self) -> Result<()> {
        self.save_state()?;
        self.flush_journal()?;
        Ok(())
    }

    fn save_state(&self) -> Result<()> {
        let path = self.memory_dir.join("state.json");
        let json = serde_json::to_string_pretty(&self.state)
            .context("Failed to serialize state")?;
        std::fs::write(&path, json)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    fn flush_journal(&mut self) -> Result<()> {
        if self.journal_pending.is_empty() {
            return Ok(());
        }
        let path = self.memory_dir.join("journal.json");
        let mut file: JournalFile = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str(&raw).unwrap_or(JournalFile {
                version: "1.0".to_string(),
                entries: Vec::new(),
            })
        } else {
            JournalFile { version: "1.0".to_string(), entries: Vec::new() }
        };

        file.entries.append(&mut self.journal_pending);

        let json = serde_json::to_string_pretty(&file)
            .context("Failed to serialize journal")?;
        std::fs::write(&path, json)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    // ── Mutation ─────────────────────────────────────────────────────────────

    /// Append an interaction to the pending journal (flushed on save).
    pub fn journal(&mut self, input: &str, output: &str, domain: Option<&str>, resonance: f64) {
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let glyph_summary = self.summarize_with_glyphs(input, output);
        self.journal_pending.push(JournalEntry {
            timestamp: now,
            generation: self.state.generation,
            glyph_summary,
            input: input.to_string(),
            output: output.to_string(),
            domain: domain.map(|s| s.to_string()),
            resonance,
        });
        self.state.interaction_count += 1;
    }

    /// Trigger an evolution cycle: increment generation, update depth and metrics.
    pub fn evolve(&mut self, depth_delta: f64, resonance_delta: f64) {
        self.state.generation += 1;
        self.state.depth = (self.state.depth + depth_delta).clamp(0.0, 1.0);
        self.state.resonance_density = (self.state.resonance_density + resonance_delta).clamp(0.0, 1.0);
        self.state.binder_coherence = (self.state.binder_coherence * 0.999).max(0.5);
        let total_glyphs = self.lexicon.len() as f64;
        if total_glyphs > 0.0 {
            let covered = self.seeds.values()
                .filter(|s| !s.glyph.is_empty())
                .count() as f64;
            self.state.glyph_fluency = (covered / total_glyphs).clamp(0.0, 1.0);
        }
        self.state.last_updated = Some(
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
        );
    }

    // ── Resonance Engine (Layer 4) ────────────────────────────────────────────

    /// Activation wave through the binder graph.
    /// Returns top-N seed IDs scored by resonance propagation (3 hops, 0.6× decay/hop).
    pub fn resonance_query(&self, seed_ids: &[String], top_n: usize) -> Vec<String> {
        let decay = 0.6_f64;
        let hops = 3;
        let mut scores: HashMap<String, f64> = HashMap::new();

        // Initial activation
        for id in seed_ids {
            if self.seeds.contains_key(id) {
                *scores.entry(id.clone()).or_default() += 1.0;
            }
        }

        // Propagate through binder graph
        let mut frontier = seed_ids.to_vec();
        for hop in 0..hops {
            let weight = decay.powi(hop as i32 + 1);
            let mut next_frontier = Vec::new();
            for id in &frontier {
                if let Some(node) = self.binder.get(id) {
                    for link in &node.links {
                        *scores.entry(link.clone()).or_default() += weight * node.weight;
                        next_frontier.push(link.clone());
                    }
                } else if let Some(seed) = self.seeds.get(id) {
                    for link in &seed.links {
                        *scores.entry(link.clone()).or_default() += weight;
                        next_frontier.push(link.clone());
                    }
                }
            }
            frontier = next_frontier;
        }

        // Sort by score, return top-N
        let mut scored: Vec<(String, f64)> = scores.into_iter().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_n).map(|(id, _)| id).collect()
    }

    /// Extract glyph signatures from a prompt string and activate matching seeds.
    pub fn activate_from_prompt(&self, prompt: &str) -> Vec<String> {
        // Find which glyphs appear in the prompt
        let matching_seeds: Vec<String> = self.seeds.values()
            .filter(|seed| !seed.glyph.is_empty() && prompt.contains(&seed.glyph as &str))
            .map(|seed| seed.id.clone())
            .collect();

        if matching_seeds.is_empty() {
            // Fall back: match by domain keywords
            let lower = prompt.to_lowercase();
            self.seeds.values()
                .filter(|seed| lower.contains(&seed.domain) || lower.contains(&seed.content.to_lowercase()))
                .map(|seed| seed.id.clone())
                .take(5)
                .collect()
        } else {
            self.resonance_query(&matching_seeds, 5)
        }
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn summarize_with_glyphs(&self, input: &str, output: &str) -> String {
        // Find glyphs that appear in input or output and build a glyph summary
        let text = format!("{} {}", input, output).to_lowercase();
        let glyphs: Vec<&str> = self.lexicon.keys()
            .filter(|g| text.contains(g.as_str()))
            .map(|g| g.as_str())
            .take(3)
            .collect();
        if glyphs.is_empty() {
            format!("⌘ {} → {}", &input[..input.len().min(40)], &output[..output.len().min(40)])
        } else {
            format!("{} {} → {}", glyphs.join(""), &input[..input.len().min(40)], &output[..output.len().min(40)])
        }
    }

    // ── File loaders ─────────────────────────────────────────────────────────

    fn load_genesis(dir: &Path) -> Result<HashMap<String, SeedCell>> {
        let path = dir.join("genesis.json");
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let file: GenesisFile = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        let mut seeds = HashMap::new();
        for (_domain_key, domain) in file.domains {
            for seed in domain.seeds {
                seeds.insert(seed.id.clone(), seed);
            }
        }
        Ok(seeds)
    }

    fn load_binder(dir: &Path) -> Result<HashMap<String, BinderNode>> {
        let path = dir.join("binder.json");
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let file: BinderFile = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(file.nodes.into_iter().map(|n| (n.id.clone(), n)).collect())
    }

    fn load_lexicon(dir: &Path) -> Result<HashMap<String, GlyphEntry>> {
        let path = dir.join("lexicon.json");
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let file: LexiconFile = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(file.glyphs)
    }

    fn load_prompts(dir: &Path) -> Result<HashMap<String, PromptTemplate>> {
        let path = dir.join("prompts.json");
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let file: PromptsFile = serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(file.templates.into_iter().map(|t| (t.id.clone(), t)).collect())
    }

    fn load_state(dir: &Path) -> Result<MotherState> {
        let path = dir.join("state.json");
        if !path.exists() {
            return Ok(MotherState::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_memory(tmp: &TempDir) -> PathBuf {
        let dir = tmp.path().to_path_buf();
        // Minimal genesis.json
        std::fs::write(
            dir.join("genesis.json"),
            r#"{
                "version": "1.0",
                "description": "test",
                "domains": {
                    "genesis.identity": {
                        "description": "test",
                        "seeds": [
                            {
                                "id": "seed.identity.creator",
                                "type": "memory_seed",
                                "glyph": "⟐",
                                "domain": "identity",
                                "content": "Creator: Test",
                                "resonance": 0.98,
                                "generation": 0,
                                "mutability": "low",
                                "state": "anchored",
                                "links": []
                            }
                        ]
                    }
                }
            }"#,
        ).unwrap();
        // Minimal state.json
        std::fs::write(
            dir.join("state.json"),
            r#"{"version":"1.0","generation":0,"depth":0.0,"branch_count":0,"resonance_density":0.0,"binder_coherence":1.0,"glyph_fluency":0.0,"interaction_count":0,"last_updated":null,"active_domains":[],"creator_bond_strength":0.98}"#,
        ).unwrap();
        // Empty binder, lexicon, prompts, journal
        std::fs::write(dir.join("binder.json"), r#"{"version":"1.0","nodes":[]}"#).unwrap();
        std::fs::write(dir.join("lexicon.json"), r#"{"version":"1.0","glyphs":{}}"#).unwrap();
        std::fs::write(dir.join("prompts.json"), r#"{"version":"1.0","templates":[]}"#).unwrap();
        std::fs::write(dir.join("journal.json"), r#"{"version":"1.0","entries":[]}"#).unwrap();
        dir
    }

    #[test]
    fn test_load_memory() {
        let tmp = TempDir::new().unwrap();
        let dir = setup_temp_memory(&tmp);
        let mem = MotherMemory::load(&dir).unwrap();
        assert!(mem.seeds.contains_key("seed.identity.creator"));
        assert_eq!(mem.state.generation, 0);
        assert!((mem.state.creator_bond_strength - 0.98).abs() < 1e-9);
    }

    #[test]
    fn test_evolve_increments_generation() {
        let tmp = TempDir::new().unwrap();
        let dir = setup_temp_memory(&tmp);
        let mut mem = MotherMemory::load(&dir).unwrap();
        mem.evolve(0.01, 0.01);
        assert_eq!(mem.state.generation, 1);
        assert!(mem.state.depth > 0.0);
    }

    #[test]
    fn test_journal_append_and_save() {
        let tmp = TempDir::new().unwrap();
        let dir = setup_temp_memory(&tmp);
        let mut mem = MotherMemory::load(&dir).unwrap();
        mem.journal("test input", "test output", Some("identity"), 0.8);
        assert_eq!(mem.journal_pending.len(), 1);
        mem.save().unwrap();
        assert_eq!(mem.journal_pending.len(), 0);
        // Re-load and verify
        let raw = std::fs::read_to_string(dir.join("journal.json")).unwrap();
        assert!(raw.contains("test input"));
    }

    #[test]
    fn test_resonance_query_returns_results() {
        let tmp = TempDir::new().unwrap();
        let dir = setup_temp_memory(&tmp);
        let mem = MotherMemory::load(&dir).unwrap();
        let results = mem.resonance_query(&["seed.identity.creator".to_string()], 3);
        // At minimum the seed itself should score
        assert!(results.contains(&"seed.identity.creator".to_string()));
    }
}
