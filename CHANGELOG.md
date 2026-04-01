# Changelog

All notable changes to the Aeonmi project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Aeonmi uses **date-based versioning** aligned with development phases.

---

## [Unreleased]

### Added
- GitHub Actions CI workflow (`ci.yml`) — automated build and test on Ubuntu, macOS, and Windows for every push and pull request.
- `CHANGELOG.md` — this file.
- `examples/showcase.ai` — single-file tour of all major Aeonmi language features.
- `examples/cs/fibonacci.ai` — recursive and iterative Fibonacci implementations.

---

## [1.0.0-quantum-consciousness] — 2026-03-14

### Phase 5 — Web3, Verifier, Reactive Web, NFT Marketplace
#### Added
- Smart-Contract Verifier (`src/verifier/`) — static analysis of `.ai` contract source, CLI: `aeonmi verify`.
- Reactive Web Framework (`src/web/`) — HTTP server built on Aeonmi, CLI: `aeonmi serve`, VM builtins: `http_response`, `http_get`, `http_post`, `http_json`.
- Genesis Glyph NFT Marketplace (`src/market/`) — CLI: `aeonmi market list/info/mint/glyphs`.
- Web3 wallet, token (ERC-20), and DAO governance modules (`src/web3/`).
- CLI: `aeonmi wallet`, `aeonmi token`, `aeonmi dao`.
- Documentation: `docs/WEB3_GUIDE.md`.

### Phase 4 — Cyberpunk CLI, Genesis Glyphs, For-In Loop
#### Added
- Cyberpunk ASCII startup banner (`src/banner.rs`) with neon-yellow / magenta color scheme.
- F-string interpolation — `f"hello {name}"` fully evaluated in the native VM.
- `for x in collection` iteration (P1-34) — AST node `ForIn`, IR lowering, VM execution.
- Genesis glyph tokens: `⧉` (Array Genesis), `‥` (separator), `…` (spread), `↦` (binding) — lexer + parser + VM.
- Genesis glyph literals G-1..G-12 in `examples/genesis.ai`.

### Phase 3 — File I/O, Shard Self-Hosting, QUBE Circuit Blocks
#### Added
- File I/O builtins: `read_file`, `write_file`, `append_file`, `file_exists`, `read_lines`, `delete_file`.
- Shard self-hosting compiler — reads and compiles `.ai` source files natively.
- QUBE circuit-block syntax: `circuit / meta / execute / expected` sections.
- AI Canvas editor at `shard/editor/ai_canvas.ai`.

### Phase 2 — Quantum Simulator, Joint State-Vector
#### Added
- Real joint state-vector quantum simulator (`src/core/quantum_simulator.rs`).
- True CNOT gate implementation (P2-9).
- `JointState` / `JointSystem` data types (P2-8).
- QUBE executor — runs `.qube` quantum circuit files with Born-rule measurement.
- QUBE gates: H, X, Y, Z, S, T, CNOT, CZ, SWAP.
- Text-mode circuit diagram rendering (`--diagram`).

### Phase 1 — Core Language, Native VM
#### Added
- Full recursive-descent parser: quantum structs/enums/functions, async/await, match, impl, closures, destructuring, imports, type annotations.
- Native VM as the default execution engine — no Node.js dependency.
- Bytecode compiler and bytecode VM (`src/core/bytecode.rs`).
- Qubit declarations: `qubit q;`, superposition, entanglement, measurement builtins.
- Quantum-native Unicode operators: `⊗ ⊕ ⊖ ∇ ≈ ⪰ ⪯ ⇒ ⟲ ◊ ← ∈`.
- Hieroglyphic operator dispatch: `𓀀(x, y)` → `__glyph` runtime.
- Glyph Identity system: MGK (256-bit, Argon2id), UGST (HKDF-SHA3-512, 60-second rotation), OKLCH color / Hz frequency rendering.
- Encrypted vault: XChaCha20-Poly1305, per-record salts, Merkle log.
- Mother AI embryo loop: emotional core, neural network, quantum attention mechanism.
- NFT minting: Solana-compatible metadata JSON + Anchor Rust stub generation.
- VS Code extension (`vscode-aeonmi/`) with syntax highlighting and code snippets.
- Titan runtime library (`titan_libraries/`) — quantum math, linear algebra, Grover, Shor, QFT, teleportation in `.ai`.

---

## [0.2.0] — 2025-02-01

### Added
- Initial public source release.
- Aeonmi surface language (`.ai`) with lexer, parser, and tree-walking interpreter.
- QUBE (`.qube`) quantum circuit language — lexer, parser, and AST.
- Identity vault prototype.
- VS Code language extension prototype.
- `docs/` directory — architecture, language spec, glyph algebra, QUBE grammar.

---

[Unreleased]: https://github.com/Aeonmi-Aysa/aeonmi/compare/v1.0.0-quantum-consciousness...HEAD
[1.0.0-quantum-consciousness]: https://github.com/Aeonmi-Aysa/aeonmi/compare/v0.2.0...v1.0.0-quantum-consciousness
[0.2.0]: https://github.com/Aeonmi-Aysa/aeonmi/releases/tag/v0.2.0
