/// QUBE circuit-syntax integration tests.
///
/// Verifies that:
///   - The new circuit { } / qubit / measure -> syntax parses correctly.
///   - meta { } / execute { run } / expected { } blocks parse correctly.
///   - CircuitDef execution: qubits are declared, gates applied, measurements done.
///   - Built-in algorithms (grover, qft, shor) execute without error.
///   - The existing symbolic syntax (state / apply / collapse) still works.
///   - shard/qube/executor.qube and shard/qube/grammar.qube parse without error.

use aeonmi_project::qube::parser::QubeParser;
use aeonmi_project::qube::executor::QubeExecutor;
use aeonmi_project::qube::ast::{QubeStmt, CircuitStmt, QuantumGate};

// ── Parser tests ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_circuit_def_simple() {
    let src = r#"
circuit BellPair {
    qubit q0;
    qubit q1;
    H q0;
    CNOT q0 q1;
    measure q0 -> c0;
    measure q1 -> c1;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse circuit");
    assert_eq!(prog.stmts.len(), 1, "one top-level CircuitDef expected");
    match &prog.stmts[0] {
        QubeStmt::CircuitDef { name, body } => {
            assert_eq!(name, "BellPair");
            assert_eq!(body.len(), 6, "6 body statements");
            // Check qubit declarations
            assert!(matches!(&body[0], CircuitStmt::QubitDecl(n) if n == "q0"));
            assert!(matches!(&body[1], CircuitStmt::QubitDecl(n) if n == "q1"));
            // Check H gate
            assert!(matches!(&body[2], CircuitStmt::GateApply { gate: QuantumGate::H, .. }));
            // Check CNOT gate
            assert!(matches!(&body[3], CircuitStmt::GateApply { gate: QuantumGate::CNOT, .. }));
            // Check measures
            assert!(matches!(&body[4], CircuitStmt::Measure { qubit, classical } if qubit == "q0" && classical == "c0"));
            assert!(matches!(&body[5], CircuitStmt::Measure { qubit, classical } if qubit == "q1" && classical == "c1"));
        }
        other => panic!("Expected CircuitDef, got {:?}", other),
    }
}

#[test]
fn test_parse_meta_block() {
    let src = r#"
meta {
    name: TestProgram,
    version: 1,
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse meta");
    assert_eq!(prog.stmts.len(), 1);
    assert!(matches!(&prog.stmts[0], QubeStmt::MetaBlock { .. }));
    if let QubeStmt::MetaBlock { entries } = &prog.stmts[0] {
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "name");
        assert_eq!(entries[1].0, "version");
    }
}

#[test]
fn test_parse_execute_block() {
    let src = r#"
circuit Foo {
    qubit q0;
    H q0;
}
execute {
    run Foo;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse execute block");
    assert_eq!(prog.stmts.len(), 2, "CircuitDef + ExecuteBlock");
    assert!(matches!(&prog.stmts[0], QubeStmt::CircuitDef { name, .. } if name == "Foo"));
    match &prog.stmts[1] {
        QubeStmt::ExecuteBlock { steps } => {
            assert_eq!(steps.len(), 1);
            assert_eq!(steps[0], "Foo");
        }
        other => panic!("Expected ExecuteBlock, got {:?}", other),
    }
}

#[test]
fn test_parse_expected_block() {
    let src = r#"
expected {
    BellPair: {
        entangled: true,
    },
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse expected");
    assert_eq!(prog.stmts.len(), 1);
    assert!(matches!(&prog.stmts[0], QubeStmt::ExpectedBlock { .. }));
}

#[test]
fn test_parse_builtin_algorithms() {
    let src = r#"
circuit Grover {
    grover(16, 7);
    qft(4);
    shor(15);
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse builtin algos");
    assert_eq!(prog.stmts.len(), 1);
    if let QubeStmt::CircuitDef { body, .. } = &prog.stmts[0] {
        assert_eq!(body.len(), 3);
        assert!(matches!(&body[0], CircuitStmt::BuiltinAlgo { name, .. } if name == "grover"));
        assert!(matches!(&body[1], CircuitStmt::BuiltinAlgo { name, .. } if name == "qft"));
        assert!(matches!(&body[2], CircuitStmt::BuiltinAlgo { name, .. } if name == "shor"));
    }
}

#[test]
fn test_parse_if_classical() {
    let src = r#"
circuit Teleport {
    qubit q0;
    bit c0;
    H q0;
    measure q0 -> c0;
    if c0 {
        X q0;
    }
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse if stmt");
    if let QubeStmt::CircuitDef { body, .. } = &prog.stmts[0] {
        let if_stmt = body.last().expect("at least one body stmt");
        assert!(matches!(if_stmt, CircuitStmt::IfClassical { condition, .. } if condition == "c0"));
    }
}

#[test]
fn test_parse_reset_and_barrier() {
    let src = r#"
circuit ResetTest {
    qubit q0;
    qubit q1;
    reset q0;
    barrier q0 q1;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("should parse reset/barrier");
    if let QubeStmt::CircuitDef { body, .. } = &prog.stmts[0] {
        assert!(matches!(&body[2], CircuitStmt::Reset(n) if n == "q0"));
        assert!(matches!(&body[3], CircuitStmt::Barrier(qs) if qs.len() == 2));
    }
}

// ── Execution tests ───────────────────────────────────────────────────────────

#[test]
fn test_execute_bell_circuit() {
    let src = r#"
circuit BellPair {
    qubit q0;
    qubit q1;
    H q0;
    CNOT q0 q1;
    measure q0 -> c0;
    measure q1 -> c1;
}
execute {
    run BellPair;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    // Both measurements must have happened
    assert!(exec.env.classical.contains_key("c0"), "c0 should exist");
    assert!(exec.env.classical.contains_key("c1"), "c1 should exist");

    let c0 = exec.env.classical["c0"];
    let c1 = exec.env.classical["c1"];
    // With Bell state the results should be correlated (both 0 or both 1)
    assert_eq!(c0, c1, "Bell state: c0 and c1 must be correlated (got c0={} c1={})", c0, c1);

    println!("✅ Bell circuit: c0={} c1={} (correlated)", c0, c1);
}

#[test]
fn test_execute_grover_algo() {
    let src = r#"
circuit Grover4 {
    grover(16, 7);
}
execute {
    run Grover4;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    // grover(16, 7) should set grover_result = 7
    let result = exec.env.classical.get("grover_result").copied().unwrap_or(u64::MAX);
    assert_eq!(result, 7, "Grover's algorithm result should be the target");
    println!("✅ Grover circuit: found target={}", result);
}

#[test]
fn test_execute_shor_factoring() {
    let src = r#"
circuit Shor15 {
    shor(15);
}
execute {
    run Shor15;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    // shor(15) should factor 15 = 3 * 5
    let p = exec.env.classical.get("shor_p").copied().unwrap_or(0);
    let q = exec.env.classical.get("shor_q").copied().unwrap_or(0);
    assert_eq!(p * q, 15, "shor(15) factors must multiply to 15, got {} * {}", p, q);
    println!("✅ Shor circuit: 15 = {} × {}", p, q);
}

#[test]
fn test_execute_circuit_without_execute_block() {
    // CircuitDefs without an execute block are registered but not run immediately.
    let src = r#"
circuit IdleCircuit {
    qubit q0;
    H q0;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    // Circuit was registered but not run — qubits map is empty
    assert!(exec.env.qubits.is_empty(), "circuit not run, qubits should be empty");
}

#[test]
fn test_execute_explicit_run_circuit() {
    let src = r#"
circuit PrepQ {
    qubit q0;
    X q0;
}
execute {
    run PrepQ;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    let q0 = exec.env.qubits.get("q0").expect("q0 should exist");
    assert!(q0.prob_one() > 0.99, "X gate should flip q0 to |1⟩");
    println!("✅ X gate: P(1)={:.4}", q0.prob_one());
}

#[test]
fn test_circuit_diagram_includes_gate_names() {
    let src = r#"
circuit DiagramTest {
    qubit q0;
    H q0;
    X q0;
}
execute {
    run DiagramTest;
}
"#;
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("execute");

    let diagram = exec.circuit_diagram();
    assert!(diagram.contains("QUBE Circuit"));
    assert!(diagram.contains('H') || diagram.contains("DiagramTest"),
        "diagram should mention H or the circuit name");
    println!("✅ Circuit diagram:\n{}", diagram);
}

// ── Backward-compatibility: old symbolic syntax still works ───────────────────

#[test]
fn test_symbolic_syntax_still_works() {
    let src = "state ψ = |0⟩\napply H → ψ\ncollapse ψ → r\nassert r ∈ {0, 1}\nprint r";
    let mut p = QubeParser::from_str(src);
    let prog = p.parse().expect("symbolic syntax should still parse");
    assert_eq!(prog.stmts.len(), 5);

    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("symbolic execution should work");
    assert_eq!(exec.env.assertions_failed, 0, "assertion should pass");
}

// ── Integration test with shard files ─────────────────────────────────────────

#[test]
fn test_parse_shard_executor_qube() {
    let src = std::fs::read_to_string("shard/qube/executor.qube")
        .expect("shard/qube/executor.qube should exist");
    let mut p = QubeParser::from_str(&src);
    let prog = p.parse().expect("shard/qube/executor.qube should parse without error");
    assert!(!prog.stmts.is_empty(), "executor.qube should have statements");
    println!("✅ executor.qube: {} top-level statements", prog.stmts.len());
}

#[test]
fn test_run_shard_executor_qube() {
    let src = std::fs::read_to_string("shard/qube/executor.qube")
        .expect("shard/qube/executor.qube should exist");
    let mut p = QubeParser::from_str(&src);
    let prog = p.parse().expect("parse");
    let mut exec = QubeExecutor::new();
    exec.execute(&prog).expect("shard/qube/executor.qube should execute without error");
    println!("✅ executor.qube ran: {}", exec.summary());
}

#[test]
fn test_parse_shard_grammar_qube() {
    let src = std::fs::read_to_string("shard/qube/grammar.qube")
        .expect("shard/qube/grammar.qube should exist");
    let mut p = QubeParser::from_str(&src);
    let prog = p.parse().expect("shard/qube/grammar.qube should parse without error");
    assert!(!prog.stmts.is_empty(), "grammar.qube should have statements");
    println!("✅ grammar.qube: {} top-level statements", prog.stmts.len());
}
