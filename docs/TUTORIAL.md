# ⧉ Aeonmi Complete Beginner Tutorial
## *How to Code in `.ai` and `.qube` — From Zero to Quantum*

**Version:** 1.0.0-quantum-consciousness  
**Audience:** Complete beginners with no prior Aeonmi experience  
**Prerequisites:** See Section 0 below

---

## Table of Contents

- [Section 0 — Prerequisites & Installation](#section-0--prerequisites--installation)
- [Section 1 — Your First `.ai` Program](#section-1--your-first-ai-program)
- [Section 2 — Variables and Data Types](#section-2--variables-and-data-types)
- [Section 3 — Functions and Closures](#section-3--functions-and-closures)
- [Section 4 — Control Flow](#section-4--control-flow)
- [Section 5 — Arrays, Objects, and Genesis Arrays](#section-5--arrays-objects-and-genesis-arrays)
- [Section 6 — String Interpolation (F-Strings)](#section-6--string-interpolation-f-strings)
- [Section 7 — Pattern Matching](#section-7--pattern-matching)
- [Section 8 — Genesis Glyphs](#section-8--genesis-glyphs)
- [Section 9 — File I/O Builtins](#section-9--file-io-builtins)
- [Section 10 — Introduction to `.qube`](#section-10--introduction-to-qube)
- [Section 11 — Quantum Gates Explained](#section-11--quantum-gates-explained)
- [Section 12 — Your First Complete Quantum Circuit](#section-12--your-first-complete-quantum-circuit)
- [Section 13 — Built-in Quantum Algorithms](#section-13--built-in-quantum-algorithms)
- [Section 14 — Mixing `.ai` and `.qube`](#section-14--mixing-ai-and-qube)
- [Section 15 — The AI Canvas Editor](#section-15--the-ai-canvas-editor)
- [Section 16 — Vault, Web3, and Beyond](#section-16--vault-web3-and-beyond)
- [Section 17 — Complete Reference Card](#section-17--complete-reference-card)
- [Section 18 — Troubleshooting FAQ](#section-18--troubleshooting-faq)

---

## Section 0 — Prerequisites & Installation

### What You Need to Know First

Before starting, you should be comfortable with:
- Using a **terminal / command prompt**  
- Basic **programming concepts** (variables, functions, loops)  
- You do NOT need to know Rust, quantum physics, or blockchain

### Hardware Requirements

| Minimum | Recommended |
|---------|-------------|
| 2 GB RAM | 8 GB RAM |
| 2-core CPU | 4-core CPU |
| 1 GB disk | 4 GB disk |
| Linux / macOS / Windows (WSL) | Ubuntu 22.04 LTS |

### Step 1 — Install Rust

Aeonmi is built with Rust. Install it first:

```bash
# On Linux/macOS:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then reload your shell:
source "$HOME/.cargo/env"

# Verify:
rustc --version    # should show rustc 1.75 or later
cargo --version
```

On Windows, download `rustup-init.exe` from https://rustup.rs

### Step 2 — Clone and Build Aeonmi

```bash
# Clone the repository
git clone https://github.com/Aeonmi-Aysa/aeonmi.git
cd aeonmi

# Build (takes 2-5 minutes on first run)
cargo build --no-default-features --features "quantum,mother-ai"

# Verify the build
./target/debug/Aeonmi --version
# or on Windows: target\debug\Aeonmi.exe --version
```

### Step 3 — Add Aeonmi to Your PATH (optional but recommended)

```bash
# Add to ~/.bashrc or ~/.zshrc:
export PATH="$HOME/path/to/aeonmi/target/debug:$PATH"

# Then reload:
source ~/.bashrc

# Now you can just type:
Aeonmi --help
```

### Step 4 — Install VS Code Extension (optional)

If you use Visual Studio Code, install the `vscode-aeonmi` extension for syntax highlighting:
```bash
cd vscode-aeonmi
npm install
npm run package   # produces aeonmi-language-support.vsix
# In VS Code: Extensions → ... → Install from VSIX
```

### Step 5 — Run the Test Suite (verify everything works)

```bash
cargo test --no-default-features --features "quantum,mother-ai"
# Expected: 165+ tests pass, 0 failures
```

---

## Section 1 — Your First `.ai` Program

Create a file called `hello.ai`:

```ai
// hello.ai — my first Aeonmi program

let greeting = "Hello, Quantum World!"
log(greeting)
```

Run it:
```bash
Aeonmi run hello.ai
```

Expected output:
```
native: executing hello.ai
Hello, Quantum World!
```

### What Just Happened?

| Code | Meaning |
|------|---------|
| `// comment` | A comment — Aeonmi ignores everything after `//` on a line |
| `let greeting = "..."` | Declare a variable named `greeting` with a string value |
| `log(greeting)` | Print the value to the screen |

> **Note:** Semicolons are **optional** in Aeonmi. You can add them if you like, but they are not required. The language uses newlines to separate statements.

---

## Section 2 — Variables and Data Types

### Basic Types

```ai
// Numbers (integers and decimals both work)
let age = 25
let pi = 3.14159
let negative = -42

// Strings
let name = "Aeonmi"
let empty_string = ""

// Booleans
let is_quantum = true
let is_classical = false

// Null (no value)
let nothing = null
```

### Mutable Variables

By default, you can reassign any variable:

```ai
let x = 10
log(x)      // prints: 10

x = 20
log(x)      // prints: 20
```

You can optionally add `mut` to be explicit:
```ai
let mut counter = 0
counter = counter + 1
log(counter)    // prints: 1
```

### Variable Naming Rules

- Names can contain letters, digits, and underscores
- Names cannot start with a digit
- Aeonmi is case-sensitive: `myVar` and `myvar` are different
- Unicode identifiers are supported: `let 量子 = "quantum"`

### Arithmetic

```ai
let a = 10
let b = 3

log(a + b)    // 13
log(a - b)    // 7
log(a * b)    // 30
log(a / b)    // 3.333...
log(a % b)    // 1   (modulo/remainder)
```

### String Concatenation

```ai
let first = "Hello"
let second = " World"
let full = first + second
log(full)     // Hello World
```

---

## Section 3 — Functions and Closures

### Defining a Function

The keyword is `function` (not `fn` like in Rust):

```ai
function greet(name) {
    log(f"Hello, {name}!")
}

greet("Aeonmi")    // Hello, Aeonmi!
greet("World")     // Hello, World!
```

### Functions with Return Values

```ai
function add(a, b) {
    return a + b
}

let result = add(5, 3)
log(result)     // 8
```

### Functions Calling Functions

```ai
function square(x) {
    return x * x
}

function sum_of_squares(a, b) {
    return square(a) + square(b)
}

log(sum_of_squares(3, 4))    // 9 + 16 = 25
```

### Closures (Anonymous Functions)

A closure is a function without a name, stored in a variable:

```ai
let double = function(x) {
    return x * 2
}

log(double(5))    // 10
log(double(7))    // 14
```

Arrow-style closures (single expression):
```ai
let triple = function(x) => x * 3

log(triple(4))    // 12
```

### Closures Capture Their Environment

This is a powerful feature — closures "remember" variables from the outer scope:

```ai
let multiplier = 5

let multiply_by_five = function(x) {
    return x * multiplier    // captures 'multiplier' from outer scope
}

log(multiply_by_five(3))    // 15
log(multiply_by_five(7))    // 35
```

### Recursive Functions

```ai
function factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

log(factorial(5))    // 120
log(factorial(10))   // 3628800
```

---

## Section 4 — Control Flow

### If / Else

```ai
let temperature = 25

if temperature > 30 {
    log("It's hot!")
} else if temperature > 20 {
    log("It's warm.")
} else {
    log("It's cool.")
}
// Output: It's warm.
```

Both C-style `if (condition)` and Rust-style `if condition` work:
```ai
if (x > 0) { log("positive") }    // C-style: parentheses OK
if x > 0 { log("positive") }      // Aeonmi-style: no parens needed
```

### While Loop

```ai
let count = 1
while count <= 5 {
    log(count)
    count = count + 1
}
// Output: 1 2 3 4 5
```

### For-In Loop

Iterate over an array:
```ai
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits {
    log(fruit)
}
// Output:
// apple
// banana
// cherry
```

Iterate over a range (using a helper):
```ai
let numbers = [1, 2, 3, 4, 5]
for n in numbers {
    log(n * n)
}
// Output: 1 4 9 16 25
```

Iterate over characters in a string:
```ai
let word = "hello"
for ch in word {
    log(ch)
}
// Output: h e l l o
```

### Destructuring in For-In

```ai
let pairs = [[1, "one"], [2, "two"], [3, "three"]]
for (num, name) in pairs {
    log(f"{num} is {name}")
}
```

---

## Section 5 — Arrays, Objects, and Genesis Arrays

### Arrays

```ai
let numbers = [10, 20, 30, 40, 50]

// Access by index (0-based)
log(numbers[0])    // 10
log(numbers[4])    // 50

// Length
log(len(numbers))  // 5

// Push (add to end)
push(numbers, 60)
log(len(numbers))  // 6

// Pop (remove from end)
let last = pop(numbers)
log(last)          // 60
```

### Array Methods

```ai
let nums = [1, 2, 3, 4, 5]

// Map: transform each element
let doubled = map(nums, function(x) => x * 2)
log(doubled)    // [2, 4, 6, 8, 10]

// Filter: keep elements that pass a test
let evens = filter(nums, function(x) => x % 2 == 0)
log(evens)      // [2, 4]
```

### Concatenating Arrays

```ai
let a = [1, 2, 3]
let b = [4, 5, 6]
let c = a + b
log(c)    // [1, 2, 3, 4, 5, 6]
```

### Objects (Key-Value Maps)

```ai
let person = {
    name: "Aeonmi",
    age: 30,
    is_quantum: true
}

log(person.name)      // Aeonmi
log(person["age"])    // 30

person.city = "Quantum City"
log(person.city)      // Quantum City
```

### Genesis Arrays `⧉[...]`

Genesis arrays are a special Aeonmi feature — they can represent quantum superpositions or data streams:

```ai
// A genesis array looks like a normal array but has quantum semantics
let glyph_set = ⧉[1, 2, 3, 4, 5]

// You can iterate over it just like a regular array
for item in glyph_set {
    log(item)
}
```

> Genesis arrays are designed for use with the genesis glyph operators (Section 8).

---

## Section 6 — String Interpolation (F-Strings)

F-strings let you embed expressions directly inside strings. Prefix the string with `f`:

```ai
let name = "Quantum"
let version = 2.0

// Basic interpolation
log(f"Welcome to {name} v{version}!")
// Output: Welcome to Quantum v2.0!

// Expressions work inside {}
let x = 5
log(f"The square of {x} is {x * x}")
// Output: The square of 5 is 25

// Function calls work too
function double(n) => n * 2
log(f"Double 7 is {double(7)}")
// Output: Double 7 is 14
```

### Multi-line F-strings

```ai
let first = "Jane"
let last = "Doe"
let age = 28

let bio = f"Name: {first} {last}\nAge: {age}\nLanguage: Aeonmi"
log(bio)
// Output:
// Name: Jane Doe
// Age: 28
// Language: Aeonmi
```

---

## Section 7 — Pattern Matching

The `match` expression is like a supercharged `switch` statement:

```ai
let status = 404

let message = match status {
    200 => "OK",
    404 => "Not Found",
    500 => "Server Error",
    _ => "Unknown Status"
}

log(message)    // Not Found
```

### Matching Strings

```ai
let command = "go"

match command {
    "go" => log("Moving forward"),
    "stop" => log("Halting"),
    "back" => log("Reversing"),
    _ => log(f"Unknown command: {command}")
}
```

### Matching in Conditions

```ai
function describe_number(n) {
    return match n {
        0 => "zero",
        1 => "one",
        _ => if n < 0 { "negative" } else { "large positive" }
    }
}

log(describe_number(0))    // zero
log(describe_number(1))    // one
log(describe_number(-5))   // negative
log(describe_number(100))  // large positive
```

---

## Section 8 — Genesis Glyphs

Genesis glyphs are special Unicode operators that give Aeonmi its unique quantum-inspired character. They are found on the keyboard through copy/paste or with your OS Unicode input method.

| Glyph | Name | Meaning |
|-------|------|---------|
| `⧉` | Genesis Mark | Opens a genesis array or marks a quantum datum |
| `‥` | Two-Dot | Range start (exclusive end): `1‥5` |
| `…` | Ellipsis | Range with spread / variadic |
| `↦` | Mapsto | Functional mapping arrow |
| `⊗` | Tensor Product | Quantum tensor product / Kronecker product |
| `⊕` | Direct Sum | Quantum XOR / superposition merge |
| `⊙` | Circle Dot | Hadamard product (element-wise) |
| `⊛` | Circled Asterisk | Convolution |
| `⊜` | Circled Equals | Equality in quantum context |
| `⟳` | Clockwise Arrow | Quantum loop / feedback |
| `⟴` | Anticlockwise Arrow | Reverse quantum loop |

### Using Genesis Glyphs

```ai
// Genesis array declaration
let qdata = ⧉[|0⟩, |1⟩, |+⟩]

// Tensor product (⊗) — symbolic, stored for later evaluation
let state = |0⟩ ⊗ |1⟩
log(state)    // prints the symbolic expression

// Direct sum (⊕)
let superpos = |0⟩ ⊕ |1⟩
log(superpos)
```

### How to Type Glyphs

**On macOS:**
- Enable Unicode Hex Input keyboard in System Preferences → Keyboard → Input Sources
- Then hold Option and type the Unicode code point

**On Linux:**
- Press `Ctrl+Shift+U`, type the hex code, press Enter
- `⧉` = U+29C9, `⊗` = U+2297, `⊕` = U+2295, `↦` = U+21A6

**On any system:**
- Copy from this document
- Use VS Code with the Aeonmi extension (glyph autocomplete planned)

---

## Section 9 — File I/O Builtins

Aeonmi has built-in functions for reading and writing files:

```ai
// Write a file
write_file("data.txt", "Hello, Aeonmi!")

// Read a file (returns the content as a string)
let content = read_file("data.txt")
log(content)    // Hello, Aeonmi!

// Append to a file
append_file("data.txt", "\nLine 2!")

// Check if a file exists
let exists = file_exists("data.txt")
log(exists)     // true

// Read all lines as an array
let lines = read_lines("data.txt")
for line in lines {
    log(line)
}

// Delete a file
delete_file("data.txt")
log(file_exists("data.txt"))    // false
```

### Practical Example: Simple Note-Taking App

```ai
function save_note(filename, note) {
    append_file(filename, note + "\n")
    log(f"Saved note to {filename}")
}

function show_notes(filename) {
    if !file_exists(filename) {
        log("No notes yet!")
        return
    }
    let lines = read_lines(filename)
    log(f"--- Notes in {filename} ---")
    for line in lines {
        log(line)
    }
}

save_note("my_notes.txt", "Learn Aeonmi today")
save_note("my_notes.txt", "Practice quantum circuits")
show_notes("my_notes.txt")
```

---

## Section 10 — Introduction to `.qube`

QUBE (Quantum Universal Block Engine) is Aeonmi's quantum circuit description language. While `.ai` files are for general-purpose programming, `.qube` files describe quantum circuits.

### Core Concepts You Need to Know

**Qubit:** The quantum equivalent of a classical bit. A classical bit is either 0 or 1. A qubit can be in a *superposition* — both 0 and 1 simultaneously until measured.

**Gate:** An operation applied to qubits, analogous to logic gates (AND, OR, NOT) in classical computing. Quantum gates are *reversible* and *unitary*.

**Measurement:** Collapsing a qubit's superposition to a definite 0 or 1. Measurement is probabilistic — you get different results on different runs.

**Circuit:** A sequence of gates applied to a register of qubits.

### A Minimal `.qube` File

Create `first_circuit.qube`:

```qube
circuit first_example {
    meta {
        name: "My First Circuit"
        qubits: 1
        author: "Your Name"
    }

    execute {
        // Declare 1 qubit
        qubit q0;

        // Apply Hadamard gate — puts q0 into superposition
        H q0;

        // Measure q0 into classical bit c0
        measure q0 -> c0;
    }

    expected {
        shots: 1000
        // After 1000 runs, should get ~500 zeros and ~500 ones
        distribution: "uniform"
    }
}
```

Run it:
```bash
Aeonmi qube first_circuit.qube
```

### The Structure of a `.qube` File

Every `.qube` file has this structure:

```qube
circuit <name> {
    meta {
        // metadata about the circuit
    }
    
    execute {
        // the actual quantum operations
    }
    
    expected {
        // expected outcomes for verification
    }
}
```

---

## Section 11 — Quantum Gates Explained

This section explains each gate as if you've never heard of quantum computing.

### The Hadamard Gate — `H`

The Hadamard gate creates **superposition**. When you apply H to a qubit that starts in state |0⟩, it becomes equally likely to be measured as 0 or 1.

```
|0⟩ ──H──  → |+⟩  (50% chance of 0, 50% chance of 1)
|1⟩ ──H──  → |-⟩  (50% chance of 0, 50% chance of 1, with phase difference)
```

```qube
execute {
    qubit q0;
    H q0;           // now q0 is in superposition
    measure q0 -> c0;
}
```

### The X Gate (Quantum NOT) — `X`

Flips a qubit: |0⟩ becomes |1⟩, and |1⟩ becomes |0⟩. It's the quantum equivalent of a NOT gate.

```qube
execute {
    qubit q0;       // starts as |0⟩
    X q0;           // flip: now |1⟩
    measure q0 -> c0;  // will always measure 1
}
```

### The Z Gate — `Z`

Applies a phase flip. |0⟩ stays |0⟩, but |1⟩ becomes -|1⟩. This matters when the qubit is in superposition.

### The Y Gate — `Y`

Combines X and Z rotations. |0⟩ → i|1⟩, |1⟩ → -i|0⟩.

### The CNOT Gate (Controlled NOT) — `CNOT`

Requires **two** qubits. The first qubit is the *control*, the second is the *target*.
- If control = |0⟩: target is unchanged
- If control = |1⟩: target is flipped (X is applied)

```qube
execute {
    qubit control;
    qubit target;
    
    H control;          // put control in superposition
    CNOT control target; // entangle control and target
    
    measure control -> c0;
    measure target -> c1;
}
```

This creates a **Bell state** (maximally entangled pair). When you measure, both qubits are always correlated: either both 0 or both 1.

### Phase Gates — `S` and `T`

These apply fractional phase rotations:
- `S` gate: applies a π/2 (90°) phase rotation
- `T` gate: applies a π/4 (45°) phase rotation

```qube
execute {
    qubit q0;
    H q0;   // superposition
    S q0;   // apply phase
    H q0;   // back to computational basis
    measure q0 -> c0;
}
```

### Rotation Gates — `Rx`, `Ry`, `Rz`

Rotate the qubit state by an angle (in radians) around the X, Y, or Z axis of the Bloch sphere:

```qube
execute {
    qubit q0;
    Rx(1.5707963) q0;   // rotate 90° around X axis (= π/2 radians)
    measure q0 -> c0;
}
```

### The SWAP Gate — `SWAP`

Exchanges the states of two qubits:

```qube
execute {
    qubit q0;
    qubit q1;
    
    X q0;           // q0 = |1⟩, q1 = |0⟩
    SWAP q0 q1;     // q0 = |0⟩, q1 = |1⟩
    
    measure q0 -> c0;   // always 0
    measure q1 -> c1;   // always 1
}
```

### The Toffoli Gate (CCNOT) — `Toffoli`

A 3-qubit gate: applies X to the target only if BOTH control qubits are |1⟩. This is the quantum equivalent of a classical AND gate and is *universal* for reversible classical computing.

```qube
execute {
    qubit ctrl1;
    qubit ctrl2;
    qubit target;
    
    X ctrl1;
    X ctrl2;
    Toffoli ctrl1 ctrl2 target;  // target flipped because both controls are 1
    measure target -> c2;         // will always measure 1
}
```

### Barrier and Reset

```qube
execute {
    qubit q0;
    H q0;
    barrier q0;     // marks a separation (for visualization and optimization)
    reset q0;       // resets qubit back to |0⟩
    measure q0 -> c0;  // always 0 after reset
}
```

### Classical Conditionals (`if` in QUBE)

Apply a gate only if a classical bit has a certain value:

```qube
execute {
    qubit q0;
    qubit q1;
    
    H q0;
    measure q0 -> c0;
    
    if c0 == 1 {
        X q1;   // flip q1 only if q0 measured as 1
    }
    measure q1 -> c1;
}
```

---

## Section 12 — Your First Complete Quantum Circuit

Let's build a complete Bell state circuit step by step:

**Goal:** Create two entangled qubits so that measuring one instantly determines the other.

```qube
circuit bell_state_tutorial {
    meta {
        name: "Bell State — Entangled Pair"
        qubits: 2
        classical_bits: 2
        description: "Demonstrates quantum entanglement"
        author: "Beginner"
    }

    execute {
        // Step 1: Start with two qubits, both in |0⟩
        qubit q0;
        qubit q1;

        // Step 2: Put q0 into superposition with Hadamard
        H q0;
        // Now q0 is in (|0⟩ + |1⟩)/√2
        // q1 is still |0⟩
        
        // Step 3: Entangle q0 and q1 with CNOT
        CNOT q0 q1;
        // Now the pair is in (|00⟩ + |11⟩)/√2
        // This is a Bell state! The qubits are entangled.
        
        // Step 4: Measure both qubits
        measure q0 -> c0;
        measure q1 -> c1;
        // Result: always either (c0=0, c1=0) or (c0=1, c1=1)
        // Never (0,1) or (1,0) — that's entanglement!
    }

    expected {
        shots: 1000
        // After 1000 runs, about half are "00" and half are "11"
        outcomes: ["00", "11"]
        correlation: "perfect"
    }
}
```

Run it:
```bash
Aeonmi qube bell_state_tutorial.qube --draw
```

The `--draw` flag shows an ASCII circuit diagram:
```
q0: ─[H]─●────[M]
          │
q1: ──────X────[M]
```

---

## Section 13 — Built-in Quantum Algorithms

Aeonmi's QUBE engine has several famous quantum algorithms built in. You can invoke them in your circuit's `execute` block:

### Grover's Search Algorithm

Finds a marked item in an unsorted database quadratically faster than any classical algorithm.

```qube
circuit grover_example {
    meta { qubits: 3 }
    
    execute {
        qubit q0;
        qubit q1;
        qubit q2;
        
        // Run Grover's search
        // The oracle marks the state |101⟩ (state 5 in decimal)
        grover(q0, q1, q2);
        
        measure q0 -> c0;
        measure q1 -> c1;
        measure q2 -> c2;
    }
    
    expected {
        // With high probability, should measure the marked state
        distribution: "peaked"
    }
}
```

### Quantum Fourier Transform (QFT)

The quantum analog of the discrete Fourier transform. Core of many quantum algorithms (Shor's, phase estimation).

```qube
circuit qft_example {
    meta { qubits: 3 }
    
    execute {
        qubit q0;
        qubit q1;
        qubit q2;
        
        X q0;   // initialize to |1⟩
        
        // Apply QFT
        qft(q0, q1, q2);
        
        measure q0 -> c0;
        measure q1 -> c1;
        measure q2 -> c2;
    }
}
```

### Quantum Teleportation

Transmit a qubit's state to another qubit using entanglement + classical communication.

```qube
circuit teleport_example {
    meta {
        qubits: 3
        description: "Quantum teleportation of q0's state to q2"
    }
    
    execute {
        qubit message;    // the qubit to teleport
        qubit alice;      // Alice's end of the Bell pair
        qubit bob;        // Bob's end of the Bell pair
        
        // Prepare the message qubit in some state
        H message;
        
        // Teleport!
        teleport(message, alice, bob);
        
        measure message -> c0;
        measure alice -> c1;
        measure bob -> c2;
        // bob (c2) now holds the original state of 'message'
    }
}
```

### Bell State Shorthand

```qube
circuit quick_bell {
    meta { qubits: 2 }
    
    execute {
        qubit q0;
        qubit q1;
        
        bell(q0, q1);   // one-liner for H + CNOT
        
        measure q0 -> c0;
        measure q1 -> c1;
    }
}
```

---

## Section 14 — Mixing `.ai` and `.qube`

You can use QUBE quantum results from within `.ai` programs using the `qube` command:

```ai
// hybrid_example.ai
// This script runs a quantum circuit and uses the result

log("Starting quantum-classical hybrid computation...")

// Use the QUBE engine to run a Bell state circuit
// The result is stored as a variable
let bell_result = qube_run("examples/bell.qube")

log(f"Quantum measurement result: {bell_result}")

// Do classical post-processing
if bell_result == "00" {
    log("Alice and Bob both measured zero")
} else {
    log("Alice and Bob both measured one")
}
```

### Quantum State Literals in `.ai`

You can write quantum state literals directly in `.ai` code:

```ai
// Quantum state literals
let zero_state = |0⟩
let one_state = |1⟩
let plus_state = |+⟩
let minus_state = |-⟩

log(zero_state)   // |0⟩
log(plus_state)   // |+⟩

// Tensor products
let two_qubit = |0⟩ ⊗ |1⟩
log(two_qubit)    // |0⟩ ⊗ |1⟩
```

---

## Section 15 — The AI Canvas Editor

The AI Canvas is Aeonmi's interactive coding environment. Launch it with:

```bash
Aeonmi canvas
```

You'll see a TUI (terminal user interface) where you can:

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New file |
| `Ctrl+O` | Open file |
| `Ctrl+S` | Save |
| `Ctrl+Z` | Undo (up to 256 steps) |
| `Ctrl+Y` | Redo |
| `Ctrl+G` | AI Generate — type a description, get code |
| `Ctrl+F` | AI Fix — let Mother AI fix your bugs |
| `Ctrl+E` | AI Explain — explain the selected code |
| `Ctrl+R` | AI Refactor — clean up the code |
| `Ctrl+Q` | Toggle Quantum Mode (QUBE snippets available) |
| `Ctrl+X` | Cross-compile (target another platform) |
| `Ctrl+V` | Vault operations |
| `Esc` | Exit |

### AI Generate Example

In the canvas, press `Ctrl+G` and type:
```
a function that takes a list of numbers and returns the sorted unique values
```

The AI will generate:
```ai
function sorted_unique(numbers) {
    let seen = {}
    let result = []
    for n in numbers {
        if !seen[n] {
            push(result, n)
            seen[n] = true
        }
    }
    // Simple insertion sort
    let i = 0
    while i < len(result) {
        let j = i + 1
        while j < len(result) {
            if result[i] > result[j] {
                let tmp = result[i]
                result[i] = result[j]
                result[j] = tmp
            }
            j = j + 1
        }
        i = i + 1
    }
    return result
}
```

---

## Section 16 — Vault, Web3, and Beyond

### The Vault

Aeonmi has a built-in cryptographic vault for storing and protecting sensitive data:

```bash
# Create a new key pair
Aeonmi vault new-key mykey

# Sign a message
Aeonmi vault sign mykey "important message"

# Verify a signature
Aeonmi vault verify mykey "important message" <signature>

# Encrypt data
Aeonmi vault encrypt "secret data" --key mykey

# Decrypt data
Aeonmi vault decrypt <ciphertext> --key mykey
```

### Web3 Wallet

```bash
# Create a new wallet
Aeonmi wallet new

# Check balance
Aeonmi wallet balance

# Transfer tokens
Aeonmi wallet transfer <recipient> <amount>
```

### ERC-20 Token

```bash
# Create a token
Aeonmi token create --name "MyToken" --symbol "MTK" --supply 1000000

# Mint more
Aeonmi token mint MTK 500

# Transfer tokens
Aeonmi token transfer MTK <recipient> 100

# Check balance
Aeonmi token balance MTK <address>
```

### DAO Governance

```bash
# Create a DAO
Aeonmi dao create --name "My DAO"

# Create a proposal
Aeonmi dao propose <dao-id> "Increase funding"

# Vote
Aeonmi dao vote <proposal-id> yes

# Execute passed proposal
Aeonmi dao execute <proposal-id>
```

### Serving a Web Application

Create a simple web handler in `.ai`:
```ai
// server.ai
function handle_request(method, path, body) {
    if path == "/" {
        return http_response(200, "text/html", "<h1>Hello from Aeonmi!</h1>")
    }
    if path == "/api/time" {
        return http_json(200, { time: "quantum-now", status: "ok" })
    }
    return http_response(404, "text/plain", "Not Found")
}
```

Serve it:
```bash
Aeonmi serve server.ai --port 8080
```

---

## Section 17 — Complete Reference Card

### `.ai` Quick Reference

```ai
// Variables
let x = 42
let name = "Aeonmi"
let flag = true

// Functions
function add(a, b) { return a + b }
let mul = function(a, b) => a * b

// Conditionals
if cond { ... } else { ... }

// Loops
while cond { ... }
for item in array { ... }
for (a, b) in pairs { ... }

// Arrays
let arr = [1, 2, 3]
push(arr, 4)
pop(arr)
len(arr)
map(arr, function(x) => x * 2)
filter(arr, function(x) => x > 1)

// Objects
let obj = { key: "value" }
obj.key
obj["key"]

// Strings
"hello " + "world"
f"Hello {name}!"
len("hello")

// F-strings
f"x = {x}, y = {x*2}"

// Quantum states
|0⟩  |1⟩  |+⟩  |-⟩
|0⟩ ⊗ |1⟩

// Genesis arrays
⧉[1, 2, 3]

// Pattern match
match val { "a" => ..., _ => ... }

// File I/O
write_file(path, content)
read_file(path)
append_file(path, content)
file_exists(path)
read_lines(path)
delete_file(path)

// HTTP (in serve context)
http_response(status, mime, body)
http_json(status, object)
http_get(url)
http_post(url, body)
```

### `.qube` Quick Reference

```qube
circuit name {
    meta {
        name: "..."
        qubits: N
        author: "..."
    }
    
    execute {
        qubit q0;
        
        // Single-qubit gates
        H q0;         // Hadamard
        X q0;         // NOT / bit-flip
        Y q0;         // Y gate
        Z q0;         // phase-flip
        S q0;         // π/2 phase
        T q0;         // π/4 phase
        Rx(angle) q0; // rotate around X
        Ry(angle) q0; // rotate around Y
        Rz(angle) q0; // rotate around Z
        
        // Two-qubit gates
        CNOT q0 q1;         // controlled-NOT
        SWAP q0 q1;         // swap
        
        // Three-qubit gate
        Toffoli q0 q1 q2;   // controlled-controlled-NOT
        
        // Utilities
        barrier q0;         // barrier marker
        reset q0;           // reset to |0⟩
        
        // Measurement
        measure q0 -> c0;   // measure qubit into classical bit
        
        // Classical conditional
        if c0 == 1 { X q1; }
        
        // Built-in algorithms
        grover(q0, q1, q2);
        qft(q0, q1, q2);
        teleport(msg, alice, bob);
        bell(q0, q1);
    }
    
    expected {
        shots: 1000
        outcomes: ["00", "11"]
    }
}
```

### CLI Reference

```bash
Aeonmi run <file.ai>           # run a .ai file
Aeonmi build <file.ai>         # compile to native
Aeonmi qube <file.qube>        # run a .qube circuit
Aeonmi qube <file.qube> --draw # show circuit diagram
Aeonmi canvas                  # AI Canvas editor
Aeonmi serve <file.ai>         # web server
Aeonmi verify <contract.ai>    # smart contract verifier

Aeonmi wallet new/balance/transfer
Aeonmi token create/mint/transfer/balance
Aeonmi dao create/propose/vote/execute

Aeonmi vault new-key/sign/verify/encrypt/decrypt

Aeonmi --pretty-errors <file>  # verbose error messages
Aeonmi --config <path>         # custom config file
Aeonmi --help                  # show all commands
```

---

## Section 18 — Troubleshooting FAQ

**Q: "command not found: Aeonmi"**  
A: Make sure you built the project and added `target/debug` to your PATH. Alternatively, always use `./target/debug/Aeonmi` from the project directory.

**Q: "Parsing error: Expected variable name" on valid-looking code**  
A: Check for typos like `let = value` (missing variable name) or `function ()` (missing function name).

**Q: My program runs but prints nothing**  
A: Make sure you're using `log(...)` not `print(...)`. The `log` function is the standard output function in Aeonmi.

**Q: "Maximum parse depth exceeded"**  
A: You have deeply nested expressions or an accidentally unclosed bracket. Look for mismatched `{`, `(`, or `[`.

**Q: Stack overflow in my recursive function**  
A: Your function is recursing too deeply. Add a base case and make sure recursion terminates.

**Q: QUBE circuit gives "unexpected measurement distribution"**  
A: Quantum measurements are probabilistic. Run with more shots (`shots: 10000`) for more reliable statistics.

**Q: "feature not available" on some commands**  
A: Some commands require specific feature flags. Make sure you built with `--features "quantum,mother-ai"`.

**Q: How do I configure the AI model for `Ctrl+G` in the canvas?**  
A: Edit `~/.aeonmi/qpoly.toml` and set your API key:
```toml
[ai]
provider = "openai"
api_key = "sk-..."
model = "gpt-4"
```

**Q: I see `@@DIAG:` JSON in my output**  
A: That's the machine-readable diagnostic format. It's normal. The human-readable error is on the next line.

**Q: Can I use Aeonmi on Windows?**  
A: Yes! Use WSL (Windows Subsystem for Linux) for the best experience. Native Windows support (Cargo build) also works but is less tested.

---

## Congratulations!

You've completed the Aeonmi beginner tutorial. You now know how to:

✅ Write `.ai` programs with variables, functions, loops, and closures  
✅ Use pattern matching and f-strings  
✅ Work with genesis glyphs and genesis arrays  
✅ Read and write files  
✅ Write `.qube` quantum circuits  
✅ Apply quantum gates and understand what they do  
✅ Use built-in quantum algorithms (Grover, QFT, Teleportation, Bell)  
✅ Mix quantum and classical computation  
✅ Use the AI Canvas editor  
✅ Work with the Vault, Web3, and HTTP server  

### What's Next?

| Next Step | Resource |
|-----------|----------|
| Deep dive into the language | `docs/Aeonmi_Language_Guide.md` |
| Full QUBE specification | `docs/QUBE_SPEC.md` |
| Genesis glyph algebra | `docs/glyph_algebra.md` |
| Web3 development | `docs/WEB3_GUIDE.md` |
| Architecture internals | `docs/architecture.md` |
| Contribute to Aeonmi | `CONTRIBUTING.md` |
| Enterprise audit report | `docs/ENTERPRISE_AUDIT.md` |

---

*Aeonmi — Where AI Meets Quantum Reality*  
*Tutorial version 1.0.0 — 2026-03-16*
