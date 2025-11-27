# Quantum Error Correction in AEONMI

Complete guide to quantum error correction algorithms and implementation in AEONMI.

## Table of Contents

1. [Introduction to Quantum Error Correction](#introduction-to-quantum-error-correction)
2. [Error Correction Codes](#error-correction-codes)
   - [Bit Flip Code](#bit-flip-code)
   - [Phase Flip Code](#phase-flip-code)
   - [Shor Code](#shor-code)
   - [Surface Code](#surface-code)
3. [VQE with Error Correction](#vqe-with-error-correction)
4. [Benchmarking and Performance Analysis](#benchmarking-and-performance-analysis)
5. [API Reference](#api-reference)
6. [Usage Examples](#usage-examples)
7. [Best Practices](#best-practices)

## Introduction to Quantum Error Correction

Quantum error correction is essential for building reliable quantum computers. Unlike classical error correction, quantum error correction must protect against both bit flips and phase errors while preserving quantum superposition and entanglement.

### Why Quantum Error Correction Matters

- **Noise Resilience**: Quantum systems are highly susceptible to environmental noise
- **Scalability**: Error correction enables large-scale quantum computation
- **Fault Tolerance**: Corrects errors without destroying quantum information

### AEONMI's Error Correction Approach

AEONMI implements multiple error correction codes with optimized performance:

- **Classical Codes**: Bit flip and phase flip codes for basic error correction
- **Advanced Codes**: Shor code and surface code for comprehensive protection
- **Integrated VQE**: Variational Quantum Eigensolver with error correction
- **Benchmarking Tools**: Comprehensive performance analysis capabilities

## Error Correction Codes

### Bit Flip Code

The bit flip code protects against X (bit flip) errors using 3 physical qubits to encode 1 logical qubit.

#### How It Works
```
Logical |0⟩ → |000⟩
Logical |1⟩ → |111⟩
```

When an X error occurs on any single qubit, the syndrome measurement reveals which qubit was affected.

#### Usage
```rust
use aeonmi::quantum::QuantumAlgorithms;

let mut alg = QuantumAlgorithms::new();

// Encode logical qubit
alg.encode_bit_flip_code("logical_q0")?;

// Simulate error (optional)
alg.simulate_quantum_error("logical_q0_0", QuantumError::BitFlip)?;

// Correct errors
alg.correct_bit_flip_error("logical_q0")?;
```

### Phase Flip Code

The phase flip code protects against Z (phase flip) errors, similar to the bit flip code but for phase errors.

#### How It Works
```
Logical |+⟩ → |+++⟩
Logical |-⟩ → |---⟩
```

Phase errors are converted to bit errors using Hadamard gates, then corrected.

#### Usage
```rust
// Encode phase flip code
alg.encode_phase_flip_code("logical_q0")?;

// Simulate phase error
alg.simulate_quantum_error("logical_q0_0", QuantumError::PhaseFlip)?;

// Correct phase errors
alg.correct_phase_flip_error("logical_q0")?;
```

### Shor Code

The Shor code combines bit flip and phase flip codes to protect against both X and Z errors simultaneously.

#### How It Works
The Shor code uses 9 physical qubits to encode 1 logical qubit:
1. First apply phase flip code (3 qubits)
2. Then apply bit flip code to each of those 3 qubits (9 total)

#### Usage
```rust
// Encode Shor code (9 qubits for 1 logical qubit)
alg.encode_shor_code("logical_q0")?;

// Simulate arbitrary single-qubit error
alg.simulate_quantum_error("logical_q0_0", QuantumError::BitPhaseFlip)?;

// Correct both bit and phase errors
alg.correct_shor_code_error("logical_q0")?;
```

### Surface Code

The surface code is a topological quantum error correction code that scales efficiently and is the leading candidate for fault-tolerant quantum computing.

#### Key Features
- **Distance d**: Error correction capability (higher d = better correction)
- **Syndrome Measurement**: Detects errors without destroying quantum information
- **Minimum Weight Matching**: Efficient error correction algorithm
- **Scalable**: Performance improves with code size

#### Surface Code Structure
```
Data qubits:     ○───○───○
                 │   │   │
Syndrome qubits: ○───○───○
                 │   │   │
Data qubits:     ○───○───○
```

#### Usage
```rust
// Encode surface code with distance 3
alg.encode_surface_code("logical_q0", 3)?;

// Simulate realistic noise
alg.apply_realistic_noise("logical_q0", 3, 0.01)?; // 1% error rate

// Correct surface code errors
alg.correct_surface_code_error("logical_q0", 3)?;
```

## VQE with Error Correction

Variational Quantum Eigensolver (VQE) combined with error correction enables noise-resilient quantum chemistry calculations.

### Basic VQE
```rust
// Define Hamiltonian (H2 molecule example)
let hamiltonian = vec![
    PauliOperator { pauli_string: "II".to_string(), coefficient: -1.0523732 },
    PauliOperator { pauli_string: "IZ".to_string(), coefficient: 0.39793742 },
    // ... more terms
];

// Define ansatz circuit
let ansatz = AnsatzCircuit {
    gates: vec![
        AnsatzGate::RY(0, 0),  // Parameterized RY gate
        AnsatzGate::CNOT(0, 1), // Entangling gate
        AnsatzGate::RY(1, 1),  // Another parameterized gate
    ],
    num_parameters: 2,
};

// Run VQE
let result = alg.vqe(&hamiltonian, &ansatz, 100, 1e-6)?;
println!("Ground state energy: {:.6}", result.ground_state_energy);
```

### VQE with Error Correction
```rust
// Run VQE with error correction (level 3 = Shor code)
let result = alg.vqe_with_error_correction(
    &hamiltonian,
    &ansatz,
    100,     // max iterations
    1e-6,    // tolerance
    3        // error correction level (0=none, 1=bit, 2=phase, 3=Shor)
)?;
```

### Error Correction Levels
- **Level 0**: No error correction (baseline)
- **Level 1**: Bit flip code (3x overhead)
- **Level 2**: Phase flip code (3x overhead)
- **Level 3**: Shor code (9x overhead)

## Benchmarking and Performance Analysis

AEONMI provides comprehensive benchmarking tools to compare error correction performance.

### Benchmarking Error Correction Levels
```rust
// Benchmark all error correction levels for a Hamiltonian
let benchmarks = alg.benchmark_error_correction_levels(&hamiltonian, &ansatz)?;

for benchmark in &benchmarks {
    println!("Level {}: Energy={:.6}, Time={:.3}s, Qubits={}",
        benchmark.error_correction_level,
        benchmark.ground_state_energy,
        benchmark.execution_time,
        benchmark.total_qubits_used
    );
}
```

### Realistic Error Correction Fidelity
```rust
// Measure fidelity under realistic noise conditions
let fidelity_result = alg.measure_realistic_error_correction_fidelity(
    "logical_q0",
    3,        // distance
    100,      // num_trials
    0.01      // error_rate
)?;

println!("Average fidelity: {:.4}", fidelity_result.average_fidelity);
println!("Error correction success rate: {:.2}%",
    fidelity_result.success_rate * 100.0);
```

### Benchmark Result Structure
```rust
pub struct VQEBenchmarkResult {
    pub error_correction_level: usize,
    pub ground_state_energy: f64,
    pub execution_time: f64,
    pub total_qubits_used: usize,
    pub convergence_achieved: bool,
    pub final_parameters: Vec<f64>,
}
```

## API Reference

### Core Methods

#### Error Correction Encoding
```rust
pub fn encode_bit_flip_code(&mut self, logical_qubit: &str) -> Result<()>
pub fn encode_phase_flip_code(&mut self, logical_qubit: &str) -> Result<()>
pub fn encode_shor_code(&mut self, logical_qubit: &str) -> Result<()>
pub fn encode_surface_code(&mut self, logical_qubit: &str, distance: usize) -> Result<()>
```

#### Error Correction Decoding
```rust
pub fn correct_bit_flip_error(&mut self, logical_qubit: &str) -> Result<()>
pub fn correct_phase_flip_error(&mut self, logical_qubit: &str) -> Result<()>
pub fn correct_shor_code_error(&mut self, logical_qubit: &str) -> Result<()>
pub fn correct_surface_code_error(&mut self, logical_qubit: &str, distance: usize) -> Result<()>
```

#### VQE Methods
```rust
pub fn vqe(&mut self, hamiltonian: &[PauliOperator], ansatz: &AnsatzCircuit,
           max_iterations: usize, tolerance: f64) -> Result<VQEResult>

pub fn vqe_with_error_correction(&mut self, hamiltonian: &[PauliOperator],
           ansatz: &AnsatzCircuit, max_iterations: usize, tolerance: f64,
           error_correction_level: usize) -> Result<VQEResult>
```

#### Benchmarking Methods
```rust
pub fn benchmark_error_correction_levels(&mut self, hamiltonian: &[PauliOperator],
           ansatz: &AnsatzCircuit) -> Result<Vec<VQEBenchmarkResult>>

pub fn measure_realistic_error_correction_fidelity(&mut self, logical_qubit: &str,
           distance: usize, num_trials: usize, error_rate: f64) -> Result<FidelityResult>
```

### Data Structures

#### PauliOperator
```rust
pub struct PauliOperator {
    pub pauli_string: String,  // e.g., "XYZ", "IIZ"
    pub coefficient: f64,      // coefficient in Hamiltonian
}
```

#### AnsatzCircuit
```rust
pub struct AnsatzCircuit {
    pub gates: Vec<AnsatzGate>,
    pub num_parameters: usize,
}

pub enum AnsatzGate {
    RY(usize, usize),    // (qubit_index, parameter_index)
    RZ(usize, usize),    // (qubit_index, parameter_index)
    CNOT(usize, usize),  // (control, target)
    H(usize),           // Hadamard on qubit
}
```

#### VQEResult
```rust
pub struct VQEResult {
    pub ground_state_energy: f64,
    pub optimal_parameters: Vec<f64>,
    pub converged: bool,
}
```

## Usage Examples

### Complete Quantum Chemistry Workflow
```rust
use aeonmi::quantum::{QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate};

// Initialize quantum algorithms
let mut alg = QuantumAlgorithms::new();

// Define H2 molecule Hamiltonian (simplified)
let hamiltonian = vec![
    PauliOperator { pauli_string: "II".to_string(), coefficient: -1.0523732 },
    PauliOperator { pauli_string: "IZ".to_string(), coefficient: 0.39793742 },
    PauliOperator { pauli_string: "ZI".to_string(), coefficient: -0.39793742 },
    PauliOperator { pauli_string: "ZZ".to_string(), coefficient: -0.01128010 },
    PauliOperator { pauli_string: "XX".to_string(), coefficient: 0.18093119 },
];

// Define UCC ansatz for H2
let ansatz = AnsatzCircuit {
    gates: vec![
        AnsatzGate::RY(0, 0),
        AnsatzGate::RY(1, 1),
        AnsatzGate::CNOT(0, 1),
        AnsatzGate::RY(0, 2),
        AnsatzGate::RY(1, 3),
    ],
    num_parameters: 4,
};

// Benchmark different error correction levels
println!("Benchmarking error correction levels...");
let benchmarks = alg.benchmark_error_correction_levels(&hamiltonian, &ansatz)?;

for bench in &benchmarks {
    println!("Level {}: Energy = {:.6} Ha, Time = {:.3}s, Qubits = {}",
        bench.error_correction_level,
        bench.ground_state_energy,
        bench.execution_time,
        bench.total_qubits_used
    );
}

// Run VQE with Shor code error correction
println!("\nRunning VQE with Shor code error correction...");
let result = alg.vqe_with_error_correction(&hamiltonian, &ansatz, 200, 1e-8, 3)?;

println!("Final results:");
println!("Ground state energy: {:.8} Hartrees", result.ground_state_energy);
println!("Converged: {}", result.converged);
println!("Optimal parameters: {:?}", result.optimal_parameters);

// Expected ground state energy for H2 is approximately -1.137 Hartrees
let expected_energy = -1.137;
let error = (result.ground_state_energy - expected_energy).abs();
println!("Energy error: {:.2e} Hartrees", error);
```

### Surface Code Error Correction Demo
```rust
// Demonstrate surface code capabilities
println!("Surface Code Error Correction Demo");

// Encode logical qubit with distance 5 surface code
alg.encode_surface_code("demo_qubit", 5)?;

// Apply realistic noise (0.5% error rate)
alg.apply_realistic_noise("demo_qubit", 5, 0.005)?;

// Measure error correction fidelity
let fidelity = alg.measure_error_correction_fidelity("demo_qubit", 5, 1000)?;
println!("Surface code fidelity: {:.4}", fidelity);

// Correct errors
alg.correct_surface_code_error("demo_qubit", 5)?;
println!("Error correction completed");
```

### Custom Error Correction Protocol
```rust
// Implement custom error correction workflow
alg.encode_surface_code("custom_logical", 3)?;

// Custom noise model (correlated errors)
for i in 0..10 {
    let qubit_name = format!("custom_logical_{}", i);
    if i % 3 == 0 {  // Every third qubit gets an error
        alg.simulate_quantum_error(&qubit_name, QuantumError::BitFlip)?;
    }
}

// Analyze syndrome before correction
// (syndrome analysis would be implemented here)

// Apply correction
alg.correct_surface_code_error("custom_logical", 3)?;
```

## Best Practices

### Error Correction Selection
- **Development/Testing**: Use bit flip or phase flip codes (simple, fast)
- **Production Chemistry**: Use Shor code or surface code (comprehensive protection)
- **Large Scale**: Surface code (scales best, most fault-tolerant)

### Performance Optimization
- **Pre-compute**: Encode logical qubits once, reuse for multiple operations
- **Batch Operations**: Process multiple logical qubits together when possible
- **Distance Selection**: Balance error correction power vs. qubit overhead

### VQE Optimization
- **Ansatz Design**: Use problem-specific ansatze (UCC, hardware-efficient)
- **Parameter Initialization**: Start near expected minimum
- **Convergence Monitoring**: Track energy vs. iteration count
- **Error Correction Level**: Match to noise characteristics

### Benchmarking Guidelines
- **Statistical Significance**: Use sufficient trial counts (≥100)
- **Realistic Noise**: Model actual hardware error rates
- **Comparative Analysis**: Always compare against no error correction baseline
- **Resource Tracking**: Monitor qubit usage and execution time

### Error Handling
```rust
// Always handle error correction failures
match alg.correct_surface_code_error("logical_q0", 3) {
    Ok(()) => println!("Error correction successful"),
    Err(e) => {
        eprintln!("Error correction failed: {}", e);
        // Implement fallback strategy (e.g., restart calculation)
    }
}
```

### Memory Management
- **Qubit Cleanup**: Reset qubits between independent calculations
- **Batch Processing**: Reuse quantum states when possible
- **Resource Limits**: Monitor total qubit usage for large calculations

This comprehensive error correction system enables reliable quantum computing applications, particularly in quantum chemistry where VQE calculations must be protected against noise to produce accurate results.