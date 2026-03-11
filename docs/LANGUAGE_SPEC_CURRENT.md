# AEONMI Language Specification — Current State
### What actually parses and runs today
**Version:** 1.0.0-bootstrap | **Date:** March 2026 | **Status:** Phase 0 complete, Phase 1 in progress

---

## 1. File Extensions

| Extension | Purpose | Status |
|-----------|---------|--------|
| `.ai`     | Aeonmi source files | ✅ Fully supported |
| `.qube`   | QUBE quantum circuit files | ✅ Parsed & executed |

---

## 2. Execution Paths

```
.ai file → Lexer → Parser → AST → CodeGenerator → JS → Node.js
.ai file → Lexer → Parser → AST → Lowering → IR → VM (native, AEONMI_NATIVE=1)
.qube file → QUBE Lexer → QUBE Parser → QUBE AST → QUBE Executor
```

Default path is JS codegen via `aeonmi run`. Native interpreter via `aeonmi exec`.

---

## 3. Variables & Literals

### Variable Declaration
```
let x = 42;
let name = "hello";
const PI = 3.14159;
```
- `let` and `const` both declare variables (const has no enforcement yet)
- Semicolons required after `let`/`const` declarations

### Number Literals
```
42          // integer
3.14        // float
𝍠𝍡𝍢        // numeric glyphs (U+1D360-1D369)
```

### String Literals
```
"hello world"
"escape: \n \t \\ \""
"\u{1F600}"          // unicode escape
f"text {variable}"   // f-string (lexed as single string, interpolation not yet evaluated)
```

### Boolean & Null
```
true
false
null
```

### Array Literals
```
[1, 2, 3]
["a", "b", "c"]
```

### Qubit Literals
```
|0⟩    |1⟩    |+⟩    |-⟩    |ψ⟩
|0>                              // also accepted, normalized to ⟩
```

---

## 4. Operators

### Arithmetic
`+`  `-`  `*`  `/`  `%`

### Comparison
`==`  `!=`  `<`  `<=`  `>`  `>=`

### Logical
`&&`  `||`  `!`

### Assignment
`=`  `:=` (colon-equals also supported)

### Quantum Operators (Unicode)
| Symbol | Name | TokenKind |
|--------|------|-----------|
| `←`    | Quantum bind | QuantumBind |
| `∈`    | Quantum membership | QuantumIn |
| `⊗`    | Tensor product | QuantumTensor |
| `≈`    | Approximation | QuantumApprox |
| `⊕`    | Quantum XOR | QuantumXor |
| `⊖`    | Quantum OR | QuantumOr |
| `⊄`    | Quantum NOT | QuantumNot |
| `∇`    | Gradient | QuantumGradient |
| `⪰`    | Quantum GEQ | QuantumGeq |
| `⪯`    | Quantum LEQ | QuantumLeq |
| `⇒`    | Implies | QuantumImplies |
| `⟲`    | Quantum loop | QuantumLoop |
| `◊`    | Quantum modulo | QuantumModulo |

### Other Operators
`->`  `=>`  `::`  `.`  `&`

---

## 5. Control Flow

### If / Else
```
// C-style (parentheses)
if (x > 10) {
    log("big");
} else {
    log("small");
}

// Rust-style (no parentheses)
if x > 10 {
    log("big");
}
```

### While Loop
```
while (i < 10) { i = i + 1; }
while i < 10 { i = i + 1; }      // parens optional
```

### For Loop (C-style)
```
for (let j = 0; j < 3; j = j + 1) {
    log(j);
}
```

### For-In Loop (Rust-style)
```
for item in collection {
    log(item);
}
```
Note: Currently lowers to a variable declaration + body block. Full iteration not yet implemented.

### Return
```
return value;
return;          // bare return (returns null)
```

---

## 6. Functions

### Regular Functions
```
function add(a, b) {
    return a + b;
}
```

### Quantum Functions
```
quantum function optimize(data) {
    qubit q;
    superpose(q);
    let result = measure(q);
    return result;
}
```

### Async Functions
```
async function fetch_data(url) {
    let result = await get(url);
    return result;
}
```
Note: Parsed and emitted as `async function` in JS. No cooperative scheduler in native path.

### Parameter Type Annotations (parsed, not enforced)
```
function greet(name: String, age: usize) { }
quantum function process(data: Vec<f64>) -> Result<()> { }
```
Type annotations are consumed by the parser and discarded. No type checking.

---

## 7. Quantum Operations

### Qubit Declaration
```
qubit q;                    // declares q as a qubit (string name internally)
```

### Built-in Quantum Functions
```
superpose(q);               // put qubit in superposition
let bit = measure(q);       // collapse and get 0 or 1
entangle(q1, q2);          // entangle two qubits (metadata only — no real CNOT)
apply_gate(q, H);          // apply gate: H, X, Y, Z, CNOT
dod(q);                    // alias for measure
```

### Quantum Gates (available as constants in JS runtime)
`H`  `X`  `Y`  `Z`  `CNOT`  `HADAMARD`  `NOT`

---

## 8. Structs, Enums, Impl

### Struct Declaration
```
struct Point {
    x: f64,
    y: f64,
}

quantum struct QubitRegister {
    size: usize,
    state: String,
}
```

### Enum Declaration
```
enum Color {
    Red,
    Green,
    Blue,
    Custom(String),
}
```

### Impl Blocks
```
impl Point {
    function distance(other) {
        return 0;
    }
}
```

---

## 9. Match Expressions

```
match value {
    0 => { log("zero"); },
    1 => { log("one"); },
    * => { log("other"); },
}
```
- Supports: literal patterns, identifier patterns, wildcard (`*`)
- NOT yet supported: match guards (`pat if cond =>`), enum variant patterns with `::`

---

## 10. Method Calls & Field Access

```
obj.method(arg1, arg2);     // method call
obj.field;                  // field access
obj.field = value;          // field assignment
Type::new();                // static/constructor call
Type::method(args);         // static method call
expr::<Type>();             // turbofish (type params skipped)
```

---

## 11. Import System

```
import { Lexer, Parser } from "./module";
import DefaultExport from "./path";
```
- Parsed into `ImportDecl` AST node
- JS backend emits as comment (no module resolution in JS path)
- Native interpreter resolves relative `.ai` files and executes them

---

## 12. Quantum-Native Syntax (Unicode)

### Quantum Variable Declaration
```
⟨x⟩ ← 42              // classical binding
⟨psi⟩ ∈ |0⟩ + |1⟩     // superposition binding
⟨tensor⟩ ⊗ value       // tensor binding
⟨approx⟩ ≈ 0.707       // approximation binding
```

### Quantum Function Markers
```
◯ classical_func⟨x, y⟩ { }    // classical function
⊙ quantum_func⟨q⟩ { }         // quantum function
🧠 ai_func⟨data⟩ { }           // AI neural function
```

### Quantum Control Flow
```
⊖ condition ≈ 0.8 ⇒ { }       // probability branch
⟲ condition ⪰ 0.5 ⇒ { }       // quantum loop with decoherence
⚡ { } ⚠ ≈ 0.1 ⇒ { } ✓ { }   // quantum try/catch/success
⏰ duration ⇒ { }               // time block
```

### Quantum Comments
```
∴ therefore comment
∵ because comment
※ note comment
```

### Hieroglyphic Operations
```
𓀀(x, y);      // hieroglyphic function call → __glyph runtime
```
Any Unicode character > U+07FF used as a function name routes through the `__glyph` runtime.

---

## 13. Quantum Blocks (Generic)

```
quantum command_line MyApp {
    version: "1.0";
    description: "text";

    function run() { }
    quantum function optimize() { }
}
```
- `quantum <tag> <Name> { body }` parses any quantum-prefixed block
- Body supports `field: type = value;` declarations and regular statements
- Emitted as a JS block with variable declarations

---

## 14. QUBE (.qube) Syntax

```
state psi = |0⟩
apply H -> psi
collapse psi -> result
assert result in {0, 1}
```

### QUBE Statements
| Statement | Syntax |
|-----------|--------|
| State declaration | `state name = \|qubit⟩` |
| Gate application (1-qubit) | `apply GATE -> target` |
| Gate application (2-qubit) | `apply GATE -> control, target` |
| Collapse/measure | `collapse name -> result` |
| Assert | `assert name in {values}` |

### QUBE Gates
`H`  `X`  `Y`  `Z`  `CNOT`  `CZ`  `SWAP`  `T`  `S`

---

## 15. CLI Commands

| Command | Purpose | Status |
|---------|---------|--------|
| `aeonmi run <file.ai>` | Compile to JS and execute with Node | ✅ |
| `aeonmi exec <file.ai>` | Execute via native interpreter | ✅ |
| `aeonmi compile <file.ai>` | Compile to JS file | ✅ |
| `aeonmi repl` | Interactive REPL | ✅ |
| `aeonmi vault init` | Create encrypted vault + glyph | ✅ |
| `aeonmi vault add` | Add card to vault | ✅ |
| `aeonmi mint --file <f>` | Generate NFT metadata JSON | ✅ |
| `aeonmi quantum` | Quantum operations CLI | ✅ |
| `aeonmi qube run` | Execute .qube files | ✅ |

---

## 16. What Does NOT Work Yet

These are syntax constructs the full Shard (`main_full.ai`) needs but the parser doesn't handle:

| Feature | Example | Roadmap |
|---------|---------|---------|
| Closures | `\|x\| -> { body }` | Phase 1 |
| Match guards | `pat if cond => body` | Phase 1 |
| Destructuring let | `let (a, b) = expr` | Phase 1 |
| Postfix `.await` | `expr.await` | Phase 1 |
| Chained `::` paths | `std::process::exit()` | Phase 1 |
| Rust macros | `println!("text")` | Phase 1 |
| Named arguments | `func(key: value)` | Phase 1 |
| Array push/pop | `arr.push(x)` | Phase 1 |
| Real multi-qubit entanglement | CNOT state math | Phase 2 |
| File I/O in .ai | Read/write files from Aeonmi code | Phase 3 |
| Real async runtime | Cooperative scheduler | Phase 5 |

---

## 17. Test Suite

**135/135 tests passing** as of March 2026.

Key test modules:
- `core::ast::tests` — AST node construction
- `core::blockchain::tests` — Merkle chain integrity
- `core::code_generator::tests` — JS codegen output
- `core::compiler::tests` — Full pipeline
- `core::quantum_algorithms::tests` — Grover, Shor, Deutsch-Jozsa, teleportation
- `core::quantum_simulator::tests` — State vectors, gates
- `glyph::*` — MGK, UGST, GDF, vault, anomaly detection
- `mother::*` — Embryo loop, emotional core, neural network, quantum attention
- `qube::*` — Lexer, parser, executor

---

## 18. Success Criteria (All Passing)

1. `aeonmi exec examples/hello.ai` → prints `42` ✅
2. `aeonmi exec examples/quantum.ai` → prints measured qubit result ✅
3. `aeonmi run shard/src/main.ai` → Shard compiler output ✅
4. `aeonmi run examples/quantum_glyph.ai` → glyph render ✅
5. `cat demo.qube | aeonmi qube run` → quantum circuit result ✅
6. `aeonmi vault init` → encrypted vault + glyph ✅
7. `aeonmi mint --file output.ai` → valid NFT metadata JSON ✅
