# HOST: Interface Specification
_Aeonmi-aeonmi02-selfhost — Last updated: 2026-04-05_

The `[HOST:]` built-in layer is the bridge between `.ai` programs and the Rust VM's
native capabilities. Self-hosting depends on this interface: once these builtins exist,
the Aeonmi compiler can be written in `.ai` itself.

---

## Design Principle

`.ai` code should never need to call `subprocess` or Python.
Instead, the Rust VM exposes controlled capabilities through `HOST:` prefixed builtins.
These are registered in `src/core/vm.rs` `call_builtin()`.

---

## Phase 1 — File System (implement first)

```ai
// Read a file → returns string content
let src = HOST:read_file("path/to/file.ai");

// Write a file
HOST:write_file("path/to/output.ai", content);

// Check file exists → returns bool  
let exists = HOST:file_exists("path/to/file");

// List directory → returns array of strings
let files = HOST:list_dir("src/");

// Delete a file
HOST:delete_file("path/to/old.tmp");
```

**Rust implementation location:** `src/core/vm.rs` → `call_builtin()` match arm `"HOST:read_file"` etc.

---

## Phase 2 — Process Execution

```ai
// Execute a shell command → returns {stdout, stderr, exit_code}
let result = HOST:exec("cargo build --release");

// Execute another .ai file → returns its output
let out = HOST:run_ai("tools/formatter.ai", args);
```

---

## Phase 3 — Compiler Pipeline Hooks

```ai
// Tokenize a string → returns token array
let tokens = HOST:lex(source_code);

// Parse tokens → returns AST (as JSON string)
let ast_json = HOST:parse(tokens);

// Emit bytecode from AST
let bytecode = HOST:emit(ast_json);
```

These allow incremental self-hosting: port lexer to .ai first, validate against `HOST:lex`,
then replace `HOST:lex` with the .ai implementation.

---

## Phase 4 — Network / HTTP

```ai
// HTTP GET → returns response body string
let body = HOST:http_get("https://api.anthropic.com/v1/messages", headers_json);

// HTTP POST → returns response body string
let resp = HOST:http_post(url, headers_json, body_json);
```

---

## Phase 5 — Mother Cognitive Bridge

```ai
// Emit a thought to Mother's inner_voice channel
HOST:emit_thought("I just compiled a file");

// Write to knowledge graph
HOST:kg_write(node_id, properties_json);

// Read from knowledge graph
let props = HOST:kg_read(node_id);
```

---

## Implementation Checklist

| Builtin              | Status       | Notes                         |
|----------------------|--------------|-------------------------------|
| `HOST:read_file`     | TODO         | Priority 1                    |
| `HOST:write_file`    | TODO         | Priority 1                    |
| `HOST:file_exists`   | TODO         | Priority 1                    |
| `HOST:list_dir`      | TODO         | Priority 1                    |
| `HOST:exec`          | TODO         | Priority 2                    |
| `HOST:run_ai`        | TODO         | Priority 2                    |
| `HOST:lex`           | TODO         | Priority 3 — compiler bridge  |
| `HOST:parse`         | TODO         | Priority 3                    |
| `HOST:emit`          | TODO         | Priority 3                    |
| `HOST:http_get`      | TODO         | Priority 4                    |
| `HOST:http_post`     | TODO         | Priority 4                    |
| `HOST:emit_thought`  | TODO         | Priority 5                    |
| `HOST:kg_write`      | TODO         | Priority 5                    |
| `HOST:kg_read`       | TODO         | Priority 5                    |
