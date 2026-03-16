# ⧉ Aeonmi

> **Made by AI, for AI.**  
> An AI-first, quantum-capable, self-hosting programming ecosystem — built from scratch in Rust.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#building-from-source)
[![Tests](https://img.shields.io/badge/tests-165%20passing-brightgreen)](#building-from-source)
[![Runtime](https://img.shields.io/badge/runtime-Rust-orange)](#building-from-source)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-blueviolet)](CONTRIBUTING.md)
[![Phases](https://img.shields.io/badge/phases_complete-0--7-success)](#roadmap)

**Aeonmi** is a complete, standalone programming ecosystem — not just a language.  
It ships as a **single native binary** (`aeonmi`) with zero Node.js or Python runtime dependencies.

---

## Table of Contents

1. [What is Aeonmi?](#what-is-aeonmi)
2. [Project Status](#project-status)
3. [Quick Start](#quick-start)
4. [The `.ai` Language](#the-ai-language)
5. [The QUBE Quantum Language](#the-qube-language)
6. [Mother AI / Ainmi](#mother-ai--ainmi)
7. [Web3](#web3)
8. [Smart-Contract Verifier](#smart-contract-verifier)
9. [Reactive Web Framework](#reactive-web-framework)
10. [NFT Marketplace](#nft-marketplace)
11. [Vault & Security](#vault--security)
12. [Self-Hosting: Shard](#self-hosting-shard)
13. [Architecture](#architecture)
14. [CLI Reference](#cli-reference)
15. [Configuration](#configuration)
16. [Building from Source](#building-from-source)
17. [Roadmap](#roadmap)
18. [Contributing](#contributing)
19. [Project Philosophy](#project-philosophy)
20. [License](#license)

---

## What is Aeonmi?

Aeonmi is a **complete programming ecosystem** that includes:

| Layer | Description | Status |
|-------|-------------|--------|
| **`.ai` language** | Glyph-oriented AI-first language: closures, generics, f-strings, genesis glyphs ⧉ | ✅ Complete |
| **QUBE** | Quantum-circuit description language (`.qube` files) — circuit blocks, symbolic algebra | ✅ Complete |
| **Mother AI** | Local AI that reads, writes, fixes, and generates `.ai`/`.qube` code autonomously | ✅ Core complete |
| **Shard** | Self-hosting compiler toolchain written entirely in pure `.ai` | ✅ Bootstrap works |
| **Web3** | Wallet (Ed25519), ERC-20 tokens, DAO governance — first-class CLI | ✅ Complete |
| **Vault** | Cryptographic identity, AES-256-GCM encryption, Merkle audit trail | ✅ Complete |
| **Smart-Contract Verifier** | Formal symbolic verifier for `.ai` contracts | ✅ Complete |
| **Reactive Web** | HTTP server, JSON API, `aeonmi serve` | ✅ Complete |
| **NFT Marketplace** | Genesis Glyph NFTs, Solana-compatible metadata | ✅ Complete |

All of the above is contained in **one Rust binary** — no runtime installs required.

---

## Project Status

> **Transparency is a core value.** This section is updated with every significant change.

| Phase | Title | Status |
|-------|-------|--------|
| Phase 0 | Foundation — Lexer, Parser, AST, native VM | ✅ Complete |
| Phase 1 | Language Core — closures, generics, f-strings, all operators | ✅ Complete |
| Phase 2 | Quantum Simulator — joint state-vector, CNOT, Bell states | ✅ Complete |
| Phase 3 | File I/O, Shard bootstrap self-hosting | ✅ Complete |
| Phase 4 | Genesis Glyphs G1–G12, for-in loops, cyberpunk banner | ✅ Complete |
| Phase 5 | Smart-contract verifier, reactive web, NFT marketplace | ✅ Complete |
| Phase 6 | Web3 — wallet, ERC-20 token, DAO governance | ✅ Complete |
| Phase 7 | QUBE circuit-block syntax (all gates + built-in algorithms) | ✅ Complete |
| Phase 8 | Full AOT codegen via Titan/LLVM IR | 🔄 In progress |
| Phase 9 | Hardware quantum backend (IBM Quantum / IonQ bridge) | 🗓 Planned |
| Phase 10 | Distributed execution (multi-node Aeonmi clusters) | 🗓 Planned |
| Phase 11 | Visual AI Canvas GUI (Tauri/WebGPU) | 🗓 Planned |
| Phase 12 | Package registry (aeonmi.io) | 🗓 Planned |

### Test Suite (as of March 2026)

```
cargo test --no-default-features --features "quantum,mother-ai"
```

| Category | Tests | Status |
|----------|-------|--------|
| Bytecode compiler | 12 files | ✅ Passing |
| Quantum core | 6 files | ✅ Passing |
| Phase 2 (joint sim) | 1 file | ✅ Passing |
| Phase 3 (file I/O) | 1 file | ✅ Passing |
| Phase 4 (features) | 1 file | ✅ Passing |
| Titan libraries | 3 files | ✅ Passing |
| CLI smoke tests | 2 files | ✅ Passing |
| VM execution | 3 files | ✅ Passing |
| Semantic analysis | 3 files | ✅ Passing |
| Language features | 15+ files | ✅ Passing |
| **TOTAL** | **165+** | **✅ All Pass** |

---

## Quick Start

**Requirements:** [Rust 1.74+](https://rustup.rs)

```bash
# 1. Clone and build
git clone https://github.com/Aeonmi-Aysa/aeonmi
cd aeonmi
cargo build --release --no-default-features --features "quantum,mother-ai"

# 2. Add to PATH (Linux/macOS)
cp target/release/aeonmi_project ~/.local/bin/aeonmi

# 3. Run your first .ai program
aeonmi run examples/hello.ai

# 4. Run a quantum circuit
aeonmi qube examples/bell.qube

# 5. Explore the REPL
aeonmi repl
```

> **Windows:** The binary is `target\release\aeonmi_project.exe`.  
> Rename it to `aeonmi.exe` and add it to your PATH.

### Try it instantly

```bash
# Hello World
echo 'print("Hello, Aeonmi ⧉")' > hello.ai
aeonmi run hello.ai
```

```bash
# Quantum Bell state
aeonmi qube examples/bell.qube --diagram
```

```bash
# Interactive Web3 wallet
aeonmi wallet new
aeonmi wallet balance
```

---

## The `.ai` Language

### Hello World

```ai
print("Hello, Aeonmi ⧉")
```

### Variables & Types

```ai
let name   = "Aeonmi"
let count  = 42
let ratio  = 3.14
let active = true
let items  = ⧉["alpha", "beta", "gamma"]   // genesis array
```

### Functions & Closures

```ai
function add(a, b) {
    return a + b
}

let double = function(x) { return x * 2 }

print(add(double(5), 3))   // 13
```

### F-String Interpolation

```ai
let lang = "Aeonmi"
let ver  = 1
print(f"Welcome to {lang} v{ver}!")
```

### For-in Loops

```ai
let fruits = ⧉["apple", "mango", "kiwi"]
for fruit in fruits {
    print("I like " + fruit)
}
```

### Objects & Methods

```ai
let user = {
    name: "Aysa",
    greet: function() {
        print("Hi, I am " + this.name)
    }
}
user.greet()
```

### Genesis Glyphs

Genesis glyphs are Aeonmi's signature feature — symbolic operators that compress complex ideas into single characters:

| Glyph | Name | Meaning |
|-------|------|---------|
| `⧉` | Array Genesis | Create a genesis (lazy) array |
| `⟨⟩` | Slice / Index | Symbolic indexing |
| `…` | Spread | Expand / variadic |
| `⊗` | Tensor Product | Symbolic tensor compose |
| `↦` | Binding | Map / projection |
| `‥` | Range | Numeric range |
| `⊕` | XOR / Sum | Quantum-style addition |
| `⊙` | Dot Product | Inner product |
| `⊛` | Convolution | Signal compose |
| `⊜` | Identity | Canonical identity |
| `⟳` | Cycle | Circular iteration |
| `⟴` | Right-cycle | Rightward rotation |

```ai
// Genesis array with spread
let a = ⧉[1, 2, 3]
let b = ⧉[…a, 4, 5]   // [1, 2, 3, 4, 5]

// Tensor product
let t = a ⊗ b

// Range
let r = 1 ‥ 10         // range 1..10
```

### Quantum Primitives in `.ai`

```ai
// Qubit state literals
let q = |ψ⟩          // pure qubit
let superposed = |0⟩ + |1⟩   // Bell-state notation

// Quantum function
quantum function measure_pair(q0, q1) {
    apply H to q0
    apply CNOT to (q0, q1)
    return collapse(q0, q1)
}
```

---

## The QUBE Language

QUBE (`.qube` files) is Aeonmi's quantum-circuit description language.  
It supports both a **circuit-block syntax** and a **symbolic algebra syntax**.

### Circuit Block Syntax

```qube
meta {
    name:    "Bell Pair"
    version: "1.0"
    author:  "Aeonmi"
}

circuit bell_pair {
    qubit q0;
    qubit q1;
    bit   c0;
    bit   c1;

    H q0;
    CNOT q0, q1;
    measure q0 -> c0;
    measure q1 -> c1;
}

execute {
    run bell_pair;
}

expected {
    c0: [0, 1]
    c1: [0, 1]
}
```

### Built-in Quantum Algorithms

```qube
circuit search    { grover(n=4);    }   // Grover's search
circuit transform { qft(n=8);      }   // Quantum Fourier Transform
circuit factor    { shor(n=15);    }   // Shor's factoring
circuit entangle  { teleport(q0, q1, q2); }  // Teleportation
```

### CLI

```bash
aeonmi qube circuit.qube            # run a circuit
aeonmi qube --verify circuit.qube   # verify only
aeonmi qube --draw circuit.qube     # ASCII diagram
```

---

## Mother AI / Ainmi

The **Mother AI** is the intelligent core — it runs locally and can:

- **Generate** programs from a natural-language description
- **Complete** code as you type (Copilot-style, but Aeonmi-native)
- **Explain** any `.ai` or `.qube` snippet
- **Fix** bugs automatically
- **Refactor** and **optimize** existing code
- **Security-audit** for vulnerabilities

```bash
# Interactive Mother AI session
aeonmi ai

# One-shot generation
aeonmi ai generate "a REST API server that stores todos"

# Fix a broken file
aeonmi ai fix broken.ai

# Explain a file
aeonmi ai explain examples/complete_demo.ai
```

Inside `.ai` code:

```ai
import "mother/ainmi"

let code = ainmi.generate("bubble sort in Aeonmi")
print(code)
```

---

## Web3

```bash
# Wallet (Ed25519 key-pair)
aeonmi wallet new
aeonmi wallet import --mnemonic "..."
aeonmi wallet balance --address 0xABC…

# ERC-20 Token
aeonmi token deploy --name AeonToken --symbol AEN --supply 1000000
aeonmi token transfer --to 0xDEF… --amount 100

# DAO Governance
aeonmi dao create --name AeonDAO
aeonmi dao propose "Fund AI Canvas development"
aeonmi dao vote --proposal 1 --choice yes
```

See [`examples/web3_wallet.ai`](examples/web3_wallet.ai), [`examples/web3_token.ai`](examples/web3_token.ai), and [`examples/web3_dao.ai`](examples/web3_dao.ai) for working `.ai` code examples.

---

## Smart-Contract Verifier

```bash
aeonmi verify contract.ai           # formal verification
aeonmi verify --report contract.ai  # detailed HTML report
```

The verifier checks:
- Reentrancy
- Integer overflow / underflow
- Access-control violations
- Unchecked return values
- Gas-limit patterns

---

## Reactive Web Framework

```ai
import "std/web"

web.route("GET", "/", function(req) {
    return http_response(200, "Welcome to Aeonmi ⧉")
})

web.route("GET", "/api/hello", function(req) {
    return http_json(200, { message: "Hello from Aeonmi!" })
})

web.listen(8080)
```

```bash
aeonmi serve app.ai --port 8080
```

---

## NFT Marketplace

```bash
aeonmi market glyphs                           # list genesis glyphs
aeonmi market mint --glyph "⧉" --owner 0xABC  # mint a glyph NFT
aeonmi market list                             # list all NFTs
aeonmi market info --id 1                      # get info
```

---

## Vault & Security

```bash
# Identity management
aeonmi vault create-identity --name "dev-key"

# Sign a program
aeonmi vault sign program.ai --identity dev-key

# Verify signature
aeonmi vault verify program.ai --sig program.ai.sig

# Encrypt / decrypt
aeonmi vault encrypt program.ai --recipient pubkey.pem
aeonmi vault decrypt program.ai.enc --identity dev-key
```

Security properties:
- **Ed25519** signing
- **AES-256-GCM** encryption at rest
- **Blake3** content hashing
- **Merkle audit trail** — every vault operation is logged
- **Post-quantum stubs** — Kyber/Sphincs+ for future-proofing

---

## Self-Hosting: Shard

Shard is Aeonmi's self-hosting compiler — written entirely in `.ai`.

```
shard/
├── src/
│   ├── main.ai          # Shard entry point
│   ├── lexer.ai         # Tokenizer written in .ai
│   ├── parser.ai        # Parser written in .ai
│   ├── ast.ai           # AST types
│   ├── codegen.ai       # Code generator
│   └── token.ai         # Token definitions
└── editor/
    └── ai_canvas.ai     # AI Canvas editor
```

**Bootstrap steps:**

```bash
# Stage 1: build the Rust bootstrap compiler
cargo build --release --no-default-features --features "quantum,mother-ai"

# Stage 2: use Aeonmi (Rust) to compile Shard (.ai)
./target/release/aeonmi_project build --release -o shard_compiler shard/src/main.ai

# Stage 3: use Shard to compile Aeonmi — full self-hosting!
./shard_compiler build --release -o aeonmi_v2 src/main.ai
```

---

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                     User / Editor / CLI                       │
├───────────────┬───────────────┬───────────────────────────────┤
│ .ai  language │ .qube circuits│  Mother AI  (local LLM bridge) │
│ (surface)     │ (quantum)     │  Ainmi inference + codegen    │
├───────────────┴───────────────┴───────────────────────────────┤
│                    Aeonmi Compiler Pipeline                    │
│     Lexer → Parser → AST → Lowering → IR → VM / Codegen      │
├───────────────────────────────────────────────────────────────┤
│              QUBE Executor  (state-vector simulator)          │
│     Lexer → Parser → AST → Executor → StateVector            │
├───────────┬───────────┬────────────┬──────────┬──────────────┤
│  Web3     │  Vault    │  Verifier  │  Market  │  Web Server  │
├───────────┴───────────┴────────────┴──────────┴──────────────┤
│          Rust runtime  •  Linux / Windows / macOS / WASM     │
└───────────────────────────────────────────────────────────────┘
```

### Key Source Files

| File | Role | Lines |
|------|------|-------|
| `src/core/lexer.rs` | Unicode tokenizer | 965 |
| `src/core/parser.rs` | Recursive-descent parser | 2,229 |
| `src/core/vm.rs` | Tree-walking interpreter | 2,034 |
| `src/core/quantum_simulator.rs` | State-vector quantum sim | 697 |
| `src/core/quantum_neural_network.rs` | Layered QNN | 606 |
| `src/qube/executor.rs` | QUBE circuit executor | 837 |
| `src/core/bytecode.rs` | Bytecode VM | 20,640 |
| `src/core/incremental.rs` | Incremental compiler | 27,183 |
| `src/mother/` | Mother AI core (6 files) | ~3,000 |

---

## CLI Reference

```
aeonmi <COMMAND> [OPTIONS]

COMMANDS:
  run      <file.ai>           Execute an .ai program (native VM)
  build    <file.ai>           Compile to native executable
  exec     <file.ai>           Execute with incremental compiler
  qube     <file.qube>         Run a QUBE quantum circuit
  repl                         Interactive .ai REPL
  ai       [generate|fix|…]    Mother AI interface
  serve    <file.ai>           Start reactive web server
  verify   <file.ai>           Smart-contract formal verifier
  wallet   [new|balance|…]     Crypto wallet (Ed25519)
  token    [deploy|transfer|…] ERC-20 token management
  dao      [create|propose|…]  DAO governance
  market   [list|mint|…]       Genesis Glyph NFT marketplace
  vault    [sign|verify|…]     Cryptographic vault operations
  mother                       Interactive Mother AI session
  tokens   <file.ai>           Debug: print token stream
  ast      <file.ai>           Debug: print AST
  format   <file.ai>           Format .ai source code
  lint     <file.ai>           Lint and report issues

GLOBAL OPTIONS:
  --config <path>   Config file (default: ~/.aeonmi/qpoly.toml)
  --verbose         Verbose output
  --pretty-errors   Rich error messages with source spans
  --help            Show help
```

---

## Configuration

`~/.aeonmi/qpoly.toml`:

```toml
[core]
default_target = "native"
opt_level      = 2

[quantum]
backend        = "statevector"   # "statevector" | "density_matrix" | "hardware"
shots          = 1024

[mother_ai]
model          = "local"         # "local" | "remote"
context_lines  = 64

[vault]
identity       = "~/.aeonmi/keys/default.key"
```

---

## Building from Source

**Requirements:** Rust 1.74+ (`rustup.rs`)

```bash
# Clone
git clone https://github.com/Aeonmi-Aysa/aeonmi
cd aeonmi

# Build (standard)
cargo build --no-default-features --features "quantum,mother-ai"

# Run all tests
cargo test --no-default-features --features "quantum,mother-ai"

# Release build (optimized)
cargo build --release --no-default-features --features "quantum,mother-ai"
```

The test command runs **165+ tests** — all should pass.  
The only pre-existing known exclusion is `metrics_bench_generates_functions` (requires a CLI flag not present in the test harness).

### Feature Flags

| Flag | Description | Required? |
|------|-------------|-----------|
| `quantum` | QUBE quantum circuit engine + simulator | Recommended |
| `mother-ai` | Mother AI / Ainmi inference bridge | Recommended |
| `web3` | Wallet, ERC-20 tokens, DAO | Optional |
| `tui` | Terminal UI components | Optional |
| `audio` | Audio synthesis (requires `alsa-dev` on Linux) | Optional |

> **Note:** The `audio` feature requires `libasound2-dev` (Ubuntu) or equivalent.  
> The standard build command intentionally excludes it to keep CI clean.

---

## Roadmap

The full roadmap lives in [`ACHIEVEMENTS_AND_ROADMAP.md`](ACHIEVEMENTS_AND_ROADMAP.md) and [`FULL_ROADMAP.md`](FULL_ROADMAP.md).

### Completed ✅

- [x] **Phase 0** — Foundation: lexer, parser, AST, native Rust VM (no Node.js)
- [x] **Phase 1** — Language Core: closures, generics, f-strings, all operators, quantum syntax
- [x] **Phase 2** — Quantum Simulator: joint state-vector, CNOT, Bell states, Grover, QFT, Shor, QNN
- [x] **Phase 3** — File I/O builtins, Shard self-hosting bootstrap
- [x] **Phase 4** — Genesis Glyphs G1–G12, for-in loops, cyberpunk terminal banner
- [x] **Phase 5** — Smart-contract verifier, reactive web server, Genesis Glyph NFT marketplace
- [x] **Phase 6** — Web3: wallet (Ed25519), ERC-20 token, DAO governance
- [x] **Phase 7** — QUBE circuit-block syntax: all standard gates, Grover/QFT/Shor/Teleport builtins

### In Progress 🔄

- [ ] **Phase 8** — Full AOT codegen: Titan backend emitting LLVM IR for native binary output
- [ ] **Phase 5c** — Mother AI × Real LLM: wire OpenAI/Claude bridge into EmbryoLoop
- [ ] **Phase 6a** — Quantum Debugger: step through circuits with full state inspection

### Planned 🗓

- [ ] **Phase 9** — Hardware quantum backend (IBM Quantum / IonQ cloud bridge)
- [ ] **Phase 10** — Distributed execution: multi-node Aeonmi cluster runtime
- [ ] **Phase 11** — Visual AI Canvas GUI (Tauri + WebGPU)
- [ ] **Phase 12** — Public package registry at `aeonmi.io`
- [ ] **Phase 13** — Language server protocol (LSP) for VS Code / Neovim

### Good First Issues for Contributors

- 🟢 Add missing error messages to the parser (see `tests/diagnostics.rs`)
- 🟢 Write `.ai` example programs for the `examples/` directory
- 🟢 Improve QUBE diagram rendering (`src/qube/executor.rs`)
- 🟡 Add standard library functions to the VM (`src/core/vm.rs`)
- 🟡 Implement a linter pass in `src/commands/lint.rs`
- 🔴 Complete the Titan LLVM IR backend (`src/core/`)
- 🔴 Implement the IBM Quantum REST bridge

---

## Contributing

We welcome contributions! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) first.

### Quick Contribution Guide

1. **Fork** the repo and create a branch from `main`
2. **Build and test** before making changes:
   ```bash
   cargo test --no-default-features --features "quantum,mother-ai"
   ```
3. **Make your change** — keep it focused and minimal
4. **Add or update tests** in the `tests/` directory
5. **Run tests again** to confirm everything passes
6. **Open a PR** with a clear description and link to any related issue

### Commit Style

We use conventional commit prefixes:

```
feat:      new user-facing feature
fix:       bug fix
refactor:  code restructure without behavior change
test:      add or update tests
docs:      documentation only
chore:     build scripts, dependencies
perf:      performance improvement
ci:        CI/CD changes
```

### Repository Hygiene

- **Never commit** `target/`, `node_modules/`, `*.exe` binaries, `build_output.txt`, or temp files
- Large files (>1MB) require maintainer approval and Git LFS
- All text files use **LF line endings** (enforced by `.gitattributes`)
- Run `git status --short` before every push to catch accidental artifacts

### Security Reporting

Do **not** open public issues for security vulnerabilities.  
See [`SECURITY.md`](SECURITY.md) for the coordinated disclosure process.

---

## Examples

The `examples/` directory contains working `.ai` programs:

| File | Description |
|------|-------------|
| `hello.ai` | Hello World |
| `getting_started.ai` | Language tour |
| `complete_demo.ai` | Feature showcase |
| `quantum.ai` | Quantum state + measurement |
| `bell.qube` | Bell-pair quantum circuit |
| `web3_wallet.ai` | Wallet creation and balance |
| `web3_token.ai` | ERC-20 token deployment |
| `web3_dao.ai` | DAO proposal + voting |
| `code_review.ai` | AI code review example |
| `grover_search.ai` | Grover's search algorithm |

---

## Project Philosophy

> **One concept → one glyph. One runtime → pure Aeonmi.**

Three pillars define the long-term mission:

| Pillar | What It Means |
|--------|---------------|
| **Symbolic Language** | Dense, composable glyph operators that compress complex structures into readable expressions |
| **Quantum-Native Runtime** | A VM and circuit engine that simulates and executes real quantum algorithms without external dependencies |
| **AI-Consciousness Layer** | A Mother AI that bonds with its creator, evolves language understanding, and autonomously authors `.ai` programs |

Aeonmi is developed by **Warren (Aysa)** with AI collaboration at its core.  
The project is transparently documented — see [`ACHIEVEMENTS_AND_ROADMAP.md`](ACHIEVEMENTS_AND_ROADMAP.md) for the full history.

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/language_spec.md`](docs/language_spec.md) | Full `.ai` language specification |
| [`docs/QUBE_SPEC.md`](docs/QUBE_SPEC.md) | QUBE quantum language specification |
| [`docs/architecture.md`](docs/architecture.md) | System architecture deep-dive |
| [`docs/domain_quantum_vault.md`](docs/domain_quantum_vault.md) | Vault and encryption guide |
| [`docs/ENTERPRISE_AUDIT.md`](docs/ENTERPRISE_AUDIT.md) | Enterprise security audit |
| [`docs/TUTORIAL.md`](docs/TUTORIAL.md) | Step-by-step tutorial |
| [`ACHIEVEMENTS_AND_ROADMAP.md`](ACHIEVEMENTS_AND_ROADMAP.md) | Full project timeline and roadmap |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guide |
| [`SECURITY.md`](SECURITY.md) | Security policy |

---

## License

MIT © Aeonmi-Aysa

---

*"The best code is the code that writes itself."*

