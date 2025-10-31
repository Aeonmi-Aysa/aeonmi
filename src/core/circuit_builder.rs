/// AEONMI Quantum Circuit Builder DSL
/// Provides high-level quantum circuit construction with gate operations,
/// circuit visualization, and hardware compilation pipeline

use crate::core::ast::ASTNode;
use crate::core::hardware_integration::{QuantumCircuit as HwCircuit, QuantumGate as HwGate};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

/// Quantum Gate Types supported by AEONMI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuantumGateType {
    // Single-qubit gates
    Hadamard,      // H - Creates superposition
    PauliX,        // X - Bit flip
    PauliY,        // Y - Bit + phase flip
    PauliZ,        // Z - Phase flip
    Phase(f64),    // P(φ) - Phase gate with angle
    RotationX(f64), // RX(θ) - Rotation around X-axis
    RotationY(f64), // RY(θ) - Rotation around Y-axis
    RotationZ(f64), // RZ(θ) - Rotation around Z-axis
    S,             // S - Phase gate (π/2)
    T,             // T - π/8 gate
    SDagger,       // S† - Inverse S gate
    TDagger,       // T† - Inverse T gate
    
    // Two-qubit gates
    CNOT,          // CX - Controlled-X
    CZ,            // CZ - Controlled-Z
    CY,            // CY - Controlled-Y
    SWAP,          // SWAP - Swap two qubits
    
    // Multi-qubit gates
    Toffoli,       // CCX - Controlled-controlled-X
    Fredkin,       // CSWAP - Controlled-SWAP
    
    // Measurement
    Measure,       // Measurement operation
    
    // Custom gates
    Custom(String, Vec<f64>), // Custom gate with parameters
}

/// Qubit identifier in AEONMI circuits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct QubitId {
    pub name: String,
    pub index: Option<usize>,
}

impl QubitId {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), index: None }
    }
    
    pub fn indexed(name: &str, index: usize) -> Self {
        Self { name: name.to_string(), index: Some(index) }
    }
}

impl fmt::Display for QubitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.index {
            Some(idx) => write!(f, "{}[{}]", self.name, idx),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Quantum Gate with qubits and parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumGate {
    pub gate_type: QuantumGateType,
    pub qubits: Vec<QubitId>,
    pub classical_bits: Vec<String>, // For measurement results
    pub metadata: HashMap<String, String>, // Additional gate metadata
}

impl QuantumGate {
    pub fn new(gate_type: QuantumGateType, qubits: Vec<QubitId>) -> Self {
        Self {
            gate_type,
            qubits,
            classical_bits: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_classical_bits(mut self, bits: Vec<String>) -> Self {
        self.classical_bits = bits;
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// AEONMI Quantum Circuit with comprehensive features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuitBuilder {
    pub name: String,
    pub qubits: Vec<QubitId>,
    pub classical_registers: Vec<String>,
    pub gates: Vec<QuantumGate>,
    pub parameters: HashMap<String, f64>, // Circuit parameters
    pub metadata: HashMap<String, String>,
    pub optimization_level: u8, // 0-3, higher = more optimization
}

impl QuantumCircuitBuilder {
    /// Create a new quantum circuit
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            qubits: Vec::new(),
            classical_registers: Vec::new(),
            gates: Vec::new(),
            parameters: HashMap::new(),
            metadata: HashMap::new(),
            optimization_level: 1,
        }
    }
    
    /// Add qubits to the circuit
    pub fn add_qubits(&mut self, count: usize) -> Vec<QubitId> {
        let start_idx = self.qubits.len();
        let mut new_qubits = Vec::new();
        
        for i in 0..count {
            let qubit = QubitId::indexed("q", start_idx + i);
            self.qubits.push(qubit.clone());
            new_qubits.push(qubit);
        }
        
        new_qubits
    }
    
    /// Add named qubit to the circuit
    pub fn add_named_qubit(&mut self, name: &str) -> QubitId {
        let qubit = QubitId::new(name);
        self.qubits.push(qubit.clone());
        qubit
    }
    
    /// Add classical register for measurements
    pub fn add_classical_register(&mut self, name: &str, size: usize) {
        for i in 0..size {
            self.classical_registers.push(format!("{}[{}]", name, i));
        }
    }
    
    /// Add a gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) -> &mut Self {
        self.gates.push(gate);
        self
    }
    
    // === Single-Qubit Gate Operations ===
    
    /// Apply Hadamard gate (creates superposition)
    pub fn h(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::Hadamard,
            vec![qubit.clone()]
        ))
    }
    
    /// Apply Pauli-X gate (bit flip)
    pub fn x(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::PauliX,
            vec![qubit.clone()]
        ))
    }
    
    /// Apply Pauli-Y gate 
    pub fn y(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::PauliY,
            vec![qubit.clone()]
        ))
    }
    
    /// Apply Pauli-Z gate (phase flip)
    pub fn z(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::PauliZ,
            vec![qubit.clone()]
        ))
    }
    
    /// Apply phase gate with angle
    pub fn phase(&mut self, qubit: &QubitId, angle: f64) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::Phase(angle),
            vec![qubit.clone()]
        ))
    }
    
    /// Apply rotation around X-axis
    pub fn rx(&mut self, qubit: &QubitId, angle: f64) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::RotationX(angle),
            vec![qubit.clone()]
        ))
    }
    
    /// Apply rotation around Y-axis
    pub fn ry(&mut self, qubit: &QubitId, angle: f64) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::RotationY(angle),
            vec![qubit.clone()]
        ))
    }
    
    /// Apply rotation around Z-axis
    pub fn rz(&mut self, qubit: &QubitId, angle: f64) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::RotationZ(angle),
            vec![qubit.clone()]
        ))
    }
    
    /// Apply S gate (π/2 phase)
    pub fn s(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::S,
            vec![qubit.clone()]
        ))
    }
    
    /// Apply T gate (π/8 phase)
    pub fn t(&mut self, qubit: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::T,
            vec![qubit.clone()]
        ))
    }
    
    // === Two-Qubit Gate Operations ===
    
    /// Apply CNOT gate (controlled-X)
    pub fn cnot(&mut self, control: &QubitId, target: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::CNOT,
            vec![control.clone(), target.clone()]
        ))
    }
    
    /// Apply controlled-Z gate
    pub fn cz(&mut self, control: &QubitId, target: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::CZ,
            vec![control.clone(), target.clone()]
        ))
    }
    
    /// Apply SWAP gate
    pub fn swap(&mut self, qubit1: &QubitId, qubit2: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::SWAP,
            vec![qubit1.clone(), qubit2.clone()]
        ))
    }
    
    // === Multi-Qubit Gate Operations ===
    
    /// Apply Toffoli gate (CCX)
    pub fn toffoli(&mut self, control1: &QubitId, control2: &QubitId, target: &QubitId) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::Toffoli,
            vec![control1.clone(), control2.clone(), target.clone()]
        ))
    }
    
    // === Measurement Operations ===
    
    /// Measure a qubit
    pub fn measure(&mut self, qubit: &QubitId, classical_bit: &str) -> &mut Self {
        self.add_gate(QuantumGate::new(
            QuantumGateType::Measure,
            vec![qubit.clone()]
        ).with_classical_bits(vec![classical_bit.to_string()]))
    }
    
    /// Measure all qubits
    pub fn measure_all(&mut self) -> &mut Self {
        let qubits = self.qubits.clone();
        for (i, qubit) in qubits.iter().enumerate() {
            let classical_bit = format!("c[{}]", i);
            self.measure(qubit, &classical_bit);
        }
        self
    }
    
    // === Circuit Composition and Utilities ===
    
    /// Set circuit parameter
    pub fn set_parameter(&mut self, name: &str, value: f64) -> &mut Self {
        self.parameters.insert(name.to_string(), value);
        self
    }
    
    /// Set optimization level (0-3)
    pub fn set_optimization_level(&mut self, level: u8) -> &mut Self {
        self.optimization_level = level.min(3);
        self
    }
    
    /// Get circuit depth (number of gate layers)
    pub fn depth(&self) -> usize {
        // Simplified depth calculation - could be enhanced with dependency analysis
        self.gates.len()
    }
    
    /// Get total gate count
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
    
    /// Get qubit count
    pub fn qubit_count(&self) -> usize {
        self.qubits.len()
    }
    
    /// Convert to hardware circuit format
    pub fn to_hardware_circuit(&self) -> HwCircuit {
        let mut hw_gates = Vec::new();
        
        for gate in &self.gates {
            let hw_gate = self.convert_gate_to_hardware(gate);
            hw_gates.push(hw_gate);
        }
        
        HwCircuit {
            gates: hw_gates,
            qubits: self.qubits.len(),
            measurements: Vec::new(), // Will be populated during execution
        }
    }
    
    /// Convert AEONMI gate to hardware gate format
    fn convert_gate_to_hardware(&self, gate: &QuantumGate) -> HwGate {
        let gate_name = self.gate_type_to_string(&gate.gate_type);
        let qubit_indices: Vec<usize> = gate.qubits.iter()
            .filter_map(|q| self.get_qubit_index(q))
            .collect();
        let parameters = self.extract_gate_parameters(&gate.gate_type);
        
        HwGate {
            gate_type: gate_name,
            qubits: qubit_indices,
            parameters,
        }
    }
    
    /// Convert gate type to string representation
    fn gate_type_to_string(&self, gate_type: &QuantumGateType) -> String {
        match gate_type {
            QuantumGateType::Hadamard => "h".to_string(),
            QuantumGateType::PauliX => "x".to_string(),
            QuantumGateType::PauliY => "y".to_string(),
            QuantumGateType::PauliZ => "z".to_string(),
            QuantumGateType::Phase(_) => "p".to_string(),
            QuantumGateType::RotationX(_) => "rx".to_string(),
            QuantumGateType::RotationY(_) => "ry".to_string(),
            QuantumGateType::RotationZ(_) => "rz".to_string(),
            QuantumGateType::S => "s".to_string(),
            QuantumGateType::T => "t".to_string(),
            QuantumGateType::SDagger => "sdg".to_string(),
            QuantumGateType::TDagger => "tdg".to_string(),
            QuantumGateType::CNOT => "cx".to_string(),
            QuantumGateType::CZ => "cz".to_string(),
            QuantumGateType::CY => "cy".to_string(),
            QuantumGateType::SWAP => "swap".to_string(),
            QuantumGateType::Toffoli => "ccx".to_string(),
            QuantumGateType::Fredkin => "cswap".to_string(),
            QuantumGateType::Measure => "measure".to_string(),
            QuantumGateType::Custom(name, _) => name.clone(),
        }
    }
    
    /// Extract parameters from gate type
    fn extract_gate_parameters(&self, gate_type: &QuantumGateType) -> Vec<f64> {
        match gate_type {
            QuantumGateType::Phase(angle) => vec![*angle],
            QuantumGateType::RotationX(angle) => vec![*angle],
            QuantumGateType::RotationY(angle) => vec![*angle],
            QuantumGateType::RotationZ(angle) => vec![*angle],
            QuantumGateType::Custom(_, params) => params.clone(),
            _ => Vec::new(),
        }
    }
    
    /// Get qubit index by QubitId
    fn get_qubit_index(&self, qubit_id: &QubitId) -> Option<usize> {
        self.qubits.iter().position(|q| q == qubit_id)
    }
}

// === Circuit Visualization ===

impl fmt::Display for QuantumCircuitBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Quantum Circuit: {}", self.name)?;
        writeln!(f, "Qubits: {}", self.qubit_count())?;
        writeln!(f, "Gates: {}", self.gate_count())?;
        writeln!(f, "Depth: {}", self.depth())?;
        writeln!(f, "Optimization Level: {}", self.optimization_level)?;
        
        if !self.parameters.is_empty() {
            writeln!(f, "Parameters:")?;
            for (name, value) in &self.parameters {
                writeln!(f, "  {} = {}", name, value)?;
            }
        }
        
        writeln!(f, "\nGate Sequence:")?;
        for (i, gate) in self.gates.iter().enumerate() {
            let qubits: Vec<String> = gate.qubits.iter().map(|q| q.to_string()).collect();
            writeln!(f, "  {}: {:?} on [{}]", i, gate.gate_type, qubits.join(", "))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circuit_builder_creation() {
        let circuit = QuantumCircuitBuilder::new("test_circuit");
        assert_eq!(circuit.name, "test_circuit");
        assert_eq!(circuit.qubit_count(), 0);
        assert_eq!(circuit.gate_count(), 0);
    }
    
    #[test]
    fn test_bell_state_circuit() {
        let mut circuit = QuantumCircuitBuilder::new("bell_state");
        let qubits = circuit.add_qubits(2);
        
        circuit
            .h(&qubits[0])
            .cnot(&qubits[0], &qubits[1])
            .measure_all();
        
        assert_eq!(circuit.qubit_count(), 2);
        assert_eq!(circuit.gate_count(), 4); // H + CNOT + 2 measurements
    }
    
    #[test]
    fn test_quantum_fourier_transform() {
        let mut circuit = QuantumCircuitBuilder::new("qft_3_qubit");
        let qubits = circuit.add_qubits(3);
        
        // 3-qubit QFT circuit
        circuit
            .h(&qubits[0])
            .phase(&qubits[0], std::f64::consts::PI / 2.0)
            .h(&qubits[1])
            .phase(&qubits[1], std::f64::consts::PI / 4.0)
            .phase(&qubits[0], std::f64::consts::PI / 4.0)
            .h(&qubits[2]);
        
        assert_eq!(circuit.qubit_count(), 3);
        assert!(circuit.gate_count() > 0);
    }
}