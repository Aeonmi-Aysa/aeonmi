use aeonmi_project::core::{code_generator, lexer, parser, semantic_analyzer};
use std::process::Command;

fn main() {
    if let Err(err) = run_quantum_feature_smoke() {
        eprintln!("Quantum feature example failed: {err}");
        std::process::exit(1);
    }
}

fn run_quantum_feature_smoke() -> Result<(), Box<dyn std::error::Error>> {
    // Compile test_quantum.ai
    let source = std::fs::read_to_string("test_quantum.ai")?;
    
    let mut lexer = lexer::Lexer::new(&source, false);
    let tokens = lexer.tokenize()?;
    
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    
    let mut analyzer = semantic_analyzer::SemanticAnalyzer::new();
    analyzer.analyze(&ast)?;
    
    let mut codegen = code_generator::CodeGenerator::new();
    let js_code = codegen.generate(&ast)?;
    
    println!("Generated JavaScript:\n{}", js_code);
    
    // Write to temp file
    std::fs::write("test_quantum_output.js", &js_code)?;
    
    // Run with node
    let output = Command::new("node")
        .arg("test_quantum_output.js")
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\nNode.js output:\n{}", stdout);
    
    // Verify quantum helpers are included
    assert!(js_code.contains("__aeonmi_superpose"), "Missing superpose helper");
    assert!(js_code.contains("__aeonmi_entangle"), "Missing entangle helper");
    assert!(js_code.contains("__aeonmi_measure"), "Missing measure helper");
    
    // Verify probabilistic branching code
    assert!(js_code.contains("Math.random()"), "Missing random number generation");
    
    // Verify decoherence threshold
    assert!(js_code.contains("__maxIters"), "Missing iteration limit");
    
    // Verify quantum try-catch
    assert!(js_code.contains("__quantumFailed"), "Missing quantum failure tracking");
    
    println!("\n✓ All quantum feature tests passed");
    Ok(())
}
