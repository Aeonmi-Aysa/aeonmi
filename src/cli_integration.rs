// Main CLI integration for Aeonmi - unified command dispatcher
// This module integrates the enhanced CLI with existing functionality

use crate::cli_enhanced::{run_cli as run_enhanced_cli, AeonCli, AeonCommand, SandboxCommand};
use crate::commands::enhanced::{handle_build, handle_check, handle_run, handle_test};
use crate::commands::project_init::{handle_init, handle_new};
use crate::sandbox::process_manager::{ProcessManager, ProcessStatus};
use anyhow::Result;
use colored::Colorize;

/// Main entry point for the enhanced CLI system
pub fn run_enhanced_aeon_cli() -> Result<()> {
    // Set console title for better UX
    set_console_title();

    // Check for legacy mode first
    let args: Vec<String> = std::env::args().collect();

    // If called with old-style arguments, provide migration message
    if should_show_migration_message(&args) {
        show_migration_message();
        std::process::exit(1);
    }

    // Run the enhanced CLI
    run_enhanced_cli()
}

/// Enhanced command dispatcher that delegates to appropriate handlers
pub fn dispatch_command(cli: AeonCli, command: AeonCommand) -> Result<()> {
    match command {
        // Core compilation and execution commands
        AeonCommand::Build { .. } => handle_build(&cli, &command),

        AeonCommand::Run { .. } => handle_run(&cli, &command),

        AeonCommand::Test { .. } => handle_test(&cli, &command),

        AeonCommand::Check { .. } => handle_check(&cli, &command),

        AeonCommand::Clean { manifest_path } => handle_clean(&cli, manifest_path),

        // Project creation and initialization
        AeonCommand::New { .. } => handle_new(&cli, &command),

        AeonCommand::Init { .. } => handle_init(&cli, &command),

        // Development and utility commands
        AeonCommand::Format {
            files,
            check,
            config_file,
        } => handle_format_command(&cli, &files, check, config_file.as_deref()),

        AeonCommand::Lint {
            files,
            fix,
            config_file,
        } => handle_lint_command(&cli, &files, fix, config_file.as_deref()),

        AeonCommand::Quantum { action } => handle_quantum_commands(&cli, &action),

        AeonCommand::Dev { action } => handle_dev_commands(&cli, &action),

        AeonCommand::Project { action } => handle_project_commands(&cli, &action),

        AeonCommand::Package { action } => handle_package_commands(&cli, &action),

        AeonCommand::Repl { backend, load } => {
            handle_repl_command(&cli, backend.as_deref(), load.as_deref())
        }

        AeonCommand::Bench {
            filter,
            save,
            compare,
        } => handle_bench_command(&cli, filter.as_deref(), save.as_deref(), compare.as_deref()),

        AeonCommand::Doc {
            open,
            private,
            target_dir,
        } => handle_doc_command(&cli, open, private, target_dir.as_deref()),

        AeonCommand::Lsp { action } => handle_lsp_commands(&cli, &action),

        AeonCommand::Config { action } => handle_config_commands(&cli, &action),

        AeonCommand::Editor {
            port,
            workspace,
            browser,
        } => handle_editor_command(&cli, port, workspace.as_deref(), browser),

        AeonCommand::Sandbox { action } => handle_sandbox_command(&cli, &action),

        AeonCommand::Version { verbose } => handle_version_command(&cli, verbose),
    }
}

/// Handle format command
fn handle_format_command(
    cli: &AeonCli,
    files: &[std::path::PathBuf],
    check: bool,
    config_file: Option<&std::path::Path>,
) -> Result<()> {
    use crate::cli_enhanced::{print_error, print_info, print_success};

    if check {
        print_info("Checking file formatting...");
    } else {
        print_info("Formatting source files...");
    }

    // If no files specified, find all .ai files in current project
    let target_files = if files.is_empty() {
        discover_source_files_in_project()?
    } else {
        files.to_vec()
    };

    if target_files.is_empty() {
        print_error("No source files found to format");
        return Ok(());
    }

    let mut formatted_count = 0;
    let mut error_count = 0;

    for file in target_files {
        if cli.verbose {
            print_info(&format!("Processing {}", file.display()));
        }

        match format_file(&file, check, config_file) {
            Ok(true) => {
                formatted_count += 1;
                if check {
                    println!("  {}: {}", file.display(), "needs formatting".yellow());
                } else {
                    println!("  {}: {}", file.display(), "formatted".green());
                }
            }
            Ok(false) => {
                if check && cli.verbose {
                    println!("  {}: {}", file.display(), "already formatted".green());
                }
            }
            Err(e) => {
                error_count += 1;
                println!("  {}: {}", file.display(), format!("error: {}", e).red());
            }
        }
    }

    if check {
        if formatted_count > 0 {
            print_error(&format!("{} file(s) need formatting", formatted_count));
            std::process::exit(1);
        } else {
            print_success("All files are properly formatted");
        }
    } else {
        if formatted_count > 0 {
            print_success(&format!("Formatted {} file(s)", formatted_count));
        } else {
            print_info("No files needed formatting");
        }
    }

    if error_count > 0 {
        print_error(&format!("{} file(s) had errors", error_count));
    }

    Ok(())
}

/// Handle lint command
fn handle_lint_command(
    cli: &AeonCli,
    files: &[std::path::PathBuf],
    fix: bool,
    config_file: Option<&std::path::Path>,
) -> Result<()> {
    use crate::cli_enhanced::{print_error, print_info, print_success, print_warning};

    if fix {
        print_info("Linting and fixing source files...");
    } else {
        print_info("Linting source files...");
    }

    let target_files = if files.is_empty() {
        discover_source_files_in_project()?
    } else {
        files.to_vec()
    };

    if target_files.is_empty() {
        print_error("No source files found to lint");
        return Ok(());
    }

    let mut total_issues = 0;
    let mut fixed_issues = 0;

    for file in target_files {
        if cli.verbose {
            print_info(&format!("Linting {}", file.display()));
        }

        match lint_file(&file, fix, config_file) {
            Ok(result) => {
                total_issues += result.issues.len();
                fixed_issues += result.fixed;

                if !result.issues.is_empty() || cli.verbose {
                    println!("  {}:", file.display());
                    for issue in result.issues {
                        let level_str = match issue.level {
                            LintLevel::Error => "error".red(),
                            LintLevel::Warning => "warning".yellow(),
                            LintLevel::Info => "info".blue(),
                        };
                        println!(
                            "    {}:{}: {} {}",
                            issue.line, issue.column, level_str, issue.message
                        );
                    }
                    if result.fixed > 0 {
                        println!("    {}", format!("Fixed {} issue(s)", result.fixed).green());
                    }
                }
            }
            Err(e) => {
                print_error(&format!("Failed to lint {}: {}", file.display(), e));
            }
        }
    }

    if total_issues > 0 {
        if fix && fixed_issues > 0 {
            print_success(&format!(
                "Fixed {} out of {} issue(s)",
                fixed_issues, total_issues
            ));
            if fixed_issues < total_issues {
                print_warning(&format!(
                    "{} issue(s) require manual attention",
                    total_issues - fixed_issues
                ));
            }
        } else {
            print_warning(&format!("Found {} issue(s)", total_issues));
        }
    } else {
        print_success("No linting issues found");
    }

    Ok(())
}

/// Handle clean command
fn handle_clean(_cli: &AeonCli, manifest_path: Option<PathBuf>) -> Result<()> {
    use crate::cli_enhanced::print_info;
    use crate::commands::project;

    print_info("Cleaning build artifacts");

    project::clean(manifest_path)?;

    Ok(())
}

/// Handle quantum-specific commands
fn handle_quantum_commands(
    cli: &AeonCli,
    action: &crate::cli_enhanced::QuantumCommand,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success, QuantumCommand};

    match action {
        QuantumCommand::Run {
            file,
            backend,
            shots,
            optimize,
        } => {
            print_info(&format!(
                "Executing quantum circuit {} on backend {}",
                file.display(),
                backend
            ));

            // Load and execute quantum circuit
            execute_quantum_circuit(file, backend, *shots, *optimize)?;
            print_success("Quantum execution completed");
        }

        QuantumCommand::Backends => {
            print_info("Available quantum backends:");
            list_quantum_backends()?;
        }

        QuantumCommand::Simulate {
            file,
            shots,
            statevector,
        } => {
            print_info(&format!("Simulating quantum circuit {}", file.display()));
            simulate_quantum_circuit(file, *shots, *statevector)?;
            print_success("Quantum simulation completed");
        }

        QuantumCommand::Visualize {
            file,
            format,
            output,
        } => {
            print_info(&format!("Visualizing quantum circuit {}", file.display()));
            visualize_quantum_circuit(file, format, output.as_deref())?;
            print_success("Quantum visualization generated");
        }
    }

    Ok(())
}

/// Handle development commands
fn handle_dev_commands(cli: &AeonCli, action: &crate::cli_enhanced::DevCommand) -> Result<()> {
    use crate::cli_enhanced::{print_info, DebugTarget, DevCommand};

    match action {
        DevCommand::Debug { target } => match target {
            DebugTarget::Lexer { file } => {
                print_info(&format!("Debugging lexer for {}", file.display()));
                debug_lexer(file)?;
            }
            DebugTarget::Parser { file } => {
                print_info(&format!("Debugging parser for {}", file.display()));
                debug_parser(file)?;
            }
            DebugTarget::Sema { file } => {
                print_info(&format!(
                    "Debugging semantic analysis for {}",
                    file.display()
                ));
                debug_semantic_analysis(file)?;
            }
            DebugTarget::Codegen { file } => {
                print_info(&format!("Debugging code generation for {}", file.display()));
                debug_code_generation(file)?;
            }
        },

        DevCommand::Profile { file, mode } => {
            print_info(&format!("Profiling {} in {} mode", file.display(), mode));
            profile_execution(file, mode)?;
        }

        DevCommand::Tokens { file } => {
            print_info(&format!("Tokenizing {}", file.display()));
            show_tokens(file)?;
        }

        DevCommand::Ast { file, format } => {
            print_info(&format!(
                "Showing AST for {} in {} format",
                file.display(),
                format
            ));
            show_ast(file, format)?;
        }

        DevCommand::Ir { file, optimize } => {
            print_info(&format!("Showing IR for {}", file.display()));
            show_ir(file, *optimize)?;
        }

        DevCommand::Disasm { file } => {
            print_info(&format!("Disassembling bytecode {}", file.display()));
            disassemble_bytecode(file)?;
        }
    }

    Ok(())
}

/// Handle project management commands
fn handle_project_commands(
    cli: &AeonCli,
    action: &crate::cli_enhanced::ProjectCommand,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success, ProjectCommand};

    match action {
        ProjectCommand::Info => {
            show_project_info()?;
        }

        ProjectCommand::Add {
            dependency,
            version,
            dev,
        } => {
            print_info(&format!("Adding dependency: {}", dependency));
            add_dependency(dependency, version.as_deref(), *dev)?;
            print_success(&format!("Added dependency {}", dependency));
        }

        ProjectCommand::Remove { dependency } => {
            print_info(&format!("Removing dependency: {}", dependency));
            remove_dependency(dependency)?;
            print_success(&format!("Removed dependency {}", dependency));
        }

        ProjectCommand::Update { dependency } => {
            if let Some(dep) = dependency {
                print_info(&format!("Updating dependency: {}", dep));
                update_dependency(Some(dep))?;
            } else {
                print_info("Updating all dependencies");
                update_dependency(None)?;
            }
            print_success("Dependencies updated");
        }

        ProjectCommand::Tree => {
            print_info("Dependency tree:");
            show_dependency_tree()?;
        }

        ProjectCommand::Clean { all } => {
            print_info("Cleaning build artifacts");
            clean_project(*all)?;
            print_success("Project cleaned");
        }

        ProjectCommand::ExportQasm {
            manifest_path,
            output,
        } => {
            print_info("Exporting project to QASM format");
            crate::commands::project::export_qasm(manifest_path.clone(), output.clone())?;
        }

        ProjectCommand::ExportPython {
            manifest_path,
            output,
        } => {
            print_info("Exporting Python runner");
            crate::commands::project::export_python(manifest_path.clone(), output.clone())?;
        }
    }

    Ok(())
}

/// Handle package management commands
fn handle_package_commands(
    cli: &AeonCli,
    action: &crate::cli_enhanced::PackageCommand,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success, PackageCommand};

    match action {
        PackageCommand::Pack { output, format } => {
            print_info(&format!("Packaging project in {} format", format));
            package_project(output.as_deref(), format)?;
            print_success("Project packaged");
        }

        PackageCommand::Publish { dry_run, registry } => {
            if *dry_run {
                print_info("Dry run: preparing package for publishing");
            } else {
                print_info("Publishing package");
            }
            publish_package(*dry_run, registry.as_deref())?;
            print_success("Package published");
        }

        PackageCommand::Install { package, version } => {
            print_info(&format!("Installing package: {}", package));
            install_package(package, version.as_deref())?;
            print_success(&format!("Installed package {}", package));
        }

        PackageCommand::Uninstall { package } => {
            print_info(&format!("Uninstalling package: {}", package));
            uninstall_package(package)?;
            print_success(&format!("Uninstalled package {}", package));
        }

        PackageCommand::List => {
            print_info("Installed packages:");
            list_installed_packages()?;
        }

        PackageCommand::Search { query } => {
            print_info(&format!("Searching for packages: {}", query));
            search_packages(query)?;
        }
    }

    Ok(())
}

/// Handle REPL command
fn handle_repl_command(
    cli: &AeonCli,
    backend: Option<&str>,
    load: Option<&std::path::Path>,
) -> Result<()> {
    use crate::cli_enhanced::print_info;

    print_info("Starting Aeonmi REPL");
    start_repl(backend, load)?;
    Ok(())
}

/// Handle benchmark command
fn handle_bench_command(
    cli: &AeonCli,
    filter: Option<&str>,
    save: Option<&std::path::Path>,
    compare: Option<&std::path::Path>,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success};

    print_info("Running benchmarks");
    run_benchmarks(filter, save, compare)?;
    print_success("Benchmarks completed");
    Ok(())
}

/// Handle documentation command
fn handle_doc_command(
    cli: &AeonCli,
    open: bool,
    private: bool,
    target_dir: Option<&std::path::Path>,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success};

    print_info("Generating documentation");
    generate_documentation(private, target_dir)?;

    if open {
        open_documentation(target_dir)?;
    }

    print_success("Documentation generated");
    Ok(())
}

/// Handle LSP commands
fn handle_lsp_commands(cli: &AeonCli, action: &crate::cli_enhanced::LspCommand) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success, LspCommand};

    match action {
        LspCommand::Start { port, log_level } => {
            print_info("Starting Aeonmi Language Server");
            start_lsp_server(*port, log_level)?;
            print_success("LSP server started");
        }

        LspCommand::Stop => {
            print_info("Stopping LSP server");
            stop_lsp_server()?;
            print_success("LSP server stopped");
        }

        LspCommand::Status => {
            show_lsp_status()?;
        }
    }

    Ok(())
}

/// Handle configuration commands
fn handle_config_commands(
    cli: &AeonCli,
    action: &crate::cli_enhanced::ConfigCommand,
) -> Result<()> {
    use crate::cli_enhanced::{print_success, ConfigCommand};

    match action {
        ConfigCommand::Show => {
            show_configuration()?;
        }

        ConfigCommand::Set { key, value } => {
            set_configuration(key, value)?;
            print_success(&format!("Set {} = {}", key, value));
        }

        ConfigCommand::Get { key } => {
            get_configuration(key)?;
        }

        ConfigCommand::Reset { key } => {
            if let Some(k) = key {
                reset_configuration_key(k)?;
                print_success(&format!("Reset configuration key: {}", k));
            } else {
                reset_all_configuration()?;
                print_success("Reset all configuration to defaults");
            }
        }

        ConfigCommand::Edit => {
            edit_configuration()?;
        }
    }

    Ok(())
}

/// Handle version command
fn handle_version_command(cli: &AeonCli, verbose: bool) -> Result<()> {
    if verbose {
        show_detailed_version_info()?;
    } else {
        show_version_info()?;
    }
    Ok(())
}

// Utility functions
fn set_console_title() {
    use crossterm::{execute, terminal::SetTitle};
    let _ = execute!(std::io::stdout(), SetTitle("Aeonmi"));
}

fn should_show_migration_message(args: &[String]) -> bool {
    // Check for old-style usage patterns that should be migrated
    args.len() > 1
        && (args.contains(&"--emit".to_string())
            || args.contains(&"--out".to_string())
            || args.contains(&"--tokens".to_string())
            || args.contains(&"--ast".to_string()))
        && !args.contains(&"build".to_string())
        && !args.contains(&"run".to_string())
        && !args.contains(&"check".to_string())
}

fn show_migration_message() {
    println!("{}", "Aeonmi CLI Migration Notice".bright_yellow().bold());
    println!();
    println!("The Aeonmi CLI has been updated to use modern subcommands.");
    println!("Please update your usage as follows:");
    println!();
    println!("{}", "Old usage:".bright_white());
    println!("  aeonmi file.ai --emit js --out output.js");
    println!("  aeonmi file.ai --tokens");
    println!("  aeonmi file.ai --ast");
    println!();
    println!("{}", "New usage:".bright_white());
    println!(
        "  {} file.ai --format js --output output.js",
        "aeon build".bright_green()
    );
    println!("  {} file.ai", "aeon dev tokens".bright_green());
    println!("  {} file.ai", "aeon dev ast".bright_green());
    println!();
    println!("For more information, run: {}", "aeon --help".bright_blue());
}

// Stub implementations for command functionality
// These would be implemented with actual logic in a complete system

use std::path::PathBuf;

fn discover_source_files_in_project() -> Result<Vec<PathBuf>> {
    // Implementation would discover .ai files in the project
    Ok(vec![])
}

fn format_file(
    file: &std::path::Path,
    check: bool,
    config: Option<&std::path::Path>,
) -> Result<bool> {
    // Implementation would format the file or check if it needs formatting
    Ok(false)
}

#[derive(Debug)]
struct LintResult {
    issues: Vec<LintIssue>,
    fixed: usize,
}

#[derive(Debug)]
struct LintIssue {
    line: usize,
    column: usize,
    level: LintLevel,
    message: String,
}

#[derive(Debug)]
enum LintLevel {
    Error,
    Warning,
    Info,
}

fn lint_file(
    file: &std::path::Path,
    fix: bool,
    config: Option<&std::path::Path>,
) -> Result<LintResult> {
    // Implementation would lint the file
    Ok(LintResult {
        issues: vec![],
        fixed: 0,
    })
}

// Quantum command implementations
fn execute_quantum_circuit(
    file: &std::path::Path,
    backend: &str,
    shots: usize,
    optimize: Option<u8>,
) -> Result<()> {
    todo!("Implement quantum circuit execution")
}

fn list_quantum_backends() -> Result<()> {
    todo!("Implement quantum backend listing")
}

fn simulate_quantum_circuit(file: &std::path::Path, shots: usize, statevector: bool) -> Result<()> {
    todo!("Implement quantum circuit simulation")
}

fn visualize_quantum_circuit(
    file: &std::path::Path,
    format: &str,
    output: Option<&std::path::Path>,
) -> Result<()> {
    todo!("Implement quantum circuit visualization")
}

// Dev command implementations
fn debug_lexer(file: &std::path::Path) -> Result<()> {
    todo!("Implement lexer debugging")
}

fn debug_parser(file: &std::path::Path) -> Result<()> {
    todo!("Implement parser debugging")
}

fn debug_semantic_analysis(file: &std::path::Path) -> Result<()> {
    todo!("Implement semantic analysis debugging")
}

fn debug_code_generation(file: &std::path::Path) -> Result<()> {
    todo!("Implement code generation debugging")
}

fn profile_execution(file: &std::path::Path, mode: &str) -> Result<()> {
    todo!("Implement execution profiling")
}

fn show_tokens(file: &std::path::Path) -> Result<()> {
    // Delegate to existing tokens command
    crate::commands::tokens::run_tokens(file)
}

fn show_ast(file: &std::path::Path, format: &str) -> Result<()> {
    // Delegate to existing AST command
    crate::commands::ast::run_ast(file, format)
}

fn show_ir(file: &std::path::Path, optimize: Option<u8>) -> Result<()> {
    todo!("Implement IR display")
}

fn disassemble_bytecode(file: &std::path::Path) -> Result<()> {
    todo!("Implement bytecode disassembly")
}

// Project command implementations
fn show_project_info() -> Result<()> {
    todo!("Implement project info display")
}

fn add_dependency(dependency: &str, version: Option<&str>, dev: bool) -> Result<()> {
    todo!("Implement dependency addition")
}

fn remove_dependency(dependency: &str) -> Result<()> {
    todo!("Implement dependency removal")
}

fn update_dependency(dependency: Option<&str>) -> Result<()> {
    todo!("Implement dependency update")
}

fn show_dependency_tree() -> Result<()> {
    todo!("Implement dependency tree display")
}

fn clean_project(all: bool) -> Result<()> {
    todo!("Implement project cleaning")
}

// Package command implementations
fn package_project(output: Option<&std::path::Path>, format: &str) -> Result<()> {
    todo!("Implement project packaging")
}

fn publish_package(dry_run: bool, registry: Option<&str>) -> Result<()> {
    todo!("Implement package publishing")
}

fn install_package(package: &str, version: Option<&str>) -> Result<()> {
    todo!("Implement package installation")
}

fn uninstall_package(package: &str) -> Result<()> {
    todo!("Implement package uninstallation")
}

fn list_installed_packages() -> Result<()> {
    todo!("Implement installed package listing")
}

fn search_packages(query: &str) -> Result<()> {
    todo!("Implement package search")
}

// Other command implementations
fn start_repl(backend: Option<&str>, load: Option<&std::path::Path>) -> Result<()> {
    // Delegate to existing REPL
    crate::commands::repl::start_repl(backend.unwrap_or("simulator"), load)
}

fn run_benchmarks(
    filter: Option<&str>,
    save: Option<&std::path::Path>,
    compare: Option<&std::path::Path>,
) -> Result<()> {
    todo!("Implement benchmark running")
}

fn generate_documentation(private: bool, target_dir: Option<&std::path::Path>) -> Result<()> {
    todo!("Implement documentation generation")
}

fn open_documentation(target_dir: Option<&std::path::Path>) -> Result<()> {
    todo!("Implement documentation opening")
}

fn start_lsp_server(port: Option<u16>, log_level: &str) -> Result<()> {
    todo!("Implement LSP server startup")
}

fn stop_lsp_server() -> Result<()> {
    todo!("Implement LSP server shutdown")
}

fn show_lsp_status() -> Result<()> {
    todo!("Implement LSP status display")
}

fn show_configuration() -> Result<()> {
    todo!("Implement configuration display")
}

fn set_configuration(key: &str, value: &str) -> Result<()> {
    todo!("Implement configuration setting")
}

fn get_configuration(key: &str) -> Result<()> {
    todo!("Implement configuration getting")
}

fn reset_configuration_key(key: &str) -> Result<()> {
    todo!("Implement configuration key reset")
}

fn reset_all_configuration() -> Result<()> {
    todo!("Implement full configuration reset")
}

fn edit_configuration() -> Result<()> {
    todo!("Implement configuration editing")
}

fn show_version_info() -> Result<()> {
    println!("aeon {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn show_detailed_version_info() -> Result<()> {
    println!("Aeonmi Quantum Programming Language");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: {}", "unknown"); // Would be set by build system
    println!("Target: {}", std::env::consts::ARCH);
    println!("OS: {}", std::env::consts::OS);
    println!("Rust: {}", "unknown"); // Would be set by build system
    Ok(())
}

/// Handle editor command
fn handle_editor_command(
    _cli: &AeonCli,
    port: u16,
    workspace: Option<&std::path::Path>,
    browser: bool,
) -> Result<()> {
    use crate::cli_enhanced::{print_info, print_success};

    print_info("🚀 Starting Aeonmi Integrated Editor...");

    let workspace_path = workspace.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });

    print_info(&format!("📁 Workspace: {}", workspace_path.display()));
    print_info(&format!("🌐 Port: {}", port));

    // Create async runtime to run the editor server
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Start the editor server
        let server_task =
            tokio::spawn(
                async move { crate::editor::start_editor_server(port, workspace_path).await },
            );

        // Optionally open browser
        if browser {
            let url = format!("http://127.0.0.1:{}", port);
            print_info(&format!("🌐 Opening browser: {}", url));

            // Try to open browser (best effort)
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd")
                    .args(&["/c", "start", &url])
                    .spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&url).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
        }

        print_success(&format!(
            "✅ Editor server running at http://127.0.0.1:{}",
            port
        ));
        print_info("Press Ctrl+C to stop the server");

        // Wait for the server to finish
        server_task.await??;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

/// Handle sandbox command
fn handle_sandbox_command(cli: &AeonCli, action: &SandboxCommand) -> Result<()> {
    use crate::cli_enhanced::{print_error, print_info, print_success, print_warning};
    use crate::sandbox::{create_workspace, AeonmiWorkspace, ExecutionLimits, WorkspaceConfig};
    use std::path::PathBuf;
    use std::time::Duration;

    match action {
        SandboxCommand::Create {
            name,
            template,
            timeout,
            memory_limit,
            max_processes,
        } => {
            print_info(&format!("Creating sandbox workspace: {}", name));

            // Create workspace directory
            let workspace_path = std::env::current_dir()?.join(name);

            if workspace_path.exists() {
                return Err(anyhow::anyhow!("Workspace '{}' already exists", name));
            }

            // Create workspace configuration
            let config = WorkspaceConfig {
                name: name.clone(),
                description: Some(format!("Aeonmi sandbox workspace: {}", name)),
                aeonmi_version: env!("CARGO_PKG_VERSION").to_string(),
                targets: vec!["quantum".to_string(), "classical".to_string()],
                dependencies: std::collections::HashMap::new(),
                max_artifacts: 10,
                auto_cleanup: true,
                allowed_commands: vec![
                    "python".to_string(),
                    "qiskit".to_string(),
                    "aeon".to_string(),
                ],
            };

            // Create the workspace
            let workspace = create_workspace(workspace_path.clone(), Some(config))?;

            print_success(&format!(
                "✅ Sandbox workspace '{}' created at: {}",
                name,
                workspace_path.display()
            ));
            print_info(&format!("   Template: {}", template));
            print_info(&format!("   Timeout: {}s", timeout));
            print_info(&format!("   Memory limit: {}MB", memory_limit));
            print_info(&format!("   Max processes: {}", max_processes));
            print_info(&format!("   Use 'aeon sandbox enter {}' to activate", name));
        }

        SandboxCommand::List { verbose } => {
            print_info("📋 Available sandbox workspaces:");

            // Look for Aeonmi.toml files in subdirectories
            let current_dir = std::env::current_dir()?;
            let mut found_workspaces = false;

            for entry in std::fs::read_dir(&current_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let manifest_path = path.join("Aeonmi.toml");
                    if manifest_path.exists() {
                        found_workspaces = true;

                        let workspace_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        if *verbose {
                            // Try to load workspace for detailed info
                            match AeonmiWorkspace::load(path.clone()) {
                                Ok(workspace) => {
                                    println!(
                                        "  📁 {} ({})",
                                        workspace_name.bright_blue(),
                                        path.display()
                                    );
                                    println!("     Source: {}", workspace.source_dir().display());
                                    println!("     Output: {}", workspace.output_dir().display());
                                }
                                Err(_) => {
                                    println!(
                                        "  📁 {} ({}) - [Invalid]",
                                        workspace_name.bright_yellow(),
                                        path.display()
                                    );
                                }
                            }
                        } else {
                            println!("  📁 {}", workspace_name.bright_blue());
                        }
                    }
                }
            }

            if !found_workspaces {
                print_info("   No sandbox workspaces found");
                print_info("   Use 'aeon sandbox create <name>' to create one");
            }
        }

        SandboxCommand::Enter { workspace } => {
            print_info(&format!("Entering sandbox workspace: {}", workspace));

            let workspace_path = PathBuf::from(workspace);
            if !workspace_path.exists() {
                return Err(anyhow::anyhow!("Workspace '{}' not found", workspace));
            }

            // Load the workspace
            let ws = AeonmiWorkspace::load(workspace_path.clone())?;

            print_success(&format!("✅ Entered sandbox: {}", workspace));
            print_info(&format!(
                "   Working directory: {}",
                workspace_path.display()
            ));
            print_info("   Available commands:");
            print_info("     aeon build        # Build project");
            print_info("     aeon run          # Run project");
            print_info("     aeon test         # Run tests");
            print_info("     aeon sandbox exec # Execute command");

            // Change to workspace directory
            std::env::set_current_dir(&workspace_path)?;
        }

        SandboxCommand::Exec {
            workspace,
            command,
            args,
            timeout,
            stats,
        } => {
            print_info(&format!(
                "Executing in sandbox '{}': {} {}",
                workspace,
                command,
                args.join(" ")
            ));

            let workspace_path = PathBuf::from(workspace);
            if !workspace_path.exists() {
                return Err(anyhow::anyhow!("Workspace '{}' not found", workspace));
            }

            // Load the workspace
            let ws = AeonmiWorkspace::load(workspace_path.clone())?;

            // Create execution context
            let mut exec_limits = ExecutionLimits::default();
            if let Some(timeout_secs) = timeout {
                exec_limits.timeout = Duration::from_secs(*timeout_secs);
            }

            // Execute command using process manager
            use crate::sandbox::{ProcessConfig, ProcessManager};
            use std::collections::HashMap;

            let process_manager = ProcessManager::new();
            let config = ProcessConfig {
                command: command.clone(),
                args: args.clone(),
                working_dir: workspace_path.clone(),
                env_vars: HashMap::new(),
                limits: exec_limits,
                capture_stdout: true,
                capture_stderr: true,
            };

            let process_id = process_manager.start_process(config)?;

            if *stats {
                print_info(&format!("📊 Process started with ID: {}", process_id));
            }

            // Wait for completion
            let status =
                process_manager.wait_for_process(process_id, Some(Duration::from_secs(60)))?;

            match status {
                ProcessStatus::Completed(exit_code) => {
                    if exit_code == 0 {
                        print_success("✅ Command completed successfully");
                    } else {
                        print_error(&format!("❌ Command failed with exit code: {}", exit_code));
                    }
                }
                ProcessStatus::TimedOut => {
                    print_error("⏰ Command timed out");
                }
                ProcessStatus::Killed => {
                    print_warning("🛑 Command was killed");
                }
                _ => {
                    print_error("❓ Command execution failed");
                }
            }
        }

        SandboxCommand::Clean {
            workspace,
            temp_only,
            force,
        } => {
            let workspace_path = if let Some(ws) = workspace {
                PathBuf::from(ws)
            } else {
                std::env::current_dir()?
            };

            if !workspace_path.exists() {
                return Err(anyhow::anyhow!(
                    "Workspace path not found: {}",
                    workspace_path.display()
                ));
            }

            // Load workspace
            let ws = AeonmiWorkspace::load(workspace_path.clone())?;

            if !force {
                print_warning("This will remove temporary files and build artifacts.");
                print_info("Press Enter to continue, or Ctrl+C to cancel...");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }

            // Clean up artifacts
            ws.cleanup_artifacts()?;

            if !temp_only {
                // Also clean output directory
                if ws.output_dir().exists() {
                    std::fs::remove_dir_all(ws.output_dir())?;
                    std::fs::create_dir_all(ws.output_dir())?;
                }
            }

            print_success("✅ Sandbox cleanup completed");
        }

        SandboxCommand::Remove { workspace, force } => {
            let workspace_path = PathBuf::from(workspace);

            if !workspace_path.exists() {
                return Err(anyhow::anyhow!("Workspace '{}' not found", workspace));
            }

            if !force {
                print_warning(&format!(
                    "This will permanently remove the sandbox workspace: {}",
                    workspace
                ));
                print_warning("All files and data will be lost!");
                print_info("Type 'yes' to confirm: ");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "yes" {
                    print_info("❌ Operation cancelled");
                    return Ok(());
                }
            }

            // Remove the entire workspace directory
            std::fs::remove_dir_all(&workspace_path)?;

            print_success(&format!("✅ Sandbox workspace '{}' removed", workspace));
        }

        SandboxCommand::Status {
            workspace,
            processes,
            resources,
        } => {
            let workspace_path = if let Some(ws) = workspace {
                PathBuf::from(ws)
            } else {
                std::env::current_dir()?
            };

            if !workspace_path.exists() {
                return Err(anyhow::anyhow!(
                    "Workspace path not found: {}",
                    workspace_path.display()
                ));
            }

            // Load workspace
            let ws = AeonmiWorkspace::load(workspace_path.clone())?;

            println!(
                "📊 Sandbox Status: {}",
                workspace_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .bright_blue()
            );
            println!("   Path: {}", workspace_path.display());
            println!("   Source: {}", ws.source_dir().display());
            println!("   Output: {}", ws.output_dir().display());

            if *processes || *resources {
                // Show process information
                let process_manager = ProcessManager::new();
                let active_processes = process_manager.list_processes();

                println!("   Active processes: {}", active_processes.len());

                if *processes && !active_processes.is_empty() {
                    for (id, status, command, duration) in active_processes {
                        println!(
                            "     🔄 {} - {} ({}s) - {:?}",
                            id,
                            command,
                            duration.as_secs(),
                            status
                        );
                    }
                }
            }

            // Show disk usage
            if *resources {
                if let Ok(metadata) = std::fs::metadata(&workspace_path) {
                    // Note: This is a simplified size calculation
                    println!("   Disk usage: {} bytes", metadata.len());
                }
            }
        }

        SandboxCommand::Kill {
            workspace,
            process_id,
            force,
        } => {
            let workspace_path = PathBuf::from(workspace);

            if !workspace_path.exists() {
                return Err(anyhow::anyhow!("Workspace '{}' not found", workspace));
            }

            let process_manager = ProcessManager::new();

            if let Some(pid_str) = process_id {
                // Kill specific process
                if let Ok(pid) = uuid::Uuid::parse_str(pid_str) {
                    process_manager.kill_process(pid)?;
                    print_success(&format!("✅ Killed process: {}", pid));
                } else {
                    return Err(anyhow::anyhow!("Invalid process ID: {}", pid_str));
                }
            } else {
                // Kill all processes
                if !force {
                    print_warning("This will kill all running processes in the sandbox.");
                    print_info("Press Enter to continue, or Ctrl+C to cancel...");

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                }

                process_manager.kill_all_processes()?;
                print_success("✅ All processes killed");
            }
        }
    }

    Ok(())
}
