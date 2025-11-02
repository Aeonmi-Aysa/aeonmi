use aeonmi_project::core::{
    compiler::Compiler,
    lexer::Lexer,
    lowering::lower_ast_to_ir,
    parser::Parser,
    vm::{Interpreter, Value},
};

fn temp_js_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("aeonmi_array_{name}.js"));
    path
}

#[test]
fn compiler_generates_js_for_arrays() {
    let code = r#"
        let data = [1, 2, 3];
        log(data[1]);
    "#;

    let output = temp_js_path("compile");
    if output.exists() {
        let _ = std::fs::remove_file(&output);
    }

    let compiler = Compiler::new();
    compiler
        .compile(code, output.to_str().unwrap())
        .expect("compilation should succeed");

    let generated = std::fs::read_to_string(&output).expect("output js exists");
    assert!(
        generated.contains("let data = [1, 2, 3];"),
        "missing array literal: {generated}"
    );
    assert!(
        generated.contains("log(data[1]);"),
        "missing index expression: {generated}"
    );

    let _ = std::fs::remove_file(output);
}

#[test]
fn vm_handles_array_indexing() {
    let code = "let data = [1, 2, 3]; let value = data[2];";

    let mut lexer = Lexer::from_str(code);
    let tokens = lexer.tokenize().expect("tokenization");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing");
    let module = lower_ast_to_ir(&ast, "array_vm").expect("lowering");

    let mut vm = Interpreter::new();
    vm.run_module(&module).expect("vm run");

    let value = vm.env.get("value").expect("value defined");
    match value {
        Value::Number(n) => assert_eq!(n, 3.0),
        other => panic!("unexpected value: {:?}", other),
    }
}
