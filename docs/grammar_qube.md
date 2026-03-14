# QUBE Grammar Reference

QUBE (Quantum Universal Base Engine) is the quantum symbolic layer of Aeonmi.

QUBE files use the `.qube` extension and are executed via:

```
aeonmi qube run <file.qube>
```

---

## Formal Grammar (EBNF)

```ebnf
program        ::= statement* EOF

statement      ::= state_decl
                 | gate_apply
                 | collapse
                 | assert_stmt
                 | log_stmt
                 | comment

state_decl     ::= "state" IDENT "=" amplitude_expr
                 | "state" IDENT "=" qubit_literal

amplitude_expr ::= amplitude_term (("+" | "-") amplitude_term)*

amplitude_term ::= NUMBER? qubit_literal
                 | IDENT

qubit_literal  ::= "|" (IDENT | NUMBER | "+" | "-") "⟩"
                 | "|" (IDENT | NUMBER | "+" | "-") ">"

gate_apply     ::= "apply" gate_name "->" IDENT
                 | "apply" gate_name "(" IDENT ("," IDENT)* ")"

gate_name      ::= "H" | "X" | "Y" | "Z" | "CNOT"
                 | "T" | "S" | "Rx" | "Ry" | "Rz"
                 | IDENT

collapse       ::= "collapse" IDENT "->" IDENT

assert_stmt    ::= "assert" IDENT "∈" "{" assert_value ("," assert_value)* "}"
                 | "assert" IDENT "==" assert_value

assert_value   ::= NUMBER | qubit_literal | IDENT

log_stmt       ::= "log" "(" expr ")"
                 | "print" "(" expr ")"

comment        ::= "//" ANY* NEWLINE
                 | "∴" ANY* NEWLINE
                 | "∵" ANY* NEWLINE

expr           ::= IDENT | NUMBER | STRING | qubit_literal

IDENT          ::= [a-zA-Z_α-ωΑ-Ω] [a-zA-Z0-9_α-ωΑ-Ω]*
NUMBER         ::= [0-9]+ ("." [0-9]+)?
STRING         ::= '"' [^"]* '"'
```

---

## Tokens

| Token         | Description                            | Examples                  |
|---------------|----------------------------------------|---------------------------|
| `state`       | Keyword: declare a quantum state       | `state q = |0⟩`           |
| `apply`       | Keyword: apply a gate to a state       | `apply H -> q`            |
| `collapse`    | Keyword: measure and bind result       | `collapse q -> r`         |
| `assert`      | Keyword: verify a classical value      | `assert r ∈ {0, 1}`       |
| `log` / `print` | Keyword: output a value              | `log(r)`                  |
| `\|0⟩`, `\|1⟩`  | Qubit basis state literals           | `|+⟩`, `|-⟩`              |
| `∴`           | Line comment (therefore)               | `∴ Bell state setup`      |
| `∵`           | Reason comment (because)               | `∵ entangled pair`        |
| `∈`           | Set membership assertion               | `assert r ∈ {0, 1}`       |

---

## Gate Reference

| Gate    | Qubits  | Matrix / Action                           |
|---------|---------|-------------------------------------------|
| `H`     | 1       | Hadamard — equal superposition            |
| `X`     | 1       | Pauli-X — bit flip                        |
| `Y`     | 1       | Pauli-Y — bit + phase flip                |
| `Z`     | 1       | Pauli-Z — phase flip                      |
| `S`     | 1       | Phase gate (π/2)                          |
| `T`     | 1       | T gate (π/4)                              |
| `Rx`    | 1       | Rotation around X axis (angle in radians) |
| `Ry`    | 1       | Rotation around Y axis                    |
| `Rz`    | 1       | Rotation around Z axis                    |
| `CNOT`  | 2       | Controlled-NOT (first arg = control)      |
| `CZ`    | 2       | Controlled-Z                              |
| `SWAP`  | 2       | Swap two qubit states                     |

---

## Examples

### Pure state

```qube
state q = |0⟩
apply X -> q
collapse q -> r
assert r == 1
log(r)
```

### Superposition and measurement

```qube
state ψ = 0.707|0⟩ + 0.707|1⟩
collapse ψ -> result
assert result ∈ {0, 1}
log(result)
```

### Bell state (two-qubit entanglement)

```qube
∴ Bell state: maximally entangled pair

state q0 = |0⟩
state q1 = |0⟩

apply H -> q0
apply CNOT(q0, q1)

collapse q0 -> r0
collapse q1 -> r1

∵ a Bell pair always collapses to the same value
assert r0 ∈ {0, 1}
assert r1 ∈ {0, 1}

log(r0)
log(r1)
```

### GHZ state (three-qubit entanglement)

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

---

## CLI Commands

```
aeonmi qube run <file.qube>         execute a QUBE program
aeonmi qube check <file.qube>       parse and type-check only
aeonmi qube diagram <file.qube>     print ASCII circuit diagram
```

---

## Importing QUBE in .ai files

```ai
import circuit from "./bell.qube"
circuit.run()
```

---

## Execution Model

1. States are complex state vectors (2ⁿ dimensions for n qubits).
2. Gates are unitary matrix operations backed by the Titan quantum simulator.
3. `collapse` performs a projective measurement using the Born rule.
4. `assert` checks classical results after measurement; fails with a non-zero exit if false.
5. `log` / `print` writes the value to stdout.

---

See also: `docs/QUBE_SPEC.md`, `examples/bell.qube`, `examples/demo.qube`.
