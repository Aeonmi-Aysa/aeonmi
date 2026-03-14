//! AEONMI Quantum Simulator - Basic quantum state simulation capabilities
//! Provides state vector simulation for quantum operations

use std::collections::HashMap;
use anyhow::Result;

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
    
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }
    
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

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real
        )
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.real * scalar, self.imag * scalar)
    }
}

/// Quantum state representation using state vectors
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub amplitudes: Vec<Complex>,
    pub num_qubits: usize,
}

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
        let norm_squared: f64 = self.amplitudes.iter()
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
        // Use time-based seed for actual randomness
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut rng_state = if seed == 0 { 1u64 } else { seed };
        
        // Generate random number using LCG
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
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
            return Err(anyhow::anyhow!("Qubit index {} out of range", qubit));
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
                new_amplitudes[other_state] = new_amplitudes[other_state] + state.amplitudes[i] * sqrt_half;
            } else {
                // |1⟩ component
                new_amplitudes[other_state] = new_amplitudes[other_state] + state.amplitudes[i] * sqrt_half;
                new_amplitudes[i] = new_amplitudes[i] + state.amplitudes[i] * (-sqrt_half);
            }
        }
        
        state.amplitudes = new_amplitudes;
        Ok(())
    }
    
    /// Apply Pauli-X gate (bit flip)
    pub fn pauli_x(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow::anyhow!("Qubit index {} out of range", qubit));
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
    
    /// Apply Pauli-Y gate (bit + phase flip)
    pub fn pauli_y(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow::anyhow!("Qubit index {} out of range", qubit));
        }
        
        let num_states = state.amplitudes.len();
        let mut new_amplitudes = vec![Complex::new(0.0, 0.0); num_states];
        
        for i in 0..num_states {
            let qubit_bit = (i >> qubit) & 1;
            let flipped_state = i ^ (1 << qubit);
            if qubit_bit == 0 {
                // |0⟩ → i|1⟩
                new_amplitudes[flipped_state] = Complex::new(-state.amplitudes[i].imag, state.amplitudes[i].real);
            } else {
                // |1⟩ → -i|0⟩
                new_amplitudes[flipped_state] = Complex::new(state.amplitudes[i].imag, -state.amplitudes[i].real);
            }
        }
        
        state.amplitudes = new_amplitudes;
        Ok(())
    }
    
    /// Apply Pauli-Z gate (phase flip)
    pub fn pauli_z(state: &mut QuantumState, qubit: usize) -> Result<()> {
        if qubit >= state.num_qubits {
            return Err(anyhow::anyhow!("Qubit index {} out of range", qubit));
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
    pub fn cnot(state: &mut QuantumState, control: usize, target: usize) -> Result<()> {
        if control >= state.num_qubits || target >= state.num_qubits {
            return Err(anyhow::anyhow!("Qubit index out of range"));
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

/// Quantum simulator managing multiple qubits.
/// Phase 2: joint state-vector model.  When two qubits are entangled they
/// share a single multi-qubit `QuantumState`.  CNOT operates on the joint
/// state so interference and Bell-state preparation work correctly.
#[derive(Debug)]
pub struct QuantumSimulator {
    /// Per-qubit state for standalone (un-entangled) qubits.
    pub qubits: HashMap<String, QuantumState>,
    /// Joint state-vector systems: each entry is a group of qubit names that
    /// share a single multi-qubit state, plus that state.
    pub joint_systems: Vec<JointSystem>,
    /// Legacy entangled_systems field kept for compatibility.
    pub entangled_systems: Vec<Vec<String>>,
}

/// A joint quantum state shared by multiple named qubits.
#[derive(Debug, Clone)]
pub struct JointSystem {
    /// Qubit names in order (qubit[0] = most-significant bit, qubit[n-1] = least).
    pub names: Vec<String>,
    /// The combined state vector (length = 2^names.len()).
    pub state: QuantumState,
}

impl JointSystem {
    /// Create a joint system for two qubits already in separate single-qubit states.
    pub fn from_two(name0: &str, s0: &QuantumState, name1: &str, s1: &QuantumState) -> Self {
        // Tensor product |s0⟩ ⊗ |s1⟩  →  4-element state vector
        let mut amplitudes = vec![Complex::new(0.0, 0.0); 4];
        for (i, a) in s0.amplitudes.iter().enumerate() {
            for (j, b) in s1.amplitudes.iter().enumerate() {
                amplitudes[i * 2 + j] = *a * *b;
            }
        }
        JointSystem {
            names: vec![name0.to_string(), name1.to_string()],
            state: QuantumState { amplitudes, num_qubits: 2 },
        }
    }

    /// Return the index of `qubit_name` in this joint system, or None.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    /// Apply a single-qubit gate (given as a 2×2 matrix in row-major order) to
    /// the qubit at `qubit_idx` within this joint system.
    pub fn apply_single_gate(&mut self, qubit_idx: usize, gate: [[Complex; 2]; 2]) {
        let n = self.names.len();
        let num_states = 1 << n;
        // Iterate over pairs (|0⟩ component, |1⟩ component) for this qubit.
        // Only process pairs where this qubit is |0⟩ to avoid double-counting.
        let mut new_amps = vec![Complex::new(0.0, 0.0); num_states];
        for state_idx in 0..num_states {
            let bit = (state_idx >> (n - 1 - qubit_idx)) & 1;
            if bit == 0 {
                let partner = state_idx ^ (1 << (n - 1 - qubit_idx));
                let a0 = self.state.amplitudes[state_idx];
                let a1 = self.state.amplitudes[partner];
                new_amps[state_idx] = gate[0][0] * a0 + gate[0][1] * a1;
                new_amps[partner]   = gate[1][0] * a0 + gate[1][1] * a1;
            }
        }
        self.state.amplitudes = new_amps;
    }

    /// Apply CNOT with control at `ctrl_idx` and target at `tgt_idx`.
    /// Reads from a snapshot of the original state to avoid interference between pairs.
    pub fn apply_cnot(&mut self, ctrl_idx: usize, tgt_idx: usize) {
        let n = self.names.len();
        let num_states = 1 << n;
        // Use original amplitudes as read-source so swaps within a single pass are correct.
        let original = self.state.amplitudes.clone();
        let mut new_amps = original.clone();
        for state_idx in 0..num_states {
            let ctrl_bit = (state_idx >> (n - 1 - ctrl_idx)) & 1;
            if ctrl_bit == 1 {
                let flipped = state_idx ^ (1 << (n - 1 - tgt_idx));
                // Only process the pair once (lower index wins)
                if state_idx < flipped {
                    new_amps[flipped]    = original[state_idx];
                    new_amps[state_idx]  = original[flipped];
                }
            }
        }
        self.state.amplitudes = new_amps;
    }

    /// Measure a single qubit within the joint system, collapsing the state.
    /// Returns 0 or 1.
    pub fn measure_qubit(&mut self, qubit_idx: usize) -> u8 {
        let n = self.names.len();
        let num_states = 1 << n;
        // Compute probability of outcome 0
        let mut p0 = 0.0f64;
        for state_idx in 0..num_states {
            let bit = (state_idx >> (n - 1 - qubit_idx)) & 1;
            if bit == 0 {
                p0 += self.state.amplitudes[state_idx].magnitude_squared();
            }
        }
        let outcome = if rand::random::<f64>() < p0 { 0u8 } else { 1u8 };
        // Collapse state
        let norm_sq: f64 = self.state.amplitudes.iter().enumerate().map(|(idx, a)| {
            let bit = (idx >> (n - 1 - qubit_idx)) & 1;
            if (bit == 0) == (outcome == 0) { a.magnitude_squared() } else { 0.0 }
        }).sum();
        let norm = norm_sq.sqrt();
        for state_idx in 0..num_states {
            let bit = (state_idx >> (n - 1 - qubit_idx)) & 1;
            if (bit == 0) == (outcome == 0) {
                if norm > 1e-15 {
                    self.state.amplitudes[state_idx] = self.state.amplitudes[state_idx] * (1.0 / norm);
                }
            } else {
                self.state.amplitudes[state_idx] = Complex::new(0.0, 0.0);
            }
        }
        outcome
    }
}

impl QuantumSimulator {
    pub fn new() -> Self {
        Self {
            qubits: HashMap::new(),
            joint_systems: Vec::new(),
            entangled_systems: Vec::new(),
        }
    }

    /// Return the joint system that contains `name`, if any.
    fn joint_system_for(&self, name: &str) -> Option<usize> {
        self.joint_systems.iter().position(|js| js.names.contains(&name.to_string()))
    }

    /// Create a new qubit in |0⟩ state
    pub fn create_qubit(&mut self, name: String) {
        self.qubits.insert(name, QuantumState::new(1));
    }

    /// Apply superposition (Hadamard) to a qubit
    pub fn superpose(&mut self, qubit_name: &str) -> Result<()> {
        let sqrt_half = 1.0 / 2.0_f64.sqrt();
        let h = [[Complex::real(sqrt_half), Complex::real(sqrt_half)],
                 [Complex::real(sqrt_half), Complex::new(-sqrt_half, 0.0)]];
        if let Some(js_idx) = self.joint_system_for(qubit_name) {
            let q_idx = self.joint_systems[js_idx].index_of(qubit_name).unwrap();
            self.joint_systems[js_idx].apply_single_gate(q_idx, h);
            return Ok(());
        }
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::hadamard(state, 0)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    fn apply_single_gate_to_qubit(&mut self, qubit_name: &str, gate: [[Complex; 2]; 2]) -> Result<()> {
        if let Some(js_idx) = self.joint_system_for(qubit_name) {
            let q_idx = self.joint_systems[js_idx].index_of(qubit_name).unwrap();
            self.joint_systems[js_idx].apply_single_gate(q_idx, gate);
            return Ok(());
        }
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // Use the existing per-qubit gate application
            let tmp_state = state.clone();
            let a0 = tmp_state.amplitudes[0];
            let a1 = tmp_state.amplitudes[1];
            state.amplitudes[0] = gate[0][0] * a0 + gate[0][1] * a1;
            state.amplitudes[1] = gate[1][0] * a0 + gate[1][1] * a1;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Apply Pauli-X gate to a qubit
    pub fn pauli_x(&mut self, qubit_name: &str) -> Result<()> {
        let x = [[Complex::new(0.0, 0.0), Complex::real(1.0)],
                 [Complex::real(1.0), Complex::new(0.0, 0.0)]];
        self.apply_single_gate_to_qubit(qubit_name, x)
    }

    /// Apply Pauli-Y gate to a qubit
    pub fn pauli_y(&mut self, qubit_name: &str) -> Result<()> {
        let y = [[Complex::new(0.0, 0.0), Complex::new(0.0, -1.0)],
                 [Complex::new(0.0, 1.0), Complex::new(0.0, 0.0)]];
        self.apply_single_gate_to_qubit(qubit_name, y)
    }

    /// Apply Pauli-Z gate to a qubit
    pub fn pauli_z(&mut self, qubit_name: &str) -> Result<()> {
        let z = [[Complex::real(1.0), Complex::new(0.0, 0.0)],
                 [Complex::new(0.0, 0.0), Complex::new(-1.0, 0.0)]];
        self.apply_single_gate_to_qubit(qubit_name, z)
    }

    /// Apply S gate (phase gate)
    pub fn phase_s(&mut self, qubit_name: &str) -> Result<()> {
        let s = [[Complex::real(1.0), Complex::new(0.0, 0.0)],
                 [Complex::new(0.0, 0.0), Complex::new(0.0, 1.0)]];
        self.apply_single_gate_to_qubit(qubit_name, s)
    }

    /// Apply T gate
    pub fn phase_t(&mut self, qubit_name: &str) -> Result<()> {
        let cos45 = std::f64::consts::FRAC_1_SQRT_2;
        let t = [[Complex::real(1.0), Complex::new(0.0, 0.0)],
                 [Complex::new(0.0, 0.0), Complex::new(cos45, cos45)]];
        self.apply_single_gate_to_qubit(qubit_name, t)
    }

    /// Apply CNOT gate using the joint state-vector model (P2-8/P2-9).
    /// If the two qubits are not already in a joint system, they are merged first.
    pub fn apply_cnot(&mut self, control_name: &str, target_name: &str) -> Result<()> {
        // Check both qubits exist
        let ctrl_in_joint = self.joint_system_for(control_name);
        let tgt_in_joint = self.joint_system_for(target_name);

        match (ctrl_in_joint, tgt_in_joint) {
            (Some(ci), Some(ti)) if ci == ti => {
                // Already in the same joint system
                let js = &mut self.joint_systems[ci];
                let ctrl_idx = js.index_of(control_name).unwrap();
                let tgt_idx = js.index_of(target_name).unwrap();
                js.apply_cnot(ctrl_idx, tgt_idx);
            }
            (Some(_), Some(_)) => {
                // In different joint systems — merging is complex; handle simple cases
                return Err(anyhow::anyhow!(
                    "CNOT between qubits in different joint systems not yet supported"
                ));
            }
            (None, None) => {
                // Both are standalone — merge them into a joint system
                let s0 = self.qubits.remove(control_name).ok_or_else(||
                    anyhow::anyhow!("Control qubit '{}' not found", control_name))?;
                let s1 = self.qubits.remove(target_name).ok_or_else(||
                    anyhow::anyhow!("Target qubit '{}' not found", target_name))?;
                let mut js = JointSystem::from_two(control_name, &s0, target_name, &s1);
                js.apply_cnot(0, 1); // ctrl=0, tgt=1 in the joint system
                self.joint_systems.push(js);
                // Track in legacy entangled_systems
                self.entangle_tracking(control_name, target_name);
            }
            (Some(ci), None) => {
                // Control is in a joint system, target is standalone
                let s1 = self.qubits.remove(target_name).ok_or_else(||
                    anyhow::anyhow!("Target qubit '{}' not found", target_name))?;
                let js = &mut self.joint_systems[ci];
                let old_n = js.names.len();
                // Extend joint state: new_state = old_state ⊗ |s1⟩
                let mut new_amps = vec![Complex::new(0.0, 0.0); js.state.amplitudes.len() * 2];
                for (idx, a) in js.state.amplitudes.iter().enumerate() {
                    new_amps[idx * 2]     = *a * s1.amplitudes[0];
                    new_amps[idx * 2 + 1] = *a * s1.amplitudes[1];
                }
                js.state = QuantumState { amplitudes: new_amps, num_qubits: old_n + 1 };
                js.names.push(target_name.to_string());
                let ctrl_idx = js.index_of(control_name).unwrap();
                let tgt_idx = js.names.len() - 1;
                js.apply_cnot(ctrl_idx, tgt_idx);
            }
            (None, Some(ti)) => {
                // Target is in a joint system, control is standalone
                let s0 = self.qubits.remove(control_name).ok_or_else(||
                    anyhow::anyhow!("Control qubit '{}' not found", control_name))?;
                let js = &mut self.joint_systems[ti];
                let old_n = js.names.len();
                // Prepend control: new_state = |s0⟩ ⊗ old_state
                let mut new_amps = vec![Complex::new(0.0, 0.0); js.state.amplitudes.len() * 2];
                for (idx, a) in js.state.amplitudes.iter().enumerate() {
                    new_amps[idx]                                          = s0.amplitudes[0] * *a;
                    new_amps[idx + js.state.amplitudes.len()] = s0.amplitudes[1] * *a;
                }
                js.state = QuantumState { amplitudes: new_amps, num_qubits: old_n + 1 };
                js.names.insert(0, control_name.to_string());
                let ctrl_idx = 0;
                let tgt_idx = js.index_of(target_name).unwrap();
                js.apply_cnot(ctrl_idx, tgt_idx);
            }
        }
        Ok(())
    }

    fn entangle_tracking(&mut self, q1: &str, q2: &str) {
        for system in self.entangled_systems.iter_mut() {
            let has1 = system.contains(&q1.to_string());
            let has2 = system.contains(&q2.to_string());
            if has1 || has2 {
                if !has1 { system.push(q1.to_string()); }
                if !has2 { system.push(q2.to_string()); }
                return;
            }
        }
        self.entangled_systems.push(vec![q1.to_string(), q2.to_string()]);
    }

    /// Measure a qubit, collapsing its state
    pub fn measure(&mut self, qubit_name: &str) -> Result<u8> {
        if let Some(js_idx) = self.joint_system_for(qubit_name) {
            let q_idx = self.joint_systems[js_idx].index_of(qubit_name).unwrap();
            return Ok(self.joint_systems[js_idx].measure_qubit(q_idx));
        }
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            Ok(state.measure() as u8)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Get the probability of measuring |0⟩ for a qubit
    pub fn get_zero_probability(&self, qubit_name: &str) -> Result<f64> {
        if let Some(js_idx) = self.joint_system_for(qubit_name) {
            let js = &self.joint_systems[js_idx];
            let q_idx = js.index_of(qubit_name).unwrap();
            let n = js.names.len();
            let p0: f64 = js.state.amplitudes.iter().enumerate()
                .filter(|(idx, _)| (idx >> (n - 1 - q_idx)) & 1 == 0)
                .map(|(_, a)| a.magnitude_squared())
                .sum();
            return Ok(p0);
        }
        if let Some(state) = self.qubits.get(qubit_name) {
            Ok(state.get_probability(0))
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Create entanglement between two qubits using the real CNOT model.
    /// For `entangle(q1, q2)` from Aeonmi scripts we apply H then CNOT to
    /// produce a Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2.
    pub fn entangle(&mut self, qubit1: &str, qubit2: &str) -> Result<()> {
        if !self.qubits.contains_key(qubit1) && self.joint_system_for(qubit1).is_none() {
            return Err(anyhow::anyhow!("Qubit '{}' not found", qubit1));
        }
        if !self.qubits.contains_key(qubit2) && self.joint_system_for(qubit2).is_none() {
            return Err(anyhow::anyhow!("Qubit '{}' not found", qubit2));
        }
        // Apply H to qubit1, then CNOT(qubit1, qubit2) to create a Bell state
        self.superpose(qubit1)?;
        self.apply_cnot(qubit1, qubit2)?;
        Ok(())
    }

    /// Reset simulator state
    pub fn reset(&mut self) {
        self.qubits.clear();
        self.joint_systems.clear();
        self.entangled_systems.clear();
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
}