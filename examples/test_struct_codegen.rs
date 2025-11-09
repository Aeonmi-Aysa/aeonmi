//! Test code generation for structs

use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::code_generator::CodeGenerator;

fn main() {
    let source = r#"
struct Point { x: Number, y: Number }

let p = Point({ x: 5, y: 0 });
log(p.x);
log(p.y);
"#;
    
    let mut lexer = Lexer::new(source, false);
    let tokens = lexer.tokenize().unwrap();
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    let mut generator = CodeGenerator::new_js();
    let js_code = generator.generate(&ast).unwrap();
    
    println!("Generated JavaScript:");
    println!("{}", js_code);
}
