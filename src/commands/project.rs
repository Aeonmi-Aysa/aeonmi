use std::path::PathBuf;
use std::fs;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::project::{BuildProfile, Project, TestReport};

/// Build target format for compilation output
#[derive(Debug, Clone, Copy)]
pub enum BuildTarget {
    Bytecode,
    Qasm,
    Python,
}

impl BuildTarget {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "bytecode" => Ok(Self::Bytecode),
            "qasm" => Ok(Self::Qasm),
            "python" => Ok(Self::Python),
            _ => bail!("Unknown build target: {}. Supported: bytecode, qasm, python", s),
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Bytecode => "bc",
            Self::Qasm => "qasm",
            Self::Python => "py",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Bytecode => "bytecode",
            Self::Qasm => "qasm",
            Self::Python => "python",
        }
    }
}

pub fn build(
    manifest_path: Option<PathBuf>,
    release: bool,
    target: Option<String>,
) -> Result<()> {
    let project = Project::load(manifest_path)?;
    let profile = if release {
        BuildProfile::Release
    } else {
        BuildProfile::Debug
    };

    // Parse build target
    let build_target = target
        .as_deref()
        .map(BuildTarget::from_str)
        .transpose()?
        .unwrap_or(BuildTarget::Bytecode);

    let artifact = project.build(profile)?;

    // Ensure output directory exists
    let output_dir = project.root().join("output");
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    // Copy artifact to output directory with appropriate extension
    let output_file = output_dir.join(format!(
        "{}.{}",
        project.package_name(),
        build_target.extension()
    ));

    // For now, just copy the bundle. In future, we'll add actual target compilation
    fs::copy(&artifact, &output_file).with_context(|| {
        format!(
            "Failed to copy artifact {} to {}",
            artifact.display(),
            output_file.display()
        )
    })?;

    println!(
        "Built {} v{} [{}] -> {} (target: {})",
        project.package_name(),
        project.package_version(),
        profile.as_str(),
        output_file.display(),
        build_target.as_str()
    );
    Ok(())
}

pub fn check(manifest_path: Option<PathBuf>) -> Result<()> {
    use crate::project::DiagnosticLogger;
    
    let project = Project::load(manifest_path)?;
    
    // Set up diagnostic logger
    let log_path = project.root().join("output").join("log.txt");
    let mut logger = DiagnosticLogger::with_log_file(&log_path)
        .unwrap_or_else(|_| DiagnosticLogger::new());
    
    println!("Checking {} v{}...", project.package_name(), project.package_version());
    
    match project.check() {
        Ok(()) => {
            println!("✓ Check passed successfully");
            Ok(())
        }
        Err(e) => {
            logger.emit_error(
                &e.to_string(),
                None,
                None,
                Some("Run 'aeon build' to see full compilation output")
            );
            Err(e)
        }
    }
}

pub fn run(manifest_path: Option<PathBuf>, release: bool, timeout_ms: Option<u64>) -> Result<()> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use crate::project::DiagnosticLogger;
    
    let project = Project::load(manifest_path.clone())?;
    
    // Set up logging to output/log.txt
    let log_path = project.root().join("output").join("log.txt");
    std::fs::create_dir_all(log_path.parent().unwrap())?;
    
    let mut logger = DiagnosticLogger::with_log_file(&log_path)
        .unwrap_or_else(|_| DiagnosticLogger::new());
    
    // Build the project
    let profile = if release {
        crate::project::BuildProfile::Release
    } else {
        crate::project::BuildProfile::Debug
    };
    
    println!("Info: Compiling and running Aeonmi program");
    
    match project.build(profile) {
        Ok(_) => {},
        Err(e) => {
            logger.emit_error(
                &format!("Build failed: {}", e),
                None,
                None,
                Some("Check your source files for syntax errors")
            );
            return Err(e);
        }
    }
    
    // Load program
    let program = match project.load_program() {
        Ok(p) => p,
        Err(e) => {
            logger.emit_error(
                &format!("Failed to load program: {}", e),
                None,
                None,
                Some("Check that your source files are valid")
            );
            return Err(e);
        }
    };
    
    // Set up timeout if specified
    let cancel_flag = Arc::new(AtomicBool::new(false));
    
    if let Some(timeout) = timeout_ms {
        if timeout > 0 {
            println!("Running with timeout: {}ms", timeout);
            let flag_clone = Arc::clone(&cancel_flag);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(timeout));
                flag_clone.store(true, Ordering::Relaxed);
            });
        }
    }
    
    // Execute with timeout and logging
    match program.execute_main_with_timeout_and_log(cancel_flag, Some(log_path.clone())) {
        Ok(()) => Ok(()),
        Err(e) => {
            logger.emit_error(
                &format!("Runtime error: {}", e),
                None,
                None,
                Some("Check output/log.txt for full execution trace")
            );
            Err(e)
        }
    }
}

pub fn test(manifest_path: Option<PathBuf>, release: bool, filter: Option<String>) -> Result<()> {
    let project = Project::load(manifest_path)?;
    let reports = project.run_tests(release, filter.as_deref())?;

    if reports.is_empty() {
        println!("No tests found");
        return Ok(());
    }

    // Create test results directory
    let test_results_dir = project.root().join("output").join("test-results");
    fs::create_dir_all(&test_results_dir).with_context(|| {
        format!(
            "Failed to create test results directory: {}",
            test_results_dir.display()
        )
    })?;

    // Save test results as JSON
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let results_file = test_results_dir.join(format!("test-results-{}.json", timestamp));
    
    let results_json = serde_json::to_string_pretty(&reports)
        .context("Failed to serialize test results")?;
    fs::write(&results_file, results_json)
        .with_context(|| format!("Failed to write test results to {}", results_file.display()))?;

    let mut passed = 0usize;
    for report in &reports {
        if report.passed {
            passed += 1;
        }
        print_report(report);
    }

    if passed == reports.len() {
        println!("test result: ok. {} passed; 0 failed", reports.len());
        println!("Results saved to: {}", results_file.display());
        Ok(())
    } else {
        let failed = reports.len() - passed;
        println!("test result: FAILED. {} passed; {} failed", passed, failed);
        println!("Results saved to: {}", results_file.display());
        bail!("{} test(s) failed", failed)
    }
}

fn print_report(report: &TestReport) {
    let group = report
        .group
        .as_ref()
        .map(|g| format!("{}::", g))
        .unwrap_or_default();
    if report.passed {
        println!("    ok - {}{}", group, report.name);
    } else {
        println!("    FAILED - {}{}", group, report.name);
        if let Some(message) = &report.message {
            println!("        reason: {}", message);
        }
    }
}

/// Clean build artifacts and output directory
pub fn clean(manifest_path: Option<PathBuf>) -> Result<()> {
    let project = Project::load(manifest_path)?;
    let root = project.root();

    let mut cleaned = Vec::new();

    // Remove output directory
    let output_dir = root.join("output");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .with_context(|| format!("Failed to remove output directory: {}", output_dir.display()))?;
        cleaned.push("output/");
    }

    // Remove target directory
    let target_dir = root.join("target");
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .with_context(|| format!("Failed to remove target directory: {}", target_dir.display()))?;
        cleaned.push("target/");
    }

    if cleaned.is_empty() {
        println!("Already clean (nothing to remove)");
    } else {
        println!("Removed: {}", cleaned.join(", "));
    }

    Ok(())
}

/// Export project to QASM format
pub fn export_qasm(manifest_path: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    use crate::project::qasm_export;
    
    let project = Project::load(manifest_path)?;
    println!("Info: Exporting project to QASM...");
    
    // Load the program
    let program = project.load_program()?;
    
    // Generate QASM
    let qasm = qasm_export::export_to_qasm(&program)?;
    
    // Determine output path
    let output_path = if let Some(path) = output {
        path
    } else {
        project.root().join("output").join("circuit.qasm")
    };
    
    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    // Write QASM to file
    fs::write(&output_path, &qasm)
        .with_context(|| format!("Failed to write QASM file: {}", output_path.display()))?;
    
    println!("Success: Exported QASM to {}", output_path.display());
    
    Ok(())
}

/// Export project to Python script that runs QASM via Qiskit
pub fn export_python(manifest_path: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    use crate::project::python_export;
    
    let project = Project::load(manifest_path)?;
    println!("Info: Exporting Python runner...");
    
    // Determine QASM path (default or user-specified)
    let qasm_path = project.root().join("output").join("circuit.qasm");
    
    // Ensure QASM file exists or generate it
    if !qasm_path.exists() {
        println!("Info: QASM file not found, generating it first...");
        export_qasm(Some(project.root().join("Aeonmi.toml")), None)?;
    }
    
    // Determine Python output path
    let output_path = if let Some(path) = output {
        path
    } else {
        project.root().join("output").join(format!("{}_runner.py", project.package_name()))
    };
    
    // Generate Python script
    python_export::export_python_runner(&qasm_path, &output_path, project.package_name())?;
    
    println!("Success: Exported Python runner to {}", output_path.display());
    println!();
    println!("To execute the quantum circuit:");
    println!("  python {}", output_path.display());
    
    Ok(())
}
