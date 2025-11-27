//! Incremental compilation and build optimization for Aeonmi projects
//!
//! This module provides:
//! - Change detection to avoid recompiling unchanged files
//! - Dependency analysis for build optimization
//! - Parallel compilation support
//! - Build cache management

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use crate::cli_enhanced::OutputFormat;

/// Build cache entry for incremental compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File path relative to project root
    pub file_path: PathBuf,
    /// Last modification time
    pub mtime: SystemTime,
    /// File hash for content change detection
    pub hash: String,
    /// Dependencies of this file
    pub dependencies: Vec<PathBuf>,
    /// Output format used for compilation
    pub output_format: OutputFormat,
    /// Optimization level used
    pub opt_level: u8,
    /// Whether debug info was included
    pub debug_info: bool,
}

impl CacheEntry {
    pub fn new(
        file_path: PathBuf,
        mtime: SystemTime,
        hash: String,
        dependencies: Vec<PathBuf>,
        output_format: OutputFormat,
        opt_level: u8,
        debug_info: bool,
    ) -> Self {
        Self {
            file_path,
            mtime,
            hash,
            dependencies,
            output_format,
            opt_level,
            debug_info,
        }
    }

    /// Check if this cache entry is still valid
    pub fn is_valid(&self, project_root: &Path) -> Result<bool> {
        let full_path = project_root.join(&self.file_path);

        // Check if file still exists
        if !full_path.exists() {
            return Ok(false);
        }

        // Check modification time
        let metadata = fs::metadata(&full_path)?;
        let current_mtime = metadata.modified()?;
        if current_mtime != self.mtime {
            return Ok(false);
        }

        // Check file hash
        let content = fs::read(&full_path)?;
        let current_hash = format!("{:x}", seahash::hash(&content));
        if current_hash != self.hash {
            return Ok(false);
        }

        // Check if any dependencies have changed
        for dep in &self.dependencies {
            let dep_path = project_root.join(dep);
            if dep_path.exists() {
                let dep_metadata = fs::metadata(&dep_path)?;
                let dep_mtime = dep_metadata.modified()?;
                if dep_mtime > self.mtime {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

/// Build cache for incremental compilation
#[derive(Debug)]
pub struct BuildCache {
    cache_file: PathBuf,
    entries: RwLock<HashMap<PathBuf, CacheEntry>>,
}

impl BuildCache {
    /// Create a new build cache
    pub fn new(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join("build_cache.json");
        Self {
            cache_file,
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Load cache from disk
    pub fn load(&self) -> Result<()> {
        if self.cache_file.exists() {
            let content = fs::read_to_string(&self.cache_file)?;
            let entries: HashMap<PathBuf, CacheEntry> = serde_json::from_str(&content)?;
            *self.entries.write() = entries;
        }
        Ok(())
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let entries = self.entries.read();
        let content = serde_json::to_string_pretty(&*entries)?;
        fs::create_dir_all(self.cache_file.parent().unwrap())?;
        fs::write(&self.cache_file, content)?;
        Ok(())
    }

    /// Get cache entry for a file
    pub fn get(&self, file_path: &Path) -> Option<CacheEntry> {
        self.entries.read().get(file_path).cloned()
    }

    /// Update cache entry for a file
    pub fn update(&self, entry: CacheEntry) {
        self.entries.write().insert(entry.file_path.clone(), entry);
    }

    /// Remove cache entry for a file
    pub fn remove(&self, file_path: &Path) {
        self.entries.write().remove(file_path);
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// Get all files that need recompilation based on changes
    pub fn needs_recompilation(&self, project_root: &Path, all_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut needs_compile = Vec::new();

        for file_path in all_files {
            let relative_path = file_path.strip_prefix(project_root).unwrap_or(file_path);

            if let Some(entry) = self.get(relative_path) {
                if !entry.is_valid(project_root)? {
                    needs_compile.push(file_path.clone());
                }
            } else {
                // No cache entry, needs compilation
                needs_compile.push(file_path.clone());
            }
        }

        Ok(needs_compile)
    }
}

/// Dependency analyzer for Aeonmi files
#[derive(Debug)]
pub struct DependencyAnalyzer {
    project_root: PathBuf,
}

impl DependencyAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Analyze dependencies for a single file
    pub fn analyze_file(&self, file_path: &Path) -> Result<Vec<PathBuf>> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut dependencies = Vec::new();

        // Simple dependency analysis - look for import statements
        // This is a basic implementation; could be enhanced with proper AST analysis
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("import ") || line.starts_with("from ") {
                // Extract module path from import statement
                // This is simplified - real implementation would parse properly
                if let Some(path_start) = line.find('"') {
                    if let Some(path_end) = line[path_start + 1..].find('"') {
                        let module_path = &line[path_start + 1..path_start + 1 + path_end];
                        let dep_path = self.resolve_module_path(module_path, file_path)?;
                        if let Some(dep) = dep_path {
                            dependencies.push(dep);
                        }
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Analyze dependencies for all files
    pub fn analyze_all(&self, files: &[PathBuf]) -> Result<HashMap<PathBuf, Vec<PathBuf>>> {
        let mut all_deps = HashMap::new();

        for file in files {
            let deps = self.analyze_file(file)?;
            let relative_path = file.strip_prefix(&self.project_root).unwrap_or(file);
            all_deps.insert(relative_path.to_path_buf(), deps);
        }

        Ok(all_deps)
    }

    /// Build dependency graph and return compilation order
    pub fn build_order(&self, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let deps = self.analyze_all(files)?;
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        // Convert absolute paths to relative for lookup
        let file_map: HashMap<PathBuf, PathBuf> = files.iter()
            .map(|f| (f.strip_prefix(&self.project_root).unwrap_or(f).to_path_buf(), f.clone()))
            .collect();

        for file in files {
            let relative = file.strip_prefix(&self.project_root).unwrap_or(file);
            self.visit_file(relative, &deps, &mut visited, &mut visiting, &mut order, &file_map)?;
        }

        Ok(order)
    }

    fn visit_file(
        &self,
        file: &Path,
        deps: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        visiting: &mut HashSet<PathBuf>,
        order: &mut Vec<PathBuf>,
        file_map: &HashMap<PathBuf, PathBuf>,
    ) -> Result<()> {
        if visited.contains(file) {
            return Ok(());
        }

        if visiting.contains(file) {
            bail!("Circular dependency detected involving {}", file.display());
        }

        visiting.insert(file.to_path_buf());

        if let Some(file_deps) = deps.get(file) {
            for dep in file_deps {
                if let Some(dep_file) = file_map.get(dep) {
                    let dep_relative = dep_file.strip_prefix(&self.project_root).unwrap_or(dep_file);
                    self.visit_file(dep_relative, deps, visited, visiting, order, file_map)?;
                }
            }
        }

        visiting.remove(file);
        visited.insert(file.to_path_buf());

        if let Some(full_path) = file_map.get(file) {
            order.push(full_path.clone());
        }

        Ok(())
    }

    fn resolve_module_path(&self, module_path: &str, current_file: &Path) -> Result<Option<PathBuf>> {
        // Simple module resolution - could be enhanced
        let mut candidate_paths = Vec::new();

        // Relative to current file
        if let Some(parent) = current_file.parent() {
            candidate_paths.push(parent.join(format!("{}.ai", module_path)));
            candidate_paths.push(parent.join(module_path).with_extension("ai"));
        }

        // Relative to project root
        candidate_paths.push(self.project_root.join(format!("{}.ai", module_path)));
        candidate_paths.push(self.project_root.join("src").join(format!("{}.ai", module_path)));
        candidate_paths.push(self.project_root.join("lib").join(format!("{}.ai", module_path)));

        for path in candidate_paths {
            if path.exists() {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }
}

/// Parallel compilation manager
#[derive(Debug)]
pub struct ParallelCompiler {
    max_jobs: usize,
}

impl ParallelCompiler {
    pub fn new(max_jobs: usize) -> Self {
        Self { max_jobs }
    }

    /// Compile files in parallel
    pub async fn compile_parallel<F, Fut, T>(
        &self,
        files: Vec<PathBuf>,
        compile_fn: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(PathBuf) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        use futures_util::stream::{self, StreamExt};

        let compile_fn = Arc::new(compile_fn);
        let results = stream::iter(files)
            .map(|file| {
                let compile_fn = Arc::clone(&compile_fn);
                async move {
                    compile_fn(file).await
                }
            })
            .buffer_unordered(self.max_jobs)
            .collect::<Vec<_>>()
            .await;

        let mut outputs = Vec::new();
        for result in results {
            outputs.push(result?);
        }

        Ok(outputs)
    }
}

/// Calculate file hash for change detection
pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path)?;
    Ok(format!("{:x}", seahash::hash(&content)))
}

/// Get file modification time
pub fn get_file_mtime(file_path: &Path) -> Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.modified()?)
}