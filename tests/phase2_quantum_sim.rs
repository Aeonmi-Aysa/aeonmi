/// Phase 2 — P2-8 / P2-9: Joint multi-qubit state-vector simulator tests.
///
/// Verifies that:
///   - CNOT on |+⟩|0⟩ produces the Bell state (|00⟩+|11⟩)/√2
///   - QUBE executor runs Bell-state circuits correctly
///   - `entangle()` in the VM prepares a real Bell state
///   - SWAP and CZ work on joint state vectors

use aeonmi_project::qube::executor::{JointState, QubitState, QubeExecutor, QubeEnv};
use aeonmi_project::qube::parser::QubeParser;
use aeonmi_project::qube::lexer::QubeLexer;

// ── JointState unit tests ─────────────────────────────────────────────────────

#[test]
fn test_joint_state_cnot_bell_state() {
    // Prepare |+⟩ ⊗ |0⟩  then apply CNOT.
    // Expected: Bell state (|00⟩+|11⟩)/√2  =>  amplitudes[0] ≈ 1/√2, amplitudes[3] ≈ 1/√2
    let q_plus = QubitState::plus();
    let q_zero = QubitState::zero();
    let js_ctrl = JointState::from_single("ctrl".to_string(), &q_plus);
    let js_tgt  = JointState::from_single("tgt".to_string(),  &q_zero);
    let mut joint = js_ctrl.tensor(js_tgt); // ctrl=bit0, tgt=bit1

    assert_eq!(joint.dim(), 4);

    joint.apply_cnot(0, 1); // CNOT(ctrl=bit0, tgt=bit1)

    let s = std::f64::consts::FRAC_1_SQRT_2;
    let amp00 = joint.amplitudes[0]; // |00⟩
    let amp11 = joint.amplitudes[3]; // |11⟩
    let amp01 = joint.amplitudes[1]; // |01⟩  (should be 0)
    let amp10 = joint.amplitudes[2]; // |10⟩  (should be 0)

    assert!((amp00.0 - s).abs() < 1e-10, "amp|00⟩.real ≈ 1/√2, got {}", amp00.0);
    assert!((amp11.0 - s).abs() < 1e-10, "amp|11⟩.real ≈ 1/√2, got {}", amp11.0);
    assert!(amp01.0.abs() < 1e-10, "amp|01⟩ ≈ 0, got {}", amp01.0);
    assert!(amp10.0.abs() < 1e-10, "amp|10⟩ ≈ 0, got {}", amp10.0);

    println!("✅ CNOT Bell state: |00⟩+|11⟩ amplitudes correct");
}

#[test]
fn test_joint_state_x_gate_flips_bit() {
    let q = QubitState::zero();
    let mut js = JointState::from_single("q".to_string(), &q);
    js.apply_x(0);
    // Should now be |1⟩: amplitudes[0]=0, amplitudes[1]=1
    assert!(js.amplitudes[0].0.abs() < 1e-10);
    assert!((js.amplitudes[1].0 - 1.0).abs() < 1e-10);
    println!("✅ X gate flips |0⟩ → |1⟩");
}

#[test]
fn test_joint_state_hadamard_creates_superposition() {
    let q = QubitState::zero();
    let mut js = JointState::from_single("q".to_string(), &q);
    js.apply_h(0);
    let s = std::f64::consts::FRAC_1_SQRT_2;
    assert!((js.amplitudes[0].0 - s).abs() < 1e-10);
    assert!((js.amplitudes[1].0 - s).abs() < 1e-10);
    println!("✅ H gate: |0⟩ → |+⟩");
}

#[test]
fn test_joint_state_swap() {
    // SWAP |01⟩ → |10⟩
    let q0 = QubitState::zero();
    let q1 = QubitState::one();
    let mut joint = JointState::from_single("q0".to_string(), &q0)
        .tensor(JointState::from_single("q1".to_string(), &q1));
    // |q1 q0⟩ encoding: bit0=q0, bit1=q1
    // State |01⟩ = q0=|0⟩, q1=|1⟩ → index = 0 + 1*2 = 2
    assert!((joint.amplitudes[2].0 - 1.0).abs() < 1e-10, "initial |01⟩ at index 2");

    joint.apply_swap(0, 1);
    // After SWAP: q0=|1⟩, q1=|0⟩ → index = 1 + 0*2 = 1
    assert!((joint.amplitudes[1].0 - 1.0).abs() < 1e-10, "after swap: |10⟩ at index 1");
    println!("✅ SWAP gate: |01⟩ → |10⟩");
}

#[test]
fn test_joint_state_cz() {
    // CZ|11⟩ → -|11⟩
    let q1a = QubitState::one();
    let q1b = QubitState::one();
    let mut joint = JointState::from_single("a".to_string(), &q1a)
        .tensor(JointState::from_single("b".to_string(), &q1b));
    // |11⟩ is at index 3 (bit0=1, bit1=1)
    assert!((joint.amplitudes[3].0 - 1.0).abs() < 1e-10, "initial |11⟩ at index 3");
    joint.apply_cz(0, 1);
    assert!((joint.amplitudes[3].0 + 1.0).abs() < 1e-10, "after CZ: amplitude flips sign");
    println!("✅ CZ gate: |11⟩ → -|11⟩");
}

#[test]
fn test_joint_state_measure_collapses() {
    // After measuring a Bell state, both qubits should have the same value.
    let q_plus = QubitState::plus();
    let q_zero = QubitState::zero();
    let mut joint = JointState::from_single("ctrl".to_string(), &q_plus)
        .tensor(JointState::from_single("tgt".to_string(), &q_zero));
    joint.apply_cnot(0, 1);

    // Measure qubit 0 (ctrl)
    let seed1 = 0x4AE0_4D5E_1337_BEEFu64;
    let bit0 = joint.measure(0, seed1);

    // After collapse the system is in |00⟩ or |11⟩ depending on bit0.
    // Measure qubit 1 (tgt) — must match.
    let bit1 = joint.measure(1, seed1.wrapping_add(1));
    assert_eq!(bit0, bit1, "Bell state: both qubits must collapse to same value");
    println!("✅ Bell state measurement: bits correlated ({} == {})", bit0, bit1);
}

// ── QUBE executor Bell-state test ─────────────────────────────────────────────

#[test]
fn test_qube_executor_bell_state() {
    // Using the exact syntax from examples/demo.qube (proper Unicode)
    let program = "state psi = |0\u{27E9}\nstate phi = |0\u{27E9}\napply H \u{2192} psi\napply CNOT \u{2192} (psi, phi)\ncollapse psi \u{2192} r1\ncollapse phi \u{2192} r2\nassert r1 \u{2208} {0, 1}\nassert r2 \u{2208} {0, 1}\n";
    let mut lexer = QubeLexer::new(program);
    let tokens = lexer.tokenize();
    let mut parser = QubeParser::new(tokens);
    let prog = parser.parse().expect("QUBE parse error");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("QUBE execution error");

    let r1 = *exec.env.classical.get("r1").expect("r1 should be in classical results");
    let r2 = *exec.env.classical.get("r2").expect("r2 should be in classical results");
    // After Bell circuit the two measurements must be equal (correlation)
    assert_eq!(r1, r2, "Bell state: r1 and r2 must be correlated");
    assert!(exec.env.assertions_failed == 0, "assertions should pass");
    println!("✅ QUBE Bell circuit: r1={} r2={} (correlated)", r1, r2);
    println!("{}", exec.circuit_diagram());
}

// ── QuantumSimulator joint-state tests (P2-9) ────────────────────────────────

#[test]
fn test_quantum_simulator_entangle_bell() {
    use aeonmi_project::core::quantum_simulator::QuantumSimulator;

    let mut sim = QuantumSimulator::new();
    sim.create_qubit("q0".to_string());
    sim.create_qubit("q1".to_string());

    // entangle() = H(q0) then CNOT(q0, q1)
    sim.entangle("q0", "q1").expect("entangle failed");

    // Both qubits should now be in a joint system
    assert!(sim.find_joint("q0").is_some(), "q0 should be in a joint system");
    assert!(sim.find_joint("q1").is_some(), "q1 should be in a joint system");
    assert_eq!(sim.find_joint("q0"), sim.find_joint("q1"), "q0 and q1 in same system");

    // Measure q0; measure q1; they must agree (Bell correlation)
    let b0 = sim.measure("q0").expect("measure q0");
    let b1 = sim.measure("q1").expect("measure q1");
    assert_eq!(b0, b1, "entangled qubits must collapse to same value");
    println!("✅ QuantumSimulator entangle: b0={} b1={} (correlated)", b0, b1);
}

#[test]
fn test_quantum_simulator_cnot_real_gate() {
    use aeonmi_project::core::quantum_simulator::QuantumSimulator;

    let mut sim = QuantumSimulator::new();
    sim.create_qubit("ctrl".to_string());
    sim.create_qubit("tgt".to_string());

    // Put ctrl in |1⟩
    sim.pauli_x("ctrl").unwrap();
    // CNOT(ctrl, tgt): tgt should flip to |1⟩
    sim.apply_cnot("ctrl", "tgt").unwrap();

    let bit_tgt = sim.measure("tgt").unwrap();
    assert_eq!(bit_tgt, 1, "CNOT(|1⟩, |0⟩) should give tgt=1");
    println!("✅ CNOT real gate: ctrl=|1⟩ → tgt flipped to 1");
}
