//! Test code generation for enums

use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::code_generator::CodeGenerator;

fn main() {
    let source = r#"
enum Color { Red, Green, Blue }

let c1 = Color.Red;
let c2 = Color.Blue;
log(c1 == Color.Red);
log(c2 == Color.Blue);
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
