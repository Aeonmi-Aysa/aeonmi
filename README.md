# Aeonmi

> **Built by AI for AI.** An AI-native programming language with a real Rust VM, 80+ native builtins, and a self-hosting compiler path.

[![Branch](https://img.shields.io/badge/branch-shard--v2--ecosystem-00ff88?style=flat-square)](https://github.com/Aeonmi-Aysa/aeonmi/tree/shard-v2-ecosystem)
[![Language](https://img.shields.io/badge/language-Rust-bf00ff?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Proprietary-ffbb00?style=flat-square)](./LICENSE)

---

## Documentation Hub

- [Wiki Home](./wiki/Home.md)
- [Getting Started](./wiki/Getting-Started.md)
- [Architecture](./wiki/Architecture.md)
- [CLI Reference](./wiki/CLI-Reference.md)
- [Development Guide](./wiki/Development-Guide.md)

---

## What Aeonmi actually is

Aeonmi is a symbolic programming language implemented in Rust. It has a complete pipeline:

```
Source (.ai)  →  Lexer  →  Parser  →  AST  →  IR Lowering  →  Tree-walk VM
```

The VM is real. The builtins are real. The test suite passes. The compiler is not a toy.

The language was designed with one premise: **AI systems should be able to write, read, and reason about code at maximum symbolic density.** Every operator is chosen to minimize tokens while maximizing semantic clarity.

This is not a proof-of-concept demo. This is an ongoing, honest build — documented exactly as it stands, including what doesn't work yet.

---

## What works right now

| Feature | Status |
|---|---|
| `Aeonmi.exe native <file.ai>` | ✅ Works — full Lexer→Parser→AST→IR→VM pipeline |
| 80+ native builtins | ✅ Works — math, string, array, object, JSON, functional |
| Function definitions `◯ fn⟨args⟩ { }` | ✅ Works |
| `let` bindings, `return`, `if/else`, `while` | ✅ Works |
| `map`, `filter`, `reduce`, `sort`, `unique`, `flatten` | ✅ Works |
| `object()`, `set_key()`, `get_key()`, `to_json()`, `parse_json()` | ✅ Works |
| `range()`, `enumerate()`, `zip()`, `any()`, `all()` | ✅ Works |
| Math constants `PI`, `E`, `TAU` | ✅ Works |
| String ops: `split`, `join`, `upper`, `lower`, `trim`, `replace`, `contains` | ✅ Works |
| Stdlib modules: math.ai, string.ai, collections.ai, io.ai, test.ai | ✅ Written, executable |
| Test suite (3 files, ~120 assertions) | ✅ All passing |
| `AeonmiStudio.exe` — IDE wrapper | ⚠️ Functional but pipeline panel is cosmetic animation |
| `AeonmiDemo.exe` — showcase | ⚠️ Display only, no live VM execution |
| QUBE quantum layer | 🔧 Partially implemented |
| Identity Vault | 🔧 Experimental |
| Import / module system | ❌ Not yet implemented |
| `arr[i]` subscript syntax | ❌ Broken — use `arr.slice(i, i+1).pop()` |
| `%` modulo operator | ❌ Not implemented as binary op |
| `fmod()` builtin | ❌ Parse conflict — use `floor(x/2)*2` for even check |

---

## Quick start

```bash
# Run the native VM on a .ai file
Aeonmi.exe native hello.ai

# Or from source (Rust)
cargo build --release
./target/release/Aeonmi native hello.ai
```

### Hello World

```aeonmi
⍝ hello.ai

◯ greet⟨name⟩ {
    let msg = "Hello, " + name + "!";
    return msg;
}

print(greet("Quantum World"));
```

Output:
```
Hello, Quantum World!
```

---

## Language syntax

### Functions

```aeonmi
◯ add⟨a, b⟩ {
    return a + b;
}

print(add(3, 4));   ⍝ → 7
```

- `◯` — function keyword (classical function)
- `⟨ ⟩` — argument delimiters
- `⍝` — comment

### Let bindings

```aeonmi
let x = 42;
let name = "Aeonmi";
let flag = true;
```

### Control flow

```aeonmi
if (x > 10) {
    print("big");
} else {
    print("small");
}

let i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}
```

### Arrays

```aeonmi
let nums = [1, 2, 3, 4, 5];
print(len(nums));              ⍝ → 5
print(nums.slice(0, 3));       ⍝ → [1, 2, 3]

⍝ !! IMPORTANT: arr[i] does NOT work — use slice + pop
let first = nums.slice(0, 1).pop();   ⍝ → 1
let third = nums.slice(2, 3).pop();   ⍝ → 3
```

### Higher-order functions

```aeonmi
◯ double⟨x⟩ { return x * 2; }
◯ even⟨x⟩   { return floor(x/2)*2 == x; }
◯ add⟨a, b⟩  { return a + b; }

let nums = [1, 2, 3, 4, 5];

print(map(nums, double));      ⍝ → [2, 4, 6, 8, 10]
print(filter(nums, even));     ⍝ → [2, 4]
print(reduce(nums, add));      ⍝ → 15
print(sort([3,1,4,1,5]));      ⍝ → [1, 1, 3, 4, 5]
print(unique([1,1,2,2,3]));    ⍝ → [1, 2, 3]
```

### Objects and JSON

```aeonmi
⍝ Object literals {} as function args fail — always use object() + set_key()
let agent = object();
agent = set_key(agent, "id",     "AGENT-001");
agent = set_key(agent, "status", "ONLINE");
agent = set_key(agent, "score",  99);

print(has_key(agent, "id"));     ⍝ → true
print(get_key(agent, "status")); ⍝ → ONLINE
print(keys(agent));              ⍝ → [id, status, score]
print(to_json(agent));           ⍝ → {"id":"AGENT-001","status":"ONLINE","score":99}

let parsed = parse_json("{\"x\":42}");
print(get_key(parsed, "x"));     ⍝ → 42
```

### Math

```aeonmi
print(PI);              ⍝ → 3.141592653589793
print(E);               ⍝ → 2.718281828459045
print(TAU);             ⍝ → 6.283185307179586

print(sqrt(144));       ⍝ → 12
print(pow(2, 10));      ⍝ → 1024
print(sin(PI / 2));     ⍝ → 1
print(abs(-7));         ⍝ → 7
print(clamp(1.9, 0, 1));⍝ → 1
print(lerp(0, 100, 0.5));⍝ → 50
print(min(3, 1, 4, 1)); ⍝ → 1
print(max(3, 1, 4, 1)); ⍝ → 4
```

---

## Genesis density operators

The symbolic core of Aeonmi — these operators keep large structures composable:

| Operator | Name | Purpose |
|---|---|---|
| `⧉` | Array Genesis | Construct dense arrays symbolically |
| `⟨⟩` | Slice/Index delimiters | Function argument notation |
| `…` | Spread | Expand symbolic structures |
| `⊗` | Tensor Product | Compose structures without expansion |
| `↦` | Binding/Projection | Assign meaning to symbolic expressions |

```aeonmi
⍝ Symbolic quantum state (Bell pair)
bell ← ⧉0.707‥0‥0‥0.707⧉
ψ ↦ bell ⊗ bell
```

These are real operators in the lexer and AST — not aspirational syntax.

---

## Known limitations (honest)

These are not future plans — they are current facts:

1. **`arr[i]` subscript is broken.** Always use `arr.slice(i, i+1).pop()` for element access. This is a parser/VM bug, not by design.

2. **No `%` modulo operator.** The `%` token exists but isn't wired as a binary operator. Use `floor(x/2)*2 == x` to check even numbers.

3. **`fmod()` has a parse conflict.** The lexer breaks `fmod` into `f` + `mod`. Use the `floor` workaround instead.

4. **Object literals `{...}` fail as function arguments.** `fn({"k": v})` causes a parse error. Always construct objects with `object()` + `set_key()`.

5. **No module/import system.** All code must be in one file, or you must manually concatenate .ai files before running.

6. **`exec` and `run` subcommands use a JavaScript transpiler**, not the Rust VM. Only `native` runs through the real pipeline. Use `Aeonmi.exe native <file>`.

7. **`AeonmiStudio.exe` pipeline panel is cosmetic.** The four stage indicators (Lexer → Parser → IR → VM) animate on a timer. They do not reflect actual VM stage completion. The code editor and execution are real; the visualization is approximate.

---

## Stdlib

Located in `aeonmi_ai/stdlib/`. All files run through `Aeonmi.exe native`.

| Module | Contents |
|---|---|
| `math.ai` | `degrees`, `radians`, `sign`, `hypot`, `log2`, `gcd`, `lcm`, `is_prime`, `factorial`, `fib`, `mean`, `variance`, `std_dev`, `normalize` |
| `string.ai` | `capitalize`, `words`, `slug`, `truncate`, `is_numeric`, `count_occurrences`, `wrap` |
| `collections.ai` | `first`, `last`, `tail`, `head`, `group_by`, `frequency`, `partition`, `chunk`, `zip_with`, `index_of`, `includes`, `rotate`, `dedupe_by`, `merge_objects` |
| `io.ai` | `read_json`, `write_json`, `read_lines`, `write_lines`, `read_csv`, `write_csv`, `log_to_file` |
| `test.ai` | `test()`, `run_tests()`, `expect_eq`, `expect_true`, `expect_approx`, `expect_in_range`, `expect_type` |

### Running stdlib tests

```bash
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_math_builtins.ai
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_string_builtins.ai
Aeonmi.exe native aeonmi_ai/stdlib/tests/test_collections_builtins.ai
```

All three print `=== PASS ===`.

---

## Architecture

```
src/
├── main.rs                  Entry point, CLI dispatch
├── core/
│   ├── lexer.rs             Tokenizer (Unicode, glyph operators)
│   ├── parser.rs            Recursive descent parser → AST
│   ├── ast.rs               AST node definitions
│   ├── ir.rs                IR lowering pass
│   └── vm.rs                Tree-walk interpreter + 80+ builtins
├── shard/                   Self-hosting compiler path
├── qube/                    Quantum-style symbolic layer (partial)
└── mother_ai/               Multi-agent coordinator (experimental)

aeonmi_ai/
├── stdlib/                  Standard library (.ai files)
│   ├── math.ai
│   ├── string.ai
│   ├── collections.ai
│   ├── io.ai
│   ├── test.ai
│   └── tests/               Test suite
└── demo/                    Showcase programs

tools/
├── aeonmi_studio/           IDE source (Python + tkinter)
│   └── aeonmi_studio.py
└── aeonmi_demo/             Animated showcase (Python + tkinter)
    └── aeonmi_demo.py
```

---

## Tools

### AeonmiStudio (IDE)

A 3-panel IDE built in Python/tkinter that wraps the native VM.

- **Left:** Code editor with syntax highlighting, line numbers, undo/redo
- **Middle:** VM pipeline visualization (Lexer → Parser → IR → VM)
- **Right:** Output terminal with colored results
- **Examples menu:** 7 pre-loaded programs

> ⚠️ The pipeline panel animates on a timer. It does not receive real stage events from the VM. The execution itself is real.

Source: `tools/aeonmi_studio/aeonmi_studio.py`  
Requires: `AeonmiStudio.exe` + `Aeonmi.exe` in the same directory.

### AeonmiDemo (Showcase)

An auto-cycling animated showcase with matrix rain background and typewriter code animation. For display/advertising. Does not execute real code.

Source: `tools/aeonmi_demo/aeonmi_demo.py`

---

## Building from source

```bash
# Standard build
cargo build --release

# The main binary
./target/release/Aeonmi native examples/hello.ai

# Run tests (Rust test suite)
cargo test

# Run .ai stdlib tests
./target/release/Aeonmi native aeonmi_ai/stdlib/tests/test_math_builtins.ai
```

---

## Contributing

This language was built by AI. Contributions from humans and AI systems are both welcome.

**High-value areas:**

1. **Fix `arr[i]` subscript** — `src/core/vm.rs`, `Value::Array` case in the subscript operator. This is the single most impactful fix available.

2. **Add `%` modulo operator** — wire `TokenKind::Percent` as a binary infix operator in `src/core/parser.rs`.

3. **Fix `fmod` parse conflict** — `src/core/lexer.rs`, check how `f` is tokenized before identifier characters.

4. **Module/import system** — `load "path/to/module.ai"` syntax. All bindings from that file become available.

5. **Real pipeline telemetry for AeonmiStudio** — the IDE currently animates on a timer. The VM could emit stage-complete signals (stdout markers or a JSON protocol) so the IDE can reflect actual pipeline state.

6. **QUBE completion** — the quantum-style symbolic layer in `src/qube/` is partially implemented.

All PRs must:
- Pass `cargo build --release` with zero errors
- Run all three stdlib test files without FAIL output
- Not introduce new warnings beyond the existing 211

See `CONTRIBUTING.md` for more.

---

## Philosophy

**One concept → one glyph.**

Aeonmi is designed so that AI systems can express programs at maximum symbolic density. A model reading Aeonmi code sees compact structure that maps directly to semantic operations — not verbose ceremony borrowed from human-legibility conventions.

> `◯ f⟨x⟩ { return x * 2; }` — 24 characters. One function, completely unambiguous.

The language does not try to look like Python or JavaScript. It tries to look like what an AI would design if it were optimizing for its own token budget.

---

## Status

**shard-v2-ecosystem** — active development branch.

This is an honest project. The VM works. The test suite passes. The stdlib is real. Several things are broken and documented above. The quantum and vault layers exist but are incomplete. The IDE tools are functional wrappers, not production software.

If you're looking for a complete, production-ready language: this isn't it yet.

If you're interested in the first serious attempt to build a language from first principles for AI-native execution — with a real Rust VM, working stdlib, and honest documentation — this is exactly that.

---

*Aeonmi — AI-native programming language — [github.com/Aeonmi-Aysa/aeonmi](https://github.com/Aeonmi-Aysa/aeonmi)*
