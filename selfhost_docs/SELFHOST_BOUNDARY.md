# Self-Hosting Boundary — Rules of the Road
_Aeonmi-aeonmi02-selfhost — Last updated: 2026-04-07_

This document defines the rules that govern what can and cannot be changed in this
repository as self-hosting progresses. These rules exist to protect the baseline and
ensure that when the Aeonmi compiler can compile itself, we know it's real.

---

## The Core Rule

> **The Rust VM (`src/core/vm.rs`) is the ground truth.**
> It may only be modified to ADD `HOST:` builtins or fix bugs.
> It may NOT be modified to make a ported .ai file "work around" a limitation.
> The .ai port must be correct, not the VM bent to fit it.

---

## What "Self-Hosting" Means Here

Self-hosting is achieved when:
```
Aeonmi.exe native Aeonmi_Master/aeonmi_ai/compiler/main.ai input.ai
```
produces the same bytecode output as:
```
Aeonmi.exe compile input.ai
```

That is the milestone. Everything else is scaffolding toward it.

---

## Rules for .ai Ports

1. **No cheating with HOST: builtins.** You may not add a `HOST:compile_expression`
   builtin that calls back into the Rust compiler. The .ai port must implement the logic.

2. **Tests first.** Before porting a module, write a `.ai` test file that exercises
   its behavior using the Rust version as oracle. The .ai port passes when tests pass.

3. **Parallel operation.** Keep the Rust version running alongside the .ai port until
   the port passes all tests. Do not delete the Rust version until verified.

4. **No modifications to aeonmi01.** `Aeonmi-aeonmi01` is the reference implementation.
   Never modify it for the sake of self-hosting work. Port decisions are tracked in
   `port_decisions.md` and apply only to this repository.

---

## Rules for the Python Layer

1. `dashboard.py` and `knowledge_store.py` may be modified freely — they are
   Category C/D (see port_decisions.md) and not part of the self-hosting critical path.

2. Python dependencies are a debt to be paid off, not a foundation to build on.
   Do not add new Python libraries without documenting them in this file.

   **Current Python dependencies:**
   - Flask (HTTP server)
   - anthropic (API client)
   - pdfplumber (PDF ingestion)
   - reportlab (PDF generation)
   - python-dotenv (env loading)

---

## Rules for the Rust Build

1. This project builds to `target/` (relative, `force = true` in `.cargo/config.toml`).
   Never change `CARGO_TARGET_DIR` to point outside this project directory.

2. `MotherAI.exe`, `Aeonmi.exe`, `aeonmi_project.exe` must all build successfully
   before any self-hosting port is considered "ready for testing."

3. The baseline build log is at `selfhost_docs/baseline_build.log`.
   Update it after each significant build.

---

## Milestone Checklist

- [x] Baseline build passes — all binaries built (`2026-04-05`)
- [x] Runtime verification — `shard/main.ai` 6/6 PASS (`2026-04-05`)
- [x] `HOST:read_file`, `HOST:write_file`, `HOST:file_exists`, `list_dir`, `delete_file`, `make_dir` in vm.rs (`2026-04-07`)
- [x] `HOST:shell_exec`, `run_ai`, `get_env`, `set_env` implemented in vm.rs (`2026-04-07`)
- [x] Lexer ported to .ai — 18/18 tests pass (`2026-04-07`) — `Aeonmi_Master/aeonmi_ai/compiler/lexer.ai`
- [ ] Parser ported to .ai — tests pass
- [ ] Compiler ported to .ai — produces valid bytecode
- [ ] **SELF-HOSTING ACHIEVED** — .ai compiler produces same output as Rust compiler
- [ ] Mother cognitive loop ported to .ai
- [ ] AI provider calls ported to .ai (no Python HTTP)
- [ ] Dashboard replaced with .ai HTTP server (no Python runtime)
- [ ] **FULL PYTHON ELIMINATION**
