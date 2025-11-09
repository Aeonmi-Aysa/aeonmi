#![cfg(feature = "bytecode_vm")]
#![cfg(feature = "bytecode")]

use aeonmi_project::core::bytecode::BytecodeCompiler;
use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::vm_bytecode::{Value, VM};

fn eval(src: &str) -> Option<Value> {
    let mut lexer = Lexer::from_str(src);
    let tokens = lexer.tokenize().expect("lexer failure");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parser failure");
    let chunk = BytecodeCompiler::new().compile(&ast);
    let mut vm = VM::new(&chunk);
    vm.run()
}

#[test]
fn superpose_measure_yields_boolean() {
    let program = r#"
        let q = superpose(false);
        let result = measure(q);
        return result;
    "#;
    let value = eval(program).expect("vm produced value");
    match value {
        Value::Bool(_) => {}
        other => panic!("expected boolean outcome, got {:?}", other),
    }
}

#[test]
fn entangled_qubits_collapse_correlated() {
    let program = r#"
        let q1 = superpose(false);
        let q2 = superpose(false);
        entangle(q1, q2);
        let a = measure(q1);
        let b = measure(q2);
        return a == b;
    "#;
    let value = eval(program).expect("vm produced value");
    match value {
        Value::Bool(true) => {}
        other => panic!("expected correlated collapse, got {:?}", other),
    }
}
