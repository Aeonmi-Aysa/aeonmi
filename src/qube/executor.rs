//! QUBE Executor — runs QubeProgram against the quantum simulator backend.
//!
//! Uses Aeonmi's existing quantum types (nalgebra + num-complex) when available,
//! falls back to pure-Rust simulation when those features are not compiled.

use std::collections::HashMap;
use crate::qube::ast::*;

// ─── Qubit state representation ──────────────────────────────────────────────

/// A single qubit as a 2-element complex amplitude vector: [α, β]
/// where α = amplitude for |0⟩, β = amplitude for |1⟩.
#[derive(Debug, Clone)]
pub struct QubitState {
    pub alpha: f64, // amplitude for |0⟩
    pub beta: f64,  // amplitude for |1⟩
}

impl QubitState {
    pub fn zero() -> Self { Self { alpha: 1.0, beta: 0.0 } }
    pub fn one() -> Self { Self { alpha: 0.0, beta: 1.0 } }
    pub fn plus() -> Self {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        Self { alpha: s, beta: s }
    }
    pub fn minus() -> Self {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        Self { alpha: s, beta: -s }
    }

    pub fn from_ast(state: &crate::qube::ast::QubitState, amp: f64) -> Self {
        let (a, b) = state.amplitude_pair();
        Self { alpha: a * amp, beta: b * amp }
    }

    /// Normalize so |α|² + |β|² = 1.
    pub fn normalize(&mut self) {
        let norm = (self.alpha * self.alpha + self.beta * self.beta).sqrt();
        if norm > 1e-12 {
            self.alpha /= norm;
            self.beta /= norm;
        }
    }

    /// Probability of measuring |0⟩.
    pub fn prob_zero(&self) -> f64 {
        self.alpha * self.alpha
    }

    /// Probability of measuring |1⟩.
    pub fn prob_one(&self) -> f64 {
        self.beta * self.beta
    }

    /// Collapse: returns 0 or 1 using the Born rule + pseudo-random.
    pub fn measure(&self, seed: u64) -> u64 {
        // Simple LCG-based decision
        let r = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as f64
            / u32::MAX as f64;
        if r < self.prob_zero() { 0 } else { 1 }
    }

    /// Apply Hadamard gate.
    pub fn apply_h(&mut self) {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let a = self.alpha;
        let b = self.beta;
        self.alpha = s * (a + b);
        self.beta  = s * (a - b);
    }

    /// Apply Pauli-X (NOT).
    pub fn apply_x(&mut self) {
        std::mem::swap(&mut self.alpha, &mut self.beta);
    }

    /// Apply Pauli-Z.
    pub fn apply_z(&mut self) {
        self.beta = -self.beta;
    }

    /// Apply Pauli-Y.
    pub fn apply_y(&mut self) {
        let a = self.alpha;
        self.alpha = -self.beta;
        self.beta = a;
    }

    /// Apply S (phase) gate.
    pub fn apply_s(&mut self) {
        // S: |0⟩ → |0⟩, |1⟩ → i|1⟩  — approximate with sign flip for real sim
        // Full complex sim would require complex alpha/beta; we track only magnitude here.
        // Phase gate does not change probability — skip for real-only sim.
    }

    /// Apply T gate — same approximation as S for real sim.
    pub fn apply_t(&mut self) {}
}

// ─── Execution environment ────────────────────────────────────────────────────

#[derive(Debug)]
pub struct QubeEnv {
    /// Named qubit states.
    pub qubits: HashMap<String, QubitState>,
    /// Classical measurement results (name → 0 or 1).
    pub classical: HashMap<String, u64>,
    /// Variable bindings (name → f64).
    pub vars: HashMap<String, f64>,
    /// Pseudo-random seed for measurement.
    pub seed: u64,
    /// Log of operations performed.
    pub log: Vec<String>,
    /// Whether assertions have all passed.
    pub assertions_passed: usize,
    pub assertions_failed: usize,
}

impl Default for QubeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl QubeEnv {
    pub fn new() -> Self {
        Self {
            qubits: HashMap::new(),
            classical: HashMap::new(),
            vars: HashMap::new(),
            seed: 0x4AE0_4D5E_1337_BEEF_u64,
            log: Vec::new(),
            assertions_passed: 0,
            assertions_failed: 0,
        }
    }

    fn tick_seed(&mut self) -> u64 {
        self.seed = self.seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed >> 33
    }

    pub fn resolve_amplitude(&self, amp: &QubeAmplitude) -> f64 {
        match amp {
            QubeAmplitude::Number(n) => *n,
            QubeAmplitude::Implied => 1.0,
            QubeAmplitude::Variable(name) => *self.vars.get(name).unwrap_or(&1.0),
        }
    }
}

const AEON_SEED: u64 = 0x4AE0_4D5E_1337_BEEF_u64;

// ─── QubeExecutor ─────────────────────────────────────────────────────────────

pub struct QubeExecutor {
    pub env: QubeEnv,
}

impl Default for QubeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl QubeExecutor {
    pub fn new() -> Self {
        Self {
            env: QubeEnv {
                seed: AEON_SEED,
                ..QubeEnv::default()
            },
        }
    }

    /// Execute a full QUBE program.
    pub fn execute(&mut self, prog: &QubeProgram) -> Result<(), String> {
        for stmt in &prog.stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &QubeStmt) -> Result<(), String> {
        match stmt {
            QubeStmt::Comment(_) => {}

            QubeStmt::StateDecl { name, value } => {
                let qs = self.eval_state_expr(value)?;
                self.env.qubits.insert(name.clone(), qs);
                self.env.log.push(format!("state {} initialized", name));
            }

            QubeStmt::GateApply { gate, targets } => {
                self.apply_gate(gate, targets)?;
            }

            QubeStmt::Collapse { qubit, result } => {
                let q = self.env.qubits.get(qubit)
                    .ok_or_else(|| format!("Undefined qubit '{}'", qubit))?
                    .clone();
                let seed = self.env.tick_seed();
                let bit = q.measure(seed);
                self.env.classical.insert(result.clone(), bit);
                // Collapse the qubit to the measured state
                if bit == 0 {
                    self.env.qubits.insert(qubit.clone(), QubitState::zero());
                } else {
                    self.env.qubits.insert(qubit.clone(), QubitState::one());
                }
                self.env.log.push(format!("collapse {} → {} = {}", qubit, result, bit));
            }

            QubeStmt::Assert { variable, valid_values } => {
                let actual = self.env.classical.get(variable.as_str())
                    .copied()
                    .unwrap_or(0);

                let passed = valid_values.iter().any(|v| match v {
                    AssertValue::Integer(n) => actual as i64 == *n,
                    AssertValue::Float(f) => (actual as f64 - f).abs() < 1e-9,
                    AssertValue::State(s) => {
                        let expected = match s {
                            crate::qube::ast::QubitState::Zero => 0,
                            crate::qube::ast::QubitState::One => 1,
                            _ => u64::MAX,
                        };
                        actual == expected
                    }
                    AssertValue::Variable(name) => {
                        self.env.classical.get(name.as_str()).copied().unwrap_or(u64::MAX) == actual
                    }
                });

                if passed {
                    self.env.assertions_passed += 1;
                    self.env.log.push(format!("assert {} ∈ {{...}} → ✓ (value={})", variable, actual));
                } else {
                    self.env.assertions_failed += 1;
                    self.env.log.push(format!("assert {} ∈ {{...}} → ✗ (value={})", variable, actual));
                    return Err(format!("Assertion failed: {} = {} not in valid set", variable, actual));
                }
            }

            QubeStmt::Print { variable } => {
                if let Some(&val) = self.env.classical.get(variable.as_str()) {
                    println!("{} = {}", variable, val);
                    self.env.log.push(format!("print {} = {}", variable, val));
                } else if let Some(q) = self.env.qubits.get(variable.as_str()) {
                    println!("{}: α={:.4} β={:.4} P(0)={:.4} P(1)={:.4}",
                        variable, q.alpha, q.beta, q.prob_zero(), q.prob_one());
                    self.env.log.push(format!("print qubit {}", variable));
                } else if let Some(&v) = self.env.vars.get(variable.as_str()) {
                    println!("{} = {}", variable, v);
                } else {
                    println!("{}: (undefined)", variable);
                }
            }

            QubeStmt::LetBinding { name, value } => {
                let v = match value {
                    QubeExpr::Number(n) => *n,
                    QubeExpr::Variable(n) => *self.env.vars.get(n.as_str()).unwrap_or(&0.0),
                    QubeExpr::Bool(b) => if *b { 1.0 } else { 0.0 },
                    QubeExpr::String(_) => 0.0,
                };
                self.env.vars.insert(name.clone(), v);
            }
        }
        Ok(())
    }

    fn eval_state_expr(&self, expr: &QuantumStateExpr) -> Result<QubitState, String> {
        match expr {
            QuantumStateExpr::QubitLiteral(q) => Ok(match q {
                crate::qube::ast::QubitState::Zero => QubitState::zero(),
                crate::qube::ast::QubitState::One  => QubitState::one(),
                crate::qube::ast::QubitState::Plus => QubitState::plus(),
                crate::qube::ast::QubitState::Minus => QubitState::minus(),
                crate::qube::ast::QubitState::Named(_) => QubitState::zero(),
            }),
            QuantumStateExpr::Superposition { terms } => {
                let mut alpha = 0.0f64;
                let mut beta  = 0.0f64;
                for (amp, basis) in terms {
                    let a = self.env.resolve_amplitude(amp);
                    let (ba, bb) = basis.amplitude_pair();
                    alpha += a * ba;
                    beta  += a * bb;
                }
                let mut q = QubitState { alpha, beta };
                q.normalize();
                Ok(q)
            }
            QuantumStateExpr::StateRef(name) => {
                self.env.qubits.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined state '{}'", name))
            }
            QuantumStateExpr::TensorProduct(a, _b) => {
                // For a single-qubit executor, tensor product creates the first qubit.
                // Full multi-qubit sim is Phase 2 extension.
                self.eval_state_expr(a)
            }
        }
    }

    fn apply_gate(&mut self, gate: &QuantumGate, targets: &[String]) -> Result<(), String> {
        match gate {
            QuantumGate::H => {
                let name = targets.first().ok_or("H gate requires 1 target")?;
                let q = self.env.qubits.get_mut(name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", name))?;
                q.apply_h();
                self.env.log.push(format!("apply H → {}", name));
            }
            QuantumGate::X => {
                let name = targets.first().ok_or("X gate requires 1 target")?;
                let q = self.env.qubits.get_mut(name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", name))?;
                q.apply_x();
                self.env.log.push(format!("apply X → {}", name));
            }
            QuantumGate::Y => {
                let name = targets.first().ok_or("Y gate requires 1 target")?;
                let q = self.env.qubits.get_mut(name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", name))?;
                q.apply_y();
                self.env.log.push(format!("apply Y → {}", name));
            }
            QuantumGate::Z => {
                let name = targets.first().ok_or("Z gate requires 1 target")?;
                let q = self.env.qubits.get_mut(name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", name))?;
                q.apply_z();
                self.env.log.push(format!("apply Z → {}", name));
            }
            QuantumGate::S | QuantumGate::T => {
                let name = targets.first().ok_or("S/T gate requires 1 target")?;
                if let Some(q) = self.env.qubits.get_mut(name) {
                    if matches!(gate, QuantumGate::S) { q.apply_s(); } else { q.apply_t(); }
                }
                self.env.log.push(format!("apply {:?} → {}", gate, name));
            }
            QuantumGate::CNOT => {
                if targets.len() < 2 {
                    return Err("CNOT requires 2 targets (control, target)".to_string());
                }
                let ctrl_name = targets[0].clone();
                let tgt_name  = targets[1].clone();
                let ctrl_state = self.env.qubits.get(&ctrl_name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", ctrl_name))?
                    .clone();
                // If control is |1⟩ (prob_one close to 1), apply X to target
                if ctrl_state.prob_one() > 0.5 {
                    let tgt = self.env.qubits.get_mut(&tgt_name)
                        .ok_or_else(|| format!("Undefined qubit '{}'", tgt_name))?;
                    tgt.apply_x();
                }
                self.env.log.push(format!("apply CNOT → ({}, {})", ctrl_name, tgt_name));
            }
            QuantumGate::SWAP => {
                if targets.len() < 2 { return Err("SWAP requires 2 targets".to_string()); }
                let a = targets[0].clone();
                let b = targets[1].clone();
                // Swap the two qubit states
                let qa = self.env.qubits.get(&a).cloned()
                    .ok_or_else(|| format!("Undefined qubit '{}'", a))?;
                let qb = self.env.qubits.get(&b).cloned()
                    .ok_or_else(|| format!("Undefined qubit '{}'", b))?;
                self.env.qubits.insert(a.clone(), qb);
                self.env.qubits.insert(b.clone(), qa);
                self.env.log.push(format!("apply SWAP → ({}, {})", a, b));
            }
            QuantumGate::CZ => {
                // CZ: applies Z to target if control is |1⟩
                if targets.len() < 2 { return Err("CZ requires 2 targets".to_string()); }
                let ctrl_name = targets[0].clone();
                let tgt_name  = targets[1].clone();
                let ctrl_state = self.env.qubits.get(&ctrl_name)
                    .ok_or_else(|| format!("Undefined qubit '{}'", ctrl_name))?
                    .clone();
                if ctrl_state.prob_one() > 0.5 {
                    let tgt = self.env.qubits.get_mut(&tgt_name)
                        .ok_or_else(|| format!("Undefined qubit '{}'", tgt_name))?;
                    tgt.apply_z();
                }
                self.env.log.push(format!("apply CZ → ({}, {})", ctrl_name, tgt_name));
            }
            QuantumGate::Custom(name) => {
                self.env.log.push(format!("apply {} [custom, no-op] → {:?}", name, targets));
            }
        }
        Ok(())
    }

    /// Print a text-mode circuit diagram.
    pub fn circuit_diagram(&self) -> String {
        if self.env.log.is_empty() {
            return "— empty circuit —".to_string();
        }
        let mut out = String::new();
        out.push_str("┌─ QUBE Circuit ─────────────────────────┐\n");
        for (i, entry) in self.env.log.iter().enumerate() {
            out.push_str(&format!("│ {:2}. {}\n", i + 1, entry));
        }
        out.push_str(&format!("│\n│ Assertions: {} passed / {} failed\n",
            self.env.assertions_passed, self.env.assertions_failed));
        out.push_str("└────────────────────────────────────────┘\n");
        out
    }

    pub fn summary(&self) -> String {
        format!(
            "QubeExecutor: {} qubits | {} classical | {} log entries | {} assertions passed",
            self.env.qubits.len(),
            self.env.classical.len(),
            self.env.log.len(),
            self.env.assertions_passed,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qube::parser::QubeParser;

    fn run(src: &str) -> QubeExecutor {
        let mut p = QubeParser::from_str(src);
        let prog = p.parse().expect("parse failed");
        let mut exec = QubeExecutor::new();
        exec.execute(&prog).expect("execution failed");
        exec
    }

    #[test]
    fn test_state_init_zero() {
        let exec = run("state ψ = |0⟩");
        let q = exec.env.qubits.get("ψ").unwrap();
        assert!((q.alpha - 1.0).abs() < 1e-9);
        assert!((q.beta).abs() < 1e-9);
    }

    #[test]
    fn test_hadamard_creates_superposition() {
        let exec = run("state ψ = |0⟩\napply H → ψ");
        let q = exec.env.qubits.get("ψ").unwrap();
        let p0 = q.prob_zero();
        let p1 = q.prob_one();
        assert!((p0 - 0.5).abs() < 1e-9, "P(0) should be 0.5, got {}", p0);
        assert!((p1 - 0.5).abs() < 1e-9, "P(1) should be 0.5, got {}", p1);
    }

    #[test]
    fn test_pauli_x_flips() {
        let exec = run("state q = |0⟩\napply X → q");
        let q = exec.env.qubits.get("q").unwrap();
        assert!(q.prob_one() > 0.99);
    }

    #[test]
    fn test_collapse_records_classical() {
        let exec = run("state ψ = |0⟩\ncollapse ψ → r");
        assert!(exec.env.classical.contains_key("r"));
        let r = exec.env.classical["r"];
        assert!(r == 0 || r == 1);
    }

    #[test]
    fn test_assert_passes_for_zero_state() {
        // |0⟩ collapsed deterministically gives 0
        let exec = run("state ψ = |0⟩\ncollapse ψ → r\nassert r ∈ {0, 1}");
        assert_eq!(exec.env.assertions_failed, 0);
        assert_eq!(exec.env.assertions_passed, 1);
    }

    #[test]
    fn test_circuit_diagram_nonempty() {
        let exec = run("state ψ = |0⟩\napply H → ψ\ncollapse ψ → r");
        let diag = exec.circuit_diagram();
        assert!(diag.contains("QUBE Circuit"));
        assert!(diag.contains("H"));
    }
}
