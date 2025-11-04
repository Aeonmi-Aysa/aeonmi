use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

/// Export an Aeonmi AST program to OpenQASM 2.0 code.
pub fn export_to_qasm(program: &ASTNode) -> String {
    let mut qubit_names: Vec<String> = Vec::new();
    let mut operations: Vec<(TokenKind, Vec<String>)> = Vec::new();

    fn collect_ops(
        node: &ASTNode,
        names: &mut Vec<String>,
        ops: &mut Vec<(TokenKind, Vec<String>)>,
    ) {
        match node {
            ASTNode::Program(items) | ASTNode::Block(items) => {
                for item in items {
                    collect_ops(item, names, ops);
                }
            }
            ASTNode::QuantumOp { op, qubits } => {
                let mut q_names = Vec::new();
                for q in qubits {
                    match q {
                        ASTNode::Identifier(name) => {
                            q_names.push(name.clone());
                            if !names.contains(name) {
                                names.push(name.clone());
                            }
                        }
                        ASTNode::IdentifierSpanned { name, .. } => {
                            q_names.push(name.clone());
                            if !names.contains(name) {
                                names.push(name.clone());
                            }
                        }
                        other => {
                            collect_ops(other, names, ops);
                        }
                    }
                }
                ops.push((op.clone(), q_names));
            }
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_ops(condition, names, ops);
                collect_ops(then_branch, names, ops);
                if let Some(e) = else_branch.as_ref() {
                    collect_ops(e, names, ops);
                }
            }
            ASTNode::While { condition, body } => {
                collect_ops(condition, names, ops);
                collect_ops(body, names, ops);
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = init {
                    collect_ops(init, names, ops);
                }
                if let Some(cond) = condition {
                    collect_ops(cond, names, ops);
                }
                if let Some(incr) = increment {
                    collect_ops(incr, names, ops);
                }
                collect_ops(body, names, ops);
            }
            ASTNode::ForIn { iterable, body, .. } => {
                collect_ops(iterable, names, ops);
                collect_ops(body, names, ops);
            }
            ASTNode::Return(expr) | ASTNode::Log(expr) | ASTNode::UnaryExpr { expr, .. } => {
                collect_ops(expr, names, ops);
            }
            ASTNode::QuantumLoop {
                condition, body, ..
            } => {
                collect_ops(condition, names, ops);
                collect_ops(body, names, ops);
            }
            ASTNode::QuantumIndexAccess { array, index, .. } => {
                collect_ops(array, names, ops);
                collect_ops(index, names, ops);
            }
            ASTNode::QuantumArray { elements, .. } => {
                for elem in elements {
                    collect_ops(elem, names, ops);
                }
            }
            ASTNode::Assignment { value, .. }
            | ASTNode::VariableDecl { value, .. }
            | ASTNode::QuantumVariableDecl { value, .. } => {
                collect_ops(value, names, ops);
            }
            ASTNode::Call { callee, args } => {
                collect_ops(callee, names, ops);
                for arg in args {
                    collect_ops(arg, names, ops);
                }
            }
            ASTNode::BinaryExpr { left, right, .. }
            | ASTNode::QuantumBinaryExpr { left, right, .. } => {
                collect_ops(left, names, ops);
                collect_ops(right, names, ops);
            }
            ASTNode::ProbabilityBranch {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                collect_ops(condition, names, ops);
                collect_ops(then_branch, names, ops);
                if let Some(expr) = else_branch.as_ref() {
                    collect_ops(expr, names, ops);
                }
            }
            ASTNode::SuperpositionSwitch { value, cases } => {
                collect_ops(value, names, ops);
                for case in cases {
                    for stmt in &case.body {
                        collect_ops(stmt, names, ops);
                    }
                }
            }
            ASTNode::QuantumFunction { body, .. }
            | ASTNode::Function { body, .. }
            | ASTNode::FunctionExpr { body, .. }
            | ASTNode::AILearningBlock { body, .. }
            | ASTNode::TimeBlock { body, .. } => {
                for stmt in body {
                    collect_ops(stmt, names, ops);
                }
            }
            ASTNode::QuantumTryCatch {
                attempt_body,
                catch_body,
                success_body,
                ..
            } => {
                for stmt in attempt_body {
                    collect_ops(stmt, names, ops);
                }
                if let Some(items) = catch_body {
                    for stmt in items {
                        collect_ops(stmt, names, ops);
                    }
                }
                if let Some(items) = success_body {
                    for stmt in items {
                        collect_ops(stmt, names, ops);
                    }
                }
            }
            _ => {}
        }
    }

    collect_ops(program, &mut qubit_names, &mut operations);

    qubit_names.sort();
    qubit_names.dedup();

    if qubit_names.is_empty() {
        return String::new();
    }

    let mut qasm = String::new();
    qasm.push_str("OPENQASM 2.0;\n");
    qasm.push_str("include \"qelib1.inc\";\n");
    qasm.push_str(&format!("qreg q[{}];\n", qubit_names.len()));
    qasm.push_str(&format!("creg c[{}];\n", qubit_names.len()));

    for (op_token, qlist) in operations {
        match op_token {
            TokenKind::Superpose if qlist.len() == 1 => {
                if let Some(idx) = qubit_names.iter().position(|name| name == &qlist[0]) {
                    qasm.push_str(&format!("h q[{}];\n", idx));
                }
            }
            TokenKind::Measure if qlist.len() == 1 => {
                if let Some(idx) = qubit_names.iter().position(|name| name == &qlist[0]) {
                    qasm.push_str(&format!("measure q[{}] -> c[{}];\n", idx, idx));
                }
            }
            TokenKind::Entangle if qlist.len() == 2 => {
                if let Some(i0) = qubit_names.iter().position(|name| name == &qlist[0]) {
                    if let Some(i1) = qubit_names.iter().position(|name| name == &qlist[1]) {
                        qasm.push_str(&format!("h q[{}];\n", i0));
                        qasm.push_str(&format!("cx q[{}], q[{}];\n", i0, i1));
                    }
                }
            }
            TokenKind::Dod if qlist.len() == 1 => {
                if let Some(idx) = qubit_names.iter().position(|name| name == &qlist[0]) {
                    qasm.push_str(&format!("x q[{}];\n", idx));
                }
            }
            other => {
                if !qlist.is_empty() {
                    let indices: Vec<String> = qlist
                        .iter()
                        .filter_map(|qname| qubit_names.iter().position(|n| n == qname))
                        .map(|i| format!("q[{}]", i))
                        .collect();
                    if !indices.is_empty() {
                        let gate_name = format!("{:?}", other).to_lowercase();
                        qasm.push_str(&format!("{} {};\n", gate_name, indices.join(", ")));
                    }
                }
            }
        }
    }

    qasm
}
