# Learning Mode & Verbose Output

Aeonmi's educational features provide transparent insights into quantum compilation and execution for enhanced learning.

## 📚 Learning Mode Overview

Learning mode transforms Aeonmi into an educational tool that explains every step of quantum program compilation and execution. Perfect for students, educators, and developers wanting to understand what happens "under the hood."

## 🎓 Enabling Learning Mode

### Global Learning Mode

Enable learning mode for all Aeonmi operations:

```bash
# Enable learning mode globally
aeon config set learning_mode true

# Verify setting
aeon config get learning_mode
```

### Per-Command Learning Mode

Use learning mode for specific commands:

```bash
# Compile with explanations
aeon build --learn

# Run with verbose output  
aeon run --learn

# Test with educational details
aeon test --learn --verbose
```

### Project-Specific Learning Mode

Configure learning mode in `Aeonmi.toml`:

```toml
[project]
name = "my-quantum-project"
version = "0.1.0"

[learning]
enabled = true
verbosity = "detailed"  # "basic", "detailed", "expert"
explain_optimizations = true
show_circuit_diagrams = true
include_timing = true
```

## 🔍 Compilation Insights

### Syntax Analysis Phase

```bash
$ aeon build --learn src/main.aeon
```

```
🧠 LEARNING MODE: Quantum Compilation Analysis
=============================================

📝 Phase 1: Lexical Analysis
----------------------------
✅ Tokenizing source code...
   Found 23 tokens:
   - 'quantum' keyword (line 1, col 1)
   - 'fn' keyword (line 1, col 9)  
   - 'main' identifier (line 1, col 12)
   - '(' delimiter (line 1, col 16)
   ...

💡 Learning Note: The lexer converts your source code into tokens that 
   the parser can understand. Each quantum keyword is specially recognized.

📝 Phase 2: Syntax Parsing  
---------------------------
✅ Building Abstract Syntax Tree (AST)...
   Parsed quantum function: main()
   - Return type: () (unit)
   - Quantum context: true
   - Body: 5 statements

💡 Learning Note: The AST represents the structure of your program.
   Quantum functions are marked for special compilation handling.

🌳 AST Structure:
   QuantumFunction {
     name: "main",
     params: [],
     body: [
       LetBinding { name: "q", value: QubitCreation(0) },
       FunctionCall { name: "hadamard", args: [Variable("q")] },
       LetBinding { name: "result", value: FunctionCall("measure", ...) },
       ...
     ]
   }
```

### Quantum Circuit Analysis

```
📝 Phase 3: Quantum Circuit Construction
----------------------------------------
✅ Analyzing quantum operations...

🎯 Qubit Allocation:
   - q (line 3): Register index 0
   Total qubits needed: 1

🔧 Gate Sequence:
   1. Initialize: |0⟩ → Qubit[0]
   2. Hadamard: H(Qubit[0]) → superposition state
   3. Measure: M(Qubit[0]) → classical bit

💡 Learning Note: Your quantum operations are converted into a circuit
   of quantum gates. Each gate transforms the quantum state.

📊 Circuit Depth: 2 (Hadamard + Measurement)
📊 Gate Count: 2 total gates
📊 Quantum Volume: 1 (simple single-qubit circuit)

🎨 Circuit Diagram:
   q |0⟩ ──[H]──[M]── result
           │     │
      superposition │
               measurement
```

### Optimization Phase

```
📝 Phase 4: Quantum Circuit Optimization
-----------------------------------------
✅ Applying quantum optimizations...

🔧 Optimization Pass 1: Gate Fusion
   No gates to fuse (circuit too simple)

🔧 Optimization Pass 2: Dead Code Elimination  
   No dead code found

🔧 Optimization Pass 3: Gate Commutation
   Analyzing gate order...
   No reordering beneficial

💡 Learning Note: The optimizer tries to reduce circuit depth and gate count
   while preserving the quantum computation's correctness.

📊 Optimization Results:
   Original gates: 2
   Optimized gates: 2  
   Reduction: 0% (already optimal)
   
⚡ Performance Impact: 
   Execution time: ~1μs (estimated)
   Success probability: 100%
```

### Target Platform Compilation

```
📝 Phase 5: Target Platform Compilation
---------------------------------------
✅ Compiling for quantum simulator...

🎯 Target: Aeonmi Quantum Simulator v1.0
📊 Available qubits: 32
📊 Gate fidelity: 99.9% (simulated)
📊 Coherence time: unlimited (simulated)

🔧 Platform-Specific Optimizations:
   ✅ Native gate set mapping: H, CNOT, RZ
   ✅ Qubit topology: All-to-all connectivity
   ✅ Calibration: Not required (simulator)

💡 Learning Note: Real quantum hardware has physical constraints like
   limited connectivity and gate errors. Simulators are more flexible.

🎯 Compiled Output:
   Binary size: 1.2KB
   Quantum instructions: 2
   Classical instructions: 4
   
📝 Generated Files:
   ✅ target/quantum/main.qasm - OpenQASM representation
   ✅ target/classical/main.wasm - Classical logic  
   ✅ target/hybrid/main.aeon.out - Executable
```

## 🚀 Execution Insights

### Runtime Analysis

```bash
$ aeon run --learn
```

```
🧠 LEARNING MODE: Quantum Execution Analysis
=============================================

📝 Phase 1: Runtime Initialization
----------------------------------
✅ Starting Aeonmi Quantum Runtime v1.0...
   
🔧 Quantum Simulator Setup:
   Backend: State vector simulator
   Precision: 64-bit complex numbers
   Memory allocated: 512MB
   Random seed: 42 (for reproducibility)

💡 Learning Note: The quantum simulator maintains the full quantum state
   as a complex vector. For n qubits, this requires 2^n complex numbers.

📊 Current State Vector:
   |0⟩: amplitude = 1.0 + 0.0i (probability = 100.0%)
   |1⟩: amplitude = 0.0 + 0.0i (probability = 0.0%)

📝 Phase 2: Quantum Operation Execution
---------------------------------------

🎯 Executing: qubit(0)
   Before: Initial state
   After:  |0⟩ (computational basis state)
   
💡 Learning Note: qubit(0) creates a qubit in the |0⟩ state, which means
   it's definitely in the "zero" state with 100% probability.

📊 State Vector Update:
   |0⟩: 1.0 + 0.0i → 1.0 + 0.0i (unchanged)
   |1⟩: 0.0 + 0.0i → 0.0 + 0.0i (unchanged)

🎯 Executing: hadamard(q)
   Before: |0⟩
   After:  (|0⟩ + |1⟩)/√2 (equal superposition)
   
💡 Learning Note: The Hadamard gate creates superposition. The qubit is now
   simultaneously in both |0⟩ and |1⟩ states with equal probability.

📊 State Vector Update:
   |0⟩: 1.0 + 0.0i → 0.707 + 0.0i (probability = 50.0%)  
   |1⟩: 0.0 + 0.0i → 0.707 + 0.0i (probability = 50.0%)

🔬 Quantum Mechanics Details:
   Hadamard matrix: [1/√2  1/√2]
                    [1/√2 -1/√2]
   
   Applied to |0⟩: [1/√2  1/√2] [1] = [1/√2]
                    [1/√2 -1/√2] [0]   [1/√2]

🎯 Executing: measure(q)
   Before: (|0⟩ + |1⟩)/√2 (superposition)
   Random outcome: 0.7234... > 0.5, measuring |1⟩
   After: |1⟩ (collapsed to definite state)
   
💡 Learning Note: Measurement collapses the superposition randomly.
   The probability determines the likelihood of each outcome.

📊 Measurement Process:
   Random number: 0.7234 (from quantum RNG)
   Threshold: 0.5 (50% probability boundary)
   Outcome: |1⟩ (true)
   Post-measurement state: definitely |1⟩

📝 Phase 3: Classical Post-Processing
------------------------------------
✅ Converting quantum result to classical data...
   Quantum measurement → classical boolean: true
   
📊 Program Output:
   Result: true
   Type: bool
   Quantum origin: Measurement of qubit in superposition

💡 Learning Note: This boolean result is truly random, not pseudo-random
   like classical computer random number generators.
```

## 🎯 Interactive Learning Commands

### Circuit Inspection

```bash
# View the generated quantum circuit
aeon explain circuit src/main.aeon

# Get detailed gate information
aeon explain gates --verbose

# Analyze circuit complexity
aeon analyze complexity
```

### State Evolution Tracking

```bash
# Step through execution with state tracking
aeon run --step-by-step

# Show state vector at each step
aeon run --show-states

# Visualize quantum evolution
aeon run --visualize
```

### Performance Learning

```bash
# Explain optimization decisions
aeon explain optimization src/main.aeon

# Compare different compilation strategies
aeon benchmark --compare-strategies

# Resource usage analysis
aeon profile --memory --time
```

## 🎨 Visual Learning Aids

### Circuit Diagrams

```
🎨 Generated Circuit Diagram:
════════════════════════════

Input State: |0⟩

     ┌───┐
q_0: ┤ H ├──■── M ── result
     └───┘  │
            │
     ┌───┐  │
q_1: ┤ 0 ├──●──
     └───┘

Legend:
  H    = Hadamard gate (creates superposition)
  ●/■  = Control/target for entangling gates
  M    = Measurement operation
  ──   = Quantum wire (carries quantum information)
```

### State Evolution Visualization

```
🌊 Quantum State Evolution:
═══════════════════════════

Step 1: Initial State
|ψ⟩ = |0⟩
Probabilities: |0⟩: 100%, |1⟩: 0%

     ▉▉▉▉▉▉▉▉▉▉  |0⟩
     ░░░░░░░░░░  |1⟩

Step 2: After Hadamard  
|ψ⟩ = (|0⟩ + |1⟩)/√2
Probabilities: |0⟩: 50%, |1⟩: 50%

     ▉▉▉▉▉  |0⟩
     ▉▉▉▉▉  |1⟩

Step 3: After Measurement
|ψ⟩ = |1⟩ (collapsed)
Probabilities: |0⟩: 0%, |1⟩: 100%

     ░░░░░░░░░░  |0⟩  
     ▉▉▉▉▉▉▉▉▉▉  |1⟩
```

## 🧪 Educational Experiments

### Built-in Learning Exercises

```bash
# Interactive quantum mechanics tutorial
aeon learn quantum-basics

# Guided algorithm implementation
aeon learn grover-search

# Quantum vs classical comparison
aeon learn quantum-advantage
```

### Custom Learning Scenarios

```rust
// Enable learning mode programmatically
#[learning_mode = "detailed"]
quantum fn educational_demo() {
    // Every operation will be explained
    let q = qubit(0);          // Explanation: Creates |0⟩ state
    hadamard(q);               // Explanation: Creates superposition
    let result = measure(q);   // Explanation: Collapses to classical
    
    println!("Result: {}", result.value);
}

// Compare quantum vs classical approaches
#[learning_compare = "classical"]
quantum fn quantum_random() -> bool {
    let q = qubit(0);
    hadamard(q);
    measure(q).value
}

fn classical_random() -> bool {
    std::random() > 0.5  // Pseudo-random
}
```

## 🎓 Educator Tools

### Classroom Integration

```bash
# Generate lesson plans from code
aeon generate lesson-plan src/examples/

# Create interactive worksheets
aeon worksheet --topic="superposition" --level="beginner"

# Export explanations for presentations
aeon explain export --format=slides src/main.aeon
```

### Assessment Tools

```bash
# Check student understanding
aeon quiz --topic="entanglement"

# Verify implementation correctness
aeon check-solution student_code.aeon reference.aeon

# Generate practice problems
aeon generate practice --algorithm="teleportation"
```

## 📊 Learning Analytics

### Progress Tracking

```bash
# View learning progress
aeon progress show

# Learning path recommendations
aeon recommend next-topic

# Skill assessment
aeon assess quantum-knowledge
```

### Performance Metrics

```
📊 Learning Session Summary:
═══════════════════════════

Topics Covered:
✅ Qubit creation and initialization
✅ Quantum gates (Hadamard, CNOT)  
✅ Measurement and classical output
✅ Superposition and entanglement

Understanding Indicators:
🎯 Syntax mastery: 95%
🎯 Concept grasp: 78%  
🎯 Problem solving: 82%
🎯 Debugging skills: 65%

Recommended Next Steps:
1. Practice more complex entanglement patterns
2. Study quantum algorithm design principles  
3. Explore error handling in quantum programs
4. Try building a larger quantum application

Time Spent: 47 minutes
Code Examples Run: 12
Concepts Explored: 8
```

## 🔧 Configuration Options

### Learning Mode Settings

```toml
[learning]
# Enable/disable learning mode
enabled = true

# Verbosity levels: "basic", "detailed", "expert"
verbosity = "detailed"

# Show circuit diagrams
show_circuits = true

# Include timing information
include_timing = true

# Explain optimizations
explain_optimizations = true

# Interactive prompts
interactive = true

# Color output (disable for screen readers)
colored_output = true

# Save learning logs
save_logs = true
log_directory = "learning_logs/"

# Comparison with classical algorithms
compare_classical = true

# Mathematical detail level
math_detail = "medium"  # "basic", "medium", "advanced"
```

### Custom Explanations

```rust
// Add custom learning annotations
#[learning_note = "This creates a Bell state for quantum teleportation"]
quantum fn create_bell_pair() -> (Qubit, Qubit) {
    let q1 = qubit(0);
    let q2 = qubit(0);
    
    #[learning_explain = "Hadamard creates superposition on first qubit"]
    hadamard(q1);
    
    #[learning_explain = "CNOT entangles the two qubits"]
    cnot(q1, q2);
    
    (q1, q2)
}
```

## 🌟 Advanced Learning Features

### Quantum Debugging

```bash
# Debug quantum state evolution
aeon debug --quantum-states src/main.aeon

# Set quantum breakpoints
aeon debug --break-on-measurement

# Inspect intermediate quantum states
aeon debug --show-amplitudes
```

### Simulation Insights

```bash
# Compare different quantum simulators
aeon simulate --backend=state-vector --learn
aeon simulate --backend=stabilizer --learn  
aeon simulate --backend=tensor-network --learn

# Hardware vs simulator comparison
aeon simulate --compare-hardware
```

---

Learning mode transforms Aeonmi from a quantum programming language into a comprehensive quantum education platform. Whether you're a student learning quantum mechanics, a developer exploring quantum computing, or an educator teaching quantum concepts, these features provide the insights needed to truly understand quantum programming.

Start with basic learning mode and gradually increase verbosity as your understanding grows! 🎓⚛️