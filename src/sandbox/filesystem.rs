use anyhow::Result;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use dunce::canonicalize;

#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
static FS_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Sandboxed file system that restricts access to workspace
#[derive(Debug, Clone)]
pub struct SandboxedFileSystem {
    /// Allowed root directories
    allowed_roots: Arc<Mutex<HashSet<PathBuf>>>,
    /// Whether to allow absolute paths
    allow_absolute_paths: bool,
    /// Whether to allow parent directory access (..)
    allow_parent_access: bool,
    /// Read-only mode
    read_only: bool,
}

/// File system guard that automatically enforces sandbox rules
pub struct FileSystemGuard {
    sandbox: SandboxedFileSystem,
    original_working_dir: PathBuf,
}

/// Sandbox-related errors
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Path outside sandbox: {0}")]
    PathOutsideSandbox(PathBuf),
    #[error("Absolute paths not allowed: {0}")]
    AbsolutePathNotAllowed(PathBuf),
    #[error("Parent directory access not allowed: {0}")]
    ParentAccessNotAllowed(PathBuf),
    #[error("Write operation not allowed in read-only mode: {0}")]
    WriteNotAllowed(PathBuf),
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Check if a path is safely contained within a base directory
fn is_safe_subpath(base: &Path, user: &Path) -> std::io::Result<bool> {
    let base = canonicalize(base)?;
    // Reject any .. early (before touching FS)
    if user.components().any(|c| matches!(c, Component::ParentDir)) {
        return Ok(false);
    }
    let full = canonicalize(base.join(user));
    match full {
        Ok(f) => Ok(f.starts_with(&base)),
        Err(_) => Ok(false), // nonexistent still must be under base
    }
}

impl SandboxedFileSystem {
    /// Create a new sandboxed file system
    pub fn new() -> Self {
        Self {
            allowed_roots: Arc::new(Mutex::new(HashSet::new())),
            allow_absolute_paths: false,
            allow_parent_access: false,
            read_only: false,
        }
    }

    /// Create a sandbox with a single allowed root
    pub fn with_root<P: AsRef<Path>>(root: P) -> Self {
        let mut sandbox = Self::new();
        sandbox.allow_absolute_paths = true; // Allow absolute paths when using roots
        sandbox.add_allowed_root(root);
        sandbox
    }

    /// Add an allowed root directory
    pub fn add_allowed_root<P: AsRef<Path>>(&mut self, root: P) {
        let canonical_root = canonicalize(root.as_ref())
            .unwrap_or_else(|_| root.as_ref().to_path_buf());

        let mut roots = self.allowed_roots.lock().unwrap();
        roots.insert(canonical_root);
    }

    /// Set whether absolute paths are allowed
    pub fn set_allow_absolute_paths(&mut self, allow: bool) {
        self.allow_absolute_paths = allow;
    }

    /// Set whether parent directory access is allowed
    pub fn set_allow_parent_access(&mut self, allow: bool) {
        self.allow_parent_access = allow;
    }

    /// Set read-only mode
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Validate a path against sandbox rules
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, SandboxError> {
        let path = path.as_ref();

        // Check if path is within allowed roots
        let roots = self.allowed_roots.lock().unwrap();
        let has_roots = !roots.is_empty();
        drop(roots); // Release lock early

        // If no roots are set, allow all access (no sandbox restrictions)
        if !has_roots {
            return self.resolve_path(path).map_err(|e| e.into());
        }

        // Check absolute path restriction (only when roots are set)
        if !self.allow_absolute_paths && path.is_absolute() {
            return Err(SandboxError::AbsolutePathNotAllowed(path.to_path_buf()));
        }

        // Check parent directory access
        if !self.allow_parent_access {
            for component in path.components() {
                if component.as_os_str() == ".." {
                    return Err(SandboxError::ParentAccessNotAllowed(path.to_path_buf()));
                }
            }
        }

        // Resolve the path against allowed roots
        let resolved_path = self.resolve_path(path)?;

        // Canonicalize the resolved path for comparison (use dunce to avoid UNC)
        let canonical_resolved = canonicalize(&resolved_path)
            .unwrap_or_else(|_| resolved_path.clone());

        let roots = self.allowed_roots.lock().unwrap();
        for root in roots.iter() {
            if canonical_resolved.starts_with(root) {
                return Ok(resolved_path);
            }
        }

        Err(SandboxError::PathOutsideSandbox(path.to_path_buf()))
    }

    /// Validate a path for write operations
    pub fn validate_write_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, SandboxError> {
        if self.read_only {
            return Err(SandboxError::WriteNotAllowed(path.as_ref().to_path_buf()));
        }
        self.validate_path(path)
    }

    /// Safely read a file
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, SandboxError> {
        let validated_path = self.validate_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::read(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely read a file as string
    pub fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String, SandboxError> {
        let validated_path = self.validate_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::read_to_string(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely write a file
    pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        path: P,
        contents: C,
    ) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = validated_path.parent() {
            std::fs::create_dir_all(parent).map_err(SandboxError::Io)?;
        }

        std::fs::write(&validated_path, contents).map_err(SandboxError::Io)
    }

    /// Safely create a directory
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;
        std::fs::create_dir(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely create directories recursively
    pub fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;
        std::fs::create_dir_all(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely remove a file
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::remove_file(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely remove a directory
    pub fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::remove_dir(&validated_path).map_err(SandboxError::Io)
    }

    /// Safely remove a directory and all its contents
    pub fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<(), SandboxError> {
        let validated_path = self.validate_write_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::remove_dir_all(&validated_path).map_err(SandboxError::Io)
    }

    /// List directory contents
    pub fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PathBuf>, SandboxError> {
        let validated_path = self.validate_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&validated_path).map_err(SandboxError::Io)? {
            let entry = entry.map_err(SandboxError::Io)?;
            entries.push(entry.path());
        }

        Ok(entries)
    }

    /// Get file metadata
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<std::fs::Metadata, SandboxError> {
        let validated_path = self.validate_path(path)?;

        if !validated_path.exists() {
            return Err(SandboxError::PathNotFound(validated_path));
        }

        std::fs::metadata(&validated_path).map_err(SandboxError::Io)
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.validate_path(path)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.validate_path(path)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    /// Check if a path is a directory
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.validate_path(path)
            .map(|p| p.is_dir())
            .unwrap_or(false)
    }

    // Private helper methods

    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, SandboxError> {
        let path = path.as_ref();

        if path.is_absolute() {
            // For absolute paths, try to canonicalize directly
            path.canonicalize()
                .or_else(|_| Ok(path.to_path_buf()))
                .map_err(SandboxError::Io)
        } else {
            // For relative paths, resolve against the first allowed root
            let roots = self.allowed_roots.lock().unwrap();
            if let Some(root) = roots.iter().next() {
                let full_path = root.join(path);
                full_path
                    .canonicalize()
                    .or_else(|_| Ok(full_path))
                    .map_err(SandboxError::Io)
            } else {
                // No roots set, use current directory
                std::env::current_dir()
                    .map(|cwd| cwd.join(path))
                    .map_err(SandboxError::Io)
            }
        }
    }
}

impl Default for SandboxedFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemGuard {
    /// Create a new file system guard that changes to the sandbox directory
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(sandbox_root: P) -> Result<Self, SandboxError> {
        let original_working_dir = std::env::current_dir().map_err(SandboxError::Io)?;
        let sandbox_root = sandbox_root.as_ref();

        // Change to sandbox directory
        std::env::set_current_dir(sandbox_root).map_err(SandboxError::Io)?;

        // Create sandbox with the root
        let mut sandbox = SandboxedFileSystem::new();
        sandbox.add_allowed_root(sandbox_root);

        Ok(Self {
            sandbox,
            original_working_dir,
        })
    }

    /// Get the underlying sandbox
    #[allow(dead_code)]
    pub fn sandbox(&self) -> &SandboxedFileSystem {
        &self.sandbox
    }

    /// Get a mutable reference to the sandbox
    #[allow(dead_code)]
    pub fn sandbox_mut(&mut self) -> &mut SandboxedFileSystem {
        &mut self.sandbox
    }
}

impl Drop for FileSystemGuard {
    fn drop(&mut self) {
        // Restore original working directory
        let _ = std::env::set_current_dir(&self.original_working_dir);
    }
}

/// Utility functions for common file operations

/// Copy a file within the sandbox
pub fn copy_file<P: AsRef<Path>>(
    sandbox: &SandboxedFileSystem,
    from: P,
    to: P,
) -> Result<(), SandboxError> {
    let from_path = sandbox.validate_path(from)?;
    let to_path = sandbox.validate_write_path(to)?;

    if !from_path.exists() {
        return Err(SandboxError::PathNotFound(from_path));
    }

    std::fs::copy(&from_path, &to_path).map_err(SandboxError::Io)?;
    Ok(())
}

/// Move/rename a file within the sandbox
pub fn move_file<P: AsRef<Path>>(
    sandbox: &SandboxedFileSystem,
    from: P,
    to: P,
) -> Result<(), SandboxError> {
    let from_path = sandbox.validate_write_path(from)?;
    let to_path = sandbox.validate_write_path(to)?;

    if !from_path.exists() {
        return Err(SandboxError::PathNotFound(from_path));
    }

    std::fs::rename(&from_path, &to_path).map_err(SandboxError::Io)?;
    Ok(())
}

/// Create a temporary file within the sandbox
pub fn create_temp_file<P: AsRef<Path>>(
    sandbox: &SandboxedFileSystem,
    dir: P,
    prefix: &str,
    suffix: &str,
) -> Result<PathBuf, SandboxError> {
    let dir_path = sandbox.validate_write_path(dir)?;

    // Ensure directory exists
    if !dir_path.exists() {
        sandbox.create_dir_all(&dir_path)?;
    }

    // Generate unique filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let filename = format!("{}{}{}", prefix, timestamp, suffix);
    let temp_path = dir_path.join(filename);

    // Create empty file
    sandbox.write_file(&temp_path, b"")?;

    Ok(temp_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sandbox_creation() {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let sandbox = SandboxedFileSystem::new();

        // Should allow access when no roots are set
        let temp_path = std::env::temp_dir().join("test.txt");
        assert!(sandbox.validate_path(&temp_path).is_ok());
    }

    #[test]
    fn test_sandbox_with_root() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new()?;
        let sandbox = SandboxedFileSystem::with_root(temp_dir.path());

        // Should allow access within root
        let allowed_path = temp_dir.path().join("test.txt");
        assert!(sandbox.validate_path(&allowed_path).is_ok());

        // Should deny access outside root
        let denied_path = temp_dir.path().parent().unwrap().join("outside.txt");
        assert!(sandbox.validate_path(&denied_path).is_err());

        Ok(())
    }

    #[test]
    fn test_parent_directory_restriction() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new()?;
        let mut sandbox = SandboxedFileSystem::with_root(temp_dir.path());

        // Should deny parent access by default
        let parent_path = Path::new("../outside.txt");
        assert!(sandbox.validate_path(parent_path).is_err());

        // Should allow when enabled
        sandbox.set_allow_parent_access(true);
        // Note: This might still fail due to sandbox root restriction

        Ok(())
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new()?;
        let sandbox = SandboxedFileSystem::with_root(temp_dir.path());

        // Test write and read
        let test_file = "test.txt";
        let test_content = "Hello, World!";

        sandbox.write_file(test_file, test_content.as_bytes())?;
        let read_content = sandbox.read_to_string(test_file)?;

        assert_eq!(test_content, read_content);

        // Test directory operations
        sandbox.create_dir("subdir")?;
        assert!(sandbox.is_dir("subdir"));

        // Test file listing
        let entries = sandbox.read_dir(".")?;
        assert!(entries.len() >= 2); // At least test.txt and subdir

        Ok(())
    }

    #[test]
    fn test_read_only_mode() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new()?;
        let mut sandbox = SandboxedFileSystem::with_root(temp_dir.path());

        sandbox.set_read_only(true);

        // Should deny write operations
        assert!(sandbox.write_file("test.txt", b"content").is_err());
        assert!(sandbox.create_dir("testdir").is_err());

        Ok(())
    }

    #[test]
    fn test_filesystem_guard() -> Result<(), Box<dyn std::error::Error>> {
        let _guard = FS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;

        {
            let _guard = FileSystemGuard::new(temp_dir.path())?;

            // Should be in the sandbox directory now
            let current_dir = std::env::current_dir()?;
            assert_eq!(current_dir, canonicalize(temp_dir.path())?);
        }

        // Should be back to original directory
        let current_dir = std::env::current_dir()?;
        assert_eq!(current_dir, original_dir);

        Ok(())
    }
}
