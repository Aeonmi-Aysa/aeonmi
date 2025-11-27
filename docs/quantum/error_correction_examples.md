# Quantum Error Correction Examples

Practical examples demonstrating quantum error correction usage in AEONMI.

## Basic Error Correction

### Bit Flip Code Example

```rust
use aeonmi::core::quantum_algorithms::{QuantumAlgorithms, QuantumError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Bit Flip Code Example ===");

    // Encode a logical qubit using bit flip code
    alg.encode_bit_flip_code("logical_q0")?;
    println!("✓ Encoded logical qubit with bit flip code (3 physical qubits)");

    // Simulate a bit flip error on the first physical qubit
    alg.simulate_quantum_error("logical_q0_0", QuantumError::BitFlip)?;
    println!("✓ Simulated bit flip error on physical qubit 0");

    // Correct the error
    alg.correct_bit_flip_error("logical_q0")?;
    println!("✓ Corrected bit flip error");

    // Verify the logical qubit is still intact
    // (In practice, you'd measure the logical qubit here)

    Ok(())
}
```

### Phase Flip Code Example

```rust
use aeonmi::core::quantum_algorithms::{QuantumAlgorithms, QuantumError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Phase Flip Code Example ===");

    // Encode a logical qubit using phase flip code
    alg.encode_phase_flip_code("logical_q0")?;
    println!("✓ Encoded logical qubit with phase flip code (3 physical qubits)");

    // Simulate a phase flip error
    alg.simulate_quantum_error("logical_q0_1", QuantumError::PhaseFlip)?;
    println!("✓ Simulated phase flip error on physical qubit 1");

    // Correct the error
    alg.correct_phase_flip_error("logical_q0")?;
    println!("✓ Corrected phase flip error");

    Ok(())
}
```

### Shor Code Example

```rust
use aeonmi::core::quantum_algorithms::{QuantumAlgorithms, QuantumError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Shor Code Example ===");

    // Encode with Shor code (9 physical qubits for 1 logical qubit)
    alg.encode_shor_code("logical_q0")?;
    println!("✓ Encoded logical qubit with Shor code (9 physical qubits)");

    // Simulate both bit flip and phase flip errors
    alg.simulate_quantum_error("logical_q0_0", QuantumError::BitPhaseFlip)?;
    println!("✓ Simulated combined bit and phase error");

    // Correct both types of errors
    alg.correct_shor_code_error("logical_q0")?;
    println!("✓ Corrected both bit and phase errors");

    Ok(())
}
```

## Surface Code Examples

### Basic Surface Code

```rust
use aeonmi::core::quantum_algorithms::QuantumAlgorithms;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Surface Code Example ===");

    // Encode with distance-3 surface code
    let distance = 3;
    alg.encode_surface_code("surface_q0", distance)?;
    println!("✓ Encoded with distance-{} surface code", distance);

    // Apply realistic noise (1% error rate)
    alg.apply_realistic_noise("surface_q0", distance, 0.01)?;
    println!("✓ Applied realistic noise (1% error rate)");

    // Correct errors using syndrome measurement
    alg.correct_surface_code_error("surface_q0", distance)?;
    println!("✓ Corrected errors using syndrome measurement");

    Ok(())
}
```

### Surface Code Fidelity Testing

```rust
use aeonmi::core::quantum_algorithms::QuantumAlgorithms;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Surface Code Fidelity Test ===");

    let distance = 5;
    let num_trials = 100;
    let error_rate = 0.005; // 0.5%

    // Measure error correction fidelity
    let fidelity_result = alg.measure_realistic_error_correction_fidelity(
        "test_qubit",
        distance,
        num_trials,
        error_rate
    )?;

    println!("Surface Code Distance-{} Results:", distance);
    println!("Error Rate: {:.2}%", error_rate * 100.0);
    println!("Trials: {}", num_trials);
    println!("Average Fidelity: {:.4}", fidelity_result.average_fidelity);
    println!("Success Rate: {:.2}%", fidelity_result.success_rate * 100.0);
    println!("Std Deviation: {:.4}", fidelity_result.std_deviation);

    Ok(())
}
```

## VQE with Error Correction

### Basic VQE Example

```rust
use aeonmi::core::quantum_algorithms::{
    QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate, VQEResult
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Basic VQE Example ===");

    // Simple Hamiltonian: H = Z (single qubit Z operator)
    let hamiltonian = vec![
        PauliOperator {
            pauli_string: "Z".to_string(),
            coefficient: 1.0,
        }
    ];

    // Simple ansatz: RY rotation
    let ansatz = AnsatzCircuit {
        gates: vec![
            AnsatzGate::RY(0, 0), // RY gate with parameter 0
        ],
        num_parameters: 1,
    };

    // Run VQE
    let result = alg.vqe(&hamiltonian, &ansatz, 50, 1e-6)?;

    println!("VQE Results:");
    println!("Ground state energy: {:.6}", result.ground_state_energy);
    println!("Optimal parameters: {:?}", result.optimal_parameters);
    println!("Converged: {}", result.converged);

    // For Z operator, ground state should be -1
    assert!((result.ground_state_energy - (-1.0)).abs() < 0.1);
    println!("✓ Correct ground state energy found");

    Ok(())
}
```

### VQE with Error Correction

```rust
use aeonmi::core::quantum_algorithms::{
    QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== VQE with Error Correction ===");

    // H2 molecule Hamiltonian (simplified)
    let hamiltonian = vec![
        PauliOperator { pauli_string: "II".to_string(), coefficient: -1.0523732 },
        PauliOperator { pauli_string: "IZ".to_string(), coefficient: 0.39793742 },
        PauliOperator { pauli_string: "ZI".to_string(), coefficient: -0.39793742 },
        PauliOperator { pauli_string: "ZZ".to_string(), coefficient: -0.01128010 },
        PauliOperator { pauli_string: "XX".to_string(), coefficient: 0.18093119 },
    ];

    // Two-qubit UCC ansatz
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

    // Test different error correction levels
    let levels = vec![0, 1, 3]; // No correction, bit flip, Shor code

    for &level in &levels {
        println!("\n--- Error Correction Level {} ---", level);

        let result = alg.vqe_with_error_correction(
            &hamiltonian,
            &ansatz,
            100,  // max iterations
            1e-6, // tolerance
            level // error correction level
        )?;

        println!("Ground state energy: {:.6} Ha", result.ground_state_energy);
        println!("Converged: {}", result.converged);
        println!("Parameters: {:?}", result.optimal_parameters);
    }

    Ok(())
}
```

## Benchmarking Examples

### Error Correction Benchmark

```rust
use aeonmi::core::quantum_algorithms::{
    QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Error Correction Benchmark ===");

    // Define a test Hamiltonian
    let hamiltonian = vec![
        PauliOperator { pauli_string: "Z".to_string(), coefficient: 1.0 },
    ];

    let ansatz = AnsatzCircuit {
        gates: vec![AnsatzGate::RY(0, 0)],
        num_parameters: 1,
    };

    // Benchmark all error correction levels
    let benchmarks = alg.benchmark_error_correction_levels(&hamiltonian, &ansatz)?;

    println!("Error Correction Benchmark Results:");
    println!("{:<5} {:<12} {:<10} {:<8} {:<10}",
        "Level", "Energy", "Time(s)", "Qubits", "Converged");

    for bench in &benchmarks {
        println!("{:<5} {:<12.6} {:<10.3} {:<8} {:<10}",
            bench.error_correction_level,
            bench.ground_state_energy,
            bench.execution_time,
            bench.total_qubits_used,
            bench.convergence_achieved
        );
    }

    Ok(())
}
```

### Comprehensive Performance Analysis

```rust
use aeonmi::core::quantum_algorithms::{
    QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("=== Comprehensive Error Correction Analysis ===");

    // Test Hamiltonian (simple 2-qubit system)
    let hamiltonian = vec![
        PauliOperator { pauli_string: "ZI".to_string(), coefficient: -0.5 },
        PauliOperator { pauli_string: "IZ".to_string(), coefficient: -0.5 },
        PauliOperator { pauli_string: "ZZ".to_string(), coefficient: 0.25 },
        PauliOperator { pauli_string: "XX".to_string(), coefficient: 0.25 },
    ];

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

    // Test different error rates
    let error_rates = vec![0.0, 0.001, 0.01, 0.05];
    let distance = 3;

    println!("Error Rate  | Fidelity | Success Rate | Avg Time");
    println!("------------|----------|--------------|----------");

    for &error_rate in &error_rates {
        let start = Instant::now();

        let fidelity = alg.measure_realistic_error_correction_fidelity(
            "analysis_qubit",
            distance,
            50, // fewer trials for speed
            error_rate
        )?;

        let elapsed = start.elapsed();

        println!("{:<11.3}% | {:.4} | {:.2}%       | {:.3}s",
            error_rate * 100.0,
            fidelity.average_fidelity,
            fidelity.success_rate * 100.0,
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}
```

## Advanced Usage Patterns

### Custom Error Correction Workflow

```rust
use aeonmi::core::quantum_algorithms::{QuantumAlgorithms, QuantumError};

fn custom_error_correction_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    // 1. Encode with surface code
    alg.encode_surface_code("custom_logical", 5)?;

    // 2. Apply custom noise pattern
    for i in 0..15 {  // For distance-5 surface code
        if i % 4 == 0 {  // Pattern: every 4th qubit gets error
            let qubit_name = format!("custom_logical_{}", i);
            alg.simulate_quantum_error(&qubit_name, QuantumError::BitFlip)?;
        }
    }

    // 3. Custom syndrome analysis (would implement domain-specific logic here)
    println!("Custom syndrome analysis completed");

    // 4. Apply correction
    alg.correct_surface_code_error("custom_logical", 5)?;

    // 5. Verify correction
    let fidelity = alg.measure_error_correction_fidelity("custom_logical", 5, 100)?;
    println!("Correction fidelity: {:.4}", fidelity);

    Ok(())
}
```

### Error Correction in Quantum Chemistry

```rust
use aeonmi::core::quantum_algorithms::{
    QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate
};

fn quantum_chemistry_with_error_correction() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    // LiH molecule Hamiltonian (truncated for example)
    let hamiltonian = vec![
        PauliOperator { pauli_string: "IIII".to_string(), coefficient: -4.14848983 },
        PauliOperator { pauli_string: "IIIZ".to_string(), coefficient: -0.04601495 },
        // ... many more terms would be included
    ];

    // Hardware-efficient ansatz for 4 qubits
    let ansatz = AnsatzCircuit {
        gates: vec![
            // Layer 1: single qubit rotations
            AnsatzGate::RY(0, 0), AnsatzGate::RY(1, 1),
            AnsatzGate::RY(2, 2), AnsatzGate::RY(3, 3),

            // Layer 1: entangling gates
            AnsatzGate::CNOT(0, 1), AnsatzGate::CNOT(2, 3),

            // Layer 2: single qubit rotations
            AnsatzGate::RY(0, 4), AnsatzGate::RY(1, 5),
            AnsatzGate::RY(2, 6), AnsatzGate::RY(3, 7),

            // Layer 2: entangling gates
            AnsatzGate::CNOT(1, 2), AnsatzGate::CNOT(3, 0),
        ],
        num_parameters: 8,
    };

    // Run VQE with different error correction levels
    for level in 0..=3 {
        println!("\n--- LiH VQE with Error Correction Level {} ---", level);

        let result = alg.vqe_with_error_correction(
            &hamiltonian,
            &ansatz,
            500,   // More iterations for convergence
            1e-8,  // Tight tolerance
            level
        )?;

        println!("Energy: {:.8} Ha", result.ground_state_energy);
        println!("Converged: {}", result.converged);

        // Expected ground state energy for LiH is approximately -7.882 Hartrees
        // (This is just an example - actual values would vary)
    }

    Ok(())
}
```

## Running the Examples

To run these examples:

1. Make sure you have AEONMI installed and configured
2. Create a new Rust project or add to existing one
3. Copy the example code into `src/main.rs`
4. Run with `cargo run`

For the quantum chemistry examples, you'll need to provide complete molecular Hamiltonians, which can be obtained from quantum chemistry packages like PySCF or OpenFermion.

## Performance Tips

- **Start Small**: Begin with simple error correction codes (bit flip) for testing
- **Scale Gradually**: Increase error correction level as needed
- **Monitor Resources**: Track qubit usage and execution time
- **Batch Operations**: Process multiple calculations together when possible
- **Profile Regularly**: Use the benchmarking tools to optimize performance

These examples demonstrate the full range of quantum error correction capabilities in AEONMI, from basic error correction to advanced VQE applications with noise resilience.