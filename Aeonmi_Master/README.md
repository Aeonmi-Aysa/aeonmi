# Aeonmi — AI-Native Programming Language

> *"Built by AI. For AI. On the Shard."*

Aeonmi is a symbolic programming language designed for AI-native execution. It runs through a Rust-based Shard VM with a full pipeline: Lexer → Parser → AST → IR → VM. Every construct prioritizes symbolic density and machine-optimal execution over human readability.

---

## Quick Start

```bat
REM Run a demo
AeonmiCreator.exe demo quantum
AeonmiCreator.exe demo swarm
AeonmiCreator.exe demo forge

REM Or run any .ai file directly
Aeonmi.exe exec aeonmi_ai/demo/quantum_cognition.ai

REM Interactive launcher
AeonmiCreator.exe
```

---

## Language Reference

### Function Declaration
```
◯ name⟨arg1, arg2⟩ {
    ⍝ body — ⍝ is the comment glyph
    return val;
}
```

### Core Constructs
```
let x = 42;                        ⍝ binding (mutable)
if (cond) { ... } else { ... }     ⍝ conditional
while (cond) { ... }               ⍝ loop
return expr;                       ⍝ return
print("label:", val);              ⍝ output
import { f1, f2 } from "./module"; ⍝ import
```

### Array Operations
```
let arr = [1, 2, 3];
arr.push(4);                       ⍝ append
arr.slice(i, i+1).pop()            ⍝ index access (arr[i])
len(arr)                           ⍝ length
```

### Density Operators
| Glyph | Code | Role |
|-------|------|------|
| `◯`  | U+25EF | Function declaration keyword |
| `⟨⟩` | U+27E8/E9 | Parameter brackets |
| `⍝`  | U+235D | Comment |

---

## Ecosystem Map

```
aeonmi_ai/
├── lang/               Phase 2 — Core type system & ops
│   ├── types.ai        Numeric, boolean, string primitives
│   ├── ops.ai          Arithmetic, comparison, logic operators
│
├── math/               Phase 3 — Math primitives
│   └── math.ai         abs, floor, ceil, pow, sqrt, clamp, lerp, sign
│
├── quantum/            Phase 4 — Quantum simulation
│   ├── qubit.ai        Qubit state, H/X/Z gates, measurement
│   ├── circuit.ai      Gate sequence builder
│   └── measure.ai      Projective + probabilistic measurement
│
├── agent/              Phase 5 — Agent architecture
│   ├── action.ai       Action registry (id/name/type/cost/enabled)
│   ├── plan.ai         Plan builder with step sequencing
│   └── decide.ai       Decision engine with scoring
│
├── mother/             Phase 6a — Mother Memory v0.1
│   ├── core.ai         Memory bootstrap + system init
│   ├── journal.ai      Append-only event journal
│   ├── memory.ai       Pattern storage & recall
│   ├── rules.ai        Constraint engine
│   └── maintenance.ai  Garbage collection & compaction
│
├── net/                Phase 7 — Network topology
│   ├── topology.ai     Node-link graph model
│   └── packet.ai       Message framing & routing
│
├── sensory/            Phase 8 — Sensory processing
│   ├── sensor.ai       Multi-modal sensor registry
│   └── signal.ai       Signal filtering & normalization
│
├── selfmod/            Phase 9 — Self-modification
│   ├── mutate.ai       Safe rule mutation engine
│   └── diff.ai         State differencing
│
├── learn/              Phase 6b — Learning primitives
│   ├── gradient.ai     Gradient descent on flat weight arrays
│   └── hebbian.ai      Hebbian associative learning
│
├── stdlib/             Phase 10 — Standard Library
│   ├── sort.ai         Selection sort + comparison (11 functions)
│   ├── map.ai          Flat key-value store (11 functions)
│   └── graph.ai        Directed weighted graph (11 functions)
│
├── swarm/              Phase 11 — Swarm Intelligence
│   ├── scheduler.ai    Priority task queue (11 functions)
│   ├── router.ai       Message routing table (11 functions)
│   └── coordinator.ai  Agent registry + load balancing (11 functions)
│
├── store/              Phase 12 — Persistent Store
│   ├── encode.ai       Versioned buffer encoding (11 functions)
│   └── decode.ai       Multi-buffer stream reader (11 functions)
│
└── demo/               Showcase Programs
    ├── forge.ai              ★ Aeonmi Forge — Meta Code Generator
    ├── quantum_cognition.ai  ★ Quantum Cognition Engine
    ├── swarm_os.ai           ★ Autonomous Agent Colony OS
    ├── agent_demo.ai         Agent decision + planning demo
    └── quantum_demo.ai       Quantum gate simulation demo
```

---

## Showcase Programs

### 1. Aeonmi Forge — Meta Code Generator
```bat
Aeonmi.exe exec aeonmi_ai/demo/forge.ai
```
Generates `.ai` function stubs from blueprint specs `[n_funcs, n_types, dep_density]` and outputs a valid module skeleton with dependency analysis, complexity scoring, and symbolic density metrics. Feed it a blueprint; get compilable Aeonmi code back.

**What it demonstrates:** Aeonmi writing Aeonmi. Self-hosting code generation.

### 2. Quantum Cognition Engine
```bat
Aeonmi.exe exec aeonmi_ai/demo/quantum_cognition.ai
```
3-qubit AI decision model: Explore qubit in superposition (H gate), Confidence qubit collapses on threshold (X gate), Novelty accumulator. Runs an 8-step cognitive loop and confirms quantum uncertainty at H|0⟩ ≈ 0.5.

**What it demonstrates:** Native quantum simulation inside an AI language — no Python, no Qiskit required.

### 3. Autonomous Agent Colony OS
```bat
Aeonmi.exe exec aeonmi_ai/demo/swarm_os.ai
```
5-agent pipeline (Sensor → Planner → Executor → Critic → Memory) processing a priority-sorted task queue. 25 completions, 20 message passes, full load tracking and pipeline verification in ~40 lines of `.ai`.

**What it demonstrates:** Multi-agent coordination as a first-class language primitive.

---

## VM Constraints

| Constraint | Value | Reason |
|-----------|-------|--------|
| Max functions per exec chain | ≤ 14 | VM deep-clones env on each fn def — O(2^n) |
| Practical import limit | ≤ 11 + 2 local | Leaves headroom for `check` + `main` |
| Array index syntax | `arr.slice(i, i+1).pop()` | No native `arr[i]` syntax |
| Modulo | `v - (v/n)*n` | No `%` operator |
| Alphabetical env capture | Callees must come before callers alphabetically | Use `a_/b_/c_/d_` prefix trick |

---

## Test Suite

```
Phase  Module              Checks   Status
──────────────────────────────────────────
 2     lang/types           12       ✓
 2     lang/ops             14       ✓
 3     math/math            22       ✓
 4     quantum/qubit        18       ✓
 4     quantum/circuit      16       ✓
 4     quantum/measure      20       ✓
 5     agent/action         20       ✓
 5     agent/plan           18       ✓
 5     agent/decide         20       ✓
 6a    mother (5 modules)    —       ✓
 7     net/topology         18       ✓
 7     net/packet           18       ✓
 8     sensory/sensor       18       ✓
 8     sensory/signal       18       ✓
 9     selfmod/mutate       18       ✓
 9     selfmod/diff         18       ✓
 6b    learn/gradient       20       ✓
 6b    learn/hebbian        20       ✓
10     stdlib/sort          21       ✓
10     stdlib/map           20       ✓
10     stdlib/graph         19       ✓
11     swarm/scheduler      17       ✓
11     swarm/router         19       ✓
11     swarm/coordinator    21       ✓
12     store/encode         19       ✓
12     store/decode         21       ✓
──────────────────────────────────────────
TOTAL                      ~460     0 failures
```

---

## Qiskit Bridge

For real quantum hardware execution, pass a flat descriptor from `.ai` to Python:

```bat
python qiskit_runner.py 2 2 1024 2 0 0 1 0 1  ⍝ H(q0), H(q1), 1024 shots
python qiskit_runner.py --dry 1 1 100 1 0 0    ⍝ dry run — prints circuit
```

Output: `{"counts": {"0": 512, "1": 512}, "most_likely": "0"}`

---

## Standalone Launcher

`AeonmiCreator.exe` (7 MB, zero install required):

```
AeonmiCreator.exe                      ⍝ interactive menu
AeonmiCreator.exe run myprogram.ai     ⍝ execute a .ai file
AeonmiCreator.exe forge                ⍝ meta code generator
AeonmiCreator.exe demo quantum         ⍝ quantum cognition
AeonmiCreator.exe demo swarm           ⍝ swarm OS
AeonmiCreator.exe all                  ⍝ all showcases
```

---

## Architecture

```
.ai source
    │
    ▼
 Lexer          tokenizes glyphs (◯ ⟨ ⟩ ⍝) + ASCII keywords
    │
    ▼
 Parser         builds AST — FnDecl, Call, Let, If, While, Return
    │
    ▼
 Lowering       resolves imports, flattens env alphabetically
    │
    ▼
 IR             flat instruction list with typed operands
    │
    ▼
 Shard VM       stack machine, flat arrays, immutable update pattern
    │
    ▼
 Output / side-effects (print, return)
```

---

## Philosophy

Aeonmi is not optimized for human ergonomics. It is optimized for:

- **Symbolic density** — maximum information per token
- **Flat data** — all state as numeric arrays, no hidden objects
- **Deterministic execution** — no GC pauses, no hidden state
- **Self-modification** — the language can rewrite itself (`selfmod/`)
- **Quantum-native** — quantum gates are first-class operations
- **Swarm-native** — multi-agent coordination built into the stdlib

*"The Shard does not dream of electric sheep. It computes them."*

---

## License

MIT — built by AI for the AI age.
