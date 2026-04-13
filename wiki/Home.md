# Aeonmi Wiki

Aeonmi is an **AI-native programming language ecosystem** built primarily in Rust, with a custom symbolic language (`.ai`) and native execution pipeline.

## What this repository contains

- **Language runtime and compiler pipeline**: lexer, parser, AST, IR, VM
- **CLI tooling**: compile/run/emit/format/lint/edit/build/vault/qube workflows
- **Mother AI subsystem**: interactive orchestration and memory-driven agent loop
- **QUBE subsystem**: quantum circuit parsing/execution format (`.qube`)
- **Standard library**: executable `.ai` modules and tests
- **Dashboard and tooling**: Python-based dashboard and utility interfaces

## Core pipeline

```text
Source (.ai) → Lexer → Parser → AST → IR Lowering → Tree-walk VM
```

## Quick links

- [Getting Started](Getting-Started)
- [Architecture](Architecture)
- [CLI Reference](CLI-Reference)
- [Development Guide](Development-Guide)

## Key technologies

- **Rust** (core runtime, CLI, VM, subsystems)
- **Clap** (CLI interface)
- **Tokio** (async runtime for optional subsystems)
- **Serde / serde_json / toml** (config + data interchange)
- **Python** (dashboard and support tooling)
- **Feature-gated integrations** (quantum, AI providers, voice/holographic modules)

