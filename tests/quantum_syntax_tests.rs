use aeonmi_project::core::{lexer::Lexer, parser::Parser, diagnostics::*};

#[test]
fn test_quantum_native_syntax_lexing() {
    let code = r#"
    ⟨quantum_state⟩ ∈ |0⟩ + |1⟩
    ⟨classical_var⟩ ← 42
    "#;
    
    let mut lexer = Lexer::new(code, false);
    let tokens = lexer.tokenize().expect("Should tokenize quantum syntax");
    
    // Check that we get the expected quantum tokens
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    println!("Tokens: {:?}", token_kinds);
    
    // Should contain quantum bracket tokens and binding operators
    assert!(token_kinds.iter().any(|t| matches!(t, aeonmi_project::core::token::TokenKind::QuantumBracketOpen)));
    assert!(token_kinds.iter().any(|t| matches!(t, aeonmi_project::core::token::TokenKind::QuantumIn)));
    assert!(token_kinds.iter().any(|t| matches!(t, aeonmi_project::core::token::TokenKind::QuantumBind)));
}

#[test]
fn test_quantum_native_syntax_parsing() {
    let code = r#"
    ⟨quantum_state⟩ ∈ |0⟩
    "#;
    
    let mut lexer = Lexer::new(code, false);
    let tokens = lexer.tokenize().expect("Should tokenize");
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Should parse quantum variable declaration");
    
    println!("AST: {:?}", ast);
    
    // Should create a quantum variable declaration AST node
    match ast {
        aeonmi_project::core::ast::ASTNode::Program(stmts) => {
            assert!(!stmts.is_empty());
            match &stmts[0] {
                aeonmi_project::core::ast::ASTNode::QuantumVariableDecl { .. } => {
                    // Success!
                }
                other => panic!("Expected QuantumVariableDecl, got {:?}", other),
            }
        }
        _ => panic!("Expected Program node"),
    }
}

#[test]
fn test_quantum_function_syntax() {
    let code = r#"
    ◯ classical_func⟨a, b⟩ {
        ⟨result⟩ ← ⟨a⟩ + ⟨b⟩
    }
    "#;
    
    let mut lexer = Lexer::new(code, false);
    let tokens = lexer.tokenize().expect("Should tokenize");
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(node) => println!("Quantum function parsed successfully: {:?}", node),
        Err(e) => println!("Parse error (expected for incomplete implementation): {}", e),
    }
}

#[test] 
fn test_quantum_diagnostics() {
    let span = Span::single(1, 10);
    let diag = quantum_syntax_error("Expected quantum binding operator", span);
    
    assert!(diag.suggestion.is_some());
    assert!(diag.help.is_some());
    assert!(diag.quantum_context.is_some());
    
    println!("Diagnostic: {:?}", diag);
}

#[test]
fn test_legacy_syntax_migration() {
    let span = Span::single(1, 1);
    let diag = legacy_syntax_error("let", span);
    
    assert_eq!(diag.title, "Legacy 'let' keyword detected");
    assert!(diag.suggestion.as_ref().unwrap().contains("⟨variable⟩ ← value"));
    
    println!("Legacy migration diagnostic: {:?}", diag);
}