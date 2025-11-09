use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Configuration for an Aeonmi workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Project name
    pub name: String,
    /// Project description
    pub description: Option<String>,
    /// Language version
    pub aeonmi_version: String,
    /// Target platform(s)
    pub targets: Vec<String>,
    /// Dependencies
    pub dependencies: HashMap<String, String>,
    /// Maximum artifact retention
    pub max_artifacts: usize,
    /// Auto-cleanup enabled
    pub auto_cleanup: bool,
    /// Allowed external commands
    pub allowed_commands: Vec<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: "unnamed-project".to_string(),
            description: None,
            aeonmi_version: env!("CARGO_PKG_VERSION").to_string(),
            targets: vec!["quantum".to_string(), "classical".to_string()],
            dependencies: HashMap::new(),
            max_artifacts: 10,
            auto_cleanup: true,
            allowed_commands: vec![
                "python".to_string(),
                "python3".to_string(),
                "qiskit".to_string(),
            ],
        }
    }
}

/// Represents an Aeonmi workspace with sandboxed file operations
#[derive(Debug, Clone)]
pub struct AeonmiWorkspace {
    /// Root directory of the project
    project_root: PathBuf,
    /// Source code directory
    source_dir: PathBuf,
    /// Output/artifacts directory
    output_dir: PathBuf,
    /// Temporary files directory
    temp_dir: PathBuf,
    /// Workspace configuration
    config: WorkspaceConfig,
}

/// Workspace-related errors
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Workspace not found: {0}")]
    NotFound(PathBuf),
    #[error("Invalid workspace structure: {0}")]
    #[allow(dead_code)]
    InvalidStructure(String),
    #[error("Permission denied: {0}")]
    #[allow(dead_code)]
    PermissionDenied(String),
    #[error("Path outside workspace: {0}")]
    PathOutsideWorkspace(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),
}

impl AeonmiWorkspace {
    /// Create a new workspace
    pub fn new(project_root: PathBuf, config: WorkspaceConfig) -> Result<Self> {
        let source_dir = project_root.join("src");
        let output_dir = project_root.join("output");
        let temp_dir = project_root.join(".aeonmi").join("temp");

        // Create directory structure
        fs::create_dir_all(&project_root).with_context(|| {
            format!("Failed to create project root: {}", project_root.display())
        })?;
        fs::create_dir_all(&source_dir).with_context(|| {
            format!(
                "Failed to create source directory: {}",
                source_dir.display()
            )
        })?;
        fs::create_dir_all(&output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
        fs::create_dir_all(&temp_dir)
            .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

        let workspace = Self {
            project_root: project_root.clone(),
            source_dir,
            output_dir,
            temp_dir,
            config,
        };

        // Create/update workspace manifest
        workspace.save_manifest()?;

        // Create gitignore if it doesn't exist
        workspace.create_gitignore()?;

        // Create sample files for new workspaces
        workspace.create_sample_files()?;

        Ok(workspace)
    }

    /// Load an existing workspace
    pub fn load(project_root: PathBuf) -> Result<Self> {
        if !project_root.exists() {
            return Err(WorkspaceError::NotFound(project_root).into());
        }

        let manifest_path = project_root.join("Aeonmi.toml");
        let config = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
            toml::from_str(&content)?
        } else {
            WorkspaceConfig::default()
        };

        let source_dir = project_root.join("src");
        let output_dir = project_root.join("output");
        let temp_dir = project_root.join(".aeonmi").join("temp");

        // Ensure directories exist
        fs::create_dir_all(&source_dir)?;
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            project_root,
            source_dir,
            output_dir,
            temp_dir,
            config,
        })
    }

    /// Get the project root directory
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the source directory
    pub fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    /// Get the output directory
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Get the temp directory
    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Get workspace configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &WorkspaceConfig {
        &self.config
    }

    /// Update workspace configuration
    #[allow(dead_code)]
    pub fn update_config(&mut self, config: WorkspaceConfig) -> Result<()> {
        self.config = config;
        self.save_manifest()
    }

    /// Validate that a path is within the workspace
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let canonical_workspace = self.project_root.canonicalize().with_context(|| {
            format!(
                "Failed to canonicalize workspace root: {}",
                self.project_root.display()
            )
        })?;

        let canonical_path = if path.is_absolute() {
            path.canonicalize().or_else(|_| {
                // If path doesn't exist, try parent directory
                if let Some(parent) = path.parent() {
                    let canonical_parent = parent.canonicalize()?;
                    Ok(canonical_parent.join(path.file_name().unwrap_or_default()))
                } else {
                    anyhow::bail!("Invalid path: {}", path.display())
                }
            })?
        } else {
            // Relative path - resolve against workspace root
            let full_path = self.project_root.join(path);
            full_path
                .canonicalize()
                .or_else(|_| -> std::io::Result<PathBuf> {
                    if let Some(parent) = full_path.parent() {
                        let canonical_parent = parent.canonicalize()?;
                        Ok(canonical_parent.join(full_path.file_name().unwrap_or_default()))
                    } else {
                        Ok(self.project_root.canonicalize()?.join(path))
                    }
                })?
        };

        if !canonical_path.starts_with(&canonical_workspace) {
            return Err(WorkspaceError::PathOutsideWorkspace(path.to_path_buf()).into());
        }

        Ok(canonical_path)
    }

    /// Create a new source file
    pub fn create_source_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.source_dir.join(relative_path);
        self.validate_path(&file_path)?;

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directories for: {}",
                    file_path.display()
                )
            })?;
        }

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write source file: {}", file_path.display()))?;

        Ok(file_path)
    }

    /// Read a source file
    pub fn read_source_file(&self, relative_path: &str) -> Result<String> {
        let file_path = self.source_dir.join(relative_path);
        self.validate_path(&file_path)?;

        fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read source file: {}", file_path.display()))
    }

    /// Create an output artifact
    pub fn create_artifact(&self, name: &str, content: &[u8]) -> Result<PathBuf> {
        let artifact_path = self.output_dir.join(name);
        self.validate_path(&artifact_path)?;

        fs::write(&artifact_path, content)
            .with_context(|| format!("Failed to write artifact: {}", artifact_path.display()))?;

        // Clean up old artifacts if needed
        if self.config.auto_cleanup {
            self.cleanup_old_artifacts()?;
        }

        Ok(artifact_path)
    }

    /// Create a temporary file
    pub fn create_temp_file(&self, name: &str, content: &[u8]) -> Result<PathBuf> {
        let temp_path = self.temp_dir.join(name);
        self.validate_path(&temp_path)?;

        fs::write(&temp_path, content)
            .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;

        Ok(temp_path)
    }

    /// List source files
    pub fn list_source_files(&self) -> Result<Vec<PathBuf>> {
        self.list_files_in_dir(&self.source_dir)
    }

    /// List output artifacts
    pub fn list_artifacts(&self) -> Result<Vec<PathBuf>> {
        self.list_files_in_dir(&self.output_dir)
    }

    /// Clean up all artifacts
    pub fn cleanup_artifacts(&self) -> Result<()> {
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir).with_context(|| {
                format!(
                    "Failed to remove output directory: {}",
                    self.output_dir.display()
                )
            })?;
            fs::create_dir_all(&self.output_dir).with_context(|| {
                format!(
                    "Failed to recreate output directory: {}",
                    self.output_dir.display()
                )
            })?;
        }

        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).with_context(|| {
                format!(
                    "Failed to remove temp directory: {}",
                    self.temp_dir.display()
                )
            })?;
            fs::create_dir_all(&self.temp_dir).with_context(|| {
                format!(
                    "Failed to recreate temp directory: {}",
                    self.temp_dir.display()
                )
            })?;
        }

        Ok(())
    }

    /// Get workspace statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> Result<WorkspaceStats> {
        let source_files = self.list_source_files()?.len();
        let artifacts = self.list_artifacts()?.len();

        let workspace_size = self.calculate_directory_size(&self.project_root)?;

        Ok(WorkspaceStats {
            source_files,
            artifacts,
            workspace_size,
            last_modified: self.get_last_modified()?,
        })
    }

    // Private helper methods

    fn save_manifest(&self) -> Result<()> {
        let manifest_path = self.project_root.join("Aeonmi.toml");
        let content = toml::to_string_pretty(&self.config)?;
        fs::write(&manifest_path, content)
            .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;
        Ok(())
    }

    fn create_gitignore(&self) -> Result<()> {
        let gitignore_path = self.project_root.join(".gitignore");
        if !gitignore_path.exists() {
            let content = r#"# Aeonmi workspace
.aeonmi/
output/
*.tmp
*.log

# OS specific
.DS_Store
Thumbs.db

# Editor specific
.vscode/
.idea/
*.swp
*.swo
*~
"#;
            fs::write(&gitignore_path, content).with_context(|| {
                format!("Failed to create .gitignore: {}", gitignore_path.display())
            })?;
        }
        Ok(())
    }

    fn create_sample_files(&self) -> Result<()> {
        let main_aeon = self.source_dir.join("main.aeon");
        if !main_aeon.exists() {
            let content = r#"// Welcome to Aeonmi!
// This is a sample quantum program.

use quantum::*;

fn main() {
    // Create a quantum circuit with 2 qubits
    let mut circuit = QuantumCircuit::new(2);
    
    // Apply Hadamard gate to first qubit
    circuit.h(0);
    
    // Apply CNOT gate
    circuit.cnot(0, 1);
    
    // Measure all qubits
    circuit.measure_all();
    
    // Execute the circuit
    let result = circuit.execute(1024);
    print("Measurement results: ", result);
}
"#;
            fs::write(&main_aeon, content).with_context(|| {
                format!("Failed to create sample file: {}", main_aeon.display())
            })?;
        }

        let readme = self.project_root.join("README.md");
        if !readme.exists() {
            let content = format!(
                r#"# {}

{}

## Getting Started

Run your quantum program:
```bash
aeon run
```

Build the project:
```bash
aeon build
```

## Project Structure

- `src/` - Source code files
- `output/` - Compiled artifacts and outputs
- `Aeonmi.toml` - Project configuration

## Learn More

Visit the [Aeonmi documentation](https://github.com/DarthMetaCrypro/Aeonmi) for more information.
"#,
                self.config.name,
                self.config
                    .description
                    .as_deref()
                    .unwrap_or("An Aeonmi quantum computing project")
            );
            fs::write(&readme, content)
                .with_context(|| format!("Failed to create README: {}", readme.display()))?;
        }

        Ok(())
    }

    fn cleanup_old_artifacts(&self) -> Result<()> {
        let mut artifacts = self.list_artifacts()?;
        if artifacts.len() <= self.config.max_artifacts {
            return Ok(());
        }

        // Sort by modification time (oldest first)
        artifacts.sort_by_key(|path| {
            path.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        // Remove oldest artifacts
        let to_remove = artifacts.len() - self.config.max_artifacts;
        for artifact in artifacts.iter().take(to_remove) {
            if artifact.is_file() {
                fs::remove_file(artifact).with_context(|| {
                    format!("Failed to remove old artifact: {}", artifact.display())
                })?;
            }
        }

        Ok(())
    }

    fn list_files_in_dir(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if dir.exists() {
            self.collect_files_recursive(dir, &mut files)?;
        }
        Ok(files)
    }

    fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                self.collect_files_recursive(&path, files)?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn calculate_directory_size(&self, dir: &Path) -> Result<u64> {
        let mut size = 0;
        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    size += entry.metadata()?.len();
                } else if path.is_dir() {
                    size += self.calculate_directory_size(&path)?;
                }
            }
        }
        Ok(size)
    }

    #[allow(dead_code)]
    fn get_last_modified(&self) -> Result<SystemTime> {
        let metadata = self.project_root.metadata()?;
        Ok(metadata.modified()?)
    }
}

/// Workspace statistics
#[derive(Debug)]
pub struct WorkspaceStats {
    pub source_files: usize,
    pub artifacts: usize,
    pub workspace_size: u64,
    pub last_modified: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = WorkspaceConfig {
            name: "test-project".to_string(),
            ..Default::default()
        };

        let workspace = AeonmiWorkspace::new(temp_dir.path().to_path_buf(), config)?;

        assert!(workspace.project_root().exists());
        assert!(workspace.source_dir().exists());
        assert!(workspace.output_dir().exists());
        assert!(workspace.temp_dir().exists());

        // Check manifest was created
        let manifest_path = workspace.project_root().join("Aeonmi.toml");
        assert!(manifest_path.exists());

        Ok(())
    }

    #[test]
    fn test_path_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace =
            AeonmiWorkspace::new(temp_dir.path().to_path_buf(), WorkspaceConfig::default())?;

        // Valid relative path
        let valid_path = Path::new("src/main.aeon");
        assert!(workspace.validate_path(valid_path).is_ok());

        // Invalid path outside workspace
        let invalid_path = temp_dir.path().parent().unwrap().join("outside.txt");
        assert!(workspace.validate_path(&invalid_path).is_err());

        Ok(())
    }

    #[test]
    fn test_file_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let workspace =
            AeonmiWorkspace::new(temp_dir.path().to_path_buf(), WorkspaceConfig::default())?;

        // Create and read source file
        let content = "fn main() { print(\"Hello, World!\"); }";
        workspace.create_source_file("test.aeon", content)?;
        let read_content = workspace.read_source_file("test.aeon")?;
        assert_eq!(content, read_content);

        // Create artifact
        let artifact_content = b"compiled bytecode";
        workspace.create_artifact("test.qasm", artifact_content)?;

        // List files
        let source_files = workspace.list_source_files()?;
        assert!(source_files.len() >= 1); // At least our test file + main.aeon

        Ok(())
    }
}
