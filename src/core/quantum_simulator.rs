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

/// Quantum simulator managing multiple qubits
#[derive(Debug)]
pub struct QuantumSimulator {
    pub qubits: HashMap<String, QuantumState>,
    pub entangled_systems: Vec<Vec<String>>, // Track entangled qubit groups
    /// Joint state-vector systems for multi-qubit operations (P2-8).
    /// Each entry owns a group of qubits as a single state vector.
    pub joint_systems: Vec<JointSystem>,
}

/// Joint state-vector for N named qubits in the VM simulator (P2-8).
///
/// `qubit_order[k]` = qubit k; `(state_index >> k) & 1` gives its value.
/// `amplitudes[i]` = (real, imag) for basis state |i⟩.
#[derive(Debug, Clone)]
pub struct JointSystem {
    pub qubit_order: Vec<String>,
    pub amplitudes:  Vec<Complex>,
}

impl JointSystem {
    /// Create a 1-qubit joint system from an individual `QuantumState`.
    pub fn from_single(name: String, qs: &QuantumState) -> Self {
        let a0 = qs.amplitudes[0];
        let a1 = qs.amplitudes[1];
        Self {
            qubit_order: vec![name],
            amplitudes:  vec![a0, a1],
        }
    }

    pub fn n(&self)   -> usize { self.qubit_order.len() }
    pub fn dim(&self) -> usize { 1 << self.n() }

    pub fn pos_of(&self, name: &str) -> Option<usize> {
        self.qubit_order.iter().position(|q| q == name)
    }

    /// Tensor product: `self ⊗ other`.  `self` qubits are LSBs.
    pub fn tensor(self, other: JointSystem) -> Self {
        let dim_a = self.dim();
        let dim_b = other.dim();
        let mut amplitudes = vec![Complex::new(0.0, 0.0); dim_a * dim_b];
        for (b_idx, b_amp) in other.amplitudes.iter().enumerate() {
            for (a_idx, a_amp) in self.amplitudes.iter().enumerate() {
                let out = a_idx | (b_idx << self.n());
                amplitudes[out] = Complex::new(
                    a_amp.real * b_amp.real - a_amp.imag * b_amp.imag,
                    a_amp.real * b_amp.imag + a_amp.imag * b_amp.real,
                );
            }
        }
        let mut qubit_order = self.qubit_order;
        qubit_order.extend(other.qubit_order);
        Self { qubit_order, amplitudes }
    }

    /// Apply CNOT: if ctrl bit = 1, flip tgt bit (P2-9).
    pub fn apply_cnot(&mut self, ctrl_pos: usize, tgt_pos: usize) {
        let ctrl_mask = 1usize << ctrl_pos;
        let tgt_mask  = 1usize << tgt_pos;
        let n = self.dim();
        let mut new_amps = vec![Complex::new(0.0, 0.0); n];
        for i in 0..n {
            let out = if (i & ctrl_mask) != 0 { i ^ tgt_mask } else { i };
            new_amps[out] = self.amplitudes[i];
        }
        self.amplitudes = new_amps;
    }

    /// Apply Hadamard to qubit at `pos`.
    pub fn apply_hadamard(&mut self, pos: usize) {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let mask = 1usize << pos;
        let n    = self.dim();
        let mut new_amps = self.amplitudes.clone();
        for i in 0..n {
            if (i & mask) == 0 {
                let j = i | mask;
                let a0 = self.amplitudes[i];
                let a1 = self.amplitudes[j];
                new_amps[i] = Complex::new((a0.real + a1.real) * s, (a0.imag + a1.imag) * s);
                new_amps[j] = Complex::new((a0.real - a1.real) * s, (a0.imag - a1.imag) * s);
            }
        }
        self.amplitudes = new_amps;
    }

    /// Measure qubit at `pos`, Born-rule collapse, return 0 or 1.
    pub fn measure(&mut self, pos: usize) -> u8 {
        let mask = 1usize << pos;
        let n    = self.dim();
        let prob_zero: f64 = (0..n)
            .filter(|i| (i & mask) == 0)
            .map(|i| self.amplitudes[i].magnitude_squared())
            .sum();
        // Use a simple deterministic seed derived from current amplitudes
        let raw: u64 = self.amplitudes.iter().enumerate()
            .fold(0x4AE0_4D5E_1337_BEEFu64, |acc, (i, a)| {
                acc.wrapping_add((i as u64).wrapping_mul(
                    (a.real * 1e9) as u64 ^ (a.imag.abs() * 1e9) as u64,
                ))
            });
        let seed = raw.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let r = (seed >> 33) as f64 / u32::MAX as f64;
        let outcome: usize = if r < prob_zero { 0 } else { 1 };
        // Collapse
        let norm_sq: f64 = (0..n)
            .filter(|i| ((i >> pos) & 1) == outcome)
            .map(|i| self.amplitudes[i].magnitude_squared())
            .sum();
        let norm = norm_sq.sqrt().max(1e-14);
        for i in 0..n {
            if ((i >> pos) & 1) != outcome {
                self.amplitudes[i] = Complex::new(0.0, 0.0);
            } else {
                self.amplitudes[i] = Complex::new(
                    self.amplitudes[i].real / norm,
                    self.amplitudes[i].imag / norm,
                );
            }
        }
        outcome as u8
    }

    /// Extract marginal `QuantumState` for qubit at `pos`.
    pub fn marginal(&self, pos: usize) -> QuantumState {
        let mask = 1usize << pos;
        let n    = self.dim();
        let prob_zero: f64 = (0..n)
            .filter(|i| (i & mask) == 0)
            .map(|i| self.amplitudes[i].magnitude_squared())
            .sum();
        let p0 = prob_zero.max(0.0).min(1.0);
        let p1 = 1.0 - p0;
        let mut qs = QuantumState::new(1);
        qs.amplitudes[0] = Complex::real(p0.sqrt());
        qs.amplitudes[1] = Complex::real(p1.sqrt());
        qs
    }
}

impl QuantumSimulator {
    pub fn new() -> Self {
        Self {
            qubits: HashMap::new(),
            entangled_systems: Vec::new(),
            joint_systems: Vec::new(),
        }
    }
    
    /// Create a new qubit in |0⟩ state
    pub fn create_qubit(&mut self, name: String) {
        self.qubits.insert(name, QuantumState::new(1));
    }
    
    /// Apply superposition (Hadamard) to a qubit
    pub fn superpose(&mut self, qubit_name: &str) -> Result<()> {
        // If qubit is in a joint system, apply Hadamard there
        if let Some(idx) = self.find_joint(qubit_name) {
            let pos = self.joint_systems[idx].pos_of(qubit_name).unwrap();
            self.joint_systems[idx].apply_hadamard(pos);
            let marg = self.joint_systems[idx].marginal(pos);
            if let Some(q) = self.qubits.get_mut(qubit_name) { *q = marg; }
            return Ok(());
        }
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::hadamard(state, 0)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }
    
    /// Apply Pauli-X gate to a qubit
    pub fn pauli_x(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::pauli_x(state, 0)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }
    
    /// Apply Pauli-Y gate to a qubit
    pub fn pauli_y(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::pauli_y(state, 0)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }
    
    /// Apply Pauli-Z gate to a qubit  
    pub fn pauli_z(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            QuantumGates::pauli_z(state, 0)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }
    
    /// Apply S gate (phase gate, Z^1/2) to a qubit: multiplies |1⟩ amplitude by i
    pub fn phase_s(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // S gate: |0⟩ → |0⟩, |1⟩ → i|1⟩
            // For single qubit state: amplitudes[0] = |0⟩, amplitudes[1] = |1⟩
            let a1 = state.amplitudes[1];
            state.amplitudes[1] = Complex::new(-a1.imag, a1.real); // multiply by i
            Ok(())
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Apply T gate (π/8 gate, Z^1/4) to a qubit: multiplies |1⟩ amplitude by e^(iπ/4)
    pub fn phase_t(&mut self, qubit_name: &str) -> Result<()> {
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            // T gate: |0⟩ → |0⟩, |1⟩ → e^(iπ/4)|1⟩
            let a1 = state.amplitudes[1];
            let cos45 = std::f64::consts::FRAC_1_SQRT_2;
            let sin45 = std::f64::consts::FRAC_1_SQRT_2;
            state.amplitudes[1] = Complex::new(
                a1.real * cos45 - a1.imag * sin45,
                a1.real * sin45 + a1.imag * cos45,
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    // ── Joint system helpers (P2-8) ───────────────────────────────────────────

    /// Return the index of the joint system that owns `name`, if any.
    pub fn find_joint(&self, name: &str) -> Option<usize> {
        self.joint_systems.iter().position(|js| js.pos_of(name).is_some())
    }

    /// Ensure `ctrl` and `tgt` are in the same joint system, creating or merging
    /// as needed.  Returns the index of the (possibly new) joint system.
    pub fn ensure_joint_pair(&mut self, ctrl: &str, tgt: &str) -> Result<usize> {
        let loc_ctrl = self.find_joint(ctrl);
        let loc_tgt  = self.find_joint(tgt);

        match (loc_ctrl, loc_tgt) {
            (Some(i), Some(j)) if i == j => Ok(i),
            (Some(i), Some(j)) => {
                // Merge two separate joint systems
                let (keep, remove) = if i < j { (i, j) } else { (j, i) };
                let removed = self.joint_systems.remove(remove);
                let kept    = self.joint_systems.remove(keep);
                let merged  = kept.tensor(removed);
                self.joint_systems.push(merged);
                Ok(self.joint_systems.len() - 1)
            }
            (Some(i), None) => {
                let qs_tgt = self.qubits.get(tgt)
                    .ok_or_else(|| anyhow::anyhow!("Qubit '{}' not found", tgt))?
                    .clone();
                let js_tgt = JointSystem::from_single(tgt.to_string(), &qs_tgt);
                let js = self.joint_systems.remove(i);
                let merged = js.tensor(js_tgt);
                self.joint_systems.push(merged);
                Ok(self.joint_systems.len() - 1)
            }
            (None, Some(j)) => {
                let qs_ctrl = self.qubits.get(ctrl)
                    .ok_or_else(|| anyhow::anyhow!("Qubit '{}' not found", ctrl))?
                    .clone();
                let js_ctrl = JointSystem::from_single(ctrl.to_string(), &qs_ctrl);
                let js = self.joint_systems.remove(j);
                let merged = js_ctrl.tensor(js);
                self.joint_systems.push(merged);
                Ok(self.joint_systems.len() - 1)
            }
            (None, None) => {
                let qs_ctrl = self.qubits.get(ctrl)
                    .ok_or_else(|| anyhow::anyhow!("Qubit '{}' not found", ctrl))?
                    .clone();
                let qs_tgt = self.qubits.get(tgt)
                    .ok_or_else(|| anyhow::anyhow!("Qubit '{}' not found", tgt))?
                    .clone();
                let js_ctrl = JointSystem::from_single(ctrl.to_string(), &qs_ctrl);
                let js_tgt  = JointSystem::from_single(tgt.to_string(),  &qs_tgt);
                self.joint_systems.push(js_ctrl.tensor(js_tgt));
                Ok(self.joint_systems.len() - 1)
            }
        }
    }

    /// Apply CNOT using the joint state-vector (P2-9 — real CNOT, not approximation).
    pub fn apply_cnot(&mut self, control_name: &str, target_name: &str) -> Result<()> {
        if !self.qubits.contains_key(control_name) {
            return Err(anyhow::anyhow!("Control qubit '{}' not found", control_name));
        }
        if !self.qubits.contains_key(target_name) {
            return Err(anyhow::anyhow!("Target qubit '{}' not found", target_name));
        }
        let sys_idx = self.ensure_joint_pair(control_name, target_name)?;
        let ctrl_pos = self.joint_systems[sys_idx].pos_of(control_name).unwrap();
        let tgt_pos  = self.joint_systems[sys_idx].pos_of(target_name).unwrap();
        self.joint_systems[sys_idx].apply_cnot(ctrl_pos, tgt_pos);
        // Sync marginals back to individual qubit display states
        let mc = self.joint_systems[sys_idx].marginal(ctrl_pos);
        let mt = self.joint_systems[sys_idx].marginal(tgt_pos);
        if let Some(q) = self.qubits.get_mut(control_name) { *q = mc; }
        if let Some(q) = self.qubits.get_mut(target_name)  { *q = mt; }
        // Update metadata tracking
        self.track_entanglement(control_name, target_name);
        Ok(())
    }

    /// Measure a qubit, collapsing its state.
    /// If the qubit is in a joint system, performs a joint-state measurement (P2-8).
    pub fn measure(&mut self, qubit_name: &str) -> Result<u8> {
        if let Some(sys_idx) = self.find_joint(qubit_name) {
            let pos = self.joint_systems[sys_idx].pos_of(qubit_name).unwrap();
            let bit = self.joint_systems[sys_idx].measure(pos);
            let marg = self.joint_systems[sys_idx].marginal(pos);
            if let Some(q) = self.qubits.get_mut(qubit_name) { *q = marg; }
            return Ok(bit);
        }
        if let Some(state) = self.qubits.get_mut(qubit_name) {
            Ok(state.measure() as u8)
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }
    
    /// Get the probability of measuring |0⟩ for a qubit
    pub fn get_zero_probability(&self, qubit_name: &str) -> Result<f64> {
        if let Some(state) = self.qubits.get(qubit_name) {
            Ok(state.get_probability(0))
        } else {
            Err(anyhow::anyhow!("Qubit '{}' not found", qubit_name))
        }
    }

    /// Update the metadata entanglement tracking (groups qubit names for display).
    fn track_entanglement(&mut self, qubit1: &str, qubit2: &str) {
        let mut found = false;
        for system in self.entangled_systems.iter_mut() {
            if system.contains(&qubit1.to_string()) || system.contains(&qubit2.to_string()) {
                if !system.contains(&qubit1.to_string()) { system.push(qubit1.to_string()); }
                if !system.contains(&qubit2.to_string()) { system.push(qubit2.to_string()); }
                found = true;
                break;
            }
        }
        if !found {
            self.entangled_systems.push(vec![qubit1.to_string(), qubit2.to_string()]);
        }
    }
    
    /// Create entanglement between two qubits.
    ///
    /// Performs a Bell-state preparation: applies H to `qubit1`, then CNOT(qubit1, qubit2).
    /// This is the real joint state-vector entanglement (P2-9).
    pub fn entangle(&mut self, qubit1: &str, qubit2: &str) -> Result<()> {
        if !self.qubits.contains_key(qubit1) {
            return Err(anyhow::anyhow!("Qubit '{}' not found", qubit1));
        }
        if !self.qubits.contains_key(qubit2) {
            return Err(anyhow::anyhow!("Qubit '{}' not found", qubit2));
        }
        // Step 1: Apply Hadamard to qubit1 to put it in superposition
        self.superpose(qubit1)?;
        // Step 2: Apply CNOT(qubit1, qubit2) using the joint state-vector
        self.apply_cnot(qubit1, qubit2)?;
        Ok(())
    }
    
    /// Reset simulator state
    pub fn reset(&mut self) {
        self.qubits.clear();
        self.entangled_systems.clear();
        self.joint_systems.clear();
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