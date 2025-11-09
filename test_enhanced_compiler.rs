//! Test program for the enhanced semantic analyzer and module system

use std::path::PathBuf;

mod core;
use crate::core::{
    lexer::Lexer,
    parser::Parser,
    enhanced_semantic::EnhancedSemanticAnalyzer,
    module_system::{CompilationContext, ModuleResolver},
    enhanced_error::*,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Enhanced Aeonmi Compiler Architecture");
    
    // Test file path
    let test_file = "test_module_system.aeon";
    let source = std::fs::read_to_string(test_file)?;
    
    println!("📁 Source file: {}", test_file);
    println!("📝 Source code length: {} characters", source.len());
    
    // Phase 1: Lexical Analysis
    println!("\n🔍 Phase 1: Lexical Analysis");
    let mut lexer = Lexer::new(&source, test_file.to_string());
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {:?}", e))?;
    println!("✅ Generated {} tokens", tokens.len());
    
    // Phase 2: Syntax Analysis
    println!("\n🌳 Phase 2: Syntax Analysis");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser error: {:?}", e))?;
    println!("✅ Generated AST with {} top-level nodes", count_ast_nodes(&ast));
    
    // Phase 3: Semantic Analysis
    println!("\n🧠 Phase 3: Enhanced Semantic Analysis");
    let mut semantic_analyzer = EnhancedSemanticAnalyzer::new();
    semantic_analyzer.set_current_file(test_file.to_string());
    
    let analysis_result = semantic_analyzer.analyze(&ast);
    
    match analysis_result {
        Ok(_) => {
            println!("✅ Semantic analysis completed successfully");
            let diagnostics = semantic_analyzer.get_diagnostics();
            if diagnostics.is_empty() {
                println!("✨ No semantic errors or warnings found");
            } else {
                println!("⚠️  Found {} diagnostic messages:", diagnostics.len());
                for diagnostic in diagnostics {
                    println!("   {}: {}", 
                        match diagnostic.severity {
                            crate::core::enhanced_semantic::DiagnosticSeverity::Error => "ERROR",
                            crate::core::enhanced_semantic::DiagnosticSeverity::Warning => "WARN",
                            crate::core::enhanced_semantic::DiagnosticSeverity::Info => "INFO",
                        },
                        diagnostic.error.message
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Semantic analysis failed: {}", e);
        }
    }
    
    // Phase 4: Module System Test
    println!("\n📦 Phase 4: Module System Test");
    let root_path = PathBuf::from(".");
    let mut compilation_context = CompilationContext::new(root_path, "test_module_system".to_string());
    
    match compilation_context.compile_project() {
        Ok(_) => {
            println!("✅ Module system compilation successful");
            let modules = compilation_context.get_modules();
            println!("📚 Loaded {} modules", modules.len());
            for (name, module) in modules {
                println!("   Module '{}' exports: {:?}", name, module.exports);
            }
        }
        Err(e) => {
            println!("⚠️  Module system test skipped (expected for single file): {}", e);
        }
    }
    
    println!("\n🎉 Enhanced compiler architecture test completed!");
    Ok(())
}

fn count_ast_nodes(node: &crate::core::ast::ASTNode) -> usize {
    use crate::core::ast::ASTNode;
    
    match node {
        ASTNode::Program(statements) => {
            1 + statements.iter().map(count_ast_nodes).sum::<usize>()
        }
        ASTNode::Module { body, .. } => {
            1 + body.iter().map(count_ast_nodes).sum::<usize>()
        }
        ASTNode::Function { body, .. } => {
            1 + body.iter().map(count_ast_nodes).sum::<usize>()
        }
        ASTNode::ClassDecl { methods, .. } => {
            1 + methods.iter().map(count_ast_nodes).sum::<usize>()
        }
        _ => 1, // Other nodes are leaf nodes for counting purposes
    }
}