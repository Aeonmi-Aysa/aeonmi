# Language and VM

## Pipeline

`.ai` source is processed as:

`Lexer → Parser → AST → Lowering → IR → VM`

Execution entry is the native Rust VM path.

## Core syntax markers

- `◯` function declaration
- `⟨ ⟩` function argument delimiters
- `⍝` line comment marker
- Unicode symbolic operators are supported in lexer/parser/runtime paths

## Imports

Import resolution is implemented in the VM and resolves paths relative to the executing file.

## Runtime behavior notes

- Builtins cover standard math/string/array/object flows plus quantum-oriented operations.
- `run` and `native` both use native VM semantics for `.ai`.
- `exec` auto-selects runtime by extension.

## Known language/runtime caveats

- Prefer `slice(i, i+1).pop()` over direct `arr[i]` in sensitive code paths.
- `%` modulo and `fmod()` can be problematic depending on parser path.
- Object literal argument parsing may fail in specific contexts; build objects incrementally as needed.

