# Technology Stack

This page provides a complete reference for every technology, library, and tool used in the Aeonmi ecosystem, organised by layer.

---

## Language Runtime Layer

| Technology | Version | Role |
|------------|---------|------|
| **Rust** | edition 2021 | Primary implementation language (93.7% of codebase) |
| **Cargo** | (bundled with Rust) | Build system, dependency management, feature flags |
| `clap` | 4.5 | CLI argument parsing (derive API) |
| `colored` | 2.1 | ANSI terminal colouring for the cyberpunk banner and output |
| `anyhow` | 1 | Ergonomic error propagation across all Rust modules |
| `unicode-ident` | 1.0 | Unicode identifier validation in the lexer |
| `unicode-normalization` | 0.1 | NFC normalisation of glyph characters at lex time |
| `zeroize` | 1 | Secure in-memory zeroing of secrets in the Identity Vault |

---

## Quantum Computing Layer

| Technology | Version | Role |
|------------|---------|------|
| `nalgebra` | latest | Dense and sparse matrix arithmetic for state-vector simulation |
| `num-complex` | latest | Complex number type (`Complex<f64>`) for qubit amplitudes |
| **QUBE engine** | in-tree | Domain-specific quantum circuit language and executor |
| **State-vector simulator** | in-tree (`src/core/quantum_simulator.rs`) | Full wavefunction simulation of QUBE circuits |
| **Joint state system** | in-tree (`src/qube/executor.rs`) | Multi-qubit entangled state representation (`JointState`/`JointSystem`) |

Activated via the `quantum` Cargo feature:
```toml
cargo build --no-default-features --features "quantum,mother-ai"
```

---

## AI / Mother AI Layer

| Technology | Version | Role |
|------------|---------|------|
| `tokio` | latest | Async runtime for concurrent AI provider requests |
| `reqwest` | latest | HTTP client for Mother AI provider routing |
| **Mother AI router** | in-tree (`src/mother/`) | Unified interface to multiple AI back-ends |
| OpenAI API | external | GPT model calls (feature: `ai-openai`) |
| Anthropic API | external | Claude model calls (feature: `ai-claude`) |
| DeepSeek API | external | DeepSeek model calls (feature: `ai-deepseek`) |
| Perplexity API | external | Perplexity model calls (feature: `ai-perplexity`) |
| OpenRouter API | external | Multi-model routing (feature: `ai-openrouter`) |
| GitHub Copilot API | external | Copilot model calls (feature: `ai-copilot`) |

Activated via the `mother-ai` Cargo feature.

---

## Cryptography / Vault Layer

| Technology | Version | Role |
|------------|---------|------|
| `zeroize` | 1 | Zero-on-drop for in-memory secret buffers |
| `ring` | optional | AES-GCM encryption for vault data at rest (feature: `quantum-vault`) |
| `rustls` | optional | TLS transport for vault network calls (feature: `quantum-vault`) |

---

## Web3 Layer

| Technology | Role |
|------------|------|
| **Wallet module** (`src/web3/wallet.rs`) | Key-pair generation, account ledger |
| **Token module** (`src/web3/token.rs`) | ERC-20-style fungible token |
| **DAO module** (`src/web3/dao.rs`) | On-chain governance proposals and voting |
| **NFT Marketplace** (`src/market/`) | Genesis Glyph minting and trading |

No external blockchain client library is currently vendored; the modules implement the data model in pure Rust and are designed to plug into any chain via an adapter layer.

---

## GUI / Desktop Layer

| Technology | Version | Role |
|------------|---------|------|
| `three-d` | optional | 3-D scene rendering (feature: `holographic`) |
| `wgpu` | optional | Low-level GPU compute and rendering (feature: `holographic`) |
| `winit` | optional | Cross-platform windowing (feature: `holographic`) |
| **Tauri** | planned | Native desktop packaging using the Quantum IDE HTML shell |
| **HTML5 / CSS3** | — | Quantum IDE layout and styling (2.1 % of codebase) |

---

## Editor / IDE Layer

| Technology | Version | Role |
|------------|---------|------|
| **VS Code Extension** (`vscode-aeonmi/`) | 0.1.0 | Syntax highlighting, snippets, and language config for `.ai` and `.qube` files |
| **Monaco Editor** | CDN / embedded | Code editor engine inside the Quantum IDE web interface |
| **TextMate Grammar** (JSON) | — | Grammar files powering both VS Code and Monaco highlighting |
| **JavaScript** | ES2020 | Monaco integration, WebSocket client, circuit panel (`gui/static/`) |
| **Node.js** | optional (dev) | Dev-mode proxy server (`gui/server.js`) |
| `package.json` (root) | — | Tracks VS Code extension npm metadata |

---

## Build and Automation Layer

| Technology | Role |
|------------|------|
| **Cargo features** | Conditional compilation of optional subsystems |
| `build.rs` | Compile-time code generation and platform detection |
| **PowerShell Core (pwsh)** | Cross-platform build automation and CI scripts |
| `scripts/scan_large_files.ps1` | Enforces a maximum tracked-file size (1 MB default) |
| `scripts/scan_history_large_files.ps1` | Audits full Git history for oversized blobs |
| **GitHub Actions** (`.github/`) | CI/CD pipeline: build, test, lint |
| **Git hooks** (`scripts/git-hooks/`) | Optional pre-push hooks for large-file scanning |

---

## Audio Layer (optional)

| Technology | Version | Role |
|------------|---------|------|
| `rodio` | optional | Audio playback for voice/sound features (feature: `voice`) |

---

## Python Interop (optional)

| Technology | Version | Role |
|------------|---------|------|
| `pyo3` | optional | Rust ↔ Python FFI for Qiskit integration (feature: `qiskit`) |
| `numpy` | optional | NumPy array bridge for quantum state export (feature: `qiskit`) |

---

## Core Subsystems Summary

| Subsystem | Primary Language | Key Crates / Technologies |
|-----------|-----------------|--------------------------|
| Titan Runtime (VM + compiler) | Rust | `anyhow`, `clap`, `unicode-ident` |
| Shard self-hosting compiler | Aeonmi (+ Rust host) | — |
| QUBE quantum layer | Rust | `nalgebra`, `num-complex` |
| Mother AI module | Rust | `tokio`, `reqwest` |
| Glyph runtime | Rust | — |
| Identity Vault | Rust | `zeroize`, `ring`, `rustls` |
| Web3 / NFT marketplace | Rust | — |
| Smart-contract verifier | Rust | — |
| Reactive web framework | Rust | `tokio` |
| VS Code Extension | JavaScript (declarative JSON) | TextMate grammar |
| Quantum IDE frontend | HTML + JavaScript | Monaco Editor |
| Build / CI automation | PowerShell | `git` CLI |

---

## Feature Flag Reference

| Flag | Activates |
|------|-----------|
| `quantum` | QUBE engine, state-vector simulator, `nalgebra`, `num-complex` |
| `mother-ai` | Mother AI router, `tokio`, `reqwest` |
| `titan-libraries` | Titan library extensions (implies `quantum`) |
| `holographic` | 3-D GUI: `three-d`, `wgpu`, `winit` |
| `voice` | Audio: `rodio` |
| `quantum-vault` | Encrypted vault: `ring`, `rustls` |
| `qiskit` | Python/Qiskit bridge: `pyo3`, `numpy` |
| `ai-openai` | OpenAI provider (implies `reqwest`) |
| `ai-claude` | Anthropic Claude provider |
| `ai-deepseek` | DeepSeek provider |
| `ai-perplexity` | Perplexity provider |
| `ai-openrouter` | OpenRouter multi-model provider |
| `ai-copilot` | GitHub Copilot provider |
| `full-suite` | All of the above (default) |
| `minimal` | Bare-minimum build without optional subsystems |

Recommended build for most development work:
```sh
cargo build --no-default-features --features "quantum,mother-ai"
cargo test  --no-default-features --features "quantum,mother-ai"
```

---

*See also: [Language-Composition.md](Language-Composition.md) · [Architecture-by-Language.md](Architecture-by-Language.md)*
