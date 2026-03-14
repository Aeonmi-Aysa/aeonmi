//! Aeonmi code generation front-end.
//! - Default backend: **JS** (keeps legacy tests green)
//! - Optional backend: **AI** (canonical .ai via AiEmitter)
use crate::core::ai_emitter::AiEmitter;
use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Js,
    Ai,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Helper {
    Len,
    QuantumRuntime,
    GlyphRuntime,
}

pub struct CodeGenerator {
    indent: usize,
    backend: Backend,
    helpers: BTreeSet<Helper>,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            indent: 0,
            backend: Backend::Js,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_ai() -> Self {
        Self {
            indent: 0,
            backend: Backend::Ai,
            helpers: BTreeSet::new(),
        }
    }
    pub fn generate(&mut self, ast: &ASTNode) -> Result<String, String> {
        self.generate_with_backend(ast, self.backend)
    }
    pub fn generate_with_backend(
        &mut self,
        ast: &ASTNode,
        backend: Backend,
    ) -> Result<String, String> {
        match backend {
            Backend::Js => Ok(self.emit_js(ast)),
            Backend::Ai => {
                let mut emitter = AiEmitter::new();
                emitter
                    .generate(ast)
                    .map_err(|e| format!("AiEmitter error: {e}"))
            }
        }
    }
    // JS BACKEND
    fn emit_js(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Program(items) => {
                let saved_helpers = std::mem::take(&mut self.helpers);
                let mut body = String::new();
                for item in items {
                    body.push_str(&self.emit_js(item));
                    if !body.ends_with('\n') {
                        body.push('\n');
                    }
                }

                let needed_helpers = std::mem::take(&mut self.helpers);
                let mut out = String::new();
                if !needed_helpers.is_empty() {
                    out.push_str(&Self::render_helpers(&needed_helpers));
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                out.push_str(&body);
                self.helpers = saved_helpers;
                out
            }
            ASTNode::Block(items) => {
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                for it in items {
                    s.push_str(&self.indent_str());
                    s.push_str(&self.emit_js(it));
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                }
                self.indent -= 1;
                s.push_str("}\n");
                s
            }
            ASTNode::VariableDecl { name, value, .. } => {
                format!("let {} = {};\n", name, self.emit_expr_js(value))
            }
            ASTNode::Function {
                name, params, body, ..
            } => {
                let mut s = String::new();
                s.push_str(&format!(
                    "function {}({}) ",
                    name,
                    params
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit_js(&block));
                s
            }
            ASTNode::Return(expr) => format!("return {};\n", self.emit_expr_js(expr)),
            ASTNode::Log(expr) => format!("console.log({});\n", self.emit_expr_js(expr)),
            ASTNode::Assignment { name, value, .. } => {
                format!("{} = {};\n", name, self.emit_expr_js(value))
            }
            ASTNode::Call { .. } => format!("{};\n", self.emit_expr_js(node)),
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut s = String::new();
                // Tests expect exactly one extra pair around the emitted binary expression (which is already parenthesized)
                s.push_str(&format!("if ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(then_branch));
                if let Some(e) = else_branch {
                    s.push_str(" else ");
                    s.push_str(&self.wrap_stmt_js(e));
                }
                s.push('\n');
                s
            }
            ASTNode::While { condition, body } => {
                let mut s = String::new();
                s.push_str(&format!("while ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(body));
                s.push('\n');
                s
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                let init_s = if let Some(i) = init.as_ref() {
                    Self::strip_trailing(self.emit_js(i))
                } else {
                    String::new()
                };
                let cond_s = if let Some(c) = condition.as_ref() {
                    self.emit_expr_js(c)
                } else {
                    String::new()
                };
                let incr_s = if let Some(inc) = increment.as_ref() {
                    Self::strip_trailing(self.emit_js(inc))
                } else {
                    String::new()
                };
                let mut s = String::new();
                s.push_str(&format!("for ({}; {}; {}) ", init_s, cond_s, incr_s));
                s.push_str(&self.wrap_stmt_js(body));
                s.push('\n');
                s
            }
            // P1-34: for var in iterable — emit as JS for...of
            ASTNode::ForIn { var, iterable, body } => {
                let iter_s = self.emit_expr_js(iterable);
                let mut s = String::new();
                s.push_str(&format!("for (const {} of {}) ", var, iter_s));
                s.push_str(&self.wrap_stmt_js(body));
                s.push('\n');
                s
            }
            ASTNode::BinaryExpr { .. }
            | ASTNode::UnaryExpr { .. }
            | ASTNode::Identifier(_)
            | ASTNode::IdentifierSpanned { .. }
            | ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_) => format!("{};\n", self.emit_expr_js(node)),
            ASTNode::QuantumOp { op, qubits } => {
                self.helpers.insert(Helper::QuantumRuntime);
                let opname = match op {
                    TokenKind::Superpose => "superpose",
                    TokenKind::Entangle => "entangle",
                    TokenKind::Measure => "measure",
                    TokenKind::Dod => "dod",
                    _ => "qop",
                };
                let args = qubits
                    .iter()
                    .map(|q| self.emit_expr_js(q))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({});\n", opname, args)
            }
            ASTNode::QuantumVariableDecl { name, binding_type, value, .. } => {
                let js_value = self.emit_expr_js(value);
                let binding_comment = match binding_type {
                    crate::core::ast::QuantumBindingType::Classical => "// Classical quantum variable",
                    crate::core::ast::QuantumBindingType::Superposition => "// Superposition quantum variable", 
                    crate::core::ast::QuantumBindingType::Tensor => "// Tensor product quantum variable",
                    crate::core::ast::QuantumBindingType::Approximation => "// Quantum approximation variable",
                };
                format!("let {} = {}; {}\n", name, js_value, binding_comment)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
                    .map(|e| self.emit_expr_js(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("__glyph('{}', {});\n", symbol, a)
            }
            ASTNode::Error(msg) => format!("/* ERROR NODE: {} */\n", msg),
            
            // Quantum AST Nodes Implementation
            ASTNode::QuantumBinaryExpr { op, left, right } => {
                let left_js = self.emit_expr_js(left);
                let right_js = self.emit_expr_js(right);
                let quantum_op = match op {
                    crate::core::token::TokenKind::QuantumXor => "quantumXor", // ⊕
                    crate::core::token::TokenKind::QuantumTensor => "quantumTensor", // ⊗
                    crate::core::token::TokenKind::SuperpositionState => "quantumSuperposition", // ◊
                    crate::core::token::TokenKind::Entangle => "quantumEntangle", // ∇
                    _ => "quantumOp",
                };
                format!("__quantum.{}({}, {});\n", quantum_op, left_js, right_js)
            }
            ASTNode::QuantumIndexAccess { array, index, is_quantum_index } => {
                let array_js = self.emit_expr_js(array);
                let index_js = self.emit_expr_js(index);
                if *is_quantum_index {
                    format!("__quantum.quantumIndex({}, {})\n", array_js, index_js)
                } else {
                    format!("{}[{}]\n", array_js, index_js)
                }
            }
            ASTNode::QuantumFunction { func_type, name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let params_str = param_names.join(", ");
                let body_js = body.iter().map(|stmt| self.emit_js(stmt)).collect::<String>();
                let quantum_marker = match func_type {
                    crate::core::ast::QuantumFunctionType::Quantum => "// ⊙ Quantum Function",
                    crate::core::ast::QuantumFunctionType::Classical => "// ◯ Classical Function",
                    crate::core::ast::QuantumFunctionType::AINeural => "// 🧠 AI Neural Function",
                };
                format!("function {}({}) {{ {}\n{}}}\n", name, params_str, quantum_marker, body_js)
            }
            ASTNode::ProbabilityBranch { condition, probability, then_branch, else_branch } => {
                let condition_js = self.emit_expr_js(condition);
                let then_js = self.emit_js(then_branch);
                let prob_comment = if let Some(p) = probability {
                    format!("// Probability: {:.2}%", p * 100.0)
                } else {
                    "// Quantum probability".to_string()
                };
                if let Some(else_stmt) = else_branch {
                    let else_js = self.emit_js(else_stmt);
                    format!("if (__quantum.evaluate({})) {{ {}\n{}\n}} else {{ {}\n{}\n}}\n", 
                           condition_js, prob_comment, then_js, prob_comment, else_js)
                } else {
                    format!("if (__quantum.evaluate({})) {{ {}\n{}\n}}\n", 
                           condition_js, prob_comment, then_js)
                }
            }
            ASTNode::QuantumLoop { condition, body, decoherence_threshold } => {
                let condition_js = self.emit_expr_js(condition);
                let body_js = self.emit_js(body);
                let decoherence_comment = if let Some(threshold) = decoherence_threshold {
                    format!("// Decoherence threshold: {:.3}", threshold)
                } else {
                    "// Quantum loop with decoherence protection".to_string()
                };
                format!("while (__quantum.evaluateLoop({})) {{ {}\n{}\n}}\n", 
                       condition_js, decoherence_comment, body_js)
            }
            ASTNode::SuperpositionSwitch { value, cases } => {
                let value_js = self.emit_expr_js(value);
                let cases_js = cases.iter().map(|case| {
                    format!("// Superposition case: {} -> quantum state handling", case.pattern)
                }).collect::<Vec<_>>().join("\n");
                format!("__quantum.superpositionSwitch({}) {{\n{}\n}}\n", value_js, cases_js)
            }
            ASTNode::QuantumTryCatch { attempt_body, error_probability, catch_body, success_body: _ } => {
                let attempt_js = attempt_body.iter().map(|stmt| self.emit_js(stmt)).collect::<String>();
                let prob_comment = if let Some(p) = error_probability {
                    format!("// Error probability: {:.2}%", p * 100.0)
                } else {
                    "// Quantum error correction".to_string()
                };
                let catch_js = if let Some(catch_stmts) = catch_body {
                    catch_stmts.iter().map(|stmt| self.emit_js(stmt)).collect::<String>()
                } else {
                    "// Default quantum error handling\n".to_string()
                };
                format!("try {{ {}\n{}\n}} catch (quantum_error) {{ {}\n{}\n}}\n", 
                       prob_comment, attempt_js, prob_comment, catch_js)
            }
            ASTNode::AILearningBlock { data_binding, model_binding, body } => {
                let data_comment = if let Some(data) = data_binding {
                    format!("// Data binding: {}", data)
                } else {
                    "// No data binding specified".to_string()
                };
                let model_comment = if let Some(model) = model_binding {
                    format!("// Model binding: {}", model)
                } else {
                    "// No model binding specified".to_string()
                };
                let body_js = body.iter().map(|stmt| self.emit_js(stmt)).collect::<String>();
                format!("// AI Learning Block\n{}\n{}\n__quantum.aiLearningBlock(() => {{\n{}\n}});\n",
                       data_comment, model_comment, body_js)
            }
            ASTNode::TimeBlock { duration, body } => {
                let duration_js = duration.as_ref().map(|d| self.emit_expr_js(d)).unwrap_or("auto".to_string());
                let body_js = body.iter().map(|stmt| self.emit_js(stmt)).collect::<String>();
                format!("__time.block({}) {{ {}\n}}\n", duration_js, body_js)
            }
            // Phase 1: import
            ASTNode::ImportDecl { names, path } => {
                format!("// import {{ {} }} from \"{}\"\n", names.join(", "), path)
            }
            // Phase 1: struct decl
            ASTNode::StructDecl { name, fields, is_quantum: _ } => {
                let _fields_js = fields.iter().map(|f| format!("{}:undefined", f.name)).collect::<Vec<_>>().join(", ");
                let params_js = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", ");
                let assigns = fields.iter().map(|f| format!("this.{} = {};", f.name, f.name)).collect::<Vec<_>>().join(" ");
                format!("function {}({}) {{ {} }}\n", name, params_js, assigns)
            }
            // Phase 1: enum decl — emit as JS object so Enum.Variant works
            ASTNode::EnumDecl { name, variants, .. } => {
                let fields = variants.iter().enumerate().map(|(i, v)| {
                    format!("{}: {}", v.name, i)
                }).collect::<Vec<_>>().join(", ");
                format!("const {} = {{ {} }};\n", name, fields)
            }
            // Phase 1: impl block — emit methods as named functions
            ASTNode::ImplBlock { target, methods } => {
                methods.iter().map(|m| {
                    match m {
                        ASTNode::Function { name, params, body, .. } => {
                            let fn_name = format!("{}_{}", target, name);
                            let params_str = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                            let body_js = body.iter().map(|s| self.emit_js(s)).collect::<String>();
                            format!("function {}({}) {{\n{}}}\n", fn_name, params_str, body_js)
                        }
                        _ => String::new()
                    }
                }).collect::<String>()
            }
            // Phase 1: async function
            ASTNode::AsyncFunction { name, params, body, .. } => {
                let params_str = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                let body_js = body.iter().map(|s| self.emit_js(s)).collect::<String>();
                format!("async function {}({}) {{\n{}}}\n", name, params_str, body_js)
            }
            // Phase 1: match expr — emit as nested if-else
            ASTNode::MatchExpr { value, arms } => {
                let val_js = self.emit_expr_js(value);
                let tmp = "__match_val";
                let mut out = format!("let {} = {};\n", tmp, val_js);
                for (i, arm) in arms.iter().enumerate() {
                    let pat_cond = match &arm.pattern {
                        crate::core::ast::MatchPattern::Literal(lit) => format!("({} === {})", tmp, self.emit_expr_js(lit)),
                        crate::core::ast::MatchPattern::Wildcard => "true".to_string(),
                        crate::core::ast::MatchPattern::Identifier(_) => "true".to_string(),
                        crate::core::ast::MatchPattern::EnumVariant { name, .. } => format!("({} === \"{}\")", tmp, name),
                    };
                    let cond = if let Some(guard) = &arm.guard {
                        format!("({} && {})", pat_cond, self.emit_expr_js(guard))
                    } else {
                        pat_cond
                    };
                    if i == 0 { out.push_str(&format!("if ({}) ", cond)); }
                    else { out.push_str(&format!("else if ({}) ", cond)); }
                    out.push_str(&self.wrap_stmt_js(&arm.body));
                    out.push('\n');
                }
                out
            }
            // Phase 1: method call
            ASTNode::MethodCall { object, method, args } => {
                let obj_js = self.emit_expr_js(object);
                let args_js = args.iter().map(|a| self.emit_expr_js(a)).collect::<Vec<_>>().join(", ");
                format!("{}.{}({});\n", obj_js, method, args_js)
            }
            // Phase 1: field access
            ASTNode::FieldAccess { object, field } => {
                format!("{}.{};\n", self.emit_expr_js(object), field)
            }
            // Phase 1: field assignment
            ASTNode::FieldAssign { object, field, value } => {
                format!("{}.{} = {};\n", self.emit_expr_js(object), field, self.emit_expr_js(value))
            }
            // Phase 1: array literal
            ASTNode::ArrayLiteral(elements) => {
                let elems_js = elements.iter().map(|e| self.emit_expr_js(e)).collect::<Vec<_>>().join(", ");
                format!("[{}];\n", elems_js)
            }
            // Phase 1: f-string
            ASTNode::FStringLiteral(parts) => {
                use crate::core::ast::FStringPart;
                let inner = parts.iter().map(|p| match p {
                    FStringPart::Literal(s) => format!("`{}`", s.replace('`', "\\`")),
                    FStringPart::Expr(e) => format!("String({})", self.emit_expr_js(e)),
                }).collect::<Vec<_>>().join(" + ");
                format!("{};\n", inner)
            }
            // Phase 1: await
            ASTNode::AwaitExpr(inner) => format!("await {};\n", self.emit_expr_js(inner)),
            // Phase 1: null
            ASTNode::NullLiteral => "null;\n".to_string(),
            // P1-4: quantum circuit
            ASTNode::QuantumCircuit { name, gates } => {
                self.helpers.insert(Helper::QuantumRuntime);
                let gates_js = gates.iter().map(|g| {
                    let s = self.emit_js(g);
                    format!("  {}", s.trim_start())
                }).collect::<String>();
                format!(
                    "// quantum circuit: {}\n(function __circuit_{}() {{\n{}}})(  );\n",
                    name, name, gates_js
                )
            }
            // Closure as statement
            ASTNode::Closure { params, body } => {
                let params_str = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                let body_js = body.iter().map(|s| self.emit_js(s)).collect::<String>();
                format!("(({}) => {{\n{}}});\n", params_str, body_js)
            }
            // Catch-all (should shrink to zero)
            _ => String::new(),
        }
    }
    fn emit_expr_js(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Identifier(s) => s.clone(),
            ASTNode::IdentifierSpanned { name, .. } => name.clone(),
            ASTNode::NumberLiteral(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            ASTNode::StringLiteral(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
                format!("\"{}\"", escaped)
            }
            ASTNode::BooleanLiteral(b) => format!("{}", b),
            ASTNode::BinaryExpr { op, left, right } => {
                format!(
                    "({} {} {})",
                    self.emit_expr_js(left),
                    self.op_str(op),
                    self.emit_expr_js(right)
                )
            }
            ASTNode::UnaryExpr { op, expr } => {
                format!("{}{}", self.op_str(op), self.emit_expr_js(expr))
            }
            ASTNode::Call { callee, args } => {
                let mapped = match &**callee {
                    ASTNode::Identifier(name) => self.map_helper(name),
                    ASTNode::IdentifierSpanned { name, .. } => self.map_helper(name),
                    _ => None,
                };
                let c = mapped.unwrap_or_else(|| self.emit_expr_js(callee));
                let a = args
                    .iter()
                    .map(|x| self.emit_expr_js(x))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", c, a)
            }
            ASTNode::Assignment { name, value, .. } => {
                format!("{} = {}", name, self.emit_expr_js(value))
            }
            ASTNode::QuantumOp { op, qubits } => {
                let opname = match op {
                    TokenKind::Superpose => "superpose",
                    TokenKind::Entangle => "entangle",
                    TokenKind::Measure => "measure",
                    TokenKind::Dod => "dod",
                    _ => "qop",
                };
                let args = qubits
                    .iter()
                    .map(|q| self.emit_expr_js(q))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", opname, args)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
                    .map(|e| self.emit_expr_js(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("__glyph('{}', {})", symbol, a)
            }
            ASTNode::QuantumState { state, amplitude } => {
                if let Some(amp) = amplitude {
                    format!("\"{}\" /* amplitude: {} */", state, amp)
                } else {
                    format!("\"{}\"", state)
                }
            }
            // Quantum expressions
            ASTNode::QuantumBinaryExpr { op, left, right } => {
                let left_js = self.emit_expr_js(left);
                let right_js = self.emit_expr_js(right);
                let quantum_op = match op {
                    crate::core::token::TokenKind::QuantumXor => "quantumXor", // ⊕
                    crate::core::token::TokenKind::QuantumTensor => "quantumTensor", // ⊗
                    crate::core::token::TokenKind::SuperpositionState => "quantumSuperposition", // ◊
                    crate::core::token::TokenKind::Entangle => "quantumEntangle", // ∇
                    _ => "quantumOp",
                };
                format!("__quantum.{}({}, {})", quantum_op, left_js, right_js)
            }
            ASTNode::QuantumIndexAccess { array, index, is_quantum_index } => {
                let array_js = self.emit_expr_js(array);
                let index_js = self.emit_expr_js(index);
                if *is_quantum_index {
                    format!("__quantum.quantumIndex({}, {})", array_js, index_js)
                } else {
                    format!("{}[{}]", array_js, index_js)
                }
            }
            // Phase 1 expr nodes
            ASTNode::MethodCall { object, method, args } => {
                let obj_js = self.emit_expr_js(object);
                let args_js = args.iter().map(|a| self.emit_expr_js(a)).collect::<Vec<_>>().join(", ");
                format!("{}.{}({})", obj_js, method, args_js)
            }
            ASTNode::FieldAccess { object, field } => {
                format!("{}.{}", self.emit_expr_js(object), field)
            }
            ASTNode::ArrayLiteral(elements) => {
                let elems_js = elements.iter().map(|e| self.emit_expr_js(e)).collect::<Vec<_>>().join(", ");
                format!("[{}]", elems_js)
            }
            ASTNode::FStringLiteral(parts) => {
                use crate::core::ast::FStringPart;
                parts.iter().map(|p| match p {
                    FStringPart::Literal(s) => format!("`{}`", s.replace('`', "\\`")),
                    FStringPart::Expr(e) => format!("String({})", self.emit_expr_js(e)),
                }).collect::<Vec<_>>().join(" + ")
            }
            ASTNode::AwaitExpr(inner) => format!("await {}", self.emit_expr_js(inner)),
            ASTNode::NullLiteral => "null".to_string(),
            // If as expression: emit as IIFE that returns its value
            ASTNode::If { condition, then_branch, else_branch } => {
                let cond = self.emit_expr_js(condition);
                // Extract last-statement return value from branch
                let make_returnable_block = |cg: &mut CodeGenerator, node: &ASTNode| -> String {
                    match node {
                        ASTNode::Block(stmts) if !stmts.is_empty() => {
                            let mut out = String::new();
                            for s in &stmts[..stmts.len()-1] { out.push_str(&cg.emit_js(s)); }
                            out.push_str(&format!("return ({});\n", cg.emit_expr_js(stmts.last().unwrap())));
                            out
                        }
                        _ => format!("return ({});\n", cg.emit_expr_js(node)),
                    }
                };
                let then_js = make_returnable_block(self, then_branch);
                let else_js = match else_branch {
                    Some(e) => format!(" else {{\n{}}}", make_returnable_block(self, e)),
                    None => String::new(),
                };
                format!("((() => {{ if ({cond}) {{\n{then_js}}}{else_js} return undefined; }})())")
            }
            ASTNode::Closure { params, body } => {
                let params_str = params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
                let body_js = body.iter().map(|s| self.emit_js(s)).collect::<String>();
                format!("(({}) => {{\n{}}})", params_str, body_js)
            }
            _ => "undefined".into(),
        }
    }
    fn map_helper(&mut self, name: &str) -> Option<String> {
        match name {
            "len" => {
                self.helpers.insert(Helper::Len);
                Some("__aeonmi_len".to_string())
            }
            // Quantum builtins — inject runtime preamble, keep the name
            "superpose" | "measure" | "entangle" | "dod" | "apply_gate" => {
                self.helpers.insert(Helper::QuantumRuntime);
                Some(name.to_string())
            }
            _ => {
                // Detect hieroglyphic / non-ASCII identifiers → route to __glyph
                if name.chars().any(|c| c > '\u{07FF}' && !c.is_ascii()) {
                    self.helpers.insert(Helper::GlyphRuntime);
                    return Some(format!("__glyph.bind(null, '{}')", name));
                }
                None
            }
        }
    }
    fn render_helpers(helpers: &BTreeSet<Helper>) -> String {
        let mut prelude = String::new();
        for (idx, helper) in helpers.iter().enumerate() {
            if idx > 0 {
                prelude.push('\n');
            }
            match helper {
                Helper::QuantumRuntime => {
                    prelude.push_str("// Aeonmi Quantum Runtime\n");
                    prelude.push_str("const H = 'H';\n");
                    prelude.push_str("const X = 'X';\n");
                    prelude.push_str("const Y = 'Y';\n");
                    prelude.push_str("const Z = 'Z';\n");
                    prelude.push_str("const CNOT = 'CNOT';\n");
                    prelude.push_str("const HADAMARD = 'H';\n");
                    prelude.push_str("const NOT = 'X';\n");
                    prelude.push_str("const __quantum_state = {};\n");
                    prelude.push_str("function superpose(q) {\n");
                    prelude.push_str("  __quantum_state[q] = { state: [Math.SQRT1_2, Math.SQRT1_2], superposed: true };\n");
                    prelude.push_str("  return __quantum_state[q];\n");
                    prelude.push_str("}\n");
                    prelude.push_str("function measure(q) {\n");
                    prelude.push_str("  const s = __quantum_state[q];\n");
                    prelude.push_str("  if (!s) return 0;\n");
                    prelude.push_str("  const r = Math.random() < (s.state[0] ** 2) ? 0 : 1;\n");
                    prelude.push_str("  __quantum_state[q] = { state: r === 0 ? [1,0] : [0,1], superposed: false };\n");
                    prelude.push_str("  return r;\n");
                    prelude.push_str("}\n");
                    prelude.push_str("function entangle(q1, q2) {\n");
                    prelude.push_str("  __quantum_state[q2] = __quantum_state[q1] || { state: [Math.SQRT1_2, Math.SQRT1_2], superposed: true };\n");
                    prelude.push_str("  return [__quantum_state[q1], __quantum_state[q2]];\n");
                    prelude.push_str("}\n");
                    prelude.push_str("function apply_gate(q, gate) {\n");
                    prelude.push_str("  if (!__quantum_state[q]) __quantum_state[q] = { state: [1, 0], superposed: false };\n");
                    prelude.push_str("  const s = __quantum_state[q];\n");
                    prelude.push_str("  if (gate === 'H') { s.state = [Math.SQRT1_2 * (s.state[0] + s.state[1]), Math.SQRT1_2 * (s.state[0] - s.state[1])]; s.superposed = true; }\n");
                    prelude.push_str("  else if (gate === 'X') { s.state = [s.state[1], s.state[0]]; }\n");
                    prelude.push_str("  else if (gate === 'Y') { s.state = [-s.state[1], s.state[0]]; }\n");
                    prelude.push_str("  else if (gate === 'Z') { s.state = [s.state[0], -s.state[1]]; }\n");
                    prelude.push_str("  else if (gate === 'CNOT') { /* CNOT needs 2 qubits — no-op in single-qubit apply_gate */ }\n");
                    prelude.push_str("  return s;\n");
                    prelude.push_str("}\n");
                    prelude.push_str("function dod(q) { return measure(q); }\n");
                }
                Helper::GlyphRuntime => {
                    prelude.push_str("// Aeonmi Glyph Runtime\n");
                    prelude.push_str("function __glyph(symbol, ...args) {\n");
                    prelude.push_str("  console.log('[GLYPH] ' + symbol + ' invoked with: ' + JSON.stringify(args));\n");
                    prelude.push_str("  return { symbol, args, quantum: true };\n");
                    prelude.push_str("}\n");
                }
                Helper::Len => {
                    prelude.push_str("const __aeonmi_len = (value) => {\n");
                    prelude.push_str(
                        "    if (typeof value === \"string\") { return value.length; }\n",
                    );
                    prelude.push_str("    if (Array.isArray(value)) { return value.length; }\n");
                    prelude.push_str(
                        "    if (value && typeof value === \"object\") { return Object.keys(value).length; }\n",
                    );
                    prelude
                        .push_str("    if (value === null || value === undefined) { return 0; }\n");
                    prelude.push_str("    throw new Error(\"len: unsupported type\");\n");
                    prelude.push_str("};\n");
                }
            }
        }
        prelude
    }
    /// Returns a JS statement block string **without** a trailing newline.
    fn wrap_stmt_js(&mut self, n: &ASTNode) -> String {
        match n {
            ASTNode::Block(_) => {
                let mut b = self.emit_js(n);
                if b.ends_with('\n') {
                    b.pop();
                }
                b
            }
            _ => {
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str(&self.emit_js(n));
                self.indent -= 1;
                s.push('}');
                s
            }
        }
    }
    fn op_str(&self, op: &TokenKind) -> &'static str {
        match op {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Equals => "=",
            TokenKind::DoubleEquals => "==",
            TokenKind::NotEquals => "!=",
            TokenKind::LessThan => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::GreaterThan => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::Bang => "!",
            _ => "/*op*/",
        }
    }
    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }
    fn strip_trailing(mut s: String) -> String {
        if s.ends_with('\n') {
            s.pop();
        }
        if s.ends_with(';') {
            s.pop();
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::ASTNode;

    #[test]
    fn gen_let_and_log_js() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(42.0)),
            ASTNode::new_log(ASTNode::Identifier("x".into())),
        ]);
        let mut g = CodeGenerator::new(); // JS default
        let js = g.generate(&ast).unwrap();
        assert!(js.contains("let x = 42;"));
        assert!(js.contains("console.log(x);"));
    }
    #[test]
    fn gen_assignment_and_call_js() {
        let call = ASTNode::new_call(
            ASTNode::Identifier("add".into()),
            vec![ASTNode::NumberLiteral(2.0), ASTNode::NumberLiteral(3.0)],
        );
        let prog = ASTNode::Program(vec![ASTNode::new_assignment("x", call)]);
        let mut g = CodeGenerator::new(); // JS default
        let js = g.generate(&prog).unwrap();
        assert!(js.contains("x = add(2, 3);"));
    }
    #[test]
    fn gen_minimal_ai_backend() {
        // This test requires a mock or real AiEmitter implementation.
        // If not available, adjust accordingly.
        let ast = ASTNode::Program(vec![ASTNode::new_variable_decl(
            "x",
            ASTNode::NumberLiteral(1.0),
        )]);
        let mut g = CodeGenerator::new_ai();
        let out = g.generate(&ast).unwrap();
        assert!(out.contains("x") && (out.contains("1") || out.contains("1.0")));
    }
}
