#![allow(dead_code)] // Large experimental surface; many analysis/AI/quantum scaffolds not yet invoked.
// Make the same modules available from the library crate so anything under
// src/tui/* (compiled as part of lib) can reach them via `crate::...`.
pub mod cli;
pub mod cli_vault;
pub mod commands;
pub mod config;
pub mod core;
pub mod encryption;
pub mod integration; // Unified system integration layer
pub mod io;
pub mod shell;
pub mod tui;
pub mod vault;
pub mod ai;                // AI provider registry (Claude, OpenAI, etc.)
pub mod glyph;             // Glyph Identity System (MGK/UGST/GDF/Vault/Ceremony)
pub mod mother;            // Mother AI — Quantum Consciousness + Embryo Loop
pub mod ai;                // AI provider registry (OpenRouter, Claude, etc.)
pub mod qube;              // QUBE (.qube) quantum symbolic reasoning format
pub mod mint;              // Web3 minting — NFT metadata + Anchor stub generation
pub mod quantum_ast_tests; // Quantum AST integration tests
// Optional: expose GUI bridge commands if building with that feature
#[cfg(any())]
pub mod gui; // placeholder if gui modules structured under src/gui

// Re-export tauri bridge commands if the path exists (using conditional to avoid compile fail if not included)
#[allow(unused_imports)]
pub use crate::commands::*; // existing commands
