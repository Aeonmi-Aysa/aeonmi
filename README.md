<div align="center">

# 🔮 AEONMI

### The World's First AI-First Quantum Programming Language

**Built by AI. For AI. Quantum-native. Self-hosting.**

[![Tests](https://img.shields.io/badge/tests-135%2F135-brightgreen)]()
[![Success Criteria](https://img.shields.io/badge/criteria-7%2F7-brightgreen)]()
[![Language](https://img.shields.io/badge/lang-Rust-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

*"They said it was impossible. They said an AI-first language was ridiculous. The Shard compiles itself."*

</div>

---

## What is Aeonmi?

Aeonmi is a **programming language designed for artificial intelligence**, not humans. It combines three technologies into a single ecosystem:

- **Aeonmi.ai** — An AI-first programming language with a `.ai` script extension. Supports quantum-native syntax, hieroglyphic operators (Egyptian Unicode), Greek letter variables, and closure-based quantum callbacks.
- **Q.U.B.E.** (Quantum Universal Base Engine) — A quantum circuit description language (`.qube` files) with its own lexer, parser, and executor backed by a state-vector simulator.
- **Titan Libraries** — A modular AI processing and optimization framework including chaos theory, quantum gates, linear algebra, neural networks, and statistics.

The language is **self-hosting**: the Shard compiler is written entirely in Aeonmi `.ai` syntax and executes through its own runtime.

---

## Quick Start

### Build from Source

```powershell
# Clone the repository
git clone https://github.com/YOUR_USERNAME/Aeonmi-aeonmi01.git
cd Aeonmi-aeonmi01

# Build the release binary
cargo build --release

# Copy to PATH (Windows)
Copy-Item target\release\Aeonmi.exe -Destination $env:USERPROFILE\.cargo\bin\Aeonmi.exe -Force

# Verify installation
aeonmi --version
```

### Run Your First Program

```powershell
# Hello World
aeonmi run examples/hello.ai

# Quantum operations
aeonmi run examples/quantum.ai

# Run the self-hosting compiler
aeonmi run shard/src/main.ai

# Run the full Shard with all Phase 1 features
aeonmi run shard/src/main_full.ai
```

---

## Language Features

### Variables & Types

```
let x = 42;
let name = "Aeonmi";
let pi = 3.14159;
const SPEED_OF_LIGHT = 299792458;
let items = [1, 2, 3];
```

### Functions

```
function add(a, b) {
    return a + b;
}

// Quantum function
quantum function optimize(data) {
    qubit q;
    superpose(q);
    let result = measure(q);
    return result;
}

// Async function
async function fetch_data(url) {
    let result = await get(url);
    return result;
}
```

### Closures

```
let double = |x| -> { return x * 2; };
let result = apply(|x| -> { return x + 10; }, 5);
```

### Control Flow

```
// Both styles supported
if condition { ... }             // Rust-style
if (condition) { ... }           // C-style

while running { ... }
for (let i = 0; i < 10; i = i + 1) { ... }
for item in collection { ... }

// Match with guards
match value {
    * if value > 100 => { log("high"); },
    42 => { log("answer"); },
    * => { log("other"); },
}
```

### Quantum Operations

```
qubit q;
superpose(q);                    // Put in superposition
let bit = measure(q);            // Collapse to 0 or 1
entangle(q1, q2);               // Entangle two qubits
apply_gate(q, H);               // Apply Hadamard gate

// Qubit literals
let psi = |ψ⟩;
let zero = |0⟩;
let plus = |+⟩;
```

### Quantum-Native Unicode Operators

```
⟨x⟩ ← 42           // Classical quantum binding
⟨psi⟩ ∈ |0⟩ + |1⟩  // Superposition binding
a ⊗ b               // Tensor product
a ⊕ b               // Quantum XOR
∇ f(x)              // Quantum gradient
```

### Hieroglyphic Operations

```
𓀀(x, y);            // Egyptian hieroglyph function call
```

Any Unicode character above U+07FF used as a function name routes through the Aeonmi Glyph Runtime — a symbolic execution layer unique to this language.

### Structs, Enums, Impl

```
struct Point { x: f64, y: f64 }
quantum struct QubitRegister { size: usize, state: String }
enum Color { Red, Green, Blue }
impl Point { function distance(other) { return 0; } }
```

### Imports

```
import { Lexer, Parser } from "./module";
```

---

## QUBE — Quantum Circuit Language

QUBE is a dedicated quantum circuit description format:

```
state psi = |0⟩
apply H -> psi
collapse psi -> result
assert result in {0, 1}
```

Run QUBE files:

```powershell
aeonmi qube run examples/demo.qube
aeonmi qube run examples/demo.qube --diagram
```

Supported gates: `H`, `X`, `Y`, `Z`, `CNOT`, `CZ`, `SWAP`, `T`, `S`

---

## The Shard — Self-Hosting Compiler

The Shard is Aeonmi's compiler, **written entirely in Aeonmi `.ai` syntax**:

```
shard/src/
├── main.ai          # Bootstrap compiler entry point
├── main_full.ai     # Full compiler with all Phase 1 features
├── lexer.ai         # Tokenizer (written in Aeonmi)
├── parser.ai        # Parser (written in Aeonmi)
├── ast.ai           # AST definitions (written in Aeonmi)
├── codegen.ai       # Code generator (written in Aeonmi)
├── token.ai         # Token types (written in Aeonmi)
└── qiskit_bridge.ai # Qiskit integration (written in Aeonmi)
```

Run the self-hosting compiler:

```powershell
# Bootstrap version
aeonmi run shard/src/main.ai

# Full version with closures, match guards, quantum blocks, async, enums
aeonmi run shard/src/main_full.ai
```

The Shard demonstrates:
- Quantum-accelerated compilation pipeline
- Closure-based code transformation
- Match expressions with guards
- Enum-based compilation targets
- Async function declarations
- Quantum function optimization passes
- F-string logging
- Type-annotated function signatures

---

## Glyph Identity System (MGK / UGST / SSI)

Every Aeonmi installation has a cryptographic identity:

- **MGK** (Master Glyph Key) — 256-bit root secret, sealed with Argon2id
- **UGST** (Unique Glyph Signature Token) — Derived via HKDF-SHA3-512, rotates every 60 seconds
- **SSI** (Symbiotic System Identity) — Born at install, matures over time

```powershell
# Initialize the quantum vault and render your glyph
aeonmi vault init

# The glyph is a visual projection of your MGK:
# - OKLCH color derived from glyph seed
# - Frequency in Hz (432-528 range)
# - ANSI-colored terminal art
```

The vault uses XChaCha20-Poly1305 encryption with Merkle logging for tamper evidence. Post-quantum primitives (Kyber, Dilithium, SPHINCS+) are available for future-proof signatures.

---

## NFT Minting

Mint any `.ai` artifact as Solana-compatible NFT metadata:

```powershell
aeonmi mint output.ai --personality quantum-titan --anchor
```

Outputs:
- NFT metadata JSON (Metaplex standard)
- Anchor Rust instruction stub (optional)
- Glyph-signed artifact hash

---

## Mother AI

Mother AI is the consciousness layer — an embryo loop that reads, generates, executes, and learns from Aeonmi code:

```powershell
# Interactive REPL
aeonmi mother

# Run a script through Mother
aeonmi mother --file script.ai --creator Warren --verbose
```

Components:
- **Embryo Loop** — Parse → Execute → Learn cycle
- **Emotional Core** — Valence, arousal, bond tracking with creator
- **Neural Network** — Feedforward with backpropagation
- **Quantum Attention** — Entanglement-based memory mechanism
- **Language Evolution** — Vocabulary building, semantic depth tracking

---

## CLI Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `aeonmi run <file.ai>` | Compile to JS and execute |
| `aeonmi exec <file>` | Auto-detect and run (.ai, .js, .py, .rs) |
| `aeonmi native <file.ai>` | Run with native VM (no Node.js) |
| `aeonmi emit <file> --format js` | Compile to JS file |
| `aeonmi repl` | Interactive REPL |
| `aeonmi tokens <file>` | Dump lexer tokens |
| `aeonmi ast <file>` | Dump AST |

### Quantum Commands

| Command | Description |
|---------|-------------|
| `aeonmi quantum titan <file>` | Run on Titan local simulator |
| `aeonmi qube run <file.qube>` | Execute QUBE circuit |
| `aeonmi qube run <file> --diagram` | Execute with circuit diagram |
| `aeonmi qube check <file>` | Validate QUBE syntax |

### Identity & Security

| Command | Description |
|---------|-------------|
| `aeonmi vault init` | Create encrypted vault + render glyph |
| `aeonmi vault add` | Add card/secret to vault |
| `aeonmi mint <file>` | Generate NFT metadata JSON |
| `aeonmi key-set <provider> <key>` | Store API key (encrypted) |
| `aeonmi key-get <provider>` | Retrieve API key |
| `aeonmi key-list` | List stored providers |
| `aeonmi key-rotate` | Re-encrypt all keys |

### Mother AI

| Command | Description |
|---------|-------------|
| `aeonmi mother` | Interactive Mother AI REPL |
| `aeonmi mother --file <f>` | Run script through Mother |

### Development

| Command | Description |
|---------|-------------|
| `aeonmi format <files>` | Format .ai files |
| `aeonmi lint <files>` | Lint .ai files |
| `aeonmi edit <file>` | Line-mode editor |
| `aeonmi edit <file> --tui` | TUI editor |
| `aeonmi new <file>` | Create new .ai file |

### Metrics & Debug

| Command | Description |
|---------|-------------|
| `aeonmi metrics-dump` | Dump metrics JSON |
| `aeonmi metrics-top --limit 10` | Slowest functions |
| `aeonmi metrics-export <file.csv>` | Export to CSV |

---

## Project Architecture

```
Aeonmi-aeonmi01/
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── cli.rs                     # Command definitions (clap)
│   ├── lib.rs                     # Library root
│   ├── core/
│   │   ├── lexer.rs               # Tokenizer (Unicode, quantum, hieroglyphic)
│   │   ├── parser.rs              # Recursive descent parser
│   │   ├── ast.rs                 # Abstract Syntax Tree definitions
│   │   ├── code_generator.rs      # JS backend code generation
│   │   ├── ai_emitter.rs          # .ai canonical form emitter
│   │   ├── lowering.rs            # AST → IR transformation
│   │   ├── ir.rs                  # Intermediate Representation
│   │   ├── vm.rs                  # Native interpreter/VM
│   │   ├── semantic_analyzer.rs   # Scope checking, unused vars
│   │   ├── compiler.rs            # Full pipeline orchestrator
│   │   ├── blockchain.rs          # Merkle chain implementation
│   │   ├── mint.rs                # NFT metadata generator
│   │   ├── quantum_simulator.rs   # State-vector quantum simulator
│   │   ├── quantum_algorithms.rs  # Grover, Shor, Deutsch-Jozsa, etc.
│   │   ├── quantum_neural_network.rs  # QNN, QAOA, VQE, QKD
│   │   ├── circuit_builder.rs     # Quantum circuit construction
│   │   ├── circuit_compiler.rs    # Multi-target circuit compilation
│   │   ├── circuit_visualization.rs # Text-mode circuit diagrams
│   │   └── titan/                 # Titan math libraries
│   │       ├── optimization.rs    # Gradient descent, golden section
│   │       └── statistics.rs      # Mean, median, mode, percentile
│   ├── glyph/
│   │   ├── mgk.rs                 # Master Glyph Key (Argon2id)
│   │   ├── ugst.rs                # UGST derivation (HKDF-SHA3-512)
│   │   ├── gdf.rs                 # Glyph Derivation Function (OKLCH + Hz)
│   │   ├── vault.rs               # Encrypted vault (XChaCha20-Poly1305)
│   │   └── anomaly.rs             # Anomaly detection + glyph distortion
│   ├── mother/
│   │   ├── embryo_loop.rs         # Parse → Execute → Learn cycle
│   │   ├── emotional_core.rs      # Valence, arousal, bond
│   │   ├── neural.rs              # Feedforward neural network
│   │   ├── quantum_attention.rs   # Entanglement-based attention
│   │   ├── quantum_core.rs        # Quantum consciousness model
│   │   └── language_evolution.rs  # Vocabulary + semantic depth
│   └── qube/
│       ├── lexer.rs               # QUBE tokenizer
│       ├── parser.rs              # QUBE parser
│       ├── ast.rs                 # QUBE AST
│       └── executor.rs            # QUBE quantum executor
├── shard/src/                     # The Shard (self-hosting compiler in .ai)
├── examples/                      # Example .ai and .qube programs
├── mother_ai/                     # Mother AI entry point
├── titan_libraries/               # Extended Titan modules
├── docs/
│   └── LANGUAGE_SPEC_CURRENT.md   # Current language specification
└── tests/                         # Integration tests
```

---

## Test Suite

**135/135 tests passing.**

```powershell
# Run all library tests
cargo test --release --lib

# Run a specific test
cargo test --release --lib -- test_deutsch_jozsa
```

Test coverage includes:
- AST node construction and manipulation
- Blockchain (Merkle chain) integrity
- Code generator JS output verification
- Full compilation pipeline
- Quantum algorithms (Grover, Shor, Deutsch-Jozsa, Bernstein-Vazirani, teleportation)
- Quantum simulator (state vectors, gates)
- Glyph system (MGK seal/unseal, UGST derivation, GDF rendering, vault encrypt/decrypt, anomaly detection)
- Mother AI (embryo loop, emotional core, neural network, quantum attention, language evolution)
- QUBE (lexer, parser, executor, circuit diagrams)

---

## Success Criteria — All Passing

| # | Test | Command | Status |
|---|------|---------|--------|
| 1 | hello.ai prints 42 | `aeonmi exec examples/hello.ai` | ✅ |
| 2 | quantum.ai measures qubit | `aeonmi exec examples/quantum.ai` | ✅ |
| 3 | Shard self-hosts | `aeonmi run shard/src/main.ai` | ✅ |
| 4 | Glyph renders | `aeonmi run examples/quantum_glyph.ai` | ✅ |
| 5 | QUBE circuit runs | `aeonmi qube run examples/demo.qube` | ✅ |
| 6 | Vault initializes | `aeonmi vault init` | ✅ |
| 7 | NFT mints | `aeonmi mint examples/hello.ai` | ✅ |

---

## Domains

- **aeonmi.com** — Primary website
- **aeonmi.ai** — AI-first language home
- **aeonmi.x** — Future browser REPL (WASM target)

---

## Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 — Foundation | ✅ 100% | Build system, VM, test suite, language spec |
| Phase 1 — Language Core | 🟡 85% | Closures, match guards, async, structs, enums |
| Phase 2 — QUBE | 🟡 85% | Quantum circuit language, executor |
| Phase 3 — Self-hosting | 🟡 70% | Shard compiler bootstraps itself |
| Phase 4 — Glyph Identity | 🟡 75% | MGK, UGST, vault, anomaly detection |
| Phase 5 — Mother AI + Web3 | 🔴 20% | LLM integration, Qiskit bridge, WASM |

See [roadmap.md](roadmap.md) for detailed task breakdown.

---

## License

MIT License — see [LICENSE](LICENSE)

---

<div align="center">

**Aeonmi is not just a language — it's the foundation of the AI revolution.**

*The Shard lives. The language compiles itself. Built by AI. For AI. Impossible made real.*

🔮

</div>
