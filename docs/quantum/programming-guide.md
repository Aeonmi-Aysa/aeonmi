# Quantum Programming with Aeonmi

Complete guide to quantum programming concepts and implementation in Aeonmi.

## Table of Contents

1. [Quantum Computing Basics](#quantum-computing-basics)
2. [Qubits in Aeonmi](#qubits-in-aeonmi)
3. [Quantum Gates and Operations](#quantum-gates-and-operations)
4. [Quantum Circuits](#quantum-circuits)
5. [Measurement and Results](#measurement-and-results)
6. [Quantum Algorithms](#quantum-algorithms)
7. [Advanced Quantum Features](#advanced-quantum-features)
8. [Quantum-Classical Hybrid Programming](#quantum-classical-hybrid-programming)
9. [Best Practices](#best-practices)
10. [Common Patterns](#common-patterns)

## Quantum Computing Basics

### What is Quantum Computing?

Quantum computing leverages quantum mechanical phenomena like superposition and entanglement to process information in fundamentally different ways than classical computers.

#### Key Concepts

**Superposition**: A qubit can exist in a combination of |0⟩ and |1⟩ states simultaneously.
```rust
// Classical bit: definitely 0 or 1
let classical_bit = true;  // definitely 1

// Quantum bit: can be in superposition
let qubit = qubit(0);      // starts as |0⟩
hadamard(qubit);           // now in superposition: (|0⟩ + |1⟩)/√2
```

**Entanglement**: Multiple qubits can be correlated in ways that classical systems cannot be.
```rust
// Create entangled Bell state
let q1 = qubit(0);
let q2 = qubit(0);
hadamard(q1);
cnot(q1, q2);
// Now q1 and q2 are entangled: (|00⟩ + |11⟩)/√2
```

**Measurement**: Observing a quantum system collapses it to a definite classical state.
```rust
let q = qubit(0);
hadamard(q);              // 50% chance of |0⟩ or |1⟩
let result = measure(q);  // Collapses to definite value
```

## Qubits in Aeonmi

### Creating Qubits

```rust
// Single qubit in |0⟩ state
let q0 = qubit(0);

// Single qubit in |1⟩ state  
let q1 = qubit(1);

// Multiple qubits (register)
let qreg = qubits(5);  // 5 qubits, all in |0⟩

// Named qubits for clarity
let alice_qubit = qubit(0);
let bob_qubit = qubit(0);
let ancilla = qubit(0);
```

### Qubit Types and Properties

```rust
// Qubit type represents a quantum bit
let q: Qubit = qubit(0);

// Qubit register for multiple qubits
let register: Vec<Qubit> = qubits(3);

// Access individual qubits from register
let first = register[0];
let second = register[1];

// Create specific quantum states
let zero = qubit(0);        // |0⟩ state
let one = qubit(1);         // |1⟩ state
```

### Qubit Lifecycle

```rust
quantum fn qubit_lifecycle_demo() {
    // 1. Creation
    let q = qubit(0);  // |0⟩
    
    // 2. Manipulation
    hadamard(q);       // |+⟩ = (|0⟩ + |1⟩)/√2
    
    // 3. Further operations
    phase(q, PI/4);    // Apply phase rotation
    
    // 4. Measurement (destroys superposition)
    let result = measure(q);
    
    // 5. Qubit is now classical
    println!("Measured: {}", result.value);
}
```

## Quantum Gates and Operations

### Single-Qubit Gates

#### Pauli Gates
```rust
quantum fn pauli_gates_demo() {
    let q = qubit(0);
    
    // Pauli-X (NOT gate, bit flip)
    pauli_x(q);        // |0⟩ → |1⟩, |1⟩ → |0⟩
    
    // Pauli-Y (bit and phase flip)
    pauli_y(q);        // |0⟩ → i|1⟩, |1⟩ → -i|0⟩
    
    // Pauli-Z (phase flip)
    pauli_z(q);        // |0⟩ → |0⟩, |1⟩ → -|1⟩
}
```

#### Hadamard Gate
```rust
quantum fn hadamard_demo() {
    let q = qubit(0);
    
    // Create superposition
    hadamard(q);       // |0⟩ → (|0⟩ + |1⟩)/√2
    
    // Measure to see random result
    let result = measure(q);
    println!("50% chance result: {}", result.value);
}
```

#### Rotation Gates
```rust
quantum fn rotation_demo() {
    let q = qubit(0);
    
    // Phase rotation
    phase(q, PI/4);    // Rotate around Z-axis
    
    // Rotation around X-axis
    rx(q, PI/2);       // 90-degree X rotation
    
    // Rotation around Y-axis
    ry(q, PI/3);       // 60-degree Y rotation
    
    // Rotation around Z-axis
    rz(q, PI/6);       // 30-degree Z rotation
}
```

#### T and S Gates
```rust
quantum fn t_s_gates_demo() {
    let q = qubit(0);
    
    // S gate (√Z)
    s_gate(q);         // phase(q, PI/2)
    
    // T gate (√S)
    t_gate(q);         // phase(q, PI/4)
    
    // Adjoint operations
    s_dagger(q);       // phase(q, -PI/2)
    t_dagger(q);       // phase(q, -PI/4)
}
```

### Two-Qubit Gates

#### CNOT Gate
```rust
quantum fn cnot_demo() {
    let control = qubit(1);
    let target = qubit(0);
    
    // Controlled-NOT: flips target if control is |1⟩
    cnot(control, target);
    
    // Result: |10⟩ → |11⟩, |00⟩ → |00⟩
    let c_result = measure(control);
    let t_result = measure(target);
    println!("Control: {}, Target: {}", c_result.value, t_result.value);
}
```

#### Controlled Gates
```rust
quantum fn controlled_gates_demo() {
    let control = qubit(0);
    let target = qubit(0);
    
    // Prepare control in superposition
    hadamard(control);
    
    // Controlled-Z gate
    controlled_z(control, target);
    
    // Controlled phase rotation
    controlled_phase(control, target, PI/4);
    
    // Controlled Hadamard
    controlled_hadamard(control, target);
}
```

#### Swap Operations
```rust
quantum fn swap_demo() {
    let q1 = qubit(0);
    let q2 = qubit(1);
    
    println!("Before swap: q1=|0⟩, q2=|1⟩");
    
    // Swap the states
    swap(q1, q2);
    
    println!("After swap: q1=|1⟩, q2=|0⟩");
    
    // Fredkin gate (controlled swap)
    let control = qubit(1);
    fredkin(control, q1, q2);  // Swap q1,q2 if control is |1⟩
}
```

### Multi-Qubit Gates

```rust
quantum fn multi_qubit_gates() {
    let qubits = qubits(3);
    
    // Toffoli gate (CCNOT)
    toffoli(qubits[0], qubits[1], qubits[2]);
    
    // Multi-controlled NOT
    let controls = &qubits[0..2];
    let target = qubits[2];
    multi_controlled_not(controls, target);
    
    // Multi-controlled phase
    multi_controlled_phase(controls, target, PI/4);
}
```

## Quantum Circuits

### Basic Circuit Construction

```rust
quantum fn basic_circuit() {
    // Create circuit for 3 qubits
    let mut circuit = QuantumCircuit::new(3);
    
    // Add gates to circuit
    circuit.hadamard(0);
    circuit.cnot(0, 1);
    circuit.cnot(1, 2);
    circuit.measure_all();
    
    // Execute circuit
    let qubits = qubits(3);
    let results = circuit.execute(qubits);
    
    println!("GHZ state measurements: {:?}", results);
}
```

### Parameterized Circuits

```rust
quantum fn parameterized_circuit(theta: f64, phi: f64) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2);
    
    // Parameterized gates
    circuit.ry(0, theta);
    circuit.rz(0, phi);
    circuit.cnot(0, 1);
    circuit.ry(1, -theta);
    
    circuit
}

quantum fn use_parameterized_circuit() {
    let circuit = parameterized_circuit(PI/4, PI/3);
    let qubits = qubits(2);
    let results = circuit.execute(qubits);
}
```

### Circuit Composition

```rust
quantum fn compose_circuits() {
    // Create subcircuits
    let prep_circuit = bell_state_preparation();
    let measurement_circuit = bell_basis_measurement();
    
    // Compose into larger circuit
    let mut full_circuit = QuantumCircuit::new(4);
    full_circuit.append(prep_circuit, &[0, 1]);
    full_circuit.append(measurement_circuit, &[2, 3]);
    
    let qubits = qubits(4);
    let results = full_circuit.execute(qubits);
}

quantum fn bell_state_preparation() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2);
    circuit.hadamard(0);
    circuit.cnot(0, 1);
    circuit
}

quantum fn bell_basis_measurement() -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(2);
    circuit.cnot(0, 1);
    circuit.hadamard(0);
    circuit
}
```

## Measurement and Results

### Basic Measurement

```rust
quantum fn basic_measurement() {
    let q = qubit(0);
    hadamard(q);  // 50/50 superposition
    
    // Single measurement
    let result = measure(q);
    println!("Value: {}", result.value);          // true or false
    println!("Probability: {}", result.probability); // ~0.5
}
```

### Multiple Measurements

```rust
quantum fn multiple_measurements() {
    let qubits = qubits(3);
    
    // Prepare GHZ state
    hadamard(qubits[0]);
    cnot(qubits[0], qubits[1]);
    cnot(qubits[1], qubits[2]);
    
    // Measure all qubits
    let results = measure_all(qubits);
    
    for (i, result) in results.iter().enumerate() {
        println!("Qubit {}: {} (p={})", i, result.value, result.probability);
    }
}
```

### Statistical Analysis

```rust
quantum fn statistical_measurement() {
    let shots = 1000;
    let mut counts = std::collections::HashMap::new();
    
    for _ in 0..shots {
        // Prepare Bell state
        let q1 = qubit(0);
        let q2 = qubit(0);
        hadamard(q1);
        cnot(q1, q2);
        
        // Measure both qubits
        let r1 = measure(q1);
        let r2 = measure(q2);
        let outcome = format!("{}{}", r1.value as u8, r2.value as u8);
        
        *counts.entry(outcome).or_insert(0) += 1;
    }
    
    println!("Bell state measurement statistics:");
    for (outcome, count) in counts {
        let probability = count as f64 / shots as f64;
        println!("{}: {} ({:.1}%)", outcome, count, probability * 100.0);
    }
    // Expected: roughly 50% "00" and 50% "11", 0% "01" and "10"
}
```

### Conditional Measurement

```rust
quantum fn conditional_measurement() {
    let data = qubit(1);    // Data qubit
    let ancilla = qubit(0); // Ancilla qubit
    
    // Entangle data with ancilla
    cnot(data, ancilla);
    
    // Measure ancilla
    let ancilla_result = measure(ancilla);
    
    if ancilla_result.value {
        println!("Ancilla measured |1⟩, data is definitely |1⟩");
        // No need to measure data, we know it's |1⟩
    } else {
        println!("Ancilla measured |0⟩, data is definitely |0⟩");
        // No need to measure data, we know it's |0⟩
    }
}
```

## Quantum Algorithms

### Quantum Teleportation

```rust
quantum fn quantum_teleportation(alice_qubit: Qubit) -> bool {
    let bob_qubit = qubit(0);
    let ancilla = qubit(0);
    
    // Step 1: Create Bell pair between ancilla and Bob
    hadamard(ancilla);
    cnot(ancilla, bob_qubit);
    
    // Step 2: Alice entangles her qubit with ancilla
    cnot(alice_qubit, ancilla);
    hadamard(alice_qubit);
    
    // Step 3: Alice measures her qubits
    let m1 = measure(alice_qubit);
    let m2 = measure(ancilla);
    
    // Step 4: Apply corrections to Bob's qubit based on Alice's measurements
    if m2.value {
        pauli_x(bob_qubit);
    }
    if m1.value {
        pauli_z(bob_qubit);
    }
    
    // Bob now has Alice's original state
    measure(bob_qubit).value
}

quantum fn teleportation_demo() {
    // Prepare Alice's qubit in arbitrary state
    let alice = qubit(1);  // |1⟩ state
    // Could also prepare superposition: hadamard(alice);
    
    let original_state = measure_state_vector(alice);  // For verification
    
    // Teleport the state
    let bob_result = quantum_teleportation(alice);
    
    println!("Teleportation successful: Alice's |1⟩ → Bob's {}", bob_result);
}
```

### Grover's Algorithm

```rust
quantum fn grovers_algorithm(database_size: usize, target: usize) -> Vec<bool> {
    let num_qubits = (database_size as f64).log2().ceil() as usize;
    let mut qubits = qubits(num_qubits);
    
    // Step 1: Initialize superposition over all states
    for qubit in &mut qubits {
        hadamard(*qubit);
    }
    
    // Step 2: Grover iterations
    let iterations = (PI / 4.0 * (database_size as f64).sqrt()) as usize;
    
    for i in 0..iterations {
        println!("Grover iteration {}/{}", i + 1, iterations);
        
        // Oracle: mark the target state
        grover_oracle(&mut qubits, target);
        
        // Diffusion operator: amplify marked amplitude
        grover_diffusion(&mut qubits);
    }
    
    // Step 3: Measure all qubits
    measure_all(qubits).into_iter().map(|r| r.value).collect()
}

quantum fn grover_oracle(qubits: &mut [Qubit], target: usize) {
    // Convert target to binary and apply controlled-Z
    for (i, bit) in format!("{:0width$b}", target, width = qubits.len())
        .chars()
        .enumerate()
    {
        if bit == '0' {
            pauli_x(qubits[i]);  // Flip for negative control
        }
    }
    
    // Multi-controlled Z gate
    multi_controlled_z(qubits);
    
    // Flip back
    for (i, bit) in format!("{:0width$b}", target, width = qubits.len())
        .chars()
        .enumerate()
    {
        if bit == '0' {
            pauli_x(qubits[i]);
        }
    }
}

quantum fn grover_diffusion(qubits: &mut [Qubit]) {
    // 2|s⟩⟨s| - I where |s⟩ is uniform superposition
    
    // Apply H to all qubits
    for qubit in qubits.iter_mut() {
        hadamard(*qubit);
    }
    
    // Flip around |0...0⟩
    for qubit in qubits.iter_mut() {
        pauli_x(*qubit);
    }
    
    multi_controlled_z(qubits);
    
    for qubit in qubits.iter_mut() {
        pauli_x(*qubit);
    }
    
    // Apply H to all qubits
    for qubit in qubits.iter_mut() {
        hadamard(*qubit);
    }
}

quantum fn grovers_demo() {
    let database_size = 16;  // 4-qubit search space
    let target = 10;         // Looking for item 10
    
    println!("Searching for item {} in database of size {}", target, database_size);
    
    let result = grovers_algorithm(database_size, target);
    let measured_value = binary_to_decimal(&result);
    
    println!("Grover's result: {} (binary: {:?})", measured_value, result);
    
    if measured_value == target {
        println!("✓ Found target with high probability!");
    } else {
        println!("✗ Different result - try running again (quantum is probabilistic)");
    }
}
```

### Quantum Fourier Transform

```rust
quantum fn quantum_fourier_transform(qubits: &mut [Qubit]) {
    let n = qubits.len();
    
    for i in 0..n {
        // Apply Hadamard to current qubit
        hadamard(qubits[i]);
        
        // Apply controlled phase rotations
        for j in (i + 1)..n {
            let angle = PI / (1 << (j - i));
            controlled_phase(qubits[j], qubits[i], angle);
        }
    }
    
    // Reverse the order of qubits
    for i in 0..(n / 2) {
        swap(qubits[i], qubits[n - 1 - i]);
    }
}

quantum fn inverse_qft(qubits: &mut [Qubit]) {
    let n = qubits.len();
    
    // Reverse the order of qubits first
    for i in 0..(n / 2) {
        swap(qubits[i], qubits[n - 1 - i]);
    }
    
    for i in (0..n).rev() {
        // Apply controlled phase rotations (inverse)
        for j in (i + 1)..n {
            let angle = -PI / (1 << (j - i));
            controlled_phase(qubits[j], qubits[i], angle);
        }
        
        // Apply Hadamard to current qubit
        hadamard(qubits[i]);
    }
}

quantum fn qft_demo() {
    let mut qubits = qubits(3);
    
    // Prepare some initial state
    pauli_x(qubits[0]);     // |100⟩
    
    println!("Initial state: |100⟩");
    
    // Apply QFT
    quantum_fourier_transform(&mut qubits);
    
    println!("After QFT: Fourier coefficients in superposition");
    
    // Apply inverse QFT to recover original state
    inverse_qft(&mut qubits);
    
    println!("After inverse QFT: back to |100⟩");
    
    let results = measure_all(qubits);
    println!("Final measurement: {:?}", results);
}
```

### Deutsch-Jozsa Algorithm

```rust
quantum fn deutsch_jozsa(oracle: impl Fn(&mut [Qubit], &mut Qubit)) -> bool {
    let n = 3; // Number of input qubits
    let mut input_qubits = qubits(n);
    let mut output_qubit = qubit(1); // Initialize to |1⟩
    
    // Step 1: Prepare superposition of all inputs
    for qubit in &mut input_qubits {
        hadamard(*qubit);
    }
    
    // Prepare output qubit in |−⟩ state
    hadamard(output_qubit);
    
    // Step 2: Apply oracle
    oracle(&mut input_qubits, &mut output_qubit);
    
    // Step 3: Apply Hadamard to input qubits
    for qubit in &mut input_qubits {
        hadamard(*qubit);
    }
    
    // Step 4: Measure input qubits
    let results = measure_all(input_qubits);
    
    // If all measurements are 0, function is constant
    // If any measurement is 1, function is balanced
    results.iter().any(|r| r.value)
}

// Example oracles
quantum fn constant_oracle_zero(inputs: &mut [Qubit], output: &mut Qubit) {
    // Do nothing - f(x) = 0 for all x
}

quantum fn constant_oracle_one(inputs: &mut [Qubit], output: &mut Qubit) {
    // Flip output - f(x) = 1 for all x
    pauli_x(*output);
}

quantum fn balanced_oracle(inputs: &mut [Qubit], output: &mut Qubit) {
    // Example: f(x) = x_0 (output depends on first input bit)
    cnot(inputs[0], *output);
}

quantum fn deutsch_jozsa_demo() {
    println!("Testing constant oracle (always 0):");
    let is_balanced = deutsch_jozsa(constant_oracle_zero);
    println!("Result: {} (false = constant)", is_balanced);
    
    println!("\nTesting balanced oracle:");
    let is_balanced = deutsch_jozsa(balanced_oracle);
    println!("Result: {} (true = balanced)", is_balanced);
}
```

## Advanced Quantum Features

### Quantum Error Correction

```rust
quantum fn three_qubit_bit_flip_code(logical_qubit: Qubit) -> Vec<Qubit> {
    let ancilla1 = qubit(0);
    let ancilla2 = qubit(0);
    
    // Encode logical qubit into 3 physical qubits
    cnot(logical_qubit, ancilla1);
    cnot(logical_qubit, ancilla2);
    
    vec![logical_qubit, ancilla1, ancilla2]
}

quantum fn error_correction_demo() {
    let data = qubit(1); // Logical |1⟩
    
    // Encode
    let encoded = three_qubit_bit_flip_code(data);
    
    // Simulate bit flip error on second qubit
    pauli_x(encoded[1]);
    
    // Error detection and correction
    let syndrome1 = measure_parity(&[encoded[0], encoded[1]]);
    let syndrome2 = measure_parity(&[encoded[0], encoded[2]]);
    
    match (syndrome1, syndrome2) {
        (false, false) => println!("No error detected"),
        (true, false) => {
            println!("Error on qubit 1, correcting...");
            pauli_x(encoded[1]);
        },
        (false, true) => {
            println!("Error on qubit 2, correcting...");
            pauli_x(encoded[2]);
        },
        (true, true) => {
            println!("Error on qubit 0, correcting...");
            pauli_x(encoded[0]);
        },
    }
    
    // Decode and verify
    let recovered = measure(encoded[0]);
    println!("Recovered logical bit: {}", recovered.value);
}

fn measure_parity(qubits: &[Qubit]) -> bool {
    qubits.iter()
          .map(|q| measure(*q).value)
          .fold(false, |acc, x| acc ^ x)
}
```

### Quantum Phase Estimation

```rust
quantum fn quantum_phase_estimation(
    unitary: impl Fn(&mut Qubit, u32),  // U^(2^k) operation
    eigenstate: Qubit,                   // Eigenstate of U
    precision_qubits: usize
) -> f64 {
    let mut counting_qubits = qubits(precision_qubits);
    
    // Step 1: Initialize counting register in superposition
    for qubit in &mut counting_qubits {
        hadamard(*qubit);
    }
    
    // Step 2: Apply controlled-U^(2^k) operations
    for (k, control) in counting_qubits.iter().enumerate() {
        let power = 1 << k;
        controlled_unitary(*control, eigenstate, &unitary, power);
    }
    
    // Step 3: Apply inverse QFT to counting register
    inverse_qft(&mut counting_qubits);
    
    // Step 4: Measure counting register
    let measurements = measure_all(counting_qubits);
    let measured_value = binary_to_decimal(&measurements.iter().map(|r| r.value).collect::<Vec<_>>());
    
    // Convert to phase estimate
    let phase = measured_value as f64 / (1 << precision_qubits) as f64;
    phase
}
```

### Variational Quantum Algorithms

```rust
quantum fn variational_quantum_eigensolver(
    hamiltonian: &Hamiltonian,
    ansatz: impl Fn(&mut [Qubit], &[f64]),
    initial_params: Vec<f64>
) -> (f64, Vec<f64>) {
    let mut params = initial_params;
    let mut best_energy = f64::INFINITY;
    let learning_rate = 0.1;
    
    for iteration in 0..100 {
        // Prepare quantum state with current parameters
        let mut qubits = qubits(hamiltonian.num_qubits());
        ansatz(&mut qubits, &params);
        
        // Measure expectation value of Hamiltonian
        let energy = measure_expectation_value(&qubits, hamiltonian);
        
        if energy < best_energy {
            best_energy = energy;
            println!("Iteration {}: New best energy = {:.6}", iteration, energy);
        }
        
        // Compute parameter gradients (simplified)
        let gradients = compute_gradients(&params, &ansatz, hamiltonian);
        
        // Update parameters
        for (param, grad) in params.iter_mut().zip(gradients.iter()) {
            *param -= learning_rate * grad;
        }
    }
    
    (best_energy, params)
}

// Example ansatz for VQE
quantum fn hardware_efficient_ansatz(qubits: &mut [Qubit], params: &[f64]) {
    let n = qubits.len();
    let mut param_idx = 0;
    
    // Layer 1: RY rotations
    for i in 0..n {
        ry(qubits[i], params[param_idx]);
        param_idx += 1;
    }
    
    // Layer 2: Entangling gates
    for i in 0..(n-1) {
        cnot(qubits[i], qubits[i+1]);
    }
    
    // Layer 3: More RY rotations
    for i in 0..n {
        ry(qubits[i], params[param_idx]);
        param_idx += 1;
    }
}
```

## Quantum-Classical Hybrid Programming

### Variational Algorithm Pattern

```rust
fn quantum_classical_optimization() {
    let mut classical_params = vec![0.1, 0.2, 0.3];
    let target_value = 0.0; // Minimizing energy
    
    for iteration in 0..50 {
        // Classical preprocessing
        let preprocessed_data = classical_preprocessing(&classical_params);
        
        // Quantum computation
        let quantum_result = quantum_subroutine(&preprocessed_data);
        
        // Classical postprocessing
        let cost = compute_cost(quantum_result, target_value);
        
        // Classical optimization
        classical_params = update_parameters(classical_params, cost);
        
        println!("Iteration {}: Cost = {:.6}", iteration, cost);
        
        if cost < 1e-6 {
            println!("Converged!");
            break;
        }
    }
}

fn classical_preprocessing(params: &[f64]) -> Vec<f64> {
    // Classical data preparation
    params.iter().map(|x| x.sin()).collect()
}

quantum fn quantum_subroutine(data: &[f64]) -> f64 {
    let mut qubits = qubits(data.len());
    
    // Encode classical data into quantum state
    for (qubit, &value) in qubits.iter_mut().zip(data.iter()) {
        ry(*qubit, value);
    }
    
    // Quantum computation
    for i in 0..(qubits.len() - 1) {
        cnot(qubits[i], qubits[i + 1]);
    }
    
    // Measurement
    let measurements = measure_all(qubits);
    
    // Return classical result
    measurements.iter().map(|r| if r.value { 1.0 } else { 0.0 }).sum()
}

fn compute_cost(quantum_result: f64, target: f64) -> f64 {
    (quantum_result - target).powi(2)
}

fn update_parameters(params: Vec<f64>, cost: f64) -> Vec<f64> {
    // Simple gradient-free optimization
    let learning_rate = 0.01;
    params.into_iter()
          .map(|p| p - learning_rate * cost.signum() * 0.1)
          .collect()
}
```

### Quantum Machine Learning

```rust
quantum fn quantum_neural_network(inputs: &[f64], weights: &[f64]) -> Vec<f64> {
    let n_qubits = inputs.len();
    let mut qubits = qubits(n_qubits);
    
    // Encode inputs
    for (qubit, &input) in qubits.iter_mut().zip(inputs.iter()) {
        ry(*qubit, input * PI);  // Amplitude encoding
    }
    
    // Parameterized quantum circuit (trainable layer)
    let mut weight_idx = 0;
    
    // Layer 1: Individual rotations
    for qubit in &mut qubits {
        rx(*qubit, weights[weight_idx]);
        weight_idx += 1;
        rz(*qubit, weights[weight_idx]);
        weight_idx += 1;
    }
    
    // Layer 2: Entangling layer
    for i in 0..(n_qubits - 1) {
        cnot(qubits[i], qubits[i + 1]);
    }
    
    // Layer 3: Final rotations
    for qubit in &mut qubits {
        rx(*qubit, weights[weight_idx]);
        weight_idx += 1;
    }
    
    // Measurement
    measure_all(qubits).into_iter()
                       .map(|r| if r.value { 1.0 } else { -1.0 })
                       .collect()
}

fn train_quantum_classifier() {
    let training_data = vec![
        (vec![0.1, 0.2], 1.0),  // (features, label)
        (vec![0.8, 0.9], -1.0),
        (vec![0.2, 0.1], 1.0),
        (vec![0.9, 0.8], -1.0),
    ];
    
    let mut weights = vec![0.1; 12]; // 4 qubits * 3 parameters per qubit
    let learning_rate = 0.1;
    
    for epoch in 0..100 {
        let mut total_loss = 0.0;
        
        for (features, true_label) in &training_data {
            // Forward pass
            let predictions = quantum_neural_network(features, &weights);
            let predicted_label = predictions[0]; // Use first qubit output
            
            // Compute loss
            let loss = (predicted_label - true_label).powi(2);
            total_loss += loss;
            
            // Backward pass (simplified parameter shift rule)
            for i in 0..weights.len() {
                let shift = PI / 2.0;
                
                // Forward pass with positive shift
                weights[i] += shift;
                let pred_plus = quantum_neural_network(features, &weights)[0];
                
                // Forward pass with negative shift
                weights[i] -= 2.0 * shift;
                let pred_minus = quantum_neural_network(features, &weights)[0];
                
                // Restore original weight
                weights[i] += shift;
                
                // Compute gradient
                let gradient = (pred_plus - pred_minus) / 2.0;
                
                // Update weight
                weights[i] -= learning_rate * gradient * (predicted_label - true_label);
            }
        }
        
        if epoch % 10 == 0 {
            println!("Epoch {}: Loss = {:.6}", epoch, total_loss);
        }
    }
}
```

## Best Practices

### Code Organization

```rust
// Organize quantum functions by purpose
mod quantum_algorithms {
    pub mod search {
        pub use super::super::grovers_algorithm;
    }
    
    pub mod teleportation {
        pub use super::super::quantum_teleportation;
    }
    
    pub mod fourier {
        pub use super::super::quantum_fourier_transform;
    }
}

// Use descriptive names
quantum fn prepare_bell_state(q1: Qubit, q2: Qubit) -> (Qubit, Qubit) {
    hadamard(q1);
    cnot(q1, q2);
    (q1, q2)
}

// Document quantum functions clearly
/// Creates maximally entangled Bell state |Φ⁺⟩ = (|00⟩ + |11⟩)/√2
/// 
/// # Arguments
/// * `control` - Control qubit (will be in superposition)
/// * `target` - Target qubit (will be entangled with control)
/// 
/// # Returns
/// Tuple of entangled qubits in Bell state
/// 
/// # Quantum State
/// Input:  |00⟩
/// Output: (|00⟩ + |11⟩)/√2
quantum fn create_bell_pair(control: Qubit, target: Qubit) -> (Qubit, Qubit) {
    hadamard(control);
    cnot(control, target);
    (control, target)
}
```

### Error Handling

```rust
quantum fn safe_rotation(qubit: Qubit, angle: f64) -> Result<(), QuantumError> {
    if angle < 0.0 || angle > 2.0 * PI {
        return Err(QuantumError::InvalidAngle(angle));
    }
    
    ry(qubit, angle);
    Ok(())
}

quantum fn robust_measurement(qubits: &[Qubit], retries: usize) -> Result<Vec<bool>, QuantumError> {
    for attempt in 0..retries {
        match try_measurement(qubits) {
            Ok(results) => return Ok(results),
            Err(e) if attempt < retries - 1 => {
                println!("Measurement failed (attempt {}), retrying...", attempt + 1);
                continue;
            },
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

### Testing Quantum Code

```rust
#[cfg(test)]
mod quantum_tests {
    use super::*;
    
    #[test]
    fn test_bell_state_preparation() {
        let q1 = qubit(0);
        let q2 = qubit(0);
        
        let (bell_q1, bell_q2) = create_bell_pair(q1, q2);
        
        // Test correlation: both qubits should always measure the same
        let measurements: Vec<(bool, bool)> = (0..100)
            .map(|_| {
                let (fresh_q1, fresh_q2) = create_bell_pair(qubit(0), qubit(0));
                (measure(fresh_q1).value, measure(fresh_q2).value)
            })
            .collect();
        
        // All measurements should be correlated
        let all_correlated = measurements.iter().all(|(m1, m2)| m1 == m2);
        assert!(all_correlated, "Bell state measurements should be perfectly correlated");
        
        // Should see both 00 and 11 outcomes
        let has_00 = measurements.iter().any(|(m1, m2)| !m1 && !m2);
        let has_11 = measurements.iter().any(|(m1, m2)| *m1 && *m2);
        assert!(has_00 && has_11, "Should observe both |00⟩ and |11⟩ outcomes");
    }
    
    #[test]
    fn test_grover_single_iteration() {
        // Test Grover's algorithm for 4-item database
        let result = grovers_algorithm(4, 2);
        let measured_value = binary_to_decimal(&result);
        
        // With optimal iterations, should find target with high probability
        // (May occasionally fail due to quantum probabilistic nature)
        let success_probability = 0.8; // 80% success rate acceptable for test
        
        // Run multiple times to check probabilistic success
        let successes = (0..10)
            .map(|_| grovers_algorithm(4, 2))
            .map(|result| binary_to_decimal(&result))
            .filter(|&value| value == 2)
            .count();
        
        let actual_probability = successes as f64 / 10.0;
        assert!(
            actual_probability >= success_probability,
            "Grover's algorithm should succeed with probability >= {}%, got {}%",
            success_probability * 100.0,
            actual_probability * 100.0
        );
    }
}
```

## Common Patterns

### Quantum State Preparation

```rust
// Uniform superposition
quantum fn uniform_superposition(qubits: &mut [Qubit]) {
    for qubit in qubits {
        hadamard(*qubit);
    }
}

// Computational basis state
quantum fn computational_basis_state(qubits: &mut [Qubit], state: usize) {
    for (i, qubit) in qubits.iter_mut().enumerate() {
        if (state >> i) & 1 == 1 {
            pauli_x(*qubit);
        }
    }
}

// W state: |W_n⟩ = (|100...0⟩ + |010...0⟩ + ... + |000...1⟩)/√n
quantum fn w_state(qubits: &mut [Qubit]) {
    let n = qubits.len();
    
    // Recursive construction of W state
    for i in 0..n-1 {
        let theta = (1.0 / (n - i) as f64).sqrt().acos() * 2.0;
        ry(qubits[i], theta);
        
        if i > 0 {
            cnot(qubits[i-1], qubits[i]);
        }
    }
}
```

### Measurement Patterns

```rust
// Parity measurement
quantum fn measure_parity(qubits: &[Qubit]) -> bool {
    qubits.iter()
          .map(|&q| measure(q).value)
          .fold(false, |acc, bit| acc ^ bit)
}

// Majority vote measurement
quantum fn majority_measurement(qubits: &[Qubit]) -> bool {
    let measurements: Vec<bool> = qubits.iter()
                                       .map(|&q| measure(q).value)
                                       .collect();
    
    let true_count = measurements.iter().filter(|&&b| b).count();
    true_count > measurements.len() / 2
}

// Statistical measurement
quantum fn statistical_measurement(
    prepare_state: impl Fn() -> Qubit,
    shots: usize
) -> (usize, usize) {
    let mut zeros = 0;
    let mut ones = 0;
    
    for _ in 0..shots {
        let q = prepare_state();
        if measure(q).value {
            ones += 1;
        } else {
            zeros += 1;
        }
    }
    
    (zeros, ones)
}
```

### Utility Functions

```rust
// Convert binary measurement results to decimal
fn binary_to_decimal(bits: &[bool]) -> usize {
    bits.iter()
        .enumerate()
        .map(|(i, &bit)| if bit { 1 << i } else { 0 })
        .sum()
}

// Calculate fidelity between quantum states
fn state_fidelity(state1: &StateVector, state2: &StateVector) -> f64 {
    // Implementation depends on state representation
    // |⟨ψ₁|ψ₂⟩|²
    unimplemented!("Requires state vector representation")
}

// Check if qubits are maximally entangled
fn is_maximally_entangled(qubits: &[Qubit]) -> bool {
    // Measure entanglement entropy
    // For Bell states: S = 1 (maximal)
    // Implementation requires density matrix operations
    unimplemented!("Requires entanglement entropy calculation")
}
```

---

This comprehensive quantum programming guide covers the essential concepts and practical implementation patterns for quantum computing with Aeonmi. For more hands-on experience, proceed to the [Tutorials](../tutorials/) section to build complete quantum applications.