pub mod ast;
pub mod compile;
pub mod edit;
pub mod enhanced; // Enhanced command handlers
pub mod format;
pub mod fs;
pub mod lint;
pub mod project;
pub mod project_init; // Project initialization commands
pub mod repl;
pub mod run;
pub mod tokens;
pub mod vault;
pub mod vm;

#[cfg(feature = "quantum")]
pub mod quantum;
