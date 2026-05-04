# Aeonmi

> AI-native language runtime in Rust: `.ai` + `.qube`, native VM execution, Mother AI loop, and Shard self-hosting workstream.

## Reality of the project (current)

Aeonmi is a real Rust implementation with this execution pipeline:

`Source (.ai) → Lexer → Parser → AST → IR Lowering → Tree-walk VM`

Current state:
- Native VM execution for `.ai` is live (`run`, `native`, and `.ai` `exec` path).
- QUBE (`.qube`) parser/executor is live (`qube run`, `qube check`).
- Mother AI embryo loop is live (`aeonmi mother`).
- Dashboard is live at `Aeonmi_Master/dashboard.py` (Flask app, port 7777).
- Glyph vault + identity flow is live (`aeonmi vault ...`).
- Mint metadata flow is live (`aeonmi mint ...`).
- Shard self-hosting compiler exists and is still in-progress.

---

## Quick start

```bash
# Build
cargo build --release

# Run an .ai file (native VM)
./target/release/aeonmi run examples/hello.ai

# Explicit native path
./target/release/aeonmi native examples/hello.ai

# QUBE
./target/release/aeonmi qube check examples/demo.qube
./target/release/aeonmi qube run examples/demo.qube --diagram
```

---

## Command highlights

```bash
# Compile/emit canonical .ai
aeonmi emit demo.ai -o output.ai

# Build command
aeonmi build src/main.ai --out output.ai

# Mother AI
aeonmi mother
aeonmi mother --file script.ai --creator Warren --verbose

# Extension-based execution
aeonmi exec program.ai
aeonmi exec circuit.qube
aeonmi exec script.py
aeonmi exec tool.rs

# Vault + mint
aeonmi vault init
aeonmi mint output.ai --out metadata.json
```

---

## Language snapshot

```aeonmi
⍝ hello.ai
◯ greet⟨name⟩ {
    return "Hello, " + name + "!";
}
print(greet("Aeonmi"));
```

Key glyphs:
- `◯` function keyword
- `⟨ ⟩` argument delimiters
- `⍝` comment marker
- `⧉`, `⊗`, `↦` symbolic/quantum operators supported in the language

Import support exists in VM execution (module resolution from source-relative paths).

---

## Known limitations (important)

- `arr[i]` indexing is unreliable; use `arr.slice(i, i+1).pop()` workaround.
- `%` modulo is not reliably wired in all paths; prefer arithmetic workaround when needed.
- `fmod()` naming can conflict with parser tokenization in current grammar.
- Object literal argument edge cases can fail in some parse contexts; `object()` + `set_key()` is the safer pattern.
- Shard self-hosting pipeline is present but not fully complete end-to-end.

---

## Docs and wiki

- Language spec: `docs/LANGUAGE_SPEC_CURRENT.md`
- Language guide: `docs/Aeonmi_Language_Guide.md`
- QUBE spec: `docs/QUBE_SPEC.md`
- Mother architecture: `MOTHER_AI_ARCHITECTURE.md`
- Project wiki pages in-repo: `wiki/`

Start with: `wiki/Home.md`

---

## Testing

```bash
# Rust tests
cargo test --all --quiet

# stdlib native checks
aeonmi native aeonmi_ai/stdlib/tests/test_math_builtins.ai
aeonmi native aeonmi_ai/stdlib/tests/test_string_builtins.ai
aeonmi native aeonmi_ai/stdlib/tests/test_collections_builtins.ai
```

Passing stdlib suites print `=== PASS ===`.

