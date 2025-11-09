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

pub use execution::{ExecutionContext, ExecutionLimits, ExecutionManager, ExecutionResult};
pub use filesystem::{FileSystemGuard, SandboxError, SandboxedFileSystem};
pub use process_manager::{ProcessConfig, ProcessError, ProcessHandle, ProcessManager};
pub use workspace::{AeonmiWorkspace, WorkspaceConfig, WorkspaceError};

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Default execution timeout for programs
pub const DEFAULT_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default memory limit (in MB)
pub const DEFAULT_MEMORY_LIMIT: usize = 128;

/// Maximum number of processes that can run concurrently
pub const MAX_CONCURRENT_PROCESSES: usize = 4;

/// Initialize a new sandboxed workspace
pub fn create_workspace(
    project_path: PathBuf,
    config: Option<WorkspaceConfig>,
) -> Result<AeonmiWorkspace> {
    let config = config.unwrap_or_default();
    AeonmiWorkspace::new(project_path, config)
}

/// Create an execution context with default limits
pub fn create_execution_context(workspace: &AeonmiWorkspace) -> ExecutionContext {
    ExecutionContext::new(
        workspace.clone(),
        ExecutionLimits {
            timeout: DEFAULT_EXECUTION_TIMEOUT,
            memory_limit_mb: DEFAULT_MEMORY_LIMIT,
            max_processes: MAX_CONCURRENT_PROCESSES,
            allow_network: false,
            allow_file_write: true,
            allowed_commands: vec!["python".to_string(), "qiskit".to_string()],
            allowed_env_vars: vec!["PATH".to_string(), "HOME".to_string()],
            blocked_env_vars: vec!["SECRET".to_string()],
        },
    )
}

/// Validate that a path is within the sandbox
pub fn validate_sandbox_path(
    workspace_root: &std::path::Path,
    target_path: &std::path::Path,
) -> Result<()> {
    let canonical_workspace = workspace_root.canonicalize()?;
    let canonical_target = target_path.canonicalize().or_else(|_| {
        // If the target doesn't exist, try to canonicalize its parent
        if let Some(parent) = target_path.parent() {
            let canonical_parent = parent.canonicalize()?;
            Ok(canonical_parent.join(target_path.file_name().unwrap_or_default()))
        } else {
            anyhow::bail!("Invalid path: {}", target_path.display())
        }
    })?;

    if !canonical_target.starts_with(&canonical_workspace) {
        anyhow::bail!(
            "Path '{}' is outside the workspace '{}'",
            target_path.display(),
            workspace_root.display()
        );
    }

    Ok(())
}

/// Clean up temporary files and artifacts
pub fn cleanup_workspace(workspace: &AeonmiWorkspace) -> Result<()> {
    workspace.cleanup_artifacts()
}
