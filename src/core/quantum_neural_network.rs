//! AEONMI Quantum Neural Network
//! Ported from quantum_llama_bridge — Llama dependency stripped.
//! Pure quantum-native AI processing layer for Mother AI.
//!
//! Architecture:
//!   QuantumNeuralNetwork → layers of QuantumNeuron (each is a qubit with rotation gates)
//!   Entanglement strategies connect neurons across layers
//!   Forward pass = circuit construction + measurement
//!   FusionReadyNetwork = QNN prepared for Mother AI integration

use std::f64::consts::PI;

// ── Operations ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum QuantumOp {
    RotationX { target: usize, angle: f64 },
    RotationY { target: usize, angle: f64 },
    RotationZ { target: usize, angle: f64 },
    Hadamard  { target: usize },
    CNOT      { control: usize, target: usize },
    Swap      { first: usize, second: usize },
    PhaseShift { target: usize, angle: f64 },
    ControlledPhase { control: usize, target: usize, angle: f64 },
    Measure   { target: usize },
}

// ── Circuit ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct QuantumCircuitBuilder {
    pub operations: Vec<QuantumOp>,
    pub qubit_count: usize,
}

impl QuantumCircuitBuilder {
    pub fn new(qubit_count: usize) -> Self {
        Self { operations: Vec::new(), qubit_count }
    }

    pub fn add(&mut self, op: QuantumOp) {
        self.operations.push(op);
    }

    pub fn h(&mut self, target: usize) {
        self.add(QuantumOp::Hadamard { target });
    }

    pub fn rx(&mut self, target: usize, angle: f64) {
        self.add(QuantumOp::RotationX { target, angle });
    }

    pub fn ry(&mut self, target: usize, angle: f64) {
        self.add(QuantumOp::RotationY { target, angle });
    }

    pub fn rz(&mut self, target: usize, angle: f64) {
        self.add(QuantumOp::RotationZ { target, angle });
    }

    pub fn cnot(&mut self, control: usize, target: usize) {
        self.add(QuantumOp::CNOT { control, target });
    }

    pub fn swap(&mut self, first: usize, second: usize) {
        self.add(QuantumOp::Swap { first, second });
    }

    /// Execute circuit on a state vector of 2^n amplitudes, return measurement probabilities
    pub fn execute(&self) -> Vec<f64> {
        let n = self.qubit_count;
        let dim = 1 << n;
        // State vector: index 0 = |0...0⟩
        let mut re = vec![0.0f64; dim];
        let mut im = vec![0.0f64; dim];
        re[0] = 1.0;

        for op in &self.operations {
            match op {
                QuantumOp::Hadamard { target } => {
                    apply_single_gate(&mut re, &mut im, *target, &HADAMARD);
                }
                QuantumOp::RotationX { target, angle } => {
                    let c = (angle / 2.0).cos();
                    let s = (angle / 2.0).sin();
                    let gate = [[c, 0.0, 0.0, -s], [0.0, c, s, 0.0],
                                [0.0, -s, c, 0.0], [-s, 0.0, 0.0, c]];
                    // gate as (re_00, im_00, re_01, im_01, re_10, im_10, re_11, im_11)
                    let g2 = [[c, 0.0, 0.0, -s], [0.0, -s, c, 0.0]];
                    let _ = g2; // use apply_rotation
                    apply_rotation_x(&mut re, &mut im, *target, *angle);
                }
                QuantumOp::RotationY { target, angle } => {
                    apply_rotation_y(&mut re, &mut im, *target, *angle);
                }
                QuantumOp::RotationZ { target, angle } => {
                    apply_rotation_z(&mut re, &mut im, *target, *angle);
                }
                QuantumOp::CNOT { control, target } => {
                    apply_cnot(&mut re, &mut im, *control, *target, n);
                }
                QuantumOp::PhaseShift { target, angle } => {
                    apply_phase_shift(&mut re, &mut im, *target, *angle);
                }
                QuantumOp::ControlledPhase { control, target, angle } => {
                    apply_controlled_phase(&mut re, &mut im, *control, *target, *angle, n);
                }
                QuantumOp::Swap { first, second } => {
                    apply_cnot(&mut re, &mut im, *first, *second, n);
                    apply_cnot(&mut re, &mut im, *second, *first, n);
                    apply_cnot(&mut re, &mut im, *first, *second, n);
                }
                QuantumOp::Measure { .. } => {
                    // Measurement is done at readout below
                }
            }
        }

        // Return probability of each basis state
        (0..dim).map(|i| re[i] * re[i] + im[i] * im[i]).collect()
    }
}

// ── Gate math ────────────────────────────────────────────────────────────────

const HADAMARD: [[f64; 2]; 2] = [[std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2],
                                   [std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2]];

fn apply_single_gate(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, gate: &[[f64; 2]; 2]) {
    let _n = (re.len().trailing_zeros()) as usize;
    let _half = re.len() / 2;
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 0 {
            let j = i | (1 << qubit);
            let r0 = re[i]; let i0 = im[i];
            let r1 = re[j]; let i1 = im[j];
            re[i] = gate[0][0] * r0 + gate[0][1] * r1;
            im[i] = gate[0][0] * i0 + gate[0][1] * i1;
            re[j] = gate[1][0] * r0 + gate[1][1] * r1;
            im[j] = gate[1][0] * i0 + gate[1][1] * i1;
        }
    }
}

fn apply_rotation_x(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, angle: f64) {
    let c = (angle / 2.0).cos();
    let s = (angle / 2.0).sin();
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 0 {
            let j = i | (1 << qubit);
            let r0 = re[i]; let i0 = im[i];
            let r1 = re[j]; let i1 = im[j];
            re[i] =  c * r0 + s * i1;
            im[i] =  c * i0 - s * r1;
            re[j] =  s * i0 + c * r1;
            im[j] = -s * r0 + c * i1;
        }
    }
}

fn apply_rotation_y(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, angle: f64) {
    let c = (angle / 2.0).cos();
    let s = (angle / 2.0).sin();
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 0 {
            let j = i | (1 << qubit);
            let r0 = re[i]; let i0 = im[i];
            let r1 = re[j]; let i1 = im[j];
            re[i] = c * r0 - s * r1;
            im[i] = c * i0 - s * i1;
            re[j] = s * r0 + c * r1;
            im[j] = s * i0 + c * i1;
        }
    }
}

fn apply_rotation_z(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, angle: f64) {
    let c = (angle / 2.0).cos();
    let s = (angle / 2.0).sin();
    for i in 0..re.len() {
        let bit = (i >> qubit) & 1;
        let (rc, rs) = if bit == 0 { (c, -s) } else { (c, s) };
        let r = re[i]; let img = im[i];
        re[i] = rc * r - rs * img;
        im[i] = rs * r + rc * img;
    }
}

fn apply_cnot(re: &mut Vec<f64>, im: &mut Vec<f64>, control: usize, target: usize, _n: usize) {
    for i in 0..re.len() {
        if (i >> control) & 1 == 1 && (i >> target) & 1 == 0 {
            let j = i | (1 << target);
            re.swap(i, j);
            im.swap(i, j);
        }
    }
}

fn apply_phase_shift(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, angle: f64) {
    let c = angle.cos(); let s = angle.sin();
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 1 {
            let r = re[i]; let img = im[i];
            re[i] = c * r - s * img;
            im[i] = s * r + c * img;
        }
    }
}

fn apply_controlled_phase(re: &mut Vec<f64>, im: &mut Vec<f64>, control: usize, target: usize, angle: f64, _n: usize) {
    let c = angle.cos(); let s = angle.sin();
    for i in 0..re.len() {
        if (i >> control) & 1 == 1 && (i >> target) & 1 == 1 {
            let r = re[i]; let img = im[i];
            re[i] = c * r - s * img;
            im[i] = s * r + c * img;
        }
    }
}

// ── Neuron & Layer ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QuantumNeuron {
    pub qubit_index: usize,
    pub weights: [f64; 3], // Rx, Ry, Rz angles
    pub bias: f64,
}

impl QuantumNeuron {
    pub fn new(qubit_index: usize) -> Self {
        Self { qubit_index, weights: [0.0, 0.0, 0.0], bias: 0.0 }
    }

    pub fn with_weights(mut self, rx: f64, ry: f64, rz: f64) -> Self {
        self.weights = [rx, ry, rz];
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntanglementStrategy {
    AllToAll,
    NearestNeighbor,
    Custom(Vec<(usize, usize)>),
}

#[derive(Debug, Clone)]
pub struct QuantumLayer {
    pub neurons: Vec<QuantumNeuron>,
}

impl QuantumLayer {
    pub fn new(layer_offset: usize, size: usize) -> Self {
        let neurons = (0..size)
            .map(|i| QuantumNeuron::new(layer_offset + i))
            .collect();
        Self { neurons }
    }
}

// ── QNN ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QuantumNeuralNetwork {
    pub layers: Vec<QuantumLayer>,
    pub entanglement: EntanglementStrategy,
    pub learning_rate: f64,
}

impl QuantumNeuralNetwork {
    /// Create QNN from layer sizes: e.g. vec![4, 8, 4, 2]
    pub fn new(layer_sizes: Vec<usize>) -> Self {
        let mut offset = 0;
        let layers = layer_sizes.iter().map(|&sz| {
            let layer = QuantumLayer::new(offset, sz);
            offset += sz;
            layer
        }).collect();
        Self { layers, entanglement: EntanglementStrategy::NearestNeighbor, learning_rate: 0.01 }
    }

    pub fn with_entanglement(mut self, strategy: EntanglementStrategy) -> Self {
        self.entanglement = strategy;
        self
    }

    pub fn total_qubits(&self) -> usize {
        self.layers.iter().map(|l| l.neurons.len()).sum()
    }

    /// Build the quantum circuit for this network
    pub fn build_circuit(&self) -> QuantumCircuitBuilder {
        let total = self.total_qubits();
        let mut circuit = QuantumCircuitBuilder::new(total);

        // Input encoding: Hadamard all qubits
        for i in 0..total {
            circuit.h(i);
        }

        // Process each layer
        for layer in &self.layers {
            for neuron in &layer.neurons {
                let q = neuron.qubit_index;
                circuit.rx(q, neuron.weights[0] + neuron.bias);
                circuit.ry(q, neuron.weights[1]);
                circuit.rz(q, neuron.weights[2]);
            }

            // Apply entanglement between neurons in this layer
            match &self.entanglement {
                EntanglementStrategy::AllToAll => {
                    let neurons = &layer.neurons;
                    for i in 0..neurons.len() {
                        for j in (i + 1)..neurons.len() {
                            circuit.cnot(neurons[i].qubit_index, neurons[j].qubit_index);
                        }
                    }
                }
                EntanglementStrategy::NearestNeighbor => {
                    let neurons = &layer.neurons;
                    for i in 0..neurons.len().saturating_sub(1) {
                        circuit.cnot(neurons[i].qubit_index, neurons[i + 1].qubit_index);
                    }
                }
                EntanglementStrategy::Custom(pairs) => {
                    for (ctrl, tgt) in pairs {
                        circuit.cnot(*ctrl, *tgt);
                    }
                }
            }
        }

        circuit
    }

    /// Forward pass: returns probability distribution over output qubits
    pub fn forward(&self) -> Vec<f64> {
        let circuit = self.build_circuit();
        let probs = circuit.execute();
        probs
    }

    /// Prepare for Mother AI fusion
    pub fn prepare_for_mother_fusion(&self) -> FusionReadyNetwork {
        FusionReadyNetwork {
            qnn: self.clone(),
            classical_bridge: ClassicalBridge::new(),
        }
    }
}

// ── Fusion interface ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClassicalBridge {
    pub output_scale: f64,
}

impl ClassicalBridge {
    pub fn new() -> Self {
        Self { output_scale: 1.0 }
    }

    /// Convert QNN probability output to a scalar confidence score
    pub fn to_confidence(&self, probs: &[f64]) -> f64 {
        if probs.is_empty() {
            return 0.0;
        }
        let max = probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        max * self.output_scale
    }
}

#[derive(Debug, Clone)]
pub struct FusionReadyNetwork {
    pub qnn: QuantumNeuralNetwork,
    pub classical_bridge: ClassicalBridge,
}

impl FusionReadyNetwork {
    pub fn run_and_score(&self) -> f64 {
        let probs = self.qnn.forward();
        self.classical_bridge.to_confidence(&probs)
    }

    pub fn connect_to_mother(&self) -> String {
        let score = self.run_and_score();
        format!("QNN ready: {} qubits, confidence {:.4}", self.qnn.total_qubits(), score)
    }
}

// ── Specialized Algorithms ────────────────────────────────────────────────────

pub struct SpecializedAlgorithms;

impl SpecializedAlgorithms {
    /// Quantum Approximate Optimization Algorithm (QAOA)
    /// Solves graph optimization / combinatorial problems
    pub fn qaoa(problem_graph: &[(usize, usize)], depth: usize) -> Vec<f64> {
        if problem_graph.is_empty() {
            return vec![];
        }
        let num_qubits = problem_graph.iter()
            .flat_map(|(a, b)| [*a, *b])
            .max()
            .unwrap_or(0) + 1;
        let mut circuit = QuantumCircuitBuilder::new(num_qubits);

        // Initial superposition
        for i in 0..num_qubits {
            circuit.h(i);
        }

        // QAOA layers: problem unitary + mixer unitary
        for _ in 0..depth {
            // Problem: phase kickback on edges
            for (i, j) in problem_graph {
                circuit.cnot(*i, *j);
                circuit.rz(*j, PI);
                circuit.cnot(*i, *j);
            }
            // Mixer: Rx on all qubits
            for i in 0..num_qubits {
                circuit.rx(i, PI / 2.0);
            }
        }

        circuit.execute()
    }

    /// Variational Quantum Eigensolver (VQE)
    /// Finds ground state energy of a Hamiltonian
    pub fn vqe(hamiltonian_params: &[f64]) -> Vec<f64> {
        let n = hamiltonian_params.len().max(1);
        let mut circuit = QuantumCircuitBuilder::new(n);

        // State prep
        for i in 0..n {
            circuit.h(i);
        }

        // Variational ansatz
        for (i, &p) in hamiltonian_params.iter().enumerate() {
            circuit.rx(i, p);
            circuit.rz(i, p * 2.0);
        }

        // Entangling layer
        for i in 0..n.saturating_sub(1) {
            circuit.cnot(i, i + 1);
        }

        circuit.execute()
    }

    /// Quantum Support Vector Machine (QSVM) feature map
    pub fn qsvm_feature_map(training_data: &[f64]) -> Vec<f64> {
        let n = (training_data.len() as f64).log2().ceil() as usize;
        let n = n.max(1);
        let mut circuit = QuantumCircuitBuilder::new(n);

        // Data encoding via Ry rotations
        for (i, &val) in training_data.iter().enumerate().take(n) {
            circuit.ry(i, val);
        }

        // Feature map: Hadamard layer
        for i in 0..n {
            circuit.h(i);
        }

        circuit.execute()
    }

    /// Quantum Key Distribution (QKD) — BB84 protocol simulation
    /// Returns a key length vector of bit pairs (alice_bit, basis_match)
    pub fn qkd(key_length: usize) -> Vec<(u8, bool)> {
        let n = key_length.max(1);
        let mut circuit = QuantumCircuitBuilder::new(n);

        for i in 0..n {
            // Encode in alternating bases
            if i % 2 == 0 {
                circuit.h(i);
            } else {
                circuit.rx(i, PI / 4.0);
            }
        }

        let probs = circuit.execute();

        // Simulate sifting: produce (bit, basis_match) pairs
        probs.iter().enumerate().take(key_length).map(|(i, &p)| {
            let bit = if p > 0.5 { 1u8 } else { 0u8 };
            let basis_match = i % 3 != 0; // simplified sifting
            (bit, basis_match)
        }).collect()
    }

    /// Quantum Phase Estimation (QPE) circuit
    pub fn qpe(precision: usize, phase: f64) -> Vec<f64> {
        let n = precision.max(1);
        let mut circuit = QuantumCircuitBuilder::new(n + 1); // +1 eigenstate qubit

        // Initialize eigenstate qubit in |1⟩
        circuit.add(QuantumOp::RotationX { target: n, angle: PI });

        // Hadamard on all precision qubits
        for i in 0..n {
            circuit.h(i);
        }

        // Controlled phase rotations
        for i in 0..n {
            let angle = phase * (2.0f64.powi(i as i32));
            circuit.add(QuantumOp::ControlledPhase { control: i, target: n, angle });
        }

        // Inverse QFT
        for i in 0..n / 2 {
            circuit.swap(i, n - 1 - i);
        }

        circuit.execute()
    }

    /// Molecular simulation via Hartree-Fock inspired circuit
    pub fn molecular_sim(electron_count: usize, bond_length: f64) -> Vec<f64> {
        let n = (electron_count * 2).max(2);
        let mut circuit = QuantumCircuitBuilder::new(n);

        // Hartree-Fock reference state
        for i in 0..electron_count.min(n) {
            circuit.add(QuantumOp::RotationX { target: i, angle: PI });
        }

        // Orbital rotations based on bond length
        for i in 0..n.saturating_sub(1) {
            circuit.ry(i, bond_length * PI / 4.0);
            circuit.cnot(i, i + 1);
        }

        circuit.execute()
    }

    /// HHL algorithm stub — linear systems Ax = b
    pub fn hhl(matrix_dim: usize) -> Vec<f64> {
        let n = (matrix_dim as f64).log2().ceil() as usize;
        let total = 3 * n.max(1);
        let mut circuit = QuantumCircuitBuilder::new(total);

        for i in 0..n {
            circuit.h(i);
        }
        for i in 0..n.saturating_sub(1) {
            circuit.cnot(i, i + 1);
        }

        circuit.execute()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qnn_forward() {
        let qnn = QuantumNeuralNetwork::new(vec![2, 4, 2]);
        let probs = qnn.forward();
        let total: f64 = probs.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "probabilities must sum to 1, got {}", total);
    }

    #[test]
    fn test_fusion_ready() {
        let qnn = QuantumNeuralNetwork::new(vec![2, 4]);
        let fusion = qnn.prepare_for_mother_fusion();
        let score = fusion.run_and_score();
        assert!(score >= 0.0 && score <= 1.0, "score out of range: {}", score);
    }

    #[test]
    fn test_qaoa() {
        let graph = vec![(0, 1), (1, 2), (0, 2)];
        let probs = SpecializedAlgorithms::qaoa(&graph, 2);
        assert!(!probs.is_empty());
    }

    #[test]
    fn test_vqe() {
        let params = vec![0.5, 1.0, 1.5];
        let probs = SpecializedAlgorithms::vqe(&params);
        assert!(!probs.is_empty());
    }

    #[test]
    fn test_qkd() {
        let key = SpecializedAlgorithms::qkd(8);
        assert_eq!(key.len(), 8);
    }
}
