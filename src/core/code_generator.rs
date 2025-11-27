//! Aeonmi code generation front-end.
//! - Default backend: **JS** (keeps legacy tests green)
//! - Optional backend: **AI** (canonical .ai via AiEmitter)
use crate::core::ai_emitter::AiEmitter;
use crate::core::ast::{ASTNode, StructField};
use crate::core::token::TokenKind;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Js,
    Ai,
    Bytecode,
    Ir,
    Native,
    Qasm,
    WebAssembly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Helper {
    Len,
    Quantum, // Quantum simulation helpers
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
    pub fn new_js() -> Self {
        Self {
            indent: 0,
            backend: Backend::Js,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_bytecode() -> Self {
        Self {
            indent: 0,
            backend: Backend::Bytecode,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_ir() -> Self {
        Self {
            indent: 0,
            backend: Backend::Ir,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_native() -> Self {
        Self {
            indent: 0,
            backend: Backend::Native,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_qasm() -> Self {
        Self {
            indent: 0,
            backend: Backend::Qasm,
            helpers: BTreeSet::new(),
        }
    }
    pub fn new_webassembly() -> Self {
        Self {
            indent: 0,
            backend: Backend::WebAssembly,
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
            Backend::Bytecode => {
                // For bytecode backend, use the bytecode compiler
                // This returns a compiled bytecode chunk that can be executed by the VM
                let compiler = crate::core::bytecode::BytecodeCompiler::new();
                let chunk = compiler.compile(ast);
                Ok(format!(
                    "Bytecode compiled successfully with {} instructions",
                    chunk.code.len()
                ))
            }
            Backend::Ir => {
                // Convert AST to IR and then format as string
                match crate::core::lowering::lower_ast_to_ir(ast, "aeonmi") {
                    Ok(ir_module) => {
                        let mut output = String::new();
                        output.push_str(&format!("module {}\n", ir_module.name));
                        
                        for import in &ir_module.imports {
                            output.push_str(&format!("import {}\n", import.path));
                        }
                        
                        for decl in &ir_module.decls {
                            match decl {
                                crate::core::ir::Decl::Fn(fn_decl) => {
                                    output.push_str(&format!("fn {}({}) {{\n", fn_decl.name, 
                                        fn_decl.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")));
                                    for stmt in &fn_decl.body.stmts {
                                        output.push_str(&format!("  {:?}\n", stmt));
                                    }
                                    output.push_str("}\n");
                                }
                                crate::core::ir::Decl::Let(let_decl) => {
                                    output.push_str(&format!("let {} = {:?}\n", let_decl.name, let_decl.value));
                                }
                                _ => output.push_str(&format!("{:?}\n", decl)),
                            }
                        }
                        Ok(output)
                    }
                    Err(e) => Err(format!("IR lowering failed: {}", e)),
                }
            }
            Backend::Native => {
                // Generate C code that can be compiled to native binary
                let mut output = String::new();
                output.push_str("#include <stdio.h>\n");
                output.push_str("#include <stdlib.h>\n\n");
                output.push_str("int main() {\n");
                output.push_str("    printf(\"Hello from Aeonmi native backend!\\n\");\n");
                output.push_str("    return 0;\n");
                output.push_str("}\n");
                
                Ok(output)
            }
            Backend::Qasm => {
                // Use the existing QASM exporter
                Ok(crate::compiler::qasm_exporter::export_to_qasm(ast))
            }
            Backend::WebAssembly => {
                // Generate basic WebAssembly Text Format (WAT)
                let mut output = String::new();
                output.push_str("(module\n");
                output.push_str("  (func $main (result i32)\n");
                
                // Simple WAT generation - just return 42 for now
                // TODO: Implement full WAT generation from AST
                output.push_str("    i32.const 42\n");
                output.push_str("  )\n");
                output.push_str("  (export \"main\" (func $main))\n");
                output.push_str(")\n");
                
                Ok(output)
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
                        .map(|p| {
                            let mut frag = String::new();
                            if p.is_variadic {
                                frag.push_str("...");
                            }
                            frag.push_str(&p.name);
                            if let Some(default) = &p.default {
                                frag.push_str(" = ");
                                frag.push_str(&self.emit_expr_js(default));
                            }
                            frag
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit_js(&block));
                s
            }
            ASTNode::ClassDecl {
                name,
                superclass,
                methods,
                ..
            } => self.emit_class_decl(name, superclass, methods),
            ASTNode::StructDecl { name, fields, .. } => self.emit_struct_decl(name, fields),
            ASTNode::EnumDecl { name, variants, .. } => self.emit_enum_decl(name, variants),
            ASTNode::TraitDecl { name, methods, .. } => self.emit_trait_decl(name, methods),
            ASTNode::ImplBlock {
                trait_name,
                type_name,
                methods,
                ..
            } => self.emit_impl_block(trait_name, type_name, methods),
            ASTNode::Return(expr) => format!("return {};\n", self.emit_expr_js(expr)),
            ASTNode::Log(expr) => format!("console.log({});\n", self.emit_expr_js(expr)),
            ASTNode::Break => "break;\n".to_string(),
            ASTNode::Continue => "continue;\n".to_string(),
            ASTNode::Assignment { target, value, .. } => {
                format!("{} = {};\n", self.emit_expr_js(target), self.emit_expr_js(value))
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
            ASTNode::ForIn {
                binding,
                iterable,
                body,
            } => {
                let decl_kw = if binding.is_mutable { "let" } else { "const" };
                let mut s = String::new();
                s.push_str(&format!(
                    "for ({} {} of {}) ",
                    decl_kw,
                    binding.name,
                    self.emit_expr_js(iterable)
                ));
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
            | ASTNode::BooleanLiteral(_)
            | ASTNode::ArrayLiteral(_)
            | ASTNode::IndexExpr { .. } => format!("{};\n", self.emit_expr_js(node)),
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
                format!("{}({});\n", opname, args)
            }
            ASTNode::QuantumVariableDecl {
                name,
                binding_type,
                value,
                ..
            } => {
                let js_value = self.emit_expr_js(value);
                let binding_comment = match binding_type {
                    crate::core::ast::QuantumBindingType::Classical => {
                        "// Classical quantum variable"
                    }
                    crate::core::ast::QuantumBindingType::Superposition => {
                        "// Superposition quantum variable"
                    }
                    crate::core::ast::QuantumBindingType::Tensor => {
                        "// Tensor product quantum variable"
                    }
                    crate::core::ast::QuantumBindingType::Approximation => {
                        "// Quantum approximation variable"
                    }
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
            ASTNode::QuantumIndexAccess {
                array,
                index,
                is_quantum_index,
            } => {
                let array_js = self.emit_expr_js(array);
                let index_js = self.emit_expr_js(index);
                if *is_quantum_index {
                    format!("__quantum.quantumIndex({}, {})\n", array_js, index_js)
                } else {
                    format!("{}[{}]\n", array_js, index_js)
                }
            }
            ASTNode::QuantumFunction {
                func_type,
                name,
                params,
                body,
                ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let params_str = param_names.join(", ");
                let body_js = body
                    .iter()
                    .map(|stmt| self.emit_js(stmt))
                    .collect::<String>();
                let quantum_marker = match func_type {
                    crate::core::ast::QuantumFunctionType::Quantum => "/* ⊙ Quantum Function */",
                    crate::core::ast::QuantumFunctionType::Classical => {
                        "/* ◯ Classical Function */"
                    }
                    crate::core::ast::QuantumFunctionType::AINeural => {
                        "/* 🧠 AI Neural Function */"
                    }
                };
                format!(
                    "function {}({}) {{ {}\n{}}}\n",
                    name, params_str, quantum_marker, body_js
                )
            }
            ASTNode::ProbabilityBranch {
                condition,
                probability,
                then_branch,
                else_branch,
            } => {
                // Mark that we need quantum helpers
                self.helpers.insert(Helper::Quantum);
                
                // Generate probabilistic branch using quantum evaluation
                let prob_value = probability.unwrap_or(0.5);
                let mut s = String::new();
                
                // Generate condition expression
                let condition_js = self.emit_expr_js(condition);
                
                // Use __quantum.evaluate() with probability comment
                s.push_str(&format!("// Probability: {:.2}%\n", prob_value * 100.0));
                s.push_str(&format!("if (__quantum.evaluate({})) ", condition_js));
                s.push_str(&self.wrap_stmt_js(then_branch));
                
                if let Some(else_stmt) = else_branch {
                    s.push_str(" else ");
                    s.push_str(&self.wrap_stmt_js(else_stmt));
                }
                s.push('\n');
                s
            }
            ASTNode::QuantumLoop {
                condition,
                body,
                decoherence_threshold,
            } => {
                // Mark that we need quantum helpers
                self.helpers.insert(Helper::Quantum);
                
                // Generate quantum loop with decoherence tracking
                let mut s = String::new();
                
                // Set up decoherence threshold
                let threshold = decoherence_threshold.unwrap_or(1.0);
                
                s.push_str(&format!("// Decoherence threshold: {}\n", threshold));
                s.push_str("let __qLoopState = { coherent: true };\n");
                s.push_str("while (__qLoopState.coherent) {\n");
                self.indent += 1;
                
                // Check condition if provided
                s.push_str(&self.indent_str());
                let condition_js = self.emit_expr_js(condition);
                s.push_str(&format!("if (!({})) {{\n", condition_js));
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str("break;\n");
                self.indent -= 1;
                s.push_str(&self.indent_str());
                s.push_str("}\n");
                
                // Loop body
                s.push_str(&self.indent_str());
                let body_js = self.emit_js(body);
                s.push_str(&body_js.trim());
                if !body_js.ends_with('\n') {
                    s.push('\n');
                }
                
                // Evaluate quantum loop state
                s.push_str(&self.indent_str());
                s.push_str("__qLoopState = __quantum.evaluateLoop(__qLoopState);\n");
                
                self.indent -= 1;
                s.push_str("}\n");
                s
            }
            ASTNode::SuperpositionSwitch { value, cases } => {
                let value_js = self.emit_expr_js(value);
                let cases_js = cases
                    .iter()
                    .map(|case| {
                        format!(
                            "// Superposition case: {} -> quantum state handling",
                            case.pattern
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "// ◇ Superposition Switch\n__quantum.superpositionSwitch({}) {{\n{}\n}}\n",
                    value_js, cases_js
                )
            }
            ASTNode::MatchExpr { .. } => format!("{};\n", self.emit_expr_js(node)),
            ASTNode::QuantumTryCatch {
                attempt_body,
                error_probability,
                catch_body,
                success_body,
            } => {
                // Mark that we need quantum helpers
                self.helpers.insert(Helper::Quantum);
                
                let mut s = String::new();
                
                // Track quantum failure
                s.push_str("let __quantumFailed = false;\n");
                s.push_str("try {\n");
                self.indent += 1;
                
                // Attempt block
                for stmt in attempt_body {
                    s.push_str(&self.indent_str());
                    s.push_str(&self.emit_js(stmt));
                }
                
                self.indent -= 1;
                s.push_str("} catch (e) {\n");
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str("__quantumFailed = true;\n");
                self.indent -= 1;
                s.push_str("}\n");
                
                // Check for probabilistic quantum failure
                let prob = error_probability.unwrap_or(0.0);
                if prob > 0.0 {
                    s.push_str(&format!("if (!__quantumFailed && Math.random() < {}) {{\n", prob));
                    self.indent += 1;
                    s.push_str(&self.indent_str());
                    s.push_str("__quantumFailed = true;\n");
                    self.indent -= 1;
                    s.push_str("}\n");
                }
                
                // Catch or success block
                s.push_str("if (__quantumFailed) {\n");
                self.indent += 1;
                if let Some(catch_stmts) = catch_body {
                    for stmt in catch_stmts {
                        s.push_str(&self.indent_str());
                        s.push_str(&self.emit_js(stmt));
                    }
                }
                self.indent -= 1;
                s.push_str("}");
                
                if let Some(success_stmts) = success_body {
                    s.push_str(" else {\n");
                    self.indent += 1;
                    for stmt in success_stmts {
                        s.push_str(&self.indent_str());
                        s.push_str(&self.emit_js(stmt));
                    }
                    self.indent -= 1;
                    s.push_str("}");
                }
                s.push('\n');
                s
            }
            ASTNode::AILearningBlock {
                data_binding,
                model_binding,
                body,
            } => {
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
                let body_js = body
                    .iter()
                    .map(|stmt| self.emit_js(stmt))
                    .collect::<String>();
                format!(
                    "// AI Learning Block\n{}\n{}\n__quantum.aiLearningBlock(() => {{\n{}\n}});\n",
                    data_comment, model_comment, body_js
                )
            }
            ASTNode::TimeBlock { duration, body } => {
                let duration_js = duration
                    .as_ref()
                    .map(|d| self.emit_expr_js(d))
                    .unwrap_or("auto".to_string());
                let body_js = body
                    .iter()
                    .map(|stmt| self.emit_js(stmt))
                    .collect::<String>();
                format!("__time.block({}) {{ {}\n}}\n", duration_js, body_js)
            }
            // Keep existing wildcard for any remaining unimplemented nodes
            _ => format!("/* Quantum AST node not yet implemented in code generator */\n"),
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
            ASTNode::StringLiteral(s) => format!("\"{}\"", s),
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
            ASTNode::FunctionExpr {
                name, params, body, ..
            } => {
                let mut signature = String::new();
                signature.push_str("function ");
                if let Some(n) = name {
                    signature.push_str(n);
                }
                signature.push('(');
                signature.push_str(
                    &params
                        .iter()
                        .map(|p| {
                            let mut frag = String::new();
                            if p.is_variadic {
                                frag.push_str("...");
                            }
                            frag.push_str(&p.name);
                            if let Some(default) = &p.default {
                                frag.push_str(" = ");
                                frag.push_str(&self.emit_expr_js(default));
                            }
                            frag
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                signature.push(')');
                let body_js = body
                    .iter()
                    .map(|stmt| self.emit_js(stmt))
                    .collect::<String>();
                format!("({} {{ {} }})", signature, body_js)
            }
            ASTNode::Assignment { target, value, .. } => {
                format!("{} = {}", self.emit_expr_js(target), self.emit_expr_js(value))
            }
            ASTNode::ArrayLiteral(elements) => {
                let contents = elements
                    .iter()
                    .map(|el| self.emit_expr_js(el))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", contents)
            }
            ASTNode::ObjectLiteral(fields) => {
                let entries = fields
                    .iter()
                    .map(|(k, v)| {
                        let key = if k.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
                            k.clone()
                        } else {
                            format!("\"{}\"", k)
                        };
                        format!("{}: {}", key, self.emit_expr_js(v))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", entries)
            }
            ASTNode::StructLiteral { type_name, fields } => {
                // Generate a call to the struct factory function
                let entries = fields
                    .iter()
                    .map(|(k, v)| {
                        let key = if k.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
                            k.clone()
                        } else {
                            format!("\"{}\"", k)
                        };
                        format!("{}: {}", key, self.emit_expr_js(v))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // Call the struct factory function: StructName({ field1: val1, field2: val2 })
                format!("{}({{{}}})", type_name, entries)
            }
            ASTNode::IndexExpr { array, index } => {
                format!("{}[{}]", self.emit_expr_js(array), self.emit_expr_js(index))
            }
            ASTNode::FieldAccess { object, field } => {
                if field.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
                    format!("{}.{}", self.emit_expr_js(object), field)
                } else {
                    format!("{}[\"{}\"]", self.emit_expr_js(object), field)
                }
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
            ASTNode::QuantumIndexAccess {
                array,
                index,
                is_quantum_index,
            } => {
                let array_js = self.emit_expr_js(array);
                let index_js = self.emit_expr_js(index);
                if *is_quantum_index {
                    format!("__quantum.quantumIndex({}, {})", array_js, index_js)
                } else {
                    format!("{}[{}]", array_js, index_js)
                }
            }
            ASTNode::MatchExpr { value, arms, .. } => {
                let saved_indent = self.indent;
                let mut out = String::new();
                out.push_str("(() => {\n");
                self.indent += 1;
                out.push_str(&self.indent_str());
                out.push_str(&format!(
                    "const __matchValue = {};\n",
                    self.emit_expr_js(value)
                ));
                let mut default_expr: Option<String> = None;
                for arm in arms {
                    let is_wildcard = matches!(arm.pattern, ASTNode::Identifier(ref name) if name == "_")
                        || matches!(
                            arm.pattern,
                            ASTNode::IdentifierSpanned { ref name, .. } if name == "_"
                        );
                    let body_js = self.emit_expr_js(&arm.body);
                    if is_wildcard {
                        default_expr = Some(body_js);
                        continue;
                    }
                    let pattern_js = self.emit_expr_js(&arm.pattern);
                    out.push_str(&self.indent_str());
                    out.push_str(&format!("if (__matchValue === {}) {{\n", pattern_js));
                    self.indent += 1;
                    if let Some(guard) = &arm.guard {
                        let guard_js = self.emit_expr_js(guard);
                        out.push_str(&self.indent_str());
                        out.push_str(&format!("if ({}) {{\n", guard_js));
                        self.indent += 1;
                        out.push_str(&self.indent_str());
                        out.push_str(&format!("return {};\n", body_js));
                        self.indent -= 1;
                        out.push_str(&self.indent_str());
                        out.push_str("}\n");
                    } else {
                        out.push_str(&self.indent_str());
                        out.push_str(&format!("return {};\n", body_js));
                    }
                    self.indent -= 1;
                    out.push_str(&self.indent_str());
                    out.push_str("}\n");
                }
                out.push_str(&self.indent_str());
                if let Some(default_expr) = default_expr {
                    out.push_str(&format!("return {};\n", default_expr));
                } else {
                    out.push_str("return undefined;\n");
                }
                self.indent -= 1;
                out.push_str(&self.indent_str());
                out.push_str("})()");
                self.indent = saved_indent;
                out
            }
            _ => "/*expr*/".into(),
        }
    }
    fn emit_class_decl(
        &mut self,
        name: &str,
        superclass: &Option<String>,
        methods: &[ASTNode],
    ) -> String {
        let mut s = String::new();
        s.push_str("class ");
        s.push_str(name);
        if let Some(base) = superclass {
            if !base.is_empty() {
                s.push_str(" extends ");
                s.push_str(base);
            }
        }
        s.push_str(" {\n");
        self.indent += 1;
        if methods.is_empty() {
            s.push_str(&self.indent_str());
            s.push_str("constructor() {}\n");
        } else {
            for method in methods {
                if let Some(method_js) = self.emit_method_js(method) {
                    for line in method_js.trim_end().lines() {
                        s.push_str(&self.indent_str());
                        s.push_str(line);
                        s.push('\n');
                    }
                }
            }
        }
        self.indent -= 1;
        s.push_str("}\n");
        s
    }

    fn emit_struct_decl(&mut self, name: &str, fields: &[StructField]) -> String {
        let mut s = String::new();
        s.push_str(&format!("function {}(init = {{}}) {{\n", name));
        self.indent += 1;
        s.push_str(&self.indent_str());
        s.push_str("return {\n");
        self.indent += 1;
        if fields.is_empty() {
            s.push_str(&self.indent_str());
            s.push_str("/* empty struct */\n");
        } else {
            for (idx, field) in fields.iter().enumerate() {
                s.push_str(&self.indent_str());
                // The 'default' field actually contains type annotation, not a default value
                // So we just use undefined as the default
                s.push_str(&format!(
                    "{}: (init.{} !== undefined ? init.{} : undefined)",
                    field.name, field.name, field.name
                ));
                if idx + 1 != fields.len() {
                    s.push(',');
                }
                s.push('\n');
            }
        }
        self.indent -= 1;
        s.push_str(&self.indent_str());
        s.push_str("};\n");
        self.indent -= 1;
        s.push_str(&self.indent_str());
        s.push_str("}\n");
        s
    }

    fn emit_enum_decl(&mut self, name: &str, variants: &[String]) -> String {
        let mut s = String::new();
        s.push_str(&format!("const {} = {{\n", name));
        self.indent += 1;
        for (idx, variant) in variants.iter().enumerate() {
            s.push_str(&self.indent_str());
            // For now, simple enums use Symbol for unique identity
            s.push_str(&format!("{}: Symbol(\"{}.{}\")", variant, name, variant));
            if idx + 1 != variants.len() {
                s.push(',');
            }
            s.push('\n');
        }
        self.indent -= 1;
        s.push_str("};\n");
        s
    }

    fn emit_trait_decl(&mut self, name: &str, methods: &[ASTNode]) -> String {
        format!("/* trait {} with {} method(s) */\n", name, methods.len())
    }

    fn emit_impl_block(
        &mut self,
        trait_name: &Option<String>,
        type_name: &str,
        methods: &[ASTNode],
    ) -> String {
        // Generate prototype method assignments for both trait and inherent impls
        let mut s = String::new();
        
        // Optional comment for trait implementations
        if let Some(trait_name) = trait_name {
            s.push_str(&format!("// impl {} for {}\n", trait_name, type_name));
        }
        
        // Generate prototype assignments for each method
        for method in methods {
            if let ASTNode::Function {
                name, params, body, ..
            } = method
            {
                s.push_str(&format!("{}.prototype.{} = function(", type_name, name));
                let params_str = params
                    .iter()
                    .map(|p| {
                        let mut frag = String::new();
                        if p.is_variadic {
                            frag.push_str("...");
                        }
                        frag.push_str(&p.name);
                        if let Some(default) = &p.default {
                            frag.push_str(" = ");
                            frag.push_str(&self.emit_expr_js(default));
                        }
                        frag
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                s.push_str(&params_str);
                s.push_str(") ");
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit_js(&block));
            }
        }
        s
    }

    fn emit_method_js(&mut self, method: &ASTNode) -> Option<String> {
        if let ASTNode::Function {
            name, params, body, ..
        } = method
        {
            let params_str = params
                .iter()
                .map(|p| {
                    let mut frag = String::new();
                    if p.is_variadic {
                        frag.push_str("...");
                    }
                    frag.push_str(&p.name);
                    if let Some(default) = &p.default {
                        frag.push_str(" = ");
                        frag.push_str(&self.emit_expr_js(default));
                    }
                    frag
                })
                .collect::<Vec<_>>()
                .join(", ");
            let block = ASTNode::Block(body.clone());
            let mut out = String::new();
            out.push_str(name);
            out.push('(');
            out.push_str(&params_str);
            out.push_str(") ");
            out.push_str(&self.emit_js(&block));
            Some(out)
        } else {
            None
        }
    }
    fn map_helper(&mut self, name: &str) -> Option<String> {
        match name {
            "len" => {
                self.helpers.insert(Helper::Len);
                Some("__aeonmi_len".to_string())
            }
            _ => None,
        }
    }
    fn render_helpers(helpers: &BTreeSet<Helper>) -> String {
        let mut prelude = String::new();
        for (idx, helper) in helpers.iter().enumerate() {
            if idx > 0 {
                prelude.push('\n');
            }
            match helper {
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
                Helper::Quantum => {
                    // Quantum simulation helpers using JavaScript objects and Math.random()
                    prelude.push_str("// Quantum simulation state management\n");
                    prelude.push_str("let __entanglementGroups = [];\n\n");
                    
                    // superpose: Create qubit in superposition
                    prelude.push_str("const __aeonmi_superpose = (prob = 0.5) => {\n");
                    prelude.push_str("    return {\n");
                    prelude.push_str("        __qubit: true,\n");
                    prelude.push_str("        __prob: prob,\n");
                    prelude.push_str("        __entGroup: null,\n");
                    prelude.push_str("        __value: null\n");
                    prelude.push_str("    };\n");
                    prelude.push_str("};\n\n");
                    
                    // entangle: Link qubits so they collapse together
                    prelude.push_str("const __aeonmi_entangle = (...qubits) => {\n");
                    prelude.push_str("    const groupId = __entanglementGroups.length;\n");
                    prelude.push_str("    __entanglementGroups.push({ qubits, collapsed: false, value: null });\n");
                    prelude.push_str("    qubits.forEach(q => { if (q.__qubit) q.__entGroup = groupId; });\n");
                    prelude.push_str("    return qubits;\n");
                    prelude.push_str("};\n\n");
                    
                    // measure: Collapse qubit to definite value
                    prelude.push_str("const __aeonmi_measure = (qubit) => {\n");
                    prelude.push_str("    if (!qubit || !qubit.__qubit) return qubit;\n");
                    prelude.push_str("    if (qubit.__value !== null) return qubit.__value;\n");
                    prelude.push_str("    \n");
                    prelude.push_str("    if (qubit.__entGroup !== null) {\n");
                    prelude.push_str("        const group = __entanglementGroups[qubit.__entGroup];\n");
                    prelude.push_str("        if (group.collapsed) {\n");
                    prelude.push_str("            qubit.__value = group.value;\n");
                    prelude.push_str("            return group.value;\n");
                    prelude.push_str("        }\n");
                    prelude.push_str("        const value = Math.random() < qubit.__prob ? 1 : 0;\n");
                    prelude.push_str("        group.collapsed = true;\n");
                    prelude.push_str("        group.value = value;\n");
                    prelude.push_str("        group.qubits.forEach(q => q.__value = value);\n");
                    prelude.push_str("        return value;\n");
                    prelude.push_str("    }\n");
                    prelude.push_str("    \n");
                    prelude.push_str("    const value = Math.random() < qubit.__prob ? 1 : 0;\n");
                    prelude.push_str("    qubit.__value = value;\n");
                    prelude.push_str("    return value;\n");
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
            // Only match the variants that exist in TokenKind
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
        let prog = ASTNode::Program(vec![ASTNode::new_assignment(ASTNode::Identifier("x".into()), call)]);
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
