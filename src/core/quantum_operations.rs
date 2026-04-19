//! Quantum gate operations for the Aeonmi runtime.
//!
//! Defines the `QuantumOperation` enum used by `QuantumCircuit` to represent
//! individual gate instructions in a circuit. Used by the ARC bridge and
//! grid encoder to build angle-encoded quantum circuits.

/// A single quantum gate instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum QuantumOperation {
    /// Single-qubit Ry rotation: rotates the target qubit by `angle` radians around Y-axis.
    RotationY { target: usize, angle: f64 },
    /// Single-qubit Rx rotation.
    RotationX { target: usize, angle: f64 },
    /// Single-qubit Rz rotation.
    RotationZ { target: usize, angle: f64 },
    /// Hadamard gate — creates equal superposition.
    Hadamard { target: usize },
    /// Pauli-X (NOT) gate.
    PauliX { target: usize },
    /// Pauli-Z gate.
    PauliZ { target: usize },
    /// Controlled-NOT: flips `target` when `control` is |1⟩.
    CNOT { control: usize, target: usize },
    /// Toffoli (CCNOT): flips `target` when both `control_a` and `control_b` are |1⟩.
    Toffoli { control_a: usize, control_b: usize, target: usize },
    /// Phase gate — applies phase `angle` to |1⟩ component.
    Phase { target: usize, angle: f64 },
    /// Measurement: collapses qubit to classical bit.
    Measure { target: usize },
}

impl QuantumOperation {
    /// Returns the primary target qubit index for this operation.
    pub fn target_qubit(&self) -> usize {
        match self {
            QuantumOperation::RotationY { target, .. } => *target,
            QuantumOperation::RotationX { target, .. } => *target,
            QuantumOperation::RotationZ { target, .. } => *target,
            QuantumOperation::Hadamard { target } => *target,
            QuantumOperation::PauliX { target } => *target,
            QuantumOperation::PauliZ { target } => *target,
            QuantumOperation::CNOT { target, .. } => *target,
            QuantumOperation::Toffoli { target, .. } => *target,
            QuantumOperation::Phase { target, .. } => *target,
            QuantumOperation::Measure { target } => *target,
        }
    }

    /// Human-readable gate name for diagnostics and circuit diagrams.
    pub fn name(&self) -> &'static str {
        match self {
            QuantumOperation::RotationY { .. } => "Ry",
            QuantumOperation::RotationX { .. } => "Rx",
            QuantumOperation::RotationZ { .. } => "Rz",
            QuantumOperation::Hadamard { .. } => "H",
            QuantumOperation::PauliX { .. } => "X",
            QuantumOperation::PauliZ { .. } => "Z",
            QuantumOperation::CNOT { .. } => "CNOT",
            QuantumOperation::Toffoli { .. } => "Toffoli",
            QuantumOperation::Phase { .. } => "P",
            QuantumOperation::Measure { .. } => "M",
        }
    }
}
