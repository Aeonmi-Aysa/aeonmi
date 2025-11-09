// src/core/semantic_analyzer.rs
//! Minimal semantic analyzer:
//! - Tracks variable declarations per scope
//! - Errors on re-declaration in same scope (existing behavior)
//! - NEW: Errors on assignment to undeclared identifier
//! Next steps (planned incremental expansion):
//! 1. Track function call sites to emit warning for unused private (non-exported) functions.
//! 2. Basic type tagging (number, bool, string) and arithmetic / comparison operand checks.
//! 3. Simple return path consistency: warn if some paths lack return in a function that returns early elsewhere.
//! 4. Coercion rules scaffold (e.g. number <-> string in concatenation) with warnings.
//! 5. Quantum / glyph op arity validation.

use crate::core::ast::{ASTNode, FunctionParam};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SemanticDiagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub len: usize,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ValueType {
    Number,
    String,
    Bool,
    Object,
    Array,
    Function,
    Qubit,
    Unknown,
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::Unknown
    }
}

impl ValueType {
    fn as_str(&self) -> &'static str {
        match self {
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Bool => "bool",
            ValueType::Object => "object",
            ValueType::Array => "array",
            ValueType::Function => "function",
            ValueType::Qubit => "qubit",
            ValueType::Unknown => "unknown",
        }
    }

    fn from_annotation(annotation: &str) -> Option<ValueType> {
        let normalized = annotation.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "bool" | "boolean" => Some(ValueType::Bool),
            "number" | "float" | "int" | "integer" => Some(ValueType::Number),
            "string" => Some(ValueType::String),
            "qubit" => Some(ValueType::Qubit),
            _ => None,
        }
    }
}

#[derive(Default)]
struct VarInfo {
    line: usize,
    column: usize,
    used: bool,
    ty: ValueType,
}

// Trait method signature for trait enforcement
#[derive(Debug, Clone)]
struct TraitMethodSignature {
    name: String,
    line: usize,
    column: usize,
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashSet<String>>,
    var_meta: Vec<std::collections::HashMap<String, VarInfo>>, // parallel stack with metadata
    functions: HashMap<String, (usize, usize)>, // track function declarations (line,column) for duplicate detection
    used_functions: HashSet<String>,            // function call sites
    errors: Vec<String>,                        // legacy string list for existing callers
    diags: Vec<SemanticDiagnostic>,             // unified diagnostics (errors + warnings)
    has_main_function: bool,                    // track if a main function is declared
    loop_depth: usize,                          // track loop nesting depth for break/continue validation
    traits: HashMap<String, Vec<TraitMethodSignature>>, // trait name -> required methods
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
            var_meta: vec![std::collections::HashMap::new()],
            errors: vec![],
            diags: vec![],
            functions: HashMap::new(),
            used_functions: HashSet::new(),
            has_main_function: false,
            loop_depth: 0,
            traits: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> Result<(), String> {
        self.visit(ast, false);
        self.post_pass();
        self.flush_unused_warnings();
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("\n"))
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn analyze_with_spans(&mut self, ast: &ASTNode) -> Vec<SemanticDiagnostic> {
        self.visit(ast, true);
        self.post_pass();
        self.flush_unused_warnings();
        self.diags.clone()
    }

    fn post_pass(&mut self) {
        // Check for main function requirement
        if !self.has_main_function && !self.functions.is_empty() {
            self.errors
                .push("Program must have a main() entry point function".to_string());
            self.diags.push(SemanticDiagnostic {
                message: "Program must have a main() entry point function".to_string(),
                line: 1,
                column: 1,
                len: 1,
                severity: Severity::Error,
            });
        }

        // Placeholder for future multi-pass checks (e.g., unused function detection)
        // Currently detects functions never referenced (simple heuristic: name never marked used as identifier)
        // This is conservative and may false-positive for indirect calls.
        for (name, (line, column)) in self.functions.clone() {
            // clone to avoid borrow issues
            // skip if any scope recorded it as used identifier
            if !self.used_functions.contains(&name) {
                self.diags.push(SemanticDiagnostic {
                    message: format!("Unused function '{name}'"),
                    line,
                    column,
                    len: name.len().max(1),
                    severity: Severity::Warning,
                });
            }
        }
    }

    // identifier_was_used removed (replaced by used_functions set)

    fn predeclare_functions(&mut self, items: &[ASTNode]) {
        for item in items {
            if let ASTNode::Function {
                name, line, column, ..
            } = item
            {
                if !self.functions.contains_key(name) {
                    self.functions.insert(name.clone(), (*line, *column));
                    if name == "main" {
                        self.has_main_function = true;
                    }
                }
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashSet::new());
        self.var_meta.push(std::collections::HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
        if let Some(map) = self.var_meta.pop() {
            // Emit warnings for unused variables in this scope
            for (name, info) in map.into_iter() {
                if !info.used {
                    let msg = format!("Unused variable '{}'", name);
                    self.diags.push(SemanticDiagnostic {
                        message: msg,
                        line: info.line,
                        column: info.column,
                        len: name.len().max(1),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    fn declare(&mut self, name: &str, line: Option<usize>, column: Option<usize>) {
        let scope = self.scopes.last_mut().unwrap();
        let meta = self.var_meta.last_mut().unwrap();
        if scope.contains(name) {
            let msg = format!("Redeclaration of '{}'", name);
            self.errors.push(msg.clone());
            if let (Some(l), Some(c)) = (line, column) {
                self.diags.push(SemanticDiagnostic {
                    message: msg,
                    line: l,
                    column: c,
                    len: name.len().max(1),
                    severity: Severity::Error,
                });
            }
        } else {
            scope.insert(name.to_string());
            if let (Some(l), Some(c)) = (line, column) {
                meta.insert(
                    name.to_string(),
                    VarInfo {
                        line: l,
                        column: c,
                        used: false,
                        ty: ValueType::Unknown,
                    },
                );
            } else {
                meta.insert(
                    name.to_string(),
                    VarInfo {
                        line: 0,
                        column: 0,
                        used: false,
                        ty: ValueType::Unknown,
                    },
                );
            }
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    fn visit(&mut self, node: &ASTNode, capture: bool) {
        match node {
            ASTNode::Program(items) => {
                self.predeclare_functions(items);
                for it in items {
                    self.visit(it, capture);
                }
            }
            ASTNode::Block(items) => {
                self.begin_scope();
                for it in items {
                    self.visit(it, capture);
                }
                self.end_scope();
            }
            ASTNode::Function {
                name,
                line,
                column,
                params,
                body,
                return_type: _,
            } => {
                if let Some((prev_l, prev_c)) = self.functions.get(name) {
                    if *prev_l != *line || *prev_c != *column {
                        let msg =
                            format!("Duplicate function '{name}' (previous at {prev_l}:{prev_c})");
                        self.push_error_at(
                            msg,
                            Some(*line),
                            Some(*column),
                            Some(name.len().max(1)),
                            capture,
                        );
                    }
                } else {
                    self.functions.insert(name.clone(), (*line, *column));
                    if name == "main" {
                        self.has_main_function = true;
                    }
                }
                self.begin_scope();
                for FunctionParam {
                    name,
                    line,
                    column,
                    default,
                    ..
                } in params
                {
                    self.declare(name, Some(*line), Some(*column));
                    if let Some(def_expr) = default.as_deref() {
                        self.visit(def_expr, capture);
                    }
                    // Mark parameters immediately as used? Keep as unused to surface warnings if not referenced.
                }
                let mut saw_direct_return = false;
                let mut any_return = false;
                let mut return_types: Vec<ValueType> = Vec::new();
                for it in body {
                    if saw_direct_return {
                        // unreachable code warning
                        if capture {
                            // approximate span: use start location if available via pattern matching
                            let (l, c) = match it {
                                ASTNode::VariableDecl { line, column, .. } => (*line, *column),
                                ASTNode::Assignment { line, column, .. } => (*line, *column),
                                ASTNode::Function { line, column, .. } => (*line, *column),
                                ASTNode::IdentifierSpanned { line, column, .. } => (*line, *column),
                                _ => (0, 0),
                            };
                            self.diags.push(SemanticDiagnostic {
                                message: "Unreachable code after return".into(),
                                line: l,
                                column: c,
                                len: 1,
                                severity: Severity::Warning,
                            });
                        }
                        // still traverse in case of further symbol usage (optionally skip)
                        self.visit(it, capture);
                        continue;
                    }
                    if matches!(it, ASTNode::Return(_)) {
                        saw_direct_return = true;
                    }
                    if self.extract_return_types(it, &mut return_types) {
                        any_return = true;
                    }
                    self.visit(it, capture);
                }
                self.end_scope();
                if any_return {
                    let guarantees_return = body
                        .last()
                        .map(|stmt| self.statement_guarantees_return(stmt))
                        .unwrap_or(false);
                    if !guarantees_return {
                        if capture {
                            self.diags.push(SemanticDiagnostic {
                                message: format!(
                                    "Not all code paths return a value in function '{name}'"
                                ),
                                line: *line,
                                column: *column,
                                len: name.len().max(1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                    // Return type consistency (ignore Unknown)
                    let mut distinct: Vec<ValueType> = return_types
                        .iter()
                        .copied()
                        .filter(|t| *t != ValueType::Unknown)
                        .collect();
                    distinct.sort_by(|a, b| (*a as u8).cmp(&(*b as u8)));
                    distinct.dedup();
                    if distinct.len() > 1 {
                        if capture {
                            self.diags.push(SemanticDiagnostic {
                                message: format!("Inconsistent return types in function '{name}'"),
                                line: *line,
                                column: *column,
                                len: name.len().max(1),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
            ASTNode::ClassDecl { methods, .. } => {
                self.begin_scope();
                for method in methods {
                    self.visit(method, capture);
                }
                self.end_scope();
            }
            ASTNode::StructDecl { fields, .. } => {
                for field in fields {
                    if let Some(default) = &field.default {
                        self.visit(default, capture);
                    }
                }
            }
            ASTNode::TraitDecl { name, methods, .. } => {
                // Register trait and collect required method signatures
                // Note: Don't visit the trait methods as regular functions since they're just signatures
                let mut trait_methods = Vec::new();
                for method in methods {
                    if let ASTNode::Function { name: method_name, line: method_line, column: method_column, .. } = method {
                        trait_methods.push(TraitMethodSignature {
                            name: method_name.clone(),
                            line: *method_line,
                            column: *method_column,
                        });
                        // Don't visit trait methods - they're just interface definitions
                    }
                }
                self.traits.insert(name.clone(), trait_methods);
            }
            ASTNode::ImplBlock { trait_name, type_name, methods, .. } => {
                // If implementing a trait, verify all required methods are present
                if let Some(trait_name) = trait_name {
                    // Clone required methods to avoid borrow checker issues
                    let required_methods = self.traits.get(trait_name).cloned();
                    
                    if let Some(required_methods) = required_methods {
                        // Collect implemented method names
                        let mut impl_methods: HashSet<String> = HashSet::new();
                        for method in methods {
                            if let ASTNode::Function { name, .. } = method {
                                impl_methods.insert(name.clone());
                            }
                        }
                        
                        // Check that all required methods are implemented
                        for required in &required_methods {
                            if !impl_methods.contains(&required.name) {
                                self.push_error_at(
                                    format!(
                                        "Type '{}' does not implement required method '{}' for trait '{}'",
                                        type_name, required.name, trait_name
                                    ),
                                    None,
                                    None,
                                    None,
                                    capture,
                                );
                            }
                        }
                    } else {
                        // Trait not found - this is an error
                        self.push_error_at(
                            format!("Trait '{}' not found for type '{}'", trait_name, type_name),
                            None,
                            None,
                            None,
                            capture,
                        );
                    }
                }
                
                // Visit all methods in the impl block
                for method in methods {
                    self.visit(method, capture);
                }
            }
            ASTNode::FunctionExpr {
                name,
                line,
                column,
                params,
                body,
            } => {
                self.begin_scope();
                if let Some(fname) = name {
                    self.declare(fname, Some(*line), Some(*column));
                }
                for FunctionParam {
                    name,
                    line,
                    column,
                    default,
                    ..
                } in params
                {
                    self.declare(name, Some(*line), Some(*column));
                    if let Some(def_expr) = default.as_deref() {
                        self.visit(def_expr, capture);
                    }
                }
                for stmt in body {
                    self.visit(stmt, capture);
                }
                self.end_scope();
            }
            ASTNode::VariableDecl {
                name,
                value,
                line,
                column,
                type_annotation,
            } => {
                self.visit(value, capture);
                self.declare(name, Some(*line), Some(*column));
                let expr_ty = self.expr_type(value);
                if let Some(annotation) = type_annotation {
                    if let Some(annotation_ty) = ValueType::from_annotation(annotation) {
                        if expr_ty != ValueType::Unknown && expr_ty != annotation_ty {
                            let msg = format!(
                                "Variable '{name}' annotated as {} but assigned {}",
                                annotation_ty.as_str(),
                                expr_ty.as_str()
                            );
                            self.push_error_at(
                                msg,
                                Some(*line),
                                Some(*column),
                                Some(name.len().max(1)),
                                capture,
                            );
                        }
                        self.set_var_type(name, annotation_ty);
                    } else {
                        self.set_var_type(name, expr_ty);
                    }
                } else {
                    self.set_var_type(name, expr_ty);
                }
            }
            ASTNode::QuantumVariableDecl {
                name,
                value,
                line,
                column,
                ..
            } => {
                self.visit(value, capture);
                self.declare(name, Some(*line), Some(*column));
                self.set_var_type(name, ValueType::Qubit);
            }
            ASTNode::Assignment {
                name,
                value,
                line,
                column,
            } => {
                if !self.is_declared(name) {
                    let msg = format!("Assignment to undeclared variable '{}'", name);
                    self.push_error_at(
                        msg,
                        Some(*line),
                        Some(*column),
                        Some(name.len().max(1)),
                        capture,
                    );
                }
                // write counts as a use
                self.visit(value, capture);
                self.mark_used(name);
            }
            ASTNode::Return(expr) | ASTNode::Log(expr) => {
                self.visit(expr, capture);
            }
            ASTNode::While { condition, body } => {
                self.visit(condition, capture);
                self.loop_depth += 1;
                self.visit(body, capture);
                self.loop_depth -= 1;
            }
            ASTNode::Break => {
                if self.loop_depth == 0 {
                    self.push_error_at(
                        "'break' used outside of loop".to_string(),
                        None,
                        None,
                        None,
                        capture,
                    );
                }
            }
            ASTNode::Continue => {
                if self.loop_depth == 0 {
                    self.push_error_at(
                        "'continue' used outside of loop".to_string(),
                        None,
                        None,
                        None,
                        capture,
                    );
                }
            }
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit(condition, capture);
                self.visit(then_branch, capture);
                if let Some(e) = else_branch {
                    self.visit(e, capture);
                }
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                if let Some(i) = init {
                    self.visit(i, capture);
                }
                if let Some(c) = condition {
                    self.visit(c, capture);
                }
                if let Some(inc) = increment {
                    self.visit(inc, capture);
                }
                self.loop_depth += 1;
                self.visit(body, capture);
                self.loop_depth -= 1;
                self.end_scope();
            }
            ASTNode::ForIn {
                binding,
                iterable,
                body,
            } => {
                self.visit(iterable, capture);
                self.begin_scope();
                self.declare(&binding.name, Some(binding.line), Some(binding.column));
                self.loop_depth += 1;
                self.visit(body, capture);
                self.loop_depth -= 1;
                self.end_scope();
            }
            ASTNode::MatchExpr { value, arms, .. } => {
                self.visit(value, capture);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.visit(guard, capture);
                    }
                    self.visit(&arm.body, capture);
                }
            }
            ASTNode::ProbabilityBranch {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.visit(condition, capture);
                let cond_ty = self.expr_type(condition);
                self.ensure_boolean_condition("Probability branch", cond_ty, condition, capture);
                self.visit(then_branch, capture);
                if let Some(else_node) = else_branch {
                    self.visit(else_node, capture);
                }
            }
            ASTNode::QuantumLoop {
                condition, body, ..
            } => {
                self.visit(condition, capture);
                let cond_ty = self.expr_type(condition);
                self.ensure_boolean_condition("Quantum loop", cond_ty, condition, capture);
                self.loop_depth += 1;
                self.visit(body, capture);
                self.loop_depth -= 1;
            }
            ASTNode::BinaryExpr { op, left, right } => {
                self.visit(left, capture);
                self.visit(right, capture);
                self.check_binary(op, left, right, capture);
            }
            ASTNode::ArrayLiteral(elements) => {
                for element in elements {
                    self.visit(element, capture);
                }
            }
            ASTNode::ObjectLiteral(fields) => {
                for (_, value) in fields {
                    self.visit(value, capture);
                }
            }
            ASTNode::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.visit(value, capture);
                }
            }
            ASTNode::IndexExpr { array, index } => {
                self.visit(array, capture);
                self.visit(index, capture);
            }
            ASTNode::FieldAccess { object, .. } => {
                self.visit(object, capture);
            }
            ASTNode::UnaryExpr { expr, .. } => self.visit(expr, capture),
            ASTNode::Call { callee, args } => {
                if let ASTNode::Identifier(n) = &**callee {
                    self.used_functions.insert(n.clone());
                }
                if let ASTNode::IdentifierSpanned { name, .. } = &**callee {
                    self.used_functions.insert(name.clone());
                }
                self.visit(callee, capture);
                for a in args {
                    self.visit(a, capture);
                }
            }

            // literals / identifiers / quantum/glyph / error
            ASTNode::Identifier(name) => {
                if !self.is_declared(name) && !self.functions.contains_key(name) {
                    let msg = format!("Use of undeclared identifier '{name}'");
                    self.push_error_at(msg, None, None, Some(name.len().max(1)), capture);
                }
                self.mark_used(name);
            }
            ASTNode::IdentifierSpanned {
                name, line, column, ..
            } => {
                if !self.is_declared(name) && !self.functions.contains_key(name) {
                    let msg = format!("Use of undeclared identifier '{name}'");
                    self.push_error_at(
                        msg,
                        Some(*line),
                        Some(*column),
                        Some(name.len().max(1)),
                        capture,
                    );
                }
                self.mark_used(name);
            }
            ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_)
            | ASTNode::HieroglyphicOp { .. }
            | ASTNode::Error(_) => {}
            ASTNode::QuantumOp { op, qubits } => {
                for qubit in qubits {
                    self.visit(qubit, capture);
                }
                // Arity validation
                let qlen = qubits.len();
                let (min, kind_name) = match op {
                    crate::core::token::TokenKind::Superpose => (1, "superpose"),
                    crate::core::token::TokenKind::Entangle => (2, "entangle"),
                    crate::core::token::TokenKind::Measure => (1, "measure"),
                    crate::core::token::TokenKind::Dod => (1, "dod"),
                    _ => (0, "unknown"),
                };
                if qlen < min {
                    let msg = format!(
                        "Quantum op '{kind_name}' expects >= {min} qubit(s) but got {qlen}"
                    );
                    self.push_error_at(msg, None, None, None, capture);
                }
                for qubit in qubits {
                    let ty = self.expr_type(qubit);
                    if ty != ValueType::Qubit {
                        let (line, column, len) = Self::node_span(qubit);
                        let msg = format!(
                            "Quantum operation '{kind_name}' requires qubit arguments. Declare with ⟨q⟩ ← |0⟩."
                        );
                        self.push_error_at(msg, line, column, len, capture);
                    }
                }
            }
            // Temporary wildcard for quantum AST nodes
            _ => {
                // Skip quantum AST nodes for now
            }
        }
    }

    fn mark_used(&mut self, name: &str) {
        for map in self.var_meta.iter_mut().rev() {
            if let Some(v) = map.get_mut(name) {
                v.used = true;
                return;
            }
        }
    }

    fn set_var_type(&mut self, name: &str, ty: ValueType) {
        for map in self.var_meta.iter_mut().rev() {
            if let Some(v) = map.get_mut(name) {
                v.ty = ty;
                return;
            }
        }
    }

    fn get_var_type(&self, name: &str) -> ValueType {
        for map in self.var_meta.iter().rev() {
            if let Some(v) = map.get(name) {
                return v.ty;
            }
        }
        ValueType::Unknown
    }

    fn expr_type(&self, node: &ASTNode) -> ValueType {
        use ValueType::*;
        match node {
            ASTNode::NumberLiteral(_) => Number,
            ASTNode::StringLiteral(_) => String,
            ASTNode::BooleanLiteral(_) => Bool,
            ASTNode::Identifier(n) => self.get_var_type(n),
            ASTNode::IdentifierSpanned { name, .. } => self.get_var_type(name),
            ASTNode::BinaryExpr { op, left, right } => {
                let lt = self.expr_type(left);
                let rt = self.expr_type(right);
                match op {
                    crate::core::token::TokenKind::Plus => {
                        if lt == String || rt == String {
                            String
                        } else if lt == Number && rt == Number {
                            Number
                        } else {
                            Unknown
                        }
                    }
                    crate::core::token::TokenKind::Minus
                    | crate::core::token::TokenKind::Star
                    | crate::core::token::TokenKind::Slash
                    | crate::core::token::TokenKind::Percent => {
                        if lt == Number && rt == Number {
                            Number
                        } else {
                            Unknown
                        }
                    }
                    crate::core::token::TokenKind::DoubleEquals
                    | crate::core::token::TokenKind::NotEquals
                    | crate::core::token::TokenKind::LessThan
                    | crate::core::token::TokenKind::LessEqual
                    | crate::core::token::TokenKind::GreaterThan
                    | crate::core::token::TokenKind::GreaterEqual => Bool,
                    _ => Unknown,
                }
            }
            ASTNode::UnaryExpr { op: _, expr } => self.expr_type(expr),
            ASTNode::Call { .. } => Unknown,
            ASTNode::ArrayLiteral(_) => Array,
            ASTNode::ObjectLiteral(_) => Object,
            ASTNode::StructLiteral { .. } => Object,
            ASTNode::FunctionExpr { .. } => Function,
            ASTNode::QuantumState { .. } | ASTNode::QuantumArray { .. } => Qubit,
            ASTNode::IndexExpr { .. } => Unknown,
            ASTNode::FieldAccess { .. } => Unknown,
            _ => Unknown,
        }
    }

    fn check_binary(
        &mut self,
        op: &crate::core::token::TokenKind,
        left: &ASTNode,
        right: &ASTNode,
        capture: bool,
    ) {
        use crate::core::token::TokenKind as TK;
        use ValueType::*;
        let lt = self.expr_type(left);
        let rt = self.expr_type(right);
        if matches!(
            op,
            TK::Plus | TK::Minus | TK::Star | TK::Slash | TK::Percent
        ) {
            if lt == Qubit || rt == Qubit {
                self.push_type_error(
                    "Qubit values cannot participate in arithmetic expressions",
                    capture,
                );
                return;
            }
        }
        match op {
            TK::Plus => {
                // Be permissive with Unknown types (parameters / unresolved) to avoid false positives.
                if lt == Number && rt == Number {
                    return;
                }
                if lt == String && rt == String {
                    return;
                }
                if lt == Unknown || rt == Unknown {
                    return;
                }
                if (lt == String && rt == Number) || (lt == Number && rt == String) {
                    if capture {
                        self.diags.push(SemanticDiagnostic {
                            message: "Implicit number/string coercion in '+'".into(),
                            line: 0,
                            column: 0,
                            len: 1,
                            severity: Severity::Warning,
                        });
                    }
                } else {
                    self.push_type_error("Invalid operands for '+'", capture);
                }
            }
            TK::Minus | TK::Star | TK::Slash | TK::Percent => {
                if lt != Number || rt != Number {
                    if lt != Unknown && rt != Unknown {
                        self.push_type_error("Arithmetic operands must be numbers", capture);
                    }
                }
            }
            TK::LessThan | TK::LessEqual | TK::GreaterThan | TK::GreaterEqual => {
                if lt != Number || rt != Number {
                    if lt != Unknown && rt != Unknown {
                        self.push_type_error("Comparison operands must be numbers", capture);
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_return_types(&mut self, node: &ASTNode, out: &mut Vec<ValueType>) -> bool {
        match node {
            ASTNode::Return(expr) => {
                out.push(self.expr_type(expr));
                true
            }
            ASTNode::Block(stmts) => {
                let mut any = false;
                for stmt in stmts {
                    if self.extract_return_types(stmt, out) {
                        any = true;
                    }
                }
                any
            }
            ASTNode::If {
                then_branch,
                else_branch,
                ..
            } => {
                let mut any = self.extract_return_types(then_branch, out);
                if let Some(e) = else_branch {
                    if self.extract_return_types(e, out) {
                        any = true;
                    }
                }
                any
            }
            ASTNode::While { body, .. } => self.extract_return_types(body, out),
            ASTNode::For { body, .. } => self.extract_return_types(body, out),
            ASTNode::ForIn { body, .. } => self.extract_return_types(body, out),
            ASTNode::Program(items) => {
                let mut any = false;
                for stmt in items {
                    if self.extract_return_types(stmt, out) {
                        any = true;
                    }
                }
                any
            }
            ASTNode::Function { .. } => false,
            _ => false,
        }
    }

    fn statement_guarantees_return(&mut self, node: &ASTNode) -> bool {
        match node {
            ASTNode::Return(_) => true,
            ASTNode::Block(stmts) => stmts
                .last()
                .map(|s| self.statement_guarantees_return(s))
                .unwrap_or(false),
            ASTNode::If {
                then_branch,
                else_branch,
                ..
            } => {
                if let Some(e) = else_branch {
                    self.statement_guarantees_return(then_branch)
                        && self.statement_guarantees_return(e)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn push_type_error(&mut self, msg: &str, capture: bool) {
        self.errors.push(msg.to_string());
        if capture {
            self.diags.push(SemanticDiagnostic {
                message: msg.to_string(),
                line: 0,
                column: 0,
                len: 1,
                severity: Severity::Error,
            });
        }
    }

    fn push_error_at(
        &mut self,
        msg: String,
        line: Option<usize>,
        column: Option<usize>,
        len: Option<usize>,
        capture: bool,
    ) {
        self.errors.push(msg.clone());
        if capture {
            self.diags.push(SemanticDiagnostic {
                message: msg,
                line: line.unwrap_or(0),
                column: column.unwrap_or(0),
                len: len.unwrap_or(1),
                severity: Severity::Error,
            });
        }
    }

    fn ensure_boolean_condition(
        &mut self,
        context: &str,
        ty: ValueType,
        condition: &ASTNode,
        capture: bool,
    ) {
        if ty == ValueType::Qubit {
            let msg = format!(
                "{context} condition must be a classical boolean. If you have a qubit, call measure(q) first."
            );
            let (line, column, len) = Self::node_span(condition);
            self.push_error_at(msg, line, column, len, capture);
        } else if ty != ValueType::Bool && ty != ValueType::Unknown {
            let msg = format!(
                "{context} condition must be a boolean expression (found {}).",
                ty.as_str()
            );
            let (line, column, len) = Self::node_span(condition);
            self.push_error_at(msg, line, column, len, capture);
        }
    }

    fn node_span(node: &ASTNode) -> (Option<usize>, Option<usize>, Option<usize>) {
        match node {
            ASTNode::IdentifierSpanned {
                name, line, column, ..
            } => (Some(*line), Some(*column), Some(name.len().max(1))),
            ASTNode::VariableDecl {
                name, line, column, ..
            } => (Some(*line), Some(*column), Some(name.len().max(1))),
            ASTNode::Assignment {
                name, line, column, ..
            } => (Some(*line), Some(*column), Some(name.len().max(1))),
            ASTNode::QuantumVariableDecl {
                name, line, column, ..
            } => (Some(*line), Some(*column), Some(name.len().max(1))),
            _ => (None, None, None),
        }
    }

    fn flush_unused_warnings(&mut self) {
        // Trigger end_scope logic for remaining scopes without popping global (avoid double warnings)
        // We'll only process the top-most (innermost) because outer scopes handled during normal popping.
        // Global scope warnings will be generated here.
        if self.var_meta.len() == 1 {
            if let Some(global) = self.var_meta.last() {
                for (name, info) in global.iter() {
                    if !info.used {
                        let msg = format!("Unused variable '{}'", name);
                        self.diags.push(SemanticDiagnostic {
                            message: msg,
                            line: info.line,
                            column: info.column,
                            len: name.len().max(1),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::ASTNode;

    #[test]
    fn redeclare_fails() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0)),
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(2.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_err());
    }

    #[test]
    fn assignment_to_undeclared_fails() {
        let ast = ASTNode::Program(vec![ASTNode::new_assignment(
            "x",
            ASTNode::NumberLiteral(1.0),
        )]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_err());
    }

    #[test]
    fn assignment_to_declared_ok() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0)),
            ASTNode::new_assignment("x", ASTNode::NumberLiteral(2.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_ok());
    }

    #[test]
    fn unreachable_after_return_warns() {
        let ast = ASTNode::Program(vec![ASTNode::new_function(
            "f",
            vec![],
            vec![
                ASTNode::new_return(ASTNode::NumberLiteral(1.0)),
                ASTNode::new_variable_decl("z", ASTNode::NumberLiteral(2.0)),
            ],
        )]);
        let mut a = SemanticAnalyzer::new();
        let diags = a.analyze_with_spans(&ast);
        assert!(diags
            .iter()
            .any(|d| d.message.contains("Unreachable code after return")));
    }

    #[test]
    fn unused_variable_warning() {
        let ast = ASTNode::Program(vec![ASTNode::new_variable_decl(
            "unused",
            ASTNode::NumberLiteral(0.0),
        )]);
        let mut a = SemanticAnalyzer::new();
        let diags = a.analyze_with_spans(&ast);
        assert!(diags
            .iter()
            .any(|d| d.message.contains("Unused variable 'unused'")));
    }

    #[test]
    fn semantic_diagnostic_fields_are_readable() {
        let diag = SemanticDiagnostic {
            message: "example".to_string(),
            line: 3,
            column: 7,
            len: 2,
            severity: Severity::Warning,
        };

        assert_eq!(diag.message, "example");
        assert_eq!(diag.line, 3);
        assert_eq!(diag.column, 7);
        assert_eq!(diag.len, 2);
        assert!(matches!(diag.severity, Severity::Warning));
    }
}
