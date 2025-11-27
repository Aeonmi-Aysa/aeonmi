//! Sandbox & Execution Model for Aeonmi
//!
//! This module provides a secure and organized workspace for Aeonmi projects,
//! including file system isolation, execution safety, and artifact management.

pub mod execution;
pub mod filesystem;
pub mod process_manager;
pub mod workspace;

#[cfg(test)]
mod tests;

pub use execution::ExecutionLimits;
#[allow(unused_imports)]
pub use filesystem::SandboxedFileSystem;
pub use process_manager::{ProcessConfig, ProcessManager};
pub use workspace::{AeonmiWorkspace, WorkspaceConfig};

use anyhow::Result;
use std::path::PathBuf;

/// Initialize a new sandboxed workspace
pub fn create_workspace(
    project_path: PathBuf,
    config: Option<WorkspaceConfig>,
) -> Result<AeonmiWorkspace> {
    let config = config.unwrap_or_default();
    AeonmiWorkspace::new(project_path, config)
}
