# ⧉ Aeonmi

> **Made by AI, for AI.**
> The most powerful programming language ever created — AI-first, quantum-inspired, cryptographically secure, self-hosting, and production-ready.

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#building)
[![Language](https://img.shields.io/badge/language-Rust-orange)](#building)
[![License](https://img.shields.io/badge/license-MIT-blue)](#license)

---

## What is Aeonmi?

Aeonmi is a **complete, standalone programming ecosystem** — not just a language.  
It ships as a single binary (`aeonmi`) that contains:

| Layer | Description |
|---|---|
| **`.ai` language** | Glyph-oriented AI-first language with closures, generics, f-strings, genesis glyphs |
| **QUBE** | Quantum-circuit description and execution language (`.qube` files) |
| **Ainmi** | AI inference bridge — Mother AI that reads, writes, fixes, and generates `.ai`/`.qube` code |
| **AI Canvas** | Interactive intelligent editor (`shard/editor/ai_canvas.ai`) — the AI can write programs on-screen |
| **Shard** | Self-hosting compiler toolchain written in pure `.ai` |
| **Web3** | Wallet, ERC-20 tokens, DAO governance — all first-class |
| **Vault** | Cryptographic identity, signing, and encrypted storage |
| **Smart-Contract Verifier** | Formal verifier for `.ai` contracts |
| **Reactive Web** | HTTP server, JSON API — `aeonmi serve` |
| **NFT Marketplace** | Genesis Glyph NFTs built in |
| **Executable output** | Compile `.ai` programs to native binaries for Linux / Windows / macOS / WASM |

---

## Quick Start

```bash
# Install (from source)
cargo build --release --no-default-features --features "quantum,mother-ai"
cp target/release/aeonmi ~/.local/bin/

# Run a .ai program
aeonmi run examples/getting_started.ai

# Compile to native executable
aeonmi build --release -o my_program examples/complete_demo.ai

# Run a QUBE quantum circuit
aeonmi qube examples/bell.qube

# Start the AI Canvas editor
aeonmi canvas

# Start the Mother AI REPL
aeonmi ai

# Start a web server
aeonmi serve examples/web_app.ai --port 8080

# Manage a crypto wallet
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

| Glyph | Name | Meaning |
|---|---|---|
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
circuit search {
    grover(n=4);      // Grover's search
}

circuit transform {
    qft(n=8);         // Quantum Fourier Transform
}

circuit factor {
    shor(n=15);       // Shor's factoring
}

circuit entangle {
    teleport(q0, q1, q2);   // Quantum teleportation
}
```

### Symbolic Syntax (legacy)

```qube
state psi = superposition(q0, q1)
apply H to psi
apply CNOT to (psi, q1)
collapse psi
```

### CLI

```bash
aeonmi qube circuit.qube          # run a circuit
aeonmi qube --verify circuit.qube # verify only
aeonmi qube --draw circuit.qube   # ASCII diagram
```

---

## Mother AI / Ainmi

The **Mother AI** is the intelligent core of Aeonmi.  
It runs locally and can:

- **Generate** programs from a natural-language description
- **Complete** code as you type (like Copilot, but Aeonmi-native)
- **Explain** any `.ai` or `.qube` snippet
- **Fix** bugs automatically
- **Refactor** and **optimize** existing code
- **Security-audit** for vulnerabilities
- **Design** quantum circuits from plain-English intent

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

Inside `.ai` code, import `mother/ainmi`:

```ai
import "mother/ainmi"

let code = ainmi.generate("bubble sort in Aeonmi")
print(code)
```

---

## AI Canvas Editor

The **AI Canvas** is Aeonmi's intelligent editor — written entirely in `.ai`.  
It is the primary interface for creating programs with the help of Mother AI.

```bash
aeonmi canvas                  # launch the AI Canvas TUI
aeonmi canvas --file prog.ai   # open existing file
```

### Key features

| Feature | Description |
|---|---|
| AI code generation | Describe a program; the canvas writes it |
| AI completion | Tab-complete powered by Mother AI |
| AI fix | One-key error repair |
| AI explain | Inline documentation on demand |
| AI refactor | Restructure code by instruction |
| Quantum mode | Toggle QUBE backend; insert circuit snippets |
| Undo / redo | 256-step history |
| Session save/restore | Named workspaces |
| Autosave | Every 30 seconds |
| Cross-compile | Build for Linux / Windows / macOS / WASM |
| Vault integration | Sign and encrypt programs |

The canvas source lives at `shard/editor/ai_canvas.ai` and can itself be modified, extended, and recompiled inside the canvas — full self-hosting.

---

## Building Executables

Aeonmi produces **production-ready native executables** via its Rust codegen backend.

```bash
# Debug build
aeonmi build -o out/my_app src/main.ai

# Release (optimized)
aeonmi build --release -o out/my_app src/main.ai

# Cross-compile
aeonmi build --target windows-x86_64 -o out/my_app.exe src/main.ai
aeonmi build --target wasm32         -o out/my_app.wasm src/main.ai
aeonmi build --target macos-arm64    -o out/my_app      src/main.ai
```

Executables are fully self-contained — no Aeonmi runtime required on the target machine.

---

## Web3

```bash
# Wallet
aeonmi wallet new
aeonmi wallet import --mnemonic "..."
aeonmi wallet balance --address 0xABC…

# Token (ERC-20)
aeonmi token deploy --name AeonToken --symbol AEN --supply 1000000
aeonmi token transfer --to 0xDEF… --amount 100

# DAO governance
aeonmi dao create --name AeonDAO
aeonmi dao propose "Fund AI Canvas development"
aeonmi dao vote --proposal 1 --choice yes
```

---

## Smart-Contract Verifier

```bash
aeonmi verify contract.ai          # formal verification
aeonmi verify --report contract.ai # detailed report
```

The verifier checks:
- Reentrancy
- Integer overflow / underflow  
- Access-control violations
- Unchecked return values
- Gas-limit patterns

---

## Vault & Security

Aeonmi is built with **security-first** design:

```bash
# Create identity
aeonmi vault create-identity --name "dev-key"

# Sign a program
aeonmi vault sign program.ai --identity dev-key

# Verify signature
aeonmi vault verify program.ai --sig program.ai.sig

# Encrypt
aeonmi vault encrypt program.ai --recipient pubkey.pem

# Decrypt
aeonmi vault decrypt program.ai.enc --identity dev-key
```

Inside `.ai`:

```ai
import "crypto/vault"

let sig = vault.sign(source_code, "my-identity")
let ok  = vault.verify(source_code, sig, "my-identity")
```

Security properties:
- Ed25519 signing
- AES-256-GCM encryption
- Blake3 content hashing
- Zero-knowledge proof stubs for contracts

---

## Self-Hosting: Shard

Shard is Aeonmi's self-hosting compiler — written entirely in `.ai`.

```
shard/
├── src/
│   └── main.ai           # Shard entry point
├── qube/
│   ├── executor.qube     # QUBE executor circuit spec
│   └── grammar.qube      # Grammar verification circuit
├── editor/
│   └── ai_canvas.ai      # AI Canvas (this editor)
├── mother/               # Mother AI modules
├── market/               # NFT marketplace
├── vault/                # Cryptographic vault
└── titan/                # AOT compiler backend
```

To build Aeonmi with Shard (bootstrap):

```bash
# Stage 1: build with Rust
cargo build --release --no-default-features --features "quantum,mother-ai"

# Stage 2: use Aeonmi to compile Shard
aeonmi build --release -o shard_compiler shard/src/main.ai

# Stage 3: use Shard to compile Aeonmi (full self-host)
./shard_compiler build --release -o aeonmi_v2 src/main.ai
```

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
# List genesis glyphs
aeonmi market glyphs

# Mint a glyph NFT
aeonmi market mint --glyph "⧉" --owner "0xABC…"

# List all NFTs
aeonmi market list

# Get info
aeonmi market info --id 1
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    User / AI Canvas                     │
│          (shard/editor/ai_canvas.ai  — .ai)             │
├────────────────┬────────────────┬────────────────────────┤
│  .ai language  │  .qube circuits│  .ainmi AI models      │
│  (surface)     │  (quantum)     │  (Mother AI bridge)    │
├────────────────┴────────────────┴────────────────────────┤
│                  Aeonmi Compiler Pipeline                │
│  Lexer → Parser → AST → IR → Lowering → VM / Codegen   │
├──────────────────────────────────────────────────────────┤
│              QUBE Executor  (circuit sim)                │
│  Lexer → Parser → AST → Executor → State Vector         │
├──────────────────────────────────────────────────────────┤
│  Web3  │  Vault  │  Verifier  │  Market  │  Web Server  │
├──────────────────────────────────────────────────────────┤
│         Rust runtime  (production executable output)     │
│         Linux / Windows / macOS / WASM                   │
└──────────────────────────────────────────────────────────┘
```

---

## CLI Reference

```
aeonmi <COMMAND> [OPTIONS]

COMMANDS:
  run      <file.ai>           Run an .ai program
  build    <file.ai>           Compile to native executable
  qube     <file.qube>         Run a QUBE quantum circuit
  canvas                       Launch AI Canvas editor
  ai       [generate|fix|…]    Mother AI interface
  serve    <file.ai>           Start reactive web server
  verify   <file.ai>           Smart-contract verifier
  wallet   [new|balance|…]     Crypto wallet
  token    [deploy|transfer|…] ERC-20 token management
  dao      [create|propose|…]  DAO governance
  market   [list|mint|…]       NFT marketplace
  vault    [sign|verify|…]     Cryptographic vault

GLOBAL OPTIONS:
  --config <path>   Config file (default: ~/.aeonmi/qpoly.toml)
  --verbose         Verbose output
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
backend        = "statevector"  # "statevector" | "density_matrix" | "hardware"
shots          = 1024

[mother_ai]
model          = "local"        # "local" | "remote"
context_lines  = 64

[vault]
identity       = "~/.aeonmi/keys/default.key"
```

---

## Building from Source

**Requirements:** Rust 1.74+

```bash
git clone https://github.com/Aeonmi-Aysa/aeonmi
cd aeonmi

# Standard build
cargo build --no-default-features --features "quantum,mother-ai"

# Run tests
cargo test --no-default-features --features "quantum,mother-ai"

# Release build
cargo build --release --no-default-features --features "quantum,mother-ai"
```

### Feature flags

| Flag | Description |
|---|---|
| `quantum` | QUBE quantum circuit engine |
| `mother-ai` | Mother AI / Ainmi inference bridge |
| `web3` | Wallet, tokens, DAO |
| `tui` | Terminal UI (requires ncurses) |
| `audio` | Audio synthesis (requires alsa-dev on Linux) |

---

## Roadmap

- [x] **Phase 1** — Core language: lexer, parser, AST, VM, closures, generics, f-strings
- [x] **Phase 2** — Quantum simulator: joint state-vector, CNOT, Bell states
- [x] **Phase 3** — File I/O, Shard self-hosting bootstrap
- [x] **Phase 4** — Genesis glyphs G1–G12, for-in loops, cyberpunk banner
- [x] **Phase 5** — Smart-contract verifier, reactive web, NFT marketplace
- [x] **Phase 6** — Web3 (wallet, token, DAO)
- [x] **Phase 7** — QUBE circuit-block syntax (circuit/meta/execute/expected, all gates + built-in algos)
- [x] **AI Canvas** — Intelligent editor (`shard/editor/ai_canvas.ai`)
- [ ] **Phase 8** — Full AOT codegen via Titan backend (LLVM IR emission)
- [ ] **Phase 9** — Hardware quantum backend (IBM Quantum / IonQ bridge)
- [ ] **Phase 10** — Distributed execution (multi-node Aeonmi clusters)
- [ ] **Phase 11** — Visual AI Canvas GUI (Tauri/WebGPU)
- [ ] **Phase 12** — Package registry (aeonmi.io)

---

## Project Philosophy

> **Made by AI, for AI.**

Aeonmi exists at the intersection of three paradigms:

1. **AI-first** — The language is designed so that an AI (Mother AI / Ainmi) can read, write, reason about, and improve Aeonmi programs better than any other language.

2. **Quantum-ready** — QUBE circuits integrate seamlessly with classical `.ai` programs. You write the logic; QUBE handles the superposition.

3. **Security-unmatched** — Every program can be signed, encrypted, and formally verified. The vault is not an afterthought — it is a first-class citizen.

---

## License

MIT © Aeonmi-Aysa

---

*"The best code is the code that writes itself."*
