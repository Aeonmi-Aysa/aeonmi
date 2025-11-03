#![cfg(feature = "bytecode")]
use aeonmi_project::core::bytecode::{disassemble, BytecodeCompiler};
use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;

fn compile_disasm(src: &str) -> String {
    let mut l = Lexer::from_str(src);
    let toks = l.tokenize().unwrap();
    let mut p = Parser::new(toks);
    let ast = p.parse().unwrap();
    let chunk = BytecodeCompiler::new().compile(&ast);
    disassemble(&chunk)
}

#[test]
fn disassembler_basic_layout() {
    let out = compile_disasm("fn add(a,b){ return a+b; } return add(1,2);");
    assert!(
        out.contains("== Bytecode Disassembly =="),
        "missing disassembly header"
    );
    assert!(out.contains("Constants:"), "missing constants section");
    assert!(out.contains("Functions:"), "missing functions section");
    assert!(
        out.lines().any(|l| l.contains("Call")),
        "expected Call opcode"
    );
    assert!(
        out.lines().any(|l| l.contains("Return")),
        "expected Return opcode"
    );
}
