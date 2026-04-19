//! Quantum circuit representation for the Aeonmi runtime.
//!
//! `QuantumCircuit` is the primary IR for gate sequences. Used by:
//! - `QuantumGridEncoder` (ARC bridge) — angle-encodes grids into circuits
//! - `GroverRuleSearch` (ARC bridge) — builds Grover oracle circuits
//! - General purpose quantum programming in .ai files via the VM

use crate::core::quantum_operations::QuantumOperation;

/// A linear sequence of quantum gate operations over a fixed-width register.
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    /// Number of qubits in this circuit's register.
    pub num_qubits: usize,
    /// Ordered list of gate operations.
    pub operations: Vec<QuantumOperation>,
    /// Optional label for diagnostics.
    pub label: Option<String>,
}

impl QuantumCircuit {
    /// Create an empty circuit with `num_qubits` qubits.
    pub fn new(num_qubits: usize) -> Self {
        QuantumCircuit {
            num_qubits,
            operations: Vec::new(),
            label: None,
        }
    }

    /// Create an empty circuit with a diagnostic label.
    pub fn with_label(num_qubits: usize, label: impl Into<String>) -> Self {
        QuantumCircuit {
            num_qubits,
            operations: Vec::new(),
            label: Some(label.into()),
        }
    }

    /// Append a gate operation to the circuit.
    pub fn add_operation(&mut self, op: QuantumOperation) {
        self.operations.push(op);
    }

    /// Number of gate operations in this circuit.
    pub fn depth(&self) -> usize {
        self.operations.len()
    }

    /// Returns true if the circuit contains no operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Apply Hadamard to all qubits — standard Grover initialisation.
    pub fn apply_hadamard_all(&mut self) {
        for i in 0..self.num_qubits {
            self.add_operation(QuantumOperation::Hadamard { target: i });
        }
    }

    /// Append a CNOT chain across all adjacent qubit pairs.
    pub fn apply_entangle_chain(&mut self) {
        for i in 0..self.num_qubits.saturating_sub(1) {
            self.add_operation(QuantumOperation::CNOT { control: i, target: i + 1 });
        }
    }

    /// Render a compact ASCII diagram for logging / dashboard display.
    pub fn ascii_diagram(&self) -> String {
        let mut lines: Vec<String> = (0..self.num_qubits)
            .map(|i| format!("q{:<2} ─", i))
            .collect();
        for op in &self.operations {
            match op {
                QuantumOperation::CNOT { control, target } => {
                    for (i, line) in lines.iter_mut().enumerate() {
                        if i == *control { line.push_str("●─"); }
                        else if i == *target { line.push_str("⊕─"); }
                        else { line.push_str("──"); }
                    }
                }
                _ => {
                    let tgt = op.target_qubit();
                    for (i, line) in lines.iter_mut().enumerate() {
                        if i == tgt { line.push_str(&format!("[{}]─", op.name())); }
                        else { line.push_str("────"); }
                    }
                }
            }
        }
        lines.join("\n")
    }
}

impl std::fmt::Display for QuantumCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "QuantumCircuit {{ qubits: {}, depth: {} }}",
            self.num_qubits,
            self.depth()
        )
    }
}
