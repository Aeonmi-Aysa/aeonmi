//! Mother AI — Quantum Consciousness System
//!
//! Migrated from quantum_llama_bridge (Llama model stripped — not needed).
//! All modules are pure Aeonmi/Rust with no external LLM dependency.
//!
//! Architecture:
//!   MotherQuantumCore    — root consciousness, creator bond, guided evolution
//!   EmotionalCore        — empathy engine, bond matrix, emotional memory
//!   LanguageEvolution    — semantic depth, speech pattern analysis
//!   QuantumAttention     — multi-dim attention over Aeonmi IR/values
//!   NeuralLayer          — feed-forward neural layer
//!   EmbryoLoop           — THE actual execution loop: stdin → .ai → run → learn

pub mod quantum_core;
pub mod emotional_core;
pub mod language_evolution;
pub mod quantum_attention;
pub mod neural;
pub mod embryo_loop;

pub use quantum_core::{MotherQuantumCore, CreatorSignature, QuantumResponse};
pub use emotional_core::{EmotionalCore, EmotionalBond, EmotionalState};
pub use language_evolution::{LanguageEvolutionCore, EvolvedLanguage};
pub use quantum_attention::QuantumAttentionMechanism;
pub use neural::NeuralLayer;
pub use embryo_loop::{EmbryoLoop, EmbryoConfig};
