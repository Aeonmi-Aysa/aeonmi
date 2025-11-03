#[cfg(test)]
mod quantum_ast_integration_tests {
    use crate::core::ast::{ASTNode, FunctionParam, QuantumFunctionType, SuperpositionCase};
    use crate::core::code_generator::CodeGenerator;
    use crate::core::token::TokenKind;

    #[test]
    fn test_quantum_binary_expr_generation() {
        let quantum_xor = ASTNode::QuantumBinaryExpr {
            op: TokenKind::QuantumXor,
            left: Box::new(ASTNode::QuantumState {
                state: "|0⟩".to_string(),
                amplitude: None,
            }),
            right: Box::new(ASTNode::QuantumState {
                state: "|1⟩".to_string(),
                amplitude: None,
            }),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&quantum_xor).expect("quantum xor code");

        assert!(js_code.contains("__quantum.quantumXor"));
        assert!(js_code.contains("|0⟩"));
        assert!(js_code.contains("|1⟩"));
        println!("✅ QuantumBinaryExpr code generation test passed");
    }

    #[test]
    fn test_quantum_function_generation() {
        let quantum_func = ASTNode::QuantumFunction {
            func_type: QuantumFunctionType::Quantum,
            name: "test_quantum_func".to_string(),
            params: vec![FunctionParam {
                name: "qubit".to_string(),
                line: 0,
                column: 0,
                default: None,
                is_variadic: false,
            }],
            body: vec![ASTNode::Return(Box::new(ASTNode::Identifier(
                "qubit".to_string(),
            )))],
            line: 0,
            column: 0,
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator
            .generate(&quantum_func)
            .expect("quantum func code");

        assert!(js_code.contains("function"));
        assert!(js_code.contains("test_quantum_func"));
        assert!(js_code.contains("/* ⊙ Quantum Function */"));
        println!("✅ QuantumFunction code generation test passed");
    }

    #[test]
    fn test_quantum_loop_generation() {
        let quantum_loop = ASTNode::QuantumLoop {
            condition: Box::new(ASTNode::Identifier("continue_loop".to_string())),
            body: Box::new(ASTNode::Block(vec![ASTNode::Assignment {
                name: "state".to_string(),
                value: Box::new(ASTNode::Identifier("qbit".to_string())),
                line: 0,
                column: 0,
            }])),
            decoherence_threshold: Some(0.5),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator
            .generate(&quantum_loop)
            .expect("quantum loop code");

        assert!(js_code.contains("__quantum.evaluateLoop"));
        assert!(js_code.contains("Decoherence threshold"));
        assert!(js_code.contains("state = qbit"));
        println!("✅ QuantumLoop code generation test passed");
    }

    #[test]
    fn test_probability_branch_generation() {
        let prob_branch = ASTNode::ProbabilityBranch {
            condition: Box::new(ASTNode::Identifier("quantum_state".to_string())),
            probability: Some(0.75),
            then_branch: Box::new(ASTNode::Block(vec![ASTNode::Assignment {
                name: "result".to_string(),
                value: Box::new(ASTNode::StringLiteral("success".to_string())),
                line: 0,
                column: 0,
            }])),
            else_branch: Some(Box::new(ASTNode::Block(vec![ASTNode::Assignment {
                name: "result".to_string(),
                value: Box::new(ASTNode::StringLiteral("failure".to_string())),
                line: 0,
                column: 0,
            }]))),
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator
            .generate(&prob_branch)
            .expect("probability branch code");

        assert!(js_code.contains("__quantum.evaluate(quantum_state)"));
        assert!(js_code.contains("75.00%"));
        assert!(js_code.contains("failure"));
        println!("✅ ProbabilityBranch code generation test passed");
    }

    #[test]
    fn test_superposition_switch_generation() {
        let superposition_switch = ASTNode::SuperpositionSwitch {
            value: Box::new(ASTNode::Identifier("quantum_var".to_string())),
            cases: vec![
                SuperpositionCase {
                    pattern: "|0⟩".to_string(),
                    body: vec![ASTNode::Assignment {
                        name: "outcome".to_string(),
                        value: Box::new(ASTNode::StringLiteral("zero".to_string())),
                        line: 0,
                        column: 0,
                    }],
                },
                SuperpositionCase {
                    pattern: "|1⟩".to_string(),
                    body: vec![ASTNode::Assignment {
                        name: "outcome".to_string(),
                        value: Box::new(ASTNode::StringLiteral("one".to_string())),
                        line: 0,
                        column: 0,
                    }],
                },
            ],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator
            .generate(&superposition_switch)
            .expect("superposition switch code");

        assert!(js_code.contains("◇ Superposition Switch"));
        assert!(js_code.contains("__quantum.superpositionSwitch"));
        assert!(js_code.contains("Superposition case: |0⟩"));
        println!("✅ SuperpositionSwitch code generation test passed");
    }

    #[test]
    fn test_ai_learning_block_generation() {
        let ai_block = ASTNode::AILearningBlock {
            data_binding: Some("training_data".to_string()),
            model_binding: Some("neural_net".to_string()),
            body: vec![ASTNode::Assignment {
                name: "prediction".to_string(),
                value: Box::new(ASTNode::Identifier("processed_data".to_string())),
                line: 0,
                column: 0,
            }],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator
            .generate(&ai_block)
            .expect("ai learning block code");

        assert!(js_code.contains("AI Learning Block"));
        assert!(js_code.contains("training_data"));
        assert!(js_code.contains("neural_net"));
        assert!(js_code.contains("__quantum.aiLearningBlock"));
        println!("✅ AILearningBlock code generation test passed");
    }

    #[test]
    fn test_time_block_generation() {
        let time_block = ASTNode::TimeBlock {
            duration: Some(Box::new(ASTNode::NumberLiteral(1000.0))),
            body: vec![ASTNode::Assignment {
                name: "measurement".to_string(),
                value: Box::new(ASTNode::Identifier("quantum_state".to_string())),
                line: 0,
                column: 0,
            }],
        };

        let mut generator = CodeGenerator::new();
        let js_code = generator.generate(&time_block).expect("time block code");

        assert!(js_code.contains("__time.block(1000)"));
        assert!(js_code.contains("measurement = quantum_state"));
        println!("✅ TimeBlock code generation test passed");
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

        println!("\n🎉 All quantum AST integration tests passed!");
        println!("✅ AEONMI Full Quantum AST Support is working correctly!");
    }
}
