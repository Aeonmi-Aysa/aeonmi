// Enhanced CLI for Aeonmi following modern toolchain conventions
// Uses "aeon" as the user-facing command name with comprehensive subcommands

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;

/// Aeonmi command-line interface following modern toolchain conventions
#[derive(Debug, Parser)]
#[command(
    name = "aeon",
    about = "Aeonmi Quantum Programming Language - Unified quantum computing ecosystem",
    version,
    propagate_version = true,
    disable_help_subcommand = true,
    after_help = "Examples:\n  aeon build                    # Build current project\n  aeon run main.ai              # Compile and execute a file\n  aeon test                     # Run all tests\n  aeon check                    # Verify syntax and semantics\n  aeon new my_project           # Create new project"
)]
pub struct AeonCli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    /// Path to config file (default: ~/.aeonmi/config.toml)
    #[arg(long = "config", value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Enable debug mode
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// Optimization level (0, 1, 2, 3)
    #[arg(short = 'O', long = "opt-level", value_name = "LEVEL", global = true, value_parser = clap::value_parser!(u8).range(0..=3))]
    pub opt_level: Option<u8>,

    /// Target platform/backend
    #[arg(short = 't', long = "target", value_name = "TARGET", global = true)]
    pub target: Option<String>,

    /// Number of quantum shots for execution
    #[arg(short = 's', long = "shots", value_name = "N", global = true)]
    pub shots: Option<usize>,

    /// Output directory
    #[arg(short = 'o', long = "output", value_name = "DIR", global = true)]
    pub output: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<AeonCommand>,

    /// Input file (for legacy single-file mode: aeon <file>)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AeonCommand {
    /// Compile the project or file(s) into target output
    Build {
        /// Build in release mode (optimized)
        #[arg(short, long)]
        release: bool,

        /// Path to project manifest (Aeonmi.toml)
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,

        /// Specific files to build (overrides project mode)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Output format
        #[arg(long = "format", value_enum, default_value_t = OutputFormat::Bytecode)]
        format: OutputFormat,

        /// Watch for changes and rebuild
        #[arg(short, long)]
        watch: bool,

        /// Enable parallel compilation
        #[arg(short, long)]
        parallel: bool,
    },

    /// Compile and execute the project or file
    Run {
        /// Path to project manifest (Aeonmi.toml)
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,

        /// Specific file to run (overrides project mode)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Arguments to pass to the program
        #[arg(last = true, value_name = "ARGS")]
        args: Vec<String>,

        /// Run in release mode
        #[arg(short, long)]
        release: bool,

        /// Use native VM interpreter
        #[arg(long)]
        native: bool,

        /// Use bytecode VM
        #[arg(long)]
        bytecode: bool,

        /// Use quantum backend
        #[arg(long = "quantum", value_name = "BACKEND")]
        quantum_backend: Option<String>,

        /// Execution timeout in milliseconds
        #[arg(long = "timeout", value_name = "MS")]
        timeout_ms: Option<u64>,

        /// Watch for changes and re-run
        #[arg(short, long)]
        watch: bool,

        /// Show execution statistics
        #[arg(long)]
        stats: bool,
    },

    /// Run tests in the project
    Test {
        /// Path to project manifest (Aeonmi.toml)
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,

        /// Test filter pattern
        #[arg(value_name = "FILTER")]
        filter: Option<String>,

        /// Run tests in release mode
        #[arg(short, long)]
        release: bool,

        /// Show test output
        #[arg(long)]
        nocapture: bool,

        /// Run tests in parallel
        #[arg(short, long)]
        parallel: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Watch for changes and re-run tests
        #[arg(short, long)]
        watch: bool,
    },

    /// Check syntax and semantics without building
    Check {
        /// Path to project manifest (Aeonmi.toml)
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,

        /// Specific files to check (overrides project mode)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Only check syntax (skip semantic analysis)
        #[arg(long)]
        syntax_only: bool,

        /// Watch for changes and re-check
        #[arg(short, long)]
        watch: bool,
    },

    /// Remove build artifacts and output directories
    Clean {
        /// Path to project manifest (Aeonmi.toml)
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,
    },

    /// Create a new Aeonmi project or file
    New {
        /// Project or file name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project template
        #[arg(long = "template", value_name = "TEMPLATE", default_value = "basic")]
        template: String,

        /// Initialize with git repository
        #[arg(long)]
        git: bool,

        /// Open in editor after creation
        #[arg(long)]
        open: bool,

        /// Create library project instead of binary
        #[arg(long)]
        lib: bool,
    },

    /// Initialize current directory as Aeonmi project
    Init {
        /// Project name (defaults to directory name)
        #[arg(long = "name", value_name = "NAME")]
        name: Option<String>,

        /// Project template
        #[arg(long = "template", value_name = "TEMPLATE", default_value = "basic")]
        template: String,

        /// Initialize with git repository
        #[arg(long)]
        git: bool,

        /// Create library project instead of binary
        #[arg(long)]
        lib: bool,
    },

    /// Format source files
    Format {
        /// Files to format (default: all .ai files in project)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Check if files are formatted without modifying
        #[arg(long)]
        check: bool,

        /// Format configuration file
        #[arg(long = "config", value_name = "FILE")]
        config_file: Option<PathBuf>,
    },

    /// Lint source files
    Lint {
        /// Files to lint (default: all .ai files in project)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Apply automatic fixes
        #[arg(long)]
        fix: bool,

        /// Lint configuration file
        #[arg(long = "config", value_name = "FILE")]
        config_file: Option<PathBuf>,
    },

    /// Quantum-specific operations
    Quantum {
        #[command(subcommand)]
        action: QuantumCommand,
    },

    /// Development tools and utilities
    Dev {
        #[command(subcommand)]
        action: DevCommand,
    },

    /// Project management
    Project {
        #[command(subcommand)]
        action: ProjectCommand,
    },

    /// Package management
    Package {
        #[command(subcommand)]
        action: PackageCommand,
    },

    /// Interactive REPL
    Repl {
        /// Start with specific backend
        #[arg(long = "backend", value_name = "BACKEND")]
        backend: Option<String>,

        /// Load file into REPL session
        #[arg(long = "load", value_name = "FILE")]
        load: Option<PathBuf>,
    },

    /// Benchmarking tools
    Bench {
        /// Benchmark filter pattern
        #[arg(value_name = "FILTER")]
        filter: Option<String>,

        /// Save benchmark results
        #[arg(long = "save", value_name = "FILE")]
        save: Option<PathBuf>,

        /// Compare with previous results
        #[arg(long = "compare", value_name = "FILE")]
        compare: Option<PathBuf>,
    },

    /// Documentation generation
    Doc {
        /// Generate and open documentation
        #[arg(long)]
        open: bool,

        /// Include private items
        #[arg(long)]
        private: bool,

        /// Documentation output directory
        #[arg(long = "target-dir", value_name = "DIR")]
        target_dir: Option<PathBuf>,
    },

    /// Language server operations
    Lsp {
        /// Start LSP server
        #[command(subcommand)]
        action: LspCommand,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },

    /// Launch integrated web editor
    Editor {
        /// Port to bind the editor server
        #[arg(short, long, value_name = "PORT", default_value = "4000")]
        port: u16,

        /// Workspace directory
        #[arg(short, long, value_name = "DIR")]
        workspace: Option<PathBuf>,

        /// Open browser automatically
        #[arg(long)]
        browser: bool,
    },

    /// Sandbox workspace management
    Sandbox {
        #[command(subcommand)]
        action: SandboxCommand,
    },

    /// Version information
    Version {
        /// Show detailed version information
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum QuantumCommand {
    /// Execute quantum circuit
    Run {
        /// Circuit file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Quantum backend
        #[arg(short, long, value_name = "BACKEND")]
        backend: String,

        /// Number of shots
        #[arg(short, long, value_name = "N", default_value = "1024")]
        shots: usize,

        /// Optimization level
        #[arg(short = 'O', long = "optimize", value_name = "LEVEL")]
        optimize: Option<u8>,
    },

    /// List available quantum backends
    Backends,

    /// Simulate quantum circuit locally
    Simulate {
        /// Circuit file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Number of shots
        #[arg(short, long, value_name = "N", default_value = "1024")]
        shots: usize,

        /// Show state vector
        #[arg(long)]
        statevector: bool,
    },

    /// Visualize quantum circuit
    Visualize {
        /// Circuit file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(short, long, value_name = "FORMAT", default_value = "text")]
        format: String,

        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum DevCommand {
    /// Debug tools
    Debug {
        /// Debug target
        #[command(subcommand)]
        target: DebugTarget,
    },

    /// Performance profiling
    Profile {
        /// File to profile
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Profiling mode
        #[arg(short, long, value_name = "MODE", default_value = "time")]
        mode: String,
    },

    /// Show tokens for file
    Tokens {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Show AST for file
    Ast {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(short, long, value_name = "FORMAT", default_value = "pretty")]
        format: String,
    },

    /// Show IR for file
    Ir {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Optimization level
        #[arg(short = 'O', long = "optimize", value_name = "LEVEL")]
        optimize: Option<u8>,
    },

    /// Disassemble bytecode
    Disasm {
        /// Bytecode file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum DebugTarget {
    /// Debug lexer
    Lexer {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Debug parser
    Parser {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Debug semantic analysis
    Sema {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Debug code generation
    Codegen {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProjectCommand {
    /// Show project information
    Info,

    /// Add dependency
    Add {
        /// Dependency name
        #[arg(value_name = "DEPENDENCY")]
        dependency: String,

        /// Version requirement
        #[arg(long = "version", value_name = "VERSION")]
        version: Option<String>,

        /// Add as development dependency
        #[arg(long)]
        dev: bool,
    },

    /// Remove dependency
    Remove {
        /// Dependency name
        #[arg(value_name = "DEPENDENCY")]
        dependency: String,
    },

    /// Update dependencies
    Update {
        /// Specific dependency to update
        #[arg(value_name = "DEPENDENCY")]
        dependency: Option<String>,
    },

    /// Show dependency tree
    Tree,

    /// Clean build artifacts
    Clean {
        /// Remove all target files
        #[arg(long)]
        all: bool,
    },

    /// Export project to OpenQASM 2.0 format
    ExportQasm {
        /// Path to a custom manifest file
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,
        
        /// Output path for QASM file (default: output/circuit.qasm)
        #[arg(short = 'o', long = "output", value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Export project to Python script that runs QASM via Qiskit
    ExportPython {
        /// Path to a custom manifest file
        #[arg(long = "manifest-path", value_name = "FILE")]
        manifest_path: Option<PathBuf>,
        
        /// Output path for Python file (default: output/<project>_runner.py)
        #[arg(short = 'o', long = "output", value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum PackageCommand {
    /// Package project for distribution
    Pack {
        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Package format
        #[arg(long = "format", value_name = "FORMAT", default_value = "tar.gz")]
        format: String,
    },

    /// Publish package
    Publish {
        /// Dry run (don't actually publish)
        #[arg(long)]
        dry_run: bool,

        /// Registry URL
        #[arg(long = "registry", value_name = "URL")]
        registry: Option<String>,
    },

    /// Install package
    Install {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// Version requirement
        #[arg(long = "version", value_name = "VERSION")]
        version: Option<String>,
    },

    /// Uninstall package
    Uninstall {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,
    },

    /// List installed packages
    List,

    /// Search packages
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum LspCommand {
    /// Start LSP server
    Start {
        /// Server port (stdio if not specified)
        #[arg(long = "port", value_name = "PORT")]
        port: Option<u16>,

        /// Log level
        #[arg(long = "log-level", value_name = "LEVEL", default_value = "info")]
        log_level: String,
    },

    /// Stop LSP server
    Stop,

    /// Show LSP status
    Status,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration
    Show,

    /// Set configuration value
    Set {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: String,

        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Get configuration value
    Get {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Reset configuration to defaults
    Reset {
        /// Reset specific key only
        #[arg(value_name = "KEY")]
        key: Option<String>,
    },

    /// Edit configuration file
    Edit,
}

#[derive(Debug, Clone, Subcommand)]
pub enum SandboxCommand {
    /// Create a new sandboxed workspace
    Create {
        /// Workspace name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project template
        #[arg(long = "template", value_name = "TEMPLATE", default_value = "basic")]
        template: String,

        /// Execution timeout in seconds
        #[arg(long = "timeout", value_name = "SECONDS", default_value = "30")]
        timeout: u64,

        /// Memory limit in MB
        #[arg(long = "memory", value_name = "MB", default_value = "128")]
        memory_limit: usize,

        /// Maximum number of processes
        #[arg(long = "max-processes", value_name = "N", default_value = "4")]
        max_processes: usize,
    },

    /// List all sandboxed workspaces
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Enter/activate a sandbox workspace
    Enter {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: String,
    },

    /// Execute a command in sandbox
    Exec {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: String,

        /// Command to execute
        #[arg(value_name = "COMMAND")]
        command: String,

        /// Command arguments
        #[arg(last = true, value_name = "ARGS")]
        args: Vec<String>,

        /// Execution timeout in seconds
        #[arg(long = "timeout", value_name = "SECONDS")]
        timeout: Option<u64>,

        /// Show execution statistics
        #[arg(long)]
        stats: bool,
    },

    /// Clean up sandbox artifacts
    Clean {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: Option<String>,

        /// Remove temporary files only
        #[arg(long)]
        temp_only: bool,

        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Remove a sandbox workspace
    Remove {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: String,

        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Show sandbox status and metrics
    Status {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: Option<String>,

        /// Show running processes
        #[arg(long)]
        processes: bool,

        /// Show resource usage
        #[arg(long)]
        resources: bool,
    },

    /// Kill running processes in sandbox
    Kill {
        /// Workspace name or path
        #[arg(value_name = "WORKSPACE")]
        workspace: String,

        /// Process ID to kill (all if not specified)
        #[arg(long = "pid", value_name = "PID")]
        process_id: Option<String>,

        /// Force kill without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    /// JavaScript output
    #[clap(alias = "js")]
    Javascript,

    /// Aeonmi intermediate representation
    #[clap(alias = "ai")]
    Aeonmi,

    /// Bytecode
    Bytecode,

    /// Native binary
    Native,

    /// QASM quantum assembly
    Qasm,

    /// WebAssembly
    #[clap(alias = "wasm")]
    WebAssembly,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Javascript => "javascript",
            OutputFormat::Aeonmi => "aeonmi",
            OutputFormat::Bytecode => "bytecode",
            OutputFormat::Native => "native",
            OutputFormat::Qasm => "qasm",
            OutputFormat::WebAssembly => "webassembly",
        }
    }
}

/// Enhanced command dispatcher with improved error handling and output formatting
pub fn run_cli() -> anyhow::Result<()> {
    let cli = AeonCli::parse();

    // Set up colored output
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Handle legacy single-file mode: aeon <file>
    if cli.command.is_none() && cli.file.is_some() {
        return handle_legacy_mode(cli);
    }

    // Handle no command (show help)
    if cli.command.is_none() {
        print_help_and_exit();
    }

    let command = cli.command.clone().unwrap();

    // Dispatch to appropriate handler via integration layer
    crate::cli_integration::dispatch_command(cli, command)
}

fn print_help_and_exit() -> ! {
    println!(
        "{}",
        "Aeonmi Quantum Programming Language".bright_cyan().bold()
    );
    println!();
    println!("{}", "USAGE:".bright_white().bold());
    println!("    {} [OPTIONS] [COMMAND]", "aeon".bright_green());
    println!("    {} [OPTIONS] <FILE>", "aeon".bright_green());
    println!();
    println!("{}", "COMMANDS:".bright_white().bold());
    println!(
        "    {}     Compile the project or files",
        "build".bright_green()
    );
    println!("    {}       Compile and execute", "run".bright_green());
    println!("    {}      Run tests", "test".bright_green());
    println!(
        "    {}     Check syntax and semantics",
        "check".bright_green()
    );
    println!(
        "    {}       Create new project or file",
        "new".bright_green()
    );
    println!(
        "    {}      Initialize current directory as project",
        "init".bright_green()
    );
    println!("    {}    Format source files", "format".bright_green());
    println!("    {}      Lint source files", "lint".bright_green());
    println!("    {}   Quantum operations", "quantum".bright_green());
    println!("    {}       Development tools", "dev".bright_green());
    println!("    {}   Project management", "project".bright_green());
    println!("    {}   Package management", "package".bright_green());
    println!("    {}      Interactive REPL", "repl".bright_green());
    println!("    {}     Benchmarking", "bench".bright_green());
    println!("    {}       Generate documentation", "doc".bright_green());
    println!("    {}       Language server", "lsp".bright_green());
    println!("    {}    Configuration", "config".bright_green());
    println!("    {}   Version information", "version".bright_green());
    println!();
    println!("{}", "OPTIONS:".bright_white().bold());
    println!(
        "    {}, {}          Enable verbose output",
        "-v".bright_yellow(),
        "--verbose".bright_yellow()
    );
    println!(
        "    {}, {}             Enable debug mode",
        "-d".bright_yellow(),
        "--debug".bright_yellow()
    );
    println!(
        "    {}        Optimization level (0-3)",
        "-O <LEVEL>".bright_yellow()
    );
    println!(
        "    {}, {}       Target platform",
        "-t".bright_yellow(),
        "--target <TARGET>".bright_yellow()
    );
    println!(
        "    {}              Config file path",
        "--config <FILE>".bright_yellow()
    );
    println!(
        "    {}            Disable colored output",
        "--no-color".bright_yellow()
    );
    println!(
        "    {}, {}             Print help",
        "-h".bright_yellow(),
        "--help".bright_yellow()
    );
    println!(
        "    {}, {}          Print version",
        "-V".bright_yellow(),
        "--version".bright_yellow()
    );
    println!();
    println!("{}", "EXAMPLES:".bright_white().bold());
    println!(
        "    {}                          # Build current project",
        "aeon build".bright_blue()
    );
    println!(
        "    {}                       # Run current project",
        "aeon run".bright_blue()
    );
    println!(
        "    {}                    # Compile and run a file",
        "aeon run main.ai".bright_blue()
    );
    println!(
        "    {}                         # Run all tests",
        "aeon test".bright_blue()
    );
    println!(
        "    {}                        # Check for errors",
        "aeon check".bright_blue()
    );
    println!(
        "    {}                # Create new project",
        "aeon new my_project".bright_blue()
    );
    println!(
        "    {}         # Quantum execution",
        "aeon quantum run --backend qiskit circuit.ai".bright_blue()
    );
    println!();
    std::process::exit(0)
}

// Legacy mode handler - supports old-style single file compilation
fn handle_legacy_mode(cli: AeonCli) -> anyhow::Result<()> {
    use crate::cli::EmitKind;
    use crate::commands::compile;

    let file = cli.file.unwrap();

    print_info(&format!("Legacy mode: compiling {}", file.display()));

    // Default to JavaScript output for compatibility
    let emit_kind = EmitKind::Js;
    let output = cli.output.unwrap_or_else(|| {
        let mut output = file.clone();
        output.set_extension("js");
        output
    });

    // Use existing compile pipeline
    compile::compile_pipeline(
        Some(file),
        emit_kind,
        output,
        false, // tokens
        false, // ast
        false, // pretty_errors
        false, // no_sema
        false, // debug_titan
    )
}

/// Utility function to print success messages
pub fn print_success(message: &str) {
    println!("{} {}", "Success:".bright_green().bold(), message);
}

/// Utility function to print error messages
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".bright_red().bold(), message);
}

/// Utility function to print warning messages
pub fn print_warning(message: &str) {
    println!("{} {}", "Warning:".bright_yellow().bold(), message);
}

/// Utility function to print info messages
pub fn print_info(message: &str) {
    println!("{} {}", "Info:".bright_blue().bold(), message);
}
