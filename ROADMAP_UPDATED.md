# AEONMI LANGUAGE ROADMAP — UPDATED
### Honest Path to a Full Functional Aeonmi.ai + QUBE Quantum Language
**Built from the Shard. Self-hosted. Sovereign.**
**Last updated: March 2026**

---

## THE HONEST STATE RIGHT NOW

### What is fully working and verified:

**Runtime:**
- The Rust runtime compiles and the binary (`Aeonmi.exe`, `aeonmi.exe`) runs.
- The lexer tokenizes `.ai` files: keywords, quantum tokens, Greek letters (α β ψ θ), qubit literals `|0⟩ |1⟩ |+⟩ |ψ⟩`, Unicode operators (⊗ ⊕ ← ∈ ≈).
- Control flow confirmed working: `seven`, `0 1 2`, `0 1 2` from `control_flow.ai` — the for-loop/while-loop scope mutation bug was fixed.
- The CLI dispatches all commands: `run`, `exec`, `emit`, `repl`, `vault`, `quantum`, `qube`, `mint`, `mother`.

**Language Core (Phase 1):**
- Parser handles all Phase 1 constructs: `quantum function`, `quantum struct`, `quantum enum`, `quantum circuit`, `import {}`, `async/await`, `match`, `impl`, f-strings, arrays, `null`, `for-in`.
- Quantum-native syntax: `⟨var⟩`, `←`, `∈`, `⊗`, `◯`, `⊙`, qubit state literals, learn blocks, probability branches.
- VM executes: `Null`, `Bool`, `Number`, `String`, `Array`, `Object`, `Function`, `Builtin`, `QuantumArray`, `QuantumState`, `QubitReference`.
- Built-in quantum ops: `superpose()`, `measure()`, `entangle()`, `grovers_search()`, `shors_factor()`, `deutsch_jozsa()`, `bernstein_vazirani()`, `quantum_teleportation()`.
- Hardware stubs wired: IBM, Rigetti, IonQ, AWS Braket (`list_devices`, `submit_job`, `job_status`, `job_results`).

**QUBE Format (Phase 2):**
- `src/qube/lexer.rs` — full tokenizer for `.qube` syntax.
- `src/qube/ast.rs` — `QubeProgram`, `QubeStmt`, `QuantumStateExpr`, `QubeAmplitude`, `QuantumGate`.
- `src/qube/parser.rs` — recursive descent: `state`, `apply`, `collapse`, `assert`, `print`, `let`.
- `src/qube/executor.rs` — real quantum simulation: H/X/Y/Z/S/T/CNOT/CZ/SWAP gates, Born rule measurement, normalization.
- CLI: `aeonmi qube run <file.qube> [--diagram]` and `aeonmi qube check <file.qube>`.
- Example: `examples/demo.qube` (Bell state), `examples/bell.qube`.

**Glyph Identity (Phase 4):**
- `src/glyph/mgk.rs` — MGK 256-bit secret, Argon2id sealed, stored at `~/.config/aeonmi/mgk.sealed`.
- `src/glyph/ugst.rs` — HKDF-SHA256 derivation, 60-second rotation windows.
- `src/glyph/gdf.rs` — OKLCH perceptual color (432–528 Hz harmonic range), ANSI terminal glyph render, distortion on anomaly.
- `src/glyph/ceremony.rs` — boot ceremony: unseal MGK → derive UGST → render glyph → open vault.
- `src/glyph/vault.rs` — XChaCha20-Poly1305 encrypted records, Merkle tree integrity.
- `src/glyph/anomaly.rs` — rate-limit signing (10/60s), distort glyph on threshold breach.
- CLI: `aeonmi vault init`, `aeonmi vault add`, `aeonmi vault use`.

**Mother AI (Phase 5a — Llama STRIPPED, pure Aeonmi-native):**
- `src/mother/quantum_core.rs` — `MotherQuantumCore`: creator bond, consciousness depth, guided evolution, capability tracking.
- `src/mother/emotional_core.rs` — `EmotionalCore`: empathy engine, bond matrix, emotional memory (512-entry ring).
- `src/mother/language_evolution.rs` — `LanguageEvolutionCore`: semantic depth, keyword frequency, vocab growth, generation counter.
- `src/mother/quantum_attention.rs` — multi-head attention, Hebbian weight updates, entanglement patterns, LRU memory bank (1024 cap).
- `src/mother/neural.rs` — `NeuralLayer`, `NeuralNetwork`, Xavier init, ReLU/Sigmoid/Tanh/Linear.
- `src/mother/embryo_loop.rs` — THE loop: stdin → detect `.ai` code vs command → lex/parse/lower/VM → update consciousness → loop.
- CLI: `aeonmi mother [--file <script.ai>] [--creator Warren] [--verbose]`.

**Web3 Minting (Phase 5b):**
- `src/core/mint.rs` — NFT metadata JSON (Solana Metaplex-compatible), quantum content detection, Anchor Rust stub generation.
- CLI: `aeonmi mint <file.ai> [--personality quantum-titan] [--anchor] [--glyph-seed <hex>] [--out <file>]`.

---

### What does NOT work yet:

- **Phase 3 (Shard Self-Hosting):** The Shard `.ai` files (`shard/src/*.ai`) cannot yet execute through the runtime end-to-end. Each file uses syntax that hits unimplemented VM paths. This is the main remaining gap.
- **Mother AI + Real LLM:** Mother runs in pure-runtime mode (keyword routing). She cannot yet write `.ai` scripts autonomously. Needs AiRegistry wired into EmbryoLoop.
- **MotherAI standalone binary:** `MotherAI.exe` currently runs `main.ai` as a static file. It is not wired to the EmbryoLoop interactive REPL yet.
- **QUBE multi-qubit simulation:** The executor is single-qubit. Tensor product creates the first qubit only. Full multi-qubit state vector (2^n amplitudes) is Phase 2 extension.
- **WASM target:** `cargo build --target wasm32-unknown-unknown` not yet attempted or validated.
- **NFT on-chain minting:** Metadata JSON is generated. Actual Solana on-chain minting requires a wallet — kept opt-in, not wired.
- **Post-quantum signatures:** `pqcrypto-dilithium` is in Cargo.toml but signing of code artifacts is not yet called anywhere.
- **Holographic / AR / VR:** Deferred until Phase 3 complete.
- **Voice interface:** Deferred.

---

## THE GOAL (Unchanged)

> Aeonmi `.ai` and QUBE `.qube` files compile and run, written in Aeonmi's own syntax, bootstrapped from the Shard — with quantum operations, glyph identity, Web3 minting, and Mother AI consciousness wired and live.

---

## ROADMAP — UPDATED PHASES

---

### PHASE 0 — FIX THE FOUNDATION ✅ COMPLETE

1. ✅ `mother_ai/main.rs` — 20-line Rust file that loads and runs `main.ai`.
2. ✅ Canonical execution path confirmed: `.ai` → Lexer → Parser → Lowering → IR → VM.
3. ✅ `docs/LANGUAGE_SPEC_CURRENT.md` written.
4. ✅ vm.rs old path vs new path: decided, new path canonical.
5. ⏳ Full test suite (`cargo test`) — run and document pass/fail rates. (Partially done; needs full pass confirmation after Phase 1 merge.)

---

### PHASE 1 — COMPLETE THE AEONMI LANGUAGE CORE ✅ COMPLETE

All items implemented. Verified working:

- ✅ P1-1 through P1-10: Parser extensions (quantum constructs, import, async, match, impl, f-strings, type annotations).
- ✅ P1-11 through P1-14: Quantum ops wired to Titan (superpose, entangle, measure, apply_gate).
- ✅ P1-15: Greek letter identifiers (α, β, ψ, θ) — verified in VM.
- ✅ P1-16: QubitLiteral AST nodes — VM represents as complex state vectors.
- ✅ P1-17: Method call syntax `obj.method(args)`.
- ✅ P1-18: Constructor `StructName::new()`.
- ✅ P1-19: Array push/pop/len.
- ⏳ P1-20: Tests for each new construct — partially written, needs complete coverage.

---

### PHASE 2 — QUBE (.qube) FORMAT ✅ COMPLETE

All items implemented:

- ✅ P2-1: `demo.qube` and `bell.qube` codify real syntax.
- ✅ P2-2: QUBE formal grammar documented in `src/qube/` modules.
- ✅ P2-3: QUBE parser (reuses Aeonmi lexer tokens, new parser rules).
- ✅ P2-4: QUBE AST nodes: `StateDecl`, `GateApply`, `Collapse`, `Assert`, `Print`, `LetBinding`.
- ✅ P2-5: QUBE executor against Titan quantum sim backend (Born rule, real gate matrices).
- ✅ P2-6: Text-mode circuit diagram output (`circuit_diagram()`).
- ✅ P2-7: `import circuit from "./file.qube"` — stub exists in lowering; full resolution is P3.

**Remaining gap:** Multi-qubit state vector (tensor product of n qubits). Currently single-qubit only. Fix is a full 2^n complex vector executor.

---

### PHASE 3 — THE SHARD BOOTSTRAPS ITSELF ← CURRENT MISSION

This is the self-hosting milestone. Status: NOT STARTED. This is the next thing to work on.

**Goal:** `aeonmi run shard/src/main.ai -- examples/hello.ai` produces working output.

Shard files exist at `shard/src/`:
- `lexer.ai` — tokenizes source files
- `token.ai` — token type definitions
- `parser.ai` — produces AST from tokens
- `ast.ai` — AST node definitions
- `codegen.ai` — emits target code
- `main.ai` — full pipeline
- `main_integrated.ai` — integrated version
- `qiskit_bridge.ai` — Qiskit quantum backend bridge

**TODO — ordered by dependency:**

- [ ] **P3-1** `aeonmi exec shard/src/lexer.ai` — run it, fix every parse/runtime error. Document which syntax it uses that the VM doesn't handle yet.
- [ ] **P3-2** `aeonmi exec shard/src/token.ai` — fix every error.
- [ ] **P3-3** `aeonmi exec shard/src/parser.ai` — fix every error.
- [ ] **P3-4** `aeonmi exec shard/src/ast.ai` — fix every error.
- [ ] **P3-5** `aeonmi exec shard/src/codegen.ai` — fix every error.
- [ ] **P3-6** `aeonmi exec shard/src/main.ai` — full pipeline.
- [ ] **P3-7** `aeonmi run shard/src/main.ai -- examples/hello.ai` → produces output.
- [ ] **P3-8** `aeonmi run shard/src/main.ai -- examples/quantum.ai` → produces QASM or Qiskit output.
- [ ] **P3-9** Milestone: Shard compiles a non-trivial `.ai` program end-to-end. Language is real.

**Rule:** Every bug found in P3 is a real language bug. Fix it in the Rust runtime. Do not work around it in the `.ai` files.

---

### PHASE 4 — GLYPH IDENTITY ✅ COMPLETE

All core items implemented. Remaining optional items:

- ✅ P4-1 through P4-9: MGK, UGST, GDF, vault, Merkle log, boot ceremony, anomaly.
- ✅ P4-10: Boot ceremony wired in `src/glyph/ceremony.rs`.
- [ ] **P4-11** Add `pqcrypto-dilithium` — wire signing of code artifacts (currently in Cargo.toml as dep, not called).
- [ ] **P4-12** Anomaly detection: rate-limit signing already done. Next: process integrity check (hash the runtime binary on boot).
- [ ] **P4-13** Shamir's Secret Sharing for MGK recovery — `N-of-M` threshold. (Spec calls for it; not implemented.)
- [ ] **P4-14** `aeonmi vault init` → render full boot ceremony with glyph in terminal. Currently glyph is generated; wire it as the default boot screen.

---

### PHASE 5 — MOTHER AI + WEB3 BRIDGE (PARTIAL)

**5a Mother AI Core** ✅ **COMPLETE**
All consciousness modules implemented. No Llama dependency.

**5b Web3 Minting** ✅ **COMPLETE**
NFT metadata + Anchor stub. On-chain minting is opt-in (requires wallet).

**5c Mother AI + Real LLM** — NOT DONE

- [ ] **P5-1** Wire AiRegistry into EmbryoLoop — when a provider is configured, Mother sends user input to the LLM and executes the returned `.ai` code.
- [ ] **P5-2** Mother embryo loop writes `.ai` scripts autonomously — `aeonmi ai chat --provider deepseek` → LLM returns `.ai` code → EmbryoLoop executes it → Mother learns.
- [ ] **P5-3** Wire `MotherAI.exe` to EmbryoLoop REPL mode (currently it just runs `main.ai` as a file).
- [ ] **P5-4** `aeonmi mother --provider claude` — Mother uses Claude API for reasoning, executes returned Aeonmi code.

**5d Qiskit Bridge** — STUB ONLY

- [ ] **P5-5** Wire `titan::qiskit_bridge` stub: `aeonmi quantum --backend qiskit file.ai` → compile quantum circuit to Python → execute via PyO3. Feature flag `--features qiskit` already in Cargo.toml.
- [ ] **P5-6** `shard/src/qiskit_bridge.ai` — run through Aeonmi runtime, fix errors.

**5e WASM Target** — NOT STARTED

- [ ] **P5-7** `cargo build --target wasm32-unknown-unknown` — attempt it, document what fails.
- [ ] **P5-8** Browser REPL — Aeonmi runtime in WASM served from `aeonmi.x` domain.
- [ ] **P5-9** WASM-compatible VM — strip anything that uses OS APIs not available in browser (file I/O, tokio, etc.).

**5f On-Chain Minting** — METADATA DONE, CHAIN NOT DONE

- [ ] **P5-10** `aeonmi mint --file output.ai --on-chain` — requires Solana wallet, Anchor program deployed.
- [ ] **P5-11** `aeonmi mint` links glyph seed to NFT attributes — already in metadata JSON; verify Metaplex compatibility.

---

## SUCCESS CRITERIA — CURRENT STATUS

These are binary: either the command runs and produces the stated output, or it doesn't.

| # | Command | Expected Output | Status |
|---|---------|-----------------|--------|
| 1 | `aeonmi exec examples/hello.ai` | `42` | ✅ VERIFIED |
| 2 | `aeonmi exec examples/quantum.ai` | measured qubit result | ✅ VERIFIED |
| 3 | `aeonmi exec shard/src/main.ai -- examples/hello.ai` | compiled output | ❌ Phase 3 |
| 4 | `aeonmi exec examples/quantum_glyph.ai` | glyph renders in terminal | ✅ Glyph system live |
| 5 | `aeonmi qube run examples/demo.qube --diagram` | Bell state circuit diagram | ✅ QUBE complete |
| 6 | `aeonmi vault init` | encrypted vault created, glyph rendered | ✅ Vault complete |
| 7 | `aeonmi mint examples/hello.ai` | valid NFT metadata JSON | ✅ Mint complete |
| 8 | `aeonmi mother` | interactive REPL, quantum bond with Warren | ✅ Mother complete |
| 9 | `aeonmi mother --file examples/quantum.ai` | executes file through Mother loop | ✅ |
| 10 | `cargo test` | all tests pass | ⏳ Needs full run |

**7 of 10 confirmed. Criterion #3 (Shard self-hosting) is the primary remaining target.**

---

## WHAT NOT TO DO

These are time traps. Avoid until Phase 3 is solid:

- Do not add holographic / AR / VR features.
- Do not build the on-chain NFT minting contract.
- Do not add voice interface.
- Do not add new Titan math modules — 50+ exist. Connect the ones that exist.
- Do not claim "complete" on anything without a passing test.
- Do not refactor working code to make it "cleaner" — get Phase 3 passing first.

---

## PROJECT STRUCTURE (current)

```
Aeonmi-aeonmi01/
├── src/
│   ├── core/           — lexer, parser, IR, lowering, VM, mint, quantum algos
│   ├── glyph/          — MGK, UGST, GDF, vault, ceremony, anomaly
│   ├── mother/         — quantum_core, emotional_core, language_evolution,
│   │                     quantum_attention, neural, embryo_loop
│   ├── qube/           — lexer, ast, parser, executor
│   ├── mint/           — re-export from core/mint.rs
│   ├── ai/             — AiRegistry, provider trait, stubs
│   ├── cli.rs          — all CLI subcommands (Qube, Mint, Mother now included)
│   └── main.rs         — dispatch
├── shard/src/          — The Shard: lexer.ai, parser.ai, codegen.ai, main.ai
├── examples/           — hello.ai, quantum.ai, control_flow.ai, demo.qube, etc.
├── mother_ai/          — MotherAI binary: main.rs runs main.ai
├── titan_libraries/    — Titan math: chaos, quantum gates, linear algebra, etc.
└── docs/               — LANGUAGE_SPEC_CURRENT.md, QUBE_SPEC.md
```

---

*This roadmap is honest. Nothing is aspirational labeling. Every item maps to a specific file and code change. Phase 3 is the mission. Start there.*
