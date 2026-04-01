# Language Composition

This page documents the language composition of the Aeonmi repository and explains the strategic role of each language in the project.

## Breakdown

| Language   | Share  | Role |
|------------|--------|------|
| Rust       | 93.7%  | Core runtime, compiler, VM, and all major subsystems |
| JavaScript | 4.0%   | VS Code extension, Monaco-based Quantum IDE frontend |
| HTML       | 2.1%   | Quantum IDE GUI layout and web interface assets |
| PowerShell | 0.2%   | Repository automation, build scripts, large-file scanning |

---

## Rust — 93.7%

Rust is the primary implementation language of the entire Aeonmi ecosystem.  It was chosen for three reasons:

1. **Performance** — Zero-cost abstractions and no garbage-collector pauses are essential for a bytecode VM and quantum circuit executor that must evaluate large symbolic structures quickly.
2. **Memory safety** — The borrow checker eliminates whole classes of runtime errors in the cryptographic vault, the glyph algebra engine, and the concurrent Mother AI module.
3. **Ecosystem** — `nalgebra` and `num-complex` for linear algebra / complex-number arithmetic are mature, well-maintained crates that map directly onto QUBE quantum-state representation.

All of the following subsystems are written entirely in Rust:

- **Titan Runtime** (`src/core/`) — lexer, parser, AST, IR, bytecode compiler, VM, lowering passes
- **Shard self-hosting compiler** (`shard/`) — the Aeonmi-in-Aeonmi compiler path
- **QUBE quantum layer** (`src/qube/`) — quantum-circuit lexer, parser, executor, state-vector simulator
- **Mother AI module** (`src/mother/`, `mother_ai/`) — async AI-routing, intent classification, context memory
- **Glyph runtime** (`src/glyph/`) — symbolic glyph algebra: `⧉ ‥ … ↦ ⊗`
- **Identity Vault** (`src/vault.rs`, `src/encryption.rs`) — zeroize-backed secret storage, optional ring/rustls
- **Web3 layer** (`src/web3/`) — wallet key pairs, ERC-20 token model, DAO governance stubs
- **Smart-contract verifier** (`src/verifier/`) — static safety analysis on `.ai` contracts
- **Reactive web framework** (`src/web/`) — minimal HTTP server for serve-mode
- **Genesis Glyph NFT marketplace** (`src/market/`) — list, mint, and query on-chain glyph tokens
- **CLI** (`src/cli.rs`, `src/main.rs`) — unified `Aeonmi` / `aeonmi` entry points for all sub-commands
- **Build glue** (`build.rs`) — Cargo build script for compile-time code generation

---

## JavaScript — 4.0%

JavaScript appears in two distinct areas:

### VS Code Extension (`vscode-aeonmi/`)
The extension provides first-class editor support for `.ai`, `.aeon`, `.aeonmi`, and `.qube` files inside Visual Studio Code.  It contributes:
- TextMate grammar for syntax highlighting (`syntaxes/aeonmi.tmLanguage.json`, `syntaxes/qube.tmLanguage.json`)
- IntelliSense snippet completions (`snippets/aeonmi.json`, `snippets/qube.json`)
- Language configuration (bracket matching, comment toggling) via `language-configuration.json`

The `package.json` at the repo root also tracks the VS Code extension's npm dependencies.

### Quantum IDE frontend (`gui/static/main.js`, `gui/static/monaco_ai_language.js`)
The web-based Quantum IDE embeds the Monaco editor (the same engine as VS Code) and registers the Aeonmi language grammar inside it.  `main.js` wires together the editor pane, the circuit visualisation panel, and the WebSocket bridge to the Rust back-end (`gui/server.js`).

---

## HTML — 2.1%

HTML provides the structural shell for all GUI and web-facing components:

- **`gui/quantum_ide.html`** — The main Quantum IDE layout: header toolbar, sidebar file tree, Monaco editor region, quantum circuit visualisation panel, and integrated terminal.
- **`gui/static/index.html`** — Static landing / splash page served by the Aeonmi web server in `serve` mode.
- **`gui/tauri_bridge/`** and **`gui/tauri_tauri/`** — Tauri integration stubs.  When Tauri support is enabled the HTML files become the Tauri webview frontend, giving native desktop packaging to the same IDE.

HTML templates are intentionally minimal — styling is embedded via `<style>` blocks and dynamic behaviour is handled by the accompanying JavaScript files.

---

## PowerShell — 0.2%

Two PowerShell scripts in `scripts/` automate repository hygiene tasks that would be tedious to perform manually:

| Script | Purpose |
|--------|---------|
| `scripts/scan_large_files.ps1` | Scans every Git-tracked file and fails if any exceeds a configurable size threshold (default 1 MB).  Used in CI to prevent binary blobs from bloating the repository. |
| `scripts/scan_history_large_files.ps1` | Walks the entire Git object history (`git rev-list --objects --all`) and reports historical blobs that exceeded the threshold, helping clean up pre-existing large objects. |

Both scripts are cross-platform via PowerShell Core (pwsh) and integrate naturally into Windows-based CI pipelines or developer machines.

---

## Correlation with Project Architecture

The language split mirrors the layered architecture documented in [README.md](../README.md#architecture):

```
Aeonmi Language (.ai)
        │
        ▼
     Shard                         ← Rust
self-hosting compiler
        │
        ▼
  Titan Runtime (Rust)             ← Rust
        │
 ┌──────┴──────┐
 ▼             ▼
Glyph        QUBE Engine           ← Rust
Runtime      quantum circuits
 │
 ▼
Identity Vault                     ← Rust

── GUI / IDE layer ──
VS Code Extension                  ← JavaScript
Quantum IDE HTML shell             ← HTML
Quantum IDE JS frontend            ← JavaScript

── Tooling layer ──
Build / CI scripts                 ← PowerShell
```

The 93.7% Rust share reflects the fact that every execution-critical path — parsing, compilation, optimisation, VM dispatch, quantum simulation, cryptography — is written in Rust.  The 4% JavaScript and 2.1% HTML collectively cover the editor and web GUI surfaces that are inherently browser-native technologies.  The 0.2% PowerShell handles the one platform-specific automation task (large-file CI guard) that a shell script would also serve on Linux.

---

## Performance Implications

- **No GC pauses** — The Titan VM can sustain tight execution loops (benchmark target: < 1 microsecond per bytecode dispatch) because Rust allocates and frees deterministically.
- **SIMD-friendly data layout** — `nalgebra`'s column-major matrix types map onto CPU vector units for state-vector operations in the QUBE engine.
- **Compile-time feature flags** — Optional subsystems (`holographic`, `voice`, `qiskit`) are gated behind Cargo features so release builds for constrained environments carry zero overhead from unused modules.
- **JS sandboxed** — The JavaScript layer never touches the critical execution path; it communicates with the Rust back-end over HTTP/WebSocket, so a slow Monaco render cycle cannot stall the VM.

---

*See also: [Architecture-by-Language.md](Architecture-by-Language.md) · [Technology-Stack.md](Technology-Stack.md)*
