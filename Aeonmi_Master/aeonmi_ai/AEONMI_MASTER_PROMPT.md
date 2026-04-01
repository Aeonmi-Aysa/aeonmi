# AEONMI Master Prompt — AI-Native Language Ecosystem

## Overview
Aeonmi is an AI-native programming language with quantum-first syntax.
All phase code lives in `.ai` files under `aeonmi_ai/`.
No new Rust. Every file must parse and execute through the existing pipeline:

```
.ai → Lexer → Parser → AST → Lowering → IR → VM (native, AEONMI_NATIVE=1)
.ai → Lexer → Parser → AST → CodeGenerator → JS → Node.js (default)
```

## Genesis Density Operators
These are the core symbolic operators used throughout Aeonmi code.
They are NOT decorative — each maps to a real token in the lexer/parser:

| Symbol | Name | TokenKind | Semantic |
|--------|------|-----------|----------|
| `⟨⟩` | Quantum brackets | QuantumBracketOpen/Close | Variable scoping |
| `←` | Quantum bind | QuantumBind | Classical assignment |
| `⊗` | Tensor product | QuantumTensor | Tensor/array binding |
| `∈` | Membership | QuantumIn | Superposition binding |
| `≈` | Approximation | QuantumApprox | Approximate binding |
| `⊕` | XOR / else | QuantumXor | Else branch in prob |
| `⊖` | Probability | QuantumOr | Probability branch |
| `∇` | Gradient | QuantumGradient | Gradient ops |
| `⪰` | GEQ | QuantumGeq | Loop threshold |
| `⪯` | LEQ | QuantumLeq | Comparison |
| `⇒` | Implies | QuantumImplies | Branch arrow |
| `⟲` | Loop | QuantumLoop | Quantum loop |
| `◯` | Classical func | ClassicalFunc | Classical function marker |
| `⊙` | Quantum func | QuantumFunc | Quantum function marker |
| `🧠` | AI func | AIFunc | Neural function marker |
| `⚡` | Attempt | Attempt | Quantum try |
| `⧉` | Stub marker | (string literal) | IR stub indicator |

## Phases

### Phase 1: Mother Memory v0.1
Location: `aeonmi_ai/mother/`

Files:
- `journal.ai` — Event journaling: timestamp, tag, payload, append/query
- `memory.ai` — Key-value memory with decay and reinforcement
- `rules.ai` — Rule engine: condition → action pattern matching
- `maintenance.ai` — Self-maintenance: gc, health checks, thresholds
- `core.ai` — Orchestrator: imports all modules, wires the event loop

Test files (one per module):
- `journal_test.ai`
- `memory_test.ai`
- `rules_test.ai`
- `maintenance_test.ai`
- `core_test.ai`

### Phase 2: Sensory Input v0.1 (future)
### Phase 3: Learning Loop v0.1 (future)
### Phase 4: Self-Model v0.1 (future)

## Rules
1. All code in `.ai` files only. No new Rust.
2. Every file must parse and execute through the existing runtime.
3. Use Genesis density operators — no raw duplication.
4. Write companion `_test.ai` files for every module.
5. Written by AI for AI. Optimize for symbolic density, not human readability.
6. Comments use `⍝` (APL comment) or `∴` / `∵` / `※` (quantum comments).
7. Imports use `import { X } from "./module";` — relative paths, `.ai` auto-appended.
