# Aeonmi Language & Tooling Manual

> Version: 0.2.x Native + JS Backends (Current Implemented Subset)
> Scope: Practical reference from first line to advanced patterns with current feature availability. Items marked (Planned) are not yet implemented.

---
## Table of Contents
1. Philosophy & Design Goals
2. Quick Start (10 Lines)
3. Syntax Essentials
4. Lexical Elements
5. Types (Current & Emerging)
6. Variables & Scope
7. Expressions & Operators
8. Control Flow
9. Functions & Returns
10. Built‑ins (Standard Runtime)
11. Randomness Patterns with `%`
12. Emulating Collections Pre‑Arrays
13. String Construction & Formatting
14. Diagnostics & Error Patterns
15. Semantic Analysis & `--no-sema`
16. Native vs JS Execution Matrix
17. Performance Considerations (Micro)
18. Practical Recipes
19. Quantum / Glyph Preview (Feature‑gated)
20. Roadmap & Migration Notes
21. Troubleshooting Decision Tree
22. Glossary

---
## 1. Philosophy & Design Goals
Aeonmi targets AI‑native, security‑aware, multi‑tier execution (native VM, JS transpile, future quantum backends). Early iterations prioritize a minimal, predictable core while staging richer data structures, optimization, and symbolic (QUBE) glyph constructs.

Principles:
- Deterministic Core: simple semantic model first.
- Progressive Disclosure: introduce advanced constructs only after stable basics.
- Native First Option: no hard dependency on Node for iteration.
- Instrumentable: metrics & debug signals are first-class.

## 2. Quick Start (10 Lines)
```ai
log("Aeonmi ready");
let x = 2;
let y = 3;
let z = x + y;
if (z > 4) { log("big"); } else { log("small"); }
let i = 0;
while (i < 3) { log("i=" + i); i = i + 1; }
log("rand=" + rand());
log("time=" + time_ms());
```
Run native:
```powershell
Aeonmi.exe run --native demo.ai
```

## 3. Syntax Essentials
| Category | Implemented | Notes |
|----------|-------------|-------|
| `let` variable decl | Yes | Classical binding; reassign with bare name. |
| Quantum decl `⟨q⟩ ← |0⟩` | Yes | Allocates a qubit in the named basis state. |
| Superposition `⟨ψ⟩ ∈ |0⟩ + |1⟩` | Yes | Initializes qubit with literal amplitudes. |
| Tensor bind `⟨reg⟩ ⊗ [|0⟩, |1⟩]` | Yes | Creates quantum array of per-element qubits. |
| Block `{ ... }` | Yes | New scope for locals. |
| `if (cond) {}` / `else` | Yes | Parens required. |
| `while (cond)` | Yes | Standard loop. |
| Function decl | Partial | Depends on current branch: if unsupported, avoid `fn`. |
| Return | If functions enabled | No implicit last-expression return yet. |
| Arrays `[...]` | Planned | Use patterns (Sections 11–12). |
| `%` modulo | Yes | Native remainder; see Section 11 for bucket patterns. |
| Comments | Yes | Line: `# ...` only. |

## 4. Lexical Elements
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`
- Numbers: integer literals (no float token guarantee yet).
- Strings: `"..."` (keep ASCII simple; escaping minimal).
- Whitespace: spaces, tabs, newlines separate tokens.
- Reserved (future): `fn`, `return`, `for`, `break`, `continue` (some may parse but not execute if not enabled).

## 5. Types (Current & Emerging)
Current concrete runtime types in native VM:
- Number (integer semantics; division truncates toward zero).
- String.
- Boolean (if implemented; else emulate with 0/1).
- Qubit / Quantum state references (feature `quantum`).
- Quantum arrays / tensors (feature `quantum`).
- (Planned) Classical Array, Record / Object literals.

## 6. Variables & Scope
`let` introduces a binding in the current block. Reassignment allowed without `let`:
```ai
let count = 0;
count = count + 1;
```
Inner scopes shadow outer:
```ai
let x = 5;
if (x > 0) { let x = 1; log(x); } # prints 1
log(x); # prints 5
```

## 7. Expressions & Operators
| Group | Operators | Notes |
|-------|-----------|-------|
| Arithmetic | `+ - * / %` | `/` truncates toward zero; `%` uses that quotient. |
| Comparison | `== != < <= > >=` | Booleans / numeric truthiness. |
| Logical | `! && ||` (if present) | If absent, nest `if`. |
| Grouping | `( expr )` | Needed for precedence clarity. |
| Concatenation | `+` | Number auto stringifies in concat. |

## 8. Control Flow
`if (cond) { ... } else { ... }`
`while (cond) { ... }`
Pattern – fixed loop:
```ai
let i = 0;
while (i < 5) { ...; i = i + 1; }
```

Quantum-aware control adds probabilistic branches and guarded loops:
```ai
⊖ true ≈ 0.5 ⇒ { log("Heads"); } ⊕ { log("Tails"); }
⟲ measure(q) ⪰ 0.05 ⇒ { superpose(q); }
⚡ { superpose(q); entangle(q, r); } ⚠️ ≈ 0.1 ⇒ { log("retry"); } ✓ { log("done"); }
```
Probabilistic branches sample the indicated probability when deciding the active block. Quantum loops reevaluate the condition each iteration (the decoherence threshold is currently advisory). The try/catch form executes the attempt block; catch/finally hooks are parsed and reserved for forthcoming error simulation.

## 9. Functions & Returns
If available in your build:
```ai
fn add(a, b) { return a + b; }
log(add(2, 3));
```
If not yet implemented, keep logic inline or simulate with pattern dispatch using `if` chains.

## 10. Built‑ins
| Name | Purpose |
|------|---------|
| `log(v)` | Print with newline. |
| `print(v)` | (Alias, if present). |
| `rand()` | Pseudo random integer. |
| `time_ms()` | Millisecond timestamp. |
| `len(v)` | Length of strings, arrays, or objects (0 for null). |
| `superpose(q)` | Apply Hadamard; creates qubit if missing. |
| `entangle(a, b)` | Mark/apply two-qubit entanglement. |
| `measure(q)` | Collapse qubit; returns `0` or `1`. |
| `is_entangled(a, b)` | Boolean if qubits share entanglement set. |
| `apply_matrix(q, [[a,b],[c,d]])` | Apply custom single-qubit gate. |

## 11. Randomness Patterns with `%`
Modulo simplifies bucket selection. Combine `rand()` with `%` to produce bounded ranges without chained division.

```ai
let bucket = rand() % 10;   # values 0-9

if (bucket < 3) {
  log("Group Alpha");
} else if (bucket < 6) {
  log("Group Beta");
} else {
  log("Group Gamma");
}
```

Need a fixed number of outcomes? Guard against legacy configs:

```ai
let choices = 5;
let pick = rand() % choices;
if (pick >= choices) { pick = choices - 1; } # defensive for legacy builds
```

## 12. Emulating Collections
Classical arrays remain planned; quantum tensors are separate (Section 19). Before arrays, encode via functions:
```ai
fn show_fact(n) {
  if (n == 0) { log("Honey never spoils."); return; }
  if (n == 1) { log("Octopuses have three hearts."); return; }
  if (n == 2) { log("Bananas are berries; strawberries aren't."); return; }
  if (n == 3) { log("A day on Venus is longer than a year on Venus."); return; }
  log("Wombat poop is cube-shaped.");
}
```

## 13. Strings
Concatenate with `+`. No interpolation yet:
```ai
let user = "Traveler";
log("Hi, " + user + "!");
```

## 14. Diagnostics
Common messages & meanings:
| Message | Cause | Remedy |
|---------|-------|--------|
| `Lexing error: Unexpected character '['` | Arrays not implemented | Remove `[ ]` use pattern §12. |
| `Lexing error: Unexpected character '%'` | Running an older build without modulo | Upgrade to the current shard or remove `%`. |
| `Parsing error: Expected '(' after if` | Missing parentheses | Add `( )`. |
| Runtime error: <msg> | Interpreter failure | Add `log()` around suspicious values. |
| `Runtime error: Quantum error: ...` | Invalid qubit name, out-of-range matrix, etc. | Ensure qubit exists; normalize gate matrix; check tensor bounds. |

Enable pretty: `--pretty-errors`.

## 15. Semantic Analysis
`--no-sema` skips semantic validation (faster iteration, fewer early errors). Use only when exploring known-good patterns.

## 16. Native vs JS
| Aspect | JS Transpile | Native VM |
|--------|--------------|-----------|
| Startup | Node spin-up | Direct |
| Feature Coverage | Historically broader | Growing parity |
| Debug Toggle | Classic JS toolchain | `AEONMI_DEBUG=1` internal logs |
| Dependency | Requires Node | None (post-build) |

## 17. Performance (Micro)
Guidelines:
- Minimize nested string concatenations in hot loops; reuse computed fragments.
- Avoid deep call chains (until TCO/optimizations added).
- Prefer single pass loops over multi‑condition splitting.

## 18. Recipes
### 18.1 Greeting + Fact + Quote
(See README quick example or Section 11 pattern infused twice.)

### 18.2 Counter with Timestamp
```ai
let i = 0;
while (i < 3) {
  log("tick " + i + " at ms=" + time_ms());
  i = i + 1;
}
```

### 18.3 Simple Absolute Value (no functions version)
```ai
let n = -5;
if (n < 0) { n = 0 - n; }
log(n);
```

## 19. Quantum / Glyph (Preview)
Enable the shard with `--features quantum` to unlock qubit-aware syntax, glyphs, and Titan simulator backing. The constructs below now execute in the native VM; the JS backend treats them as no-ops unless stated.

### 19.1 Declaring Qubits & States
```ai
⟨q0⟩ ← |0⟩;                 # basis state
⟨q1⟩ ← |1⟩;
⟨psi⟩ ∈ |0⟩ + |1⟩;          # literal superposition
⟨approx⟩ ≈ |0⟩;              # advisory approximation binding
```
Bindings emit qubit references in the environment. Basis literals support `|0⟩`, `|1⟩`, `|+⟩`, `|-⟩`; concatenated expressions (with optional `/* amplitude: f64 */` comments) map to normalized superpositions.

### 19.2 Tensor & Quantum Arrays
```ai
⟨reg⟩ ⊗ [|0⟩, |1⟩, |+⟩];
log(measure(reg⟦2⟧));
```
Tensor binding allocates per-element qubits (`reg[0]`, `reg[1]`, ...) and stores them in `Value::QuantumArray`. Index with brackets or the quantum `⟦ idx ⟧` form; both respect bounds.

### 19.3 Operations & Introspection
```ai
superpose(q0);                    # Hadamard
entangle(q0, q1);                 # track entanglement set
log(is_entangled(q0, q1));        # true if same register
apply_matrix(psi, [[0.7071, 0.7071],[0.7071, -0.7071]]);
let shot = measure(q1);           # collapses and returns 0/1
```
All operations target the Titan `QuantumSimulator`. `apply_matrix` accepts numeric literals or precomputed matrices (ensure normalization). Hooks exist for forwarding the captured circuit to external backends such as Qiskit via `gui/` integration.

### 19.4 Control Flow Glyphs
```ai
⊖ true ≈ 0.5 ⇒ { log("Heads"); } ⊕ { log("Tails"); }
⟲ measure(q0) ⪰ 0.02 ⇒ { superpose(q0); }
⚡ { entangle(q0, q1); } ⚠️ ≈ 0.1 ⇒ { log("retry"); } ✓ { log("done"); }
```
Probabilistic branches sample the annotated probability. Loops reevaluate the condition each iteration; the decoherence threshold is monitored but currently advisory. The quantum try/catch syntax executes the attempt block and reserves recovery hooks for the forthcoming fault model.

Examples under `examples/quantum_demo.rs` illustrate these primitives, including teleportation and Grover sketches.

## 20. Roadmap
Upcoming priorities (subject to change):
1. Array literals & indexing.
2. Bitwise operators and extended arithmetic helpers.
3. Function enhancements (default args, recursion optimizations).
4. Structured records / pattern matching prototypes.
5. Optimized bytecode path alignment (if feature enabled).

## 21. Troubleshooting Decision Tree
```
Error? -> Lexing? -> Remove unsupported char -> Re-run
      \-> Parsing? -> Check parentheses / block braces
      \-> Runtime? -> Add log probes -> Simplify input -> File issue if minimal repro
Silent? -> Add log("START") -> Confirm file path & command order
Random stuck? -> Replace rand() with fixed value for reproducibility
```

## 22. Glossary
| Term | Definition |
|------|------------|
| Native VM | Tree-walk interpreter directly executing lowered IR. |
| Lowering | Transform from parsed AST to IR consumed by native/bytecode/JS backends. |
| IR | Intermediate Representation, simplified structure for execution. |
| QUBE | Planned symbolic/hieroglyphic layer for advanced adaptive semantics. |
| Semantic Analysis | Static validation phase (names, simple type constraints). |
| Bytecode | Alternative compiled form (feature gated) for performance experiments. |

---
End of Manual. For clarifications open an issue or request an expansion.
