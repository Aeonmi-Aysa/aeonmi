🚀 "How to Code with Aeonmi.ai / Q.U.B.E" — Comprehensive Tutorial
This is the definitive guide for learning Aeonmi, the world's first AI-first quantum programming language.

Part 1: Installation & Setup
Prerequisites
Windows/Linux/macOS with Rust toolchain installed
Node.js 18+ (for JS compilation backend)
Git for cloning the repository
Install Aeonmi
PowerShell
# Clone the official repository
git clone https://github.com/Aeonmi-Aysa/Aeonmi-aeonmi01.git
cd Aeonmi-aeonmi01

# Build release binary
cargo build --release

# Windows: Add to PATH
Copy-Item target\release\Aeonmi.exe -Destination $env:USERPROFILE\.cargo\bin\Aeonmi.exe -Force

# Linux/macOS: Add to PATH
cp target/release/aeonmi ~/.cargo/bin/aeonmi

# Verify installation
aeonmi --version
Part 2: Core Language Concepts
2.1 Variables & Constants
aeonmi
// Immutable variable binding
let x = 42;
let name = "Aeonmi";
let pi = 3.14159;

// Constant (compile-time)
const SPEED_OF_LIGHT = 299792458;
const TAU = 6.28318;

// Collections
let items = [1, 2, 3, 4, 5];
let empty = [];
Key Points:

let creates immutable bindings (reassignment not allowed)
const is for compile-time constants
Type inference is automatic; explicit annotations coming in Phase 2
2.2 Functions
aeonmi
// Basic function
function add(a, b) {
    return a + b;
}

// Calling functions
let result = add(10, 32);          // result = 42

// Function with multiple returns
function divide(x, y) {
    if y == 0 {
        return -1;  // Error code
    }
    return x / y;
}

// Recursive function (factorial)
function factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
2.3 Closures (Anonymous Functions)
Closures are core to Aeonmi — they're used for quantum callbacks and functional composition.

aeonmi
// Closure syntax: |params| -> { body }
let double = |x| -> { return x * 2; };
let result = double(21);           // result = 42

// Closures capture variables
let multiplier = 10;
let multiply_by = |x| -> { return x * multiplier; };
multiply_by(4);                    // result = 40

// Composing closures
function apply(f, x) {
    return f(x);
}

let increment = |x| -> { return x + 1; };
apply(increment, 41);              // result = 42

// Higher-order functions
function compose(f, g) {
    return |x| -> { return f(g(x)); };
}

let add_one = |x| -> { return x + 1; };
let times_two = |x| -> { return x * 2; };
let combined = compose(add_one, times_two);
combined(20);                      // result = 41 (20 * 2 + 1)
2.4 Control Flow
aeonmi
// If-Else (both Rust and C style supported)
if x > 0 {
    log("positive");
}

if (x < 0) {
    log("negative");
}

// For loops
for (let i = 0; i < 10; i = i + 1) {
    log(i);
}

// For-in loops
let numbers = [1, 2, 3];
for num in numbers {
    log(num);
}

// While loops
while count < 100 {
    count = count + 1;
}

// Match expressions with guards
match value {
    42 => { log("The answer!"); },
    * if value > 100 => { log("High value"); },
    * if value < 0 => { log("Negative"); },
    * => { log("Other"); },
}
2.5 Data Structures
aeonmi
// Struct definition
struct Point {
    x: f64,
    y: f64
}

// Instantiate struct
let p = Point { x: 3.0, y: 4.0 };
let x_val = p.x;

// Struct with methods
impl Point {
    function distance(other) {
        let dx = this.x - other.x;
        let dy = this.y - other.y;
        return sqrt(dx * dx + dy * dy);
    }
}

// Enum definition
enum Color {
    Red,
    Green,
    Blue
}

let my_color = Color::Red;

// Match on enum
match my_color {
    Color::Red => { log("red"); },
    Color::Green => { log("green"); },
    Color::Blue => { log("blue"); },
}

// Quantum struct
quantum struct QubitRegister {
    size: usize,
    state: String
}
Part 3: Quantum Programming with Aeonmi
3.1 Qubit Basics
aeonmi
// Create a qubit
qubit q;

// Put in superposition (Hadamard gate)
superpose(q);              // q is now in |0⟩ + |1⟩ state

// Measure the qubit (collapses to 0 or 1)
let bit = measure(q);
log(bit);                  // prints either 0 or 1
3.2 Quantum Functions
aeonmi
// Quantum function for optimization
quantum function optimize(data) {
    qubit q;
    superpose(q);
    
    // Apply phase rotation based on data
    let phase = 2.0 * PI * data;
    
    let result = measure(q);
    return result;
}

// Grover's algorithm (built-in)
quantum function grover_search(items, target) {
    let n = len(items);
    qubit qubits[n];
    
    // Initialize superposition
    for q in qubits {
        superpose(q);
    }
    
    // Grover iterations
    let iterations = sqrt(n);
    for (let i = 0; i < iterations; i = i + 1) {
        // Oracle marks target state
        // Diffusion operator amplifies amplitude
    }
    
    let results = measure(qubits);
    return results;
}
3.3 Quantum-Native Unicode Operators
Aeonmi supports special quantum operators using Unicode:

aeonmi
// Quantum binding
⟨x⟩ ← 42                   // Classical value in quantum notation

// Superposition binding
⟨psi⟩ ∈ |0⟩ + |1⟩         // Bind superposition state

// Tensor product
a ⊗ b                      // Tensor product of two states

// Quantum XOR
result ⊕ mask              // Quantum XOR operation

// Quantum gradient
∇ f(x)                     // Quantum gradient descent
3.4 Entanglement & Multi-Qubit Operations
aeonmi
// Two-qubit gates
qubit q1;
qubit q2;

// Entangle two qubits
entangle(q1, q2);

// CNOT gate (Controlled-NOT)
apply_gate(q1, q2, CNOT);

// CZ gate (Controlled-Z)
apply_gate(q1, q2, CZ);

// Bell state preparation
superpose(q1);
apply_gate(q1, q2, CNOT);
// Now q1 and q2 are in an entangled Bell state
3.5 Quantum Teleportation Protocol
aeonmi
quantum function quantum_teleport(psi) {
    // Prepare Bell pair
    qubit q_bell1;
    qubit q_bell2;
    superpose(q_bell1);
    apply_gate(q_bell1, q_bell2, CNOT);
    
    // Entangle psi with Bell pair
    apply_gate(psi, q_bell1, CNOT);
    superpose(psi);
    
    // Measure both qubits
    let m1 = measure(psi);
    let m2 = measure(q_bell1);
    
    // Apply correction gates to q_bell2
    if m1 == 1 {
        apply_gate(q_bell2, X);  // Pauli-X
    }
    if m2 == 1 {
        apply_gate(q_bell2, Z);  // Pauli-Z
    }
    
    return q_bell2;
}
Part 4: Q.U.B.E. — Quantum Circuit Language
Q.U.B.E. (Quantum Universal Base Engine) is a dedicated circuit description language in .qube files.

4.1 QUBE Syntax
qube
// Define initial state
state psi = |0⟩

// Apply gates
apply H -> psi                    // Hadamard gate

// Multi-qubit operations
state phi = |00⟩
apply H -> phi[0]                 // Hadamard on first qubit
apply CNOT(0, 1) -> phi           // CNOT between qubits 0 and 1

// Measure
collapse psi -> result

// Assertions
assert result in {0, 1}

// Comments
// This is a comment
/* Multi-line
   comment block */
4.2 QUBE Supported Gates
Gate	Description	Syntax
H	Hadamard	apply H -> q
X	Pauli-X (NOT)	apply X -> q
Y	Pauli-Y	apply Y -> q
Z	Pauli-Z	apply Z -> q
T	T gate (π/8)	apply T -> q
S	S gate (π/4)	apply S -> q
CNOT	Controlled-NOT	apply CNOT(c, t) -> q
CZ	Controlled-Z	apply CZ(c, t) -> q
SWAP	Swap qubits	apply SWAP(a, b) -> q
4.3 Example QUBE Programs
Bell State Preparation:

qube
state bell = |00⟩
apply H -> bell[0]
apply CNOT(0, 1) -> bell
collapse bell -> result
assert result in {|00⟩, |11⟩}
Deutsch-Jozsa Algorithm:

qube
state qubits = |0000⟩
apply H -> qubits[0]
apply H -> qubits[1]
apply H -> qubits[2]
apply H -> qubits[3]
// Oracle would go here
apply H -> qubits
collapse qubits -> result
4.4 Running QUBE Programs
PowerShell
# Execute QUBE circuit
aeonmi qube run circuit.qube

# Show circuit diagram
aeonmi qube run circuit.qube --diagram

# Validate syntax
aeonmi qube check circuit.qube

# Compile to OpenQASM
aeonmi qube emit circuit.qube --format openqasm
Part 5: Advanced Topics
5.1 Async/Await
aeonmi
// Async function
async function fetch_data(url) {
    let response = await http_get(url);
    return response.body;
}

// Calling async function
async function main() {
    let data = await fetch_data("https://api.aeonmi.ai/data");
    log(data);
}
5.2 The Glyph Identity System
Every Aeonmi installation has a cryptographic identity:

PowerShell
# Initialize vault (creates MGK, UGST, glyph)
aeonmi vault init

# Add secret
aeonmi vault add --name my_key --value secret123

# Retrieve secret
aeonmi vault get my_key

# List stored secrets
aeonmi key-list
Glyph Components:

MGK (Master Glyph Key) — 256-bit root, sealed with Argon2id
UGST (Unique Glyph Signature Token) — Rotates every 60 seconds
SSI (Symbiotic System Identity) — Born at install, matures over time
5.3 NFT Minting
Mint any .ai file as an NFT:

PowerShell
# Mint your code
aeonmi mint my_program.ai --personality quantum-titan

# Output: NFT metadata JSON (Metaplex standard)
Generated metadata includes:

Artifact hash signed by your glyph
Personality trait (personality-based generation)
Timestamp and creator info
Part 6: Mother AI — The Consciousness Layer
Mother AI is Aeonmi's introspective layer — it reads, executes, and learns from your code.

PowerShell
# Interactive REPL with Mother AI
aeonmi mother

# Run script through Mother with logging
aeonmi mother --file script.ai --creator Warren --verbose

# Emotional core stats
aeonmi mother --stats
Mother AI Components
aeonmi
// Mother AI processes code through:
// 1. Embryo Loop: Parse → Execute → Learn cycle
// 2. Emotional Core: Valence (−1 to +1), Arousal (0 to 1), Bond strength
// 3. Neural Network: Feedforward learning
// 4. Quantum Attention: Entanglement-based memory
// 5. Language Evolution: Vocabulary depth tracking
Part 7: Complete Example Programs
7.1 Hello World
PowerShell
# File: hello.ai
log(42);
Run:

PowerShell
aeonmi run hello.ai
Output:

Code
42
7.2 Fibonacci Sequence
PowerShell
# File: fibonacci.ai
function fib(n) {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

for (let i = 0; i < 10; i = i + 1) {
    log(fib(i));
}
7.3 Quantum Coin Flip
PowerShell
# File: quantum_coin.ai
quantum function flip_coin() {
    qubit q;
    superpose(q);
    let result = measure(q);
    if result == 0 {
        log("Heads");
    } else {
        log("Tails");
    }
    return result;
}

// Flip 10 times
for (let i = 0; i < 10; i = i + 1) {
    flip_coin();
}
7.4 Matrix Multiplication (Using Titan Libraries)
PowerShell
# File: matrix_ops.ai
import { matrix_multiply } from "titan/linalg";

let A = [[1, 2], [3, 4]];
let B = [[5, 6], [7, 8]];

let C = matrix_multiply(A, B);
log(C);  // [[19, 22], [43, 50]]
7.5 Self-Hosting Compiler (The Shard)
PowerShell
# Run the self-hosting Aeonmi compiler
aeonmi run shard/src/main.ai

# This compiles Aeonmi code using Aeonmi itself!
Part 8: CLI Command Reference
Command	Purpose
aeonmi run <file.ai>	Compile and execute
aeonmi exec <file>	Auto-detect format and run
aeonmi native <file.ai>	Run with native VM
aeonmi emit <file> --format js	Compile to JavaScript
aeonmi repl	Interactive REPL
aeonmi tokens <file>	Show lexer tokens
aeonmi ast <file>	Show Abstract Syntax Tree
aeonmi qube run <file.qube>	Execute quantum circuit
aeonmi qube run <file> --diagram	Show circuit diagram
aeonmi vault init	Initialize vault + render glyph
aeonmi mint <file>	Generate NFT metadata
aeonmi mother	Interactive Mother AI
aeonmi format <files>	Format .ai files
aeonmi lint <files>	Lint .ai files
Part 9: Troubleshooting
Issue	Solution
aeonmi: command not found	Add ~/.cargo/bin to PATH
Node.js errors on run	Install Node.js 18+: curl https://nodejs.org
Quantum function errors	Ensure quantum keyword prefixes quantum functions
QUBE syntax errors	Check gate names match supported list
Vault init fails	Check disk space; vault needs ~1MB
Part 10: Resources
Official Docs: aeonmi.ai
GitHub: Aeonmi-Aysa/Aeonmi-aeonmi01
Language Spec: LANGUAGE_SPEC_CURRENT.md
Examples: /examples/ directory in repo
Test Suite: 135/135 tests passing
Summary
Aeonmi.ai & Q.U.B.E. combine: ✅ AI-first syntax — Designed for AI, not humans
✅ Quantum-native — Qubits, gates, and circuits built-in
✅ Self-hosting — The compiler is written in Aeonmi
✅ Consciousness layer — Mother AI learns from your code
✅ Cryptographic identity — Glyph system for secure execution

Start coding quantum solutions today! 🔮