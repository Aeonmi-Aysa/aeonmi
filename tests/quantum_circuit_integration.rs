use aeonmi_project::core::circuit_builder::QuantumCircuitBuilder;
use aeonmi_project::core::circuit_compiler::{CompilationOptions, CompilationTarget, QuantumCircuitCompiler};
use aeonmi_project::core::circuit_visualization::{AsciiStyle, CircuitVisualizer, VisualizationConfig};

#[cfg(test)]
mod circuit_integration_tests {
	use super::*;

	fn visualizer_with_style(style: AsciiStyle) -> CircuitVisualizer {
		CircuitVisualizer::new(VisualizationConfig {
			ascii_style: style,
			..Default::default()
		})
	}

	#[test]
	fn bell_state_circuit_builds() {
		let mut circuit = QuantumCircuitBuilder::new("Bell State");
		let qubits = circuit.add_qubits(2);

		circuit
			.h(&qubits[0])
			.cnot(&qubits[0], &qubits[1])
			.measure_all();

		assert_eq!(circuit.qubit_count(), 2);
		assert_eq!(circuit.gate_count(), 4);

		let ascii = visualizer_with_style(AsciiStyle::Simple).to_ascii(&circuit);
		assert!(ascii.contains("Circuit: Bell State"));
		assert!(ascii.contains("Qubits: 2"));
	}

	#[test]
	fn parameterized_rotation_circuit() {
		let mut circuit = QuantumCircuitBuilder::new("Parameterized");
		let qubits = circuit.add_qubits(1);

		circuit
			.set_parameter("theta", 0.5)
			.set_parameter("phi", 1.25)
			.rx(&qubits[0], 0.5)
			.rz(&qubits[0], 1.25);

		assert_eq!(circuit.parameters.len(), 2);
		assert!(circuit.parameters.contains_key("theta"));
		assert!(circuit.parameters.contains_key("phi"));
		assert_eq!(circuit.gate_count(), 2);

		let ascii = visualizer_with_style(AsciiStyle::Compact).to_ascii(&circuit);
		assert!(ascii.contains("Parameters"));
		assert!(ascii.contains("theta"));
	}

	#[test]
	fn visualization_outputs_are_generated() {
		let mut circuit = QuantumCircuitBuilder::new("Visualization");
		let qubits = circuit.add_qubits(2);
		circuit.h(&qubits[0]).cnot(&qubits[0], &qubits[1]);

		let latex = CircuitVisualizer::default().to_latex(&circuit);
		assert!(latex.contains("\\begin{tikzpicture}"));
		assert!(latex.contains("\\gate"));

		let json = CircuitVisualizer::default().to_json(&circuit);
		assert_eq!(json["qubits"].as_u64(), Some(2));
		assert!(json["gate_sequence"]
			.as_array()
			.map(|entries| !entries.is_empty())
			.unwrap_or(false));
	}

	#[test]
	fn circuit_compiles_to_qasm2() {
		let mut circuit = QuantumCircuitBuilder::new("QASM2 circuit");
		let qubits = circuit.add_qubits(2);

		circuit
			.h(&qubits[0])
			.cnot(&qubits[0], &qubits[1])
			.measure_all();

		let compiler = QuantumCircuitCompiler::new(CompilationOptions {
			target: CompilationTarget::OpenQASM2,
			include_measurements: true,
			..Default::default()
		});

		let qasm = compiler.compile(&circuit).expect("compile to OpenQASM 2.0");
		assert!(qasm.contains("OPENQASM 2.0"));
		assert!(qasm.contains("cx q[0], q[1];"));
		assert!(qasm.contains("measure q[0] -> c[0];"));
	}

	#[test]
	fn circuit_compiles_to_qiskit() {
		let mut circuit = QuantumCircuitBuilder::new("Qiskit circuit");
		let qubits = circuit.add_qubits(2);

		circuit.h(&qubits[0]).cnot(&qubits[0], &qubits[1]);

		let compiler = QuantumCircuitCompiler::new(CompilationOptions {
			target: CompilationTarget::Qiskit,
			include_measurements: false,
			..Default::default()
		});

		let python = compiler.compile(&circuit).expect("compile to Qiskit");
		assert!(python.contains("from qiskit import QuantumCircuit"));
		assert!(python.contains("circuit.cx(qreg[0], qreg[1])"));
	}

	#[test]
	fn circuit_compiles_to_javascript() {
		let mut circuit = QuantumCircuitBuilder::new("JavaScript circuit");
		let qubits = circuit.add_qubits(1);

		circuit
			.h(&qubits[0])
			.x(&qubits[0])
			.measure(&qubits[0], "c[0]");

		let compiler = QuantumCircuitCompiler::new(CompilationOptions {
			target: CompilationTarget::JavaScript,
			include_measurements: true,
			..Default::default()
		});

		let js = compiler.compile(&circuit).expect("compile to JavaScript");
		assert!(js.contains("QuantumSimulator"));
		assert!(js.contains("circuit.h(0);"));
		assert!(js.contains("circuit.measure(0);"));
	}
}


