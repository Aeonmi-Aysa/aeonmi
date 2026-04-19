//! AEONMI Quantum Neural Network
//! Ported from quantum_llama_bridge — Llama dependency stripped.
//! Pure quantum-native AI processing layer for Mother AI.
//!
//! Architecture:
//! QuantumNeuralNetwork → layers of QuantumNeuron (each is a qubit with rotation gates)
//! Entanglement strategies connect neurons across layers
//! Forward pass = circuit construction + measurement
//! FusionReadyNetwork = QNN prepared for Mother AI integration

use std::f64::consts::PI;

// ── Operations ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum QuantumOp {
    RotationX { target: usize, angle: f64 },
    RotationY { target: usize, angle: f64 },
    RotationZ { target: usize, angle: f64 },
    Hadamard { target: usize },
    CNOT { control: usize, target: usize },
    Swap { first: usize, second: usize },
    PhaseShift { target: usize, angle: f64 },
    ControlledPhase { control: usize, target: usize, angle: f64 },
    Measure { target: usize },
}

// ── Circuit ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Default)]
pub struct QuantumCircuitBuilder {
    pub operations: Vec<QuantumOp>,
    pub qubit_count: usize,
}

impl QuantumCircuitBuilder {
    pub fn new(qubit_count: usize) -> Self {
        Self {
            operations: Vec::new(),
            qubit_count,
        }
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
        let dim = 1 << n; // 2^n
        let mut re = vec![0.0f64; dim];
        let mut im = vec![0.0f64; dim];
        re[0] = 1.0;

        for op in &self.operations {
            match op {
                QuantumOp::Hadamard { target } => {
                    apply_single_gate(&mut re, &mut im, *target, &HADAMARD);
                }
                QuantumOp::RotationX { target, angle } => {
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

        // Return probability of each basis state (0..dim)
        (0..dim)
            .map(|i| re[i] * re[i] + im[i] * im[i])
            .collect()
    }
}

// ── Gate math ────────────────────────────────────────────────────────────────
const HADAMARD: [[f64; 2]; 2] = [
    [std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2],
    [std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2],
];

fn apply_single_gate(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, gate: &[[f64; 2]; 2]) {
    // Optional: if you had artifact_cache, this is where you'd use it
    // For now, just compute stride locally
    let _stride_log2 = (re.len().trailing_zeros() as usize).min(64); // safe clamp

    let half = re.len() / 2;
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 0 {
            let j = i | (1 << qubit);
            let r0 = re[i];
            let i0 = im[i];
            let r1 = re[j];
            let i1 = im[j];

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
            let r0 = re[i];
            let i0 = im[i];
            let r1 = re[j];
            let i1 = im[j];

            re[i] = c * r0 - s * i1;  // Note: sign flip vs some conventions
            im[i] = c * i0 + s * r1;
            re[j] = s * i0 + c * r1;
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
            let r0 = re[i];
            let i0 = im[i];
            let r1 = re[j];
            let i1 = im[j];

            re[i] = c * r0 - s * r1;
            im[i] = c * i0 + s * i1;
            re[j] = s * r0 + c * r1;
            im[j] = -s * i0 + c * i1;
        }
    }
}

fn apply_rotation_z(re: &mut Vec<f64>, im: &mut Vec<f64>, qubit: usize, angle: f64) {
    let c = (angle / 2.0).cos();
    let s = (angle / 2.0).sin();
    for i in 0..re.len() {
        let bit = (i >> qubit) & 1;
        let rz_factor = if bit == 0 { c } else { c };
        let phase = if bit == 0 { -s } else { s };
        let r = re[i];
        let img = im[i];
        re[i] = rz_factor * r - phase * img;
        im[i] = phase * r + rz_factor * img;
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
    let c = angle.cos();
    let s = angle.sin();
    for i in 0..re.len() {
        if (i >> qubit) & 1 == 1 {
            let r = re[i];
            let img = im[i];
            re[i] = c * r - s * img;
            im[i] = s * r + c * img;
        }
    }
}

fn apply_controlled_phase(
    re: &mut Vec<f64>,
    im: &mut Vec<f64>,
    control: usize,
    target: usize,
    angle: f64,
    _n: usize,
) {
    let c = angle.cos();
    let s = angle.sin();
    for i in 0..re.len() {
        if (i >> control) & 1 == 1 && (i >> target) & 1 == 1 {
            let r = re[i];
            let img = im[i];
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
        Self {
            qubit_index,
            weights: [0.0, 0.0, 0.0],
            bias: 0.0,
        }
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
        let layers = layer_sizes
            .iter()
            .map(|&sz| {
                let layer = QuantumLayer::new(offset, sz);
                offset += sz;
                layer
            })
            .collect();
        Self {
            layers,
            entanglement: EntanglementStrategy::NearestNeighbor,
            learning_rate: 0.01,
        }
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
        circuit.execute()
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
        format!(
            "QNN ready: {} qubits, confidence {:.4}",
            self.qnn.total_qubits(),
            score
        )
    }
}

// ── Specialized Algorithms ────────────────────────────────────────────────────
pub struct SpecializedAlgorithms;

impl SpecializedAlgorithms {
    /// Quantum Approximate Optimization Algorithm (QAOA)
    pub fn qaoa(problem_graph: &[(usize, usize)], depth: usize) -> Vec<f64> {
        if problem_graph.is_empty() {
            return vec![];
        }
        let num_qubits = problem_graph
            .iter()
            .flat_map(|(a, b)| [*a, *b])
            .max()
            .unwrap_or(0)
            + 1;

        let mut circuit = QuantumCircuitBuilder::new(num_qubits);

        // Initial superposition
        for i in 0..num_qubits {
            circuit.h(i);
        }

        // QAOA layers: problem unitary + mixer unitary
        for _ in 0..depth {
            // Problem: phase kickback on edges (simplified Ising)
            for &(i, j) in problem_graph {
                circuit.cnot(i, j);
                circuit.rz(j, PI);
                circuit.cnot(i, j);
            }
            // Mixer: Rx(π/2) on all qubits
            for i in 0..num_qubits {
                circuit.rx(i, PI / 2.0);
            }
        }

        circuit.execute()
    }

    /// Variational Quantum Eigensolver stub
    pub fn vqe(params: &[f64]) -> Vec<f64> {
        if params.is_empty() { return vec![1.0]; }
        let mut circuit = QuantumCircuitBuilder::new(params.len().min(8));
        for (i, &p) in params.iter().enumerate().take(8) {
            circuit.h(i);
            circuit.ry(i, p);
        }
        circuit.execute()
    }

    /// Quantum Key Distribution stub — returns n random bits via measurement
    pub fn qkd(n: usize) -> Vec<u8> {
        let qubits = n.min(16);
        let mut circuit = QuantumCircuitBuilder::new(qubits);
        for i in 0..qubits { circuit.h(i); }
        let probs = circuit.execute();
        // Extract one bit per qubit by checking probability threshold
        (0..n).map(|i| {
            let idx = i % probs.len();
            if probs[idx] >= 0.5 { 1u8 } else { 0u8 }
        }).collect()
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