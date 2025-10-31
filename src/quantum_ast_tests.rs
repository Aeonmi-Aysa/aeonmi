use crate::core::ast::ASTNode;
use crate::core::code_generator::CodeGenerator;

#[cfg(test)]
mod quantum_ast_integration_tests {
    use super::*;

    #[test]
    fn test_quantum_binary_expr_generation() {
        let quantum_xor = ASTNode::QuantumBinaryExpr {
            left: Box::new(ASTNode::Literal { value: "|0⟩".to_string() }),
            op: crate::core::ast::QuantumOp::XOR,
            right: Box::new(ASTNode::Literal { value: "|1⟩".to_string() }),
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&quantum_xor);
        
        assert!(js_code.contains("__quantum.quantumXor"));
        assert!(js_code.contains("|0⟩"));
        assert!(js_code.contains("|1⟩"));
        println!("✅ QuantumBinaryExpr code generation test passed");
    }

    #[test]
    fn test_quantum_function_generation() {
        use crate::core::ast::{QuantumFunctionType, Parameter};
        
        let quantum_func = ASTNode::QuantumFunction {
            func_type: QuantumFunctionType::Quantum,
            name: "test_quantum_func".to_string(),
            params: vec![
                Parameter { name: "qubit".to_string(), param_type: None },
            ],
            body: vec![
                ASTNode::Return {
                    value: Some(Box::new(ASTNode::Identifier { name: "qubit".to_string() })),
                },
            ],
            return_type: None,
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&quantum_func);
        
        assert!(js_code.contains("function"));
        assert!(js_code.contains("test_quantum_func"));
        assert!(js_code.contains("/* ⊙ Quantum Function */"));
        println!("✅ QuantumFunction code generation test passed");
    }

    #[test]
    fn test_quantum_loop_generation() {
        let quantum_loop = ASTNode::QuantumLoop {
            iterator: "qbit".to_string(),
            iterable: Box::new(ASTNode::Identifier { name: "quantum_range".to_string() }),
            body: vec![
                ASTNode::Assignment {
                    target: "state".to_string(),
                    value: Box::new(ASTNode::Identifier { name: "qbit".to_string() }),
                },
            ],
            is_superposition: true,
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&quantum_loop);
        
        assert!(js_code.contains("__quantum.evaluateLoop"));
        assert!(js_code.contains("quantum_range"));
        assert!(js_code.contains("⊙ Quantum Loop"));
        println!("✅ QuantumLoop code generation test passed");
    }

    #[test]
    fn test_probability_branch_generation() {
        let prob_branch = ASTNode::ProbabilityBranch {
            condition: Box::new(ASTNode::Identifier { name: "quantum_state".to_string() }),
            probability: Some(0.75),
            then_body: vec![
                ASTNode::Assignment {
                    target: "result".to_string(),
                    value: Box::new(ASTNode::Literal { value: "success".to_string() }),
                },
            ],
            else_body: Some(vec![
                ASTNode::Assignment {
                    target: "result".to_string(),
                    value: Box::new(ASTNode::Literal { value: "failure".to_string() }),
                },
            ]),
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&prob_branch);
        
        assert!(js_code.contains("∇ Probability Branch"));
        assert!(js_code.contains("0.75"));
        assert!(js_code.contains("quantum_state"));
        println!("✅ ProbabilityBranch code generation test passed");
    }

    #[test]
    fn test_superposition_switch_generation() {
        let superposition_switch = ASTNode::SuperpositionSwitch {
            expr: Box::new(ASTNode::Identifier { name: "quantum_var".to_string() }),
            cases: vec![
                (ASTNode::Literal { value: "|0⟩".to_string() }, vec![
                    ASTNode::Assignment {
                        target: "outcome".to_string(),
                        value: Box::new(ASTNode::Literal { value: "zero".to_string() }),
                    },
                ]),
                (ASTNode::Literal { value: "|1⟩".to_string() }, vec![
                    ASTNode::Assignment {
                        target: "outcome".to_string(),
                        value: Box::new(ASTNode::Literal { value: "one".to_string() }),
                    },
                ]),
            ],
            default_case: Some(vec![
                ASTNode::Assignment {
                    target: "outcome".to_string(),
                    value: Box::new(ASTNode::Literal { value: "superposition".to_string() }),
                },
            ]),
            amplitude: Some(0.866),
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&superposition_switch);
        
        assert!(js_code.contains("◇ Superposition Switch"));
        assert!(js_code.contains("__quantum.superpositionSwitch"));
        assert!(js_code.contains("0.866"));
        println!("✅ SuperpositionSwitch code generation test passed");
    }

    #[test]
    fn test_ai_learning_block_generation() {
        let ai_block = ASTNode::AILearningBlock {
            data_binding: Some("training_data".to_string()),
            model_binding: Some("neural_net".to_string()),
            body: vec![
                ASTNode::Assignment {
                    target: "prediction".to_string(),
                    value: Box::new(ASTNode::Identifier { name: "processed_data".to_string() }),
                },
            ],
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&ai_block);
        
        assert!(js_code.contains("AI Learning Block"));
        assert!(js_code.contains("training_data"));
        assert!(js_code.contains("neural_net"));
        assert!(js_code.contains("__quantum.aiLearningBlock"));
        println!("✅ AILearningBlock code generation test passed");
    }

    #[test]
    fn test_time_block_generation() {
        let time_block = ASTNode::TimeBlock {
            duration: Some(Box::new(ASTNode::Literal { value: "1000".to_string() })),
            body: vec![
                ASTNode::Assignment {
                    target: "measurement".to_string(),
                    value: Box::new(ASTNode::Identifier { name: "quantum_state".to_string() }),
                },
            ],
        };
        
        let mut generator = CodeGenerator::new();
        let js_code = generator.emit_js(&time_block);
        
        assert!(js_code.contains("Time Block"));
        assert!(js_code.contains("__time.block"));
        assert!(js_code.contains("1000"));
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