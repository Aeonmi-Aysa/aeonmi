//! AEONMI Quantum Simulator - Basic quantum state simulation capabilities
//! Provides state vector simulation for quantum operations
#![cfg_attr(not(test), allow(dead_code))]

use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Complex number representation for quantum amplitudes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    pub fn real(real: f64) -> Self {
        Self::new(real, 0.0)
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    #[allow(dead_code)]
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    #[allow(dead_code)]
    pub fn conjugate(&self) -> Self {
        Self::new(self.real, -self.imag)
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imag + other.imag)
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imag - other.imag)
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.real * scalar, self.imag * scalar)
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }
}

/// Quantum state representation using state vectors
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub amplitudes: Vec<Complex>,
    pub num_qubits: usize,
}

#[allow(dead_code)]
impl QuantumState {
    /// Create a new quantum state with n qubits in |0...0⟩ state
    pub fn new(num_qubits: usize) -> Self {
        let num_states = 1 << num_qubits; // 2^n states
        let mut amplitudes = vec![Complex::new(0.0, 0.0); num_states];
        amplitudes[0] = Complex::real(1.0); // |0...0⟩ state

        Self {
            amplitudes,
            num_qubits,
        }
    }

    /// Create superposition state |+⟩ = (|0⟩ + |1⟩)/√2
    pub fn plus_state() -> Self {
        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        Self {
            amplitudes: vec![
                Complex::real(sqrt_half), // |0⟩
                Complex::real(sqrt_half), // |1⟩
            ],
            num_qubits: 1,
        }
    }

    /// Create superposition state |-⟩ = (|0⟩ - |1⟩)/√2
    pub fn minus_state() -> Self {
        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        Self {
            amplitudes: vec![
                Complex::real(sqrt_half),  // |0⟩
                Complex::real(-sqrt_half), // |1⟩
            ],
            num_qubits: 1,
        }
    }

    /// Normalize the quantum state
    pub fn normalize(&mut self) {
        let norm_squared: f64 = self
            .amplitudes
            .iter()
            .map(|amp| amp.magnitude_squared())
            .sum();
        let norm = norm_squared.sqrt();

        if norm > 1e-10 {
            for amp in &mut self.amplitudes {
                *amp = *amp * (1.0 / norm);
            }
        }
    }

    /// Get probability of measuring a specific computational basis state
    pub fn get_probability(&self, state_index: usize) -> f64 {
        if state_index < self.amplitudes.len() {
            self.amplitudes[state_index].magnitude_squared()
        } else {
            0.0
        }
    }

    /// Measure the quantum state, collapsing it to a classical state
    pub fn measure(&mut self) -> usize {
        let mut rng_state = 1u64; // Simple LCG for deterministic results

        // Generate random number using LCG
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let random = (rng_state as f64) / (u64::MAX as f64);

        let mut cumulative_prob = 0.0;
        for (i, amplitude) in self.amplitudes.iter().enumerate() {
            cumulative_prob += amplitude.magnitude_squared();
            if random <= cumulative_prob {
                // Collapse to measured state
                self.amplitudes.fill(Complex::new(0.0, 0.0));
                self.amplitudes[i] = Complex::real(1.0);
                return i;
            }
        }

        // Fallback to last state
        let last_state = self.amplitudes.len() - 1;
        self.amplitudes.fill(Complex::new(0.0, 0.0));
        self.amplitudes[last_state] = Complex::real(1.0);
        last_state
    }
}

/// Quantum gate operations
pub struct QuantumGates;

impl QuantumGates {
    /// Apply Hadamard gate (superposition)
    pub fn hadamard(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow!("Qubit index {} out of range", qubit));
        }

        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        let num_states = state.amplitudes.len();
        let mut new_amplitudes = vec![Complex::new(0.0, 0.0); num_states];

        for i in 0..num_states {
            let qubit_bit = (i >> qubit) & 1;
            let other_state = i ^ (1 << qubit); // Flip the qubit bit

            if qubit_bit == 0 {
                // |0⟩ component
                new_amplitudes[i] = new_amplitudes[i] + state.amplitudes[i] * sqrt_half;
                new_amplitudes[other_state] =
                    new_amplitudes[other_state] + state.amplitudes[i] * sqrt_half;
            } else {
                // |1⟩ component
                new_amplitudes[other_state] =
                    new_amplitudes[other_state] + state.amplitudes[i] * sqrt_half;
                new_amplitudes[i] = new_amplitudes[i] + state.amplitudes[i] * (-sqrt_half);
            }
        }

        state.amplitudes = new_amplitudes;
        Ok(())
    }

    /// Apply Pauli-X gate (bit flip)
    pub fn pauli_x(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow!("Qubit index {} out of range", qubit));
        }

        let num_states = state.amplitudes.len();
        let mut new_amplitudes = vec![Complex::new(0.0, 0.0); num_states];

        for i in 0..num_states {
            let flipped_state = i ^ (1 << qubit); // Flip the qubit bit
            new_amplitudes[flipped_state] = state.amplitudes[i];
        }

        state.amplitudes = new_amplitudes;
        Ok(())
    }

    /// Apply Pauli-Z gate (phase flip)
    pub fn pauli_z(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow!("Qubit index {} out of range", qubit));
        }

        for i in 0..state.amplitudes.len() {
            let qubit_bit = (i >> qubit) & 1;
            if qubit_bit == 1 {
                state.amplitudes[i] = state.amplitudes[i] * (-1.0);
            }
        }

        Ok(())
    }

    /// Apply CNOT gate (controlled-X)
    #[allow(dead_code)]
    pub fn cnot(state: &mut QuantumState, control: usize, target: usize) -> Result<()> {
        if control >= state.num_qubits || target >= state.num_qubits {
            return Err(anyhow!("Qubit index out of range"));
        }

        let num_states = state.amplitudes.len();
        let mut new_amplitudes = vec![Complex::new(0.0, 0.0); num_states];

        for i in 0..num_states {
            let control_bit = (i >> control) & 1;
            if control_bit == 1 {
                // Control is |1⟩, flip target
                let flipped_state = i ^ (1 << target);
                new_amplitudes[flipped_state] = state.amplitudes[i];
            } else {
                // Control is |0⟩, no change
                new_amplitudes[i] = state.amplitudes[i];
            }
        }

        state.amplitudes = new_amplitudes;
        Ok(())
    }
}

/// Quantum simulator managing multiple qubits
#[derive(Debug)]
pub struct QuantumSimulator {
    pub qubits: HashMap<String, QuantumState>,
    pub entangled_systems: Vec<Vec<String>>, // Track entangled qubit groups
}

#[allow(dead_code)]
impl QuantumSimulator {
    pub fn new() -> Self {
        Self {
            qubits: HashMap::new(),
            entangled_systems: Vec::new(),
        }
    }

    /// Create a new qubit in |0⟩ state
    pub fn create_qubit(&mut self, name: String) {
        self.qubits.insert(name, QuantumState::new(1));
    }

    fn ensure_qubit(&mut self, name: &str) -> &mut QuantumState {
        self.qubits
            .entry(name.to_string())
            .or_insert_with(|| QuantumState::new(1))
    }

    /// Apply superposition (Hadamard) to a qubit
    pub fn superpose(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::hadamard(state, 0)
        } else {
            Err(anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Apply Pauli-X gate to a qubit
    pub fn pauli_x(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::pauli_x(state, 0)
        } else {
            Err(anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Apply Pauli-Z gate to a qubit  
    pub fn pauli_z(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::pauli_z(state, 0)
        } else {
            Err(anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Measure a qubit, collapsing its state
    pub fn measure(&mut self, qubit_name: &str) -> Result<u8> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            Ok(state.measure() as u8)
        } else {
            Err(anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    pub fn apply_controlled_z(&mut self, control_name: &str, target_name: &str) -> Result<()> {
        let control_prob = self.get_zero_probability(control_name)?;
        // Apply CZ gate: phase flip when control is |1⟩
        if control_prob < 0.5 { // control is more likely to be |1⟩
            self.pauli_z(target_name)?;
        }
        Ok(())
    }

    pub fn apply_rx(&mut self, qubit_name: &str, angle: f64) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // Apply RX rotation around X-axis
            let cos_half = (angle / 2.0).cos();
            let sin_half = (angle / 2.0).sin();

            let new_0 = Complex::new(cos_half, 0.0) * state.amplitudes[0]
                      - Complex::new(0.0, sin_half) * state.amplitudes[1];
            let new_1 = Complex::new(0.0, -sin_half) * state.amplitudes[0]
                      + Complex::new(cos_half, 0.0) * state.amplitudes[1];

            state.amplitudes[0] = new_0;
            state.amplitudes[1] = new_1;
            state.normalize();
        }
        Ok(())
    }

    pub fn apply_ry(&mut self, qubit_name: &str, angle: f64) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // Apply RY rotation around Y-axis
            let cos_half = (angle / 2.0).cos();
            let sin_half = (angle / 2.0).sin();

            let new_0 = Complex::new(cos_half, 0.0) * state.amplitudes[0]
                      - Complex::new(sin_half, 0.0) * state.amplitudes[1];
            let new_1 = Complex::new(sin_half, 0.0) * state.amplitudes[0]
                      + Complex::new(cos_half, 0.0) * state.amplitudes[1];

            state.amplitudes[0] = new_0;
            state.amplitudes[1] = new_1;
            state.normalize();
        }
        Ok(())
    }

    pub fn apply_rz(&mut self, qubit_name: &str, angle: f64) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // Apply RZ rotation around Z-axis
            let cos_half = (angle / 2.0).cos();
            let sin_half = (angle / 2.0).sin();

            let new_0 = Complex::new(cos_half, -sin_half) * state.amplitudes[0];
            let new_1 = Complex::new(cos_half, sin_half) * state.amplitudes[1];

            state.amplitudes[0] = new_0;
            state.amplitudes[1] = new_1;
            state.normalize();
        }
        Ok(())
    }

    /// Apply S gate (Z^0.5 = sqrt(Z)) to a qubit
    pub fn apply_s_gate(&mut self, qubit_name: &str) -> Result<()> {
        self.apply_rz(qubit_name, std::f64::consts::PI / 2.0)
    }

    /// Apply T gate (Z^0.25) to a qubit
    pub fn apply_t_gate(&mut self, qubit_name: &str) -> Result<()> {
        self.apply_rz(qubit_name, std::f64::consts::PI / 4.0)
    }

    /// Apply Hadamard gate to a qubit
    pub fn apply_hadamard(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // Hadamard gate: |0⟩ → (|0⟩ + |1⟩)/√2, |1⟩ → (|0⟩ - |1⟩)/√2
            let sqrt_2_inv = Complex::new(1.0 / (2.0_f64).sqrt(), 0.0);
            let new_0 = sqrt_2_inv * (state.amplitudes[0] + state.amplitudes[1]);
            let new_1 = sqrt_2_inv * (state.amplitudes[0] - state.amplitudes[1]);

            state.amplitudes[0] = new_0;
            state.amplitudes[1] = new_1;
            state.normalize();
        }
        Ok(())
    }

    /// Apply controlled-X (CNOT) gate
    pub fn apply_controlled_x(&mut self, control_name: &str, target_name: &str) -> Result<()> {
        // Get control probability first (avoids borrowing issues)
        let control_prob_1 = self.get_zero_probability(control_name)? * (-1.0) + 1.0; // P(|1⟩) = 1 - P(|0⟩)

        // Apply CNOT if control is in |1⟩ state
        if control_prob_1 > 0.5 {
            if let Some(target_state) = self.qubits.get_mut(target_name) {
                // Control is in |1⟩ state, apply X to target
                let temp = target_state.amplitudes[0];
                target_state.amplitudes[0] = target_state.amplitudes[1];
                target_state.amplitudes[1] = temp;
            } else {
                return Err(anyhow!("Target qubit '{}' not found", target_name));
            }
        }
        Ok(())
    }

    /// Reset a qubit to |0⟩ state
    pub fn reset_qubit(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            state.amplitudes[0] = Complex::new(1.0, 0.0);
            state.amplitudes[1] = Complex::new(0.0, 0.0);
        }
        Ok(())
    }

    /// Get the probability of measuring |0⟩ for a qubit
    pub fn get_zero_probability(&self, qubit_name: &str) -> Result<f64> {
        if let Some(state) = self.qubits.get(qubit_name) {
            Ok(state.get_probability(0))
        } else {
            Err(anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    pub fn prepare_named_state(
        &mut self,
        qubit_name: &str,
        label: &str,
        amplitude: Option<f64>,
    ) -> Result<()> {
        let components = vec![(label.to_string(), amplitude)];
        self.prepare_state_from_components(qubit_name, &components)
    }

    pub fn prepare_state_from_components(
        &mut self,
        qubit_name: &str,
        components: &[(String, Option<f64>)],
    ) -> Result<()> {
        let amplitudes = Self::components_to_vector(components)?;
        self.set_state(qubit_name, amplitudes)
    }

    pub fn probability_from_components(&self, components: &[(String, Option<f64>)]) -> Result<f64> {
        let amplitudes = Self::components_to_vector(components)?;
        let mut state = QuantumState::new(1);
        state.amplitudes = amplitudes;
        state.normalize();
        Ok(state.get_probability(1))
    }

    pub fn set_state(&mut self, qubit_name: &str, amplitudes: Vec<Complex>) -> Result<()> {
        if amplitudes.len() != 2 {
            return Err(anyhow!(
                "Only single-qubit states are supported in this simulator (got {} amplitudes)",
                amplitudes.len()
            ));
        }

        let state = self.ensure_qubit(qubit_name);
        state.amplitudes = amplitudes;
        state.num_qubits = 1;
        state.normalize();
        Ok(())
    }

    /// Create entanglement between two qubits (simplified)
    pub fn entangle(&mut self, qubit1: &str, qubit2: &str) -> Result<()> {
        // For now, just track that these qubits are entangled
        // A full implementation would merge their state spaces

        if !self.qubits.contains_key(qubit1) {
            return Err(anyhow!("Qubit '{}' not found", qubit1));
        }
        if !self.qubits.contains_key(qubit2) {
            return Err(anyhow!("Qubit '{}' not found", qubit2));
        }

        // Find or create entangled system
        let mut found_system = None;
        for (i, system) in self.entangled_systems.iter_mut().enumerate() {
            if system.contains(&qubit1.to_string()) || system.contains(&qubit2.to_string()) {
                if !system.contains(&qubit1.to_string()) {
                    system.push(qubit1.to_string());
                }
                if !system.contains(&qubit2.to_string()) {
                    system.push(qubit2.to_string());
                }
                found_system = Some(i);
                break;
            }
        }

        if found_system.is_none() {
            self.entangled_systems
                .push(vec![qubit1.to_string(), qubit2.to_string()]);
        }

        Ok(())
    }

    /// Reset simulator state
    pub fn reset(&mut self) {
        self.qubits.clear();
        self.entangled_systems.clear();
    }

    fn canonical_label(label: &str) -> String {
        let trimmed = label.trim();
        let without_pipe = trimmed
            .trim_start_matches('|')
            .trim_end_matches('⟩')
            .trim_end_matches('>')
            .trim();
        without_pipe.to_string()
    }

    fn vector_for_label(label: &str) -> Result<[Complex; 2]> {
        let canonical = Self::canonical_label(label);
        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        match canonical.as_str() {
            "0" | "zero" => Ok([Complex::real(1.0), Complex::real(0.0)]),
            "1" | "one" => Ok([Complex::real(0.0), Complex::real(1.0)]),
            "+" | "plus" => Ok([Complex::real(sqrt_half), Complex::real(sqrt_half)]),
            "-" | "minus" => Ok([Complex::real(sqrt_half), Complex::real(-sqrt_half)]),
            other => Err(anyhow!("Unsupported quantum state '{}'", other)),
        }
    }

    fn components_to_vector(components: &[(String, Option<f64>)]) -> Result<Vec<Complex>> {
        let mut accumulator = vec![Complex::new(0.0, 0.0); 2];
        for (label, amplitude) in components {
            let mut vec = Self::vector_for_label(label)?;
            if let Some(scale) = amplitude {
                vec[0] = vec[0] * *scale;
                vec[1] = vec[1] * *scale;
            }
            accumulator[0] = accumulator[0] + vec[0];
            accumulator[1] = accumulator[1] + vec[1];
        }

        if accumulator[0].magnitude_squared() + accumulator[1].magnitude_squared() < 1e-12 {
            accumulator[0] = Complex::real(1.0);
        }

        Ok(accumulator)
    }
}

impl Default for QuantumSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_state_creation() {
        let state = QuantumState::new(2);
        assert_eq!(state.num_qubits, 2);
        assert_eq!(state.amplitudes.len(), 4); // 2^2 states
        assert_eq!(state.amplitudes[0], Complex::real(1.0)); // |00⟩
    }

    #[test]
    fn test_hadamard_gate() {
        let mut state = QuantumState::new(1);
        QuantumGates::hadamard(&mut state, 0).unwrap();

        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        assert!((state.amplitudes[0].real - sqrt_half).abs() < 1e-10);
        assert!((state.amplitudes[1].real - sqrt_half).abs() < 1e-10);
    }

    #[test]
    fn test_quantum_simulator() {
        let mut sim = QuantumSimulator::new();
        sim.create_qubit("q1".to_string());

        // Test superposition
        sim.superpose("q1").unwrap();
        let prob = sim.get_zero_probability("q1").unwrap();
        assert!((prob - 0.5).abs() < 1e-10);

        // Test measurement
        let result = sim.measure("q1").unwrap();
        assert!(result == 0 || result == 1);
    }

    #[test]
    fn test_superposition_states() {
        let mut plus = QuantumState::plus_state();
        plus.normalize();
        assert!((plus.get_probability(0) - 0.5).abs() < 1e-10);
        assert!((plus.get_probability(1) - 0.5).abs() < 1e-10);

        let mut minus = QuantumState::minus_state();
        minus.normalize();
        assert!((minus.get_probability(0) - 0.5).abs() < 1e-10);
        assert!((minus.get_probability(1) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_simulator_reset() {
        let mut sim = QuantumSimulator::new();
        sim.create_qubit("q1".to_string());
        sim.superpose("q1").unwrap();
        assert!(!sim.qubits.is_empty());

        sim.reset();
        assert!(sim.qubits.is_empty());
        assert!(sim.entangled_systems.is_empty());
    }
}
