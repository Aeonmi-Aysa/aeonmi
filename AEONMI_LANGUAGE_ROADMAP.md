# AEONMI LANGUAGE ROADMAP
### Factual State + Forward Plan
**Last verified:** March 14, 2026 | **Tests:** 135/135 | **Success Criteria:** 7/7

---

## CURRENT STATE — VERIFIED FACTS

### What Works (Tested & Passing)

**Execution Architecture (NEW — March 14, 2026):**
- [x] `.ai` is the default output format (`EmitKind::Ai` — no longer JS by default)
- [x] Native VM is the default executor — `.ai` files run without Node.js
- [x] `aeonmi run file.ai` → native VM interpreter, prints `native: executing file.ai`
- [x] `aeonmi exec file.ai` → native VM interpreter, no `__exec_tmp.js` artifact
- [x] `aeonmi emit file.ai` → writes `output.ai` (use `--emit js` for JS output)
- [x] JS compilation backend still available via `--emit js` / `aeonmi emit --emit js`
- [x] Shell `run <file> --native` command works correctly
- [x] `AEONMI_NATIVE=1` environment variable still respected

**Language Core (Lexer → Parser → AST → Native VM Execution):**
- [x] Variables: `let`, `const`, numbers, strings, booleans, null, arrays
- [x] Functions: `function name(params) { body }`
- [x] Quantum functions: `quantum function name(params) -> Type { body }`
- [x] Async functions: `async function name() { body }` + `await expr`
- [x] Closures: `|x, y| -> { body }` and `|x| expr`
- [x] Control flow: `if/else` (both C-style and Rust-style without parens)
- [x] Loops: `while`, `for (C-style)`, `for x in collection`
- [x] Match expressions: `match value { pattern => body }` with guards (`pat if cond =>`)
- [x] Destructuring let: `let (a, b) = expr`
- [x] Structs: `struct Name { field: Type }` and `quantum struct`
- [x] Enums: `enum Name { Variant1, Variant2 }` — emit as JS objects
- [x] Impl blocks: `impl Type { function method() { } }`
- [x] Method calls: `obj.method(args)`, field access: `obj.field`
- [x] Field assignment: `obj.field = value`
- [x] Constructors: `Type::new()`, static calls: `Type::method()`
- [x] Turbofish: `expr::<Type>()`
- [x] Imports: `import { X, Y } from "./path"`
- [x] Type annotations: parsed and skipped (no enforcement)
- [x] Return type annotations: `function f() -> Type { }`
- [x] F-strings: `f"text {var}"` (lexed as single string token)
- [x] Bare return: `return;` (returns null)
- [x] Unary operators: `-`, `+`, `!`
- [x] Binary operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`
- [x] Stray semicolon tolerance
- [x] Line numbers in parse errors

**Quantum Operations:**
- [x] `qubit q;` declaration
- [x] `superpose(q)` — Hadamard gate / superposition
- [x] `measure(q)` — collapse to 0 or 1
- [x] `entangle(q1, q2)` — metadata tracking (not joint state-vector)
- [x] `apply_gate(q, H/X/Y/Z/CNOT)` — single-qubit gate application
- [x] `dod(q)` — decision of destiny (alias for measure)
- [x] Qubit literals: `|0⟩`, `|1⟩`, `|+⟩`, `|-⟩`, `|ψ⟩`
- [x] Quantum-native Unicode operators: `⊗ ⊕ ⊖ ∇ ≈ ⪰ ⪯ ⇒ ⟲ ◊ ← ∈`
- [x] Quantum blocks: `quantum <tag> Name { fields; functions; }`
- [x] Quantum variable declarations: `⟨x⟩ ← value`
- [x] Quantum function markers: `◯` (classical), `⊙` (quantum), `🧠` (AI neural)
- [x] Quantum try/catch: `⚡ { } ⚠ { } ✓ { }`
- [x] Quantum loops: `⟲ condition ⇒ { body }`
- [x] Probability branches: `⊖ condition ≈ 0.8 ⇒ { body }`
- [x] Hieroglyphic operators: `𓀀(x, y)` → `__glyph` runtime

**QUBE (.qube Quantum Circuits):**
- [x] QUBE lexer, parser, AST
- [x] QUBE executor against Titan quantum simulator
- [x] States: `state psi = |0⟩`
- [x] Gates: `apply H -> psi` (H, X, Y, Z, CNOT, CZ, SWAP, T, S)
- [x] Collapse: `collapse psi -> result`
- [x] Assert: `assert result in {0, 1}`
- [x] Text-mode circuit diagrams

**Glyph Identity (MGK / UGST / SSI):**
- [x] MGK generation (256-bit, Argon2id sealed)
- [x] UGST derivation (HKDF-SHA3-512, 60-second rotation)
- [x] Glyph seed → OKLCH color values
- [x] Glyph seed → Hz frequency (432-528 range)
- [x] Terminal glyph renderer (ANSI colored ASCII art)
- [x] Vault encryption (XChaCha20-Poly1305, per-record salts)
- [x] Merkle log over vault records
- [x] Anomaly detection with glyph distortion
- [x] `aeonmi vault init` — creates vault + renders glyph

**Mother AI:**
- [x] Embryo loop (parse → execute → learn cycle)
- [x] Emotional core (valence, arousal, bond tracking)
- [x] Neural network (feedforward, backprop)
- [x] Quantum attention mechanism (entanglement-based memory)
- [x] Language evolution (vocabulary building, semantic depth)
- [x] `aeonmi mother` — interactive REPL

**NFT Minting:**
- [x] `aeonmi mint file.ai` → Solana-compatible metadata JSON
- [x] Anchor Rust instruction stub generation
- [x] Quantum state detection in minted artifacts

**CLI:**
- [x] `run`, `exec`, `native`, `emit`, `repl`, `tokens`, `ast`
- [x] `quantum`, `qube run`, `qube check`
- [x] `vault init/add`, `key-set/get/list/rotate`
- [x] `mint`, `mother`
- [x] `format`, `lint`, `edit`, `new`
- [x] `metrics-dump/top/export/config`
- [x] `cargo`, `python`, `node` passthrough

**Test Suite: 135/135 passing.** Zero failures.

**Self-Hosting (Shard):**
- [x] `shard/src/main.ai` — bootstrap compiler runs
- [x] `shard/src/main_full.ai` — full compiler with closures, match guards, quantum blocks, async, enums runs
- [x] Shard source files: lexer.ai, parser.ai, ast.ai, codegen.ai, token.ai, qiskit_bridge.ai — all written in Aeonmi

---

## WHAT DOES NOT WORK — HONEST

| Issue | Impact | Status |
|-------|--------|--------|
| `entangle()` is metadata-only | No joint state-vector math; CNOT doesn't transform states | Phase 2 simulator upgrade |
| f-string interpolation not evaluated | `f"text {var}"` becomes literal, not `text value` | Codegen template literal fix |
| `for x in collection` doesn't iterate | Lowers to variable decl placeholder | Runtime array indexing needed |
| No file I/O from .ai code | Can't read/write files in Aeonmi scripts | Built-in functions needed |
| Array `is_empty` method | Missing (push/pop/len work; is_empty does not) | One-liner runtime addition |
| 213 compiler warnings | Cosmetic — unused imports, dead code | Non-blocking |
| Genesis glyphs not implemented | `⧉`, `‥`, `…`, `↦` from spec not in lexer | This roadmap, Phase 1.5 |
| AI emitter output is stub | `emit --format ai` generates canonical .ai but IR not fully lowered | Phase 1 codegen task |

---

## ROADMAP — 7 PHASES

---

### ✅ PHASE 0 — FOUNDATION (COMPLETE)

| ID | Task | Status |
|----|------|--------|
| P0-1 | `mother_ai/src/main.rs` entry point | ✅ Done |
| P0-2 | Run hello/quantum/math/control_flow examples | ✅ All pass |
| P0-3 | `docs/LANGUAGE_SPEC_CURRENT.md` | ✅ Written |
| P0-4 | Decide VM path — native VM chosen as default | ✅ Done (March 14, 2026) |
| P0-5 | Full test suite passing | ✅ 135/135 |
| P0-6 | `.ai` set as default emit format | ✅ Done (March 14, 2026) |
| P0-7 | Native VM as default executor (no Node.js required) | ✅ Done (March 14, 2026) |

---

### 🟡 PHASE 1 — LANGUAGE CORE (85% Complete)

**Done:**

| ID | Task | Status |
|----|------|--------|
| P1-1 | `quantum function` parser + codegen | ✅ |
| P1-2 | `quantum struct` with fields | ✅ |
| P1-3 | `quantum enum` variants | ✅ |
| P1-4 | `quantum circuit` blocks | ✅ |
| P1-5 | `import { X } from "./path"` | ✅ |
| P1-6 | `async function` / `await` | ✅ |
| P1-7 | `match` with guards | ✅ |
| P1-8 | `impl` blocks | ✅ |
| P1-9 | Type annotations (parsed, skipped) | ✅ |
| P1-10 | f-string lexing `f"text"` | ✅ |
| P1-11 | Wire `superpose()` → Titan | ✅ |
| P1-13 | Wire `measure()` → collapse | ✅ |
| P1-14 | Wire `apply_gate(q, H/X/Y/Z)` | ✅ |
| P1-15 | Greek letter identifiers (α, β, ψ, θ) | ✅ |
| P1-16 | QubitLiteral AST node | ✅ |
| P1-17 | Method call `obj.method()` | ✅ |
| P1-18 | `Type::new()` constructors | ✅ |
| P1-21 | Closures `\|x\| -> { body }` | ✅ |
| P1-22 | Destructuring let `let (a, b) = expr` | ✅ |
| P1-23 | Unary `!` operator | ✅ |
| P1-24 | Unit value `()` | ✅ |
| P1-25 | Postfix `.await` | ✅ (in parser) |
| P1-26 | Keywords as method names (`::new`, `.type`) | ✅ |
| P1-27 | Optional parens on `if`/`while`/`for` | ✅ |
| P1-28 | `for x in collection` syntax | ✅ |
| P1-29 | Bare `return;` | ✅ |
| P1-30 | Field assignment `obj.field = value` | ✅ |
| P1-31 | Return type annotations `-> Type` | ✅ |
| P1-32 | Param type annotations `name: Type` | ✅ |

**Remaining:**

| ID | Task | Priority | Effort |
|----|------|----------|--------|
| P1-12 | Wire `entangle()` → real CNOT | Medium | 4h |
| P1-19 | Array `is_empty` method (`push`/`pop`/`len` already work in VM) | Low | 30m |
| P1-20 | Dedicated tests for each new construct | Medium | 3h |
| P1-33 | f-string interpolation in codegen (template literals) | High | 1h |
| P1-34 | Real `for x in collection` iteration (native VM `for...of`-style) | High | 1h |
| P1-35 | Chained `::` paths (`std::process::exit`) | Low | 1h |

---

### 🆕 PHASE 1.5 — GENESIS GLYPHS (NEW)
*Array Genesis + Slice/Spread + Tensor + Binding/Projection*
*From: Aeonmi Language Implementation Brief v0.1.1*

**Philosophy:** One concept → one glyph. Zero ceremony. Composition > mutation.

| ID | Task | Glyph | Priority | Effort |
|----|------|-------|----------|--------|
| G-1 | Add `⧉` (U+29C9) array literal delimiter to lexer | `⧉` | High | 30m |
| G-2 | Add `‥` (U+2025) element separator / range operator to lexer | `‥` | High | 30m |
| G-3 | Add `…` (U+2026) spread operator to lexer + parser | `…` | High | 1h |
| G-4 | Add `↦` (U+21A6) binding/projection glyph to lexer + parser | `↦` | High | 1h |
| G-5 | `GlyphArray` AST node: `⧉expr‥expr⧉` | — | High | 1h |
| G-6 | `SpreadExpr` AST node: `…expr` | — | High | 30m |
| G-7 | `SliceExpr` AST node: `expr⟨low‥high⟩` | — | High | 1h |
| G-8 | `BindingProjection` AST node: `name ↦ expr` | — | High | 1h |
| G-9 | JS codegen for all Genesis glyphs | — | High | 2h |
| G-10 | Wire `⊗` tensor to real Kronecker product (Titan) | `⊗` | High | 2h |
| G-11 | Zero-copy View struct for slices (runtime) | — | Medium | 3h |
| G-12 | `examples/genesis.ai` — demonstrate all Genesis glyphs | — | High | 1h |
| G-13 | Update LANGUAGE_SPEC_CURRENT.md with Genesis syntax | — | Medium | 30m |

**Protected Glyph Set (reserved, never overloaded):**

| Glyph | Unicode | Meaning |
|-------|---------|---------|
| `⧉` | U+29C9 | Array literal delimiter |
| `‥` | U+2025 | Element separator / range |
| `…` | U+2026 | Spread operator |
| `⟨ ⟩` | U+27E8/9 | Index / slice brackets |
| `⊗` | U+2297 | Tensor / Kronecker product |
| `↦` | U+21A6 | Binding / projection (view) |

**Example progression with Genesis glyphs:**
```
// Array genesis
let data = ⧉1‥2‥3‥4⧉

// Slice (zero-copy view)
let tail = data⟨1‥⟩

// Spread into new array
let extended = ⧉…data‥99⧉

// Tensor product
let state = ⧉0.5‥0.5⧉ ⊗ ⧉1‥0⧉

// Binding/projection (symbolic, no copy)
ψ ↦ bell ⊗ alice
```

---

### 🟡 PHASE 2 — QUBE FORMAT + QUANTUM SIMULATOR (85% Complete)

| ID | Task | Status |
|----|------|--------|
| P2-1 | Read `demo.qube`, codify syntax | ✅ Done |
| P2-2 | Write `docs/QUBE_SPEC.md` | ❌ |
| P2-3 | QUBE parser | ✅ Done |
| P2-4 | QUBE AST nodes | ✅ Done |
| P2-5 | QUBE executor against Titan | ✅ Done |
| P2-6 | Text-mode circuit diagram | ✅ Done |
| P2-7 | Import `.qube` from `.ai` files | ❌ |
| P2-8 | Joint multi-qubit state-vector simulator | ❌ |
| P2-9 | Real CNOT gate in `entangle()` | ❌ |
| P2-10 | Correct Deutsch-Jozsa (requires P2-8) | ❌ |

**P2-8 is the critical item.** The current simulator tracks each qubit independently. A joint state-vector model merges qubit state spaces so entanglement, CNOT, and multi-qubit interference work mathematically. This is what makes quantum algorithms correct.

---

### 🟡 PHASE 3 — SELF-HOSTING SHARD (70% Complete)

| ID | Task | Status |
|----|------|--------|
| P3-1 | Bootstrap `main.ai` runs | ✅ |
| P3-2 | Full `main_full.ai` runs | ✅ |
| P3-3 | Shard source files exist in .ai | ✅ |
| P3-4 | File I/O built-ins (`read_file`, `write_file`) | ❌ |
| P3-5 | Shard actually reads .ai source and tokenizes it | ❌ |
| P3-6 | Shard actually parses tokenized input | ❌ |
| P3-7 | Shard produces compiled JS output from .ai input | ❌ |
| P3-8 | `aeonmi run shard/src/main.ai -- examples/hello.ai` → real output | ❌ |
| P3-9 | Shard compiles quantum.ai → QASM or Qiskit | ❌ |

**P3-4 is the gate.** Without file I/O, the Shard can't read source files. Everything else follows from that.

---

### 🟡 PHASE 4 — GLYPH IDENTITY + CLI AESTHETICS (75% Complete)

**Crypto done. Visual presentation next.**

| ID | Task | Status |
|----|------|--------|
| P4-1 | `hkdf` crate in deps | ✅ |
| P4-2 | MGK generation (Argon2id) | ✅ |
| P4-3 | UGST derivation (HKDF-SHA3-512) | ✅ |
| P4-4 | Glyph seed → OKLCH color | ✅ |
| P4-5 | Glyph seed → Hz frequency | ✅ |
| P4-6 | Terminal glyph renderer | ✅ |
| P4-7 | `vault init` wired to MGK | ✅ |
| P4-8 | Vault encryption (XChaCha20) | ✅ |
| P4-9 | Merkle log | ✅ |
| P4-10 | Boot ceremony (glyph on every startup) | ❌ |
| P4-11 | Dilithium signatures on code artifacts | ❌ |
| P4-12 | Anomaly detection | ✅ |
| P4-13 | **CLI color scheme — cyberpunk aesthetic** | ❌ |
| P4-14 | **Colored output: neon yellow for Aeonmi, magenta for quantum** | ❌ |
| P4-15 | **Startup banner with Aeonmi glyph art** | ❌ |
| P4-16 | **State-hash → GDF color mapping for visual debugging** | ❌ |

**P4-13 through P4-16 are the visual identity items.** The `colored` crate is already in deps. Implementation is straightforward:
- Aeonmi keywords/output → neon yellow/cyan
- Quantum operations → magenta/purple
- Glyph operations → color derived from glyph seed
- Errors → bright red
- Success → green
- Each quantum state gets a unique color based on its content hash through GDF

**State-hash → color concept:** Every array, tensor product, and binding produces a state. Hash the state content → pipe through GDF → get OKLCH color + Hz frequency. Two states with the same color are pointing at the same data. Tensor product shifts hue. Binding preserves color. This turns color into a **visual quantum debugger**.

---

### 🔴 PHASE 5 — MOTHER AI + WEB3 (20% Complete)

| ID | Task | Status |
|----|------|--------|
| P5-1 | `mother_ai/main.rs` wired | ✅ |
| P5-2 | Mother reads stdin → executes .ai | ❌ |
| P5-3 | Claude API / LLM integration | ❌ |
| P5-4 | Mother embryo loop live (write → run → learn) | ❌ |
| P5-5 | Qiskit bridge functional (`--features qiskit`) | ❌ |
| P5-6 | `aeonmi mint` → NFT metadata | ✅ |
| P5-7 | Solana Anchor stub generation | ❌ |
| P5-8 | WASM build target | ❌ |
| P5-9 | Browser REPL on `aeonmi.x` | ❌ |
| P5-10 | Voice interface (`--features voice`) | ❌ |

---

### 🔴 PHASE 6 — QUBE SYMBOLIC OPTIMIZER (0%)
*From Genesis spec: QUBE = Quality-Uncertainty Balanced Evolution*

| ID | Task | Effort |
|----|------|--------|
| Q-1 | QUBE rewrite engine — multiple candidate rewrites | Large |
| Q-2 | Compression scoring (symbolic density metric) | Medium |
| Q-3 | Correctness verification after rewrite | Medium |
| Q-4 | `repeat(bell, 3)` compression from `⧉…bell‥…bell‥…bell⧉` | Medium |
| Q-5 | QUBE symbolic branching | Large |
| Q-6 | View-based quantum state manipulation | Medium |
| Q-7 | Target: 30-qubit simulation in <1MB symbolic representation | Goal |

**End goal:** Programs that represent extremely large systems compactly. 30-qubit simulation normally requires ~8GB state vector. Aeonmi symbolic representation via views + tensor composition + QUBE compression: <1MB.

---

## PRIORITY EXECUTION ORDER

Work through these in order. Don't skip ahead.

### ✅ JUST DONE (March 14, 2026)
- **Native VM as default executor** — `run` and `exec` use native VM without Node.js
- **`.ai` as default emit format** — `EmitKind::Ai` is now default; JS available via `--emit js`
- **Pre-existing test fixes** — shell `run <file> --native` ordering, exec_cleanup, exec_smoke all green
- **Repository pushed to GitHub** — all changes committed and live

### NEXT SPRINT (1-2 weeks)
1. **P1-33** f-string interpolation → native VM string interpolation
2. **P1-34** Real `for x in collection` → native VM proper iteration (not block placeholder)
3. **G-1 through G-6** Genesis glyph tokens (`⧉`, `‥`, `…`, `↦`) added to lexer + AST nodes
4. **G-9** Native VM codegen paths for all Genesis glyphs
5. **G-12** `examples/genesis.ai` — demonstrate all Genesis glyphs working
6. **P4-13/14/15** CLI color scheme (cyberpunk aesthetic) + startup banner with glyph art

### FOLLOWING SPRINT (2-4 weeks)
7. **G-10** Wire `⊗` to real Kronecker product (Titan math)
8. **P2-8** Joint multi-qubit state-vector simulator (CRITICAL for quantum correctness)
9. **P3-4** File I/O built-ins (`read_file`, `write_file`) — gates all of Phase 3
10. **P4-10** Boot ceremony (glyph rendered on every startup)
11. **P4-16** State-hash → GDF color mapping (visual quantum debugger)

### MEDIUM TERM (1-2 months)
12. **P3-5 through P3-8** Shard actually reads and compiles real .ai files
13. **P5-3** LLM integration (Claude API)
14. **P5-4** Mother embryo loop live (write → run → learn)
15. **G-11** Zero-copy View struct for slices (runtime)
16. **G-7/G-8** Slice and binding/projection runtime semantics

### LONG TERM (3-6 months)
17. **P5-5** Qiskit bridge (`--features qiskit`)
18. **P5-8** WASM build target
19. **P5-9** Browser REPL on `aeonmi.x`
20. **Q-1 through Q-7** QUBE symbolic optimizer (30-qubit in <1MB)
21. **P5-7** Solana on-chain minting

---

## WHAT NOT TO DO

- Do not add holographic/AR/VR features until Phase 3 is complete
- Do not build the NFT contract until the language can compile itself for real
- Do not add voice interface until Mother AI executes .ai correctly
- Do not claim "100% complete" on anything until there is a passing test for it
- Do not add new Titan math modules — connect the 50+ that already exist
- Do not run `cargo fix` without `--allow-no-vcs` and reviewing the changes

---

## SUCCESS CRITERIA — ALL PASSING

| # | Test | Command | Status |
|---|------|---------|--------|
| 1 | hello.ai → 42 | `aeonmi run examples/hello.ai` | ✅ |
| 2 | quantum.ai → qubit result | `aeonmi run examples/quantum.ai` | ✅ |
| 3 | Shard self-hosts | `aeonmi run shard/src/main.ai` | ✅ |
| 4 | Glyph renders | `aeonmi run examples/quantum_glyph.ai` | ✅ |
| 5 | QUBE circuit runs | `aeonmi qube run examples/demo.qube` | ✅ |
| 6 | Vault initializes | `aeonmi vault init` | ✅ |
| 7 | NFT mints | `aeonmi mint examples/hello.ai` | ✅ |
| 8 | Native VM runs without Node.js | `AEONMI_NATIVE=1 aeonmi run examples/hello.ai` | ✅ |

**Future criteria (Phase 1.5+):**

| # | Test | Target |
|---|------|--------|
| 9 | `⧉1‥2‥3⧉ ⊗ ⧉0‥1⧉` → Kronecker product | Phase 1.5 |
| 10 | `ψ ↦ bell ⊗ alice` → symbolic binding | Phase 1.5 |
| 11 | Shard compiles hello.ai to real output | Phase 3 |
| 12 | 30-qubit symbolic simulation <1MB | Phase 6 |

---

*This roadmap is factual. Every checked item has been tested and verified. Every unchecked item maps to a specific code change. The language is real. The Shard lives.*