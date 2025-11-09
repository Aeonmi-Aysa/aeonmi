# Contributing to Aeonmi

Welcome to the Aeonmi quantum programming platform! We're excited to have you contribute to the future of quantum computing.

## 🌟 Overview

Aeonmi is an open-source quantum programming language and platform designed to make quantum computing accessible to developers, students, and researchers. Your contributions help build the foundation for practical quantum applications.

### Project Structure

```
aeonmi/
├── src/                    # Core language implementation
│   ├── compiler/          # Quantum circuit compiler
│   ├── runtime/           # Quantum execution runtime
│   ├── core/              # Language core features
│   └── cli/               # Command-line interface
├── docs/                  # Documentation
├── examples/              # Example projects
├── tests/                 # Comprehensive test suite
├── benchmarks/            # Performance benchmarks
└── tools/                 # Development tools
```

## 🎯 Ways to Contribute

### 🐛 Bug Reports
Found a bug? Help us fix it!
- Check existing issues first
- Provide minimal reproduction case
- Include system information
- Describe expected vs actual behavior

### ✨ Feature Requests
Have ideas for new features?
- Describe the use case
- Explain the quantum advantage
- Consider implementation complexity
- Discuss with community first

### 📝 Documentation
Documentation is crucial for adoption:
- Fix typos and improve clarity
- Add examples and tutorials
- Translate to other languages
- Create video walkthroughs

### 💻 Code Contributions
Ready to write code?
- Start with "good first issue" labels
- Follow our coding standards
- Add comprehensive tests
- Update documentation

### 🧪 Example Projects
Show real-world applications:
- Implement quantum algorithms
- Create educational examples
- Build practical applications
- Benchmark performance

## 🚀 Getting Started

### 1. Development Setup

```bash
# Clone the repository
git clone https://github.com/aeonmi/aeonmi.git
cd aeonmi

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install development dependencies
cargo install --path .
cargo install cargo-watch
cargo install cargo-tarpaulin  # For test coverage

# Build the project
cargo build

# Run tests
cargo test

# Run in development mode
cargo run -- --help
```

### 2. IDE Setup

#### VS Code (Recommended)
```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension ms-vscode.vscode-json
code --install-extension yzhang.markdown-all-in-one

# Open workspace
code .
```

#### Other Editors
- **Vim/Neovim**: Use rust.vim and coc-rust-analyzer
- **Emacs**: Use rust-mode and lsp-mode
- **IntelliJ**: Use Rust plugin

### 3. Understanding the Codebase

#### Core Components

**Compiler Pipeline** (`src/compiler/`)
```rust
// High-level compilation flow
Source Code → Lexer → Parser → AST → 
Quantum Circuit → Optimizer → Target Code
```

**Runtime System** (`src/runtime/`)
```rust
// Quantum execution environment
Classical Driver ↔ Quantum Backend ↔ Hardware/Simulator
```

**Language Features** (`src/core/`)
```rust
// Core language constructs
quantum fn example() {
    let q = qubit(0);     // Qubit allocation
    hadamard(q);          // Quantum operations
    let result = measure(q); // Measurement
}
```

## 📋 Contribution Guidelines

### Code Style

#### Rust Code Standards
Follow Rust's official style guide with these additions:

```rust
// Use descriptive names for quantum operations
quantum fn create_bell_state(control: Qubit, target: Qubit) -> (Qubit, Qubit) {
    hadamard(control);
    cnot(control, target);
    (control, target)
}

// Document quantum transformations in comments
/// Applies Grover's oracle for marked item `target`
/// 
/// Quantum State Transformation:
/// |ψ⟩ → O|ψ⟩ where O flips amplitude of |target⟩
quantum fn grover_oracle(qubits: &mut [Qubit], target: usize) {
    // Implementation...
}

// Use type annotations for clarity
let measurement_result: MeasurementResult = measure(qubit);
let probability: f64 = measurement_result.probability;

// Handle errors explicitly
match quantum_operation(qubit) {
    Ok(result) => process_result(result),
    Err(QuantumError::DecoherenceError) => handle_decoherence(),
    Err(e) => return Err(e.into()),
}
```

#### Quantum-Specific Conventions

```rust
// Qubit naming: use descriptive names
let alice_qubit = qubit(0);
let bob_qubit = qubit(0);
let ancilla = qubit(0);

// Function naming: verb_object pattern
quantum fn apply_hadamard(q: Qubit) { ... }
quantum fn measure_all_qubits(qubits: &[Qubit]) -> Vec<bool> { ... }
quantum fn create_ghz_state(qubits: &mut [Qubit]) { ... }

// Constants: use quantum physics conventions
const SQRT_2: f64 = 1.4142135623730951;
const PI: f64 = std::f64::consts::PI;
const HADAMARD_ANGLE: f64 = PI / 4.0;
```

### Testing Standards

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bell_state_creation() {
        let (q1, q2) = create_bell_state();
        
        // Test multiple runs for statistical validation
        let mut correlation_count = 0;
        let trials = 100;
        
        for _ in 0..trials {
            let (fresh_q1, fresh_q2) = create_bell_state();
            let r1 = measure(fresh_q1).value;
            let r2 = measure(fresh_q2).value;
            
            if r1 == r2 {
                correlation_count += 1;
            }
        }
        
        // Bell states should be perfectly correlated
        assert_eq!(correlation_count, trials, 
                   "Bell states must be perfectly correlated");
    }
    
    #[test]
    fn test_quantum_measurement_probabilities() {
        let mut zero_count = 0;
        let trials = 1000;
        
        for _ in 0..trials {
            let q = qubit(0);
            hadamard(q);  // 50/50 superposition
            
            if !measure(q).value {
                zero_count += 1;
            }
        }
        
        let zero_probability = zero_count as f64 / trials as f64;
        
        // Should be approximately 50% with statistical tolerance
        assert!((zero_probability - 0.5).abs() < 0.05,
                "Hadamard should create 50/50 superposition, got {}%", 
                zero_probability * 100.0);
    }
}
```

#### Integration Tests
```rust
// tests/integration/algorithms.rs
use aeonmi::prelude::*;

#[test]
fn test_grovers_algorithm_integration() {
    let database_size = 16;
    let target_item = 10;
    
    // Run Grover's algorithm
    let result = grovers_search(database_size, target_item);
    
    // Should find target with high probability
    assert_eq!(result, target_item, 
               "Grover's algorithm should find the target item");
}

#[test]
fn test_quantum_teleportation_fidelity() {
    let test_states = vec![
        ("Classical |0⟩", |q: Qubit| { /* already |0⟩ */ }),
        ("Classical |1⟩", |q: Qubit| { pauli_x(q); }),
        ("Superposition |+⟩", |q: Qubit| { hadamard(q); }),
    ];
    
    for (name, prepare_state) in test_states {
        let success_rate = test_teleportation_protocol(prepare_state, 100);
        assert!(success_rate > 0.95, 
                "Teleportation of {} should succeed >95% of the time, got {}%",
                name, success_rate * 100.0);
    }
}
```

### Documentation Standards

#### Code Documentation
```rust
/// Creates a maximally entangled Bell state between two qubits
///
/// # Arguments
/// * `control` - The control qubit (will be in superposition after operation)
/// * `target` - The target qubit (will be entangled with control)
///
/// # Returns
/// A tuple containing the entangled qubits
///
/// # Quantum State Transformation
/// ```text
/// Input:  |00⟩
/// Output: (|00⟩ + |11⟩)/√2
/// ```
///
/// # Examples
/// ```rust
/// let (alice, bob) = create_bell_state(qubit(0), qubit(0));
/// let alice_result = measure(alice);
/// let bob_result = measure(bob);
/// assert_eq!(alice_result.value, bob_result.value); // Always correlated
/// ```
///
/// # Performance
/// - Gate count: 2 (1 Hadamard + 1 CNOT)
/// - Circuit depth: 2
/// - Fidelity: 100% (in ideal simulator)
quantum fn create_bell_state(control: Qubit, target: Qubit) -> (Qubit, Qubit) {
    hadamard(control);
    cnot(control, target);
    (control, target)
}
```

#### README Templates
Each module should include:

```markdown
# Module Name

Brief description of the module's purpose and quantum advantage.

## Features
- Feature 1: Description and benefit
- Feature 2: Description and benefit

## Usage
```rust
// Basic usage example
use aeonmi::module_name::*;

quantum fn example() {
    // Example code
}
```

## Performance
- Time complexity: O(...)
- Space complexity: O(...)
- Quantum advantage: Description

## Implementation Notes
Key design decisions and trade-offs.
```

### Commit Message Format

Follow conventional commits with quantum-specific scopes:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/modifications
- `perf`: Performance improvements
- `refactor`: Code refactoring
- `ci`: CI/CD changes

**Scopes:**
- `compiler`: Quantum circuit compilation
- `runtime`: Quantum execution runtime
- `lang`: Language features
- `cli`: Command-line interface
- `docs`: Documentation
- `examples`: Example projects
- `quantum`: Quantum algorithm implementations

**Examples:**
```
feat(quantum): implement Grover's search algorithm

Add complete implementation of Grover's algorithm with oracle
construction and amplitude amplification. Includes statistical
testing and performance benchmarks.

Closes #123

fix(runtime): correct measurement probability calculation

The probability calculation was using amplitude instead of 
amplitude squared. This affected measurement statistics
in superposition states.

docs(examples): add quantum teleportation tutorial

Complete step-by-step tutorial with explanations of:
- Bell state preparation
- Quantum measurements
- Classical communication
- Correction operations

perf(compiler): optimize quantum circuit compilation

Reduce compilation time by 40% through improved gate
fusion and dead code elimination algorithms.

Benchmark results:
- Small circuits: 250ms → 150ms
- Large circuits: 2.1s → 1.3s
```

## 🔄 Development Workflow

### 1. Issue Assignment
- Browse open issues
- Comment to express interest
- Wait for assignment before starting work
- Ask questions if requirements unclear

### 2. Feature Development
```bash
# Create feature branch
git checkout -b feat/grover-algorithm

# Make changes with atomic commits
git add src/algorithms/grover.rs
git commit -m "feat(quantum): add Grover oracle implementation"

git add tests/grover_tests.rs
git commit -m "test(quantum): add Grover algorithm tests"

git add docs/algorithms/grover.md
git commit -m "docs(quantum): add Grover algorithm documentation"

# Push to your fork
git push origin feat/grover-algorithm
```

### 3. Pull Request Process
1. **Create PR** with descriptive title and body
2. **Link issues** using "Closes #123" syntax
3. **Request review** from appropriate maintainers
4. **Address feedback** promptly and thoroughly
5. **Ensure CI passes** all tests and checks
6. **Update documentation** if needed
7. **Rebase if requested** to maintain clean history

### 4. Code Review Guidelines

#### For Contributors
- **Self-review first**: Check your own code thoroughly
- **Write clear descriptions**: Explain what and why
- **Include examples**: Show how to use new features
- **Test edge cases**: Consider unusual inputs
- **Document breaking changes**: Note API modifications

#### For Reviewers
- **Be constructive**: Suggest improvements, don't just criticize
- **Ask questions**: Understand the rationale behind decisions
- **Test locally**: Verify functionality works as described
- **Check documentation**: Ensure changes are documented
- **Consider performance**: Look for optimization opportunities

## 🎯 Areas for Contribution

### 🚀 High Priority

#### Quantum Algorithms
- **Shor's Algorithm**: Integer factorization
- **Quantum Simulation**: Hamiltonian evolution
- **VQE**: Variational quantum eigensolver
- **QAOA**: Quantum approximate optimization

#### Language Features
- **Type System**: Enhanced quantum type checking
- **Error Handling**: Quantum-aware error propagation
- **Debugging**: Quantum state inspection tools
- **IDE Support**: Language server protocol implementation

#### Performance Optimization
- **Circuit Optimization**: Advanced gate fusion algorithms
- **Memory Management**: Efficient qubit allocation
- **Parallel Execution**: Multi-threaded quantum simulation
- **Hardware Integration**: Real quantum device support

### 🌟 Medium Priority

#### Developer Tools
- **Package Manager**: Quantum library ecosystem
- **Testing Framework**: Quantum-specific test utilities
- **Profiler**: Quantum performance analysis
- **Linter**: Code quality checking

#### Educational Resources
- **Interactive Tutorials**: Web-based learning platform
- **Video Content**: Algorithm explanations
- **Workshop Materials**: Conference and classroom content
- **Translations**: Documentation in multiple languages

#### Ecosystem Integration
- **Python Bindings**: Use Aeonmi from Python
- **JavaScript Interface**: Web-based quantum programming
- **Cloud Integration**: AWS, IBM, Google quantum services
- **Container Support**: Docker and Kubernetes deployment

### 🔧 Specialized Areas

#### Research Contributions
- **Novel Algorithms**: New quantum algorithm implementations
- **Optimization Techniques**: Compiler improvement research
- **Benchmarking**: Performance comparison studies
- **Hardware Analysis**: Real device characterization

#### Community Building
- **Events**: Organize meetups and conferences
- **Mentorship**: Guide new contributors
- **Partnerships**: Academic and industry collaboration
- **Standards**: Contribute to quantum computing standards

## 📊 Contribution Recognition

### Contributor Levels

#### 🌱 New Contributor
- First successful PR merged
- Added to contributors list
- Welcome package with stickers

#### 🌿 Regular Contributor  
- 5+ merged PRs or significant contributions
- Invitation to contributor Discord channel
- Recognition in release notes

#### 🌳 Core Contributor
- 20+ merged PRs or major feature contributions
- Commit access to repository
- Invitation to monthly planning meetings

#### 🏆 Maintainer
- Sustained high-quality contributions
- Code review responsibilities
- Technical decision-making authority

### Recognition Programs

#### Monthly Spotlight
Outstanding contributors featured in:
- Project newsletter
- Social media recognition
- Conference presentation opportunities

#### Annual Awards
- **Innovation Award**: Novel quantum algorithm implementations
- **Education Award**: Outstanding educational contributions
- **Community Award**: Exceptional community building efforts

## 📞 Getting Help

### Communication Channels

#### Discord Server
- **#general**: General discussion
- **#development**: Development questions
- **#quantum-theory**: Quantum computing theory
- **#help**: Get unstuck with contributions

#### GitHub Discussions
- **Ideas**: Feature requests and brainstorming
- **Q&A**: Technical questions
- **Show and Tell**: Share your quantum projects

#### Office Hours
- **Weekly**: Thursdays 3-4 PM UTC
- **Platform**: Discord voice channel
- **Topics**: Any contribution-related questions

### Mentorship Program

#### For New Contributors
- Paired with experienced contributor
- Guided through first PR process
- Regular check-ins and feedback
- Access to private mentorship channel

#### For Experienced Developers
- Quantum computing concepts introduction
- Algorithm implementation guidance
- Best practices for quantum programming
- Research opportunity discussions

## 🎉 Thank You!

Your contributions make Aeonmi better for everyone in the quantum computing community. Whether you're fixing a typo, implementing a new algorithm, or helping someone learn quantum programming, you're building the future of quantum computing.

### Quick Start Checklist
- [ ] ⭐ Star the repository
- [ ] 🍴 Fork the project  
- [ ] 📋 Pick an issue or create one
- [ ] 🔧 Set up development environment
- [ ] 💻 Make your changes
- [ ] ✅ Add tests and documentation
- [ ] 🚀 Submit a pull request

Welcome to the quantum future! 🌟⚛️

---

*This contributing guide is itself open source. If you see ways to improve it, please submit a PR!*