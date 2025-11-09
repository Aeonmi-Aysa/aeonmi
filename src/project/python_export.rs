use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Export a Python script that loads and executes the QASM file using Qiskit
/// 
/// The generated script:
/// - Attempts to import Qiskit
/// - Loads the QASM file from the specified path
/// - Executes the circuit on a simulator
/// - Displays measurement results and circuit diagram
/// - Provides installation guidance if Qiskit is not available
pub fn export_python_runner(
    qasm_path: &Path,
    output_path: &Path,
    project_name: &str,
) -> Result<()> {
    let qasm_path_str = qasm_path.display().to_string().replace("\\", "/");
    
    let python_code = format!(r#"#!/usr/bin/env python3
"""
Aeonmi Quantum Circuit Runner
Generated for project: {}

This script loads and executes the quantum circuit defined in the QASM file.
Requires: Qiskit (pip install qiskit qiskit-aer)
"""

import sys
from pathlib import Path

def main():
    print("="*60)
    print("Aeonmi Quantum Circuit Execution")
    print("Project: {}")
    print("="*60)
    print()
    
    # Check for Qiskit
    try:
        from qiskit import QuantumCircuit
        from qiskit.qasm2 import load
        from qiskit_aer import AerSimulator
        print("✓ Qiskit is installed and available")
    except ImportError as e:
        print("✗ Qiskit is not installed")
        print()
        print("To run this quantum circuit, you need to install Qiskit:")
        print()
        print("  pip install qiskit qiskit-aer")
        print()
        print("Or using conda:")
        print("  conda install -c conda-forge qiskit qiskit-aer")
        print()
        print("After installation, run this script again:")
        print(f"  python {{__file__}}")
        print()
        print("For more information, visit: https://qiskit.org/")
        return 1
    
    # Load QASM file
    qasm_file = Path(r"{}")
    
    if not qasm_file.exists():
        print(f"✗ Error: QASM file not found: {{qasm_file}}")
        print()
        print("Please ensure the QASM file has been generated:")
        print("  aeon project export-qasm")
        return 1
    
    print(f"✓ Loading QASM file: {{qasm_file}}")
    print()
    
    try:
        # Load the quantum circuit from QASM
        circuit = load(qasm_file)
        
        print(f"Circuit Information:")
        print(f"  Qubits: {{circuit.num_qubits}}")
        print(f"  Classical bits: {{circuit.num_clbits}}")
        print(f"  Gates/Operations: {{len(circuit.data)}}")
        print()
        
        # Display circuit operations
        print("Circuit Operations:")
        for i, (gate, qubits, clbits) in enumerate(circuit.data, 1):
            qubit_str = ", ".join(f"q[{{circuit.qubits.index(q)}}]" for q in qubits)
            if clbits:
                clbit_str = ", ".join(f"c[{{circuit.clbits.index(c)}}]" for c in clbits)
                print(f"  {{i:2}}. {{gate.name:10}} {{qubit_str:15}} -> {{clbit_str}}")
            else:
                print(f"  {{i:2}}. {{gate.name:10}} {{qubit_str}}")
        print()
        
        # Display circuit diagram
        print("Circuit Diagram:")
        print("-" * 60)
        try:
            print(circuit.draw(output='text', fold=-1))
        except Exception:
            print(circuit.draw(output='text'))
        print("-" * 60)
        print()
        
        # Create simulator
        simulator = AerSimulator()
        
        # Execute the circuit
        print("Executing circuit on Aer simulator...")
        job = simulator.run(circuit, shots=1024)
        result = job.result()
        counts = result.get_counts(circuit)
        
        print("✓ Execution complete!")
        print()
        
        # Display results
        print("Measurement Results (1024 shots):")
        print("-" * 60)
        
        # Sort by count (descending) then by state
        sorted_counts = sorted(counts.items(), key=lambda x: (-x[1], x[0]))
        
        max_count = max(counts.values())
        for state, count in sorted_counts:
            percentage = (count / 1024) * 100
            bar_length = int((count / max_count) * 40)
            bar = "█" * bar_length
            print(f"  |{{state}}⟩: {{count:4}} ({{percentage:5.1f}}%) {{bar}}")
        
        print("-" * 60)
        print()
        
        # Summary statistics
        print("Summary:")
        print(f"  Total measurements: 1024")
        print(f"  Unique states observed: {{len(counts)}}")
        print(f"  Most frequent state: {{sorted_counts[0][0]}} ({{sorted_counts[0][1]}} shots)")
        print()
        
        print("="*60)
        print("✓ Circuit execution successful!")
        print("="*60)
        
        return 0
        
    except Exception as e:
        print(f"✗ Error executing circuit: {{e}}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())
"#, project_name, project_name, qasm_path_str);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    
    // Write Python file
    fs::write(output_path, python_code)
        .with_context(|| format!("Failed to write Python file: {}", output_path.display()))?;
    
    Ok(())
}
