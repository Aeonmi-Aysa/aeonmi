# Aeonmi Shard README

## 1. What the Shard Is
Aeonmi is an AI-native, quantum-conscious programming environment built as a "shard": one self-contained stack that bundles language, runtimes, simulation engines, security vaults, and assistant interfaces. This document explains every major layer so new contributors can navigate the codebase and connect their own AI or quantum tooling without hunting through multiple specs.

**Shard pillars**
- **Aeonmi language core** – surface syntax, parser, semantic passes, native VM, and transpilers.
- **QUBE symbolic layer** – hieroglyphic and tensor-native syntax for future adaptive execution.
- **Titan computation engine** – math, quantum, and physics primitives exposed to the shard.
- **Mother AI** – orchestrator REPL that glues AI providers, system health, and user command loops.
- **Domain Quantum Vault** – zero-trust registrar vault with hybrid crypto and Titan simulations.
- **Quantum bridges** – Titan-native simulators plus optional Qiskit integration through Python.
- **AI provider registry** – feature-gated connectors (OpenAI, Copilot, Perplexity, DeepSeek, ...).

## 2. Repository Tour
| Area | Purpose | Key Paths |
|------|---------|-----------|
| Language pipeline | Lexer → parser → semantic analysis → IR → VM/JS emit | `src/core/{lexer,parser,semantic_analyzer,lowering,vm}.rs` |
| Bytecode & metrics | Optional bytecode VM, incremental recompilation, metrics persistence | `src/core/{vm_bytecode,incremental,metrics_*}.rs` |
| Titan engine | Numerics, algebra, quantum ops, simulations, vault helpers | `src/core/titan/` (many modules) |
| AI integrations | Provider registry + backend shims | `src/ai/` |
| CLI & shell | Command parsing, REPL, TUI editor, vault commands | `src/{cli,commands,cli_vault,tui}` |
| Mother AI shard | Async launcher + conversational interface | `mother_ai/main.rs` |
| Vault | Hybrid encryption primitives + domain manager | `src/{encryption,vault}.rs`, `src/core/titan/quantum_vault.rs` |
| Docs & guides | Language guide, vault whitepaper, shard readme (this file) | `docs/` |

Build artifacts land in `target/`. Examples of `.ai` programs live under `examples/`.

## 3. Getting Started
1. **Prerequisites**: Rust 1.75+, Cargo, (optional) Python 3.10 with `qiskit` + `qiskit-aer` for the bridge, Node.js if you plan to run the JS emitter, and feature-gated AI provider credentials.
2. **Clone**: `git clone https://github.com/DarthMetaCrypro/Aeonmi.git`
3. **Build core Shard**: `cargo build --release`
4. **Run CLI help**: `cargo run -- --help`
5. **Launch Mother AI**: `cargo run --bin MotherAI`
6. **Enable extras**:
   - Quantum simulation: `cargo build --features quantum`
   - Qiskit bridge: `cargo build --features qiskit`
   - Mother + full suite: `cargo build --release --no-default-features --features full-suite`

> Windows shortcut: `powershell -ExecutionPolicy Bypass -File .\build_windows.ps1 -Features "full-suite"`

## 4. Language Overview (Aeonmi Surface Syntax)
Aeonmi today ships a slim, production-ready surface language that compiles to the native VM or JavaScript. It favors deterministic control flow, explicit mutation, and simple diagnostics while staging richer quantum glyph syntax through QUBE.

```ai
log("Aeonmi ready");
let stamina = 5;
if (stamina > 3) {
    log("push harder");
} else {
    log("recover");
}

let i = 0;
while (i < 3) {
    log("lap " + i);
    i = i + 1;
}
```

**Available constructs**
- `let` declarations with block scoping and reassignment
- Control flow: `if`, `while`, and (via newer branches) `for`
- Arithmetic `+ - * /`, comparisons, logical operators, string concatenation
- Built-ins: `log`, `print`, `rand`, `time_ms`, `len`
- Optional function syntax (`fn`) depending on feature gates

See `docs/Aeonmi_Language_Guide.md` for exhaustive tokens, troubleshooting, and roadmap items such as arrays, modulo, and structured types.

## 5. QUBE & Quantum Glyphs
QUBE is Aeonmi’s hieroglyphic layer that introduces Unicode-driven quantum constructs, tensor expressions, and probability-aware control flow. The syntax is staged in `docs/AEONMI_UNIQUE_SYNTAX.md` and gradually wired into the parser.

Examples of forthcoming glyphs:
- `𓀀⟨q⟩` – Hadamard (superpose)
- `𓀁⟨q1, q2⟩` – CNOT entanglement
- `𓀄⟨q⟩ → ⟨m⟩` – Measurement binding result into `m`
- `⊖ ⟨condition⟩ ≈ 0.8 ⇒ { ... } ⊕ { ... }` – Probability-weighted branching

Current builds treat these glyphs as forward-compatible tokens. When the `quantum` feature is enabled, Titan provides the backend representations so the CLI can lint glyph usage, export JSON, or run simulations through `titan::gates` and `titan::ops` modules.

## 6. Titan Computational Engine
Titan underpins Aeonmi with a dense collection of numerical, quantum, and symbolic math utilities (all in `src/core/titan/`). Key areas:
- **Linear/advanced algebra**: matrix factorization, tensor calculus, lattice math
- **Quantum toolkits**: `quantum_gates`, `quantum_superposition`, `quantum_tensor_ops`, gate synthesis helpers
- **Simulation physics**: chaos systems, differential equations, fractals, stochastic processes
- **Security & crypto**: `algorithmic_crypto`, `merkle`, `qkd`, hybrid KEM/Sphincs wrappers
- **Vault integration**: `quantum_vault` exposes `simulate_resilience`, `execute_qube_policy`, and crypto helpers consumed by the CLI vault commands
- **External bridges**: `qiskit_bridge` (feature `qiskit`) marshals Titan matrices through PyO3 into Python’s Qiskit for Aer simulations

Titan modules are `pub` but many are intentionally unused until shard features wire them in; `#![allow(dead_code)]` keeps builds clean while integration continues.

## 7. Domain Quantum Vault (DQV)
The Domain Quantum Vault secures registrar secrets, DNSSEC material, and policy automation.

- Hybrid crypto: AES-256-GCM + Kyber1024 KEM + Sphincs+ signatures (`src/encryption.rs`)
- Vault orchestrator: `DomainQuantumVault` (`src/vault.rs`) handles encrypted persistence, audit logs, Merkle exports, and recovery flows
- CLI: `aeonmi vault ...` (commands in `src/cli_vault.rs` and `src/commands/vault.rs`) provides subcommands such as `register`, `fortify`, `watch`, `qube-run`, `export-merkle`
- Titan assists with resilience scoring and symbolic policies (`src/core/titan/quantum_vault.rs`)
- State lives in `~/.aeonmi/vault/domain_quantum_vault.json`; backups and Merkle snapshots enable decentralized mirrors

Reference `docs/domain_quantum_vault.md` for full architecture, command recipes, and roadmap.

## 8. Mother AI Orchestrator
`mother_ai/main.rs` replaces the earlier pseudo-code shard with an async Rust binary that:
- Parses command-line flags (`--voice`, `--holographic`, `--debug`, `--quantum-backend`, `--personality`)
- Boots the Mother AI stack: decision engine, memory subsystem, personality matrix, system coordinator
- Verifies subsystem health and spawns a periodic health monitor (tokio task)
- Opens an interactive REPL (`mother-ai>` prompt) with helper commands (`help`, `status`, `capabilities`, `config`, `goodbye`)
- Handles graceful shutdown via `ctrlc`

The launcher is designed as the human front door into the shard: future builds will stream Titan metrics, vault alerts, and AI-assistant insights directly into this REPL.

## 9. AI Provider Registry
Located in `src/ai/mod.rs`, the registry exposes a trait-based abstraction so Mother AI, the CLI, or GUI can route chat requests to any provider compiled into the build. Enable providers via Cargo features:

| Feature flag | Module | Notes |
|--------------|--------|-------|
| `ai-openai` | `src/ai/openai.rs` | Uses OpenAI Chat Completion API |
| `ai-copilot` | `src/ai/copilot.rs` | GitHub Copilot Chat bridge |
| `ai-perplexity` | `src/ai/perplexity.rs` | Perplexity API integration |
| `ai-deepseek` | `src/ai/deepseek.rs` | DeepSeek research models |

`AiRegistry::list()` returns active providers; `AiRegistry::get(name)` fetches a provider by ID. Provider implementations typically read secrets from Aeonmi’s encrypted API key store (`src/core/api_keys.rs`).

## 10. Quantum & External Integrations
- **Titan simulators**: With `--features quantum`, the CLI exposes `qsim`, `qgates`, and example runners (`qexample grover`). Titan’s `types`, `gates`, and `ops` modules implement the math.
- **Qiskit bridge**: Enable `--features qiskit` to compile `titan::qiskit_bridge`. Provide a Python env with Qiskit installed (see `env_qiskit.cmd`). Use `titan::qiskit_bridge::run_1q_unitary_shots` to ship Titan matrices to Aer and collect shot counts.
- **QUBE execution**: CLI vault commands can run `.qube` policy scripts through Titan to automate registrar checks.

## 11. CLI & Tooling Highlights
`aeonmi` (the main binary) ships a rich command surface:
- `run`, `native`, `exec`, `metrics-*`, `key-rotate` for core language flows
- `edit --tui` for the Ratatui editor
- `repl` for the shard shell
- `vault ...` for Domain Quantum Vault operations
- `cargo`, `python`, `node` passthrough helpers for project automation

Environment flags such as `AEONMI_BYTECODE` and `AEONMI_NATIVE` tweak execution engines; metrics persist to `aeonmi_metrics.json` for cross-session analytics. See the root `README.md` for exhaustive command documentation.

## 12. Extending the Shard
1. **Add a new AI provider**: implement `AiProvider` in `src/ai/your_provider.rs`, gate it behind a feature, and register inside `AiRegistry::new()`.
2. **Expose new Titan capabilities**: add modules under `src/core/titan/` and wire them through `pub mod` exports; use feature flags if the dependency surface is large.
3. **Introduce glyph syntax**: extend the lexer/parser (`src/core/lexer.rs`, `src/core/parser.rs`), add semantic checks, and map to Titan operations.
4. **Augment the vault**: update `DomainQuantumVault` methods and add corresponding CLI commands; ensure encryption helpers cover new payload types.
5. **Surface features in Mother AI**: adjust `MotherAILauncher::run` to broadcast new subsystem metrics or commands.

## 13. Connecting External AI Systems
- Store API keys with `aeonmi key-store add <provider>` (encrypted via `src/core/api_keys.rs`).
- Implement your provider `chat` method to call external APIs and return `String` outputs; streaming support is optional but recommended.
- Hook custom providers into Mother AI by extending the registry or injecting them before `MotherAILauncher::run` starts conversation loops.

## 14. Resources & Next Steps
- `docs/Aeonmi_Language_Guide.md` – deep dive into syntax and diagnostics
- `docs/domain_quantum_vault.md` – vault architecture, simulations, and roadmap
- `gui/plan.md` – cross-platform IDE plan grounded in Tauri + Monaco
- `examples/` – runnable `.ai` samples (classical, glyph previews, quantum demos)
- `tests/` – regression suite covering bytecode, metrics, CLI, and quantum behaviors

**Suggested exploration path**
1. Run `cargo run -- examples/getting_started.ai --native`
2. Launch `cargo run -- repl` and try vault commands (use mock secrets)
3. Build Mother AI (`cargo run --bin MotherAI`) and explore system status
4. Enable `--features quantum` and run `cargo run -- qexample grover`
5. Wire your AI provider or Titan extension following Section 12

The shard is intentionally modular: you can adopt the full stack or cherry-pick subsystems (vault, Titan, parser) for separate deployments. Contributions should preserve this modularity by adding new feature flags and keeping optional components opt-in.
