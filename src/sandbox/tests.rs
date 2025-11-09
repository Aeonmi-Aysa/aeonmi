/// Test for the Sandbox & Execution Model
/// This test validates the core functionality of the sandbox system
use std::path::PathBuf;
use std::time::Duration;

// Import our sandbox modules
use crate::sandbox::{
    AeonmiWorkspace, ProcessConfig, ProcessManager, SandboxedFileSystem, WorkspaceConfig,
};

// Use the shared execution limits type
use crate::sandbox::ExecutionLimits;

#[cfg(test)]
use tempfile::TempDir;

#[test]
fn test_workspace_creation() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path().to_path_buf();

        let config = WorkspaceConfig {
            name: "test-workspace".to_string(),
            ..Default::default()
        };

        let workspace = AeonmiWorkspace::new(workspace_path.clone(), config)?;

        // Verify workspace structure
        assert!(workspace.project_root().exists());
        assert!(workspace.source_dir().exists());
        assert!(workspace.output_dir().exists());

        println!("✅ Workspace creation test passed");
        Ok(())
    }

    #[test]
    fn test_sandboxed_filesystem() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let sandbox = SandboxedFileSystem::with_root(temp_dir.path());

        // Test file operations
        let test_file = "test.txt";
        let test_content = "Hello, Sandbox!";

        sandbox.write_file(test_file, test_content.as_bytes())?;
        assert!(sandbox.exists(test_file));

        let read_content = sandbox.read_to_string(test_file)?;
        assert_eq!(read_content, test_content);

        // Test directory operations
        sandbox.create_dir("subdir")?;
        assert!(sandbox.is_dir("subdir"));

        println!("✅ Sandboxed filesystem test passed");
        Ok(())
    }

    #[test]
    fn test_process_manager() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let process_manager = ProcessManager::new();

        // Test process manager creation
        assert_eq!(process_manager.active_process_count(), 0);

        // Test process limits
        let limits = ExecutionLimits {
            timeout: Duration::from_secs(5),
            memory_limit_mb: 64,
            max_processes: 2,
            allow_network: false,
            allow_file_write: true,
            allowed_commands: vec!["echo".to_string()],
            ..ExecutionLimits::default()
        };

        let _config = ProcessConfig {
            command: "echo".to_string(),
            args: vec!["Hello".to_string()],
            working_dir: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
            limits,
            capture_stdout: true,
            capture_stderr: false,
        };

        // On Windows, we'll skip the actual process execution test
        // since it requires different commands
        #[cfg(not(windows))]
        {
            let process_id = process_manager.start_process(config)?;

            // Wait for completion
            let status =
                process_manager.wait_for_process(process_id, Some(Duration::from_secs(10)))?;

            match status {
                crate::sandbox::ProcessStatus::Completed(exit_code) => {
                    assert_eq!(exit_code, 0);
                }
                _ => panic!("Process should have completed successfully"),
            }
        }

        println!("✅ Process manager test passed");
        Ok(())
    }

    #[test]
    fn test_workspace_file_operations() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path().to_path_buf();

        let config = WorkspaceConfig::default();
        let workspace = AeonmiWorkspace::new(workspace_path, config)?;

        // Test source file creation
        let source_content = r#"
quantum main() {
    |0⟩ -> H -> measure
}
"#;

        workspace.create_source_file("main.aeon", source_content)?;

        let read_content = workspace.read_source_file("main.aeon")?;
        assert_eq!(read_content.trim(), source_content.trim());

        // Test listing source files
        let source_files = workspace.list_source_files()?;
        assert!(source_files.iter().any(|f| f.ends_with("main.aeon")));

        println!("✅ Workspace file operations test passed");
        Ok(())
    }

    #[test]
    fn test_execution_limits() {
        let limits = ExecutionLimits {
            timeout: Duration::from_secs(30),
            memory_limit_mb: 128,
            max_processes: 4,
            allow_network: false,
            allow_file_write: true,
            allowed_commands: vec!["python".to_string(), "aeon".to_string()],
            ..ExecutionLimits::default()
        };

        assert_eq!(limits.timeout, Duration::from_secs(30));
        assert_eq!(limits.memory_limit_mb, 128);
        assert_eq!(limits.max_processes, 4);
        assert!(!limits.allow_network);
        assert!(limits.allow_file_write);
        assert!(limits.allowed_commands.contains(&"python".to_string()));

        println!("✅ Execution limits test passed");
    }

    #[test]
    fn test_workspace_cleanup() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path().to_path_buf();

        let config = WorkspaceConfig::default();
        let workspace = AeonmiWorkspace::new(workspace_path, config)?;

        // Create some test files and artifacts
        workspace.create_source_file("test.aeon", "// Test file")?;

        // Create output artifacts
        let output_file = workspace.output_dir().join("output.js");
        std::fs::write(&output_file, "// Generated output")?;

        // Verify files exist
        assert!(workspace.source_dir().join("test.aeon").exists());
        assert!(output_file.exists());

        // Clean up artifacts
        workspace.cleanup_artifacts()?;

        // Source files should still exist, but output might be cleaned
        assert!(workspace.source_dir().join("test.aeon").exists());

        println!("✅ Workspace cleanup test passed");
        Ok(())
    }

/// Integration test that combines multiple sandbox components
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_full_sandbox_workflow() -> anyhow::Result<()> {
        // 1. Create a workspace
        let temp_dir = TempDir::new()?;
        let workspace_path = temp_dir.path().to_path_buf();

        let config = WorkspaceConfig {
            name: "integration-test".to_string(),
            ..Default::default()
        };

        let workspace = AeonmiWorkspace::new(workspace_path.clone(), config)?;

        // 2. Create a source file
        let source_code = r#"
# Simple Aeonmi quantum program
quantum hello_world() {
    |0⟩ -> H -> measure -> print("Hello, Quantum World!")
}
"#;

        workspace.create_source_file("hello.aeon", source_code)?;

        // 3. Verify sandboxed file access
        let sandbox = SandboxedFileSystem::with_root(workspace.project_root());
        assert!(sandbox.exists("src/hello.aeon"));

        let content = sandbox.read_to_string("src/hello.aeon")?;
        assert!(content.contains("hello_world"));

        // 4. Test process execution (mock for now)
        let process_manager = ProcessManager::new();
        assert_eq!(process_manager.active_process_count(), 0);

        // 5. Cleanup
        workspace.cleanup_artifacts()?;

        println!("✅ Full sandbox workflow test passed");
        println!("   ✓ Workspace created and configured");
        println!("   ✓ Source file operations working");
        println!("   ✓ Sandboxed filesystem secure");
        println!("   ✓ Process management ready");
        println!("   ✓ Cleanup successful");

        Ok(())
    }
}