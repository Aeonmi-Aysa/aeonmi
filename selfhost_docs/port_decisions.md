# Port Decisions
_Aeonmi-aeonmi02-selfhost — Last updated: 2026-04-05_

Each file/module classified by migration strategy.
**A** = Port to .ai now | **B** = Keep Rust (performance) | **C** = Keep Python | **D** = Redesign first

---

## Classification Key

| Class | Meaning                                                               |
|-------|-----------------------------------------------------------------------|
| A     | Port to .ai — this is the self-hosting goal, do it phase by phase     |
| B     | Keep in Rust — performance-critical, FFI, or hardware-facing          |
| C     | Keep in Python — tooling, PDF build, not runtime-critical             |
| D     | Redesign — current implementation has architectural debt              |

---

## src/core/ — Compiler & VM

| File                  | Class | Reasoning                                                  |
|-----------------------|-------|------------------------------------------------------------|
| `lexer.rs`            | A     | Port first — pure text transformation, ideal .ai target    |
| `parser.rs`           | A     | Port second — recursive descent, maps well to .ai functions|
| `ast.rs`              | A     | Data structures only, define as .ai records                |
| `compiler.rs`         | A     | Port third — transforms AST to IR                          |
| `ir.rs`               | A     | IR data structures                                         |
| `bytecode.rs`         | B     | Binary format, keep Rust for byte-level ops                |
| `vm.rs`               | B     | Keep Rust — execution engine must be native                |
| `vm_bytecode.rs`      | B     | Keep Rust — bytecode interpreter                           |
| `semantic_analyzer.rs`| A     | Port alongside compiler                                    |
| `symbols.rs`          | A     | Symbol table — pure data management                        |
| `types.rs`            | A     | Type system definitions                                    |
| `scope_map.rs`        | A     | Scope tracking — pure logic                                |
| `formatter.rs`        | A     | Code formatter — pure text                                 |
| `diagnostics.rs`      | A     | Error reporting                                            |
| `error.rs`            | A     | Error types                                                |
| `mother_core.rs`      | A     | Mother's runtime loop — port to .ai (core self-hosting)    |
| `incremental.rs`      | B     | Incremental compilation cache — keep Rust for perf         |
| `code_generator.rs`   | A     | Code gen — after compiler                                  |
| `lowering.rs`         | A     | IR lowering                                                |

---

## src/mother/ — Cognitive Systems

| File                  | Class | Reasoning                                                  |
|-----------------------|-------|------------------------------------------------------------|
| `embryo_loop.rs`      | A     | Mother's core cognitive loop — the self-hosting beachhead  |
| `emotional_core.rs`   | A     | Emotional state tracking — pure logic                      |
| `inner_voice.rs`      | A     | Inner voice stream — event emitter pattern                 |
| `knowledge_graph.rs`  | A     | Knowledge graph — port after HOST:kg_* builtins exist      |
| `language_evolution.rs`| A    | Language self-modification — must be .ai to be authentic   |
| `neural.rs`           | D     | Not yet wired (Phase 5) — redesign when ready              |
| `quantum_attention.rs`| D     | Quantum-cognitive bridge — redesign with Qiskit clarity    |
| `quantum_core.rs`     | B     | Quantum simulation math — keep Rust for precision          |

---

## src/ai/ — AI Provider Integrations

| File             | Class | Reasoning                                             |
|------------------|-------|-------------------------------------------------------|
| `openrouter.rs`  | A     | Simple HTTP POST — port to .ai using HOST:http_post   |
| `claude.rs`      | A     | Same                                                  |
| `openai.rs`      | A     | Same                                                  |
| `grok.rs`        | A     | Same                                                  |
| `deepseek.rs`    | A     | Same                                                  |
| `perplexity.rs`  | A     | Same                                                  |
| `copilot.rs`     | A     | Same                                                  |
| `ai_provider.rs` | A     | Provider abstraction — port as .ai trait/interface    |

---

## src/qube/ — Quantum Engine

| File                  | Class | Reasoning                                            |
|-----------------------|-------|------------------------------------------------------|
| `executor.rs`         | B     | Quantum simulation — keep Rust for math perf         |
| `parser.rs`           | A     | Qube syntax parser — port alongside main parser      |
| `ast.rs`              | B     | Qube AST — tightly coupled to executor               |
| `circuit_builder.rs`  | B     | Keep — performance-critical quantum ops              |
| `circuit_compiler.rs` | A     | Circuit compilation logic — port                     |

---

## Aeonmi_Master/ — Python Layer

| File                    | Class | Reasoning                                          |
|-------------------------|-------|----------------------------------------------------|
| `dashboard.py`          | D     | Redesign — replace Flask with .ai HTTP server      |
| `knowledge_store.py`    | C     | Keep Python for now — replace later with .ai store |
| `build_textbook_pdf.py` | C     | Keep Python — ReportLab, not worth porting         |
| `genesis.json`          | A     | Port to .ai config format                          |

---

## Priority Order for Porting

1. `HOST:` builtins in `vm.rs` (Phase 1-2 of host_interface.md)
2. `lexer.rs` → `.ai`
3. `parser.rs` → `.ai`  
4. `compiler.rs` → `.ai` (milestone: Aeonmi compiles itself)
5. `embryo_loop.rs` → `.ai` (milestone: Mother runs herself)
6. AI provider files → `.ai` (drop Python HTTP dependency)
7. `dashboard.py` → `.ai` HTTP server (drop Python entirely)
