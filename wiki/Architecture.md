# Architecture

## High-level

Aeonmi combines language runtime, AI loop, quantum DSL, and identity components in one Rust workspace.

## Major areas

- `src/core/` — lexer, parser, AST, IR, lowering, VM
- `src/commands/` — command handlers
- `src/mother/` — Mother AI loop and subsystems
- `src/qube/` — QUBE lexer/parser/AST/executor
- `src/glyph/` — vault, key derivation, glyph ceremony
- `src/mint/` / `src/core/mint.rs` — metadata mint flow
- `Aeonmi_Master/` — dashboard and supporting AI assets
- `aeonmi_ai/` — stdlib, shard, and `.ai` modules

## Entry points

- `src/main.rs` — CLI dispatch + unified runtime startup
- `src/cli.rs` — command definitions and arguments
- `src/commands/run.rs` — native interpreter path
- `Aeonmi_Master/dashboard.py` — web dashboard

