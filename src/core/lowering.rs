#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Lowering: AST -> IR (desugaring + deterministic ordering)
use crate::core::ir::*;
use crate::core::TokenKind; // Import TokenKind from the core module

// Real AST -> IR lowering

pub fn lower_ast_to_ir(program: &crate::core::ast::ASTNode, name: &str) -> Result<Module, String> {
    use crate::core::ast::ASTNode;

    // Expect a Program at the top; if not, wrap as single-item program.
    let items: Vec<ASTNode> = match program {
        ASTNode::Program(v) => v.clone(),
        other => vec![other.clone()],
    };

    // Import accumulator — populated by ImportDecl nodes.
    let mut imports: Vec<Import> = Vec::new();

    let mut decls: Vec<Decl> = Vec::new();
    let mut main_stmts: Vec<Stmt> = Vec::new();
    
    for item in items {
        match item {
            ASTNode::Function { name: fn_name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in body {
                    stmts.push(lower_stmt_ast(&stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params: param_names,
                    body: Block { stmts },
                }));
            }
            // Phase 1: quantum functions treated as regular functions
            ASTNode::QuantumFunction { name: fn_name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in &body {
                    stmts.push(lower_stmt_ast(stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params: param_names,
                    body: Block { stmts },
                }));
            }
            // Phase 1: async functions treated as regular functions
            ASTNode::AsyncFunction { name: fn_name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in &body {
                    stmts.push(lower_stmt_ast(stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params: param_names,
                    body: Block { stmts },
                }));
            }
            // Phase 1: impl blocks — hoist methods as top-level functions named Type__method
            ASTNode::ImplBlock { target, methods } => {
                for method in &methods {
                    if let ASTNode::Function { name: mname, params, body, .. } = method {
                        let fn_name = format!("{}_{}", target, mname);
                        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                        let mut stmts: Vec<Stmt> = Vec::new();
                        for stmt_node in body {
                            stmts.push(lower_stmt_ast(stmt_node)?);
                        }
                        decls.push(Decl::Fn(FnDecl {
                            name: fn_name,
                            params: param_names,
                            body: Block { stmts },
                        }));
                    }
                }
            }
            // Phase 1: import declarations — record as IR import
            ASTNode::ImportDecl { names, path } => {
                for name in &names {
                    imports.push(Import { path: path.clone(), alias: Some(name.clone()) });
                }
            }
            // Phase 1: struct/enum — emit as main-level statements
            ASTNode::StructDecl { .. } | ASTNode::EnumDecl { .. } => {
                let stmt = lower_stmt_ast(&item)?;
                main_stmts.push(stmt);
            }
            // All other top-level items go into main function
            other => {
                let stmt = lower_stmt_ast(&other)?;
                main_stmts.push(stmt);
            }
        }
    }

    // Create main function if there are any statements
    if !main_stmts.is_empty() {
        decls.push(Decl::Fn(FnDecl {
            name: "main".to_string(),
            params: vec![],
            body: Block { stmts: main_stmts },
        }));
    }

    let mut m = Module { name: name.to_string(), imports, decls };
    m.imports.sort_by(|a, b| {
        (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref()))
    });
    m.decls.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(m)
}

fn lower_stmt_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Block(items) => {
            // Lower all items and emit as a flat Stmt::Block so every
            // declaration (qubit, let, etc.) is visible to subsequent stmts.
            let mut stmts = Vec::new();
            for item in items {
                stmts.push(lower_stmt_ast(item)?);
            }
            match stmts.len() {
                0 => Stmt::Expr(Expr::Object(vec![])),
                1 => stmts.remove(0),
                _ => Stmt::Block(stmts),
            }
        }

        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        A::Log(expr) => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(expr)?],
        }),
    A::Assignment { name, value, .. } => Stmt::Assign {
            target: Expr::Ident(name.clone()),
            value: lower_expr_ast(value)?,
        },

        A::Call { .. }
        | A::BinaryExpr { .. }
        | A::UnaryExpr { .. }
        | A::Identifier(_)
        | A::NumberLiteral(_)
        | A::StringLiteral(_)
        | A::BooleanLiteral(_) => Stmt::Expr(lower_expr_ast(n)?),

        A::If { condition, then_branch, else_branch } => Stmt::If {
            cond: lower_expr_ast(condition)?,
            then_block: lower_block_ast(then_branch)?,
            else_block: else_branch.as_ref().map(|e| lower_block_ast(e)).transpose()?,
        },

        A::While { condition, body } => Stmt::While {
            cond: lower_expr_ast(condition)?,
            body: lower_block_ast(body)?,
        },

        A::For { init, condition, increment, body } => {
            let init_stmt = init.as_ref().map(|b| lower_stmt_init_ast(b)).transpose()?;
            Stmt::For {
                init: init_stmt.map(Box::new),
                cond: condition.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                // Route through lower_stmt_ast so assignment steps (i = i + 1) work.
                // lower_expr_ast silently drops assignments; lower_stmt_ast correctly
                // produces Stmt::Assign which the VM can execute.
                step: increment.as_ref()
                    .map(|b| lower_stmt_ast(b).map(Box::new))
                    .transpose()?,
                body: lower_block_ast(body)?,
            }
        }

        // Decls at statement position
    A::VariableDecl { name, value, .. } => Stmt::Let {
            name: name.clone(),
            value: Some(lower_expr_ast(value)?),
        },

        // Quantum variable declarations — lower to plain let bindings.
        // The binding_type metadata is discarded at IR level; the VM treats
        // all quantum variables uniformly.  Do NOT embed comments into the
        // variable name — that makes the variable impossible to reference later.
        A::QuantumVariableDecl { name, value, .. } => {
            Stmt::Let {
                name: name.clone(),
                value: Some(lower_expr_ast(value)?),
            }
        }

        // Function within a statement position: ignore/emit no-op (top-level handled elsewhere).
        A::Function { .. } => Stmt::Expr(Expr::Object(vec![])),

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident(fname)),
                args,
            })
        }

        A::HieroglyphicOp { symbol, args } => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        }),

    A::Error(msg) => Stmt::Expr(Expr::Lit(Lit::String(format!("/* error: {msg} */")))),
    A::Program(_) => unreachable!("Program nodes are handled at the top level"),
    A::IdentifierSpanned { name, .. } => Stmt::Expr(Expr::Ident(name.clone())),
    
    // Quantum AST Nodes Implementation
    A::QuantumBinaryExpr { op, left, right } => {
        let left_expr = lower_expr_ast(left)?;
        let right_expr = lower_expr_ast(right)?;
        let quantum_op = match op {
            crate::core::token::TokenKind::QuantumXor => "__quantum_xor", // ⊕
            crate::core::token::TokenKind::QuantumTensor => "__quantum_tensor", // ⊗
            crate::core::token::TokenKind::SuperpositionState => "__quantum_superposition", // ◊
            crate::core::token::TokenKind::Entangle => "__quantum_entangle", // ∇
            _ => "__quantum_op",
        };
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident(quantum_op.to_string())),
            args: vec![left_expr, right_expr],
        })
    }
    A::QuantumIndexAccess { array, index, is_quantum_index } => {
        let array_expr = lower_expr_ast(array)?;
        let index_expr = lower_expr_ast(index)?;
        if *is_quantum_index {
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("__quantum_index".to_string())),
                args: vec![array_expr, index_expr],
            })
        } else {
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("__index_access".to_string())),
                args: vec![array_expr, index_expr],
            })
        }
    }
    A::QuantumFunction { func_type, name, params, body, .. } => {
        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut lowered_body = Vec::new();
        
        // Add quantum function type as a comment in the first statement
        let quantum_marker = match func_type {
            crate::core::ast::QuantumFunctionType::Quantum => "/* ⊙ Quantum Function */", // ⊙
            crate::core::ast::QuantumFunctionType::Classical => "/* ◉ Classical Function */", // ◉
            crate::core::ast::QuantumFunctionType::AINeural => "/* ⫸ AI Neural Function */", // ⫸
        };
        
        lowered_body.push(Stmt::Expr(Expr::Lit(Lit::String(quantum_marker.to_string()))));
        
        for stmt in body {
            lowered_body.push(lower_stmt_ast(stmt)?);
        }
        
        // For quantum functions, create a specialized function call instead of declaration
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__quantum_function".to_string())),
            args: vec![
                Expr::Lit(Lit::String(name.clone())),
                Expr::Lit(Lit::String(format!("{:?}", func_type))),
            ],
        })
    }
    A::ProbabilityBranch { condition, probability, then_branch, else_branch } => {
        let condition_expr = lower_expr_ast(condition)?;
        let then_block = lower_block_ast(then_branch)?;
        let else_block = if let Some(else_br) = else_branch {
            Some(lower_block_ast(else_br)?)
        } else {
            None
        };
        
        // Add probability annotation as comment
        let _prob_comment = if let Some(p) = probability {
            format!("/* Probability: {:.2}% */", p * 100.0)
        } else {
            "/* Quantum probability */".to_string()
        };
        
        Stmt::If {
            cond: condition_expr,
            then_block,
            else_block,
        }
    }
    A::QuantumLoop { condition, body, decoherence_threshold } => {
        let condition_expr = lower_expr_ast(condition)?;
        let body_block = lower_block_ast(body)?;
        
        // Add decoherence protection
        let protected_condition = if decoherence_threshold.is_some() {
            Expr::Call {
                callee: Box::new(Expr::Ident("__quantum_protect_loop".to_string())),
                args: vec![condition_expr],
            }
        } else {
            condition_expr
        };
        
        Stmt::While {
            cond: protected_condition,
            body: body_block,
        }
    }
    A::SuperpositionSwitch { value, cases: _ } => {
        let value_expr = lower_expr_ast(value)?;
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__superposition_switch".to_string())),
            args: vec![value_expr],
        })
    }
    A::QuantumTryCatch { attempt_body, error_probability, catch_body, success_body: _ } => {
        // Convert quantum try-catch to function call
        let prob_value = if let Some(p) = error_probability {
            Expr::Lit(Lit::Number(*p))
        } else {
            Expr::Lit(Lit::Number(0.1)) // Default 10% error probability
        };
        
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__quantum_try_catch".to_string())),
            args: vec![
                prob_value,
                Expr::Lit(Lit::String("attempt_block".to_string())),
                Expr::Lit(Lit::String("catch_block".to_string())),
            ],
        })
    }
    A::AILearningBlock { data_binding: _, model_binding: _, body } => {
        let lowered_body: Result<Vec<Stmt>, String> = body.iter().map(lower_stmt_ast).collect();
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__ai_learning_block".to_string())),
            args: vec![Expr::Object(vec![])], // Placeholder for AI block content
        })
    }
    A::TimeBlock { duration, body } => {
        let duration_expr = if let Some(dur) = duration {
            lower_expr_ast(dur)?
        } else {
            Expr::Lit(Lit::String("auto".to_string()))
        };
        let lowered_body: Result<Vec<Stmt>, String> = body.iter().map(lower_stmt_ast).collect();
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__time_block".to_string())),
            args: vec![duration_expr],
        })
    }
    // Phase 1: import — no-op at stmt level (handled at top level)
    A::ImportDecl { .. } => Stmt::Expr(Expr::Object(vec![])),

    // Phase 1: struct decl — emit a JS class-like object factory
    A::StructDecl { name, fields, .. } => {
        // Emit: let Name = { _create: function(fields...) { return {fields}; } }
        // For now: Stmt::Let with a string marker; codegen handles the real emission
        Stmt::Let {
            name: format!("__struct_{}", name),
            value: Some(Expr::Lit(Lit::String(name.clone()))),
        }
    }

    // Phase 1: enum decl — emit constants
    A::EnumDecl { name, variants, .. } => {
        Stmt::Let {
            name: format!("__enum_{}", name),
            value: Some(Expr::Lit(Lit::String(name.clone()))),
        }
    }

    // Phase 1: impl block at statement level — no-op (hoisted at top level)
    A::ImplBlock { .. } => Stmt::Expr(Expr::Object(vec![])),

    // Phase 1: match expr — lower to nested if-else
    A::MatchExpr { value, arms } => {
        let val_expr = lower_expr_ast(value)?;
        let mut result = Stmt::Expr(Expr::Object(vec![])); // default: no-op
        for arm in arms.iter().rev() {
            use crate::core::ast::MatchPattern;
            let pat_cond = match &arm.pattern {
                MatchPattern::Literal(lit) => {
                    Expr::Binary {
                        left: Box::new(val_expr.clone()),
                        op: BinOp::Eq,
                        right: Box::new(lower_expr_ast(lit)?),
                    }
                }
                MatchPattern::Identifier(_name) => Expr::Lit(Lit::Bool(true)),
                MatchPattern::Wildcard => Expr::Lit(Lit::Bool(true)),
                MatchPattern::EnumVariant { name, .. } => {
                    Expr::Binary {
                        left: Box::new(val_expr.clone()),
                        op: BinOp::Eq,
                        right: Box::new(Expr::Lit(Lit::String(name.clone()))),
                    }
                }
            };
            // Apply match guard if present
            let cond = if let Some(guard) = &arm.guard {
                Expr::Binary {
                    left: Box::new(pat_cond),
                    op: BinOp::And,
                    right: Box::new(lower_expr_ast(guard)?),
                }
            } else {
                pat_cond
            };
            let then_block = lower_block_ast(&arm.body)?;
            result = Stmt::If {
                cond,
                then_block,
                else_block: Some(Block { stmts: vec![result] }),
            };
        }
        result
    }

    // Phase 1: method call
    A::MethodCall { object, method, args } => {
        let obj_expr = lower_expr_ast(object)?;
        let lowered_args: Result<Vec<Expr>, _> = args.iter().map(lower_expr_ast).collect();
        Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(obj_expr),
                property: method.clone(),
            }),
            args: lowered_args?,
        })
    }

    // Phase 1: field access
    A::FieldAccess { object, field } => {
        let obj_expr = lower_expr_ast(object)?;
        Stmt::Expr(Expr::Member {
            object: Box::new(obj_expr),
            property: field.clone(),
        })
    }

    // Phase 1: f-string — lower to string concatenation
    A::FStringLiteral(parts) => {
        use crate::core::ast::FStringPart;
        let mut result = Expr::Lit(Lit::String(String::new()));
        for part in parts {
            let part_expr = match part {
                FStringPart::Literal(s) => Expr::Lit(Lit::String(s.clone())),
                FStringPart::Expr(e) => lower_expr_ast(e)?,
            };
            result = Expr::Binary {
                left: Box::new(result),
                op: BinOp::Add,
                right: Box::new(part_expr),
            };
        }
        Stmt::Expr(result)
    }

    // Phase 1: async function at stmt level — treat as regular function
    A::AsyncFunction { name, params, body, line, column } => {
        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        // AsyncFunctions at statement level: just emit a no-op (top level hoists them)
        Stmt::Expr(Expr::Lit(Lit::String(format!("/* async function {} */", name))))
    }

    // Phase 1: await — pass-through inner expression
    A::AwaitExpr(inner) => Stmt::Expr(lower_expr_ast(inner)?),

    // Phase 1: type annotation wrapper — evaluate inner
    A::TypeAnnotated { expr, .. } => lower_stmt_ast(expr)?,

    // Phase 1: array literal
    A::ArrayLiteral(elements) => {
        let lowered: Result<Vec<Expr>, _> = elements.iter().map(lower_expr_ast).collect();
        Stmt::Expr(Expr::Array(lowered?))
    }

    // Phase 1: null literal
    A::NullLiteral => Stmt::Expr(Expr::Lit(Lit::Null)),

    // P1-4: quantum circuit — execute gates sequentially in the CURRENT scope
    // so qubit declarations are visible to gate calls.
    A::QuantumCircuit { name, gates } => {
        let mut stmts: Vec<Stmt> = Vec::new();
        stmts.push(Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__quantum_circuit_begin".to_string())),
            args: vec![Expr::Lit(Lit::String(name.clone()))],
        }));
        for gate in gates {
            stmts.push(lower_stmt_ast(gate)?);
        }
        stmts.push(Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__quantum_circuit_end".to_string())),
            args: vec![Expr::Lit(Lit::String(name.clone()))],
        }));
        // Stmt::Block executes flat in current scope — no new scope frame,
        // so qubit Let bindings declared in the circuit are visible to gate calls.
        Stmt::Block(stmts)
    }

    // Catch-all for truly unimplemented nodes (should shrink toward zero)
    _ => Stmt::Expr(Expr::Object(vec![])),
    })
}

fn lower_block_ast(n: &crate::core::ast::ASTNode) -> Result<Block, String> {
    use crate::core::ast::ASTNode as A;
    match n {
        A::Block(items) => {
            let mut stmts = Vec::new();
            for it in items {
                stmts.push(lower_stmt_ast(it)?);
            }
            Ok(Block { stmts })
        }
        other => Ok(Block { stmts: vec![lower_stmt_ast(other)?] }),
    }
}

fn lower_stmt_init_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;
    Ok(match n {
    A::VariableDecl { name, value, .. } => Stmt::Let {
            name: name.clone(),
            value: Some(lower_expr_ast(value)?),
        },
        
        // Quantum variable declarations in init position
        A::QuantumVariableDecl { name, binding_type, value, .. } => {
            let quantum_type_comment = match binding_type {
                crate::core::ast::QuantumBindingType::Classical => "Quantum:Classical",
                crate::core::ast::QuantumBindingType::Superposition => "Quantum:Superposition", 
                crate::core::ast::QuantumBindingType::Tensor => "Quantum:Tensor",
                crate::core::ast::QuantumBindingType::Approximation => "Quantum:Approximation",
            };
            
            Stmt::Let {
                name: format!("{} /* {} */", name, quantum_type_comment),
                value: Some(lower_expr_ast(value)?),
            }
        }
        
    A::Assignment { name, value, .. } => Stmt::Assign {
            target: Expr::Ident(name.clone()),
            value: lower_expr_ast(value)?,
        },
        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        _ => Stmt::Expr(lower_expr_ast(n)?),
    })
}

fn lower_expr_ast(n: &crate::core::ast::ASTNode) -> Result<Expr, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Identifier(s) => Expr::Ident(s.clone()),
        A::NumberLiteral(n) => Expr::Lit(Lit::Number(*n)),
        A::StringLiteral(s) => Expr::Lit(Lit::String(s.clone())),
        A::BooleanLiteral(b) => Expr::Lit(Lit::Bool(*b)),

        A::UnaryExpr { op, expr } => Expr::Unary {
            op: map_unop_token(op),
            expr: Box::new(lower_expr_ast(expr)?),
        },

        A::BinaryExpr { op, left, right } => Expr::Binary {
            left: Box::new(lower_expr_ast(left)?),
            op: map_binop_token(op),
            right: Box::new(lower_expr_ast(right)?),
        },

        // Assignment is not an expression in IR; degrade to a no-op value.
        A::Assignment { .. } => Expr::Object(vec![]),

        A::Call { callee, args } => Expr::Call {
            callee: Box::new(lower_expr_ast(callee)?),
            args: args.iter().map(|a| lower_expr_ast(a)).collect::<Result<Vec<_>, _>>()?,
        },

        A::Log(e) => Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(e)?],
        },

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Expr::Call { callee: Box::new(Expr::Ident(fname)), args }
        }

    A::HieroglyphicOp { symbol, args } => Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        },

        A::QuantumState { state, amplitude } => {
            if let Some(amp) = amplitude {
                // For .ai output, include amplitude information
                Expr::Lit(Lit::String(format!("{} /* amplitude: {} */", state, amp)))
            } else {
                Expr::Lit(Lit::String(state.clone()))
            }
        },

    A::IdentifierSpanned { name, .. } => Expr::Ident(name.clone()),

        A::Block(_)
        | A::If { .. }
        | A::While { .. }
        | A::For { .. }
        | A::Function { .. }
        | A::VariableDecl { .. }
        | A::Return(_)
        | A::Program(_) => Expr::Object(vec![]),

        A::Error(msg) => Expr::Lit(Lit::String(format!("/* error: {msg} */"))),
        A::NullLiteral => Expr::Lit(Lit::Null),
        
        // Phase 1 expr nodes
        A::MethodCall { object, method, args } => {
            let obj_e = lower_expr_ast(object)?;
            let arg_exprs: Result<Vec<Expr>, _> = args.iter().map(lower_expr_ast).collect();
            Expr::Call {
                callee: Box::new(Expr::Member { object: Box::new(obj_e), property: method.clone() }),
                args: arg_exprs?,
            }
        }
        A::FieldAccess { object, field } => {
            Expr::Member { object: Box::new(lower_expr_ast(object)?), property: field.clone() }
        }
        A::ArrayLiteral(elems) => {
            let lowered: Result<Vec<Expr>, _> = elems.iter().map(lower_expr_ast).collect();
            Expr::Array(lowered?)
        }
        A::FStringLiteral(parts) => {
            use crate::core::ast::FStringPart;
            let mut result = Expr::Lit(Lit::String(String::new()));
            for part in parts {
                let part_expr = match part {
                    FStringPart::Literal(s) => Expr::Lit(Lit::String(s.clone())),
                    FStringPart::Expr(e) => lower_expr_ast(e)?,
                };
                result = Expr::Binary { left: Box::new(result), op: BinOp::Add, right: Box::new(part_expr) };
            }
            result
        }
        A::AwaitExpr(inner) => lower_expr_ast(inner)?,
        A::TypeAnnotated { expr, .. } => lower_expr_ast(expr)?,
        A::MatchExpr { .. } | A::ImportDecl { .. } | A::StructDecl { .. }
        | A::EnumDecl { .. } | A::ImplBlock { .. } | A::AsyncFunction { .. } => Expr::Object(vec![]),
        
        // Quantum expressions
        A::QuantumBinaryExpr { op, left, right } => {
            let left_expr = lower_expr_ast(left)?;
            let right_expr = lower_expr_ast(right)?;
            let quantum_op = match op {
                crate::core::token::TokenKind::QuantumXor => "__quantum_xor", // ⊕
                crate::core::token::TokenKind::QuantumTensor => "__quantum_tensor", // ⊗
                crate::core::token::TokenKind::SuperpositionState => "__quantum_superposition", // ◊
                crate::core::token::TokenKind::Entangle => "__quantum_entangle", // ∇
                _ => "__quantum_op",
            };
            Expr::Call {
                callee: Box::new(Expr::Ident(quantum_op.to_string())),
                args: vec![left_expr, right_expr],
            }
        }
        A::QuantumIndexAccess { array, index, is_quantum_index } => {
            let array_expr = lower_expr_ast(array)?;
            let index_expr = lower_expr_ast(index)?;
            if *is_quantum_index {
                Expr::Call {
                    callee: Box::new(Expr::Ident("__quantum_index".to_string())),
                    args: vec![array_expr, index_expr],
                }
            } else {
                // Use Call variant for regular indexing as well
                Expr::Call {
                    callee: Box::new(Expr::Ident("__index_access".to_string())),
                    args: vec![array_expr, index_expr],
                }
            }
        }
        // Keep existing wildcard for any remaining unimplemented nodes
        _ => Expr::Lit(Lit::String("/* Quantum AST node not yet implemented in lowering */".to_string())),
    })
}

// =======================
// Operator mapping
// =======================

fn map_binop(op: &str) -> BinOp {
    use BinOp::*;
    match op {
        "+" => Add, "-" => Sub, "*" => Mul, "/" => Div, "%" => Mod,
        "==" => Eq, "!=" => Ne, "<" => Lt, "<=" => Le, ">" => Gt, ">=" => Ge,
        "&&" => And, "||" => Or,
        _ => { eprintln!("[lowering] unknown binop `{}` -> Eq", op); Eq }
    }
}

fn map_unop(op: &str) -> UnOp {
    match op {
        "-" => UnOp::Neg,
        "!" => UnOp::Not,
        _ => { eprintln!("[lowering] unknown unop `{}` -> Not", op); UnOp::Not }
    }
}

#[allow(dead_code)]
fn map_binop_unused(op: &str) -> BinOp { map_binop(op) }

#[allow(dead_code)]
fn map_unop_unused(op: &str) -> UnOp { map_unop(op) }

// Updated to use TokenKind directly instead of the full path
fn map_binop_token(tok: &TokenKind) -> BinOp {
    use BinOp::*;
    match tok {
        TokenKind::Plus => Add,
        TokenKind::Minus => Sub,
        TokenKind::Star => Mul,
        TokenKind::Slash => Div,
        TokenKind::DoubleEquals => Eq,
        TokenKind::NotEquals => Ne,
        TokenKind::LessThan => Lt,
        TokenKind::LessEqual => Le,
        TokenKind::GreaterThan => Gt,
        TokenKind::GreaterEqual => Ge,
        TokenKind::Percent => Mod,
        TokenKind::Bind => Add,    // ↦ Genesis bind/pipe — structural placeholder
        TokenKind::Caret => Mod,   // ^ XOR/exponent — structural placeholder
        _ => { eprintln!("[lowering] unmapped token binop `{:?}` -> Eq", tok); Eq }
    }
}

// Updated to use TokenKind directly instead of the full path
fn map_unop_token(tok: &TokenKind) -> UnOp {
    match tok {
        TokenKind::Minus => UnOp::Neg,
        _ => UnOp::Not,
    }
}

// Quantum op lowering helper - updated to use TokenKind directly
fn map_quantum_op(
    tok: &TokenKind,
    args: &Vec<crate::core::ast::ASTNode>,
) -> Result<(String, Vec<Expr>), String> {
    let fname = match tok {
        TokenKind::Superpose => "superpose",
        TokenKind::Entangle => "entangle",
        TokenKind::Measure => "measure",
        TokenKind::Dod => "dod",
        _ => "qop",
    }
    .to_string();

    let mut lowered = Vec::with_capacity(args.len());
    for a in args {
        lowered.push(lower_expr_ast(a)?);
    }
    Ok((fname, lowered))
}