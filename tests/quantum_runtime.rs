use aeonmi_project::core::{
    ir::*,
    vm::{Interpreter, Value},
};

fn run_module_with_statements(stmts: Vec<Stmt>) -> Interpreter {
    let module = Module {
        name: "quantum_rt".into(),
        imports: vec![],
        decls: vec![Decl::Fn(FnDecl {
            name: "main".into(),
            params: vec![],
            body: Block { stmts },
        })],
    };

    let mut vm = Interpreter::new();
    vm.run_module(&module).expect("module execution");
    vm
}

#[test]
fn quantum_variable_initialization_creates_qubit() {
    let stmts = vec![Stmt::QuantumLet {
        name: "q".into(),
        binding: QuantumBinding::Classical,
        value: Expr::QuantumState {
            label: "|1⟩".into(),
            amplitude: None,
        },
    }];

    let vm = run_module_with_statements(stmts);

    let stored = vm.env.get("q").expect("qubit stored");
    match stored {
        Value::QubitReference(name) => assert_eq!(name, "q"),
        other => panic!("expected qubit reference, got {:?}", other),
    }

    let zero_probability = vm
        .quantum_sim
        .get_zero_probability("q")
        .expect("qubit probability");
    assert!(zero_probability < 1e-6, "qubit should be in |1⟩ state");
}

#[test]
fn quantum_tensor_binding_creates_references() {
    let stmts = vec![Stmt::QuantumLet {
        name: "qarr".into(),
        binding: QuantumBinding::Tensor,
        value: Expr::QuantumArray {
            elements: vec![
                Expr::QuantumState {
                    label: "|0⟩".into(),
                    amplitude: None,
                },
                Expr::QuantumState {
                    label: "|1⟩".into(),
                    amplitude: None,
                },
            ],
            is_superposition: false,
        },
    }];

    let vm = run_module_with_statements(stmts);
    let stored = vm.env.get("qarr").expect("tensor stored");
    match stored {
        Value::QuantumArray(items, _) => {
            assert_eq!(items.len(), 2);
            for (idx, item) in items.iter().enumerate() {
                match item {
                    Value::QubitReference(name) => {
                        assert_eq!(name, &format!("qarr[{}]", idx));
                        assert!(vm.quantum_sim.qubits.contains_key(name));
                    }
                    other => panic!("expected qubit reference, got {:?}", other),
                }
            }
        }
        other => panic!("expected quantum array, got {:?}", other),
    }
}

#[test]
fn probability_branch_uses_weighted_decision() {
    let stmts = vec![
        Stmt::Let {
            name: "flag".into(),
            value: Some(Expr::Lit(Lit::Number(0.0))),
        },
        Stmt::ProbabilityBranch {
            condition: Expr::Lit(Lit::Number(1.0)),
            probability: Some(1.0),
            then_block: Block {
                stmts: vec![Stmt::Assign {
                    target: Expr::Ident("flag".into()),
                    value: Expr::Lit(Lit::Number(1.0)),
                }],
            },
            else_block: None,
        },
        Stmt::ProbabilityBranch {
            condition: Expr::Lit(Lit::Number(0.0)),
            probability: Some(0.0),
            then_block: Block {
                stmts: vec![Stmt::Assign {
                    target: Expr::Ident("flag".into()),
                    value: Expr::Lit(Lit::Number(-1.0)),
                }],
            },
            else_block: Some(Block {
                stmts: vec![Stmt::Assign {
                    target: Expr::Ident("flag".into()),
                    value: Expr::Lit(Lit::Number(2.0)),
                }],
            }),
        },
    ];

    let vm = run_module_with_statements(stmts);
    let stored = vm.env.get("flag").expect("flag stored");
    match stored {
        Value::Number(n) => assert_eq!(n, 2.0),
        other => panic!("expected number, got {:?}", other),
    }
}
