use aeonmi_project::core::{
    compiler::Compiler,
    lexer::Lexer,
    lowering::lower_ast_to_ir,
    parser::Parser,
    vm::{Interpreter, Value},
};

#[test]
fn function_decl_and_body() {
    let code = r#"
        function add(a, b) { 
            let sum = a + b; 
            log(sum); 
            return sum; 
        }
        function main() {
            let r = 1 + 2;
            log(r);
        }
    "#;

    let out = std::env::temp_dir().join("aeonmi_func_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap())
        .expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");

    // Function signature + body structure
    assert!(js.contains("function add(a, b)"));
    assert!(js.contains("let sum = (a + b);"));
    assert!(js.contains("console.log(sum);"));
    assert!(js.contains("return sum;"));

    // Main function with post-function code
    assert!(js.contains("function main()"));
    assert!(js.contains("let r = (1 + 2);"));
    assert!(js.contains("console.log(r);"));
}

#[test]
fn vm_supports_default_params_and_variadics() {
    let source = r#"
        fn greet(name, prefix = "Hello", ...extras) {
            let suffix = "!";
            if (len(extras) > 0) {
                suffix = extras[0];
            }
            return prefix + " " + name + suffix;
        }

        fn count(prefix = 0, ...values) {
            return len(values) + prefix;
        }

        let lambda = fn(value, factor = 2) {
            return value * factor;
        };

        let lambda_result = lambda(21);
        let inline_result = (fn(x, ...rest) { return x + len(rest); })(5, 1, 2);

        let a = greet("Aeonmi");
        let b = greet("Aeonmi", "Hi", "?");
        let c = count(1, 10, 20, 30);
        let d = count();
    "#;

    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize().expect("tokenization");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parse");
    let module = lower_ast_to_ir(&ast, "fn_features").expect("lowering");

    let mut vm = Interpreter::new();
    vm.run_module(&module).expect("vm run");

    match vm.env.get("a").expect("a defined") {
        Value::String(s) => assert_eq!(s, "Hello Aeonmi!"),
        other => panic!("unexpected value for a: {:?}", other),
    }
    match vm.env.get("b").expect("b defined") {
        Value::String(s) => assert_eq!(s, "Hi Aeonmi?"),
        other => panic!("unexpected value for b: {:?}", other),
    }
    match vm.env.get("c").expect("c defined") {
        Value::Number(n) => assert_eq!(n, 4.0),
        other => panic!("unexpected value for c: {:?}", other),
    }
    match vm.env.get("d").expect("d defined") {
        Value::Number(n) => assert_eq!(n, 0.0),
        other => panic!("unexpected value for d: {:?}", other),
    }
    match vm.env.get("lambda_result").expect("lambda result defined") {
        Value::Number(n) => assert_eq!(n, 42.0),
        other => panic!("unexpected lambda result: {:?}", other),
    }
    match vm.env.get("inline_result").expect("inline result defined") {
        Value::Number(n) => assert_eq!(n, 7.0),
        other => panic!("unexpected inline result: {:?}", other),
    }
}

#[test]
fn vm_supports_object_literals_and_member_access() {
    let source = r#"
        fn build_user(name, age = 1) {
            return { name: name, age: age, meta: { active: true } };
        }

        let user = build_user("Aeonmi", 5);
        let name = user.name;
        let age = user.age;
        let active = user.meta.active;
        let missing = user.missing;
    "#;

    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize().expect("tokenization");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parse");
    let module = lower_ast_to_ir(&ast, "object_features").expect("lowering");

    let mut vm = Interpreter::new();
    vm.run_module(&module).expect("vm run");

    match vm.env.get("name").expect("name defined") {
        Value::String(s) => assert_eq!(s, "Aeonmi"),
        other => panic!("unexpected name: {:?}", other),
    }
    match vm.env.get("age").expect("age defined") {
        Value::Number(n) => assert_eq!(n, 5.0),
        other => panic!("unexpected age: {:?}", other),
    }
    match vm.env.get("active").expect("active defined") {
        Value::Bool(b) => assert!(b),
        other => panic!("unexpected active: {:?}", other),
    }
    match vm.env.get("missing").expect("missing defined") {
        Value::Null => {}
        other => panic!("expected null for missing, got {:?}", other),
    }
}
