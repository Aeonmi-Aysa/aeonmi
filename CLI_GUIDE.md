# Aeonmi CLI Command Reference

Complete guide to the Aeonmi command-line interface for quantum computing and `.ai` language development.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Commands](#project-commands)
3. [File Commands](#file-commands)
4. [Development Commands](#development-commands)
5. [Quantum Commands](#quantum-commands)
6. [Vault Commands](#vault-commands)
7. [Shell & Interactive](#shell--interactive)
8. [Global Options](#global-options)

---

## Getting Started

### Installation

Build the Aeonmi CLI:
```bash
cargo build --release
```

The binary will be available as:
- `target/release/aeonmi_shard.exe` (main executable)
- `target/release/aeonmi.exe` (CLI alias)

### Basic Usage

```bash
aeonmi <COMMAND> [OPTIONS]
```

Get help:
```bash
aeonmi --help
aeonmi <COMMAND> --help
```

---

## Project Commands

Commands for managing Aeonmi projects defined by `Aeonmi.toml` manifests.

### `build`

**Description**: Build an Aeonmi project and generate bundle artifacts.

**Usage**:
```bash
aeonmi build [OPTIONS]
```

**Options**:
- `--release` - Build with optimizations (release profile)
- `--manifest-path <FILE>` - Use a custom manifest file path

**Examples**:
```bash
# Build in debug mode (default)
aeonmi build

# Build optimized release version
aeonmi build --release

# Build with custom manifest
aeonmi build --manifest-path /path/to/Aeonmi.toml
```

**Output**:
- Creates `target/debug/[project-name].bundle.json` (or `target/release/`)
- Bundle contains package metadata and compiled module index

---

### `check`

**Description**: Validate project syntax and semantics without producing artifacts.

**Usage**:
```bash
aeonmi check [OPTIONS]
```

**Options**:
- `--manifest-path <FILE>` - Use a custom manifest file path

**Examples**:
```bash
# Check current project
aeonmi check

# Check specific manifest
aeonmi check --manifest-path /path/to/Aeonmi.toml
```

**What it checks**:
- All `.ai` source files parse correctly
- Required `fn main` entry point exists
- Module imports are valid
- No syntax errors

---

### `test`

**Description**: Run project test suite (inline tests and `tests/` directory).

**Usage**:
```bash
aeonmi test [OPTIONS]
```

**Options**:
- `--release` - Run tests in release mode
- `--manifest-path <FILE>` - Use a custom manifest file path
- `--filter <SUBSTRING>` - Only run tests whose name contains this filter

**Examples**:
```bash
# Run all tests
aeonmi test

# Run only tests matching "addition"
aeonmi test --filter addition

# Run tests in release mode
aeonmi test --release

# Run specific test file with filter
aeonmi test --filter arithmetic
```

**Test Discovery**:
- Inline tests: `test test_name:` blocks in source modules
- Project tests: All `.ai` files in `tests/` directory

**Output Format**:
```
    ok - test_name
    ok - group::test_name
    FAILED - failed_test
        reason: assertion failed
test result: ok. 4 passed; 0 failed
```

---

### `run` (Project Mode)

**Description**: Execute the project's main function. When invoked without an input file, runs the project defined by `Aeonmi.toml`.

**Usage**:
```bash
aeonmi run [OPTIONS]
```

**Options** (project mode):
- `--release` - Execute using release profile
- `--manifest-path <FILE>` - Use a custom manifest file path

**Examples**:
```bash
# Run the project
aeonmi run

# Run with release optimizations
aeonmi run --release

# Run custom manifest project
aeonmi run --manifest-path /path/to/Aeonmi.toml
```

**Behavior**:
- Builds the project if needed
- Executes `fn main` from the entry point
- Loads all modules defined in manifest

---

## File Commands

Commands for working with individual `.ai` files (not project-based).

### `emit`

**Description**: Compile a single `.ai` file to various output formats.

**Usage**:
```bash
aeonmi emit <INPUT> [OPTIONS]
```

**Positional Arguments**:
- `<INPUT>` - Input file path (`.ai` or `.qube`)

**Options**:
- `--emit <FORMAT>` / `--format <FORMAT>` - Output format (default: `js`)
  - `js` - JavaScript code
  - `ai` - Canonical Aeonmi form
- `-o, --out <FILE>` - Output file path (default: `output.js` or `output.ai`)
- `--tokens` - Dump token stream (debug)
- `--ast` - Dump abstract syntax tree (debug)
- `--debug-titan` - Enable Titan debug mode
- `--watch` - Watch file for changes and recompile

**Examples**:
```bash
# Compile to JavaScript
aeonmi emit demo.ai --emit js -o demo.js

# Compile to canonical AI form
aeonmi emit demo.ai --emit ai -o demo.ai

# Debug: show tokens and AST
aeonmi emit demo.ai --tokens --ast

# Watch mode (recompile on changes)
aeonmi emit demo.ai --watch
```

---

### `run` (File Mode)

**Description**: Execute a single `.ai` file directly.

**Usage**:
```bash
aeonmi run <INPUT> [OPTIONS]
```

**Positional Arguments**:
- `<INPUT>` - Input `.ai` file to execute

**Options**:
- `--out <FILE>` - Intermediate output file path
- `--watch` - Watch and re-run on file changes
- `--native` - Force native VM execution (no JavaScript)
- `--bytecode` - Use bytecode VM (requires `bytecode` feature)
- `--emit-ai <FILE>` - Additionally emit canonical AI form
- `--opt-stats` - Print optimization statistics (bytecode mode)
- `--opt-stats-json` - Emit optimization stats as JSON
- `--disasm` - Disassemble compiled bytecode

**Environment Variables**:
- `AEONMI_NATIVE=1` - Enable native VM mode
- `AEONMI_BYTECODE=1` - Enable bytecode VM mode

**Examples**:
```bash
# Run file (uses Node.js if available)
aeonmi run demo.ai

# Force native VM execution
aeonmi run demo.ai --native

# Use bytecode VM with optimization stats
aeonmi run demo.ai --bytecode --opt-stats

# Watch mode (re-run on changes)
aeonmi run demo.ai --watch

# Emit AI form and run
aeonmi run demo.ai --emit-ai output.ai --native
```

---

### `format`

**Description**: Format `.ai` files according to Aeonmi style guidelines.

**Usage**:
```bash
aeonmi format <INPUTS>... [OPTIONS]
```

**Positional Arguments**:
- `<INPUTS>...` - One or more `.ai` files to format

**Options**:
- `--check` - Check formatting without modifying files

**Examples**:
```bash
# Format files in place
aeonmi format src/main.ai src/engine.ai

# Check formatting (CI mode)
aeonmi format --check src/**/*.ai
```

---

### `lint`

**Description**: Lint `.ai` files for code quality issues.

**Usage**:
```bash
aeonmi lint <INPUTS>... [OPTIONS]
```

**Positional Arguments**:
- `<INPUTS>...` - One or more `.ai` files to lint

**Options**:
- `--fix` - Automatically fix issues where possible

**Examples**:
```bash
# Lint files
aeonmi lint src/main.ai

# Lint and auto-fix
aeonmi lint --fix src/**/*.ai
```

---

## Development Commands

Commands for inspecting and debugging code.

### `tokens`

**Description**: Display the token stream for a file (lexical analysis).

**Usage**:
```bash
aeonmi tokens <INPUT>
```

**Positional Arguments**:
- `<INPUT>` - Input file to tokenize

**Examples**:
```bash
aeonmi tokens demo.ai
```

**Output**: Displays each token with its type and value.

---

### `ast`

**Description**: Display the abstract syntax tree (AST) for a file.

**Usage**:
```bash
aeonmi ast <INPUT>
```

**Positional Arguments**:
- `<INPUT>` - Input file to parse

**Examples**:
```bash
aeonmi ast demo.ai
```

**Output**: Pretty-printed AST structure showing the parse tree.

---

### `edit`

**Description**: Launch the integrated code editor / IDE.

**Usage**:
```bash
aeonmi edit [FILE]
```

**Positional Arguments**:
- `[FILE]` - Optional file to open (opens editor otherwise)

**Examples**:
```bash
# Launch editor
aeonmi edit

# Open specific file
aeonmi edit src/main.ai
```

**Features**:
- Syntax highlighting for `.ai` files
- Real-time error checking
- Integrated REPL

---

### `repl`

**Description**: Start an interactive Read-Eval-Print Loop for Aeonmi.

**Usage**:
```bash
aeonmi repl
```

**Examples**:
```bash
aeonmi repl
```

**REPL Commands**:
- Type Aeonmi expressions and see results immediately
- Multi-line input supported
- Access to full language features

---

## Quantum Commands

Commands for quantum computing operations (requires `quantum` feature).

### `quantum`

**Description**: Execute quantum circuits using various backends.

**Usage**:
```bash
aeonmi quantum <BACKEND> <FILE> [OPTIONS]
```

**Positional Arguments**:
- `<BACKEND>` - Quantum backend to use:
  - `simulator` - Local Titan quantum simulator
  - `qiskit` - IBM Qiskit backend (Python bridge)
  - `cloud` - Cloud quantum hardware (requires credentials)
- `<FILE>` - Aeonmi file containing quantum circuit code

**Options**:
- `--shots <N>` - Number of measurement shots (default: 1024)

**Examples**:
```bash
# Run on local simulator
aeonmi quantum simulator quantum_demo.ai

# Run on Qiskit backend
aeonmi quantum qiskit quantum_demo.ai --shots 2048

# Execute on cloud hardware
aeonmi quantum cloud entanglement.ai
```

**Circuit Export**:
- Automatically converts Aeonmi AST to OpenQASM 2.0
- Dispatches to selected backend
- Returns measurement results

**Requirements**:
- Qiskit backend: Python 3.x with `qiskit` package installed
- Cloud backend: Valid API credentials

---

## Vault Commands

Commands for managing encrypted quantum vaults (requires `quantum-vault` feature).

### `vault init`

**Description**: Initialize a new quantum vault.

**Usage**:
```bash
aeonmi vault init [OPTIONS]
```

**Options**:
- `--path <DIR>` - Vault directory path
- `--algorithm <ALG>` - Encryption algorithm (default: `kyber`)

**Examples**:
```bash
# Initialize vault in default location
aeonmi vault init

# Custom vault location
aeonmi vault init --path ~/my-vault
```

---

### `vault store`

**Description**: Store data in the vault.

**Usage**:
```bash
aeonmi vault store <KEY> <VALUE>
```

**Positional Arguments**:
- `<KEY>` - Unique identifier for the data
- `<VALUE>` - Data to store (encrypted)

**Examples**:
```bash
aeonmi vault store quantum-key "secret data"
```

---

### `vault retrieve`

**Description**: Retrieve data from the vault.

**Usage**:
```bash
aeonmi vault retrieve <KEY>
```

**Positional Arguments**:
- `<KEY>` - Key to retrieve

**Examples**:
```bash
aeonmi vault retrieve quantum-key
```

---

### `vault rotate`

**Description**: Rotate vault encryption keys.

**Usage**:
```bash
aeonmi vault rotate
```

**Examples**:
```bash
aeonmi vault rotate
```

**Note**: Automatically re-encrypts all vault data with new keys.

---

## Shell & Interactive

### `shell`

**Description**: Launch the Neon Shard interactive shell - an integrated environment for Aeonmi development.

**Usage**:
```bash
aeonmi shell
```

**Features**:
- Interactive command-line environment
- File system operations
- Quantum circuit visualization
- Mother AI integration (if enabled)

**Important**: The Aeonmi shell has its own command set and does NOT support standard `cargo` or `aeonmi` commands. Exit the shell to use those commands.

---

## Shell Commands Reference

When inside the Aeonmi shell (`aeonmi shell`), use these commands:

### Navigation Commands

**`pwd`** - Print working directory
```
⟦AEONMI⟧ › pwd
```

**`cd [dir]`** - Change directory
```
⟦AEONMI⟧ › cd /path/to/project
⟦AEONMI⟧ › cd ..
⟦AEONMI⟧ › cd ~
```

**`ls [dir]`** - List directory contents
```
⟦AEONMI⟧ › ls
⟦AEONMI⟧ › ls src/
```

**`mkdir <path>`** - Create directory
```
⟦AEONMI⟧ › mkdir new-project
```

**`mv <src> <dst>`** - Move/rename files
```
⟦AEONMI⟧ › mv old.ai new.ai
```

**`cp <src> <dst>`** - Copy files or directories
```
⟦AEONMI⟧ › cp template.ai project.ai
```

### File Commands

**`cat <file>`** - Display file contents
```
⟦AEONMI⟧ › cat main.ai
```

**`rm <path>`** - Remove file or directory
```
⟦AEONMI⟧ › rm old-file.ai
```

**`edit [--tui] [FILE]`** - Open editor
```
⟦AEONMI⟧ › edit main.ai
⟦AEONMI⟧ › edit --tui main.ai  # TUI mode
```

### Build & Run Commands (Shell)

**`compile <file.ai> [OPTIONS]`** - Compile Aeonmi file

Options:
- `--emit js|ai` - Output format
- `--out FILE` - Output path
- `--no-sema` - Disable semantic analysis

```
⟦AEONMI⟧ › compile demo.ai --emit js --out output.js
⟦AEONMI⟧ › compile demo.ai --emit ai
```

**`run <file.ai> [OPTIONS]`** - Execute file (JavaScript path by default)

Options:
- `--native` - Use native VM execution
- `--out FILE` - Output path

```
⟦AEONMI⟧ › run demo.ai
⟦AEONMI⟧ › run demo.ai --native
⟦AEONMI⟧ › run demo.ai --native --out temp.js
```

**`native-run <file.ai> [--out FILE]`** - Legacy alias for native VM execution
```
⟦AEONMI⟧ › native-run demo.ai
```

### Quantum Commands (Shell)

**`qsim <file.ai> [OPTIONS]`** - Run quantum simulation

Options:
- `--shots NUM` - Number of measurement shots
- `--backend titan|qiskit` - Backend to use

```
⟦AEONMI⟧ › qsim quantum_demo.ai
⟦AEONMI⟧ › qsim quantum_demo.ai --shots 2048 --backend qiskit
```

**`qstate`** - Display quantum system information
```
⟦AEONMI⟧ › qstate
```

**`qgates`** - Show available quantum gates
```
⟦AEONMI⟧ › qgates
```

**`qexample [name]`** - Run quantum examples
```
⟦AEONMI⟧ › qexample
⟦AEONMI⟧ › qexample bell-state
```

### Help & Exit

**`help`** - Show shell help
```
⟦AEONMI⟧ › help
```

**`exit`** - Quit the shell
```
⟦AEONMI⟧ › exit
```

---

## When to Use Shell vs CLI

### Use the Shell (`aeonmi shell`) for:
- ✅ Interactive exploration
- ✅ Quick file operations
- ✅ Running single `.ai` files
- ✅ Quantum circuit experiments
- ✅ Learning and prototyping

### Use the CLI (outside shell) for:
- ✅ Project builds (`aeonmi build`)
- ✅ Running test suites (`aeonmi test`)
- ✅ CI/CD pipelines
- ✅ Cargo integration
- ✅ Production workflows

### Example Workflow:

**Outside Shell** (standard terminal):
```bash
# Build the Aeonmi toolchain
cargo build --release

# Work with projects
cd my-project
aeonmi build
aeonmi test
aeonmi run
```

**Inside Shell** (after running `aeonmi shell`):
```
⟦AEONMI⟧ › cd my-project
⟦AEONMI⟧ › compile main.ai --emit js
⟦AEONMI⟧ › run main.ai --native
⟦AEONMI⟧ › qsim quantum.ai --backend titan
⟦AEONMI⟧ › exit
```

**Note**: To run project commands like `aeonmi build` or `aeonmi test`, you must exit the shell first

---

## Global Options

Options that work with all commands:

### `--config <PATH>`

Override the default configuration file location.

**Default Locations**:
- Windows: `%APPDATA%\aeonmi\config.toml`
- Unix: `~/.config/aeonmi/config.toml`

**Example**:
```bash
aeonmi --config /custom/config.toml build
```

---

### `--debug-titan`

Enable Titan library debug output (overrides config).

**Example**:
```bash
aeonmi --debug-titan run demo.ai
```

---

### `--no-sema`

Disable semantic analysis during compilation.

**Example**:
```bash
aeonmi --no-sema emit demo.ai
```

---

### `--pretty-errors`

Enable pretty-printed error messages with color and context.

**Example**:
```bash
aeonmi --pretty-errors check
```

---

## Feature Flags

Aeonmi supports conditional compilation with Cargo features:

### Available Features

- `full-suite` - All features enabled
- `minimal` - Minimal feature set (default)
- `quantum` - Quantum computing support
- `mother-ai` - Mother AI consciousness integration
- `titan-libraries` - Titan mathematical libraries
- `holographic` - 3D holographic visualization
- `voice` - Voice interface
- `quantum-vault` - Encrypted quantum vault
- `qiskit` - Qiskit Python bridge
- `bytecode` - Bytecode VM execution
- `debug-metrics` - Performance metrics collection

### Using Features

```bash
# Build with specific features
cargo build --features quantum,bytecode

# Build with all features
cargo build --features full-suite

# Run with features
cargo run --features quantum -- quantum simulator demo.ai
```

---

## Project Structure

### Aeonmi.toml Manifest

Required for project commands (`build`, `check`, `test`, `run`).

**Example**:
```toml
[package]
name = "my-project"
version = "0.1.0"

[aeonmi]
entry = "src/main.ai"  # Entry point (default: src/main.ai)
modules = [            # Additional modules to include
    "src/engine.ai",
    "src/logic.ai"
]
tests = [              # Optional: explicit test files
    { name = "unit", path = "tests/unit.ai" }
]
```

### Directory Layout

```
project/
├── Aeonmi.toml       # Project manifest
├── src/
│   ├── main.ai       # Entry point (must have fn main)
│   ├── engine.ai     # Module 1
│   └── logic.ai      # Module 2
└── tests/
    └── arithmetic.ai # Auto-discovered tests
```

---

## Workflow Examples

### Typical Development Workflow

```bash
# 1. Create new project
mkdir my-project
cd my-project

# 2. Create manifest
cat > Aeonmi.toml << EOF
[package]
name = "my-project"
version = "0.1.0"

[aeonmi]
entry = "src/main.ai"
EOF

# 3. Create source files
mkdir src
cat > src/main.ai << EOF
fn main:
    print "Hello, Aeonmi!"
EOF

# 4. Check syntax
aeonmi check

# 5. Run tests
aeonmi test

# 6. Build project
aeonmi build

# 7. Execute
aeonmi run
```

### Quantum Computing Workflow

```bash
# 1. Write quantum circuit
cat > bell_state.ai << EOF
fn create_bell_state:
    qubit q1
    qubit q2
    hadamard q1
    cnot q1, q2
    measure q1
    measure q2
EOF

# 2. Test locally
aeonmi quantum simulator bell_state.ai

# 3. Run on Qiskit
aeonmi quantum qiskit bell_state.ai --shots 4096
```

### File-Based Development

```bash
# Quick script execution
aeonmi run script.ai

# Compile to JavaScript
aeonmi emit script.ai --emit js -o output.js

# Watch mode for development
aeonmi run script.ai --watch --native
```

---

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Compilation error
- `3` - Test failure
- `4` - Missing entry point

---

## Environment Variables

- `AEONMI_NATIVE=1` - Force native VM execution
- `AEONMI_BYTECODE=1` - Enable bytecode VM
- `RUST_LOG=debug` - Enable debug logging
- `NO_COLOR=1` - Disable colored output

---

## Tips & Best Practices

### Performance

- Use `--release` for production builds and benchmarks
- Enable `bytecode` feature for faster execution
- Use `--opt-stats` to identify optimization opportunities

### Development

- Use `--watch` during development for instant feedback
- Enable `--pretty-errors` for better error messages
- Run `aeonmi check` frequently (it's fast!)

### Testing

- Organize tests in the `tests/` directory for auto-discovery
- Use `--filter` to run specific test subsets during development
- Inline tests in modules for unit testing

### Debugging

- Use `aeonmi tokens` to debug lexical issues
- Use `aeonmi ast` to understand parse structure
- Enable `--debug-titan` for Titan library debugging

---

## Getting Help

### Command-Specific Help

```bash
aeonmi <command> --help
```

### Version Information

```bash
aeonmi --version
```

### Documentation

- Build guide: `build.md`
- Language guide: `docs/Aeonmi_Language_Guide.md`
- Architecture: `MOTHER_AI_ARCHITECTURE.md`

---

## Troubleshooting

### "No Aeonmi.toml found"

**Solution**: Create a manifest file or use `--manifest-path`

### "Program must contain a `fn main:` entry point"

**Solution**: Add a `fn main:` function to your entry point file

### Qiskit backend not working

**Solution**: 
```bash
# Install Qiskit
pip install qiskit

# Verify qiskit_runner.py is in project root
ls qiskit_runner.py
```

### Compilation errors with minimal features

**Solution**: Build with required features:
```bash
cargo build --features quantum,bytecode
```

---

*Aeonmi CLI v1.0.0-quantum-consciousness*