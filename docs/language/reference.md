# Language Reference

Complete reference for the Aeonmi programming language syntax, constructs, and features.

## Table of Contents

1. [Syntax Overview](#syntax-overview)
2. [Keywords](#keywords)
3. [Data Types](#data-types)
4. [Variables](#variables)
5. [Functions](#functions)
6. [Classes & Structs](#classes--structs)
7. [Quantum Operations](#quantum-operations)
8. [Control Flow](#control-flow)
9. [Modules & Imports](#modules--imports)
10. [Error Handling](#error-handling)

## Syntax Overview

Aeonmi uses a Rust-inspired syntax with quantum extensions. The language is statically typed with type inference and supports both classical and quantum operations.

### Basic Program Structure
```rust
// Import declarations
use quantum;
use std::io;

// Global constants
const MAX_QUBITS: u32 = 32;

// Function definitions
fn main() {
    println!("Hello, Aeonmi!");
}

// Quantum function
quantum fn bell_state(q1: Qubit, q2: Qubit) -> (Qubit, Qubit) {
    hadamard(q1);
    cnot(q1, q2);
    return (q1, q2);
}
```

### Comments
```rust
// Single-line comment

/* 
   Multi-line comment
   Can span multiple lines
*/

/// Documentation comment for functions and types
/// These are used to generate API documentation
fn documented_function() {}
```

## Keywords

Aeonmi includes all standard programming keywords plus quantum-specific ones:

### Classical Keywords
```rust
// Control flow
if, else, while, for, loop, break, continue, return, match

// Declarations  
fn, let, mut, const, static, struct, impl, trait, enum

// Types
bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, str

// Modules
mod, use, pub, crate, super, self

// Memory & ownership
move, ref, Box, Vec, Option, Result
```

### Quantum Keywords
```rust
// Quantum declarations
quantum, qubit, qreg, circuit

// Quantum operations
hadamard, pauli_x, pauli_y, pauli_z, cnot, phase, measure

// Quantum types
Qubit, QubitRegister, QuantumCircuit, MeasurementResult
```

## Data Types

### Primitive Types
```rust
// Integers
let small: i8 = 42;
let large: i64 = 1_000_000;
let unsigned: u32 = 123;

// Floating point
let pi: f64 = 3.14159;
let float: f32 = 2.718;

// Boolean
let is_quantum: bool = true;
let is_classical: bool = false;

// Character and strings
let letter: char = 'Q';
let message: &str = "Quantum computing";
let owned: String = String::from("Aeonmi");
```

### Collections
```rust
// Arrays (fixed size)
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let measurements: [bool; 10] = [false; 10];

// Vectors (dynamic)
let mut results: Vec<f64> = Vec::new();
results.push(0.707);

// Tuples
let coordinates: (f64, f64) = (3.0, 4.0);
let measurement: (bool, f64) = (true, 0.85);
```

### Quantum Types
```rust
// Single qubit
let q: Qubit = qubit(0);  // |0⟩ state
let q_one: Qubit = qubit(1);  // |1⟩ state

// Qubit register
let qreg: QubitRegister = qubits(5);  // 5 qubits in |00000⟩

// Quantum circuit
let circuit: QuantumCircuit = QuantumCircuit::new(3);

// Measurement result
let result: MeasurementResult = measure(q);
```

## Variables

### Variable Declaration
```rust
// Immutable by default
let x = 42;
let name = "Alice";

// Mutable variables
let mut counter = 0;
counter += 1;

// Type annotations
let explicit: f64 = 3.14159;
let quantum_state: Qubit = qubit(0);

// Constants
const PI: f64 = 3.14159;
const MAX_ITERATIONS: u32 = 1000;
```

### Quantum Variable Binding
```rust
// Create and bind qubits
let q1 = qubit(0);
let q2 = qubit(0);

// Quantum register binding
let qreg = qubits(5);
let first_qubit = qreg[0];

// Circuit binding
let mut circuit = QuantumCircuit::new(2);
circuit.add_gate(Gate::Hadamard, &[0]);
```

## Functions

### Classical Functions
```rust
// Basic function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with no return value
fn print_message(msg: &str) {
    println!("{}", msg);
}

// Function with multiple return values
fn divide(a: f64, b: f64) -> (f64, Option<f64>) {
    if b != 0.0 {
        (a / b, Some(a / b))
    } else {
        (0.0, None)
    }
}

// Generic function
fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}
```

### Quantum Functions
```rust
// Quantum function declaration
quantum fn prepare_superposition(q: Qubit) -> Qubit {
    hadamard(q);
    q
}

// Multi-qubit quantum function
quantum fn entangle_pair(q1: Qubit, q2: Qubit) -> (Qubit, Qubit) {
    hadamard(q1);
    cnot(q1, q2);
    (q1, q2)
}

// Quantum function with measurement
quantum fn measure_superposition(q: Qubit) -> bool {
    hadamard(q);
    measure(q).value
}

// Quantum circuit function
quantum fn grover_oracle(qubits: &mut [Qubit], target: usize) {
    // Mark the target state
    pauli_z(qubits[target]);
}
```

### Function Parameters
```rust
// By value
fn consume(q: Qubit) { /* q is moved here */ }

// By reference
fn borrow(q: &Qubit) { /* q is borrowed */ }

// Mutable reference
fn modify(q: &mut Qubit) { /* q can be modified */ }

// Optional parameters with defaults
fn run_algorithm(iterations: Option<u32>) {
    let count = iterations.unwrap_or(100);
    // Implementation
}
```

## Classes & Structs

### Struct Definition
```rust
// Basic struct
struct Point {
    x: f64,
    y: f64,
}

// Quantum state struct
struct QuantumState {
    qubits: Vec<Qubit>,
    name: String,
    probability: f64,
}

// Tuple struct
struct Amplitude(f64, f64);  // (real, imaginary)

// Unit struct
struct Marker;
```

### Struct Implementation
```rust
impl Point {
    // Constructor
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    // Method
    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    // Mutable method
    fn move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

impl QuantumState {
    fn new(num_qubits: usize, name: &str) -> Self {
        QuantumState {
            qubits: qubits(num_qubits).into_iter().collect(),
            name: name.to_string(),
            probability: 1.0,
        }
    }
    
    quantum fn apply_hadamard(&mut self, index: usize) {
        if index < self.qubits.len() {
            hadamard(self.qubits[index]);
        }
    }
}
```

### Traits
```rust
// Trait definition
trait Measurable {
    fn measure(&self) -> f64;
    fn reset(&mut self);
}

// Trait implementation
impl Measurable for QuantumState {
    fn measure(&self) -> f64 {
        // Calculate overall probability
        self.probability
    }
    
    fn reset(&mut self) {
        // Reset all qubits to |0⟩
        for qubit in &mut self.qubits {
            *qubit = qubit(0);
        }
        self.probability = 1.0;
    }
}
```

## Quantum Operations

### Basic Quantum Gates
```rust
// Single-qubit gates
let q = qubit(0);
hadamard(q);        // Put in superposition
pauli_x(q);         // Bit flip (NOT gate)
pauli_y(q);         // Y rotation
pauli_z(q);         // Phase flip
phase(q, PI/4);     // Phase rotation

// Measurement
let result = measure(q);
println!("Measured: {} with probability {}", result.value, result.probability);
```

### Two-Qubit Gates
```rust
let q1 = qubit(0);
let q2 = qubit(0);

// Controlled gates
cnot(q1, q2);           // Controlled-X
controlled_z(q1, q2);    // Controlled-Z
controlled_phase(q1, q2, PI/2);  // Controlled phase

// Swap operations
swap(q1, q2);           // Swap qubit states
```

### Quantum Circuits
```rust
// Create a circuit
let mut circuit = QuantumCircuit::new(3);

// Add gates
circuit.hadamard(0);
circuit.cnot(0, 1);
circuit.cnot(1, 2);

// Execute circuit
let qubits = qubits(3);
let result = circuit.execute(qubits);

// Measure all qubits
let measurements = measure_all(result);
```

### Advanced Quantum Operations
```rust
// Quantum Fourier Transform
quantum fn qft(qubits: &mut [Qubit]) {
    let n = qubits.len();
    for i in 0..n {
        hadamard(qubits[i]);
        for j in (i+1)..n {
            let angle = PI / (1 << (j - i));
            controlled_phase(qubits[j], qubits[i], angle);
        }
    }
}

// Grover's algorithm diffusion operator
quantum fn diffusion_operator(qubits: &mut [Qubit]) {
    // Apply H to all qubits
    for qubit in qubits.iter_mut() {
        hadamard(*qubit);
    }
    
    // Flip around average
    for qubit in qubits.iter_mut() {
        pauli_x(*qubit);
    }
    
    // Multi-controlled Z
    multi_controlled_z(qubits);
    
    // Undo X gates
    for qubit in qubits.iter_mut() {
        pauli_x(*qubit);
    }
    
    // Apply H to all qubits
    for qubit in qubits.iter_mut() {
        hadamard(*qubit);
    }
}
```

## Control Flow

### Conditional Statements
```rust
// If-else
let measurement = measure(qubit);
if measurement.value {
    println!("Measured |1⟩");
} else {
    println!("Measured |0⟩");
}

// If-let for pattern matching
if let Some(value) = optional_result {
    println!("Got value: {}", value);
}

// Quantum conditional
let q1 = qubit(0);
let q2 = qubit(0);
hadamard(q1);

if measure(q1).value {
    pauli_x(q2);  // Apply X if q1 measured as |1⟩
}
```

### Loops
```rust
// For loop
for i in 0..10 {
    println!("Iteration: {}", i);
}

// While loop
let mut attempts = 0;
while attempts < 100 {
    let result = run_quantum_algorithm();
    if result.success {
        break;
    }
    attempts += 1;
}

// Loop with quantum operations
let qubits = qubits(5);
for (i, qubit) in qubits.iter().enumerate() {
    if i % 2 == 0 {
        hadamard(qubit);
    }
}
```

### Pattern Matching
```rust
// Match on measurement results
let result = measure(qubit);
match result.value {
    true => println!("Qubit collapsed to |1⟩"),
    false => println!("Qubit collapsed to |0⟩"),
}

// Match on enum
enum QuantumGate {
    Hadamard,
    PauliX,
    PauliY,
    PauliZ,
    CNOT(usize, usize),
}

fn apply_gate(gate: QuantumGate, qubits: &mut [Qubit]) {
    match gate {
        QuantumGate::Hadamard => hadamard(qubits[0]),
        QuantumGate::PauliX => pauli_x(qubits[0]),
        QuantumGate::PauliY => pauli_y(qubits[0]),
        QuantumGate::PauliZ => pauli_z(qubits[0]),
        QuantumGate::CNOT(control, target) => cnot(qubits[control], qubits[target]),
    }
}
```

## Modules & Imports

### Module Declaration
```rust
// In file: quantum_algorithms.aeon
pub mod teleportation {
    pub quantum fn teleport(source: Qubit, ancilla: Qubit, target: Qubit) -> Qubit {
        // Bell state preparation
        hadamard(ancilla);
        cnot(ancilla, target);
        
        // Entangle source with ancilla
        cnot(source, ancilla);
        hadamard(source);
        
        // Measure source and ancilla
        let m1 = measure(source);
        let m2 = measure(ancilla);
        
        // Apply corrections to target
        if m2.value { pauli_x(target); }
        if m1.value { pauli_z(target); }
        
        target
    }
}

pub mod grovers {
    pub quantum fn grovers_search(database_size: usize, target: usize) -> Vec<bool> {
        let num_qubits = (database_size as f64).log2().ceil() as usize;
        let mut qubits = qubits(num_qubits);
        
        // Initialize superposition
        for qubit in &mut qubits {
            hadamard(*qubit);
        }
        
        // Grover iterations
        let iterations = (PI / 4.0 * (database_size as f64).sqrt()) as usize;
        for _ in 0..iterations {
            oracle(&mut qubits, target);
            diffusion_operator(&mut qubits);
        }
        
        // Measure all qubits
        measure_all(qubits).into_iter().map(|r| r.value).collect()
    }
}
```

### Import Usage
```rust
// Import specific functions
use quantum_algorithms::teleportation::teleport;
use quantum_algorithms::grovers::grovers_search;

// Import entire module
use quantum_algorithms::teleportation;

// Import with alias
use quantum_algorithms::grovers as grover_mod;

// Import multiple items
use quantum_algorithms::{teleportation, grovers};

// Re-export
pub use quantum_algorithms::*;

fn main() {
    // Use imported functions
    let result = grovers_search(16, 10);
    println!("Grover's result: {:?}", result);
    
    // Use module prefix
    let qubits = qubits(3);
    let teleported = teleportation::teleport(qubits[0], qubits[1], qubits[2]);
}
```

## Error Handling

### Result Type
```rust
// Function that can fail
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Quantum function with error handling
quantum fn controlled_rotation(control: Qubit, target: Qubit, angle: f64) -> Result<(), String> {
    if angle < 0.0 || angle > 2.0 * PI {
        return Err("Angle must be between 0 and 2π".to_string());
    }
    
    controlled_phase(control, target, angle);
    Ok(())
}
```

### Error Propagation
```rust
// Using ? operator
fn quantum_computation() -> Result<f64, String> {
    let q1 = qubit(0);
    let q2 = qubit(0);
    
    // This will propagate any errors
    controlled_rotation(q1, q2, PI/2)?;
    
    let result = measure(q2);
    Ok(result.probability)
}

// Handling errors
fn main() {
    match quantum_computation() {
        Ok(probability) => println!("Success: {}", probability),
        Err(error) => eprintln!("Error: {}", error),
    }
}
```

### Option Type
```rust
// Function that might not return a value
fn find_qubit(qubits: &[Qubit], condition: impl Fn(&Qubit) -> bool) -> Option<&Qubit> {
    qubits.iter().find(|q| condition(q))
}

// Using Option
fn main() {
    let qubits = qubits(5);
    
    if let Some(qubit) = find_qubit(&qubits, |q| measure(q).probability > 0.5) {
        println!("Found high-probability qubit");
    } else {
        println!("No suitable qubit found");
    }
}
```

## Best Practices

### Code Organization
```rust
// Group related quantum operations
mod quantum_utils {
    pub quantum fn prepare_ghz_state(qubits: &mut [Qubit]) {
        hadamard(qubits[0]);
        for i in 1..qubits.len() {
            cnot(qubits[0], qubits[i]);
        }
    }
    
    pub fn measure_parity(qubits: &[Qubit]) -> bool {
        qubits.iter()
              .map(|q| measure(q).value)
              .fold(false, |acc, x| acc ^ x)
    }
}
```

### Error Handling Best Practices
```rust
// Always handle quantum measurement uncertainty
quantum fn run_experiment() -> Result<ExperimentResult, QuantumError> {
    let qubits = qubits(5)?;
    
    // Prepare state
    prepare_state(&mut qubits)?;
    
    // Run algorithm
    let measurements = run_algorithm(&qubits)?;
    
    // Analyze results
    if measurements.len() != qubits.len() {
        return Err(QuantumError::MeasurementMismatch);
    }
    
    Ok(ExperimentResult::new(measurements))
}
```

### Documentation
```rust
/// Implements the Quantum Fourier Transform on a register of qubits.
/// 
/// # Arguments
/// * `qubits` - A mutable slice of qubits to transform
/// 
/// # Example
/// ```
/// let mut qreg = qubits(3);
/// qft(&mut qreg);
/// let measurements = measure_all(qreg);
/// ```
/// 
/// # Quantum Complexity
/// - Gate count: O(n²) where n is the number of qubits
/// - Depth: O(n²) for sequential implementation
quantum fn qft(qubits: &mut [Qubit]) {
    // Implementation here
}
```

---

This language reference provides comprehensive coverage of Aeonmi's syntax and features. For more detailed examples and tutorials, see the [Tutorials](../tutorials/) section.