# CLI Commands Guide

Complete guide to using the `aeon` command-line interface for building, running, and managing Aeonmi projects.

## Table of Contents

1. [Installation](#installation)
2. [Command Overview](#command-overview)
3. [Project Management](#project-management)
4. [Building & Compilation](#building--compilation)
5. [Running Programs](#running-programs)
6. [Testing](#testing)
7. [Development Tools](#development-tools)
8. [Sandbox Operations](#sandbox-operations)
9. [Editor Integration](#editor-integration)
10. [Configuration](#configuration)

## Installation

### Quick Install
```bash
# Install from releases (recommended)
curl -sSL https://aeonmi.dev/install | bash

# Or using cargo
cargo install aeonmi

# Verify installation
aeon --version
```

### Build from Source
```bash
git clone https://github.com/DarthMetaCrypro/Aeonmi.git
cd Aeonmi
cargo build --release
cargo install --path .
```

## Command Overview

The `aeon` CLI provides a comprehensive set of commands for Aeonmi development:

```bash
aeon [COMMAND] [OPTIONS] [ARGS]

COMMANDS:
    new         Create a new Aeonmi project
    init        Initialize an existing directory as Aeonmi project
    build       Compile Aeonmi source files
    run         Run compiled Aeonmi programs
    test        Execute tests
    check       Check code for errors without building
    clean       Remove build artifacts
    doc         Generate documentation
    fmt         Format source code
    lint        Run linter on source code
    
    # Quantum-specific commands
    quantum     Quantum circuit operations
    simulate    Run quantum simulations
    
    # Development tools
    debug       Debug compilation stages
    explain     Explain compilation process (learning mode)
    benchmark   Run performance benchmarks
    
    # Project management
    add         Add dependencies
    remove      Remove dependencies
    update      Update dependencies
    package     Package project for distribution
    
    # Sandbox operations
    sandbox     Manage sandboxed environments
    
    # Editor integration
    editor      Launch Shard Editor
    lsp         Language server operations
    
    # Configuration
    config      Manage configuration settings
    version     Show version information
    help        Show help information
```

## Project Management

### Creating New Projects

```bash
# Create a new project
aeon new my_quantum_app
cd my_quantum_app

# Create with specific template
aeon new my_project --template quantum-algorithm
aeon new my_project --template hybrid-app
aeon new my_project --template educational

# Create library project
aeon new my_lib --lib

# Create project with specific quantum backend
aeon new my_project --quantum-backend qiskit
```

### Initializing Existing Directories

```bash
# Initialize current directory
aeon init

# Initialize with specific configuration
aeon init --name my_project --lib
aeon init --quantum-backend simulator
```

### Project Structure

A typical Aeonmi project structure:

```
my_project/
├── Aeonmi.toml          # Project configuration
├── src/
│   ├── main.aeon        # Main source file
│   ├── lib.aeon         # Library code
│   └── quantum/
│       ├── algorithms.aeon
│       └── circuits.aeon
├── tests/
│   ├── integration.aeon
│   └── quantum_tests.aeon
├── examples/
│   └── demo.aeon
├── docs/
│   └── README.md
└── target/              # Build artifacts (auto-generated)
    ├── debug/
    └── release/
```

## Building & Compilation

### Basic Building

```bash
# Build current project
aeon build

# Build specific file
aeon build src/main.aeon

# Build with release optimizations
aeon build --release

# Build for specific target
aeon build --target wasm32-unknown-unknown
aeon build --target x86_64-pc-windows-msvc
```

### Advanced Build Options

```bash
# Build with verbose output
aeon build --verbose

# Build with specific features
aeon build --features "quantum,optimization"

# Build without quantum backend
aeon build --no-quantum

# Build with specific optimization level
aeon build --opt-level 2

# Build with debug information
aeon build --debug-info

# Show compilation stages (learning mode)
aeon build --explain
```

### Cross-Compilation

```bash
# List available targets
aeon build --list-targets

# Add target support
aeon target add wasm32-unknown-unknown

# Build for target
aeon build --target wasm32-unknown-unknown --release
```

## Running Programs

### Basic Execution

```bash
# Run main program
aeon run

# Run specific file
aeon run src/examples/demo.aeon

# Run with command line arguments
aeon run -- --input data.txt --iterations 100

# Run with environment variables
aeon run --env QUANTUM_BACKEND=qiskit
```

### Quantum-Specific Running

```bash
# Run with specific quantum backend
aeon run --quantum-backend simulator
aeon run --quantum-backend qiskit --shots 1000

# Run with quantum debugging
aeon run --quantum-debug --show-circuits

# Run with measurement statistics
aeon run --quantum-stats

# Run in learning mode (verbose quantum operations)
aeon run --verbose --explain-quantum
```

### Development Running

```bash
# Run with auto-rebuild on file changes
aeon run --watch

# Run with profiling
aeon run --profile

# Run with memory tracking
aeon run --track-memory

# Run with timeout
aeon run --timeout 30s
```

## Testing

### Running Tests

```bash
# Run all tests
aeon test

# Run specific test file
aeon test tests/quantum_tests.aeon

# Run tests matching pattern
aeon test --filter grover

# Run with verbose output
aeon test --verbose

# Run tests in parallel
aeon test --parallel 4
```

### Quantum Testing

```bash
# Run quantum tests with specific backend
aeon test --quantum-backend simulator

# Run tests with multiple shots for statistical accuracy
aeon test --quantum-shots 1000

# Run tests with noise simulation
aeon test --quantum-noise 0.01

# Generate test coverage for quantum operations
aeon test --coverage --quantum-coverage
```

### Test Configuration

```bash
# Run tests without capturing output
aeon test --nocapture

# Run ignored tests
aeon test --ignored

# Run with specific test threads
aeon test --test-threads 1

# Generate test report
aeon test --report-format json > test_results.json
```

## Development Tools

### Code Checking

```bash
# Check for compilation errors
aeon check

# Check specific files
aeon check src/quantum/*.aeon

# Check with all features enabled
aeon check --all-features

# Fast syntax-only check
aeon check --syntax-only
```

### Code Formatting

```bash
# Format all source files
aeon fmt

# Format specific files
aeon fmt src/main.aeon

# Check formatting without modifying files
aeon fmt --check

# Format with specific style
aeon fmt --style compact
```

### Linting

```bash
# Run linter
aeon lint

# Run with auto-fix
aeon lint --fix

# Run specific lint rules
aeon lint --rules quantum-best-practices,performance

# Generate lint report
aeon lint --report-format json
```

### Debugging

```bash
# Debug lexer output
aeon debug lexer src/main.aeon

# Debug parser AST
aeon debug parser src/main.aeon --show-ast

# Debug semantic analysis
aeon debug semantic src/main.aeon

# Debug bytecode generation
aeon debug codegen src/main.aeon

# Debug quantum circuit compilation
aeon debug quantum src/main.aeon --show-circuits
```

### Learning Mode

```bash
# Explain compilation process
aeon explain src/main.aeon

# Explain with detailed output
aeon explain src/main.aeon --detailed

# Explain specific compilation stage
aeon explain src/main.aeon --stage semantic

# Generate compilation report
aeon explain src/main.aeon --report > compilation_report.html
```

## Sandbox Operations

### Managing Sandboxes

```bash
# Create new sandbox
aeon sandbox create my_sandbox

# List all sandboxes
aeon sandbox list

# Enter sandbox environment
aeon sandbox enter my_sandbox

# Remove sandbox
aeon sandbox remove my_sandbox

# Clean sandbox artifacts
aeon sandbox clean my_sandbox
```

### Sandbox Execution

```bash
# Execute command in sandbox
aeon sandbox exec my_sandbox "aeon run examples/demo.aeon"

# Execute with timeout
aeon sandbox exec my_sandbox --timeout 30s "aeon test"

# Execute with resource limits
aeon sandbox exec my_sandbox --memory 256MB --cpu 50% "aeon build"

# Monitor sandbox status
aeon sandbox status my_sandbox
```

### Sandbox Security

```bash
# Create sandbox with specific permissions
aeon sandbox create secure_env --no-network --read-only

# Execute with restricted environment
aeon sandbox exec secure_env --no-file-write "aeon run safe_program.aeon"

# Kill running processes in sandbox
aeon sandbox kill my_sandbox <process_id>
```

## Editor Integration

### Shard Editor

```bash
# Launch web-based editor
aeon editor

# Open specific project in editor
aeon editor my_project/

# Launch editor on specific port
aeon editor --port 8080

# Launch editor with specific features
aeon editor --quantum-visualizer --debug-mode
```

### Language Server

```bash
# Start LSP server
aeon lsp start --port 9257

# Check LSP status
aeon lsp status

# Stop LSP server
aeon lsp stop

# Configure LSP features
aeon lsp config --quantum-completion --hover-docs
```

## Configuration

### Project Configuration

```bash
# Set project configuration
aeon config set project.name "My Quantum App"
aeon config set project.version "1.0.0"
aeon config set quantum.backend "qiskit"

# Get configuration value
aeon config get quantum.backend

# List all configuration
aeon config list

# Reset configuration to defaults
aeon config reset
```

### Global Configuration

```bash
# Set global configuration
aeon config set --global editor.theme "dark"
aeon config set --global quantum.default_shots 1000

# Get global configuration
aeon config get --global editor.theme

# Edit configuration file directly
aeon config edit
aeon config edit --global
```

### Configuration File (Aeonmi.toml)

```toml
[project]
name = "my_quantum_app"
version = "1.0.0"
edition = "2023"
authors = ["Your Name <you@example.com>"]

[dependencies]
quantum_algorithms = "1.0"
math_utils = "0.5"

[quantum]
backend = "qiskit"
default_shots = 1000
noise_model = "ibm_quantum"

[build]
optimization_level = 2
target = "native"
features = ["quantum", "visualization"]

[dev]
debug_info = true
test_threads = 4

[editor]
theme = "quantum"
show_circuit_diagrams = true
enable_auto_completion = true
```

## Package Management

### Dependencies

```bash
# Add dependency
aeon add quantum_algorithms
aeon add math_utils@0.5.0

# Add development dependency
aeon add --dev test_framework

# Remove dependency
aeon remove quantum_algorithms

# Update dependencies
aeon update
aeon update quantum_algorithms

# List dependencies
aeon list
```

### Publishing

```bash
# Package project
aeon package

# Publish to registry
aeon publish --registry quantum-registry

# Publish with dry run
aeon publish --dry-run

# Install published package
aeon install my_quantum_lib
```

## Quantum-Specific Commands

### Circuit Operations

```bash
# Compile quantum circuit
aeon quantum compile src/circuits/teleportation.aeon

# Visualize circuit
aeon quantum visualize src/circuits/grover.aeon --format svg

# Optimize circuit
aeon quantum optimize src/circuits/qft.aeon --level 2

# Simulate circuit
aeon quantum simulate src/circuits/demo.aeon --shots 1000
```

### Backend Management

```bash
# List available quantum backends
aeon quantum backends

# Configure backend
aeon quantum backend configure qiskit --token YOUR_TOKEN

# Test backend connection
aeon quantum backend test qiskit

# Get backend status
aeon quantum backend status
```

## Performance and Monitoring

### Benchmarking

```bash
# Run benchmarks
aeon benchmark

# Run specific benchmark
aeon benchmark quantum_algorithms

# Compare with baseline
aeon benchmark --compare baseline.json

# Generate benchmark report
aeon benchmark --report-format html > benchmark.html
```

### Profiling

```bash
# Profile compilation
aeon profile build src/main.aeon

# Profile execution
aeon profile run src/main.aeon

# Profile memory usage
aeon profile memory src/main.aeon

# Generate profiling report
aeon profile report --format flamegraph
```

## Environment Variables

Aeonmi CLI respects several environment variables:

```bash
# Quantum backend configuration
export AEON_QUANTUM_BACKEND=qiskit
export AEON_QUANTUM_TOKEN=your_token_here

# Build configuration
export AEON_TARGET_DIR=custom_target
export AEON_OPT_LEVEL=3

# Editor configuration
export AEON_EDITOR_PORT=8080
export AEON_EDITOR_THEME=dark

# Development
export AEON_LOG_LEVEL=debug
export AEON_VERBOSE=1
```

## Command Examples

### Complete Workflow Example

```bash
# Create new quantum project
aeon new quantum_teleportation --template quantum-algorithm
cd quantum_teleportation

# Add dependencies
aeon add quantum_utils

# Write code in src/main.aeon
cat > src/main.aeon << EOF
use quantum;

fn main() {
    println!("Quantum Teleportation Demo");
    
    let alice = qubit(1);  // Alice's qubit to teleport
    let bob = qubit(0);    // Bob's target qubit
    let ancilla = qubit(0); // Entanglement ancilla
    
    // Create Bell pair between ancilla and Bob
    hadamard(ancilla);
    cnot(ancilla, bob);
    
    // Alice's operations
    cnot(alice, ancilla);
    hadamard(alice);
    
    // Measure Alice's qubits
    let m1 = measure(alice);
    let m2 = measure(ancilla);
    
    // Apply corrections to Bob's qubit
    if m2.value { pauli_x(bob); }
    if m1.value { pauli_z(bob); }
    
    // Verify teleportation
    let result = measure(bob);
    println!("Teleportation result: {}", result.value);
}
EOF

# Check code
aeon check

# Build with learning mode
aeon build --explain

# Run with quantum backend
aeon run --quantum-backend simulator --verbose

# Test the implementation
aeon test

# Format code
aeon fmt

# Generate documentation
aeon doc

# Package for distribution
aeon package
```

## Getting Help

```bash
# General help
aeon help

# Command-specific help
aeon help build
aeon help run
aeon help quantum

# Show all available commands
aeon --help

# Show version information
aeon version --verbose
```

---

This CLI guide covers all aspects of using the Aeonmi toolchain. For language-specific help, see the [Language Reference](../language/reference.md), and for quantum programming guidance, check the [Quantum Programming Guide](../quantum/basics.md).