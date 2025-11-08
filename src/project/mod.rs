use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;

use crate::config::resolve_config_path;
use crate::core::lexer::Lexer;
use crate::core::parser::Parser as QubeParser;
use walkdir::WalkDir;

mod parser;
pub mod qasm_export;
pub mod python_export;
pub mod diagnostics;

pub use parser::{Program, TestReport};
pub use diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticLogger};

/// Supported build profiles for Aeonmi projects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
        }
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    package: PackageSection,
    #[serde(default)]
    aeonmi: AeonmiSection,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize, Default)]
struct AeonmiSection {
    #[serde(default = "default_entry_path")]
    entry: PathBuf,
    #[serde(default)]
    modules: Vec<PathBuf>,
    #[serde(default)]
    tests: Vec<TestEntry>,
}

fn default_entry_path() -> PathBuf {
    PathBuf::from("src/main.ai")
}

#[derive(Debug, Deserialize)]
struct TestEntry {
    name: String,
    path: PathBuf,
}

pub struct Project {
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: Manifest,
}

impl Project {
    /// Locate and load a project starting from the given working directory.
    pub fn load(manifest_path: Option<PathBuf>) -> Result<Self> {
        let root = std::env::current_dir().context("resolve working directory")?;
        let manifest_path = match manifest_path {
            Some(path) => path,
            None => root.join("Aeonmi.toml"),
        };

        if !manifest_path.exists() {
            bail!(
                "No Aeonmi.toml manifest found at {}",
                manifest_path.display()
            );
        }

        let manifest_dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.clone());

        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read manifest {}", manifest_path.display()))?;
        let manifest: Manifest = toml::from_str(&raw)
            .with_context(|| format!("parse manifest {}", manifest_path.display()))?;

        Ok(Project {
            root: manifest_dir,
            manifest_path,
            manifest,
        })
    }

    pub fn package_name(&self) -> &str {
        &self.manifest.package.name
    }

    pub fn package_version(&self) -> &str {
        &self.manifest.package.version
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn entry_path(&self) -> PathBuf {
        self.root.join(&self.manifest.aeonmi.entry)
    }

    fn module_paths(&self) -> Vec<PathBuf> {
        self.manifest
            .aeonmi
            .modules
            .iter()
            .map(|p| self.root.join(p))
            .collect()
    }

    fn test_paths(&self) -> Result<Vec<(String, PathBuf)>> {
        if self.manifest.aeonmi.tests.is_empty() {
            let default_dir = self.root.join("tests");
            if !default_dir.exists() {
                return Ok(Vec::new());
            }
            let mut collected = Vec::new();
            for entry in WalkDir::new(&default_dir) {
                let entry = entry?;
                if entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .map(|ext| ext == "ai")
                        .unwrap_or(false)
                {
                    let file_name = entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("test")
                        .to_string();
                    collected.push((file_name, entry.path().to_path_buf()));
                }
            }
            Ok(collected)
        } else {
            Ok(self
                .manifest
                .aeonmi
                .tests
                .iter()
                .map(|t| (t.name.clone(), self.root.join(&t.path)))
                .collect())
        }
    }

    pub fn load_program(&self) -> Result<Program> {
        let entry_path = self.entry_path();
        let entry_src = fs::read_to_string(&entry_path)
            .with_context(|| format!("read entry {}", entry_path.display()))?;

        // Check file extension to determine parser
        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "aeonmi" | "qube" => {
                // Use full QUBE parser for .aeonmi and .qube files
                let mut lexer = Lexer::new(&entry_src, true); // ai_access_authorized = true
                let tokens = lexer
                    .tokenize()
                    .map_err(|e| anyhow::anyhow!("Lexer error: {}", e))?;
                let mut parser = QubeParser::new(tokens);
                let _ast = parser
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Parser error: {}", e))?;

                // For now, just validate parsing works
                // TODO: Implement full AST execution or compilation
                bail!("Full QUBE execution not yet implemented. Use 'aeonmi run {}' for direct execution.", entry_path.display())
            }
            "ai" => {
                // Use simple .ai parser
                let mut fragments = Vec::new();
                fragments.push(parser::parse_fragment(&entry_path, &entry_src)?);

                for module in self.module_paths() {
                    let src = fs::read_to_string(&module)
                        .with_context(|| format!("read module {}", module.display()))?;
                    fragments.push(parser::parse_fragment(&module, &src)?);
                }

                let mut builder = parser::ProgramBuilder::new();
                for fragment in fragments {
                    builder.add_fragment(fragment)?;
                }
                Ok(builder.build())
            }
            _ => {
                bail!(
                    "Unsupported file extension '{}' for entry point. Use .ai, .aeonmi, or .qube",
                    ext
                )
            }
        }
    }

    fn ensure_target_dir(&self, profile: BuildProfile) -> Result<PathBuf> {
        let target = self.root.join("target").join(profile.as_str());
        fs::create_dir_all(&target)
            .with_context(|| format!("create target directory {}", target.display()))?;
        Ok(target)
    }

    pub fn build(&self, profile: BuildProfile) -> Result<PathBuf> {
        let program = self.load_program()?;
        program.require_main()?;

        let target_dir = self.ensure_target_dir(profile)?;
        let artifact_path = target_dir.join(format!("{}.bundle.json", self.package_name()));

        let compiled = json!({
            "package": {
                "name": self.package_name(),
                "version": self.package_version(),
            },
            "profile": profile.as_str(),
            "entry": self
                .manifest
                .aeonmi
                .entry
                .to_string_lossy()
                .to_string(),
            "generated_modules": program.function_names(),
        });

        fs::write(&artifact_path, serde_json::to_string_pretty(&compiled)?).with_context(|| {
            format!(
                "write build artifact {}",
                artifact_path
                    .strip_prefix(&self.root)
                    .unwrap_or(&artifact_path)
                    .display()
            )
        })?;

        Ok(artifact_path)
    }

    pub fn check(&self) -> Result<()> {
        let program = self.load_program()?;
        program.require_main()
    }

    pub fn run(&self, release: bool) -> Result<()> {
        let profile = if release {
            BuildProfile::Release
        } else {
            BuildProfile::Debug
        };
        self.build(profile)?;
        let program = self.load_program()?;
        program.execute_main()
    }

    pub fn run_tests(&self, release: bool, filter: Option<&str>) -> Result<Vec<TestReport>> {
        let profile = if release {
            BuildProfile::Release
        } else {
            BuildProfile::Debug
        };
        self.build(profile)?;
        let mut program = self.load_program()?;

        let mut reports = program.run_tests(filter)?;

        for (name, path) in self.test_paths()? {
            let src = fs::read_to_string(&path)
                .with_context(|| format!("read test {}", path.display()))?;
            let fragment = parser::parse_fragment(&path, &src)?;
            let mut builder = parser::ProgramBuilder::new();
            builder.add_fragment(fragment)?;
            let mut isolated = builder.build();
            let mut isolated_reports = isolated.run_tests(filter)?;
            for report in &mut isolated_reports {
                report.group = Some(name.clone());
            }
            reports.extend(isolated_reports);
        }

        Ok(reports)
    }
}

pub fn load_project_from_cli(manifest: Option<PathBuf>) -> Result<Project> {
    Project::load(manifest)
}

pub fn resolve_project_manifest(cli_override: &Option<PathBuf>) -> Option<PathBuf> {
    resolve_config_path(cli_override)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_project(tmp: &tempfile::TempDir, main_src: &str) -> Result<Project> {
        let root = tmp.path();
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src/main.ai"), main_src)?;
        fs::write(
            root.join("Aeonmi.toml"),
            "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\n[aeonmi]\nentry=\"src/main.ai\"\n",
        )?;

        // Load project with explicit manifest path instead of changing directory
        let manifest_path = root.join("Aeonmi.toml");
        Project::load(Some(manifest_path))
    }

    #[test]
    fn build_and_run_simple_program() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let project = fixture_project(
            &tmp,
            r#"
fn main:
    print "hello"
    let answer = 2 + 2
    assert answer == 4
"#,
        )?;
        project.check()?;
        let artifact = project.build(BuildProfile::Debug)?;
        assert!(artifact.exists());
        project.run(false)?;
        Ok(())
    }

    #[test]
    fn detect_missing_main() {
        let tmp = tempfile::tempdir().unwrap();
        let project = fixture_project(
            &tmp,
            r#"
fn helper:
    print "no entry"
"#,
        )
        .unwrap();

        let err = project.check().unwrap_err();
        assert!(err.to_string().contains("main"));
    }
}
