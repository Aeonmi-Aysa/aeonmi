//! Test trait enforcement and impl blocks

use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::code_generator::CodeGenerator;
use aeonmi_project::core::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let source = r#"
trait Greeter {
    function greet() {
        log("Default greeting");
    }
}

impl Greeter for MyType {
    function greet() {
        log("Hello from MyType!");
    }
}

function main() {
    log("Trait implementation test complete");
}
"#;
    
    println!("=== Testing Trait Implementation ===\n");
    
    // Parse
    let mut lexer = Lexer::new(source, false);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(&ast) {
        Ok(_) => println!("✓ Semantic analysis passed - trait requirements met\n"),
        Err(e) => {
            println!("✗ Semantic analysis failed:");
            println!("{}\n", e);
            return;
        }
    }
    
    // Generate JavaScript
    let mut generator = CodeGenerator::new_js();
    let js_code = generator.generate(&ast).unwrap();
    
    println!("Generated JavaScript:");
    println!("{}", js_code);
}
