// Phase 4 native VM integration tests:
// - P1-33: f-string interpolation
// - P1-34: for x in collection loop
// - G-1/G-5: ⧉ genesis array literal
// - G-3/G-6: … spread inside genesis array
// - G-4/G-8: ↦ binding projection

use aeonmi_project::core::{
    lexer::Lexer,
    lowering::lower_ast_to_ir,
    parser::Parser,
    vm::{Interpreter, Value},
};

/// Run source, call `main()`, and return its value.
fn run_val(src: &str) -> Value {
    let mut lex = Lexer::from_str(src);
    let tokens = lex.tokenize().expect("lex");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parse");
    let module = lower_ast_to_ir(&ast, "test").expect("lower");
    let mut interp = Interpreter::new();
    // run_module loads decls and calls main() if present but discards return value.
    // We run manually to capture the return value.
    interp.run_module_return(&module).expect("run_module_return")
}

// ── P1-33: f-string interpolation ────────────────────────────────────────────

#[test]
fn fstring_simple_variable() {
    let v = run_val(r#"
        let x = 42;
        return f"value is {x}";
    "#);
    match v {
        Value::String(s) => assert_eq!(s, "value is 42"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn fstring_expression_interpolation() {
    let v = run_val(r#"
        let a = 3;
        let b = 4;
        return f"{a} + {b} = {a + b}";
    "#);
    match v {
        Value::String(s) => assert_eq!(s, "3 + 4 = 7"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn fstring_no_interpolation() {
    let v = run_val(r#"return f"plain text";"#);
    match v {
        Value::String(s) => assert_eq!(s, "plain text"),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── P1-34: for x in collection ───────────────────────────────────────────────

#[test]
fn for_in_array_sum() {
    let v = run_val(r#"
        let nums = [1, 2, 3, 4, 5];
        let total = 0;
        for n in nums { total = total + n; }
        return total;
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 15.0),
        other => panic!("expected 15, got {:?}", other),
    }
}

#[test]
fn for_in_array_count() {
    let v = run_val(r#"
        let items = ["a", "b", "c"];
        let count = 0;
        for item in items { count = count + 1; }
        return count;
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 3.0),
        other => panic!("expected 3, got {:?}", other),
    }
}

#[test]
fn for_in_string_chars() {
    let v = run_val(r#"
        let count = 0;
        for ch in "hello" { count = count + 1; }
        return count;
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 5.0),
        other => panic!("expected 5, got {:?}", other),
    }
}

// ── G-1/G-5: ⧉ Genesis array literal ────────────────────────────────────────

#[test]
fn genesis_array_literal_length() {
    let v = run_val("return len(⧉1‥2‥3⧉);");
    match v {
        Value::Number(n) => assert_eq!(n, 3.0),
        other => panic!("expected 3, got {:?}", other),
    }
}

#[test]
fn genesis_array_spread_flattens() {
    let v = run_val(r#"
        let base = [1, 2, 3];
        let result = ⧉…base‥4‥5⧉;
        return len(result);
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 5.0, "spread should flatten 3+2=5 elements"),
        other => panic!("expected 5, got {:?}", other),
    }
}

// ── G-4/G-8: ↦ binding projection ───────────────────────────────────────────

#[test]
fn binding_projection_simple() {
    let v = run_val(r#"
        result ↦ 99;
        return result;
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 99.0),
        other => panic!("expected 99, got {:?}", other),
    }
}

#[test]
fn binding_projection_expression() {
    let v = run_val(r#"
        let x = 10;
        y ↦ x * 2;
        return y;
    "#);
    match v {
        Value::Number(n) => assert_eq!(n, 20.0),
        other => panic!("expected 20, got {:?}", other),
    }
}
