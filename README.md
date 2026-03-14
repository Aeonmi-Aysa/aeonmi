# Aeonmi

Aeonmi is an experimental symbolic programming language exploring AI-native execution, quantum-style composition, and self-hosting compiler design.

**Status:** early-stage research language under active development.

Aeonmi combines:

* a glyph-oriented surface language (`.ai`)
* a symbolic composition model for dense data structures
* a Rust runtime
* a self-hosting compiler path through Shard
* a symbolic optimization layer through QUBE
* an experimental identity and vault layer

---

## Core Idea

Aeonmi is built around a compact symbolic data algebra.

### Core glyph primitives

* `⧉` Array Genesis
* `⟨⟩` Slice / Index
* `…` Spread
* `⊗` Tensor Product
* `↦` Binding / Projection

These operators are designed to keep large structures symbolic and composable for as long as possible before expansion or evaluation.

---

## Example

```aeonmi
bell ← ⧉0.707‥0‥0‥0.707⧉
ψ ↦ bell ⊗ bell
```

This expresses a symbolic tensor composition without requiring immediate full expansion.

---

## Architecture

```text
             Aeonmi Language (.ai)
                      │
                      ▼
                   Shard
           self-hosting compiler
                      │
                      ▼
              Titan Runtime (Rust)
                      │
        ┌─────────────┼─────────────┐
        ▼                           ▼
    Glyph Runtime               QUBE Engine
   symbolic algebra          quantum circuits
                      │
                      ▼
                Identity Vault
```

---

## Project Components

### Aeonmi Language

The main surface language for writing symbolic programs with glyph-based syntax and dense structural expressions.

### Shard

The self-hosting compiler path for Aeonmi. This is where the language progressively moves toward compiling and interpreting itself.

### Titan Runtime

The Rust-based execution layer responsible for memory handling, execution, and runtime support for symbolic structures.

### QUBE

A symbolic optimizer and quantum-style execution layer intended to compress, rewrite, and evolve dense structures into more compact or efficient forms.

### Identity Vault

An experimental layer for persistent symbolic identity, cryptographic binding, and system-level execution context.

---

## Documentation

* `docs/architecture.md`
* `docs/language_spec.md`
* `docs/glyph_algebra.md`
* `docs/grammar_qube.md`

---

## Current Focus

Current development is centered on:

* symbolic array construction
* slice and spread semantics
* tensor composition
* binding / projection behavior
* QUBE grammar and runtime direction
* Shard integration
* runtime stabilization
* documentation cleanup

---

## Philosophy

Aeonmi follows a simple rule:

**one concept → one glyph**

Design priorities:

* composition over mutation
* minimal syntax
* symbolic density
* mathematically meaningful operators
* AI-friendly structure

---

## Running Aeonmi

Example commands:

```bash
aeonmi --version
aeonmi run examples/hello.ai
aeonmi run shard/src/main.ai
aeonmi qube run examples/demo.qube
aeonmi vault init
```

---

## QUBE

QUBE is Aeonmi’s symbolic quantum-style layer.

It is intended to support:

* state declaration
* gate application
* tensor composition
* collapse / measurement
* assertions
* future optimization and diagram output

See:

* `docs/grammar_qube.md`
* `Q.U.B.E.md`

---

## Roadmap

See:

* `AEONMI_LANGUAGE_ROADMAP.md`
* `Q.U.B.E.md`
* `test_suite.md`

---

## Positioning

Aeonmi is best understood today as:

**an experimental symbolic programming language for AI-native and quantum-style computation**

It is not yet production-ready, but it is a real, working language project under active development.

---

## Repository Notes

This repository contains active research, runtime experiments, and evolving compiler infrastructure.

Some subsystems are stable enough to demonstrate; others remain exploratory and are still being integrated more deeply into the runtime and compiler path.

---

## Long-Term Direction

Aeonmi is being developed toward a system that can support:

* dense symbolic data representation
* quantum-style composition and execution
* AI-native program structure
* deeper self-hosting through Shard
* stronger runtime identity and vault capabilities

The project is intentionally being built in public and documented honestly as features become real.

---

## Root Directory

This section describes the top-level contents of the repository.

### Directories

| Directory | Description |
|-----------|-------------|
| `src/` | Main Rust source code — lexer, parser, VM, bytecode compiler, quantum runtime, glyph system, vault, TUI/CLI, and all language subsystems |
| `tests/` | Integration and unit test files (Rust `.rs` test modules) |
| `examples/` | Example `.ai` and `.qube` programs covering language features, quantum algorithms, and demos |
| `docs/` | Language and architecture documentation — spec, grammar, glyph algebra, and getting-started guides |
| `shard/` | The Shard self-hosting compiler — Aeonmi compiler written in Aeonmi (Phase 3) |
| `scripts/` | Utility scripts — git hooks, large-file scanners |
| `assets/` | Project assets (application icon) |
| `gui/` | GUI and web interface — Tauri bridge, quantum IDE HTML, static assets |
| `mother_ai/` | Mother AI module — standalone AI consciousness layer with its own `main.rs` entry point |
| `titan_libraries/` | Titan algorithm library — quantum math, linear algebra, and advanced algorithms written in `.ai` |

### Core Project Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Rust workspace manifest — package metadata, feature flags, and all dependencies |
| `Cargo.lock` | Dependency lockfile (pinned versions) |
| `build.rs` | Rust build script — Windows resource embedding and build-time configuration |
| `package.json` | Node.js package manifest used by the JS compilation backend |
| `package-lock.json` | Node.js dependency lockfile |

### Documentation

| File | Description |
|------|-------------|
| `README.md` | This file — project overview, architecture, and quick-start guide |
| `LICENSE` | Project license |
| `CONTRIBUTING.md` | Contribution guidelines and repository hygiene rules |
| `SECURITY.md` | Security policy and vulnerability reporting procedure |
| `BUILD_STATUS.md` | Current build and feature completion status across all phases |
| `AEONMI_LANGUAGE_ROADMAP.md` | Language development roadmap with factual phase tracking |
| `Q.U.B.E.md` | QUBE tutorial and comprehensive language guide |
| `QUANTUM_ROADMAP_2.0.md` | Quantum feature roadmap version 2.0 |
| `QUANTUM_GAPS_ANALYSIS.md` | Analysis of current gaps in the quantum implementation |
| `SHARD_STRATEGY.md` | Strategic plan for the Shard self-hosting compiler |
| `SHARD_STATUS.md` | Current implementation status of the Shard compiler |
| `MOTHER_AI_ARCHITECTURE.md` | Architecture documentation for the Mother AI system |
| `MOTHER_AI_STATUS.md` | Current implementation status of the Mother AI module |
| `TITAN_INTEGRATION_STATUS.md` | Integration status of the Titan algorithm libraries |
| `SEAMLESS_INTEGRATION_CONFIRMED.md` | Record of confirmed cross-subsystem integration milestones |
| `NEW_SESSION_PROMPT.md` | Session startup prompt for continuing development in a new AI session |
| `array_genesis.md` | Documentation for the Array Genesis (`⧉`) glyph and symbolic arrays |
| `roadmap.md` | High-level project roadmap |
| `test_suite.md` | Test suite overview and coverage summary |
| `Aeonmi_Glyph_Identity_Spec_and_Manual.odg` | Glyph identity specification and visual design manual (LibreOffice Draw) |

### Build and Run Scripts

| File | Description |
|------|-------------|
| `build_unified.ps1` | Unified PowerShell build script for all project components |
| `build_windows.ps1` | Windows-specific build script |
| `clean_build.ps1` | Clean build script — removes artifacts before rebuilding |
| `apply_fixes.ps1` | Script for applying batched source fixes |
| `quantum_demo.ps1` | PowerShell script for running quantum demos |
| `run_tests.ps1` | Test runner script |
| `env_qiskit.cmd` | Windows command file for setting up the Qiskit Python environment |

### Pre-built Binaries

| File | Description |
|------|-------------|
| `Aeonmi.exe` | Pre-built Windows executable for the main Aeonmi runtime |
| `MotherAI.exe` | Pre-built Windows executable for the standalone Mother AI binary |

### Example and Test Files at Root

The root directory also contains a number of `.ai`, `.aeon`, and `.aeonmi` source files used for quick testing and demonstration, as well as generated output files (`out.ai`, `out.js`, `test_results.txt`, `build_output.txt`, `warnings.txt`). These are workspace artifacts from active development.

---

## License

See `LICENSE`.
