# Dependency Map — Cross-Boundary Calls
_Aeonmi-aeonmi02-selfhost — Last updated: 2026-04-05_

This document maps every place where `.ai` code or the `Aeonmi_Master/` Python layer
calls into Rust, and every Rust module that calls out to Python or external systems.
This is the authoritative reference for what must be ported to remove the boundary.

---

## Layer Overview

```
┌──────────────────────────────────────────────────────────┐
│  Python Layer  (Aeonmi_Master/)                          │
│  dashboard.py · knowledge_store.py · build scripts       │
│  Flask HTTP ↕ subprocess ↕ file I/O                      │
├──────────────────────────────────────────────────────────┤
│  .ai Runtime Layer  (Aeonmi_Master/aeonmi_ai/)           │
│  shard/main.ai · agents · tasks                          │
│  Calls Rust via `Aeonmi.exe native <file>`               │
├──────────────────────────────────────────────────────────┤
│  Rust Core  (src/)                                       │
│  VM · Compiler · Mother · Quantum · CLI                  │
│  Calls Python via gui_bridge.rs + subprocess             │
└──────────────────────────────────────────────────────────┘
```

---

## Boundary A: Python → Rust (subprocess calls)

| Caller (Python)             | Command                            | Purpose                        | Port Decision |
|-----------------------------|------------------------------------|--------------------------------|---------------|
| `dashboard.py` route `/run` | `Aeonmi.exe native <file>`         | Execute .ai programs           | B — keep      |
| `dashboard.py` shell panel  | `Aeonmi.exe <subcommand>`          | CLI commands from UI           | B — keep      |
| `build_textbook_pdf.py`     | none (pure Python)                 | PDF generation                 | C — keep Python |

---

## Boundary B: Rust → Python (gui_bridge.rs)

| Caller (Rust)              | File                       | What it calls                        | Port Decision |
|----------------------------|----------------------------|--------------------------------------|---------------|
| `gui_bridge.rs`            | `src/gui_bridge.rs`        | Spawns Python dashboard process      | D — redesign  |
| `MotherAI.exe` main        | `src/bin/mother_ai.rs`     | Calls dashboard via HTTP             | A — port now  |
| `integration.rs`           | `src/integration.rs`       | PyO3 / subprocess interop            | A — port now  |

---

## Boundary C: .ai Builtins → Rust VM (src/core/vm.rs)

All `.ai` builtin calls are dispatched through `vm.rs` `call_builtin()`.
These are already native Rust — no boundary to cross.

| Builtin family | vm.rs handler | Status |
|----------------|---------------|--------|
| `print`, `input` | `call_builtin` line ~300 | Native |
| `sqrt`, `pow`, math | `call_builtin` | Native |
| `len`, `substr`, strings | `call_builtin` | Native |
| `quantum_*` | `src/qube/` | Native |
| `agent_*` | `src/mother/` | Native |

---

## Boundary D: Rust → External APIs (src/ai/)

| File               | External call              | Replaceability           |
|--------------------|----------------------------|--------------------------|
| `openrouter.rs`    | OpenRouter HTTPS           | Port to .ai `http_get`   |
| `claude.rs`        | Anthropic API HTTPS        | Port to .ai `http_get`   |
| `openai.rs`        | OpenAI API HTTPS           | Port to .ai `http_get`   |
| `grok.rs`          | xAI API HTTPS              | Port to .ai `http_get`   |

---

## Critical Path for Self-Hosting

To execute self-hosting (run Aeonmi compiler written in .ai), the minimum required:

1. `vm.rs` — must support `HOST:read_file`, `HOST:write_file`, `HOST:exec`
2. `compiler.rs` / `lexer.rs` / `parser.rs` — transliterate to .ai (Phase A)
3. `mother_core.rs` — Mother's loop, transliterate to .ai (Phase A)
4. `gui_bridge.rs` — replace with native .ai HTTP server (Phase D)
