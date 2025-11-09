use std::fs;

fn main() {
    let source = fs::read_to_string("test_struct.ai").unwrap();
    
    let mut lexer = aeonmi_project::core::lexer::Lexer::new(&source, false);
    let tokens = lexer.tokenize().unwrap();
    
    let mut parser = aeonmi_project::core::parser::Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    let mut generator = aeonmi_project::core::code_generator::CodeGenerator::new_js();
    let js_code = generator.generate(&ast).unwrap();
    
    fs::write("test_struct.js", js_code).unwrap();
    println!("Generated test_struct.js");
}
