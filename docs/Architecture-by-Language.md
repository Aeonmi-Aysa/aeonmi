# Architecture by Language

This page maps every major component of the Aeonmi codebase to the language it is written in and explains how the components interact with each other.

---

## Rust Components

Rust accounts for 93.7 % of the codebase.  The subsystems below each live under `src/` unless noted otherwise.

### Titan Runtime — `src/core/`

The central execution engine for the Aeonmi language.

| Module | File(s) | Responsibility |
|--------|---------|----------------|
| Lexer | `src/core/lexer.rs` | Tokenises `.ai` source into a flat token stream; emits `TokenKind::HieroglyphicOp` for non-ASCII glyph characters |
| Parser | `src/core/parser.rs` | Recursive-descent parser; builds a typed AST from the token stream |
| AST | `src/core/ast.rs` | Node definitions for all Aeonmi constructs (expressions, statements, glyphs, quantum ops) |
| IR | `src/core/ir.rs` | Intermediate representation between the AST and the bytecode emitter |
| Lowering | `src/core/lowering.rs` | Desugars high-level constructs (short-circuit operators, `for-in`, f-string interpolation) into flat IR |
| Bytecode compiler | `src/core/` | Emits a compact bytecode stream from the IR |
| VM | `src/core/vm.rs` | Stack-based bytecode interpreter; dispatches all opcodes including glyph algebra and quantum builtins |
| Code generator | `src/core/code_generator.rs` | Optional emit path for JavaScript back-end (transpilation target) |
| Circuit builder | `src/core/circuit_builder.rs` | Constructs quantum circuits from Aeonmi expressions |
| Circuit visualisation | `src/core/circuit_visualization.rs` | Renders circuits to ASCII or structured output |
| Quantum simulator | `src/core/quantum_simulator.rs` | State-vector simulation of quantum circuits using `nalgebra` / `num-complex` |

### Shard Self-Hosting Compiler — `shard/`

Shard is the Aeonmi-in-Aeonmi bootstrap path.  The goal is for the language to eventually compile itself without a Rust host.

| File | Responsibility |
|------|----------------|
| `shard/src/main.ai` | Top-level Shard entry point (written in Aeonmi) |
| `shard/editor/ai_canvas.ai` | AI-assisted code canvas — interactive in-language editing surface |

The Rust host exposes a stable ABI that Shard uses until the self-hosting gap closes completely.

### QUBE Quantum Layer — `src/qube/`

QUBE (Quantum Unified Bytecode Engine) is a domain-specific sub-language for writing and executing quantum circuits inline with Aeonmi programs.

| Module | Responsibility |
|--------|----------------|
| `src/qube/lexer.rs` | Tokenises `.qube` source and QUBE blocks inside `.ai` files |
| `src/qube/ast.rs` | AST nodes for `circuit`, `meta`, `execute`, and `expected` blocks |
| `src/qube/parser.rs` | Parses the circuit-block syntax into the QUBE AST |
| `src/qube/executor.rs` | Executes QUBE circuits: builds joint state vectors (`JointState`/`JointSystem`), applies real CNOT and single-qubit gates |

QUBE circuits can be embedded directly inside `.ai` programs and their output fed back into Aeonmi expressions.

### Mother AI Module — `src/mother/`, `mother_ai/`

The Mother AI layer provides runtime AI-routing, intent classification, and context memory.

| Component | Responsibility |
|-----------|----------------|
| `mother_ai/main.rs` | Standalone `MotherAI` binary entry point |
| `src/mother/` | Library integration: async HTTP calls (`tokio` + `reqwest`) to external AI providers |
| Multi-provider support | Routes requests to OpenAI, Anthropic, DeepSeek, Perplexity, OpenRouter, or GitHub Copilot based on configuration |

### Glyph Runtime — `src/glyph/`

Implements the five genesis glyph operators:

| Glyph | Token | Semantics |
|-------|-------|-----------|
| `⧉` | `ArrayGenesisBracket` | Symbolic array construction |
| `‥` | separator | Element separator inside genesis arrays |
| `…` | `Spread` | Spread / expand a symbolic structure |
| `↦` | `Binding` | Bind or project a value |
| `⊗` | `TensorProduct` | Symbolic tensor product (keeps structures lazy) |

### Identity Vault — `src/vault.rs`, `src/encryption.rs`

Secure key–value store backed by `zeroize` for in-memory secret hygiene.  The optional `quantum-vault` Cargo feature enables `ring`-backed AES-GCM encryption and `rustls` for TLS.

### Web3 Layer — `src/web3/`

Three modules providing blockchain primitives:

| Module | CLI command | Responsibility |
|--------|-------------|----------------|
| `src/web3/wallet.rs` | `aeonmi wallet` | Key-pair generation, ledger-style account book |
| `src/web3/token.rs` | `aeonmi token` | ERC-20-style fungible token model |
| `src/web3/dao.rs` | `aeonmi dao` | On-chain governance proposal and voting stubs |

### Smart-Contract Verifier — `src/verifier/`

Static safety analyser for Aeonmi smart contracts.  Invoked with `aeonmi verify <file>`.  Checks for unsafe state mutations, un-guarded external calls, and type mismatches.

### Reactive Web Framework — `src/web/`

Minimal HTTP server embedded in the Aeonmi binary.  `aeonmi serve` starts it and serves the Quantum IDE HTML/JS assets over localhost.  The VM exposes HTTP builtins (`http_response`, `http_get`, `http_post`, `http_json`) so `.ai` programs can handle web requests natively.

### Genesis Glyph NFT Marketplace — `src/market/`

CLI-accessible marketplace for on-chain glyph tokens:

```
aeonmi market list      # list glyphs available for purchase
aeonmi market info <id> # inspect a specific glyph
aeonmi market mint      # mint a new glyph NFT
aeonmi market glyphs    # show owned glyphs
```

### TUI — `src/tui/`

Terminal User Interface shell written with Rust TUI primitives.  Provides an interactive REPL-like interface without a full GUI stack.

### CLI — `src/cli.rs`, `src/main.rs`

Unified `Aeonmi` / `aeonmi` entry point that dispatches to every sub-command.  Built with `clap` (derive API).

### Build Script — `build.rs`

Cargo build script that runs at compile time to generate or verify platform-specific bindings before the main crate compiles.

### Cyberpunk Banner — `src/banner.rs`

Startup banner printed in ANSI colours when the CLI launches interactively.

---

## JavaScript Components

JavaScript accounts for 4.0 % of the codebase and covers two areas.

### VS Code Extension — `vscode-aeonmi/`

| File | Responsibility |
|------|----------------|
| `syntaxes/aeonmi.tmLanguage.json` | TextMate grammar for `.ai` / `.aeon` / `.aeonmi` files |
| `syntaxes/qube.tmLanguage.json` | TextMate grammar for `.qube` files |
| `snippets/aeonmi.json` | IntelliSense completions for Aeonmi keywords and glyph operators |
| `snippets/qube.json` | IntelliSense completions for QUBE circuit syntax |
| `package.json` | Extension manifest: contributes `aeonmi` and `qube` language IDs, wires grammars and snippets |

The extension is a pure declarative contribution (no TypeScript activation code) so it imposes no runtime dependency on Node.js after installation.

### Quantum IDE Frontend — `gui/static/`

| File | Responsibility |
|------|----------------|
| `gui/static/main.js` | Initialises the Monaco editor instance, registers the Aeonmi language, manages the circuit panel, and opens the WebSocket connection to the Rust `serve` back-end |
| `gui/static/monaco_ai_language.js` | Registers Aeonmi language tokens, folding rules, and bracket pairs inside Monaco's language registry |
| `gui/server.js` | Node.js companion server (optional dev mode) — proxies WebSocket messages between the browser and the `aeonmi serve` Rust process |

---

## HTML Components

HTML accounts for 2.1 % of the codebase.

### Quantum IDE — `gui/quantum_ide.html`

Full-screen, single-page IDE layout:

- CSS grid with four regions: header toolbar, sidebar file explorer, Monaco editor, quantum circuit visualisation panel
- Embedded `<style>` block using the Aeonmi cyberpunk palette (deep navy + lavender on dark)
- Progressive enhancement — functional without JavaScript for static viewing; full interactive mode requires `main.js`

### Web Server Static Assets — `gui/static/index.html`

Landing page served by `aeonmi serve` at the root URL.  Provides links to the Quantum IDE and a brief project overview.

### Tauri Webview Assets — `gui/tauri_bridge/`, `gui/tauri_tauri/`

HTML files used as Tauri webview frontends when the `holographic` Cargo feature is enabled.  These allow the same IDE to be packaged as a native desktop application on Windows, macOS, and Linux.

---

## PowerShell Components

PowerShell accounts for 0.2 % of the codebase.

### Large-File Guard — `scripts/scan_large_files.ps1`

Iterates every Git-tracked path and measures file size on disk.  Exits non-zero if any file exceeds the threshold (default: 1 MB).  Intended as a pre-push hook or CI step.

Parameters:
- `-ThresholdMB <int>` — size limit in megabytes (default: 1)
- `-Quiet` — suppress informational output

### History Scanner — `scripts/scan_history_large_files.ps1`

Walks the full Git object database using `git rev-list --objects --all` to surface historical blobs that already exceeded the threshold.  Useful when adopting Git LFS retroactively or auditing repository size growth.

---

## Cross-Language Integration Points

| Boundary | Mechanism |
|----------|-----------|
| Rust ↔ JavaScript (IDE) | HTTP REST + WebSocket (`aeonmi serve` exposes endpoints; `main.js` consumes them) |
| Rust ↔ HTML (Tauri) | Tauri IPC bridge — Rust commands registered with `#[tauri::command]` invoked from the HTML webview |
| Rust ↔ VS Code | Extension is pure declarative JSON; VS Code spawns the `aeonmi` binary as a language server subprocess if LSP mode is enabled |
| Rust ↔ PowerShell | PowerShell scripts call `git` CLI; they do not link against any Rust library |

---

*See also: [Language-Composition.md](Language-Composition.md) · [Technology-Stack.md](Technology-Stack.md)*
