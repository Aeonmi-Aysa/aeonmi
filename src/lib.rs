#![allow(dead_code)] // Large experimental surface; many analysis/AI/quantum scaffolds not yet invoked.
                     // Make the same modules available from the library crate so anything under
                     // src/tui/* (compiled as part of lib) can reach them via `crate::...`.
pub mod cli;
pub mod cli_vault;
pub mod commands;
pub mod compiler;
pub mod config;
pub mod core;
pub mod encryption;
pub mod integration; // Unified system integration layer
pub mod io;
pub mod project;
pub mod quantum_ast_tests;
pub mod runtime;
pub mod shell;
pub mod tui;
pub mod vault; // Quantum AST integration tests
               // GUI components live in a separate workspace app, so avoid declaring a
               // missing Rust module here (rustfmt will try to resolve the path).

// Re-export tauri bridge commands if the path exists (using conditional to avoid compile fail if not included)
#[allow(unused_imports)]
pub use crate::commands::*; // existing commands
