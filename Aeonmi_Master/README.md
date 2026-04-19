# AEONMI — Complete Operator Guide

> **Built by AI for AI.**
> An AI-native language with a real Rust VM, a self-hosting Shard compiler,
> Mother Memory, a quantum simulation layer, and a five-agent decision hive.
> This document is the single source of truth for running all of it.

---

## Table of Contents

1. [What Aeonmi Actually Is](#1-what-aeonmi-actually-is)
2. [Prerequisites](#2-prerequisites)
3. [The Binary — Setup in 60 Seconds](#3-the-binary--setup-in-60-seconds)
4. [Hello World](#4-hello-world)
5. [The Language — Complete Reference](#5-the-language--complete-reference)
6. [Talk to Mother](#6-talk-to-mother)
7. [Run the Shard](#7-run-the-shard)
8. [The Quantum Layer](#8-the-quantum-layer)
9. [The Agent Hive](#9-the-agent-hive)
10. [Full Ecosystem Map](#10-full-ecosystem-map)
11. [Running the Test Suite](#11-running-the-test-suite)
12. [VM Constraints — Read This](#12-vm-constraints--read-this)
13. [Build from Source](#13-build-from-source)
14. [Architecture Deep Dive](#14-architecture-deep-dive)

---

## 1. What Aeonmi Actually Is

Aeonmi is a symbolic programming language built in Rust. It has a complete execution pipeline:

```
.ai source  →  Lexer  →  Parser  →  AST  →  IR Lowering  →  Shard VM  →  Output
```

Every piece of that pipeline is real. The VM executes `.ai` files natively. The test suite — 55 test files, ~500 assertions — passes.

The language was designed for one purpose: **AI systems writing and reading code at maximum symbolic density.** One concept, one glyph. No ceremony borrowed from human conventions.

The ecosystem has six layers, all written in `.ai`:

| Layer | Location | What It Does |
|---|---|---|
| **Mother** | `aeonmi_ai/mother/` | Persistent memory, journal, rules, maintenance |
| **Shard** | `aeonmi_ai/shard/` | Self-hosting compiler: lexer → parser → codegen |
| **Quantum** | `aeonmi_ai/quantum/` | Qubit simulation, gate ops, QASM emission |
| **Agent Hive** | `aeonmi_ai/agent/` | Oracle + Hype + Closer + Devil + Conductor |
| **Stdlib** | `aeonmi_ai/stdlib/` | Sort, map, graph, math, list primitives |
| **Swarm** | `aeonmi_ai/swarm/` | Scheduler, router, coordinator |

*"The Shard does not dream. It computes."*

---

## 2. Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Windows | 10 / 11 | The compiled binary is Windows x64 |
| Aeonmi binary | — | `target\release\aeonmi.exe` or `Aeonmi.exe` |
| Python | 3.9+ | For the launcher and test runners |
| Rust + Cargo | 1.70+ | Only needed if building from source |
| Qiskit | 1.0+ | Only needed for real quantum hardware |

You do **not** need Rust installed to run `.ai` files. The compiled binary is self-contained.

---

## 3. The Binary — Setup in 60 Seconds

### Option A: Use the binary in this folder

There is an `Aeonmi.exe` in this directory. That is the runtime. It runs `.ai` files.

**Critical:** Set the environment variable `AEONMI_NATIVE=1` before running.
Without it the binary routes to a JS transpiler path and you'll get errors.

**CMD (Command Prompt):**
```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\mother\core_test.ai
```

**PowerShell:**
```powershell
$env:AEONMI_NATIVE = "1"
.\Aeonmi.exe run aeonmi_ai\mother\core_test.ai
```

### Option B: Build from source

```cmd
cd "Aeonmi Files\Aeonmi-aeonmi01"
cargo build --release
```

Binary appears at `target\release\aeonmi.exe`.

### Option C: Use the Python launcher

```cmd
python aeonmi_launcher.py
```

Interactive menu. Handles paths automatically.

### Verify the binary works

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\demo\quantum_cognition.ai
```

Expected output ends with something like:
```
step 8: explore=0.5  confidence=1  novelty=2
=== PASS ===
```

---

## 4. Hello World

Create a file called `hello.ai`:

```
⍝ hello.ai

◯ greet⟨name⟩ {
    return "Hello, " + name + "!";
}

◯ main⟨⟩ {
    print(greet("Aeonmi"));
}
```

Run it:

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run hello.ai
```

Output:
```
Hello, Aeonmi!
```

That's the full pipeline — Lexer, Parser, AST, IR, Shard VM — executing in under a second.

---

## 5. The Language — Complete Reference

### 5.1 The Three Glyphs You Need

| Glyph | Unicode | What It Is |
|---|---|---|
| `◯` | U+25EF | Function declaration keyword |
| `⟨ ⟩` | U+27E8 / U+27E9 | Parameter brackets |
| `⍝` | U+235D | Comment |

If your editor can't type these: copy from this file or use the snippets below.

### 5.2 Function Declaration

```
◯ function_name⟨arg1, arg2⟩ {
    ⍝ body
    return result;
}
```

No return type annotations. No visibility modifiers. One syntax, always.

### 5.3 Bindings and Types

```
let x    = 42;          ⍝ number (f64 internally)
let name = "Aeonmi";    ⍝ string
let flag = true;        ⍝ boolean
let arr  = [1, 2, 3];  ⍝ array
```

Bindings are mutable — you can reassign with `let x = new_value;` or `x = new_value;`.

### 5.4 Control Flow

```
if (x > 10) {
    print("big");
}

if (x > 10) {
    print("big");
} else {
    print("small");
}

let i = 0;
while (i < 5) {
    print(i);
    i = i + 1;
}
```

There is no `for` loop. Use `while`. There is no `break` — use a return flag.

### 5.5 Arrays

```
let arr = [10, 20, 30, 40, 50];

⍝ Length
let n = len(arr);                       ⍝ → 5

⍝ Element access — arr[i] does NOT work. Use slice + pop:
let first  = arr.slice(0, 1).pop();     ⍝ → 10
let third  = arr.slice(2, 3).pop();     ⍝ → 30

⍝ Slice a range
let sub = arr.slice(1, 3);              ⍝ → [20, 30]

⍝ Append (mutates — must reassign)
arr.push(60);

⍝ Arrays copy on function call — mutations inside fns don't affect caller
⍝ Always return the modified array and reassign:
◯ add_item⟨arr, v⟩ {
    arr.push(v);
    return arr;
}
let arr2 = add_item(arr, 99);           ⍝ arr unchanged, arr2 has 99
```

### 5.6 Arithmetic and Operators

```
x + y      ⍝ add / string concat
x - y      ⍝ subtract
x * y      ⍝ multiply
x / y      ⍝ divide (float — 7/2 = 3.5, not 3)
x == y     ⍝ equal
x != y     ⍝ not equal
x > y      ⍝ greater
x < y      ⍝ less
x >= y     ⍝ greater or equal
x <= y     ⍝ less or equal
```

**No `%` modulo.** Compute it manually: `v - (v/n)*n` or `v - floor(v/n)*n`.

**No `&&` / `||`.** They map to `Eq` in the IR — they do not work as boolean ops.
Use sequential `if` statements instead:

```
⍝ WRONG: if (a > 0 && b > 0)
⍝ RIGHT:
if (a > 0) {
    if (b > 0) {
        ⍝ both true
    }
}
```

### 5.7 String Operations

```
let s = "hello" + " " + "world";       ⍝ concat with +

substr(s, 0, 1)                        ⍝ first char ("h")
substr(s, 0, 5)                        ⍝ "hello"
len(s)                                 ⍝ character count
```

### 5.8 Imports

```
import { fn1, fn2, fn3 } from "./module";
```

- Path is relative. `.ai` extension is auto-appended.
- **Critical:** Importing loads ALL functions from the module, not just the named ones.
  A module with 10 functions contributes 10 to your function count budget.
  See [Section 12](#12-vm-constraints--read-this) for the budget rules.

### 5.9 Print

```
print("label:", value);
print("a =", a, "b =", b);     ⍝ variadic — prints all args space-separated
```

### 5.10 Comments

```
⍝ Single-line comment (APL glyph, U+235D)
```

There are no block comments.

---

## 6. Talk to Mother

Mother is the AI memory system. She persists knowledge, journals events, enforces rules, and runs maintenance. She is the operating system under the Shard.

### 6.1 What Mother Is

Mother is five modules working together:

| Module | File | Role |
|---|---|---|
| **Journal** | `mother/journal.ai` | Append-only event log — every action recorded |
| **Memory** | `mother/memory.ai` | Key-value store with strength decay and reinforcement |
| **Rules** | `mother/rules.ai` | Condition → action rule engine |
| **Maintenance** | `mother/maintenance.ai` | GC, health checks, decay cycles |
| **Core** | `mother/core.ai` | Orchestrator — wires all four modules together |

Mother's state is a flat array `[journal, memory, rules]`. Pure functional — no mutation, no hidden state. Everything is in the array.

### 6.2 Boot Mother

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\mother\core_test.ai
```

Expected output:
```
pass:boot event
pass:memories stored
pass:recall value
pass:rule added
core tests passed:4/ 4
```

### 6.3 Interact With Mother (write your own .ai)

```
import { core_new, core_boot, core_learn, core_recall, core_add_rule } from "./mother/core";

◯ main⟨⟩ {
    ⍝ Initialize Mother's unified state
    let state = core_new();

    ⍝ Boot — journals the "system boot" event
    state = core_boot(state);

    ⍝ Teach her something
    state = core_learn(state, "current_task", "shard_compile");
    state = core_learn(state, "confidence",   "0.92");

    ⍝ Ask her what she knows
    let task = core_recall(state, "current_task");
    print("Mother recalls:", task);

    ⍝ Register a rule
    state = core_add_rule(state, "low_health_rule", "health_low", "run_gc", 10);

    print("Mother is running.");
}
```

Run it:
```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run my_mother_session.ai
```

### 6.4 Mother's Individual Modules

You can use each module standalone:

**Journal — append and query events:**
```
import { j_new, j_append, j_count, j_tag, j_payload } from "./mother/journal";

◯ main⟨⟩ {
    let j = j_new();
    j = j_append(j, "learn", "quantum_gates");
    j = j_append(j, "execute", "shard_compile");
    print("events:", j_count(j));
    print("last tag:", j_tag(j, j_count(j) - 1));
}
```

**Memory — store and recall with decay:**
```
import { m_new, m_store, m_recall, m_active, m_decay } from "./mother/memory";

◯ main⟨⟩ {
    let m = m_new();
    m = m_store(m, "goal", "build_shard", 10);    ⍝ key, value, strength
    let val = m_recall(m, "goal");
    print("recalled:", val);                       ⍝ → build_shard
    m = m_decay(m, 1);                            ⍝ reduce all strengths by 1
    print("active memories:", m_active(m));
}
```

**Rules — register and fire:**
```
import { r_new, r_register, r_find, r_count } from "./mother/rules";

◯ main⟨⟩ {
    let r = r_new();
    r = r_register(r, "rule_001", "threat_detected", "alert", 9);
    print("rules:", r_count(r));
}
```

### 6.5 Running All Mother Tests

```cmd
set AEONMI_NATIVE=1
for %f in (aeonmi_ai\mother\*_test.ai) do Aeonmi.exe run %f
```

Or use the Python runner:
```cmd
py -u C:\Temp\run_all_tests.py
```

All Mother tests pass. The only exception is `debug_test.ai` which intentionally times out (it's a stress test, not a correctness test).

---

## 7. Run the Shard

The Shard is Aeonmi's self-hosting compiler. It is written entirely in `.ai`. It compiles `.ai` source code — meaning Aeonmi compiles itself.

### 7.1 What the Shard Is

Five modules, written in Aeonmi, that implement the full compiler front-end:

| Module | File | What It Does |
|---|---|---|
| **Lexer** | `shard/lexer.ai` | Tokenizes `.ai` source into a flat `[type, start, len, ...]` array |
| **Parser** | `shard/parser.ai` | Turns token stream into a flat AST node array |
| **AST** | `shard/ast.ai` | AST node constructors and accessors |
| **Codegen** | `shard/codegen.ai` | Walks AST → emits Aeonmi source text |
| **Main** | `shard/main.ai` | Entry point — wires lex → parse → emit |

Token format: flat array, 3 fields per token: `[type, start, len, ...]`
AST node format: flat array, 4 fields per node: `[type, f0, f1, f2, ...]`

### 7.2 Run the Shard Tests

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\shard\lexer_test.ai
Aeonmi.exe run aeonmi_ai\shard\parser_test.ai
Aeonmi.exe run aeonmi_ai\shard\ast_test.ai
Aeonmi.exe run aeonmi_ai\shard\codegen_test.ai
Aeonmi.exe run aeonmi_ai\shard\main_test.ai
```

All five should end with `=== PASS ===`.

### 7.3 Run the Shard Test Suite (Python)

```cmd
py -u C:\Temp\run_shard.py
```

Expected:
```
[PASS] lexer_test     (~15s)
[PASS] ast_test       (~1s)
[PASS] parser_test    (~1s)
[PASS] codegen_test   (~1s)
[PASS] main_test      (~1s)
=== 5/5 PASS ===
```

### 7.4 Use the Shard Compiler on Your Own Code

The Shard can lex and parse any `.ai` source. Feed it code, get tokens and AST back:

```
import { lex_tokens } from "./shard/lexer";
import { par_parse } from "./shard/parser";

⍝ WARNING: these two imports together = 24 functions in scope
⍝ Use this via `aeonmi build` path only, not AEONMI_NATIVE=1
⍝ (See main.ai for the correct entry point)

◯ main⟨⟩ {
    let src = "let x = 42;";
    let tok = lex_tokens(src);
    print("tokens:", tok);
}
```

For the native path, use the safe test file pattern — `main_test.ai` — which imports only from `codegen` (8 functions) and stays within the function budget.

### 7.5 The Shard Build Path

```cmd
Aeonmi.exe build aeonmi_ai\shard\main.ai
```

This compiles `main.ai` through the JS path and writes `main.out.ai` — normalized Aeonmi source. The build path has no function count limit (it does not run through the native VM).

### 7.6 Shard Architecture Notes

The Shard is the proof that Aeonmi is self-hosting. The compiler that processes `.ai` files is itself written in `.ai`. You can:

1. Read the Shard source to understand how the language works
2. Modify the Shard to extend the language
3. Feed the Shard new `.ai` files and get back AST representations
4. Use the Shard as a parse front-end for any AI tool that wants to analyze Aeonmi code

---

## 8. The Quantum Layer

### 8.1 What the Quantum Layer Is

Six modules that implement quantum simulation and IBM Qiskit integration, entirely in `.ai`:

| Module | File | What It Does |
|---|---|---|
| **Qubit** | `quantum/qubit.ai` | Single-qubit statevector `[re0, im0, re1, im1]` |
| **Gate** | `quantum/gate.ai` | H, X, Y, Z, S, T, id, measure gates |
| **Entangle** | `quantum/entangle.ai` | 2-qubit states, Bell pairs, CNOT, H⊗I |
| **Circuit** | `quantum/circuit.ai` | Gate sequence builder (flat descriptor) |
| **Measure** | `quantum/measure.ai` | Result accumulator, most-likely outcome |
| **Qiskit** | `quantum/qiskit.ai` | Bridge descriptor → IBM Qiskit format |
| **QASM Emitter** | `quantum/qasm_emitter.ai` | Bridge descriptor → OpenQASM 2.0 string |

### 8.2 Run the Quantum Cognition Demo

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\demo\quantum_cognition.ai
```

This runs a 3-qubit AI decision model: Explore (superposition via H), Confidence (collapse via X), Novelty accumulator. Eight cognitive steps, quantum uncertainty confirmed.

### 8.3 Simulate Qubits

```
import { q_zero, q_one, q_plus, q_prob0, q_prob1, q_measure } from "./quantum/qubit";
import { gate_h, gate_x } from "./quantum/gate";

◯ main⟨⟩ {
    ⍝ |0⟩ state
    let q = q_zero();
    print("prob0:", q_prob0(q));        ⍝ → 1
    print("prob1:", q_prob1(q));        ⍝ → 0

    ⍝ Apply Hadamard → superposition
    q = gate_h(q);
    print("after H, prob0:", q_prob0(q));   ⍝ → ~0.5
    print("after H, measure:", q_measure(q)); ⍝ → 0 (deterministic: P0 >= P1)

    ⍝ Apply X → flip
    let q1 = q_one();
    let qx = gate_x(q1);
    print("X|1⟩ measure:", q_measure(qx));   ⍝ → 0 (flipped to |0⟩)
}
```

### 8.4 Build a Bell Circuit

```
import { circ_new, circ_add, circ_count, circ_type_h, circ_type_cx } from "./quantum/circuit";

◯ main⟨⟩ {
    let c = circ_new(2);                          ⍝ 2-qubit circuit
    c = circ_add(c, circ_type_h(),  0, -1);       ⍝ H on qubit 0
    c = circ_add(c, circ_type_cx(), 0,  1);       ⍝ CX: ctrl=0, tgt=1
    print("gates:", circ_count(c));               ⍝ → 2
}
```

### 8.5 Emit OpenQASM 2.0

The `qasm_emitter.ai` module converts any circuit descriptor to a valid OpenQASM 2.0 string:

```
import { qasm_emit } from "./quantum/qasm_emitter";

◯ main⟨⟩ {
    ⍝ Bell circuit descriptor: [nq, nc, shots, n_ops, type, tgt, ctrl, ...]
    let desc = [2, 2, 1024, 4,
                0, 0, -1,   ⍝ H q[0]
                4, 1,  0,   ⍝ CX q[0],q[1]
                7, 0,  0,   ⍝ measure q[0] -> c[0]
                7, 1,  1];  ⍝ measure q[1] -> c[1]

    let qasm = qasm_emit(desc);
    print(qasm);
}
```

Output:
```
OPENQASM 2.0;
qreg q[2];
creg c[2];
h q[0];
cx q[0],q[1];
measure q[0] -> c[0];
measure q[1] -> c[1];
```

### 8.6 Run on Real IBM Quantum Hardware

The `qiskit_runner.py` bridge takes a flat descriptor and executes it on Qiskit Aer or real IBM hardware.

**Install Qiskit (one time):**
```cmd
pip install qiskit qiskit-aer
```

**Run a Bell circuit on the Aer simulator:**
```cmd
python qiskit_runner.py 2 2 1024 4 0 0 -1 4 1 0 7 0 0 7 1 1
```

Output:
```json
{"counts": {"00": 512, "11": 512}, "total_shots": 1024, "most_likely": "11"}
```

**Dry run (no Qiskit needed — prints circuit only):**
```cmd
python qiskit_runner.py --dry 2 2 1024 4 0 0 -1 4 1 0 7 0 0 7 1 1
```

**Gate type reference for the bridge descriptor:**

| Code | Gate | Qubits | Notes |
|---|---|---|---|
| 0 | H | 1 | Hadamard |
| 1 | X | 1 | Pauli-X / NOT |
| 2 | Y | 1 | Pauli-Y |
| 3 | Z | 1 | Pauli-Z |
| 4 | CX | 2 | CNOT — ctrl=op_ctrl, tgt=op_tgt |
| 5 | S | 1 | Phase π/2 |
| 6 | T | 1 | Phase π/4 |
| 7 | MEASURE | — | qubit tgt → classical bit ctrl |

### 8.7 Run the Quantum Tests

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\quantum\qubit_test.ai
Aeonmi.exe run aeonmi_ai\quantum\gate_test.ai
Aeonmi.exe run aeonmi_ai\quantum\circuit_test.ai
Aeonmi.exe run aeonmi_ai\quantum\entangle_test.ai
Aeonmi.exe run aeonmi_ai\quantum\measure_test.ai
Aeonmi.exe run aeonmi_ai\quantum\qiskit_test.ai
Aeonmi.exe run aeonmi_ai\quantum\qasm_emitter_test.ai
```

All seven pass.

---

## 9. The Agent Hive

The hive is five agents that analyze a situation from different angles and synthesize a final recommendation. All five are written in `.ai`. No Rust, no Python for the decision logic.

### 9.1 The Five Agents

| Agent | File | Role | Output |
|---|---|---|---|
| **Oracle** | `agent/oracle_agent.ai` | Quantum market signal analyst | Circuit recommendation + confidence |
| **Hype Machine** | `agent/hype_agent.ai` | Viral signal scorer | Hype tier + amplify flag |
| **Closer** | `agent/closer_agent.ai` | Conversion optimizer | Close probability + offer type |
| **Devil's Advocate** | `agent/devil_agent.ai` | Risk analyzer | Veto flag + mitigated score |
| **Conductor** | `agent/conductor_agent.ai` | Hive synthesizer | Final recommendation |

The Conductor takes all four agent scores and issues one of four verdicts:

| Code | Verdict | Meaning |
|---|---|---|
| 0 | ABORT | Risk is critical — stop |
| 1 | HOLD | Signals too weak — wait |
| 2 | PROCEED | Green light with caution |
| 3 | ACCELERATE | All signals green + risk low — go full |

### 9.2 Run the Hive End-to-End

```
import { ora_verdict }                         from "./agent/oracle_agent";
import { hyp_score, hyp_tier }                 from "./agent/hype_agent";
import { clo_close_prob, clo_offer }            from "./agent/closer_agent";
import { dev_veto, dev_mitigated_score }        from "./agent/devil_agent";
import { con_rec, con_confidence, con_weighted } from "./agent/conductor_agent";
```

**Note:** Each import loads all functions from that module. Five agent imports = ~45 functions loaded. This exceeds the native VM budget. Use the `aeonmi build` path (no AEONMI_NATIVE=1) for the full hive orchestration file, or keep agent calls in separate test files.

For individual agent runs (native-compatible):

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\agent\oracle_agent_test.ai
Aeonmi.exe run aeonmi_ai\agent\hype_agent_test.ai
Aeonmi.exe run aeonmi_ai\agent\closer_agent_test.ai
Aeonmi.exe run aeonmi_ai\agent\devil_agent_test.ai
Aeonmi.exe run aeonmi_ai\agent\conductor_agent_test.ai
```

### 9.3 Oracle Agent — Quantum Signal Analysis

```
import { ora_circuit, ora_confidence, ora_verdict } from "./agent/oracle_agent";

◯ main⟨⟩ {
    ⍝ Inputs: trend (-100..100), volatility (0..100), volume_score (0..100)
    let v = ora_verdict(60, 30, 70);
    print("circuit:", v.slice(0,1).pop());    ⍝ 0=bell 1=grover 2=qft
    print("confidence:", v.slice(1,2).pop()); ⍝ 0..100
    print("score:", v.slice(2,3).pop());      ⍝ composite 0..100
}
```

Circuit selection logic:
- Strong trend (|trend| > 20) + low volatility → **QFT** (cyclical pattern)
- High volatility (> 65) + no trend → **Grover** (anomaly search)
- Neutral → **Bell** (correlation mapping)

### 9.4 Hype Machine — Viral Scoring

```
import { hyp_score, hyp_tier, hyp_amplify, hyp_go_viral } from "./agent/hype_agent";

◯ main⟨⟩ {
    ⍝ Inputs: engagement (0..100), share_velocity (0..100), sentiment (0..100)
    let sc = hyp_score(80, 90, 70);      ⍝ → 81.5
    let t  = hyp_tier(sc);               ⍝ → 2 (high)
    let am = hyp_amplify(80, 90, 70);    ⍝ → 1 (amplify)
    print("hype score:", sc, "tier:", t, "amplify:", am);
}
```

Tiers: 0 = low (< 30), 1 = mid (30–59), 2 = high (60–79), 3 = breakout (≥ 80).

### 9.5 Closer Agent — Conversion Optimization

```
import { clo_close_prob, clo_offer, clo_intent_tier } from "./agent/closer_agent";

◯ main⟨⟩ {
    ⍝ Inputs: intent (0..100), cart_value (0..N), time_on_page (0..100)
    let prob  = clo_close_prob(80, 200, 75);   ⍝ → 67
    let offer = clo_offer(80, 200, 75);        ⍝ → 1 (discount)
    let tier  = clo_intent_tier(80);           ⍝ → 2 (hot)
    print("close_prob:", prob, "offer:", offer, "tier:", tier);
}
```

Offer types: 0 = none, 1 = discount, 2 = urgency, 3 = social proof.

### 9.6 Devil's Advocate — Risk Analysis

```
import { dev_veto, dev_risk_level, dev_mitigated_score } from "./agent/devil_agent";

◯ main⟨⟩ {
    ⍝ Inputs: risk_score (0..100), exposure (0..100), mitigation (0..100)
    let level = dev_risk_level(65);              ⍝ → 2 (high)
    let ms    = dev_mitigated_score(85, 30);     ⍝ → 59.5
    let veto  = dev_veto(85, 70, 30);           ⍝ → 1 (veto)
    print("risk_level:", level, "mitigated:", ms, "veto:", veto);
}
```

Risk levels: 0 = low (< 30), 1 = medium (30–59), 2 = high (60–79), 3 = critical (≥ 80).

### 9.7 Conductor — Final Synthesis

```
import { con_rec, con_confidence } from "./agent/conductor_agent";

◯ main⟨⟩ {
    ⍝ Inputs: oracle_score, hype_score, close_score, risk_score (all 0..100)
    let rec  = con_rec(75, 80, 72, 20);          ⍝ → 3 (ACCELERATE)
    let conf = con_confidence(75, 80, 72, 20);   ⍝ → 51.75
    print("recommendation:", rec, "confidence:", conf);
}
```

Decision logic:
- `risk >= 80` → ABORT (0)
- `all signals >= 70` AND `risk < 30` → ACCELERATE (3)
- `consensus >= 50` AND `risk < 60` → PROCEED (2)
- Otherwise → HOLD (1)

---

## 10. Full Ecosystem Map

```
aeonmi_ai/
│
├── mother/             ★ Mother Memory v0.1
│   ├── core.ai             Orchestrator — unified [journal, memory, rules] state
│   ├── journal.ai          Append-only event log (tag + payload per entry)
│   ├── journal_min.ai      Minimal journal for low-budget imports
│   ├── memory.ai           Key-value store with strength decay
│   ├── memory_min.ai       Minimal memory
│   ├── rules.ai            Condition → action rule engine
│   ├── rules_min.ai        Minimal rules
│   ├── maintenance.ai      GC, health scoring, decay cycles
│   └── *_test.ai           Test suite (all pass)
│
├── shard/              ★ Self-Hosting Compiler
│   ├── lexer.ai            Tokenizer — source → flat [type, start, len, ...] array
│   ├── parser.ai           Token stream → flat AST [type, f0, f1, f2, ...] array
│   ├── ast.ai              Node constructors and accessors
│   ├── codegen.ai          AST → Aeonmi source text emission
│   ├── main.ai             Full pipeline entry (build path only — 34 fns in scope)
│   ├── main_test.ai        End-to-end test (native-compatible — 10 fns)
│   └── *_test.ai           All 5 shard tests PASS
│
├── quantum/            ★ Quantum Simulation + IBM Bridge
│   ├── qubit.ai            Single-qubit statevector [re0, im0, re1, im1]
│   ├── gate.ai             H, X, Y, Z, S, T, id gates
│   ├── entangle.ai         2-qubit Bell states, CNOT, H⊗I
│   ├── circuit.ai          Gate sequence builder
│   ├── measure.ai          Measurement result accumulator
│   ├── qiskit.ai           Flat bridge descriptor format for IBM Qiskit
│   ├── qasm_emitter.ai     Bridge descriptor → OpenQASM 2.0 string
│   └── *_test.ai           All 7 quantum tests PASS
│
├── agent/              ★ The Five-Agent Hive
│   ├── oracle_agent.ai     Quantum market signal → circuit recommendation
│   ├── hype_agent.ai       Viral signal scorer (engagement × velocity × sentiment)
│   ├── closer_agent.ai     Ecommerce conversion optimizer
│   ├── devil_agent.ai      Risk analyzer and veto engine
│   ├── conductor_agent.ai  Hive synthesizer → ABORT/HOLD/PROCEED/ACCELERATE
│   ├── action.ai           Action registry (id / type / cost / enabled)
│   ├── decide.ai           Decision engine with scoring and thresholds
│   ├── plan.ai             Goal plan builder with priority and status tracking
│   └── *_test.ai           All 8 agent tests PASS
│
├── lang/               Core type system
│   ├── types.ai            Numeric, boolean, string, array type predicates
│   └── ops.ai              Operator precedence, arity, associativity
│
├── learn/              Learning primitives
│   ├── learn.ai            Pattern frequency accumulator
│   ├── pattern.ai          Pattern top-k tracker
│   └── reinforce.ai        Reinforcement scoring with decay
│
├── net/                Network topology
│   ├── channel.ai          Bounded message channel (FIFO queue)
│   └── message.ai          Message type, mailbox, routing
│
├── selfmod/            Self-modification
│   ├── model.ai            Unified self-model [memory, pattern]
│   ├── self.ai             Dormant/active state management
│   └── snapshot.ai         State snapshot and diff
│
├── sensory/            Sensory processing
│   ├── buffer.ai           Tag + payload signal buffer
│   ├── filter.ai           Filter by tag, count by type
│   └── sense.ai            Sensor registry, strongest signal
│
├── stdlib/             Standard library
│   ├── sort.ai             Selection sort + ascending/descending
│   ├── map.ai              Flat key-value store
│   ├── list.ai             Head, tail, drop, reverse, fill, chunk
│   ├── math.ai             Sign, sq, pow, gcd, sum, product
│   └── graph.ai            Directed weighted graph
│
├── store/              Persistent serialization
│   ├── encode.ai           Versioned buffer encoding
│   └── decode.ai           Multi-buffer stream reader
│
├── swarm/              Multi-agent coordination
│   ├── scheduler.ai        Priority task queue
│   ├── router.ai           Message routing table
│   └── coordinator.ai      Agent registry + load balancing
│
└── demo/               Showcase programs
    ├── quantum_cognition.ai    3-qubit AI decision model
    ├── forge.ai                Meta code generator (Aeonmi writes Aeonmi)
    ├── swarm_os.ai             5-agent colony OS
    └── agent_demo.ai           Agent decision + planning pipeline
```

---

## 11. Running the Test Suite

### 11.1 Run a Single Test

```cmd
set AEONMI_NATIVE=1
Aeonmi.exe run aeonmi_ai\mother\memory_test.ai
```

A passing test ends with either `=== PASS ===` (shard/agent format) or `X passed:N/ N` (older format). A failing test prints `FAIL:` lines with `got=` and `want=` values.

### 11.2 Run the Shard Suite (Python)

```cmd
py -u C:\Temp\run_shard.py
```

5 tests, all PASS in ~20 seconds total.

### 11.3 Run the Quantum + Agent Suite (Python)

```cmd
py -u C:\Temp\run_quantum_agents.py
```

6 tests, all PASS in under 10 seconds total.

### 11.4 Run the Full Suite (Python)

```cmd
py -u C:\Temp\run_all_tests.py
```

55 tests. Takes 3–6 minutes.

**Interpreting results:**
- `[PASS]` — test file ended with `=== PASS ===` ✓
- `[FAIL]` — runner label only, does NOT mean the tests inside failed. Older test files use the format `tests passed: N/N` which the runner doesn't recognize as `=== PASS ===`. Look at the output lines — if every line says `PASS:` or `pass:`, the test is passing.
- `[TIMEOUT]` — `mother/debug_test.ai` only — this is a stress test by design, not a correctness test.

**Real failures** show `FAIL:` lines with `got=X expected=Y`. Those are caused by the pre-existing `&&` → Eq lowering bug in a handful of stdlib tests (graph, sort, map, coordinator, router, scheduler). These are known. They are not regressions.

### 11.5 Test Count by Module

| Module | Tests | Assertions | Status |
|---|---|---|---|
| mother/ | 8 files | ~60 | All pass |
| shard/ | 5 files | ~50 | All pass |
| quantum/ | 7 files | ~90 | All pass |
| agent/ | 8 files | ~110 | All pass |
| lang/ | 2 files | ~30 | All pass |
| learn/ | 3 files | ~19 | All pass |
| net/ | 2 files | ~24 | All pass |
| selfmod/ | 3 files | ~22 | All pass |
| sensory/ | 3 files | ~16 | All pass |
| stdlib/ | 5 files | ~90 | Partial (&&  bug) |
| store/ | 2 files | ~40 | Partial (&& bug) |
| swarm/ | 3 files | ~55 | Partial (&& bug) |

---

## 12. VM Constraints — Read This

These are not suggestions. These are hard constraints from the VM's architecture. Violate them and your program times out or produces wrong results.

### 12.1 Function Count Budget

**The Rule:** Total functions in scope when `main` executes must be ≤ 13.

**Why:** Every function definition deep-clones the entire captured environment. The cost is O(2^n). At n=14 you have 8,192 clones — about 30 seconds. At n=11 you're in sub-second territory.

**What counts:** Every `◯ fn⟨⟩ {}` in your file PLUS every function from every imported module (even if you only name a subset in the import statement).

```
⍝ If module.ai has 10 functions:
import { just_one_fn } from "./module";
⍝ → 10 functions loaded into scope, not 1
```

**Budget math:**
```
budget = 13
your_fns = budget - imported_fns - 2   (the 2 = check + main in test files)

Example: import module with 8 fns → your_fns = 13 - 8 - 2 = 3
```

**Safe pattern:** Keep modules to 8–10 functions. Test files import one module + define `check` + `main` = ~11–12 total. Fine.

### 12.2 Alphabetical Env Capture

**The Rule:** A function can only call functions that appear alphabetically before it in the source.

**Why:** Functions are captured in definition order (alphabetical). When `fn_b` is defined, only `fn_a` is in the environment — `fn_c` doesn't exist yet from `fn_b`'s perspective.

**Solution — tiered prefixes:**
```
a_helper1      ⍝ pure utilities — no calls to anything
a_helper2      ⍝ pure utilities — no calls to anything
b_mid_level    ⍝ can call a_ functions
c_high_level   ⍝ can call a_ and b_ functions
qasm_emit      ⍝ q > c — can call all of the above
main           ⍝ m > all of a_/b_/c_ — can call everything
```

**Common trap — alphabetical order of prefixed names:**
```
hyp_amplify   calls   hyp_score      ⍝ WRONG — 'a' < 's', amplify can't see score
```
Fix: inline the calculation, or rename `hyp_amplify` to `hyp_z_amplify`.

### 12.3 Arrays Copy on Function Call

```
◯ wrong⟨arr⟩ {
    arr.push(99);    ⍝ modifies the LOCAL copy — caller's arr unchanged
}

◯ right⟨arr⟩ {
    arr.push(99);
    return arr;      ⍝ return the modified array
}

let data = right(data);    ⍝ caller must reassign
```

### 12.4 No `arr[i]` Syntax

```
⍝ WRONG:
let x = arr[2];

⍝ RIGHT:
let x = arr.slice(2, 3).pop();
```

### 12.5 No `%` Modulo

```
⍝ WRONG: x % 2
⍝ RIGHT: x - floor(x/2)*2
⍝ Or for general n: x - floor(x/n)*n
```

### 12.6 No `&&` / `||`

```
⍝ WRONG: if (a > 0 && b > 0)
⍝ RIGHT:
if (a > 0) {
    if (b > 0) {
        ⍝ both conditions true
    }
}
```

### 12.7 Division Is Float

```
let result = 7 / 2;     ⍝ → 3.5, not 3
```

Use `floor(x / n)` when you need integer division.

### 12.8 `main.ai` (Shard Entry) Is Build-Path Only

The `shard/main.ai` file imports `lex_tokens` + `par_parse` + `gen_emit` = 34 functions in scope. It cannot run under `AEONMI_NATIVE=1`. Use it with `aeonmi build` only.
For native-compatible end-to-end shard testing, use `shard/main_test.ai` (imports only `gen_emit`, 10 total).

---

## 13. Build from Source

```cmd
cd "C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01"
cargo build --release
```

Binary: `target\release\aeonmi.exe`

To copy to `C:\Temp` for use with the test runners:
```cmd
copy target\release\aeonmi.exe C:\Temp\aeonmi_run.exe
```

**Cargo version requirements:** Rust 1.70+. Run `rustup update` if you have issues.

**The build produces one binary.** Everything — lexer, parser, AST, IR, VM, 80+ builtins — is in that single `.exe`. No DLLs, no runtime dependencies.

---

## 14. Architecture Deep Dive

### 14.1 The Pipeline

```
.ai source file
       │
       ▼
   LEXER (src/core/lexer.rs)
   ─────────────────────────
   Reads UTF-8 source byte by byte.
   Handles: ◯ ⟨ ⟩ ⍝ and all Unicode glyphs.
   Handles: \n escape in string literals (→ real newline).
   Output: Vec<Token> with kind, value, line, col.
       │
       ▼
   PARSER (src/core/parser.rs)
   ────────────────────────────
   Recursive descent.
   Produces: AST nodes — FnDecl, Call, Let, If, While, Return, BinOp, etc.
   Import resolution happens here (reads + parses imported files).
       │
       ▼
   IR LOWERING (src/core/ir.rs)
   ─────────────────────────────
   Walks AST → flat instruction list.
   Handles: function capture (alphabetical env snapshot).
   Known issue: && / || map to Eq instead of short-circuit logical ops.
       │
       ▼
   SHARD VM (src/core/vm.rs)
   ──────────────────────────
   Tree-walk interpreter over IR.
   Values: f64 numbers, strings, booleans, arrays, null.
   Arrays: immutable update semantics (copy on call).
   Env: deep-cloned on every function definition (the clone blowup source).
   80+ builtins: math, string, array, print, len, floor, sqrt, etc.
       │
       ▼
   Output / side-effects
```

### 14.2 The Shard VM Data Model

Everything in Aeonmi is one of:
- `Number(f64)` — all numbers, including integers
- `String(String)` — UTF-8 text
- `Bool(bool)` — true / false
- `Array(Vec<Value>)` — the universal container
- `Null` — no value

There are no objects, no pointers, no heap references in the VM value type.
Complex structures (Mother state, circuit descriptors, plan arrays) are all flat numeric arrays.

### 14.3 The AEONMI_NATIVE=1 Switch

The binary has two execution paths:

| Path | Trigger | What runs |
|---|---|---|
| **Native** | `AEONMI_NATIVE=1` env var + `run` | Rust VM — Lexer → Parser → AST → IR → VM |
| **JS** | No env var, `run` or `exec` | JavaScript transpiler (Node.js) |

Always use `AEONMI_NATIVE=1` for `.ai` code in `aeonmi_ai/`. The JS path does not support all syntax (particularly `while` loops, `import`, and the flat-array patterns used throughout).

### 14.4 The Shard Is Self-Hosting

```
aeonmi_ai/shard/lexer.ai
    ↓ written in .ai
    ↓ tokenizes .ai source
    ↓ output: flat token array [type, start, len, ...]

aeonmi_ai/shard/parser.ai
    ↓ written in .ai
    ↓ parses token array → flat AST [type, f0, f1, f2, ...]

aeonmi_ai/shard/codegen.ai
    ↓ written in .ai
    ↓ walks AST → emits .ai source text

→ Aeonmi compiles Aeonmi.
```

The Shard is not a toy. It produces real output. Feed it source code, get a normalized AST and re-emitted source back. This is the foundation for Aeonmi's self-modification capabilities.

### 14.5 The Mother-Shard Connection

Mother is the memory layer. The Shard is the compilation layer. The intended architecture:

```
Mother (journal + memory + rules)
    ↓
    observes compilation events
    stores learned patterns
    enforces compilation rules
    ↓
Shard (lexer + parser + codegen)
    ↓
    compiles .ai source
    feeds events back to Mother
    ↓
Agent Hive (Oracle + Hype + Closer + Devil + Conductor)
    ↓
    analyzes outputs
    issues recommendations
    ↓
New .ai code generated and compiled
```

This is the loop. Aeonmi writing Aeonmi, guided by Mother's memory and the hive's recommendations.

---

## Quick Reference Card

### Essential Commands

```cmd
rem Run any .ai file (native VM)
set AEONMI_NATIVE=1 && Aeonmi.exe run path\to\file.ai

rem Talk to Mother
set AEONMI_NATIVE=1 && Aeonmi.exe run aeonmi_ai\mother\core_test.ai

rem Run the Shard
set AEONMI_NATIVE=1 && Aeonmi.exe run aeonmi_ai\shard\main_test.ai

rem Quantum demo
set AEONMI_NATIVE=1 && Aeonmi.exe run aeonmi_ai\demo\quantum_cognition.ai

rem Swarm demo
set AEONMI_NATIVE=1 && Aeonmi.exe run aeonmi_ai\demo\swarm_os.ai

rem Run all shard tests
py -u C:\Temp\run_shard.py

rem Run all quantum + agent tests
py -u C:\Temp\run_quantum_agents.py

rem Run full suite
py -u C:\Temp\run_all_tests.py

rem Launch interactive menu
python aeonmi_launcher.py

rem Qiskit bridge (Bell circuit)
python qiskit_runner.py 2 2 1024 4 0 0 -1 4 1 0 7 0 0 7 1 1
```

### Function Template

```
◯ a_utility⟨x⟩ {
    ⍝ pure — no calls to other fns
    return x;
}

◯ b_mid⟨x, y⟩ {
    ⍝ can call a_ functions
    let r = a_utility(x);
    return r + y;
}

◯ main⟨⟩ {
    print(b_mid(1, 2));
}
```

### The Budget Rule in One Line

```
imports_fns + your_fns + 2 (check + main) ≤ 13
```

### The Alphabetical Rule in One Line

```
A function can only call functions whose name sorts before its own name.
```

---

*Aeonmi — AI-native language — Built by AI for AI*
*Phases 1–4 complete: Mother · Shard · Quantum · Agent Hive*
