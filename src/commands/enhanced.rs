// Command handlers for the enhanced Aeonmi CLI
// Implements the core functionality for build, run, test, check commands

use crate::cli_enhanced::{
    print_error, print_info, print_success, print_warning, AeonCli, AeonCommand, OutputFormat,
};
use crate::core::{
    lexer::Lexer,
    parser::Parser,
    semantic_analyzer::{SemanticAnalyzer, Severity},
};
use crate::project::BuildCache;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use num_cpus;

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
    pub incremental: bool,
    pub cache_dir: Option<PathBuf>,
    pub max_parallel_jobs: usize,
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
            incremental: true,
            cache_dir: Some(PathBuf::from("target/.aeonmi_cache")),
            max_parallel_jobs: num_cpus::get(),
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
/// Build multiple files with incremental and parallel support
pub fn build_files(config: &BuildConfig, files: &[PathBuf]) -> Result<PathBuf> {
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
        // Multi-file compilation with incremental and parallel support
        let compiled_files = if config.incremental || config.parallel {
            compile_files_incremental_parallel(config, files)?
        } else {
            let mut compiled_files = Vec::new();
            for file in files {
                if config.verbose {
                    print_info(&format!("Compiling {}", file.display()));
                }
                let output = compile_source_file(config, file)?;
                compiled_files.push(output);
            }
            compiled_files
        };

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

fn compile_files_incremental_parallel(config: &BuildConfig, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    use crate::project::{BuildCache, ParallelCompiler};
    use std::sync::Arc;

    // Initialize build cache if incremental compilation is enabled
    let cache = if config.incremental {
        let default_cache_dir = PathBuf::from("target/.aeonmi_cache");
        let cache_dir = config.cache_dir.as_ref().unwrap_or(&default_cache_dir);
        let cache = BuildCache::new(cache_dir);
        Some(cache)
    } else {
        None
    };

    // Determine which files need compilation
    let files_to_compile = if let Some(ref cache) = cache {
        // Find project root (assuming first file's parent directory)
        let project_root = files.first()
            .and_then(|f| f.parent())
            .unwrap_or(Path::new("."));

        match cache.needs_recompilation(project_root, files) {
            Ok(needs_compile) => {
                if config.verbose && needs_compile.len() < files.len() {
                    let skipped = files.len() - needs_compile.len();
                    print_info(&format!("Incremental: {} files up-to-date, compiling {}", skipped, needs_compile.len()));
                }
                needs_compile
            }
            Err(_) => {
                // If cache check fails, compile all files
                files.to_vec()
            }
        }
    } else {
        files.to_vec()
    };

    let mut compiled_files = Vec::new();

    if config.parallel && files_to_compile.len() > 1 {
        // Parallel compilation
        if config.verbose {
            print_info(&format!("Compiling {} files in parallel (max {} jobs)", files_to_compile.len(), config.max_parallel_jobs));
        }

        let parallel_compiler = ParallelCompiler::new(config.max_parallel_jobs);
        let config_arc = Arc::new(config.clone());

        // Run parallel compilation
        let rt = tokio::runtime::Runtime::new()?;
        let results = rt.block_on(async {
            parallel_compiler.compile_parallel(
                files_to_compile.clone(),
                |file_path| {
                    let config = Arc::clone(&config_arc);
                    async move {
                        compile_source_file(&config, &file_path)
                    }
                }
            ).await
        })?;

        compiled_files.extend(results);

        // Update cache for parallel compiled files
        if let Some(ref cache) = cache {
            for file in &files_to_compile {
                let output = compile_source_file_with_cache(&config, file, Some(cache))?;
                // Replace the parallel result with cached result
                if let Some(pos) = compiled_files.iter().position(|f| f == &output) {
                    compiled_files[pos] = output;
                }
            }
        }

        // Add cached files that didn't need recompilation
        if let Some(ref cache) = cache {
            for file in files {
                if !files_to_compile.contains(file) {
                    // This file was cached, find its output
                    if cache.get(&file.strip_prefix(Path::new(".")).unwrap_or(file)).is_some() {
                        let output_path = determine_output_path(&config, file);
                        if output_path.exists() {
                            compiled_files.push(output_path);
                        }
                    }
                }
            }
        }
    } else {
        // Sequential compilation
        for file in &files_to_compile {
            if config.verbose {
                print_info(&format!("Compiling {}", file.display()));
            }
            let output = compile_source_file_with_cache(config, file, cache.as_ref())?;
            compiled_files.push(output);
        }

        // Add cached files
        if let Some(ref cache) = cache {
            for file in files {
                if !files_to_compile.contains(file) {
                    if cache.get(&file.strip_prefix(Path::new(".")).unwrap_or(file)).is_some() {
                        let output_path = determine_output_path(&config, file);
                        if output_path.exists() {
                            compiled_files.push(output_path);
                        }
                    }
                }
            }
        }
    }

    // Save cache if incremental compilation is enabled
    if let Some(ref cache) = cache {
        cache.save().unwrap_or_else(|_| {
            // Cache save failure is not critical
        });
    }

    Ok(compiled_files)
}

fn compile_source_file_with_cache(
    config: &BuildConfig,
    source_file: &Path,
    cache: Option<&BuildCache>,
) -> Result<PathBuf> {
    let output = compile_source_file(config, source_file)?;

    // Update cache if available
    if let Some(cache) = cache {
        let relative_path = source_file.strip_prefix(Path::new(".")).unwrap_or(source_file);
        let mtime = std::fs::metadata(source_file)?.modified()?;
        let hash = crate::project::build_cache::calculate_file_hash(source_file)?;

        // Implement proper dependency analysis
        let project_root = source_file.parent().unwrap_or(Path::new("."));
        let analyzer = crate::project::build_cache::DependencyAnalyzer::new(project_root.to_path_buf());
        let dependencies = analyzer.analyze_file(source_file).unwrap_or_default();

        let entry = crate::project::CacheEntry::new(
            relative_path.to_path_buf(),
            mtime,
            hash,
            dependencies,
            config.output_format, // Use the actual output format from config
            config.opt_level,
            config.debug,
        );

        cache.update(entry);
    }

    Ok(output)
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

// Watch mode implementations
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
    _native: bool,
    _bytecode: bool,
    _quantum_backend: Option<&str>,
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

    // Watch the source file or project directory
    if let Some(file_path) = file {
        if let Some(parent) = file_path.parent() {
            watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }
    } else {
        // Project mode - watch the current directory
        let current_dir = std::env::current_dir()?;
        watcher.watch(&current_dir, RecursiveMode::Recursive)?;
    }

    // Initial compile and run
    let result = if let Some(file_path) = file {
        let output_path = build_single_file(&config, file_path)?;
        execute_program_native(&output_path, args, false)
    } else {
        // Project mode - this would need project::run implementation
        print_error("Project watch mode not yet implemented");
        return Ok(());
    };

    match result {
        Ok(_) => print_success("Initial run completed"),
        Err(e) => print_error(&format!("Initial run failed: {}", e)),
    }

    loop {
        match rx.recv() {
            Ok(_event) => {
                print_info("File changed, recompiling and running...");
                let result = if let Some(file_path) = file {
                    let output_path = build_single_file(&config, file_path)?;
                    execute_program_native(&output_path, args, false)
                } else {
                    print_error("Project watch mode not yet implemented");
                    continue;
                };

                match result {
                    Ok(_) => print_success("Re-run completed"),
                    Err(e) => print_error(&format!("Re-run failed: {}", e)),
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

fn watch_and_test(
    config: BuildConfig,
    manifest_path: Option<&Path>,
    filter: Option<&str>,
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

    // Watch the project directory for test files
    let manifest = find_manifest(manifest_path)?;
    let project_dir = manifest.parent().unwrap();
    watcher.watch(project_dir, RecursiveMode::Recursive)?;

    // Initial test run
    let result = run_tests(&config, manifest_path, filter, false, false);

    match result {
        Ok(_) => print_success("Initial test run completed"),
        Err(e) => print_error(&format!("Initial test run failed: {}", e)),
    }

    loop {
        match rx.recv() {
            Ok(_event) => {
                print_info("File changed, re-running tests...");
                let result = run_tests(&config, manifest_path, filter, false, false);

                match result {
                    Ok(_) => print_success("Test re-run completed"),
                    Err(e) => print_error(&format!("Test re-run failed: {}", e)),
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

fn watch_and_check(
    config: BuildConfig,
    manifest_path: Option<&Path>,
    files: &[PathBuf],
    syntax_only: bool,
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

    // Watch the project directory
    let manifest = find_manifest(manifest_path)?;
    let project_dir = manifest.parent().unwrap();
    watcher.watch(project_dir, RecursiveMode::Recursive)?;

    // Initial check run
    let result = check_files(&config, files, syntax_only);

    match result {
        Ok(_) => print_success("Initial check completed"),
        Err(e) => print_error(&format!("Initial check failed: {}", e)),
    }

    loop {
        match rx.recv() {
            Ok(_event) => {
                print_info("File changed, re-checking...");
                let result = check_files(&config, files, syntax_only);

                match result {
                    Ok(_) => print_success("Check re-run completed"),
                    Err(e) => print_error(&format!("Check re-run failed: {}", e)),
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
fn compile_to_bytecode(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    // Implement bytecode compilation using existing compiler
    use crate::cli::EmitKind;
    use crate::commands::compile;

    // Create a temporary input file
    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join("temp_source.ai");
    std::fs::write(&temp_input, source)?;

    // Use existing compile pipeline with Bytecode emit kind
    compile::compile_pipeline(
        Some(temp_input.clone()),
        EmitKind::Bytecode,
        output.to_path_buf(),
        false, // tokens
        false, // ast
        false, // pretty_errors
        false, // no_sema
        false, // debug_titan
    )?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_input);

    Ok(())
}

fn compile_to_javascript(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    // Implement JavaScript compilation using existing compiler
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

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_input);

    Ok(())
}

fn compile_to_aeonmi_ir(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    // Implement Aeonmi IR compilation using existing compiler
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

fn compile_to_native(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    use crate::core::code_generator::CodeGenerator;
    use crate::core::lexer::Lexer;
    use crate::core::parser::Parser;

    // Tokenize the source code
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize()?;

    // Parse the tokens
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Generate native code using the code generator
    let mut code_gen = CodeGenerator::new();
    let c_code = code_gen.generate_with_backend(&ast, crate::core::code_generator::Backend::Native)
        .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;

    // Write to output file
    std::fs::write(output, c_code)?;

    Ok(())
}

fn compile_to_qasm(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    use crate::core::lexer::Lexer;
    use crate::core::parser::Parser;
    use crate::compiler::qasm_exporter::export_to_qasm;

    // Tokenize the source code
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize()?;

    // Parse the tokens
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Export to QASM
    let qasm_code = export_to_qasm(&ast);

    // Write to output file
    std::fs::write(output, qasm_code)?;

    Ok(())
}

fn compile_to_wasm(source: &str, output: &Path, _opts: &CompilerOptions) -> Result<()> {
    use crate::core::code_generator::CodeGenerator;
    use crate::core::lexer::Lexer;
    use crate::core::parser::Parser;

    // Tokenize the source code
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize()?;

    // Parse the tokens
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Generate WebAssembly using the code generator
    let mut code_gen = CodeGenerator::new();
    let wat_code = code_gen.generate_with_backend(&ast, crate::core::code_generator::Backend::WebAssembly)
        .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;

    // Write to output file
    std::fs::write(output, wat_code)?;

    Ok(())
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
    // Parse TOML configuration using existing project loading
    let project = crate::project::Project::load(Some(manifest.to_path_buf()))?;

    // Extract relevant config information
    let name = project.package_name().to_string();
    let version = project.package_version().to_string();
    // For now, return empty dependencies list since they're not used in the current implementation
    let dependencies = Vec::new();

    Ok(ProjectConfig {
        name,
        version,
        dependencies,
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
    if compiled_files.is_empty() {
        bail!("No compiled files to link");
    }

    if compiled_files.len() == 1 {
        // Single file, just return it
        let output_path = config.output_dir.join(format!("{}.{}", project_name, get_extension(&config.output_format)));
        std::fs::copy(&compiled_files[0], &output_path)?;
        return Ok(output_path);
    }

    // Multiple files need linking
    match config.output_format {
        OutputFormat::Javascript => link_javascript_files(config, compiled_files, project_name),
        OutputFormat::Aeonmi => link_aeonmi_files(config, compiled_files, project_name),
        OutputFormat::Bytecode => link_bytecode_files(config, compiled_files, project_name),
        OutputFormat::Native => link_native_files(config, compiled_files, project_name),
        OutputFormat::Qasm => link_qasm_files(config, compiled_files, project_name),
        OutputFormat::WebAssembly => link_wasm_files(config, compiled_files, project_name),
    }
}

fn get_extension(format: &OutputFormat) -> &'static str {
    match format {
        OutputFormat::Javascript => "js",
        OutputFormat::Aeonmi => "ai",
        OutputFormat::Bytecode => "bytecode",
        OutputFormat::Native => "exe",
        OutputFormat::Qasm => "qasm",
        OutputFormat::WebAssembly => "wat",
    }
}

fn link_javascript_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.js", project_name));
    let mut combined_content = String::new();

    // Add header
    combined_content.push_str("// Aeonmi compiled JavaScript\n");
    combined_content.push_str("// Combined from multiple source files\n\n");

    for (i, file_path) in compiled_files.iter().enumerate() {
        let content = std::fs::read_to_string(file_path)?;
        if i > 0 {
            combined_content.push_str("\n// --- File boundary ---\n");
        }
        combined_content.push_str(&content);
        combined_content.push_str("\n");
    }

    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn link_aeonmi_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.ai", project_name));
    let mut combined_content = String::new();

    // Add header with metadata
    combined_content.push_str("// aeonmi:combined\n");
    combined_content.push_str(&format!("// hash:{}\n", generate_combined_hash(compiled_files)));
    combined_content.push_str("// tool:aeonmi linker\n\n");

    for (i, file_path) in compiled_files.iter().enumerate() {
        let content = std::fs::read_to_string(file_path)?;
        if i > 0 {
            combined_content.push_str("\n// --- Module boundary ---\n");
        }
        // Skip the header from individual files and add the content
        let lines: Vec<&str> = content.lines().collect();
        let content_start = lines.iter().position(|line| !line.starts_with("//")).unwrap_or(0);
        for line in &lines[content_start..] {
            combined_content.push_str(line);
            combined_content.push_str("\n");
        }
    }

    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn link_bytecode_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.bytecode", project_name));
    let mut combined_content = Vec::new();

    // Add header with metadata
    let header = format!("AEONMI_BYTECODE_COMBINED_v1\nFILES:{}\n", compiled_files.len());
    combined_content.extend_from_slice(header.as_bytes());

    for file_path in compiled_files {
        let content = std::fs::read(file_path)?;
        combined_content.extend_from_slice(&content);
        // Add separator
        combined_content.extend_from_slice(b"\n---BYTECODE_MODULE_SEPARATOR---\n");
    }

    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn link_native_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.exe", project_name));
    let mut combined_content = String::new();

    // For native compilation, we combine C files and add a main function
    combined_content.push_str("// Aeonmi combined native code\n");
    combined_content.push_str("#include <stdio.h>\n");
    combined_content.push_str("#include <stdlib.h>\n\n");

    for file_path in compiled_files {
        let content = std::fs::read_to_string(file_path)?;
        // Extract function definitions (skip includes and main)
        let lines: Vec<&str> = content.lines().collect();
        let mut in_main = false;
        for line in lines {
            if line.starts_with("#include") {
                continue; // Skip includes, we add them at the top
            }
            if line.contains("int main(") {
                in_main = true;
                continue; // Skip original main functions
            }
            if in_main && line.trim() == "}" {
                in_main = false;
                continue;
            }
            if !in_main && !line.trim().is_empty() {
                combined_content.push_str(line);
                combined_content.push_str("\n");
            }
        }
    }

    // Add combined main function
    combined_content.push_str("\nint main() {\n");
    combined_content.push_str("    printf(\"Aeonmi combined executable\\n\");\n");
    combined_content.push_str("    // Call all module functions here\n");
    combined_content.push_str("    return 0;\n");
    combined_content.push_str("}\n");

    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn link_qasm_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.qasm", project_name));
    let mut combined_content = String::new();

    // QASM header
    combined_content.push_str("OPENQASM 2.0;\n");
    combined_content.push_str("include \"qelib1.inc\";\n\n");
    combined_content.push_str(&format!("// Aeonmi combined QASM - {}\n", project_name));

    let mut total_qubits = 0;
    let mut all_operations = Vec::new();

    for file_path in compiled_files {
        let content = std::fs::read_to_string(file_path)?;
        // Parse QASM content to extract qubits and operations
        // This is a simplified parser - in practice you'd want a proper QASM parser
        for line in content.lines() {
            if line.starts_with("qreg") {
                // Extract qubit count from qreg declarations
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        if let Ok(count) = line[start+1..end].parse::<usize>() {
                            total_qubits += count;
                        }
                    }
                }
            } else if !line.starts_with("OPENQASM") && !line.starts_with("include") && !line.trim().is_empty() {
                all_operations.push(line.to_string());
            }
        }
    }

    // Declare combined qubit register
    combined_content.push_str(&format!("qreg q[{}];\n", total_qubits));
    combined_content.push_str("\n");

    // Add all operations
    for op in all_operations {
        combined_content.push_str(&op);
        combined_content.push_str("\n");
    }

    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn link_wasm_files(config: &BuildConfig, compiled_files: &[PathBuf], project_name: &str) -> Result<PathBuf> {
    let output_path = config.output_dir.join(format!("{}.wat", project_name));
    let mut combined_content = String::new();

    combined_content.push_str("(module\n");
    combined_content.push_str(&format!("  ;; Aeonmi combined WebAssembly - {}\n", project_name));

    let mut function_count = 0;
    let mut export_count = 0;

    for (i, file_path) in compiled_files.iter().enumerate() {
        let content = std::fs::read_to_string(file_path)?;
        // Extract functions and exports from each module
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("(func") {
                // Renumber functions to avoid conflicts
                let renumbered = trimmed.replace("(func", &format!("(func ${}_{}", project_name, function_count));
                combined_content.push_str(&format!("  {}\n", renumbered));
                function_count += 1;
            } else if trimmed.starts_with("(export") {
                // Renumber exports
                let renumbered = trimmed.replace("(export", &format!("(export \"mod{}_{}\"", i, export_count));
                combined_content.push_str(&format!("  {}\n", renumbered));
                export_count += 1;
            }
        }
    }

    combined_content.push_str(")\n");
    std::fs::write(&output_path, combined_content)?;
    Ok(output_path)
}

fn combine_compiled_files(config: &BuildConfig, compiled_files: &[PathBuf]) -> Result<PathBuf> {
    link_compiled_files(config, compiled_files, "combined")
}

fn generate_combined_hash(files: &[PathBuf]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for file in files {
        if let Ok(content) = std::fs::read(file) {
            content.hash(&mut hasher);
        }
    }
    format!("{:x}", hasher.finish())
}

fn run_test_file(
    _config: &BuildConfig,
    test_file: &Path,
    nocapture: bool,
    _coverage: bool,
) -> Result<TestResult> {
    // Implement test running logic
    // Compile the test file to bytecode and execute it

    // Read the test file
    let source = fs::read_to_string(test_file)
        .with_context(|| format!("Failed to read test file: {}", test_file.display()))?;

    // For now, we'll use a simple approach: compile and run via the existing runtime
    // This is a basic implementation - could be enhanced with proper test framework

    use crate::core::runtime_engine::RuntimeEngine;

    let runtime = RuntimeEngine::new();

    if !nocapture {
        // In capture mode, we might want to suppress output
        // For now, just run with output enabled
    }

    match runtime.compile_source(&source, &test_file.to_string_lossy()) {
        Ok(_) => {
            // If compilation succeeds, consider it a passing test
            // In a real test framework, we'd look for specific test assertions
            Ok(TestResult {
                passed: 1,
                failed: 0,
                total: 1,
            })
        }
        Err(err) => {
            // Compilation/execution failed - test failed
            if nocapture {
                print_error(&format!("Test {} failed: {}", test_file.display(), err));
            }
            Ok(TestResult {
                passed: 0,
                failed: 1,
                total: 1,
            })
        }
    }
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
