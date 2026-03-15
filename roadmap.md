# AEONMI LANGUAGE ROADMAP
### Honest Path to a Full Functional Aeonmi.ai + QUBE Quantum Language
**Built from the Shard. Self-hosted. Sovereign.**
**Last verified: March 2026**

---

## CURRENT STATE — MARCH 2026

### What Works (Verified in Code)
- Rust runtime compiles. Zero build errors.
- Lexer tokenizes all `.ai` syntax: keywords, quantum tokens, Unicode, Greek letters,
  qubit literals `|ψ⟩`, hieroglyphs, genesis glyphs (⧉ ‥ … ↦).
- Parser: full Phase 1 + Phase 1.5 support — `import {}`, `quantum function`,
  `quantum struct/enum/circuit`, `async function`/`await`, `match`, `impl`,
  method calls, f-strings (with runtime interpolation), `for x in collection`
  (proper iteration), type annotations, genesis array literals `⧉…⧉`, binding `↦`.
- VM: variables, functions, closures, if/while/for (C-style + for-in), all operators,
  array/object, method dispatch, string builtins, file I/O, quantum simulator.
- Quantum: joint state-vector simulator, real CNOT (Bell states work), 10+ algorithms,
  hardware stubs, QUBE circuit executor with text-mode diagrams.
- CLI: `run`, `exec`, `repl`, `vault`, `quantum`, `qube run/check`, `mint`,
  `mother`, `key-set/get/list/rotate`, `metrics-*` commands.
- Glyph identity (MGK/UGST/GDF): boot ceremony, OKLCH colors, Hz frequency,
  ANSI terminal render, vault encryption (XChaCha20), Merkle log, anomaly detection.
- Mother AI: embryo loop, emotional core, neural network, quantum attention.
- NFT minting: Solana-compatible metadata JSON, Anchor stub.
- Shard self-hosting: `shard/src/main.ai` runs; Shard reads, tokenizes, parses `.ai`.
- Cyberpunk banner: printed to stderr on startup (`src/banner.rs`).

### What Does NOT Work Yet
- Shard does NOT yet compile `.ai` to QASM/Qiskit (P3-9).
- Solana on-chain minting — stub only (wallet integration required).
- Qiskit/PyO3 bridge — feature-gated, not compiled by default.
- Mother AI does NOT yet write `.ai` scripts autonomously or learn persistently.
- No LLM/Claude integration yet.
- No WASM build target.
- No browser REPL.
- No post-quantum (Dilithium) signing of code artifacts.
- No Shamir secret sharing for MGK recovery.
- Web3 wallet/token/DAO Rust modules not yet implemented.
- Smart-contract verifier not yet implemented.
- HTTP/serve framework not yet implemented.
- NFT marketplace CLI not yet implemented.
- QUBE symbolic optimizer (30-qubit in <1MB) not yet implemented.

---

## ROADMAP — 8 PHASES

### ✅ PHASE 0 — FOUNDATION (COMPLETE)
1. ✅ `mother_ai/src/main.rs` entry point
2. ✅ Canonical execution path: `.ai` → Lexer → Parser → Lowering → IR → VM
3. ✅ `docs/LANGUAGE_SPEC_CURRENT.md`
4. ✅ Native VM as default executor (no Node.js required)
5. ✅ `.ai` as default emit format

---

### ✅ PHASE 1 — LANGUAGE CORE (COMPLETE)
- ✅ Full lexer + parser: all quantum/classical constructs
- ✅ f-string interpolation `f"text {var}"` — evaluates at runtime
- ✅ `for x in collection` — proper iteration over arrays/strings
- ✅ Method calls, closures, destructuring, impl blocks
- ✅ Type annotations (parsed, not enforced)
- ✅ Greek letter identifiers (α, β, ψ, θ)
- ✅ Qubit literals `|0⟩`, `|1⟩`, `|+⟩`, `|ψ⟩`
- ✅ Genesis glyph tokens lexed: `⧉` (ArrayGenesisOpen), `‥` (GenesisSep), `…` (GenesisSpread), `↦` (GenesisBinding)
- ✅ Genesis array literal `⧉1‥2‥3⧉` evaluates to `[1, 2, 3]`
- ✅ Tensor / binding operators `⊗`, `↦` parsed as binary quantum expressions
- ✅ Cyberpunk banner on startup (`src/banner.rs`)

---

### ✅ PHASE 2 — QUANTUM BRIDGE INTEGRATION (COMPLETE)
- ✅ `QuantumNeuralNetwork` — layered QNN
- ✅ `SpecializedAlgorithms` — QAOA/VQE/QSVM/QPE/HHL/QKD/molecular
- ✅ `MotherQuantumCore` + `LanguageEvolutionCore` + `EmotionalCore`
- ✅ `QuantumAttentionMechanism`
- ✅ `Blockchain` — SHA256 chain, transaction ledger
- ✅ `Mint` NFT metadata generator
- ✅ `Glyph` system (MGK, UGST, OKLCH, Hz, ANSI terminal)
- ✅ All VM builtins wired: `qnn_*`, `qaoa/vqe/qsvm/qkd`, `blockchain_*`
- ✅ `aeonmi mint --file <artifact>` → Solana NFT metadata JSON
- ✅ Boot ceremony: unseal → UGST → render glyph

---

### ✅ PHASE 3 — QUBE FORMAT + QUANTUM SIMULATOR (COMPLETE)
- ✅ `docs/QUBE_SPEC.md` — formal grammar
- ✅ QUBE parser (full recursive-descent)
- ✅ QUBE AST + executor against Titan quantum sim
- ✅ Text-mode circuit diagram (`--diagram`)
- ✅ Joint multi-qubit state-vector simulator (JointState/JointSystem)
- ✅ Real CNOT gate in `entangle()` — Bell states work
- ✅ `aeonmi qube run examples/demo.qube` — works
- ❌ **P3-9** Shard compiles `.ai` → QASM/Qiskit (remaining)

---

### ✅ PHASE 4 — SELF-HOSTING SHARD (MOSTLY COMPLETE)
- ✅ File I/O built-ins (`read_file/write_file/append_file/file_exists/read_lines/delete_file`)
- ✅ `shard/src/main.ai` bootstraps — runs via `aeonmi run shard/src/main.ai`
- ✅ `shard/src/main_full.ai` — full compiler pipeline
- ✅ Shard reads + tokenizes + parses `.ai` source
- ✅ `aeonmi run shard/src/main.ai -- examples/hello.ai` → real output
- ❌ **P4-9** Shard compiles non-trivial `.ai` program end-to-end with correct output

---

### 🔶 PHASE 5 — GLYPH IDENTITY HARDENING (25% COMPLETE)
- ✅ MGK generation (Argon2id)
- ✅ UGST derivation (HKDF-SHA3-512, 60-second rotation)
- ✅ Vault encryption (XChaCha20-Poly1305)
- ✅ Merkle log over vault records
- ✅ Anomaly detection with glyph distortion
- ✅ `aeonmi vault init` — creates vault + renders glyph
- ❌ **P5-1** Dilithium (post-quantum) signatures on code artifacts
- ❌ **P5-3** Shamir secret sharing for MGK recovery (N-of-M threshold)
- ❌ **P5-5** Challenge-response for sensitive operations using UGST_t
- ❌ **P5-7** Device attestation stub (TPM path)

---

### 🔴 PHASE 6 — MOTHER AI FULL LOOP (10% COMPLETE)
- ✅ Mother embryo loop (parse → execute → learn cycle wired)
- ✅ `aeonmi mother` — interactive REPL
- ❌ **P6-2** Claude API / LLM integration: `aeonmi ai chat --provider claude`
- ❌ **P6-3** Mother writes `.ai` scripts autonomously → runs → logs
- ❌ **P6-4** Persistent learning: `~/.aeonmi/mother_memory.json`
- ❌ **P6-5** Qiskit bridge functional (`--features qiskit`)
- ❌ **P6-6** WASM build target: `cargo build --target wasm32-unknown-unknown`
- ❌ **P6-7** Browser REPL on `aeonmi.x`

---

### 🔴 PHASE 7 — WEB3 SOVEREIGN BRIDGE (5% COMPLETE)
- ✅ `aeonmi mint --file output.ai` → Solana-compatible metadata JSON (basic done)
- ✅ Anchor Rust instruction stub generation
- ❌ **P7-W1** Web3 wallet module (`src/web3/wallet.rs`) — key-pair derivation, transfers
- ❌ **P7-W2** Web3 token module (`src/web3/token.rs`) — ERC-20-style mint/burn/transfer
- ❌ **P7-W3** Web3 DAO module (`src/web3/dao.rs`) — proposals, voting, quorum
- ❌ **P7-V1** Smart-contract verifier (`src/verifier/`) — `aeonmi verify <contract.ai>`
- ❌ **P7-S1** HTTP/serve framework (`src/web/`) — `aeonmi serve <app.ai>`
- ❌ **P7-M1** NFT marketplace CLI (`src/market/`) — `aeonmi market list/mint/buy`
- ❌ **P7-3** Real on-chain Solana minting (requires wallet — opt-in)
- ❌ **P7-4** `aeonmi.x` domain serves browser REPL as living dApp

---

### 🔴 PHASE 8 — QUBE SYMBOLIC OPTIMIZER (0% COMPLETE)
- ❌ **Q-1** QUBE rewrite engine — candidate rewrites with symbolic density scoring
- ❌ **Q-2** Compression scoring (density metric)
- ❌ **Q-3** Correctness verification after rewrite
- ❌ **Q-6** View-based quantum state manipulation
- ❌ **Q-7** Target: 30-qubit simulation in <1MB symbolic representation

---

## EXECUTION ORDER (NEXT SPRINTS)

### NEXT (1–2 weeks)
1. **P3-9** Shard compiles `.ai` to QASM output — the last Shard milestone
2. **P7-W1..W3** Web3 wallet/token/DAO Rust modules + CLI (`aeonmi wallet|token|dao`)
3. **P5-1** Dilithium signatures on minted artifacts

### SHORT-TERM (1–3 months)
4. **P7-V1** Smart-contract verifier (`aeonmi verify`)
5. **P7-S1** HTTP/serve framework (`aeonmi serve`)
6. **P7-M1** NFT marketplace CLI (`aeonmi market`)
7. **P6-2** Claude API integration (`aeonmi ai chat`)
8. **P6-4** Mother persistent memory

### MEDIUM-TERM (3–6 months)
9. **P5-3** Shamir secret sharing for MGK recovery
10. **P6-3** Mother autonomous script writing
11. **P6-5** Qiskit bridge (`--features qiskit`)
12. **P6-6** WASM build target

### LONG-TERM (6–12 months)
13. **Q-1..Q-7** QUBE symbolic optimizer (30-qubit in <1MB)
14. **P6-7** Browser REPL on `aeonmi.x`
15. **P7-3** Solana on-chain minting
16. **P5-7** Device attestation (TPM)

---

## SUCCESS CRITERIA

| # | Test | Status |
|---|------|--------|
| 1 | `aeonmi exec examples/hello.ai` → prints 42 | ✅ |
| 2 | `aeonmi exec examples/quantum.ai` → qubit result | ✅ |
| 3 | `aeonmi run examples/quantum_glyph.ai` → glyph in terminal | ✅ |
| 4 | `aeonmi qube run examples/demo.qube` → executes circuit | ✅ |
| 5 | `aeonmi vault init` → creates vault + renders glyph | ✅ |
| 6 | `aeonmi mint examples/hello.ai` → NFT metadata JSON | ✅ |
| 7 | `aeonmi run shard/src/main.ai` → Shard bootstraps | ✅ |
| 8 | `for x in [1,2,3] { sum = sum + x }` → iterates correctly | ✅ |
| 9 | `⧉1‥2‥3⧉` → array [1, 2, 3] | ✅ |
| 10 | Shard compiles `.ai` → QASM output | ⏳ P3-9 |
| 11 | `aeonmi wallet create` → key-pair derived | ⏳ P7-W1 |
| 12 | `aeonmi verify contract.ai` → security report | ⏳ P7-V1 |
| 13 | `aeonmi serve app.ai` → HTTP server running | ⏳ P7-S1 |
| 14 | `aeonmi market list` → mintable glyph table | ⏳ P7-M1 |
| 15 | 30-qubit symbolic simulation <1MB | ⏳ Q-7 |

---

## WHAT NOT TO DO

- Do not add holographic/AR/VR features until WASM target is stable.
- Do not build real Solana contract until smart-contract verifier is complete.
- Do not add voice interface until Mother AI writes `.ai` autonomously.
- Do not claim "100% complete" without a passing test.
- Do not add new Titan math modules — connect the 50+ that already exist.
- Do not re-add any Llama/external LLM dependency. Mother AI is quantum-native.

---

*This roadmap is honest. Every checked item has been verified in code. Every pending item maps to a specific file and function change.*
*Phases 0–4 complete. Phase 5 (25%), Phase 6 (10%), Phase 7 (5%). Phase 8 not started.*
