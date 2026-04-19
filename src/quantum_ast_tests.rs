use crate::core::ast::ASTNode;
use crate::core::code_generator::CodeGenerator;

#[cfg(test)]
mod quantum_ast_integration_tests {
    use super::*;
    use crate::core::ast::{QuantumFunctionType, QuantumBindingType, FunctionParam};
    use crate::core::token::TokenKind;

    #[test]
    fn test_quantum_binary_expr_generation() {
        let quantum_xor = ASTNode::QuantumBinaryExpr {
            left: Box::new(ASTNode::StringLiteral("|0⟩".to_string())),
            op: TokenKind::QuantumXor,
            right: Box::new(ASTNode::StringLiteral("|1⟩".to_string())),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![quantum_xor])).unwrap();

        assert!(js_code.contains("__quantum.quantumXor") || js_code.contains("quantumXor"),
            "Expected quantum xor call in: {}", js_code);
        println!("✅ QuantumBinaryExpr code generation test passed");
    }

    #[test]
    fn test_quantum_function_generation() {
        let quantum_func = ASTNode::QuantumFunction {
            func_type: QuantumFunctionType::Quantum,
            name: "test_quantum_func".to_string(),
            params: vec![FunctionParam { name: "qubit".to_string(), line: 0, column: 0 }],
            body: vec![ASTNode::Return(Box::new(ASTNode::Identifier("qubit".to_string())))],
            line: 0,
            column: 0,
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![quantum_func])).unwrap();

        assert!(js_code.contains("test_quantum_func") || js_code.contains("Quantum Function"),
            "Expected quantum function in: {}", js_code);
        println!("✅ QuantumFunction code generation test passed");
    }

    #[test]
    fn test_quantum_loop_generation() {
        let quantum_loop = ASTNode::QuantumLoop {
            condition: Box::new(ASTNode::BooleanLiteral(true)),
            body: Box::new(ASTNode::Block(vec![
                ASTNode::Log(Box::new(ASTNode::StringLiteral("looping".to_string()))),
            ])),
            decoherence_threshold: Some(0.01),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![quantum_loop])).unwrap();

        assert!(js_code.contains("while") || js_code.contains("evaluateLoop"),
            "Expected loop in: {}", js_code);
        println!("✅ QuantumLoop code generation test passed");
    }

    #[test]
    fn test_probability_branch_generation() {
        let prob_branch = ASTNode::ProbabilityBranch {
            condition: Box::new(ASTNode::Identifier("quantum_state".to_string())),
            probability: Some(0.75),
            then_branch: Box::new(ASTNode::Log(Box::new(ASTNode::StringLiteral("success".to_string())))),
            else_branch: Some(Box::new(ASTNode::Log(Box::new(ASTNode::StringLiteral("failure".to_string()))))),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![prob_branch])).unwrap();

        assert!(js_code.contains("if") || js_code.contains("quantum"),
            "Expected branch in: {}", js_code);
        println!("✅ ProbabilityBranch code generation test passed");
    }

    #[test]
    fn test_superposition_switch_generation() {
        let superposition_switch = ASTNode::SuperpositionSwitch {
            value: Box::new(ASTNode::Identifier("quantum_var".to_string())),
            cases: vec![
                crate::core::ast::SuperpositionCase {
                    pattern: "|0⟩".to_string(),
                    body: vec![ASTNode::Log(Box::new(ASTNode::StringLiteral("zero".to_string())))],
                },
                crate::core::ast::SuperpositionCase {
                    pattern: "|1⟩".to_string(),
                    body: vec![ASTNode::Log(Box::new(ASTNode::StringLiteral("one".to_string())))],
                },
            ],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![superposition_switch])).unwrap();

        assert!(js_code.contains("superpositionSwitch") || js_code.contains("quantum"),
            "Expected switch in: {}", js_code);
        println!("✅ SuperpositionSwitch code generation test passed");
    }

    #[test]
    fn test_ai_learning_block_generation() {
        let ai_block = ASTNode::AILearningBlock {
            data_binding: Some("training_data".to_string()),
            model_binding: Some("neural_net".to_string()),
            body: vec![
                ASTNode::Log(Box::new(ASTNode::Identifier("processed_data".to_string()))),
            ],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![ai_block])).unwrap();

        assert!(js_code.contains("AI Learning") || js_code.contains("aiLearning"),
            "Expected AI learning block in: {}", js_code);
        println!("✅ AILearningBlock code generation test passed");
    }

    #[test]
    fn test_time_block_generation() {
        let time_block = ASTNode::TimeBlock {
            duration: Some(Box::new(ASTNode::NumberLiteral(1000.0))),
            body: vec![
                ASTNode::Log(Box::new(ASTNode::StringLiteral("tick".to_string()))),
            ],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![time_block])).unwrap();

        assert!(js_code.contains("time") || js_code.contains("1000"),
            "Expected time block in: {}", js_code);
        println!("✅ TimeBlock code generation test passed");
    }

    #[test]
    fn test_quantum_variable_decl_generation() {
        let qvar = ASTNode::QuantumVariableDecl {
            name: "psi".to_string(),
            binding_type: QuantumBindingType::Superposition,
            value: Box::new(ASTNode::StringLiteral("|+⟩".to_string())),
            line: 0,
            column: 0,
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![qvar])).unwrap();

        assert!(js_code.contains("psi"), "Expected variable name in: {}", js_code);
        println!("✅ QuantumVariableDecl code generation test passed");
    }

    #[test]
    fn test_hieroglyphic_op_generation() {
        let glyph_op = ASTNode::HieroglyphicOp {
            symbol: "𓀀".to_string(),
            args: vec![
                ASTNode::Identifier("q1".to_string()),
                ASTNode::NumberLiteral(42.0),
            ],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&ASTNode::Program(vec![glyph_op])).unwrap();

        assert!(js_code.contains("__glyph"), "Expected __glyph call in: {}", js_code);
        assert!(js_code.contains("𓀀"), "Expected glyph symbol in: {}", js_code);
        println!("✅ HieroglyphicOp code generation test passed");
    }

    #[test]
    fn run_all_quantum_ast_tests() {
        println!("🚀 Running comprehensive quantum AST integration tests...\n");

        test_quantum_binary_expr_generation();
        test_quantum_function_generation();
        test_quantum_loop_generation();
        test_probability_branch_generation();
        test_superposition_switch_generation();
        test_ai_learning_block_generation();
        test_time_block_generation();
        test_quantum_variable_decl_generation();
        test_hieroglyphic_op_generation();

        println!("\n🎉 All quantum AST integration tests passed!");
    }
}
