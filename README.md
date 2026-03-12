# Aeonmi

Aeonmi is an experimental symbolic programming language exploring AI-native execution, quantum-style composition, and self-hosting compiler design.

**Status:** early-stage research language under active development.

Aeonmi combines:

- a glyph-oriented surface language (`.ai`)
- a symbolic composition model for dense data structures
- a Rust runtime
- a self-hosting compiler path through Shard
- a symbolic optimization layer through QUBE
- an experimental identity and vault layer

---

## Core Idea

Aeonmi is built around a compact symbolic data algebra.

### Core glyph primitives

- `⧉` Array Genesis
- `⟨⟩` Slice / Index
- `…` Spread
- `⊗` Tensor Product
- `↦` Binding / Projection

These operators are designed to keep large structures symbolic and composable for as long as possible before expansion or evaluation.

---

## Example

```aeonmi
bell ← ⧉0.707‥0‥0‥0.707⧉
ψ ↦ bell ⊗ bell

This expresses a symbolic tensor composition without requiring immediate full expansion.

Architecture
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
Project Components
Aeonmi Language

The main surface language for writing symbolic programs with glyph-based syntax and dense structural expressions.

Shard

The self-hosting compiler path for Aeonmi. This is where the language progressively moves toward compiling and interpreting itself.

Titan Runtime

The Rust-based execution layer responsible for memory handling, execution, and runtime support for symbolic structures.

QUBE

A symbolic optimizer and quantum-style execution layer intended to compress, rewrite, and evolve dense structures into more compact or efficient forms.

Identity Vault

An experimental layer for persistent symbolic identity, cryptographic binding, and system-level execution context.

Documentation

docs/architecture.md

docs/language_spec.md

docs/glyph_algebra.md

docs/grammar_qube.md

Current Focus

Current development is centered on:

symbolic array construction

slice and spread semantics

tensor composition

binding / projection behavior

QUBE grammar and runtime direction

Shard integration

runtime stabilization

documentation cleanup

Philosophy

Aeonmi follows a simple rule:

one concept → one glyph

Design priorities:

composition over mutation

minimal syntax

symbolic density

mathematically meaningful operators

AI-friendly structure

Running Aeonmi

Example commands:

aeonmi --version
aeonmi run examples/hello.ai
aeonmi run shard/src/main.ai
aeonmi qube run examples/demo.qube
aeonmi vault init
QUBE

QUBE is Aeonmi’s symbolic quantum-style layer.

It is intended to support:

state declaration

gate application

tensor composition

collapse / measurement

assertions

future optimization and diagram output

See:

docs/grammar_qube.md

Q.U.B.E.md

Roadmap

See:

AEONMI_LANGUAGE_ROADMAP.md

Q.U.B.E.md

test_suite.md

Positioning

Aeonmi is best understood today as:

an experimental symbolic programming language for AI-native and quantum-style computation

It is not yet production-ready, but it is a real, working language project under active development.

Repository Notes

This repository contains active research, runtime experiments, and evolving compiler infrastructure.

Some subsystems are stable enough to demonstrate; others remain exploratory and are still being integrated more deeply into the runtime and compiler path.

Long-Term Direction

Aeonmi is being developed toward a system that can support:

dense symbolic data representation

quantum-style composition and execution

AI-native program structure

deeper self-hosting through Shard

stronger runtime identity and vault capabilities

The project is intentionally being built in public and documented honestly as features become real.
