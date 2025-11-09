use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser as AeParser;
use aeonmi_project::core::semantic_analyzer::{SemanticAnalyzer, Severity};

fn gather(source: &str) -> Vec<(String, Severity)> {
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize().expect("lex");
    let mut parser = AeParser::new(tokens);
    let ast = parser.parse().expect("parse");
    let mut sema = SemanticAnalyzer::new();
    sema.analyze_with_spans(&ast)
        .into_iter()
        .map(|d| (d.message, d.severity))
        .collect()
}

#[test]
fn quantum_op_arguments_require_qubits() {
    let src = r#"
let classical = 0;
superpose(classical);
"#;
    let diags = gather(src);
    assert!(
        diags.iter().any(|(m, s)| {
            matches!(s, Severity::Error)
                && m.contains("requires qubit arguments")
                && m.contains("⟨q⟩ ← |0⟩")
        }),
        "expected quantum op argument error, got: {:?}",
        diags
    );
}

#[test]
fn probability_branch_qubit_condition_errors() {
    let src = r#"
fn main() {
    ⟨q⟩ ← |0⟩;
    ⊖ q ⇒ { log(true); }
}
"#;
    let diags = gather(src);
    assert!(
        diags.iter().any(|(m, s)| {
            matches!(s, Severity::Error)
                && m.contains("Probability branch condition")
                && m.contains("measure(q)")
        }),
        "expected probability branch condition error, got: {:?}",
        diags
    );
}

#[test]
fn quantum_loop_qubit_condition_errors() {
    let src = r#"
fn main() {
    ⟨q⟩ ← |0⟩;
    ⟲ q ⇒ { log(true); }
}
"#;
    let diags = gather(src);
    assert!(
        diags.iter().any(|(m, s)| {
            matches!(s, Severity::Error)
                && m.contains("Quantum loop condition")
                && m.contains("measure(q)")
        }),
        "expected quantum loop condition error, got: {:?}",
        diags
    );
}

#[test]
fn undeclared_identifier_is_error() {
    let src = r#"
fn main() {
    let x = ghost + 1;
}
"#;
    let diags = gather(src);
    assert!(
        diags.iter().any(|(m, s)| {
            matches!(s, Severity::Error) && m.contains("Use of undeclared identifier 'ghost'")
        }),
        "expected undeclared identifier error, got: {:?}",
        diags
    );
}

#[test]
fn type_annotation_mismatch_reports_error() {
    let src = r#"
fn main() {
    let flag: bool = 42;
}
"#;
    let diags = gather(src);
    assert!(
        diags.iter().any(|(m, s)| {
            matches!(s, Severity::Error)
                && m.contains("annotated as bool")
                && m.contains("assigned number")
        }),
        "expected type annotation mismatch error, got: {:?}",
        diags
    );
}
