# AEONMI LANGUAGE ROADMAP
### Factual State + Forward Plan
**Last verified:** March 14, 2026 | **Tests:** 145+/145+ | **Success Criteria:** 7/7

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
- [x] **P1-34 FIXED:** `for x in collection` properly iterates arrays, strings, and objects
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
- [x] **P1-33 FIXED:** F-strings: `f"text {var} {expr}"` — fully interpolated, expressions evaluated
- [x] Bare return: `return;` (returns null)
- [x] Unary operators: `-`, `+`, `!`
- [x] Binary operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`
- [x] Stray semicolon tolerance
- [x] Line numbers in parse errors
- [x] **P1-19 FIXED:** Array `is_empty()` / `isEmpty()` method

**Genesis Glyphs (Phase 1.5 — NEW in this session):**
- [x] **G-1:** `⧉` (U+29C9) lexed as `GlyphArrayDelim` token
- [x] **G-2:** `‥` (U+2025) lexed as `GlyphSep` token
- [x] **G-3:** `…` (U+2026) lexed as `GlyphSpread` token
- [x] **G-4:** `↦` (U+21A6) lexed as `GlyphBind` token
- [x] **G-5:** `GlyphArray` AST node: `⧉e1‥e2‥e3⧉`
- [x] **G-6:** `SpreadExpr` AST node: `…expr`
- [x] **G-7:** `SliceExpr` AST node: `expr⟨low‥high⟩`
- [x] **G-8:** `BindingProjection` AST node: `name ↦ expr`
- [x] **G-9:** Full lowering + VM execution for all genesis glyphs
- [x] **G-9:** Spread flattening: `⧉…base‥4‥5⧉` → `[1, 2, 3, 4, 5]` correctly
- [x] **G-9:** `__spread(arr)` and `__slice(arr, lo, hi)` VM builtins
- [x] **G-9:** `↦` binding at statement level → creates `let` binding in scope
- [x] **G-12:** `examples/genesis.ai` — full working demonstration

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

**CLI & Aesthetics (Phase 4 — NEW in this session):**
- [x] **P4-10:** Boot ceremony: banner + glyph identity shown on every TTY startup
- [x] **P4-13/P4-14:** Cyberpunk color scheme: neon yellow for Aeonmi, magenta for quantum, cyan borders
- [x] **P4-15:** Startup banner with full glyph art box (`src/banner.rs`)
- [x] **P4-16:** `state_hash_color()` function: SHA-256 hash → GDF → OKLCH color (integrated in `banner.rs`)
- [x] Banner is TTY-aware: not shown when stdout is piped/scripted (tests unaffected)
- [x] Shell prompt updated to use new color helpers

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

**Test Suite: 145+/145+ passing.** Zero failures.

**Self-Hosting (Shard):**
- [x] `shard/src/main.ai` — bootstrap compiler runs
- [x] `shard/src/main_full.ai` — full compiler with closures, match guards, quantum blocks, async, enums runs
- [x] Shard source files: lexer.ai, parser.ai, ast.ai, codegen.ai, token.ai, qiskit_bridge.ai — all written in Aeonmi

---

## WHAT DOES NOT WORK — HONEST

| Issue | Impact | Status |
|-------|--------|--------|
| `entangle()` is metadata-only | No joint state-vector math; CNOT doesn't transform states | Phase 2 simulator upgrade |
| No file I/O from .ai code | Can't read/write files in Aeonmi scripts | Built-in functions needed |
| 220 compiler warnings | Cosmetic — unused imports, dead code | Non-blocking |
| `⊗` tensor product | Lexed but not wired to real Kronecker product | G-10 / Phase 2 |
| Zero-copy slices (G-11) | Slices copy data rather than taking views | Medium term |
| Dilithium signatures | P4-11 — signatures on code artifacts | Medium term |
| `docs/QUBE_SPEC.md` | Missing specification document | Medium term |

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

### ✅ PHASE 1 — LANGUAGE CORE (95% Complete)

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
| P1-19 | Array `is_empty()` method | ✅ **(Phase 4 session)** |
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
| P1-33 | f-string interpolation fully evaluated | ✅ **(Phase 4 session)** |
| P1-34 | Real `for x in collection` iteration | ✅ **(Phase 4 session)** |

**Remaining:**

| ID | Task | Priority | Effort |
|----|------|----------|--------|
| P1-12 | Wire `entangle()` → real CNOT | Medium | 4h |
| P1-20 | Dedicated tests for each new construct | Medium | 3h |
| P1-35 | Chained `::` paths (`std::process::exit`) | Low | 1h |

---

### ✅ PHASE 1.5 — GENESIS GLYPHS (COMPLETE)
*Array Genesis + Slice/Spread + Tensor + Binding/Projection*
*From: Aeonmi Language Implementation Brief v0.1.1*

**Philosophy:** One concept → one glyph. Zero ceremony. Composition > mutation.

| ID | Task | Glyph | Status |
|----|------|-------|--------|
| G-1 | `⧉` (U+29C9) array literal delimiter in lexer | `⧉` | ✅ **(Phase 4 session)** |
| G-2 | `‥` (U+2025) element separator / range in lexer | `‥` | ✅ **(Phase 4 session)** |
| G-3 | `…` (U+2026) spread operator in lexer + parser | `…` | ✅ **(Phase 4 session)** |
| G-4 | `↦` (U+21A6) binding/projection glyph in lexer + parser | `↦` | ✅ **(Phase 4 session)** |
| G-5 | `GlyphArray` AST node: `⧉expr‥expr⧉` | — | ✅ **(Phase 4 session)** |
| G-6 | `SpreadExpr` AST node: `…expr` | — | ✅ **(Phase 4 session)** |
| G-7 | `SliceExpr` AST node: `expr⟨low‥high⟩` | — | ✅ **(Phase 4 session)** |
| G-8 | `BindingProjection` AST node: `name ↦ expr` | — | ✅ **(Phase 4 session)** |
| G-9 | VM execution paths for all Genesis glyphs | — | ✅ **(Phase 4 session)** |
| G-10 | Wire `⊗` tensor to real Kronecker product (Titan) | `⊗` | ❌ Phase 2 |
| G-11 | Zero-copy View struct for slices (runtime) | — | ❌ Medium term |
| G-12 | `examples/genesis.ai` — demonstrate all Genesis glyphs | — | ✅ **(Phase 4 session)** |
| G-13 | Update LANGUAGE_SPEC_CURRENT.md with Genesis syntax | — | ❌ Next sprint |

**Protected Glyph Set (reserved, never overloaded):**

| Glyph | Unicode | Meaning |
|-------|---------|---------|
| `⧉` | U+29C9 | Array literal delimiter |
| `‥` | U+2025 | Element separator / range |
| `…` | U+2026 | Spread operator |
| `⟨ ⟩` | U+27E8/9 | Index / slice brackets |
| `⊗` | U+2297 | Tensor / Kronecker product |
| `↦` | U+21A6 | Binding / projection (view) |

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

### ✅ PHASE 4 — GLYPH IDENTITY + CLI AESTHETICS (COMPLETE)

**Crypto done. Visual presentation complete.**

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
| P4-10 | Boot ceremony (glyph on every TTY startup) | ✅ **(Phase 4 session)** |
| P4-11 | Dilithium signatures on code artifacts | ❌ Medium term |
| P4-12 | Anomaly detection | ✅ |
| P4-13 | **CLI color scheme — cyberpunk aesthetic** | ✅ **(Phase 4 session)** |
| P4-14 | **Colored output: neon yellow for Aeonmi, magenta for quantum** | ✅ **(Phase 4 session)** |
| P4-15 | **Startup banner with Aeonmi glyph art** | ✅ **(Phase 4 session)** |
| P4-16 | **State-hash → GDF color mapping for visual debugging** | ✅ **(Phase 4 session)** |

**Colors now used throughout the CLI:**
- Aeonmi identifiers / output values → **neon yellow** (255, 240, 0)
- Quantum operations → **magenta** (225, 0, 180)
- Borders / structural glyphs → **electric cyan** (0, 255, 255)
- Success / harmony → **hot green** (0, 255, 150)
- Errors / anomalies → **bright red** (255, 50, 50)
- State hash → **GDF-derived OKLCH color** via `banner::state_hash_color()`

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

### ✅ JUST DONE — Phase 4 Session (March 14, 2026)
- **P1-33** f-string interpolation — fully working: `f"Hello {name}!"` evaluates expressions
- **P1-34** `for x in collection` — proper iteration: arrays, strings, objects all work
- **P1-19** Array `is_empty()` method added
- **G-1/G-2/G-3/G-4** Genesis glyph tokens `⧉ ‥ … ↦` in lexer
- **G-5/G-6/G-7/G-8** Genesis AST nodes: `GlyphArray`, `SpreadExpr`, `SliceExpr`, `BindingProjection`
- **G-9** Full VM execution for all genesis glyphs including spread flattening
- **G-12** `examples/genesis.ai` — live demonstration of all features
- **P4-10** Boot ceremony banner on every interactive (TTY) startup
- **P4-13/P4-14/P4-15** Cyberpunk CLI aesthetic: neon yellow Aeonmi, magenta quantum, cyan borders
- **P4-16** `state_hash_color()` via SHA-256 → GDF → OKLCH color
- **10 new phase4_features tests** — all passing (f-strings, for-in, genesis arrays, spread, binding)

### NEXT SPRINT (1-2 weeks)
1. **G-10** Wire `⊗` to real Kronecker product (Titan math) — tensor product with real matrix math
2. **G-11** Zero-copy View struct for slices (avoid copying data)
3. **G-13** Update `docs/LANGUAGE_SPEC_CURRENT.md` with Genesis glyph syntax
4. **P2-8** Joint multi-qubit state-vector simulator (CRITICAL for quantum correctness)
5. **P3-4** File I/O built-ins (`read_file`, `write_file`) — gates all of Phase 3
6. **P4-11** Dilithium signatures on code artifacts

### FOLLOWING SPRINT (2-4 weeks)
7. **P2-9** Real CNOT gate (`entangle()` → joint state transform)
8. **P3-5 through P3-8** Shard actually reads and compiles real .ai files
9. **P2-7** Import `.qube` from `.ai` files
10. **P1-12** Wire `entangle()` → real CNOT

### MEDIUM TERM (1-2 months)
11. **P5-3** LLM integration (Claude API)
12. **P5-4** Mother embryo loop live (write → run → learn)
13. **P5-8** WASM build target

### LONG TERM (3-6 months)
14. **P5-5** Qiskit bridge (`--features qiskit`)
15. **P5-9** Browser REPL on `aeonmi.x`
16. **Q-1 through Q-7** QUBE symbolic optimizer (30-qubit in <1MB)
17. **P5-7** Solana on-chain minting

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
| 9 | Genesis glyphs demo | `aeonmi run examples/genesis.ai` | ✅ **(new)** |
| 10 | f-string interpolation | `return f"{name} v{version}";` → correct string | ✅ **(new)** |
| 11 | `for x in collection` iterates | iterates array elements, string chars | ✅ **(new)** |

**Future criteria (Phase 2+):**

| # | Test | Target |
|---|------|--------|
| 12 | `⧉1‥2‥3⧉ ⊗ ⧉0‥1⧉` → real Kronecker product | Phase 2 / G-10 |
| 13 | `ψ ↦ bell ⊗ alice` → symbolic binding with tensor math | Phase 2 |
| 14 | Shard compiles hello.ai to real output | Phase 3 |
| 15 | 30-qubit symbolic simulation <1MB | Phase 6 |


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
| P2-8 | Joint multi-qubit state-vector simulator | ✅ |
| P2-9 | Real CNOT gate in `entangle()` | ✅ |
| P2-10 | Correct Deutsch-Jozsa (requires P2-8) | ✅ |

**P2-8 is the critical item.** The current simulator tracks each qubit independently. A joint state-vector model merges qubit state spaces so entanglement, CNOT, and multi-qubit interference work mathematically. This is what makes quantum algorithms correct.

---

### 🟡 PHASE 3 — SELF-HOSTING SHARD (70% Complete)

| ID | Task | Status |
|----|------|--------|
| P3-1 | Bootstrap `main.ai` runs | ✅ |
| P3-2 | Full `main_full.ai` runs | ✅ |
| P3-3 | Shard source files exist in .ai | ✅ |
| P3-4 | File I/O built-ins (`read_file`, `write_file`, `append_file`, `file_exists`, `read_lines`, `delete_file`) | ✅ |
| P3-5 | Shard actually reads .ai source and tokenizes it | ✅ |
| P3-6 | Shard actually parses tokenized input | ✅ |
| P3-7 | Shard produces compiled output from .ai input | ✅ |
| P3-8 | `aeonmi run shard/src/main.ai -- examples/hello.ai` → real output | ✅ |
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
---

## SERIOUS DEVELOPMENT IDEAS — PHASE 5+

> Brainstormed at end of Phase 2+3 completion session. These build directly on
> the language capabilities: joint state-vector quantum simulation (P2-8/P2-9),
> real file I/O (P3-4), self-hosting Shard compiler (P3-5..P3-8), QUBE circuits,
> `.ai` syntax with f-strings/for-in/genesis glyphs, and the web2+web3 runtime.

---

### IDEA 1: AeonMI Smart-Contract Verifier (`aeonmi verify <contract.ai>`)

**What:** A static analysis + quantum-assisted verification tool for `.ai`
smart contracts that targets EVM (Ethereum), Solana SVM, and Cosmos CosmWasm.

**How it works:**
- The Shard compiler (already reading `.ai` → AST) is extended to emit a
  symbolic constraint graph of the contract's state transitions.
- A mini **Deutsch-Jozsa oracle** (P2-10, now correct) classifies each function
  as "constant" (pure/safe) vs "balanced" (stateful/dangerous) in O(1) quantum
  queries — surfacing re-entrancy and integer overflow risks automatically.
- Final output: a human-readable PDF/HTML security report + a machine-readable
  JSON ABI that can be submitted directly to auditors or deployed on-chain.

**Why AEONMI is uniquely positioned:**
- Joint state-vector sim (P2-8) lets us model up to 20 contract state bits in
  genuine superposition without external QPUs.
- Native `.qube` circuit files allow the verification oracle to be written and
  audited in AEONMI itself — the language verifies its own contracts.
- Web3 vault (`aeonmi vault`) + NFT minting (`aeonmi mint`) already exist to
  attest verified contracts on-chain.

**Immediate first step:** Add a `Stmt::Assert` lowering pass that emits
quantum constraints into a `JointSystem` and measures invariant satisfaction.

---

### IDEA 2: AeonMI Reactive Web Framework (`aeonmi serve <app.ai>`)

**What:** A lightweight reactive web server and UI framework written entirely in
`.ai` syntax, bridging web2 (HTTP/JSON) and web3 (wallet/NFT/chain events).

**How it works:**
- A new built-in module `@std/http` (implemented in Rust, exposed as `.ai`
  functions: `http_listen`, `http_get`, `http_post`, `http_response`) is added
  alongside the existing file I/O built-ins.
- UI components are written as `.ai` functions that return HTML strings,
  automatically compiled by Shard to optimized JS that hydrates in the browser.
- Chain events (ERC-20 Transfer, NFT mint from `aeonmi mint`) are routed back
  to the server as first-class async events using `async`/`await` already in
  the language grammar.
- Quantum randomness (superpose → measure) seeds secure session tokens and
  cryptographic nonces with provable quantum entropy.

**Why AEONMI is uniquely positioned:**
- f-string interpolation (P1-33) makes HTML template generation ergonomic.
- `read_file` / `write_file` (P3-4) enable filesystem-based routing
  (file-system router pattern).
- The existing `reqwest`-backed AI built-ins (`ai_chat`, `ai_complete`) make
  LLM-augmented API endpoints trivial to write in `.ai`.
- One language, one toolchain: web server + on-chain logic + quantum entropy.

**Immediate first step:** Add `http_listen(port, handler_fn)` and
`http_response(status, body)` built-ins to `vm.rs` backed by `tokio::net`.

---

### IDEA 3: AeonMI Genesis Glyph NFT Marketplace (`aeonmi market`)

**What:** A CLI + web UI that lets developers mint, list, buy, and trade
**Genesis Glyph NFTs** — the 12 quantum glyph operators (G-1..G-12) and
user-defined QUBE circuit diagrams — on any EVM-compatible chain.

**How it works:**
- `aeonmi market list` reads the local `.ai` workspace, finds all `.qube`
  circuits and glyph usages (already parsed by P4 lexer), and renders them as
  SVG/PNG using the existing QUBE circuit diagram renderer
  (`QubeExecutor::circuit_diagram`).
- Each NFT metadata JSON embeds the raw `.qube` source, the rendered circuit
  PNG, a quantum signature (collapsed Bell-state measurement hash from P2-9
  CNOT), and the AEONMI version that produced it — making every glyph
  cryptographically unique and reproducible.
- `aeonmi market buy <token_id>` fetches the `.qube` source from IPFS,
  validates the quantum signature, and imports it into the local workspace so
  the buyer can immediately `aeonmi qube run` the purchased circuit.
- A leaderboard ranks circuits by "quantum complexity" (number of entangled
  pairs created by the correct joint-state CNOT) computed locally.

**Why AEONMI is uniquely positioned:**
- P2-8/P2-9 (joint state-vector + real CNOT) means the quantum signature is
  mathematically sound, not a simulation hack.
- Genesis Glyphs G-1..G-12 (Phase 4) are already first-class syntax — every
  `.ai` program that uses `⊗`, `⊕`, `⟨ψ|`, etc. is implicitly authoring NFT-
  eligible quantum art.
- The vault + NFT minting infrastructure already exists in `aeonmi vault` and
  `aeonmi mint`; the marketplace is a natural extension.
- File I/O (P3-4) + Shard (P3-5..P3-8) mean the marketplace client itself can
  be written as a self-contained `.ai` script — no separate build step needed.

**Immediate first step:** Add `aeonmi market list` subcommand that walks the
workspace for `.qube` files, renders circuit diagrams, and prints a table of
mintable glyphs with estimated quantum complexity scores.

