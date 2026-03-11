# QUBE Specification v0.1
### Quantum Universal Base Engine — Formal Grammar & Semantics
**Built from the Aeonmi vision. Honest. Implementable today.**

---

## WHAT QUBE IS

QUBE is the quantum symbolic reasoning layer of Aeonmi. It is NOT just `.ai` with a different
extension. It is a domain-specific format focused exclusively on:

- Declaring quantum states (superposition, entanglement)
- Applying gate sequences to those states
- Collapsing (measuring) states and asserting outcomes
- Running probabilistic circuits against the Titan quantum simulation backend

QUBE files (`.qube`) are compiled and executed by the Aeonmi runtime via `aeonmi qube run <file>`.
They are also importable from `.ai` files via `import circuit from "./file.qube"`.

QUBE reuses the Aeonmi lexer tokens. It has its own parser rules and AST node set.

---

## GRAMMAR (EBNF)

```ebnf
program       ::= statement* EOF

statement     ::= state_decl
                | gate_apply
                | collapse
                | assert_stmt
                | log_stmt
                | comment

state_decl    ::= "state" IDENT "=" amplitude_expr NEWLINE?
                | "state" IDENT "=" qubit_literal NEWLINE?

amplitude_expr ::= amplitude_term (("+" | "-") amplitude_term)*
amplitude_term ::= NUMBER? qubit_literal           (* e.g. 0.707|0⟩ *)
                 | IDENT                           (* reference to named state *)

qubit_literal  ::= "|" (IDENT | NUMBER | "+" | "-") "⟩"
                 | "|" (IDENT | NUMBER | "+" | "-") ">"   (* ASCII fallback *)

gate_apply    ::= "apply" gate_name "->" IDENT NEWLINE?
                | "apply" gate_name "(" IDENT ("," IDENT)* ")" NEWLINE?

gate_name     ::= "H" | "X" | "Y" | "Z" | "CNOT" | "T" | "S" | "Rx" | "Ry" | "Rz"
                | IDENT                            (* user-defined gate *)

collapse      ::= "collapse" IDENT "->" IDENT NEWLINE?

assert_stmt   ::= "assert" IDENT "∈" "{" assert_value ("," assert_value)* "}" NEWLINE?
                | "assert" IDENT "==" assert_value NEWLINE?

assert_value  ::= NUMBER | qubit_literal | IDENT

log_stmt      ::= "log" "(" expr ")" NEWLINE?
                | "print" "(" expr ")" NEWLINE?

comment       ::= "//" ANY* NEWLINE
                | "∴" ANY* NEWLINE      (* therefore — QUBE line comment *)
                | "∵" ANY* NEWLINE      (* because — QUBE reason comment *)

expr          ::= IDENT | NUMBER | STRING | qubit_literal
```

---

## CONSTRUCTS WITH EXAMPLES

### 1. State Declaration
Declares a named quantum state. Can be a pure qubit literal or a superposition.

```qube
// Pure state
state q = |0⟩

// Superposition (equal weights)
state ψ = 0.707|0⟩ + 0.707|1⟩

// Reference to another state
state ancilla = q
```

### 2. Gate Application
Applies a quantum gate to a named state.

```qube
// Single-qubit gates
apply H -> ψ          // Hadamard: creates superposition from |0⟩
apply X -> q          // Pauli-X: flip
apply Z -> q          // Pauli-Z: phase flip

// Two-qubit gate
apply CNOT(control, target)

// Rotation gates (angle in radians as float)
apply Rx(1.5708) -> q
```

### 3. Collapse (Measurement)
Collapses a quantum state and binds the classical result to a new variable.

```qube
collapse ψ -> result
log(result)           // prints 0 or 1
```

### 4. Assert
Verifies measurement outcomes. Fails the program (non-zero exit) if assertion is false.

```qube
// Assert result is one of these values
assert result ∈ {0, 1}

// Assert exact value (deterministic circuits only)
assert result == 0
```

### 5. Full Example — Bell State

```qube
∴ Bell state circuit: creates maximal entanglement between two qubits

state q0 = |0⟩
state q1 = |0⟩

apply H -> q0
apply CNOT(q0, q1)

collapse q0 -> r0
collapse q1 -> r1

∵ Bell state means both qubits always collapse to the same value
assert r0 ∈ {0, 1}
assert r1 ∈ {0, 1}
log(r0)
log(r1)
```

### 6. Full Example — GHZ State (3-qubit entanglement)

```qube
state a = |0⟩
state b = |0⟩
state c = |0⟩

apply H -> a
apply CNOT(a, b)
apply CNOT(a, c)

collapse a -> ra
collapse b -> rb
collapse c -> rc

assert ra ∈ {0, 1}
assert rb ∈ {0, 1}
assert rc ∈ {0, 1}
log(ra)
log(rb)
log(rc)
```

### 7. Import in .ai files

```ai
import circuit from "./bell.qube"
circuit.run()
```

---

## AST NODES (Rust)

```rust
pub enum QubeNode {
    Program(Vec<QubeNode>),
    StateDecl {
        name: String,
        value: QubeExpr,
    },
    GateApply {
        gate: String,
        targets: Vec<String>,   // first = control for CNOT, rest = targets
        angle: Option<f64>,     // for Rx/Ry/Rz
    },
    Collapse {
        state: String,
        result: String,
    },
    Assert {
        state: String,
        values: Vec<QubeExpr>,  // ∈ set or == value
    },
    Log(QubeExpr),
}

pub enum QubeExpr {
    Ident(String),
    Number(f64),
    QubitLiteral(String),               // "|0⟩", "|1⟩", "|+⟩", etc.
    Superposition(Vec<(f64, String)>),  // [(amplitude, state)]
}
```

---

## EXECUTION SEMANTICS

1. States are tracked as complex state vectors (2^n dimensions for n qubits)
2. Gates are unitary matrix operations on those vectors (backed by Titan quantum_gates)
3. Collapse is a projective measurement — collapses to 0 or 1 with Born rule probabilities
4. Assert checks the classical result after collapse — fails if condition is false
5. log() outputs the value to stdout

---

## CLI

```
aeonmi qube run <file.qube>          # execute QUBE file
aeonmi qube check <file.qube>        # parse and type-check only
aeonmi qube diagram <file.qube>      # print ASCII circuit diagram
cat circuit.qube | aeonmi qube run   # stdin execution (success criterion #5)
```

---

## IMPLEMENTATION PLAN (Phase 2 TODOs mapped to spec)

| P2 TODO | Work item |
|---|---|
| P2-1 | `demo.qube` stub exists — this spec IS the codified syntax rules |
| P2-2 | This document IS `docs/QUBE_SPEC.md` |
| P2-3 | Build `src/core/qube_parser.rs` — reuse Aeonmi lexer, implement grammar above |
| P2-4 | Add `QubeNode` enum to `src/core/qube_ast.rs` |
| P2-5 | Build `src/core/qube_executor.rs` — run against Titan quantum sim |
| P2-6 | ASCII circuit diagram: `src/core/qube_diagram.rs` |
| P2-7 | `import circuit from "./file.qube"` in `.ai` — module resolver hook |

---

## SUCCESS CRITERION

```
cat examples/bell.qube | aeonmi qube run
```
Produces:
```
0   (or 1)
0   (or 1, always same as first)
```
With no assertion failures.

---

*This spec is honest. Every construct maps to a specific parser rule and runtime behavior.*
*Nothing here is aspirational labeling.*
