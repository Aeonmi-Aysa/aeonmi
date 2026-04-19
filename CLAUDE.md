# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Aeonmi is an **AI-native programming language** implemented in Rust. It has a custom syntax using Unicode glyphs (◯ ⟨⟩ ⧉ ⊗ ↦ …) and a tree-walk VM that executes `.ai` source files directly. The pipeline is:

```
Source (.ai) → Lexer → Parser → AST → IR Lowering → Tree-walk VM
```

The project is a multi-binary Rust workspace: the main `aeonmi` CLI, a `mother_ai` consciousness binary, and a `shard` self-hosting compiler (Phase 3, in progress). **There is no JavaScript runtime dependency** — all execution goes through the native Rust VM.

---

## Build & Run

```bash
# Standard release build
cargo build --release

# PowerShell build variants
.\build_unified.ps1 -Release -BuildType full-suite   # everything
.\build_unified.ps1 -Release -BuildType mother-ai
.\build_unified.ps1 -Release -BuildType titan-only
.\build_unified.ps1 -Release -BuildType core
.\clean_build.ps1                                     # clean then rebuild
```

All execution paths use the **native Rust VM** — there is no JS/Node dependency:

```bash
# All of these run through the native VM:
Aeonmi.exe native examples/hello.ai     # explicit native subcommand
Aeonmi.exe run examples/hello.ai        # also native (JS path removed)
Aeonmi.exe exec examples/hello.ai       # also native

# Other commands:
aeonmi emit demo.ai -o out.ai           # emit canonical .ai form (default)
aeonmi build src/main.ai                # compile to .ai output
aeonmi qube run circuit.qube --diagram  # run .qube quantum circuit
aeonmi qube check circuit.qube          # validate .qube file
aeonmi mother [--file f.ai] [--creator Warren] [--verbose]
aeonmi vault init                       # initialize identity vault
aeonmi mint file.ai                     # generate NFT metadata JSON
aeonmi exec file.py                     # run Python file (pass-through)
aeonmi exec file.rs                     # compile+run Rust file (temp rustc)
```

No args = interactive Shard shell with glyph boot ceremony.

---

## Dashboard

The unified dashboard (`Aeonmi_Master/dashboard.py`) is a Flask web app providing:
- **Mother AI conversation** as the primary interface (center panel)
- **File explorer** with create / edit / run / delete (left panel)
- **Shard canvas** — output, action queue, build controls (right panel)

```bash
python Aeonmi_Master/dashboard.py   # starts on http://localhost:7777
```

Dashboard chat commands: `run <file>` · `compile <file>` · `qube <file>` · `status` · `actions` · `plan <action>` · `next` · `log`

**Agent panel**: Right panel shows all 8 agents (oracle, hype, closer, conductor, devil, decide, action, plan) as clickable buttons. Click to run via `/api/agent`. API: `POST /api/agent {"agent":"oracle"}`.

**Mother memory**: Persistent across restarts via `Aeonmi_Master/genesis.json`. Loaded on startup, saved after each chat response. Stores interaction count, key facts, recent actions. Add a fact: `POST /api/memory {"fact":"..."}`.

**Multi-turn history**: Mother's Anthropic API calls now pass the full conversation history (last 40 entries) as alternating user/assistant messages.

Set `ANTHROPIC_API_KEY`, `OPENROUTER_API_KEY`, or `OPENAI_API_KEY` to enable full AI responses.

---

## Testing

```bash
# Rust unit tests
cargo test --all --quiet

# Single stdlib test via native VM
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_math_builtins.ai
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_string_builtins.ai
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_collections_builtins.ai
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_native_ops.ai

# PowerShell test runner
.\run_tests.ps1
```

Passing stdlib tests print `=== PASS ===`.

**Pre-existing build errors** (not regressions): 3 errors in `src/core/titan/arc_bridge.rs` referencing `quantum_circuits` and `quantum_operations` modules that don't exist yet. These are from incomplete Titan bridge code and do not affect CLI functionality.

---

## Architecture

### Core Compiler Pipeline (`src/core/`)

| File | Size | Role |
|------|------|------|
| `src/core/lexer.rs` | ~37k lines | Unicode tokenizer; handles glyph operators |
| `src/core/parser.rs` | ~84k lines | Recursive descent → AST; handles quantum syntax |
| `src/core/ast.rs` | ~16k lines | AST node definitions |
| `src/core/lowering.rs` | ~30k lines | AST → IR |
| `src/core/ir.rs` | ~3k lines | IR definitions |
| `src/core/vm.rs` | ~110k lines | Tree-walk interpreter with 80+ builtins |

### Other Major Components

- **`src/commands/run.rs`** — `run_native()` is the single execution entry point. `main_with_opts()` delegates to it unconditionally.
- **`src/glyph/`** — Glyph Identity System: 256-bit MGK (Argon2id), 60-second UGST tokens, XChaCha20-Poly1305 vault, boot ceremony.
- **`src/mother/embryo_loop.rs`** — Mother AI REPL: action queue (`pending_actions`), action log, AI provider routing, consciousness updates. Commands: `status` · `emotion` · `evolve` · `actions` · `next` · `log` · `plan` · `exit`.
- **`src/qube/`** — QUBE format: lexer/parser/executor for `.qube` quantum circuit files.
- **`Aeonmi_Master/aeonmi_ai/shard/`** — Self-hosting shard compiler in `.ai`: `token.ai`, `lexer.ai`, `parser.ai`, `codegen.ai`, `main.ai`. Run via `aeonmi native Aeonmi_Master/aeonmi_ai/shard/main.ai`.
- **`Aeonmi_Master/aeonmi_ai/mother/`** — Mother AI `.ai` modules: `core.ai`, `memory.ai`, `journal.ai`, `rules.ai`, `maintenance.ai`.
- **`src/core/titan/`** — Titan quantum algorithm library.
- **`aeonmi_ai/stdlib/`** — Standard library: `math.ai`, `string.ai`, `collections.ai`, `io.ai`, `test.ai`.

### Entry Points

- `src/main.rs` — CLI entry, glyph startup, all command dispatch
- `src/cli.rs` — CLI argument structure (`EmitKind` default is now `Ai`, `Node` command removed)
- `src/mother/embryo_loop.rs` — Mother AI loop with action queue
- `Aeonmi_Master/dashboard.py` — Unified web dashboard

### Feature Flags (in `Cargo.toml`)

`full-suite` (default) enables everything. Key flags: `quantum`, `mother-ai`, `titan-libraries`, `holographic` (3D/wgpu), `voice` (rodio), `minimal` (core only).

---

## `.ai` Language Syntax

```
◯ function_name⟨arg1, arg2⟩ {
    ⍝ Comment
    let var = expression;
    return result;
}
```

| Glyph | Meaning |
|-------|---------|
| `◯` | Function keyword |
| `⟨ ⟩` | Argument delimiters |
| `⍝` | Comment marker |
| `⧉` | Array genesis (dense construction) |
| `⊗` | Tensor product |
| `↦` | Binding/projection |
| `\|0⟩ \|1⟩` | Qubit literals |

**Known VM limitations** (bugs — work around them):

1. `arr[i]` subscript is broken → use `arr.slice(i, i+1).pop()`
2. No `%` modulo → use `floor(x/2)*2 == x` or floor arithmetic
3. `fmod()` has a parse conflict with `f` prefix
4. Object literals `{...}` fail as function args → use `object()` + `set_key()`
5. ~~No module/import system~~ — **RESOLVED**: VM has full import support. `resolve_import()` in `vm.rs` reads, lexes, parses, and loads imported `.ai` files with no function-count limit. `main.ai` runs natively.

---

## Key Docs

- `docs/LANGUAGE_SPEC_CURRENT.md` — Formal language spec
- `docs/Aeonmi_Language_Guide.md` — Practical language reference
- `MOTHER_AI_ARCHITECTURE.md` — Mother AI 3-tier consciousness design
- `SHARD_STATUS.md` — Self-hosting compiler progress
- `ROADMAP_UPDATED.md` — Phase-by-phase completion status
- `BUILD_STATUS.md` — Build phase checklist
- `Aeonmi_Master/MOTHER_BRIEF.md` — Mother AI assignment brief
- `Aeonmi_Master/aeonmi_ai/AEONMI_MASTER_PROMPT.md` — Language architecture master prompt
