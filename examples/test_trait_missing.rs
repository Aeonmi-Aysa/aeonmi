//! Test trait enforcement with missing method

use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let source = r#"
trait Greeter {
    function greet() {
        log("Default greeting");
    }
    function goodbye() {
        log("Default goodbye");
    }
}

impl Greeter for MyType {
    function greet() {
        log("Hello from MyType!");
    }
    // Missing goodbye() - should cause semantic error
}

function main() {
    log("Should not get here");
}
"#;
    
    println!("=== Testing Trait Enforcement (Missing Method) ===\n");
    
    // Parse
    let mut lexer = Lexer::new(source, false);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    // Semantic analysis - should fail
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(&ast) {
        Ok(_) => println!("✗ ERROR: Semantic analysis should have failed but passed!"),
        Err(e) => {
            println!("✓ Semantic analysis correctly failed with error:");
            println!("{}", e);
        }
    }
}
