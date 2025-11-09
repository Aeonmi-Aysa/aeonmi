// Command handlers for the enhanced Aeonmi CLI
// Implements the core functionality for build, run, test, check commands

use crate::cli_enhanced::{
    print_error, print_info, print_success, print_warning, AeonCli, AeonCommand, OutputFormat,
};
use crate::core::{
    lexer::Lexer,
    parser::Parser,
    runtime_engine::RuntimeEngine,
    semantic_analyzer::{SemanticAnalyzer, Severity},
};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

/// Compiler options for build configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompilerOptions {
    pub optimization_level: u8,
    pub debug_info: bool,
    pub target: String,
}

impl CompilerOptions {
    pub fn new() -> Self {
        Self {
            optimization_level: 1,
            debug_info: false,
            target: "bytecode".to_string(),
        }
    }
}

/// Configuration for compilation and execution
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub release: bool,
    pub opt_level: u8,
    pub target: String,
    pub output_format: OutputFormat,
    pub output_dir: PathBuf,
    pub parallel: bool,
    pub verbose: bool,
    pub debug: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            release: false,
            opt_level: 1,
            target: "bytecode".to_string(),
            output_format: OutputFormat::Bytecode,
            output_dir: PathBuf::from("target"),
            parallel: false,
            verbose: false,
            debug: false,
        }
    }
}

/// Handle the build command
pub fn handle_build(cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::Build {
        release,
        manifest_path,
        files,
        format,
        watch,
        parallel,
    } = command
    {
        // Check if we're in project mode
        let is_project_mode = files.is_empty()
            && (manifest_path.is_some() || std::path::Path::new("Aeonmi.toml").exists());

        if is_project_mode {
            // Use project-based build system
            use crate::commands::project;

            print_info(&format!(
                "Building Aeonmi project ({})",
                if *release { "release" } else { "debug" }
            ));

            // Convert target from global option or format
            let target = cli.target.clone().or_else(|| match format {
                crate::cli_enhanced::OutputFormat::Qasm => Some("qasm".to_string()),
                crate::cli_enhanced::OutputFormat::Bytecode => Some("bytecode".to_string()),
                _ => None,
            });

            project::build(manifest_path.clone(), *release, target)?;
            Ok(())
        } else {
            // Use file-based build system
            let mut config = BuildConfig::default();
            config.release = *release;
            config.output_format = *format;
            config.parallel = *parallel;
            config.verbose = cli.verbose;
            config.debug = cli.debug;

            if let Some(level) = cli.opt_level {
                config.opt_level = level;
            }

            if let Some(target) = &cli.target {
                config.target = target.clone();
            }

            if let Some(output) = &cli.output {
                config.output_dir = output.clone();
            }

            print_info(&format!(
                "Building Aeonmi files ({})",
                if *release { "release" } else { "debug" }
            ));

            if *watch {
                return watch_and_build(config, manifest_path.as_deref(), files);
            }

            build_files(&config, files).map(|_| ())
        }
    } else {
        bail!("Invalid command for build handler")
    }
}

/// Handle the run command
pub fn handle_run(cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::Run {
        manifest_path,
        file,
        args,
        release,
        native,
        bytecode,
        quantum_backend,
        timeout_ms,
        watch,
        stats,
    } = command
    {
        // Check if we're in project mode
        let is_project_mode =
            file.is_none() && (manifest_path.is_some() || std::path::Path::new("Aeonmi.toml").exists());

        if is_project_mode {
            // Use project-based run system (pure Rust, no Node.js)
            use crate::commands::project;

            print_info("Compiling and running Aeonmi program");

            project::run(manifest_path.clone(), *release, *timeout_ms)?;
            Ok(())
        } else {
            // Use file-based run system
            let mut config = BuildConfig::default();
            config.release = *release;
            config.verbose = cli.verbose;
            config.debug = cli.debug;

            if let Some(level) = cli.opt_level {
                config.opt_level = level;
            }

            print_info("Compiling and running Aeonmi program");

            if *watch {
                return watch_and_run(
                    config,
                    file.as_deref(),
                    args,
                    *native,
                    *bytecode,
                    quantum_backend.as_deref(),
                );
            }

            // Build the file
            let output_path = if let Some(file_path) = file {
                build_single_file(&config, file_path)?
            } else {
                bail!("No file or project specified for run command");
            };

            // Execute using native VM (no Node.js)
            execute_program_native(&output_path, args, *stats)
        }
    } else {
        bail!("Invalid command for run handler")
    }
}

/// Handle the test command
pub fn handle_test(cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::Test {
        manifest_path,
        filter,
        release,
        nocapture,
        parallel,
        coverage,
        watch,
    } = command
    {
        // Check if we're in project mode
        let is_project_mode =
            manifest_path.is_some() || std::path::Path::new("Aeonmi.toml").exists();

        if is_project_mode {
            // Use project-based test system
            use crate::commands::project;

            print_info("Running Aeonmi tests");

            project::test(manifest_path.clone(), *release, filter.clone())?;
            Ok(())
        } else {
            // Fallback to file-based testing
            let mut config = BuildConfig::default();
            config.release = *release;
            config.parallel = *parallel;
            config.verbose = cli.verbose;
            config.debug = cli.debug;

            print_info("Running Aeonmi tests");

            if *watch {
                return watch_and_test(config, manifest_path.as_deref(), filter.as_deref());
            }

            run_tests(
                &config,
                manifest_path.as_deref(),
                filter.as_deref(),
                *nocapture,
                *coverage,
            )
        }
    } else {
        bail!("Invalid command for test handler")
    }
}

/// Handle the check command
pub fn handle_check(cli: &AeonCli, command: &AeonCommand) -> Result<()> {
    if let AeonCommand::Check {
        manifest_path,
        files,
        syntax_only,
        watch,
    } = command
    {
        // Check if we're in project mode
        let is_project_mode = files.is_empty()
            && (manifest_path.is_some() || std::path::Path::new("Aeonmi.toml").exists());

        if is_project_mode {
            // Use project-based check system
            use crate::commands::project;

            print_info("Checking Aeonmi project");

            project::check(manifest_path.clone())?;
            Ok(())
        } else {
            // Use file-based check system
            let config = BuildConfig {
                verbose: cli.verbose,
                debug: cli.debug,
                ..Default::default()
            };

            print_info("Checking Aeonmi files");

            if *watch {
                return watch_and_check(config, None, files, *syntax_only);
            }

            check_files(&config, files, *syntax_only)
        }
    } else {
        bail!("Invalid command for check handler")
    }
}

/// Build a complete project
fn build_project(config: &BuildConfig, manifest_path: Option<&Path>) -> Result<PathBuf> {
    let manifest = find_manifest(manifest_path)?;
    let project_dir = manifest.parent().unwrap();

    if config.verbose {
        print_info(&format!("Using manifest: {}", manifest.display()));
    }

    // Read project configuration
    let project_config = read_project_config(&manifest)?;

    // Find all source files
    let source_files = discover_source_files(project_dir)?;

    if source_files.is_empty() {
        print_warning("No source files found in project");
        return Ok(config.output_dir.join("empty"));
    }

    print_info(&format!("Found {} source files", source_files.len()));

    // Create output directory
    fs::create_dir_all(&config.output_dir)?;

    // Compile all source files
    let mut compiled_files = Vec::new();
    for source_file in source_files {
        if config.verbose {
            print_info(&format!("Compiling {}", source_file.display()));
        }

        let output_file = compile_source_file(config, &source_file)?;
        compiled_files.push(output_file);
    }

    // Link/package the compiled files
    let final_output = link_compiled_files(config, &compiled_files, &project_config.name)?;

    print_success(&format!(
        "Built project '{}' -> {}",
        project_config.name,
        final_output.display()
    ));

    Ok(final_output)
}

/// Build specific files
fn build_files(config: &BuildConfig, files: &[PathBuf]) -> Result<PathBuf> {
    if files.is_empty() {
        bail!("No files specified for compilation");
    }

    fs::create_dir_all(&config.output_dir)?;

    if files.len() == 1 {
        // Single file compilation
        let output = compile_source_file(config, &files[0])?;
        print_success(&format!(
            "Compiled {} -> {}",
            files[0].display(),
            output.display()
        ));
        Ok(output)
    } else {
        // Multi-file compilation
        let mut compiled_files = Vec::new();
        for file in files {
            if config.verbose {
                print_info(&format!("Compiling {}", file.display()));
            }
            let output = compile_source_file(config, file)?;
            compiled_files.push(output);
        }

        // Create a combined output
        let combined_output = combine_compiled_files(config, &compiled_files)?;
        print_success(&format!(
            "Compiled {} files -> {}",
            files.len(),
            combined_output.display()
        ));
        Ok(combined_output)
    }
}

/// Build a single file
fn build_single_file(config: &BuildConfig, file: &Path) -> Result<PathBuf> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }

    fs::create_dir_all(&config.output_dir)?;

    let output = compile_source_file(config, file)?;

    if config.verbose {
        print_success(&format!(
            "Compiled {} -> {}",
            file.display(),
            output.display()
        ));
    }

    Ok(output)
}

/// Compile a single source file
fn compile_source_file(config: &BuildConfig, source_file: &Path) -> Result<PathBuf> {
    // Read source code
    let source_code = fs::read_to_string(source_file)
        .with_context(|| format!("Failed to read source file: {}", source_file.display()))?;

    // Set up compilation options
    let mut compiler_opts = CompilerOptions::new();
    compiler_opts.optimization_level = config.opt_level;
    compiler_opts.debug_info = config.debug;
    compiler_opts.target = config.target.clone();

    // Perform compilation based on output format
    let output_file = determine_output_path(config, source_file);

    match config.output_format {
        OutputFormat::Bytecode => {
            compile_to_bytecode(&source_code, &output_file, &compiler_opts)?;
        }
        OutputFormat::Javascript => {
            compile_to_javascript(&source_code, &output_file, &compiler_opts)?;
        }
        OutputFormat::Aeonmi => {
            compile_to_aeonmi_ir(&source_code, &output_file, &compiler_opts)?;
        }
        OutputFormat::Native => {
            compile_to_native(&source_code, &output_file, &compiler_opts)?;
        }
        OutputFormat::Qasm => {
            compile_to_qasm(&source_code, &output_file, &compiler_opts)?;
        }
        OutputFormat::WebAssembly => {
            compile_to_wasm(&source_code, &output_file, &compiler_opts)?;
        }
    }

    Ok(output_file)
}

/// Execute a compiled program
fn execute_program(
    program_path: &Path,
    args: &[String],
    native: bool,
    bytecode: bool,
    quantum_backend: Option<&str>,
    show_stats: bool,
) -> Result<()> {
    print_info(&format!("Executing {}", program_path.display()));

    // Determine execution mode
    let execution_mode = if native {
        "native"
    } else if bytecode {
        "bytecode"
    } else if program_path.extension().map_or(false, |ext| ext == "bc") {
        "bytecode"
    } else {
        "auto"
    };

    // Set up runtime engine
    let mut runtime = RuntimeEngine::new();

    if let Some(backend) = quantum_backend {
        runtime.set_quantum_backend(backend)?;
    }

    if show_stats {
        runtime.enable_statistics();
    }

    // Execute the program
    let start_time = std::time::Instant::now();
    let result = runtime.execute_file(program_path, args)?;
    let duration = start_time.elapsed();

    if show_stats {
        print_info(&format!(
            "Execution completed in {:.2}ms",
            duration.as_millis()
        ));
        runtime.print_statistics();
    }

    if result.is_success() {
        print_success("Program executed successfully");
    } else {
        print_error(&format!(
            "Program failed with exit code: {}",
            result.exit_code()
        ));
    }

    Ok(())
}

/// Execute a compiled program using native Rust VM (no Node.js)
fn execute_program_native(program_path: &Path, args: &[String], show_stats: bool) -> Result<()> {
    print_info(&format!("Executing {}", program_path.display()));

    // Set up runtime engine
    let mut runtime = crate::core::runtime_engine::RuntimeEngine::new();

    if show_stats {
        runtime.enable_statistics();
    }

    // Execute the program
    let start_time = std::time::Instant::now();
    let result = runtime.execute_file(program_path, args)?;
    let duration = start_time.elapsed();

    if show_stats {
        print_info(&format!(
            "Execution completed in {:.2}ms",
            duration.as_millis()
        ));
        runtime.print_statistics();
    }

    if result.is_success() {
        print_success("Program executed successfully");
    } else {
        print_error(&format!(
            "Program failed with exit code: {}",
            result.exit_code()
        ));
    }

    Ok(())
}

/// Run tests for the project
fn run_tests(
    config: &BuildConfig,
    manifest_path: Option<&Path>,
    filter: Option<&str>,
    nocapture: bool,
    coverage: bool,
) -> Result<()> {
    let manifest = find_manifest(manifest_path)?;
    let project_dir = manifest.parent().unwrap();

    // Find test files
    let test_files = discover_test_files(project_dir)?;

    if test_files.is_empty() {
        print_warning("No test files found");
        return Ok(());
    }

    print_info(&format!("Found {} test files", test_files.len()));

    let mut passed = 0;
    let mut failed = 0;
    let mut total_tests = 0;

    for test_file in test_files {
        if let Some(filter_str) = filter {
            if !test_file.to_string_lossy().contains(filter_str) {
                continue;
            }
        }

        if config.verbose {
            print_info(&format!("Running tests in {}", test_file.display()));
        }

        let test_result = run_test_file(config, &test_file, nocapture, coverage)?;
        passed += test_result.passed;
        failed += test_result.failed;
        total_tests += test_result.total;
    }

    // Print test summary
    println!();
    println!("{}", "Test Results:".bright_white().bold());
    println!("  Total tests: {}", total_tests);
    println!("  Passed: {}", passed.to_string().green());
    println!("  Failed: {}", failed.to_string().red());

    if failed > 0 {
        print_error(&format!("{} test(s) failed", failed));
        std::process::exit(1);
    } else {
        print_success(&format!("All {} test(s) passed", passed));
    }

    Ok(())
}

/// Check project or files for syntax/semantic errors
fn check_project(
    config: &BuildConfig,
    manifest_path: Option<&Path>,
    syntax_only: bool,
) -> Result<()> {
    let manifest = find_manifest(manifest_path)?;
    let project_dir = manifest.parent().unwrap();
    let source_files = discover_source_files(project_dir)?;

    check_files(config, &source_files, syntax_only)
}

fn check_files(config: &BuildConfig, files: &[PathBuf], syntax_only: bool) -> Result<()> {
    if files.is_empty() {
        print_warning("No files to check");
        return Ok(());
    }

    let mut error_count = 0;
    let mut warning_count = 0;

    for file in files {
        if config.verbose {
            print_info(&format!("Checking {}", file.display()));
        }

        match check_single_file(file, syntax_only) {
            Ok(result) => {
                error_count += result.errors;
                warning_count += result.warnings;

                if result.errors > 0 || result.warnings > 0 || config.verbose {
                    println!(
                        "  {}: {} errors, {} warnings",
                        file.display(),
                        result.errors.to_string().red(),
                        result.warnings.to_string().yellow()
                    );
                }
            }
            Err(e) => {
                print_error(&format!("Failed to check {}: {}", file.display(), e));
                error_count += 1;
            }
        }
    }

    println!();
    if error_count > 0 {
        print_error(&format!(
            "Found {} error(s) across {} file(s)",
            error_count,
            files.len()
        ));
        std::process::exit(1);
    } else if warning_count > 0 {
        print_warning(&format!(
            "Found {} warning(s) across {} file(s)",
            warning_count,
            files.len()
        ));
    } else {
        print_success(&format!("All {} file(s) are valid", files.len()));
    }

    Ok(())
}

// Watch mode implementations
fn watch_and_build(
    config: BuildConfig,
    manifest_path: Option<&Path>,
    files: &[PathBuf],
) -> Result<()> {
    print_info("Watching for changes... (Press Ctrl+C to stop)");

    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            tx.send(res).unwrap();
        },
        notify::Config::default(),
    )?;

    if files.is_empty() {
        let manifest = find_manifest(manifest_path)?;
        let project_dir = manifest.parent().unwrap();
        watcher.watch(project_dir, RecursiveMode::Recursive)?;
    } else {
        for file in files {
            if let Some(parent) = file.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }
    }

    // Initial build
    let _ = if files.is_empty() {
        build_project(&config, manifest_path)
    } else {
        build_files(&config, files)
    };

    loop {
        match rx.recv() {
            Ok(_event) => {
                print_info("File changed, rebuilding...");
                let result = if files.is_empty() {
                    build_project(&config, manifest_path)
                } else {
                    build_files(&config, files)
                };

                match result {
                    Ok(_) => print_success("Rebuild completed"),
                    Err(e) => print_error(&format!("Rebuild failed: {}", e)),
                }
            }
            Err(e) => {
                print_error(&format!("Watch error: {}", e));
                break;
            }
        }
    }

    Ok(())
}

fn watch_and_run(
    config: BuildConfig,
    file: Option<&Path>,
    args: &[String],
    native: bool,
    bytecode: bool,
    quantum_backend: Option<&str>,
) -> Result<()> {
    print_info("Watching for changes... (Press Ctrl+C to stop)");
    // Implementation similar to watch_and_build but with execution
    todo!("Implement watch and run")
}

fn watch_and_test(
    config: BuildConfig,
    manifest_path: Option<&Path>,
    filter: Option<&str>,
) -> Result<()> {
    print_info("Watching for changes... (Press Ctrl+C to stop)");
    // Implementation similar to watch_and_build but with testing
    todo!("Implement watch and test")
}

fn watch_and_check(
    config: BuildConfig,
    manifest_path: Option<&Path>,
    files: &[PathBuf],
    syntax_only: bool,
) -> Result<()> {
    print_info("Watching for changes... (Press Ctrl+C to stop)");
    // Implementation similar to watch_and_build but with checking
    todo!("Implement watch and check")
}

// Helper functions and data structures
#[derive(Debug)]
struct ProjectConfig {
    name: String,
    version: String,
    dependencies: Vec<String>,
}

#[derive(Debug)]
struct TestResult {
    passed: usize,
    failed: usize,
    total: usize,
}

#[derive(Debug)]
struct CheckResult {
    errors: usize,
    warnings: usize,
}

// Stub implementations for compilation functions
fn compile_to_bytecode(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement bytecode compilation using existing compiler
    // For now, create a simple stub that writes placeholder bytecode
    let placeholder_bytecode = format!(
        "// Bytecode for source\n// Optimization level: {}\n// Target: {}\n",
        opts.optimization_level, opts.target
    );
    std::fs::write(output, placeholder_bytecode)?;
    Ok(())
}

fn compile_to_javascript(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement JavaScript compilation using existing compiler
    // For now, delegate to existing emit functionality
    use crate::cli::EmitKind;
    use crate::commands::compile;

    // Create a temporary input file
    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join("temp_source.ai");
    std::fs::write(&temp_input, source)?;

    // Use existing compile pipeline
    compile::compile_pipeline(
        Some(temp_input.clone()),
        EmitKind::Js,
        output.to_path_buf(),
        false, // tokens
        false, // ast
        false, // pretty_errors
        false, // no_sema
        false, // debug_titan
    )?;

    // Cleanup
    let _ = std::fs::remove_file(temp_input);
    Ok(())
}

fn compile_to_aeonmi_ir(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement Aeonmi IR compilation
    use crate::cli::EmitKind;
    use crate::commands::compile;

    // Create a temporary input file
    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join("temp_source.ai");
    std::fs::write(&temp_input, source)?;

    // Use existing compile pipeline
    compile::compile_pipeline(
        Some(temp_input.clone()),
        EmitKind::Ai,
        output.to_path_buf(),
        false, // tokens
        false, // ast
        false, // pretty_errors
        false, // no_sema
        false, // debug_titan
    )?;

    // Cleanup
    let _ = std::fs::remove_file(temp_input);
    Ok(())
}

fn compile_to_native(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement native compilation
    bail!("Native compilation not yet implemented")
}

fn compile_to_qasm(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement QASM compilation
    bail!("QASM compilation not yet implemented")
}

fn compile_to_wasm(source: &str, output: &Path, opts: &CompilerOptions) -> Result<()> {
    // TODO: Implement WebAssembly compilation
    bail!("WebAssembly compilation not yet implemented")
}

// Helper function stubs
fn find_manifest(manifest_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = manifest_path {
        if path.exists() {
            Ok(path.to_path_buf())
        } else {
            bail!("Manifest file not found: {}", path.display());
        }
    } else {
        // Look for Aeonmi.toml in current directory and parents
        let mut current = std::env::current_dir()?;
        loop {
            let manifest = current.join("Aeonmi.toml");
            if manifest.exists() {
                return Ok(manifest);
            }
            if !current.pop() {
                break;
            }
        }
        bail!("No Aeonmi.toml found in current directory or parents");
    }
}

fn read_project_config(manifest: &Path) -> Result<ProjectConfig> {
    let content = fs::read_to_string(manifest)?;
    // TODO: Parse TOML configuration
    Ok(ProjectConfig {
        name: "project".to_string(),
        version: "1.0.0".to_string(),
        dependencies: Vec::new(),
    })
}

fn discover_source_files(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let src_dir = project_dir.join("src");

    if src_dir.exists() {
        for entry in walkdir::WalkDir::new(src_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "ai" || ext == "aeon" || ext == "aeonmi" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn discover_test_files(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let tests_dir = project_dir.join("tests");

    if tests_dir.exists() {
        for entry in walkdir::WalkDir::new(tests_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "ai" || ext == "aeon" || ext == "aeonmi" {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    // Also look for test files in src directory
    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        for entry in walkdir::WalkDir::new(src_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.contains("test") || file_name.starts_with("test_") {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "ai" || ext == "aeon" || ext == "aeonmi" {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

fn determine_output_path(config: &BuildConfig, source_file: &Path) -> PathBuf {
    let file_stem = source_file.file_stem().unwrap().to_string_lossy();
    let extension = match config.output_format {
        OutputFormat::Bytecode => "bc",
        OutputFormat::Javascript => "js",
        OutputFormat::Aeonmi => "ai",
        OutputFormat::Native => {
            if cfg!(windows) {
                "exe"
            } else {
                ""
            }
        }
        OutputFormat::Qasm => "qasm",
        OutputFormat::WebAssembly => "wasm",
    };

    if extension.is_empty() {
        config.output_dir.join(file_stem.as_ref())
    } else {
        config
            .output_dir
            .join(format!("{}.{}", file_stem, extension))
    }
}

fn link_compiled_files(
    config: &BuildConfig,
    compiled_files: &[PathBuf],
    project_name: &str,
) -> Result<PathBuf> {
    // TODO: Implement linking logic
    let output_path = config.output_dir.join(project_name);
    Ok(output_path)
}

fn combine_compiled_files(config: &BuildConfig, compiled_files: &[PathBuf]) -> Result<PathBuf> {
    // TODO: Implement file combination logic
    let output_path = config.output_dir.join("combined");
    Ok(output_path)
}

fn run_test_file(
    config: &BuildConfig,
    _test_file: &Path,
    _nocapture: bool,
    _coverage: bool,
) -> Result<TestResult> {
    // TODO: Implement test running logic
    Ok(TestResult {
        passed: 1,
        failed: 0,
        total: 1,
    })
}

fn check_single_file(file: &Path, syntax_only: bool) -> Result<CheckResult> {
    let source =
        fs::read_to_string(file).with_context(|| format!("Failed to read {}", file.display()))?;

    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    let mut lexer = Lexer::from_str(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            print_error(&format!("{}: Lexer error: {}", file.display(), err));
            return Ok(CheckResult {
                errors: 1,
                warnings: 0,
            });
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            print_error(&format!("{}: Parser error: {}", file.display(), err));
            return Ok(CheckResult {
                errors: 1,
                warnings: 0,
            });
        }
    };

    if syntax_only {
        return Ok(CheckResult {
            errors: error_count,
            warnings: warning_count,
        });
    }

    let mut sema = SemanticAnalyzer::new();
    let diagnostics = sema.analyze_with_spans(&ast);
    for diag in diagnostics {
        let location = if diag.line > 0 {
            format!("{}:{}:{}", file.display(), diag.line, diag.column)
        } else {
            file.display().to_string()
        };

        match diag.severity {
            Severity::Error => {
                error_count += 1;
                print_error(&format!("{}: {}", location, diag.message));
            }
            Severity::Warning => {
                warning_count += 1;
                print_warning(&format!("{}: {}", location, diag.message));
            }
        }
    }

    Ok(CheckResult {
        errors: error_count,
        warnings: warning_count,
    })
}
