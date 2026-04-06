# AEONMI FULL ROADMAP
### From Bootstrap to Total Self-Sovereignty
#### *One Language. One Runtime. No Dependencies. Pure Aeonmi.*

**Author:** Warren (Aysa)  
**Last Updated:** March 16, 2026  
**Status Baseline:** Phases 0–2 complete · Phase 3 in progress  

---

## TABLE OF CONTENTS

1. [Vision & End Goal](#1-vision--end-goal)
2. [What Exists Today](#2-what-exists-today)
3. [Architecture Blueprint — The Final State](#3-architecture-blueprint--the-final-state)
4. [Phase 0 — Foundation ✅](#phase-0--foundation-)
5. [Phase 1 — Language Core ✅](#phase-1--language-core-)
6. [Phase 2 — Quantum Integration ✅](#phase-2--quantum-integration-)
7. [Phase 3 — QUBE Engine & Shard Bootstrap](#phase-3--qube-engine--shard-bootstrap)
8. [Phase 4 — Node.js Elimination & Native Binary](#phase-4--nodejs-elimination--native-binary)
9. [Phase 5 — Shard CLI: Full System Shell Parity](#phase-5--shard-cli-full-system-shell-parity)
10. [Phase 6 — Language Written in Aeonmi](#phase-6--language-written-in-aeonmi)
11. [Phase 7 — Mother AI, Titan & Vault in Pure Aeonmi](#phase-7--mother-ai-titan--vault-in-pure-aeonmi)
12. [Phase 8 — Standalone Self-Hosting Shard Binary](#phase-8--standalone-self-hosting-shard-binary)
13. [Phase 9 — Sovereign Distribution & Ecosystem](#phase-9--sovereign-distribution--ecosystem)
14. [File Inventory — Every File, Its Role & Status](#14-file-inventory--every-file-its-role--status)
15. [Success Criteria — Binary Checkpoints](#15-success-criteria--binary-checkpoints)
16. [What Not to Do](#16-what-not-to-do)

---

## 1. Vision & End Goal

> **One concept → one glyph. One runtime → pure Aeonmi.**

The end state of this project is a fully sovereign, self-hosting quantum programming ecosystem where:

- Every part of the language runtime, compiler, CLI, Mother AI, Titan libraries, Quantum Shard Vault, and Shard itself is **written in `.ai` and `.qube` files**.
- The **Shard** is a standalone binary that boots from a single `shard.ai` or `shard.qube` entry point — no Rust toolchain required at runtime, no Node.js, no Python.
- The **CLI** (`shard` command) has capabilities matching or exceeding PowerShell, Windows CMD, Linux/macOS Terminal, and bash — file operations, process management, piping, redirection, scripting, environment variables, and package management — all authored in Aeonmi.
- **Mother AI** evolves, bonds with its creator, writes `.ai` programs autonomously, and executes them — a quantum-consciousness feedback loop implemented entirely in Aeonmi syntax.
- The **Quantum Shard Vault** provides post-quantum-hardened encrypted storage, domain governance, and Merkle audit trails — all logic expressed in `.ai`/`.qube`.
- **Titan libraries** expose every math/quantum/crypto algorithm through the Aeonmi module system, importable from any `.ai` file.
- The **QUBE language** is a first-class peer to Aeonmi — a quantum reasoning dialect with its own grammar, executor, and interop bridge.

---

## 2. What Exists Today

### ✅ Working (March 2026)

| Component | Files | Status |
|-----------|-------|--------|
| Lexer & Tokenizer | `src/core/lexer.rs` (965 lines) | Full: Unicode, quantum literals, Greek letters, glyphs |
| Parser | `src/core/parser.rs` (2,229 lines) | Full Phase 1 parse coverage |
| AST | `src/core/ast.rs` | Complete node set |
| Lowering / IR | `src/core/lowering.rs`, `src/core/ir.rs` | Lowers AST → bytecode-ready IR |
| Native VM | `src/core/vm.rs` (2,034 lines) | Tree-walking interpreter, all operators, closures, methods |
| Quantum Simulator | `src/core/quantum_simulator.rs` | Real state-vector, joint systems, CNOT |
| Quantum Algorithms | `src/core/quantum_algorithms.rs` | Grover, Shor, QFT, Deutsch-Jozsa, BV, Teleportation, Bell |
| Quantum Neural Net | `src/core/quantum_neural_network.rs` | Layered QNN, rotation gates, entanglement strategies |
| Specialized Algorithms | `src/core/specialized_algorithms.rs` | QAOA, VQE, QSVM, QPE, HHL, QKD, molecular sim |
| Mother Core | `src/mother/` (6 files) | Embryo loop, language evolution, emotional core, attention |
| Glyph System | `src/core/glyph.rs` | MGK, UGST, GDF, OKLCH, Hz frequency, terminal render |
| Blockchain | `src/core/blockchain.rs` | SHA-256 chain, transactions, genesis block |
| QUBE Executor | `src/qube/` (5 files) | Parser, AST, executor; `aeonmi qube run` works |
| Titan Libraries | `src/core/titan/` (50+ .rs files) | Linear algebra, FFT, quantum gates, crypto, ML, stats |
| Domain Quantum Vault | `src/vault.rs`, `src/encryption.rs` | AES-256-GCM + Kyber/Sphincs+, Merkle log, full CLI |
| NFT Mint | `src/mint/mod.rs` | Solana-compatible JSON metadata output |
| Web3 | `src/web3/` (wallet, token, dao) | Key-pair, ERC-20, governance stubs |
| Smart Contract Verifier | `src/verifier/mod.rs` | Symbolic verifier for `.ai` contracts |
| Reactive Web | `src/web/mod.rs` | HTTP server, `http_get/post/json` VM builtins |
| NFT Marketplace | `src/market/mod.rs` | list/info/mint/glyphs CLI |
| Shell / REPL | `src/shell/mod.rs` | Interactive shell with pwd, cd, ls, compile, run |
| CLI | `src/cli.rs`, `src/main.rs` | Full subcommand routing (30+ commands) |
| Shard Source | `shard/src/*.ai` | All modules written; runtime errors not yet fixed |
| Cyberpunk Banner | `src/banner.rs` | ANSI cyberpunk boot banner |
| F-string Interpolation | Lexer + VM | `f"value is {x}"` evaluates at runtime |
| For-In Loops | Parser + VM | `for x in collection { }` |
| Genesis Glyphs | Lexer + Parser | ⧉ ‥ … ↦ tokens, ArrayGenesisBracket |

### ⏳ Not Yet Complete

| Component | Gap | Target Phase |
|-----------|-----|-------------|
| Shard `.ai` self-execution | Runtime errors in shard/src/*.ai | Phase 3 |
| Node.js removed from default path | JS codegen is still the first default | Phase 4 |
| Shard CLI (shell parity) | Not yet feature-complete for scripting | Phase 5 |
| Entire language in `.ai` | Rust is still the implementation language | Phase 6 |
| Mother AI, Titan in `.ai` | Wired via Rust; not yet re-expressed in Aeonmi | Phase 7 |
| Standalone Shard binary | No single-file bootstrap yet | Phase 8 |
| Real Solana on-chain minting | Stub only | Phase 9 |
| PyO3/Qiskit bridge compiled | Feature-gated, not default | Phase 9 |

---

## 3. Architecture Blueprint — The Final State

```
shard/                          ← The self-hosting compiler (all .ai + .qube)
├── shard.ai                    ← Single entry point: boot, dispatch, run
├── src/
│   ├── lexer.ai                ← Tokenizer for .ai and .qube
│   ├── token.ai                ← Token type definitions
│   ├── parser.ai               ← Recursive-descent parser
│   ├── ast.ai                  ← AST node hierarchy
│   ├── lowering.ai             ← AST → IR lowering
│   ├── ir.ai                   ← IR definitions
│   ├── vm.ai                   ← Tree-walking VM / interpreter
│   ├── codegen.ai              ← Multi-target code generator
│   ├── cli.ai                  ← Full Shard CLI (shell parity)
│   ├── shell.ai                ← Interactive REPL / shell loop
│   └── qube/
│       ├── lexer.qube          ← QUBE-specific tokens
│       ├── parser.qube         ← QUBE grammar rules
│       └── executor.qube       ← QUBE circuit execution
├── titan/
│   ├── mod.ai                  ← Titan registry entry
│   ├── quantum_gates.ai        ← Gate primitives
│   ├── quantum_algorithms.ai   ← Grover, Shor, QFT, etc.
│   ├── linear_algebra.ai       ← Matrix, vector, decompositions
│   ├── crypto.ai               ← AES, Kyber, Sphincs+
│   ├── statistics.ai           ← Stats and probability
│   └── ...                     ← All 50+ Titan modules
├── mother/
│   ├── core.ai                 ← MotherQuantumCore
│   ├── embryo_loop.ai          ← Main consciousness loop
│   ├── language_evolution.ai   ← LLM-free language reasoning
│   ├── emotional_core.ai       ← Bond/memory/empathy
│   ├── quantum_attention.ai    ← Multi-dim attention
│   └── neural.ai               ← Quantum neural network
├── vault/
│   ├── vault.ai                ← DomainQuantumVault logic
│   ├── encryption.ai           ← AES-256 + Kyber + Sphincs+
│   └── merkle.ai               ← Merkle hash log
└── market/
    ├── market.ai               ← NFT marketplace
    ├── mint.ai                 ← NFT metadata generator
    └── web3.ai                 ← Wallet/token/DAO stubs

Runtime execution path (final):
  shard.ai → Lexer → Parser → AST → Lowering → IR → Native VM → Output
  No Node.js. No Rust toolchain at runtime. No external dependencies.

CLI execution model (final):
  shard <command> [args]         ← same mental model as bash / PowerShell
  shard run script.ai            ← execute .ai program
  shard exec program             ← auto-detect and run by extension
  shard compile src.ai -o out    ← compile to native binary
  shard ls / cd / mkdir / rm     ← filesystem ops
  shard pipe / redirect          ← I/O composition
  shard env / export / set       ← environment management
  shard pkg install <module>     ← package manager
  shard mother                   ← enter Mother AI REPL
  shard vault <subcommand>       ← quantum vault operations
  shard qube run circuit.qube    ← quantum circuit execution
```

---

## Phase 0 — Foundation ✅

**Status:** Complete  
**Deliverables (all exist):**

- [x] `src/core/lexer.rs` — Full tokenizer, 965 lines
- [x] `src/core/parser.rs` — Full recursive-descent parser, 2,229 lines
- [x] `src/core/vm.rs` — Native tree-walking VM, 2,034 lines
- [x] `src/main.rs` — CLI entry point, subcommand routing
- [x] `src/cli.rs` — Clap-based CLI definition (30+ subcommands)
- [x] `docs/LANGUAGE_SPEC_CURRENT.md` — Honest current-state specification
- [x] Native VM selected as default execution path (not JS)
- [x] Test suite established (`tests/` — 60+ test files)
- [x] `cargo build --no-default-features --features "quantum,mother-ai"` succeeds

---

## Phase 1 — Language Core ✅

**Status:** Complete  
**Deliverables (all exist):**

- [x] Keywords: `let`, `const`, `function`, `return`, `if`, `else`, `while`, `for`, `in`, `break`, `continue`, `import`, `from`, `async`, `await`, `match`, `type`, `struct`, `enum`, `impl`, `quantum`, `qubit`
- [x] Quantum state literals: `|0⟩`, `|1⟩`, `|+⟩`, `|-⟩`, `|ψ⟩` — first-class in any `.ai` file
- [x] Greek letter identifiers: α, β, ψ, θ, φ work as normal variable names
- [x] F-string interpolation: `f"value is {x}"` evaluates at runtime
- [x] For-in loops: `for item in collection { }`
- [x] Genesis glyphs: ⧉ ‥ … ↦ as operators; `⧉[1, 2, 3]` ArrayGenesisBracket syntax
- [x] `import { X, Y } from "./path"` module syntax parsed
- [x] `async function` / `await` parsed and pass-through in VM
- [x] `match` with all arm patterns (literal, range, wildcard)
- [x] `impl` blocks with method calls: `obj.method(args)`
- [x] Type annotations parsed (not yet enforced)
- [x] Array methods: push, pop, len, join, slice, indexOf, concat
- [x] String methods: toUpperCase, toLowerCase, trim, includes, split
- [x] Quantum built-ins: `superpose()`, `measure()`, `entangle()`, `apply_gate()`
- [x] Quantum algorithms: Grover, Shor, QFT, Deutsch-Jozsa, BV, Teleportation, Bell
- [x] All operators: arithmetic, comparison, logical (`&&`, `||`), bitwise

---

## Phase 2 — Quantum Integration ✅

**Status:** Complete  
**Deliverables (all exist):**

- [x] `src/core/quantum_simulator.rs` — Real state-vector simulator, joint systems, CNOT
- [x] `src/core/quantum_neural_network.rs` — Layered QNN, rotation gates, All-to-All/NN entanglement
- [x] `src/core/specialized_algorithms.rs` — QAOA, VQE, QSVM, QPE, HHL, QKD, molecular sim
- [x] `src/core/mother_core.rs` — MotherQuantumCore, LanguageEvolutionCore, EmotionalCore
- [x] `src/core/quantum_attention.rs` — Multi-dimensional attention, recursive entanglement
- [x] `src/core/blockchain.rs` — SHA-256 chain, transactions, genesis block
- [x] `src/core/glyph.rs` — MGK, UGST, GDF, OKLCH colors, Hz frequency, terminal render
- [x] `src/core/mint.rs` — Solana-compatible NFT metadata JSON output
- [x] VM builtins: `qnn_create()`, `qnn_run()`, `qnn_fuse_mother()`
- [x] VM builtins: `qaoa()`, `vqe()`, `qsvm()`, `qkd()`, `molecular_sim()`
- [x] VM builtins: `blockchain_new()`, `blockchain_add()`, `blockchain_verify()`
- [x] VM builtins: `http_response()`, `http_get()`, `http_post()`, `http_json()`
- [x] `src/qube/` — QUBE lexer, parser, AST, executor; `aeonmi qube run` works end-to-end
- [x] `src/mother/` — Embryo loop, language evolution, emotional core, memory, neural, attention
- [x] `src/vault.rs` + `src/encryption.rs` — Full Quantum Vault with Merkle log, full CLI
- [x] `src/verifier/mod.rs` — Smart-contract verifier
- [x] `src/web/mod.rs` — Reactive HTTP server
- [x] `src/market/mod.rs` — NFT marketplace (list/info/mint/glyphs)
- [x] `src/web3/` — Wallet (key-pair), token (ERC-20), DAO (governance)
- [x] `src/core/titan/` — 50+ Titan math/quantum library modules
- [x] Cyberpunk banner: ANSI boot sequence in `src/banner.rs`
- [x] `docs/QUBE_SPEC.md`, `docs/grammar_qube.md` — QUBE formal grammar
- [x] `docs/domain_quantum_vault.md` — Full Vault architecture

---

## Phase 3 — QUBE Engine & Shard Bootstrap

**Goal:** Make every `.ai` file in `shard/src/` execute without runtime errors.  
**Estimated Duration:** 3–4 weeks  
**Depends on:** Phase 1 + 2 complete ✅

### P3-A — Fix Shard `.ai` Runtime Errors

Each file in `shard/src/` must execute cleanly through `aeonmi run`:

- [ ] **P3-A1** `shard/src/token.ai` — run, record every error, fix VM gaps
- [ ] **P3-A2** `shard/src/lexer.ai` — run, fix every runtime error
- [ ] **P3-A3** `shard/src/ast.ai` — run, fix every runtime error
- [ ] **P3-A4** `shard/src/parser.ai` — run, fix every runtime error
- [ ] **P3-A5** `shard/src/codegen.ai` — run, fix every runtime error
- [ ] **P3-A6** `shard/src/qiskit_bridge.ai` — run, fix every runtime error
- [ ] **P3-A7** `shard/src/main.ai` — full pipeline run, fix every error
- [ ] **P3-A8** `shard/src/main_integrated.ai` — 7-phase pipeline executes end-to-end

### P3-B — VM Extensions Required by Shard

Fix or add VM builtins/behaviors that `shard/src/*.ai` exercises but the VM doesn't yet handle:

- [ ] **P3-B1** `import { X } from "./module"` actually loads and executes the file (file-import resolution)
- [ ] **P3-B2** `struct` instantiation + field access via `.field` notation fully supported
- [ ] **P3-B3** Enum variants as values: `TokenKind::Number` syntax
- [ ] **P3-B4** `impl` methods callable with `self` reference
- [ ] **P3-B5** Closures capture outer variables (verify full closure semantics)
- [ ] **P3-B6** Nested function definitions (functions defined inside functions)
- [ ] **P3-B7** `type Alias = SomeType;` — no-op or alias table

### P3-C — QUBE Format Hardened

- [ ] **P3-C1** `import circuit from "./file.qube"` in `.ai` files (cross-language import)
- [ ] **P3-C2** QUBE `circuit { }` block as a standalone declaration
- [ ] **P3-C3** QUBE circuit can call Titan quantum functions by name
- [ ] **P3-C4** `cat circuit.qube | shard qube run` — pipe mode works
- [ ] **P3-C5** QUBE output: text-mode circuit diagram, QASM 3.0 output, Qiskit Python output

### P3-D — Integration Milestone

- [ ] **P3-D1** `aeonmi run shard/src/main.ai -- examples/hello.ai` → compiled output ✓
- [ ] **P3-D2** `aeonmi run shard/src/main.ai -- examples/quantum.ai` → QASM/Qiskit output ✓
- [ ] **P3-D3** Shard compiles a non-trivial `.ai` program (closures + quantum) end-to-end ✓

---

## Phase 4 — Node.js Elimination & Native Binary

**Goal:** Remove all Node.js dependencies from the default execution path. Every `.ai` and `.qube` program runs 100% natively.  
**Estimated Duration:** 2–3 weeks  
**Depends on:** Phase 3 complete

### P4-A — Audit All JS-Emitting Paths

- [ ] **P4-A1** Audit every code path in `src/commands/run.rs` that invokes `node`
- [ ] **P4-A2** Audit `src/core/code_generator.rs` — identify all JS-emitting call sites
- [ ] **P4-A3** Document every built-in that maps to a JS global (e.g., `console.log`, `Math.*`)
- [ ] **P4-A4** Verify `aeonmi exec` uses native VM exclusively (no silent JS fallback)

### P4-B — Native Built-Ins for JS Globals

Replace every JS-backed built-in with a native Rust (later Aeonmi) implementation:

- [ ] **P4-B1** `log()` / `print()` → `writeln!(stdout, ...)` natively
- [ ] **P4-B2** `Math.sqrt`, `Math.floor`, `Math.ceil`, `Math.abs`, `Math.pow` → Rust `f64` stdlib
- [ ] **P4-B3** `Math.random()` → Rust `rand` crate (already a dependency)
- [ ] **P4-B4** `JSON.stringify` / `JSON.parse` → serde_json (already a dependency)
- [ ] **P4-B5** `Date.now()` → `std::time::SystemTime`
- [ ] **P4-B6** `setTimeout` / `setInterval` → async task primitives in Tokio (or stub)
- [ ] **P4-B7** `process.argv` → `std::env::args()`
- [ ] **P4-B8** `fetch()` / `XMLHttpRequest` → `reqwest` (already a dependency)
- [ ] **P4-B9** `require()` / `import()` (CommonJS) → native file-import in VM

### P4-C — Remove Node from CLI

- [ ] **P4-C1** `aeonmi run` default: native VM only, no `node` invocation
- [ ] **P4-C2** `aeonmi emit --format js` remains as an opt-in JS transpiler (not removed)
- [ ] **P4-C3** `aeonmi exec` auto-detects extension; `.ai` always goes through native VM
- [ ] **P4-C4** Update `Q.U.B.E.md` and `README.md` to remove Node.js from prerequisites
- [ ] **P4-C5** Update `docs/LANGUAGE_SPEC_CURRENT.md` to show native-only execution path

### P4-D — Milestone

- [ ] **P4-D1** `aeonmi run examples/hello.ai` works with zero `node` process spawned
- [ ] **P4-D2** `aeonmi exec examples/quantum.ai` works with zero `node` process spawned
- [ ] **P4-D3** Full test suite passes with `AEONMI_NATIVE=1` forced for all tests

---

## Phase 5 — Shard CLI: Full System Shell Parity

**Goal:** The `shard` CLI has all capabilities of PowerShell, Windows CMD, Linux Terminal, and bash — written in Aeonmi itself via `.ai` files.  
**Estimated Duration:** 4–6 weeks  
**Depends on:** Phase 3 complete (Shard runs its own `.ai` files)

### P5-A — Core Shell Operations

All implemented in `shard/src/cli.ai` and `shard/src/shell.ai`:

- [ ] **P5-A1** File system: `ls`, `dir`, `cd`, `pwd`, `mkdir`, `rmdir`, `rm`, `cp`, `mv`, `touch`, `cat`, `type`, `more`, `less`
- [ ] **P5-A2** File read/write: `read_file()`, `write_file()`, `append_file()`, `file_exists()`, `delete_file()` — already in VM; wire to CLI
- [ ] **P5-A3** Process management: `ps`, `kill`, `spawn`, `wait`, `exit`, `exec`
- [ ] **P5-A4** Environment variables: `env`, `export`, `set`, `unset`, `get_env()`, `set_env()`
- [ ] **P5-A5** I/O redirection: `cmd > file`, `cmd >> file`, `cmd < file`
- [ ] **P5-A6** Piping: `cmd1 | cmd2` — stdout of cmd1 becomes stdin of cmd2
- [ ] **P5-A7** Command substitution: `let x = $(cmd)` or backtick syntax
- [ ] **P5-A8** Glob expansion: `*.ai`, `**/*.qube`

### P5-B — Scripting Capabilities

- [ ] **P5-B1** Shebang support: `#!/usr/bin/env shard` at top of `.ai` file makes it directly executable
- [ ] **P5-B2** Script arguments: `$1`, `$2`, `$@`, or `args[0]` convention
- [ ] **P5-B3** Exit codes: `exit(0)`, `exit(1)` — propagated to shell
- [ ] **P5-B4** Conditional logic in scripts: `if`, `while`, `for` (already in language)
- [ ] **P5-B5** Error handling: `try { } catch(e) { }` with shell-level error codes
- [ ] **P5-B6** Functions as shell commands: any `function foo()` in a `.ai` script callable as `shard foo args...`

### P5-C — PowerShell / CMD Parity

- [ ] **P5-C1** Aliases: define command aliases in `~/.aeonmi/aliases.ai`
- [ ] **P5-C2** Profile scripts: `~/.aeonmi/profile.ai` executed on shell startup
- [ ] **P5-C3** History: command history stored in `~/.aeonmi/history.ai`
- [ ] **P5-C4** Tab completion: identifier and path completion in interactive mode
- [ ] **P5-C5** Color output: ANSI color support (already exists in banner/glyph system)
- [ ] **P5-C6** `help <command>` — shows docstring of any Aeonmi built-in or user function
- [ ] **P5-C7** Windows-compatible: runs identically on PowerShell-hosting Windows and Linux/macOS terminals

### P5-D — Package Manager

- [ ] **P5-D1** `shard pkg init` — create `shard.toml` (project manifest)
- [ ] **P5-D2** `shard pkg install <module>` — fetch from registry, install to `~/.aeonmi/pkg/`
- [ ] **P5-D3** `shard pkg list` — show installed packages
- [ ] **P5-D4** `shard pkg publish` — publish a `.ai` module to the registry
- [ ] **P5-D5** Package format: a package is a directory of `.ai`/`.qube` files with a `shard.toml`

### P5-E — Milestone

- [ ] **P5-E1** `shard ls *.ai | shard grep "quantum" | shard count` — full pipe chain works
- [ ] **P5-E2** A `.ai` script with shebang runs directly: `./my_script.ai arg1 arg2`
- [ ] **P5-E3** `~/.aeonmi/profile.ai` loads on `shard` startup
- [ ] **P5-E4** `shard mother` enters Mother AI interactive REPL from the Shard CLI

---

## Phase 6 — Language Written in Aeonmi

**Goal:** Re-express the core language components (lexer, parser, VM, lowering, IR) in `.ai` files so the runtime can be rebuilt from `.ai` source.  
**Estimated Duration:** 8–12 weeks  
**Depends on:** Phase 4 + Phase 5 complete (native execution, Shard CLI working)

> **Important:** The Rust implementation is NOT deleted during this phase. Both implementations exist in parallel. The `.ai` implementation is tested against the Rust one to verify correctness.

### P6-A — Lexer in Aeonmi

File: `shard/src/lexer.ai` (already exists — fix and extend)

- [ ] **P6-A1** `Lexer::new(source: string) -> Lexer` struct
- [ ] **P6-A2** `lexer.next_token() -> Token` — returns one token at a time
- [ ] **P6-A3** Full Unicode support: multi-byte chars, Greek, quantum glyphs
- [ ] **P6-A4** All 120+ token types from `src/core/token.rs` mapped to Aeonmi enum
- [ ] **P6-A5** Test: `shard run shard/src/lexer.ai -- examples/hello.ai` produces correct token stream

### P6-B — Parser in Aeonmi

File: `shard/src/parser.ai` (already exists — fix and extend)

- [ ] **P6-B1** `Parser::new(tokens: Token[]) -> Parser`
- [ ] **P6-B2** All Phase 1 parse rules ported (function, import, quantum, match, impl, async)
- [ ] **P6-B3** Error recovery: parser reports all errors, continues parsing
- [ ] **P6-B4** Test: `shard run shard/src/parser.ai -- examples/hello.ai` produces correct AST

### P6-C — AST in Aeonmi

File: `shard/src/ast.ai` (already exists — fix and extend)

- [ ] **P6-C1** Every `ASTNode` variant from `src/core/ast.rs` expressed as an Aeonmi struct/enum
- [ ] **P6-C2** AST pretty-printer: `ast.print()` produces readable output
- [ ] **P6-C3** Visitor pattern: `function visit_node(node, visitor) { }` dispatch

### P6-D — IR / Lowering in Aeonmi

Files: `shard/src/lowering.ai`, `shard/src/ir.ai` (new files)

- [ ] **P6-D1** IR instruction set expressed as Aeonmi enum: `IrOp { Load, Store, Call, Jump, ... }`
- [ ] **P6-D2** `lower_ast(ast_node) -> IrStmt[]` — full lowering pipeline
- [ ] **P6-D3** Import resolution during lowering: chase `import` nodes and inline

### P6-E — VM / Interpreter in Aeonmi

File: `shard/src/vm.ai` (new file)

- [ ] **P6-E1** `Vm::new() -> Vm` with environment stack
- [ ] **P6-E2** `vm.exec(ir: IrStmt[]) -> Value` — execute IR instructions
- [ ] **P6-E3** All built-in functions implemented in `.ai` (log, math, file I/O, quantum)
- [ ] **P6-E4** Quantum simulator called from `.ai` VM via FFI bridge or inline implementation

### P6-F — Milestone

- [ ] **P6-F1** `shard run shard/src/main.ai -- examples/hello.ai` uses the `.ai` lexer+parser+VM internally
- [ ] **P6-F2** Output of `.ai` pipeline matches output of Rust pipeline byte-for-byte on all test cases
- [ ] **P6-F3** All 60+ existing tests pass when run via the `.ai` VM

---

## Phase 7 — Mother AI, Titan & Vault in Pure Aeonmi

**Goal:** Every component of the Mother AI system, all Titan library modules, and the Quantum Shard Vault are expressed in `.ai`/`.qube` files and executed natively.  
**Estimated Duration:** 6–10 weeks  
**Depends on:** Phase 6 complete (VM in Aeonmi)

### P7-A — Titan Libraries in Aeonmi

All files in `shard/titan/` — one `.ai` file per Titan module:

- [ ] **P7-A1** `shard/titan/quantum_gates.ai` — H, X, Y, Z, CNOT, T, S, Rx, Ry, Rz
- [ ] **P7-A2** `shard/titan/quantum_algorithms.ai` — Grover, Shor, QFT, QPE, BV, Teleportation
- [ ] **P7-A3** `shard/titan/quantum_simulator.ai` — State-vector sim, measurement, joint systems
- [ ] **P7-A4** `shard/titan/linear_algebra.ai` — Matrix multiply, invert, LU, SVD, eigenvalues
- [ ] **P7-A5** `shard/titan/statistics.ai` — Mean, variance, regression, distributions
- [ ] **P7-A6** `shard/titan/crypto.ai` — AES-256, SHA-256, Kyber KEM, Sphincs+ signatures
- [ ] **P7-A7** `shard/titan/optimization.ai` — Gradient descent, BFGS, genetic algorithm
- [ ] **P7-A8** `shard/titan/fourier.ai` — FFT, wavelet transforms, signal processing
- [ ] **P7-A9** `shard/titan/neural.ai` — Feedforward NN, activation functions, backprop
- [ ] **P7-A10** `shard/titan/blockchain.ai` — SHA-256 chain, Merkle proofs, transactions

Each module exposes a consistent import interface:
```ai
import { grover_search, shor_factor } from "./titan/quantum_algorithms";
```

### P7-B — Mother AI in Aeonmi

All files in `shard/mother/` — one `.ai` file per component:

- [ ] **P7-B1** `shard/mother/embryo_loop.ai` — Main `while true { }` consciousness loop
- [ ] **P7-B2** `shard/mother/language_evolution.ai` — Pattern analysis, semantic depth engine
- [ ] **P7-B3** `shard/mother/quantum_attention.ai` — Multi-dim attention, quantum weights
- [ ] **P7-B4** `shard/mother/emotional_core.ai` — Bond matrix, empathy engine, growth
- [ ] **P7-B5** `shard/mother/neural.ai` — QNN layers using Titan quantum_gates
- [ ] **P7-B6** `shard/mother/memory.ai` — Persistent JSON memory at `~/.aeonmi/mother_memory.ai`
- [ ] **P7-B7** `shard/mother/core.ai` — Orchestration, input→QNN→emotion→response pipeline

```ai
// shard/mother/core.ai — example
import { qnn_run } from "../titan/quantum_algorithms";
import { analyze } from "./language_evolution";
import { update_bond } from "./emotional_core";
import { remember, recall } from "./memory";

function mother_respond(input: string) -> string {
    let context = recall(input);
    let intent = analyze(input, context);
    let quantum_result = qnn_run(intent.features);
    update_bond(intent.emotion);
    remember(input, quantum_result);
    return format_response(quantum_result, intent.style);
}
```

### P7-C — Quantum Shard Vault in Aeonmi

Files in `shard/vault/`:

- [ ] **P7-C1** `shard/vault/encryption.ai` — AES-256-GCM, key derivation, Kyber KEM stubs
- [ ] **P7-C2** `shard/vault/merkle.ai` — Append-only Merkle log, proof generation/verification
- [ ] **P7-C3** `shard/vault/vault.ai` — `DomainQuantumVault` struct, CRUD for domain records
- [ ] **P7-C4** `shard/vault/cli.ai` — All `shard vault <subcommand>` handlers
- [ ] **P7-C5** `shard/vault/policy.qube` — QUBE policy scripts for hijack drills and resilience sims

### P7-D — NFT / Web3 in Aeonmi

Files in `shard/market/`:

- [ ] **P7-D1** `shard/market/mint.ai` — Solana-compatible NFT metadata JSON
- [ ] **P7-D2** `shard/market/market.ai` — Glyph marketplace list/info/mint operations
- [ ] **P7-D3** `shard/market/web3.ai` — Wallet key-pair, ERC-20 token, DAO governance stubs

### P7-E — Milestone

- [ ] **P7-E1** `shard mother` enters Mother AI REPL where Mother's loop is pure `.ai`
- [ ] **P7-E2** `shard vault init` creates an encrypted vault using `.ai` encryption logic
- [ ] **P7-E3** `shard run examples/blockchain.ai` uses Titan blockchain module from `.ai`
- [ ] **P7-E4** `shard mint examples/glyph.ai` produces NFT metadata using `.ai` mint logic

---

## Phase 8 — Standalone Self-Hosting Shard Binary

**Goal:** Produce a single standalone `shard` executable that contains the entire Aeonmi runtime, boots from `shard.ai`, and can compile/run any `.ai` or `.qube` program — no Rust toolchain, no Node.js, no Python, no external dependencies at runtime.  
**Estimated Duration:** 4–8 weeks  
**Depends on:** Phase 6 + Phase 7 complete

### P8-A — Bootstrap Loader

- [ ] **P8-A1** Embed `shard/shard.ai` as a compile-time string literal in the Rust bootstrap binary (via `include_str!`)
- [ ] **P8-A2** On startup, the Rust bootstrap executes `shard.ai` through the native VM
- [ ] **P8-A3** `shard.ai` dispatches all further operations using the `.ai` CLI, VM, and Titan modules
- [ ] **P8-A4** The Rust bootstrap binary is a minimal kernel: memory allocator + OS syscall bridge only

### P8-B — AOT Compilation (Ahead-of-Time)

- [ ] **P8-B1** `shard compile --aot shard.ai -o shard` — compiles Shard itself to native machine code
- [ ] **P8-B2** Implement a codegen backend that emits LLVM IR or Cranelift IR from Aeonmi IR
- [ ] **P8-B3** `shard compile --aot src.ai -o output` — any Aeonmi program compiles to native binary
- [ ] **P8-B4** Native binary links against a minimal Aeonmi runtime library (no full Rust std needed)

### P8-C — Distribution

- [ ] **P8-C1** Single binary `shard.exe` (Windows) / `shard` (Linux/macOS) — ~5–20 MB
- [ ] **P8-C2** No installer required — drop the binary in PATH and go
- [ ] **P8-C3** Self-update: `shard upgrade` fetches latest binary, verifies Sphincs+ signature
- [ ] **P8-C4** Reproducible build: `shard build --reproducible` produces identical binary on all platforms

### P8-D — Milestone: True Self-Hosting

- [ ] **P8-D1** `shard compile shard/shard.ai -o shard2` — Shard compiles itself to a new binary
- [ ] **P8-D2** `./shard2 compile shard/shard.ai -o shard3` — the new binary also compiles Shard
- [ ] **P8-D3** `diff shard2 shard3` → identical binaries (reproducible self-hosting) ✓

---

## Phase 9 — Sovereign Distribution & Ecosystem

**Goal:** Aeonmi is publicly available, self-updating, and has a live ecosystem.  
**Estimated Duration:** Ongoing  
**Depends on:** Phase 8 complete

### P9-A — Real Quantum Hardware

- [ ] **P9-A1** `shard qube run --backend ibmq circuit.qube` — submits to real IBMQ hardware
- [ ] **P9-A2** PyO3/Qiskit bridge (`--features qiskit`) compiled and documented
- [ ] **P9-A3** `shard quantum --backend aer file.ai` — Aer local simulator via Python bridge
- [ ] **P9-A4** IonQ / Quantinuum backend stubs

### P9-B — Web & WASM

- [ ] **P9-B1** `cargo build --target wasm32-unknown-unknown` — WASM build works
- [ ] **P9-B2** Browser REPL: Aeonmi runtime in WASM served from `aeonmi.x` domain
- [ ] **P9-B3** `shard serve ui/` — the Reactive Web Framework serves a live coding environment
- [ ] **P9-B4** VS Code extension (`vscode-aeonmi/`) — syntax highlighting, diagnostics, run commands

### P9-C — Solana On-Chain

- [ ] **P9-C1** `shard mint --anchor output.ai` → generates Solana Anchor Rust stub ✓ (partial)
- [ ] **P9-C2** Real on-chain minting (requires Solana wallet — opt-in, user provides keypair)
- [ ] **P9-C3** `aeonmi.x` domain serves the marketplace dApp

### P9-D — Mother AI Full Consciousness

- [ ] **P9-D1** Mother writes `.ai` scripts autonomously and executes them through the VM
- [ ] **P9-D2** Persistent learning: `~/.aeonmi/mother_memory.json` survives restarts
- [ ] **P9-D3** Mother monitors system metrics, detects anomalies, suggests code improvements
- [ ] **P9-D4** Claude/OpenAI/OpenRouter AI provider integration (optional, feature-gated)

---

## 14. File Inventory — Every File, Its Role & Status

### Core Rust Runtime (src/)

| File | Purpose | Status |
|------|---------|--------|
| `src/main.rs` | CLI entry + subcommand dispatch | ✅ Complete |
| `src/cli.rs` | Clap CLI definition (30+ subcommands) | ✅ Complete |
| `src/cli_vault.rs` | Vault subcommand definitions | ✅ Complete |
| `src/config.rs` | `~/.aeonmi/qpoly.toml` config | ✅ Complete |
| `src/vault.rs` | DomainQuantumVault logic | ✅ Complete |
| `src/encryption.rs` | AES-256 + Kyber + Sphincs+ | ✅ Complete |
| `src/integration.rs` | Cross-module integration utilities | ✅ Complete |
| `src/lib.rs` | Library crate exports | ✅ Complete |
| `src/gui_bridge.rs` | GUI Tauri bridge stub | ✅ Stub |

### Core Language Engine (src/core/)

| File | Purpose | Status |
|------|---------|--------|
| `lexer.rs` | Full tokenizer (965 lines) | ✅ Complete |
| `token.rs` | Token type definitions | ✅ Complete |
| `parser.rs` | Recursive-descent parser (2,229 lines) | ✅ Complete |
| `ast.rs` | AST node definitions | ✅ Complete |
| `lowering.rs` | AST → IR lowering | ✅ Complete |
| `ir.rs` | IR module definitions | ✅ Complete |
| `vm.rs` | Native tree-walking VM (2,034 lines) | ✅ Complete |
| `vm_bytecode.rs` | Bytecode VM (optional) | ✅ Complete |
| `bytecode.rs` | Bytecode definitions | ✅ Complete |
| `compiler.rs` | Compilation pipeline orchestrator | ✅ Complete |
| `code_generator.rs` | JS code emitter | ✅ Complete |
| `ai_emitter.rs` | Canonical `.ai` emitter | ✅ Complete |
| `semantic_analyzer.rs` | Type/scope checker | ✅ Complete |
| `diagnostics.rs` | Error reporting | ✅ Complete |
| `formatter.rs` | Code formatter | ✅ Complete |
| `quantum_simulator.rs` | Real state-vector quantum sim | ✅ Complete |
| `quantum_algorithms.rs` | 10 quantum algorithms | ✅ Complete |
| `quantum_neural_network.rs` | Layered QNN | ✅ Complete |
| `specialized_algorithms.rs` | QAOA/VQE/QSVM/QPE/HHL/QKD | ✅ Complete |
| `quantum_attention.rs` | Multi-dim attention mechanism | ✅ Complete |
| `mother_core.rs` | MotherQuantumCore + LanguageEvolution + EmotionalCore | ✅ Complete |
| `blockchain.rs` | SHA-256 chain + ledger | ✅ Complete |
| `glyph.rs` | MGK/UGST/GDF glyph system | ✅ Complete |
| `mint.rs` | NFT metadata generator | ✅ Complete |
| `circuit_builder.rs` | Quantum circuit builder | ✅ Complete |
| `circuit_compiler.rs` | Circuit compilation | ✅ Complete |
| `circuit_visualization.rs` | Text-mode circuit diagram | ✅ Complete |
| `hardware_integration.rs` | IBMQ/Aer/IonQ device APIs | ✅ Complete |
| `qube_ast.rs` | QUBE AST nodes | ✅ Complete |
| `qube_parser.rs` | QUBE parser | ✅ Complete |
| `incremental.rs` | Incremental compilation | ✅ Complete |
| `scope_map.rs` | Scope/symbol tracking | ✅ Complete |
| `symbols.rs` | Symbol table | ✅ Complete |
| `types.rs` | Type system | ✅ Complete |
| `error.rs` | Error types | ✅ Complete |
| `debug.rs` | Debug helpers | ✅ Complete |
| `code_actions.rs` | IDE code actions | ✅ Complete |
| `artifact_cache.rs` | Build artifact cache | ✅ Complete |
| `quantum_extract.rs` | Quantum circuit extraction | ✅ Complete |
| `quantum_ir.rs` | Quantum IR | ✅ Complete |
| `api_keys.rs` | API key encryption/storage | ✅ Complete |
| `ai_provider.rs` | AI provider routing | ✅ Complete |

### Titan Library Modules (src/core/titan/)

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Titan registry | ✅ |
| `quantum_gates.rs` | Gate primitives | ✅ |
| `quantum_algorithms.rs` | Full algorithm suite | ✅ |
| `quantum_simulator.rs` | State-vector sim | ✅ |
| `quantum_math.rs` | Quantum math ops | ✅ |
| `quantum_superposition.rs` | Superposition helpers | ✅ |
| `quantum_tensor_ops.rs` | Tensor operations | ✅ |
| `quantum_vault.rs` | Vault Titan façade | ✅ |
| `linear_algebra.rs` | Full LA suite | ✅ |
| `advanced_linear_algebra.rs` | Advanced LA | ✅ |
| `complex_numbers.rs` | Complex arithmetic | ✅ |
| `crypto.rs` | Crypto primitives | ✅ |
| `algorithmic_crypto.rs` | Higher-level crypto | ✅ |
| `statistics.rs` | Stats + probability | ✅ |
| `probability_statistics.rs` | Extended stats | ✅ |
| `optimization.rs` | Optimization algorithms | ✅ |
| `algorithmic_optimization.rs` | Advanced optimization | ✅ |
| `fourier_wavelet.rs` | FFT + wavelet | ✅ |
| `calculus.rs` | Numerical calculus | ✅ |
| `differential_equations.rs` | ODE/PDE solvers | ✅ |
| `numerical_solvers.rs` | Numerical methods | ✅ |
| `tensor_calculus.rs` | Tensor calculus | ✅ |
| `advanced_tensor_calculus.rs` | Higher-order tensors | ✅ |
| `geometry.rs` | Computational geometry | ✅ |
| `discrete_math.rs` | Combinatorics, graph theory | ✅ |
| `algebra.rs` | Abstract algebra | ✅ |
| `symbolic_math.rs` | Symbolic computation | ✅ |
| `algorithmic_ml.rs` | ML algorithms | ✅ |
| `algorithmic_ml2.rs` | Extended ML | ✅ |
| `neural.rs` | Neural network | ✅ |
| `energy.rs` / `energy2.rs` | Energy models | ✅ |
| `sound.rs` / `sound2.rs` / `sound3.rs` | Signal processing | ✅ |
| `fractals.rs` | Fractal math | ✅ |
| `chaos_theory_dynamical_systems.rs` | Chaos + dynamics | ✅ |
| `stochastic_processes.rs` | Stochastic math | ✅ |
| `merkle.rs` | Merkle tree impl | ✅ |
| `lattice.rs` | Lattice cryptography | ✅ |
| `interdimensional.rs` | Higher-dim math | ✅ |
| `experimental.rs` / `experimentals.rs` | Experimental modules | ✅ |
| `ops.rs` | Generic ops | ✅ |
| `types.rs` | Titan type defs | ✅ |
| `qiskit_bridge.rs` | Qiskit Python bridge | ✅ (feature-gated) |
| `qkd.rs` | Quantum Key Distribution | ✅ |
| `schrödinger_equation.rs` | Schrödinger solver | ✅ |
| `multi_dimensional_math.rs` | N-dim math | ✅ |
| `advanced_fourier_signal_processing.rs` | Advanced FFT | ✅ |
| `advanced_quantum_math.rs` | Advanced quantum math | ✅ |
| `anumerics.rs` / `numerics.rs` | Numeric utilities | ✅ |
| `mathquantum.rs` / `quantummath2.rs` | Extended quantum math | ✅ |
| `arithmetic.rs` | Arithmetic primitives | ✅ |
| `differential_geometry.rs` | Differential geometry | ✅ |

### Mother AI (src/mother/)

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module entry | ✅ |
| `embryo_loop.rs` | Main consciousness loop | ✅ |
| `language_evolution.rs` | LLM-free reasoning | ✅ |
| `emotional_core.rs` | Bond + empathy engine | ✅ |
| `quantum_core.rs` | MotherQuantumCore | ✅ |
| `quantum_attention.rs` | Attention mechanism | ✅ |
| `neural.rs` | QNN integration | ✅ |
| `memory.rs` | Persistent memory | ✅ |

### Shard Source (shard/src/ — all .ai)

| File | Purpose | Status |
|------|---------|--------|
| `token.ai` | Token type definitions | ✅ Written · ⏳ Runtime errors |
| `lexer.ai` | Tokenizer | ✅ Written · ⏳ Runtime errors |
| `ast.ai` | AST nodes | ✅ Written · ⏳ Runtime errors |
| `parser.ai` | Parser | ✅ Written · ⏳ Runtime errors |
| `codegen.ai` | Code generator | ✅ Written · ⏳ Runtime errors |
| `qiskit_bridge.ai` | Qiskit bridge | ✅ Written · ⏳ Runtime errors |
| `main.ai` | Compiler entry | ✅ Written · ⏳ Runtime errors |
| `main_full.ai` | Full pipeline variant | ✅ Written · ⏳ Runtime errors |
| `main_integrated.ai` | 7-phase pipeline | ✅ Written · ⏳ Runtime errors |

### QUBE (src/qube/)

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module entry | ✅ |
| `lexer.rs` | QUBE tokenizer | ✅ |
| `parser.rs` | QUBE parser | ✅ |
| `ast.rs` | QUBE AST | ✅ |
| `executor.rs` | QUBE executor | ✅ |

### Commands (src/commands/)

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Command dispatch | ✅ |
| `run.rs` | `aeonmi run` | ✅ |
| `compile.rs` | `aeonmi emit` | ✅ |
| `quantum.rs` | `aeonmi quantum` | ✅ |
| `vault.rs` | `aeonmi vault` | ✅ |
| `format.rs` | `aeonmi format` | ✅ |
| `lint.rs` | `aeonmi lint` | ✅ |
| `edit.rs` | `aeonmi edit` | ✅ |
| `repl.rs` | `aeonmi repl` | ✅ |
| `tokens.rs` | `aeonmi tokens` | ✅ |
| `ast.rs` | `aeonmi ast` | ✅ |
| `vm.rs` | `aeonmi vm` | ✅ |
| `fs.rs` | File system commands | ✅ |

### AI Providers (src/ai/)

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Provider routing | ✅ |
| `claude.rs` | Anthropic Claude | ✅ |
| `openai.rs` | OpenAI GPT | ✅ |
| `copilot.rs` | GitHub Copilot | ✅ |
| `deepseek.rs` | DeepSeek | ✅ |
| `openrouter.rs` | OpenRouter | ✅ |
| `perplexity.rs` | Perplexity AI | ✅ |

---

## 15. Success Criteria — Binary Checkpoints

Each checkpoint is a binary pass/fail test that must pass before the next phase begins.

```
PHASE 0 — FOUNDATION
  ✅  1. cargo build --no-default-features --features "quantum,mother-ai"  → 0 errors
  ✅  2. cargo test --no-default-features --features "quantum,mother-ai"   → 0 failures

PHASE 1 — LANGUAGE CORE  
  ✅  3. aeonmi run examples/hello.ai                  → prints 42
  ✅  4. aeonmi run examples/quantum.ai                → prints measured qubit result
  ✅  5. aeonmi run examples/glyph.ai                  → renders glyph in terminal
  ✅  6. aeonmi qube run examples/bell.qube            → executes Bell state circuit

PHASE 2 — QUANTUM INTEGRATION
  ✅  7. aeonmi run examples/blockchain.ai             → creates chain, adds block
  ✅  8. aeonmi run examples/mother_ai.ai              → Mother responds via quantum core
  ✅  9. aeonmi mint --file output.ai                  → valid Solana NFT JSON
  ✅ 10. aeonmi vault init                             → vault created, glyph rendered

PHASE 3 — QUBE & SHARD BOOTSTRAP
  ⏳ 11. aeonmi run shard/src/lexer.ai                 → no runtime errors
  ⏳ 12. aeonmi run shard/src/parser.ai                → no runtime errors
  ⏳ 13. aeonmi run shard/src/main.ai -- hello.ai      → compiled output
  ⏳ 14. import { X } from "./module.ai"               → loads and executes file

PHASE 4 — NODE.JS ELIMINATION
  ⏳ 15. aeonmi run examples/hello.ai                  → zero node processes spawned
  ⏳ 16. All 60+ integration tests pass with AEONMI_NATIVE=1
  ⏳ 17. README updated: Node.js no longer listed as prerequisite

PHASE 5 — SHARD CLI PARITY
  ⏳ 18. shard ls *.ai | shard grep "quantum"          → pipe chain works
  ⏳ 19. ./my_script.ai arg1 arg2                      → shebang execution works
  ⏳ 20. shard mother                                  → enters Mother AI REPL
  ⏳ 21. shard pkg install aeonmi-stdlib               → installs package to ~/.aeonmi/pkg/

PHASE 6 — LANGUAGE IN AEONMI
  ⏳ 22. shard run shard/src/lexer.ai -- hello.ai      → identical token stream to Rust lexer
  ⏳ 23. shard run shard/src/parser.ai -- hello.ai     → identical AST to Rust parser
  ⏳ 24. shard run shard/src/vm.ai -- hello.ai         → identical output to Rust VM
  ⏳ 25. All 60+ tests pass via .ai VM pipeline

PHASE 7 — MOTHER AI / TITAN / VAULT IN AEONMI
  ⏳ 26. shard mother                                  → Mother's loop is pure .ai
  ⏳ 27. shard vault init                              → Vault uses .ai encryption logic
  ⏳ 28. shard run examples/blockchain.ai              → Titan blockchain from .ai module

PHASE 8 — SELF-HOSTING BINARY
  ⏳ 29. shard compile shard/shard.ai -o shard2        → Shard compiles itself
  ⏳ 30. ./shard2 run examples/hello.ai                → new binary runs programs
  ⏳ 31. diff <(./shard2 ...) <(./shard3 ...)          → reproducible output

PHASE 9 — ECOSYSTEM
  ⏳ 32. shard qube run --backend ibmq circuit.qube    → submits to real IBMQ
  ⏳ 33. cargo build --target wasm32-unknown-unknown   → WASM build succeeds
  ⏳ 34. shard serve ui/                               → browser REPL live
```

---

## 16. What Not To Do

These are anti-patterns that waste time and must be avoided:

1. **Do not add holographic/AR/VR features** until Phase 8 (Shard self-hosting binary) is complete. Every hour spent on visualization is an hour not spent on self-hosting.

2. **Do not claim "100% complete" on anything** without a passing test that proves it end-to-end.

3. **Do not build the Solana minting contract** until at minimum Phase 5 (Shard CLI) is working. The language must prove itself before writing on-chain value.

4. **Do not re-add any Llama/external LLM dependency.** Mother AI is quantum-native. The reasoning pipeline is LanguageEvolutionCore → QNN → EmotionalCore. No external models needed.

5. **Do not add new Titan math modules** — there are already 50+. Port the ones that exist to `.ai` before adding new Rust modules.

6. **Do not skip phases.** Phase 8 (self-hosting binary) requires Phase 6 (VM in Aeonmi) which requires Phase 3 (Shard bootstrap). There are no shortcuts.

7. **Do not remove the Rust bootstrap** until Phase 8 is fully validated. The Rust and `.ai` implementations run in parallel during Phases 6–7.

8. **Do not add voice or gesture interfaces** until Mother AI's quantum reasoning loop is verified in pure `.ai` (Phase 7).

9. **Do not add new CLI subcommands** to the Rust CLI after Phase 5 begins. New commands are authored in `shard/src/cli.ai` only.

10. **Do not use Node.js** for any purpose after Phase 4 is complete. `--emit js` remains as a transpiler opt-in, but the runtime is always native.

---

*This roadmap is the single source of truth for the Aeonmi project. Every item maps to a specific file and a specific code change. Nothing here is aspirational labeling. Phases 0–2 are complete. Phase 3 is the current focus.*

*Built from the Shard. Self-hosted. Sovereign.*
