# Error Correction Benchmarks

Comprehensive benchmarking suite for comparing quantum error correction performance across all implemented codes and algorithms.

## Overview

This benchmarking framework provides detailed performance analysis and comparison of:
- All error correction codes (Bit Flip, Phase Flip, Shor, Surface Code)
- Multiple surface code distances (d=3, 5, 7, 9)
- Various noise models and error rates
- VQE performance with different error correction levels
- Resource usage and computational efficiency

## Benchmark Categories

### 1. Error Correction Fidelity Benchmarks

#### Surface Code Distance Scaling
```rust
use aeonmi::core::quantum_algorithms::QuantumAlgorithms;

fn benchmark_surface_code_distances() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();
    let distances = vec![3, 5, 7, 9];
    let error_rates = vec![0.001, 0.005, 0.01, 0.05];
    let trials = 1000;

    println!("Surface Code Distance Scaling Benchmark");
    println!("Distance | Error Rate | Fidelity | Success Rate | Time (ms)");
    println!("---------|------------|----------|--------------|-----------");

    for &distance in &distances {
        for &error_rate in &error_rates {
            let start = std::time::Instant::now();

            let fidelity = alg.measure_realistic_error_correction_fidelity(
                "bench_qubit", distance, trials, error_rate
            )?;

            let elapsed = start.elapsed();

            println!("{:8} | {:.3}      | {:.4}   | {:.1}%         | {:.0}",
                distance, error_rate, fidelity.average_fidelity,
                fidelity.success_rate * 100.0, elapsed.as_millis());
        }
    }

    Ok(())
}
```

#### Error Correction Code Comparison
```rust
fn benchmark_error_correction_codes() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();
    let error_rates = vec![0.001, 0.01, 0.05];
    let trials = 500;

    println!("Error Correction Code Comparison");
    println!("Code Type | Error Rate | Fidelity | Qubit Overhead | Time (ms)");
    println!("----------|------------|----------|----------------|-----------");

    // Test different error correction levels
    for level in 0..=4 {  // 0=None, 1=BitFlip, 2=PhaseFlip, 3=Shor, 4=Surface(d=3)
        for &error_rate in &error_rates {
            let start = std::time::Instant::now();

            let fidelity = alg.measure_error_correction_fidelity_level(
                "bench_qubit", level, trials, error_rate
            )?;

            let elapsed = start.elapsed();

            let code_name = match level {
                0 => "None",
                1 => "Bit Flip",
                2 => "Phase Flip",
                3 => "Shor",
                4 => "Surface-3",
                _ => "Unknown"
            };

            let overhead = match level {
                0 => 0,
                1 => 2, // 3x total, 2x overhead
                2 => 2,
                3 => 8, // 9x total, 8x overhead
                4 => 16, // 17x total, 16x overhead
                _ => 0
            };

            println!("{:9} | {:.3}      | {:.4}   | {:2}             | {:.0}",
                code_name, error_rate, fidelity.average_fidelity, overhead, elapsed.as_millis());
        }
    }

    Ok(())
}
```

### 2. VQE Performance Benchmarks

#### VQE with Error Correction Scaling
```rust
use aeonmi::core::quantum_algorithms::{QuantumAlgorithms, PauliOperator, AnsatzCircuit, AnsatzGate};

fn benchmark_vqe_error_correction() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

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
            AnsatzGate::RY(0, 0), AnsatzGate::RY(1, 1),
            AnsatzGate::CNOT(0, 1),
            AnsatzGate::RY(0, 2), AnsatzGate::RY(1, 3),
        ],
        num_parameters: 4,
    };

    println!("VQE Error Correction Performance Benchmark");
    println!("Level | Energy | Converged | Iterations | Time (s) | Qubits");
    println!("------|--------|-----------|------------|----------|--------");

    for level in 0..=4 {
        let start = std::time::Instant::now();

        let result = alg.vqe_with_error_correction(
            &hamiltonian, &ansatz, 100, 1e-6, level
        )?;

        let elapsed = start.elapsed();

        let qubit_count = match level {
            0 => 2,
            1 => 6,   // Bit flip: 3x per qubit
            2 => 6,   // Phase flip: 3x per qubit
            3 => 18,  // Shor: 9x per qubit
            4 => 34,  // Surface-3: 17x per qubit
            _ => 2
        };

        println!("{:5} | {:.6} | {:9} | {:10} | {:.2}     | {:6}",
            level, result.ground_state_energy, result.converged,
            result.iterations, elapsed.as_secs_f64(), qubit_count);
    }

    Ok(())
}
```

#### Resource Usage Analysis
```rust
fn benchmark_resource_usage() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("Resource Usage Benchmark");
    println!("Operation | Qubits | Memory (MB) | Time (ms)");
    println!("----------|--------|-------------|-----------");

    let operations = vec![
        ("Bit Flip Encode", 3),
        ("Phase Flip Encode", 3),
        ("Shor Encode", 9),
        ("Surface-3 Encode", 17),
        ("Surface-5 Encode", 49),
        ("Surface-7 Encode", 97),
    ];

    for (name, expected_qubits) in operations {
        let start = std::time::Instant::now();

        // Measure memory before operation
        let memory_before = alg.get_memory_usage();

        match name {
            "Bit Flip Encode" => {
                alg.encode_bit_flip_code("resource_test")?;
            }
            "Phase Flip Encode" => {
                alg.encode_phase_flip_code("resource_test")?;
            }
            "Shor Encode" => {
                alg.encode_shor_code("resource_test")?;
            }
            "Surface-3 Encode" => {
                alg.encode_surface_code("resource_test", 3)?;
            }
            "Surface-5 Encode" => {
                alg.encode_surface_code("resource_test", 5)?;
            }
            "Surface-7 Encode" => {
                alg.encode_surface_code("resource_test", 7)?;
            }
            _ => {}
        }

        let elapsed = start.elapsed();
        let memory_after = alg.get_memory_usage();
        let memory_used = memory_after - memory_before;

        println!("{:17} | {:6} | {:.1}         | {:.0}",
            name, expected_qubits, memory_used, elapsed.as_millis());
    }

    Ok(())
}
```

### 3. Noise Model Validation Benchmarks

#### Realistic Noise Model Testing
```rust
fn benchmark_noise_models() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();
    let distance = 5;
    let trials = 500;

    let noise_models = vec![
        ("Ideal", 0.0, 0.0, 0.0, 0.0),
        ("Bit Flip Only", 0.01, 0.0, 0.0, 0.0),
        ("Phase Flip Only", 0.0, 0.01, 0.0, 0.0),
        ("Dephasing", 0.0, 0.0, 0.01, 0.0),
        ("Amplitude Damping", 0.0, 0.0, 0.0, 0.01),
        ("Mixed Noise", 0.005, 0.005, 0.005, 0.005),
        ("High Noise", 0.05, 0.05, 0.02, 0.02),
    ];

    println!("Noise Model Validation Benchmark (Distance-{})", distance);
    println!("Noise Model     | Fidelity | Success Rate | Time (ms)");
    println!("----------------|----------|--------------|-----------");

    for (name, bit_flip, phase_flip, dephasing, amplitude_damping) in noise_models {
        let start = std::time::Instant::now();

        let fidelity = alg.measure_custom_noise_fidelity(
            "noise_test", distance, trials,
            bit_flip, phase_flip, dephasing, amplitude_damping
        )?;

        let elapsed = start.elapsed();

        println!("{:15} | {:.4}   | {:.1}%         | {:.0}",
            name, fidelity.average_fidelity,
            fidelity.success_rate * 100.0, elapsed.as_millis());
    }

    Ok(())
}
```

### 4. Fault-Tolerant Gate Benchmarks

#### Logical Gate Performance
```rust
fn benchmark_logical_gates() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();
    let distances = vec![3, 5, 7];
    let trials = 100;

    println!("Logical Gate Performance Benchmark");
    println!("Gate | Distance | Fidelity | Time (μs)");
    println!("-----|----------|----------|----------");

    let gates = vec!["X", "Z", "H", "S", "T"];

    for gate in gates {
        for &distance in &distances {
            let start = std::time::Instant::now();

            let fidelity = alg.benchmark_logical_gate(gate, distance, trials)?;

            let elapsed = start.elapsed();

            println!("{:4} | {:8} | {:.4}   | {:.0}",
                gate, distance, fidelity.average_fidelity,
                elapsed.as_micros() / trials as u128);
        }
    }

    Ok(())
}
```

#### Logical Circuit Benchmarks
```rust
fn benchmark_logical_circuits() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();

    println!("Logical Circuit Benchmark");
    println!("Circuit | Distance | Fidelity | Depth | Time (ms)");
    println!("--------|----------|----------|-------|-----------");

    let circuits = vec![
        ("Identity", vec![]),
        ("X Gate", vec!["X"]),
        ("H Gate", vec!["H"]),
        ("Bell Prep", vec!["H", "CNOT"]),
        ("QFT-2", vec!["H", "CU1", "H"]),
        ("Random-3", vec!["X", "H", "S", "CNOT", "T"]),
    ];

    for (name, gate_sequence) in circuits {
        for distance in [3, 5] {
            let start = std::time::Instant::now();

            let fidelity = alg.benchmark_logical_circuit(
                name, &gate_sequence, distance, 50
            )?;

            let elapsed = start.elapsed();

            println!("{:10} | {:8} | {:.4}   | {:5} | {:.0}",
                name, distance, fidelity.average_fidelity,
                gate_sequence.len(), elapsed.as_millis());
        }
    }

    Ok(())
}
```

### 5. Scalability and Performance Benchmarks

#### Large-Scale Surface Code Performance
```rust
fn benchmark_large_scale_surface_codes() -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = QuantumAlgorithms::new();
    let distances = vec![7, 9];
    let error_rates = vec![0.001, 0.01];

    println!("Large-Scale Surface Code Benchmark");
    println!("Distance | Error Rate | Qubits | Fidelity | Time (s) | Memory (GB)");
    println!("---------|------------|--------|----------|----------|-------------");

    for &distance in &distances {
        for &error_rate in &error_rates {
            let start = std::time::Instant::now();
            let memory_before = alg.get_memory_usage();

            let fidelity = alg.measure_realistic_error_correction_fidelity(
                "large_test", distance, 100, error_rate
            )?;

            let elapsed = start.elapsed();
            let memory_after = alg.get_memory_usage();
            let memory_used = (memory_after - memory_before) / 1024.0; // Convert to GB

            let total_qubits = distance * distance + 2 * (distance - 1) * (distance - 1);

            println!("{:8} | {:.3}      | {:6} | {:.4}   | {:.2}     | {:.2}",
                distance, error_rate, total_qubits, fidelity.average_fidelity,
                elapsed.as_secs_f64(), memory_used);
        }
    }

    Ok(())
}
```

#### Parallel Benchmarking
```rust
use std::thread;
use std::sync::mpsc;

fn benchmark_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
    let num_threads = num_cpus::get();
    let (tx, rx) = mpsc::channel();

    println!("Parallel Processing Benchmark ({} threads)", num_threads);
    println!("Thread | Distance | Trials | Time (s) | Throughput");
    println!("-------|----------|--------|----------|------------");

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            let mut alg = QuantumAlgorithms::new();
            let distance = 3 + (thread_id % 3); // Vary distance per thread
            let trials = 1000;

            let start = std::time::Instant::now();

            let fidelity = alg.measure_realistic_error_correction_fidelity(
                &format!("thread_{}", thread_id), distance, trials, 0.01
            ).unwrap();

            let elapsed = start.elapsed();

            tx_clone.send((thread_id, distance, trials, elapsed, fidelity)).unwrap();
        });
        handles.push(handle);
    }

    // Drop the original sender so receiver knows when to stop
    drop(tx);

    // Collect results
    for (thread_id, distance, trials, elapsed, fidelity) in rx {
        let throughput = trials as f64 / elapsed.as_secs_f64();
        println!("{:6} | {:8} | {:6} | {:.2}     | {:.0} trials/s",
            thread_id, distance, trials, elapsed.as_secs_f64(), throughput);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
```

## Running the Benchmarks

### Basic Benchmark Suite
```bash
# Run all benchmarks
cargo run --bin benchmark_suite

# Run specific benchmark categories
cargo run --bin benchmark_suite -- --category fidelity
cargo run --bin benchmark_suite -- --category vqe
cargo run --bin benchmark_suite -- --category resources
cargo run --bin benchmark_suite -- --category noise
cargo run --bin benchmark_suite -- --category gates
```

### Advanced Benchmark Options
```bash
# Custom parameters
cargo run --bin benchmark_suite -- \
  --distances 3,5,7 \
  --error-rates 0.001,0.01,0.05 \
  --trials 1000 \
  --output results.json \
  --parallel

# Performance profiling
cargo run --bin benchmark_suite -- --profile --memory-tracking
```

### Benchmark Results Analysis
```bash
# Generate performance reports
cargo run --bin benchmark_analyzer -- --input results.json --format html

# Compare benchmark runs
cargo run --bin benchmark_analyzer -- \
  --compare run1.json run2.json \
  --metrics fidelity,speed,memory
```

## Benchmark Result Interpretation

### Key Performance Metrics

#### Fidelity Thresholds
- **Excellent**: >99.9% - Suitable for production quantum chemistry
- **Good**: 99.0-99.9% - Acceptable for development and testing
- **Poor**: <99.0% - Requires investigation and optimization

#### Performance Targets
- **Encoding Time**: <1ms for distance ≤5, <10ms for distance ≤9
- **Correction Time**: <5ms for distance ≤5, <50ms for distance ≤9
- **Memory Usage**: <100MB for distance ≤5, <1GB for distance ≤9

#### Scaling Expectations
- **Fidelity**: Improves exponentially with distance (d^2 scaling)
- **Time**: Scales as O(d^2) for encoding, O(d^4) for correction
- **Memory**: Scales as O(d^2) for basic operations

## Benchmark Maintenance

### Adding New Benchmarks
1. Implement benchmark function in `src/benchmarks/`
2. Add to benchmark registry in `src/benchmarks/mod.rs`
3. Update command-line interface in `src/bin/benchmark_suite.rs`
4. Add documentation to this file

### Performance Regression Detection
- Automated nightly benchmark runs
- Statistical analysis of performance changes
- Alert system for significant regressions
- Historical performance tracking

This comprehensive benchmarking suite ensures AEONMI's quantum error correction maintains high performance and reliability across all supported configurations and use cases.