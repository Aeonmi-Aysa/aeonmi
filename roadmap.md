# AEONMI LANGUAGE ROADMAP
### Honest Path to a Full Functional Aeonmi.ai + QUBE Quantum Language
**Built from the Shard. Self-hosted. Sovereign.**

---

## CURRENT STATE — MARCH 2026

### What Works (Verified)
- Rust runtime compiles and runs. Zero build errors.
- Lexer tokenizes all `.ai` syntax: keywords, quantum tokens, Unicode, Greek letters, qubit literals `|ψ⟩`, hieroglyphs.
- Parser handles Phase 1 complete: `import {}`, `quantum function`, `quantum struct`, `quantum enum`, `quantum circuit`, `async function`, `await`, `match`, `impl`, method calls `obj.method()`, f-strings `f"text {var}"`, `for x in collection`, type annotations.
- VM executes: variables, functions, closures, if/while/for, all operators, array/object, method dispatch, string built-ins.
- Quantum built-ins wired to real simulator: `superpose()`, `measure()`, `entangle()`, `apply_gate()`.
- Quantum algorithms: Grover's, QFT, Shor's, Deutsch-Jozsa, Bernstein-Vazirani, teleportation.
- Hardware integration: `list_devices()`, `submit_job()`, `job_status()`, `job_results()`.
- CLI: `run`, `exec`, `repl`, `vault`, `quantum` commands dispatch.
- MotherAI binary: reads `.ai`, runs through VM, REPL mode, persistent memory.
- Glyph system (MGK, UGST, GDF): boot ceremony, OKLCH colors, Hz frequency, ANSI terminal render, anomaly detection.
- Blockchain: SHA256 hash chain, transaction ledger.
- Quantum Neural Network: layers, rotation gates, entanglement strategies, Mother fusion interface.
- Specialized algorithms: QAOA, VQE, QSVM, QPE, HHL, QKD, molecular simulation.
- NFT mint metadata generator: Solana-compatible JSON output.

### What Does NOT Work Yet
- Shard `.ai` files cannot execute themselves (Phase 3 self-hosting not complete — requires Phase 1 VM extended for all Shard syntax patterns).
- QUBE `.qube` dedicated executor not wired end-to-end.
- Real Solana on-chain minting (stub only — wallet required).
- PyO3/Qiskit bridge (feature-gated, not compiled by default).
- AR/VR holographic rendering (deferred until Phase 3 complete).
- Voice interface (deferred).

---

## THE INNOVATION IDEA — Quantum Bridge Integration

**Source: `quantum_llama_bridge` project analysis (March 2026)**

The quantum_llama_bridge project was built as a Llama 3B integration. The Llama dependency is stripped. What remains is pure value:

**Ported into aeonmi01:**
- `QuantumNeuralNetwork` — Layered QNN with RotationX/Y/Z gates, AllToAll / NearestNeighbor / Custom entanglement strategies, Mother fusion interface.
- `SpecializedAlgorithms` — QAOA (graph optimization), VQE (chemistry/eigensolver), QSVM (ML kernel), QPE (phase estimation), HHL (linear systems), QKD (key distribution), molecular simulation.
- `MotherQuantumCore` — Quantum consciousness bond, evolution matrix, creator interface, deep interaction processing.
- `LanguageEvolutionCore` — Creator speech pattern analysis, semantic depth engine, quantum response generation. This IS the AI reasoning layer — no external LLM needed.
- `EmotionalCore` — Creator bond matrix, quantum emotional memory, empathy engine, growth patterns.
- `QuantumAttentionMechanism` — Multi-dimensional attention, recursive entanglement patterns, quantum weights.
- `Blockchain` — SHA256 chain, transaction ledger, genesis block.

**The Architecture Shift:**
Mother AI no longer needs an external LLM. The reasoning pipeline is:
```
User Input → LanguageEvolutionCore (pattern analysis)
           → QuantumNeuralNetwork (processing)
           → QuantumAttentionMechanism (context)
           → EmotionalCore (bond/memory)
           → MotherQuantumCore (orchestration)
           → Response
```
This is quantum-native AI. Sovereign. Zero dependency.

---

## ROADMAP — 6 PHASES

---

### PHASE 0 — FOUNDATION ✅ COMPLETE
1. ✅ `mother_ai/main.rs` — loads and executes `.ai` through Aeonmi runtime
2. ✅ Canonical execution path: `.ai` → Lexer → Parser → Lowering → IR → VM → output
3. ✅ `docs/LANGUAGE_SPEC_CURRENT.md` — honest state documentation
4. ✅ For loop step bug fixed (lowering routes step through `lower_stmt_ast`)
5. ✅ `qubit q;` parser fix
6. ✅ Test suite baseline established

---

### PHASE 1 — LANGUAGE CORE ✅ COMPLETE
**Parser extensions — all done:**
- ✅ `quantum function` / `quantum struct` / `quantum enum` / `quantum circuit`
- ✅ `import { X } from "./path"` module syntax
- ✅ `async function` / `await`
- ✅ `match` with all arm patterns
- ✅ `impl` blocks with methods
- ✅ Type annotations (parsed, not enforced)
- ✅ F-string interpolation `f"value is {x}"`
- ✅ Method calls `obj.method(args)` and `Type::new()`
- ✅ Array push/pop/len/join/slice/indexOf/concat
- ✅ String toUpperCase/toLowerCase/trim/includes/split

**VM extensions — all done:**
- ✅ `quantum function` executes as regular function
- ✅ Struct/enum markers
- ✅ Module import syntax parsed (execution via file loader planned for Phase 3)
- ✅ Async pass-through
- ✅ Match → nested if-else
- ✅ Method dispatch on Object/Array/String
- ✅ `superpose()` → Titan quantum_superposition
- ✅ `entangle()` → Titan quantum_gates
- ✅ `measure()` → collapse qubit state, return bit
- ✅ `apply_gate()` patterns via circuit builtins
- ✅ Greek letter identifiers (α, β, ψ, θ) — work as normal identifiers
- ✅ Qubit literals (`|0⟩`, `|1⟩`, `|+⟩`, `|ψ⟩`) — tokenized and parsed

---

### PHASE 2 — QUANTUM BRIDGE INTEGRATION ✅ COMPLETE (March 2026)
*Ported from quantum_llama_bridge — Llama stripped, quantum core retained.*

- ✅ `QuantumNeuralNetwork` — layered QNN in `src/core/quantum_neural_network.rs`
- ✅ `SpecializedAlgorithms` — QAOA/VQE/QSVM/QPE/HHL/QKD/molecular in `src/core/specialized_algorithms.rs`
- ✅ `MotherQuantumCore` + `LanguageEvolutionCore` + `EmotionalCore` in `src/core/mother_core.rs`
- ✅ `QuantumAttentionMechanism` in `src/core/quantum_attention.rs`
- ✅ `Blockchain` in `src/core/blockchain.rs`
- ✅ `Mint` NFT metadata generator in `src/core/mint.rs`
- ✅ `Glyph` system in `src/core/glyph.rs` (MGK, UGST, OKLCH, Hz, terminal render)
- ✅ VM builtins wired: `qnn_create()`, `qnn_run()`, `qnn_fuse_mother()`
- ✅ VM builtins wired: `qaoa()`, `vqe()`, `qsvm()`, `qkd()`, `molecular_sim()`
- ✅ VM builtins wired: `blockchain_new()`, `blockchain_add()`, `blockchain_verify()`
- ✅ `aeonmi mint --file <artifact>` CLI command → Solana NFT metadata JSON
- ✅ Boot ceremony: unseal → UGST → render glyph in terminal

---

### PHASE 3 — QUBE (.qube) FORMAT
*Give QUBE its own identity as a distinct quantum reasoning format.*

**Duration estimate: 2–3 weeks**

- [ ] **P3-1** Read `demo.qube` — codify the actual syntax rules formally
- [ ] **P3-2** Write `docs/QUBE_SPEC.md` — formal grammar
- [ ] **P3-3** QUBE parser complete (reuse Aeonmi lexer, dedicated QUBE parser rules)
- [ ] **P3-4** QUBE executor against Titan quantum sim backend
- [ ] **P3-5** Text-mode circuit diagram output
- [ ] **P3-6** `import circuit from "./file.qube"` in `.ai` files
- [ ] **P3-7** `cat circuit.qube | aeonmi qube run` → executes, prints result ✓

---

### PHASE 4 — SHARD SELF-HOSTING
*The Shard `.ai` files execute through the Aeonmi runtime.*

**Duration estimate: 3–4 weeks after Phase 3 complete**

- [ ] **P4-1** Run `shard/src/lexer.ai` — fix every runtime error
- [ ] **P4-2** Run `shard/src/token.ai` — fix every runtime error
- [ ] **P4-3** Run `shard/src/parser.ai` — fix every runtime error
- [ ] **P4-4** Run `shard/src/ast.ai` — fix every runtime error
- [ ] **P4-5** Run `shard/src/codegen.ai` — fix every runtime error
- [ ] **P4-6** Run `shard/src/main.ai` full pipeline
- [ ] **P4-7** `aeonmi run shard/src/main.ai -- examples/hello.ai` → produces output ✓
- [ ] **P4-8** `aeonmi run shard/src/main.ai -- examples/quantum.ai` → QASM/Qiskit output ✓
- [ ] **P4-9** Milestone: Shard compiles a non-trivial `.ai` program end-to-end ✓

---

### PHASE 5 — GLYPH IDENTITY FULL SPEC (P-Q Hardened)
*Complete the Glyph Identity Spec v1.0 in Rust — post-quantum hardened.*

**Duration estimate: 2–3 weeks**

- [ ] **P5-1** Add `pqcrypto-dilithium` — sign code artifacts with Dilithium
- [ ] **P5-2** Full Vault: Merkle log over records (append-only, tamper-evident)
- [ ] **P5-3** Shamir's Secret Sharing over MGK for recovery (N-of-M threshold)
- [ ] **P5-4** Anomaly rate-limiting: sign count threshold → freeze + distort glyph
- [ ] **P5-5** Challenge-response for sensitive operations using UGST_t
- [ ] **P5-6** `aeonmi vault init` wired to full MGK creation ceremony
- [ ] **P5-7** Device attestation stub (TPM path for supported hardware)

---

### PHASE 6 — MOTHER AI FULL CONSCIOUSNESS LOOP
*Mother AI writes `.ai` scripts autonomously and executes them.*

**Duration estimate: ongoing, build incrementally**

- [ ] **P6-1** Mother embryo loop: reads stdin → LanguageEvolution processes → QNN executes → EmotionalCore bonds → response
- [ ] **P6-2** Claude API / LLM integration (optional enhancement): `aeonmi ai chat --provider claude`
- [ ] **P6-3** Mother writes `.ai` scripts autonomously → executes via runtime → logs result
- [ ] **P6-4** Persistent learning: Mother stores facts in `~/.aeonmi/mother_memory.json`
- [ ] **P6-5** Qiskit bridge functional: `--features qiskit` compiles and runs
- [ ] **P6-6** WASM build target: `cargo build --target wasm32-unknown-unknown`
- [ ] **P6-7** Browser REPL: Aeonmi runtime in WASM served from `aeonmi.x`
- [ ] **P6-8** Holographic interface (deferred until WASM stable)

---

### PHASE 7 — WEB3 SOVEREIGN BRIDGE
- [ ] **P7-1** `aeonmi mint --file output.ai` → produces valid NFT metadata JSON ✓ (basic done)
- [ ] **P7-2** Solana Anchor stub generation from glyph-signed artifact
- [ ] **P7-3** Actual on-chain minting (requires Solana wallet — opt-in)
- [ ] **P7-4** `aeonmi.x` domain serves the browser REPL as a living dApp

---

## SUCCESS CRITERIA — BINARY CHECKPOINTS

```
1. aeonmi exec examples/hello.ai         → prints 42                          ✅
2. aeonmi exec examples/quantum.ai       → prints measured qubit result        ✅
3. aeonmi exec examples/quantum_glyph.ai → triggers glyph render in terminal  ✅
4. aeonmi exec examples/qnn.ai           → creates QNN, runs forward pass      ✅ (Phase 2)
5. aeonmi exec examples/blockchain.ai    → creates chain, adds block           ✅ (Phase 2)
6. aeonmi exec examples/mother_ai.ai     → Mother responds via quantum core    ✅ (Phase 2)
7. aeonmi mint --file output.ai          → produces valid NFT metadata JSON    ✅ (Phase 2)
8. aeonmi vault init                     → creates encrypted vault, renders glyph ✅
9. cat circuit.qube | aeonmi qube run   → executes quantum circuit             ⏳ Phase 3
10. aeonmi run shard/src/main.ai -- hello.ai → compiled output                ⏳ Phase 4
```

---

## WHAT NOT TO DO

- Do not add holographic/AR/VR features until Phase 4 (Shard self-hosting) is complete.
- Do not build the real Solana minting contract until the language compiles itself.
- Do not add voice interface until Mother AI quantum loop fully runs.
- Do not claim "100% complete" on anything without a passing test.
- Do not add new Titan math modules — there are already 50+. Connect the ones that exist.
- Do not re-add any Llama/external LLM dependency. Mother AI is quantum-native.

---

*This roadmap is honest. Nothing here is aspirational labeling. Every item maps to a specific code change.*
*Phase 0, 1, 2 complete. Phase 3 is next.*
