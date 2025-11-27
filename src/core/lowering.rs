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

    // No explicit import nodes yet; keep empty and deterministic.
    let imports: Vec<Import> = Vec::new();

    let mut decls: Vec<Decl> = Vec::new();
    let mut main_stmts: Vec<Stmt> = Vec::new();

    for item in items {
        match item {
            ASTNode::Function {
                name: fn_name,
                params,
                body,
                ..
            } => {
                let lowered_params: Vec<FnParam> = params
                    .iter()
                    .map(|p| lower_fn_param(p))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in body {
                    stmts.push(lower_stmt_ast(&stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params: lowered_params,
                    body: Block { stmts },
                }));
            }
            ASTNode::VariableDecl { name, value, .. } => {
                let expr = lower_expr_ast(&value)?;
                decls.push(Decl::Let(LetDecl {
                    name,
                    value: Some(expr),
                }));
            }
            ASTNode::QuantumVariableDecl {
                name,
                binding_type,
                value,
                ..
            } => {
                let expr = lower_expr_ast(&value)?;
                decls.push(Decl::QuantumLet(QuantumLetDecl {
                    name,
                    binding: map_binding(binding_type),
                    value: Some(expr),
                }));
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

    let mut m = Module {
        name: name.to_string(),
        imports,
        decls,
    };
    m.imports.sort_by(|a, b| {
        (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref()))
    });
    m.decls.sort_by(|a, b| {
        use std::cmp::Ordering;
        let kind_order = decl_kind_rank(a).cmp(&decl_kind_rank(b));
        if kind_order != Ordering::Equal {
            return kind_order;
        }
        match (a, b) {
            (Decl::Fn(_), Decl::Fn(_)) => a.name().cmp(b.name()),
            _ => Ordering::Equal,
        }
    });
    Ok(m)
}

fn decl_kind_rank(decl: &Decl) -> u8 {
    match decl {
        Decl::Const(_) => 0,
        Decl::Fn(_) => 1,
        Decl::Let(_) => 2,
        Decl::QuantumLet(_) => 2,
    }
}

fn lower_stmt_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Block(items) => {
            // Blocks are handled by `lower_block_ast` at call sites; produce a no-op here.
            let _ = items;
            Stmt::Expr(Expr::Object(vec![]))
        }

        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        A::Log(expr) => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(expr)?],
        }),
        A::Assignment { target, value, .. } => {
            match &**target {
                A::Identifier(name) => Stmt::Assign {
                    target: Expr::Ident(name.clone()),
                    value: lower_expr_ast(value)?,
                },
                A::IdentifierSpanned { name, .. } => Stmt::Assign {
                    target: Expr::Ident(name.clone()),
                    value: lower_expr_ast(value)?,
                },
                _ => return Err(format!("Complex assignment targets not yet supported: {:?}", target)),
            }
        },

        A::Call { .. }
        | A::BinaryExpr { .. }
        | A::UnaryExpr { .. }
        | A::FunctionExpr { .. }
        | A::Identifier(_)
        | A::NumberLiteral(_)
        | A::StringLiteral(_)
        | A::BooleanLiteral(_)
        | A::ArrayLiteral(_)
        | A::ObjectLiteral(_)
        | A::StructLiteral { .. }
        | A::IndexExpr { .. }
        | A::FieldAccess { .. } => Stmt::Expr(lower_expr_ast(n)?),

        A::If {
            condition,
            then_branch,
            else_branch,
        } => Stmt::If {
            cond: lower_expr_ast(condition)?,
            then_block: lower_block_ast(then_branch)?,
            else_block: else_branch
                .as_ref()
                .map(|e| lower_block_ast(e))
                .transpose()?,
        },

        A::While { condition, body } => Stmt::While {
            cond: lower_expr_ast(condition)?,
            body: lower_block_ast(body)?,
        },

        A::For {
            init,
            condition,
            increment,
            body,
        } => {
            let init_stmt = init.as_ref().map(|b| lower_stmt_init_ast(b)).transpose()?;
            Stmt::For {
                init: init_stmt.map(Box::new),
                cond: condition.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                step: increment.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                body: lower_block_ast(body)?,
            }
        }

        // Decls at statement position
        A::VariableDecl { name, value, .. } => Stmt::Let {
            name: name.clone(),
            value: Some(lower_expr_ast(value)?),
        },

        // Quantum variable declarations
        A::QuantumVariableDecl {
            name,
            binding_type,
            value,
            ..
        } => {
            let binding = map_binding(binding_type.clone());
            Stmt::QuantumLet {
                name: name.clone(),
                binding,
                value: lower_expr_ast(value)?,
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
        A::QuantumIndexAccess {
            array,
            index,
            is_quantum_index,
        } => {
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
        A::QuantumFunction {
            func_type,
            name,
            params: _params,
            body,
            ..
        } => {
            let mut _lowered_body = Vec::new();

            // Add quantum function type as a comment in the first statement
            let quantum_marker = match func_type {
                crate::core::ast::QuantumFunctionType::Quantum => "/* ⊙ Quantum Function */", // ⊙
                crate::core::ast::QuantumFunctionType::Classical => "/* ◉ Classical Function */", // ◉
                crate::core::ast::QuantumFunctionType::AINeural => "/* ⫸ AI Neural Function */", // ⫸
            };

            _lowered_body.push(Stmt::Expr(Expr::Lit(Lit::String(
                quantum_marker.to_string(),
            ))));

            for stmt in body {
                _lowered_body.push(lower_stmt_ast(stmt)?);
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
        A::ProbabilityBranch {
            condition,
            probability,
            then_branch,
            else_branch,
        } => Stmt::ProbabilityBranch {
            condition: lower_expr_ast(condition)?,
            probability: *probability,
            then_block: lower_block_ast(then_branch)?,
            else_block: if let Some(else_br) = else_branch {
                Some(lower_block_ast(else_br)?)
            } else {
                None
            },
        },
        A::QuantumLoop {
            condition,
            body,
            decoherence_threshold,
        } => {
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
        A::QuantumTryCatch {
            attempt_body: _attempt_body,
            error_probability,
            catch_body: _catch_body,
            success_body: _,
        } => {
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
        A::AILearningBlock {
            data_binding: _,
            model_binding: _,
            body,
        } => {
            let _lowered_body: Result<Vec<Stmt>, String> =
                body.iter().map(lower_stmt_ast).collect();
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
            let _lowered_body: Result<Vec<Stmt>, String> =
                body.iter().map(lower_stmt_ast).collect();
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("__time_block".to_string())),
                args: vec![duration_expr],
            })
        }
        // Keep existing wildcard for any remaining unimplemented nodes
        _ => Stmt::Expr(Expr::Lit(Lit::String(
            "/* Quantum AST node not yet implemented in lowering */".to_string(),
        ))),
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
        other => Ok(Block {
            stmts: vec![lower_stmt_ast(other)?],
        }),
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
        A::QuantumVariableDecl {
            name,
            binding_type,
            value,
            ..
        } => Stmt::QuantumLet {
            name: name.clone(),
            binding: map_binding(binding_type.clone()),
            value: lower_expr_ast(value)?,
        },

        A::Assignment { target, value, .. } => {
            match &**target {
                A::Identifier(name) => Stmt::Assign {
                    target: Expr::Ident(name.clone()),
                    value: lower_expr_ast(value)?,
                },
                A::IdentifierSpanned { name, .. } => Stmt::Assign {
                    target: Expr::Ident(name.clone()),
                    value: lower_expr_ast(value)?,
                },
                _ => return Err(format!("Complex assignment targets not yet supported: {:?}", target)),
            }
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
            args: args
                .iter()
                .map(|a| lower_expr_ast(a))
                .collect::<Result<Vec<_>, _>>()?,
        },

        A::Log(e) => Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(e)?],
        },

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Expr::Call {
                callee: Box::new(Expr::Ident(fname)),
                args,
            }
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

        A::QuantumState { state, amplitude } => Expr::QuantumState {
            label: state.clone(),
            amplitude: *amplitude,
        },

        A::IdentifierSpanned { name, .. } => Expr::Ident(name.clone()),

        A::ArrayLiteral(elements) => Expr::Array(
            elements
                .iter()
                .map(|el| lower_expr_ast(el))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        A::ObjectLiteral(fields) => Expr::Object(
            fields
                .iter()
                .map(|(k, v)| Ok((k.clone(), lower_expr_ast(v)?)))
                .collect::<Result<Vec<_>, String>>()?,
        ),
        A::StructLiteral { type_name, fields } => {
            let mut lowered_fields = Vec::with_capacity(fields.len() + 1);
            lowered_fields.push(("__type".into(), Expr::Lit(Lit::String(type_name.clone()))));
            for (k, v) in fields {
                lowered_fields.push((k.clone(), lower_expr_ast(v)?));
            }
            Expr::Object(lowered_fields)
        }
        A::QuantumArray {
            elements,
            dimensions: _,
            is_superposition,
        } => Expr::QuantumArray {
            elements: elements
                .iter()
                .map(|el| lower_expr_ast(el))
                .collect::<Result<Vec<_>, _>>()?,
            is_superposition: *is_superposition,
        },
        A::IndexExpr { array, index } => Expr::Index {
            target: Box::new(lower_expr_ast(array)?),
            index: Box::new(lower_expr_ast(index)?),
        },
        A::FieldAccess { object, field } => Expr::Member {
            object: Box::new(lower_expr_ast(object)?),
            field: field.clone(),
        },
        A::FunctionExpr {
            name, params, body, ..
        } => {
            let lowered_params = params
                .iter()
                .map(|p| lower_fn_param(p))
                .collect::<Result<Vec<_>, _>>()?;
            let mut stmts = Vec::new();
            for stmt in body {
                stmts.push(lower_stmt_ast(stmt)?);
            }
            Expr::Lambda {
                name: name.clone(),
                params: lowered_params,
                body: Block { stmts },
            }
        }

        A::Block(_)
        | A::If { .. }
        | A::While { .. }
        | A::For { .. }
        | A::Function { .. }
        | A::VariableDecl { .. }
        | A::Return(_)
        | A::Program(_) => Expr::Object(vec![]),

        A::Error(msg) => Expr::Lit(Lit::String(format!("/* error: {msg} */"))),

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
        A::QuantumIndexAccess {
            array,
            index,
            is_quantum_index,
        } => {
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
        _ => Expr::Lit(Lit::String(
            "/* Quantum AST node not yet implemented in lowering */".to_string(),
        )),
    })
}

fn lower_fn_param(param: &crate::core::ast::FunctionParam) -> Result<FnParam, String> {
    let default = match &param.default {
        Some(expr) => Some(lower_expr_ast(expr)?),
        None => None,
    };
    Ok(FnParam {
        name: param.name.clone(),
        default,
        is_variadic: param.is_variadic,
    })
}

fn map_binding(binding: crate::core::ast::QuantumBindingType) -> crate::core::ir::QuantumBinding {
    use crate::core::ast::QuantumBindingType as Src;
    use crate::core::ir::QuantumBinding as Dest;

    match binding {
        Src::Classical => Dest::Classical,
        Src::Superposition => Dest::Superposition,
        Src::Tensor => Dest::Tensor,
        Src::Approximation => Dest::Approximation,
    }
}

// =======================
// Operator mapping
// =======================

fn map_binop(op: &str) -> BinOp {
    use BinOp::*;
    match op {
        "+" => Add,
        "-" => Sub,
        "*" => Mul,
        "/" => Div,
        "%" => Mod,
        "==" => Eq,
        "!=" => Ne,
        "<" => Lt,
        "<=" => Le,
        ">" => Gt,
        ">=" => Ge,
        "&&" => And,
        "||" => Or,
        _ => {
            eprintln!("[lowering] unknown binop `{}` -> Eq", op);
            Eq
        }
    }
}

fn map_unop(op: &str) -> UnOp {
    match op {
        "-" => UnOp::Neg,
        "!" => UnOp::Not,
        _ => {
            eprintln!("[lowering] unknown unop `{}` -> Not", op);
            UnOp::Not
        }
    }
}

#[allow(dead_code)]
fn map_binop_unused(op: &str) -> BinOp {
    map_binop(op)
}

#[allow(dead_code)]
fn map_unop_unused(op: &str) -> UnOp {
    map_unop(op)
}

// Updated to use TokenKind directly instead of the full path
fn map_binop_token(tok: &TokenKind) -> BinOp {
    use BinOp::*;
    match tok {
        TokenKind::Plus => Add,
        TokenKind::Minus => Sub,
        TokenKind::Star => Mul,
        TokenKind::Slash => Div,
        TokenKind::Percent => Mod,
        TokenKind::DoubleEquals => Eq,
        TokenKind::NotEquals => Ne,
        TokenKind::LessThan => Lt,
        TokenKind::LessEqual => Le,
        TokenKind::GreaterThan => Gt,
        TokenKind::GreaterEqual => Ge,
        _ => {
            eprintln!("[lowering] unmapped token binop `{:?}` -> Eq", tok);
            Eq
        }
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
        TokenKind::Entangle | TokenKind::Dod => "entangle",
        TokenKind::Measure => "measure",
        _ => "qop",
    }
    .to_string();

    let mut lowered = Vec::with_capacity(args.len());
    for a in args {
        lowered.push(lower_expr_ast(a)?);
    }
    Ok((fname, lowered))
}
