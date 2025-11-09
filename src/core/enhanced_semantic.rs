//! Enhanced Semantic Analysis System for Aeonmi
//!
//! This module provides comprehensive semantic analysis including:
//! - Symbol table management with proper scoping
//! - Type checking and inference
//! - Flow analysis and control flow validation
//! - Multi-file dependency tracking
//! - Detailed error reporting with source location info

use crate::core::ast::{ASTNode, FunctionParam, MatchArm, StructField};
use crate::core::enhanced_error::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

/// Represents different types in the Aeonmi type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Primitive types
    Number,
    String,
    Boolean,
    Void,

    // Composite types
    Array(Box<Type>),
    Object(HashMap<String, Type>),

    // Function types
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    // Quantum types
    Qubit,
    QuantumArray(Box<Type>),
    QuantumState,

    // User-defined types
    Struct(String),
    Class(String),
    Trait(String),

    // Special types
    Unknown,
    Any,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "boolean"),
            Type::Void => write!(f, "void"),
            Type::Array(inner) => write!(f, "array<{}>", inner),
            Type::Object(_) => write!(f, "object"),
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "function(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Qubit => write!(f, "qubit"),
            Type::QuantumArray(inner) => write!(f, "quantum_array<{}>", inner),
            Type::QuantumState => write!(f, "quantum_state"),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Class(name) => write!(f, "class {}", name),
            Type::Trait(name) => write!(f, "trait {}", name),
            Type::Unknown => write!(f, "unknown"),
            Type::Any => write!(f, "any"),
        }
    }
}

/// Symbol information stored in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub line: usize,
    pub column: usize,
    pub is_mutable: bool,
    pub is_used: bool,
    pub scope_depth: usize,
}

/// Scope information for proper symbol resolution
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub scope_type: ScopeType,
    pub parent: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Global,
    Function(String),
    Block,
    Class(String),
    Struct(String),
    Trait(String),
    ImplBlock {
        trait_name: Option<String>,
        type_name: String,
    },
}

/// Enhanced diagnostic with severity levels and categories
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub error: CompilerError,
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticCategory {
    UndefinedVariable,
    TypeMismatch,
    RedefinedSymbol,
    UnusedVariable,
    UnusedFunction,
    InvalidOperation,
    MissingReturn,
    UnreachableCode,
    InvalidAccess,
}

/// Enhanced semantic analyzer with comprehensive checking
pub struct EnhancedSemanticAnalyzer {
    scopes: Vec<Scope>,
    current_scope: usize,
    diagnostics: Vec<Diagnostic>,
    current_file: String,

    // Type information
    user_types: HashMap<String, Type>,
    function_signatures: HashMap<String, Type>,

    // Control flow analysis
    current_function: Option<String>,
    has_return: bool,

    // Multi-file support
    imported_modules: HashMap<String, HashSet<String>>,
    exported_symbols: HashSet<String>,
}

impl EnhancedSemanticAnalyzer {
    pub fn new() -> Self {
        let global_scope = Scope {
            symbols: HashMap::new(),
            scope_type: ScopeType::Global,
            parent: None,
        };

        let mut analyzer = Self {
            scopes: vec![global_scope],
            current_scope: 0,
            diagnostics: Vec::new(),
            current_file: String::new(),
            user_types: HashMap::new(),
            function_signatures: HashMap::new(),
            current_function: None,
            has_return: false,
            imported_modules: HashMap::new(),
            exported_symbols: HashSet::new(),
        };

        // Add built-in types and functions
        analyzer.add_builtin_symbols();
        analyzer
    }

    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> Result<(), Vec<Diagnostic>> {
        self.diagnostics.clear();
        self.visit_node(ast);

        // Post-analysis passes
        self.check_unused_symbols();
        self.check_undefined_references();

        if self.has_errors() {
            Err(self.diagnostics.clone())
        } else {
            Ok(())
        }
    }

    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    fn add_builtin_symbols(&mut self) {
        let builtins = vec![
            (
                "log",
                Type::Function {
                    params: vec![Type::Any],
                    return_type: Box::new(Type::Void),
                },
            ),
            (
                "print",
                Type::Function {
                    params: vec![Type::Any],
                    return_type: Box::new(Type::Void),
                },
            ),
            (
                "len",
                Type::Function {
                    params: vec![Type::Any],
                    return_type: Box::new(Type::Number),
                },
            ),
            (
                "rand",
                Type::Function {
                    params: vec![],
                    return_type: Box::new(Type::Number),
                },
            ),
            (
                "time_ms",
                Type::Function {
                    params: vec![],
                    return_type: Box::new(Type::Number),
                },
            ),
            // Quantum built-ins
            (
                "superpose",
                Type::Function {
                    params: vec![Type::Qubit],
                    return_type: Box::new(Type::Void),
                },
            ),
            (
                "entangle",
                Type::Function {
                    params: vec![Type::Qubit, Type::Qubit],
                    return_type: Box::new(Type::Void),
                },
            ),
            (
                "measure",
                Type::Function {
                    params: vec![Type::Qubit],
                    return_type: Box::new(Type::Number),
                },
            ),
        ];

        for (name, func_type) in builtins {
            self.declare_symbol(name, func_type, 0, 0, false);
        }
    }

    fn visit_node(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program(statements) => {
                for stmt in statements {
                    self.visit_node(stmt);
                }
            }

            ASTNode::Function {
                name,
                params,
                body,
                line,
                column,
                ..
            } => {
                self.visit_function_declaration(name, params, body, *line, *column);
            }

            ASTNode::ClassDecl {
                name,
                methods,
                line,
                column,
                ..
            } => {
                self.visit_class_declaration(name, methods, *line, *column);
            }

            ASTNode::StructDecl {
                name,
                fields,
                line,
                column,
            } => {
                self.visit_struct_declaration(name, fields, *line, *column);
            }

            ASTNode::TraitDecl {
                name,
                methods,
                line,
                column,
            } => {
                self.visit_trait_declaration(name, methods, *line, *column);
            }

            ASTNode::ImplBlock {
                trait_name,
                type_name,
                methods,
                ..
            } => {
                self.visit_impl_block(trait_name, type_name, methods);
            }

            ASTNode::VariableDecl {
                name,
                value,
                line,
                column,
                ..
            } => {
                self.visit_variable_declaration(name, value, *line, *column);
            }

            ASTNode::Assignment {
                name,
                value,
                line,
                column,
            } => {
                self.visit_assignment(name, value, *line, *column);
            }

            ASTNode::Block(statements) => {
                self.enter_scope(ScopeType::Block);
                for stmt in statements {
                    self.visit_node(stmt);
                }
                self.exit_scope();
            }

            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_expression_type(condition, &Type::Boolean);
                self.visit_node(then_branch);
                if let Some(else_stmt) = else_branch {
                    self.visit_node(else_stmt);
                }
            }

            ASTNode::While { condition, body } => {
                self.check_expression_type(condition, &Type::Boolean);
                self.visit_node(body);
            }

            ASTNode::MatchExpr { value, arms, .. } => {
                let value_type = self.infer_expression_type(value);
                for arm in arms {
                    self.check_match_arm(arm, &value_type);
                }
            }

            ASTNode::Return(expr) => {
                if let Some(func_name) = &self.current_function {
                    if let Some(expected_type) = self.get_function_return_type(func_name) {
                        self.check_expression_type(expr, &expected_type);
                    }
                    self.has_return = true;
                } else {
                    self.add_diagnostic(
                        "Return statement outside function".to_string(),
                        DiagnosticSeverity::Error,
                        DiagnosticCategory::InvalidOperation,
                        0,
                        0,
                        6,
                        None,
                    );
                }
            }

            // Handle other node types...
            _ => {
                // For now, recursively visit child nodes
                self.visit_children(node);
            }
        }
    }

    fn visit_function_declaration(
        &mut self,
        name: &str,
        params: &[FunctionParam],
        body: &[ASTNode],
        line: usize,
        column: usize,
    ) {
        // Check for redefinition
        if self.symbol_exists_in_current_scope(name) {
            self.add_diagnostic(
                format!("Function '{}' is already defined", name),
                DiagnosticSeverity::Error,
                DiagnosticCategory::RedefinedSymbol,
                line,
                column,
                name.len(),
                Some(format!(
                    "Consider using a different name or removing the duplicate definition"
                )),
            );
            return;
        }

        // Create function type
        let param_types: Vec<Type> = params
            .iter()
            .map(|_p| Type::Unknown) // For now, we'll need to infer or annotate types
            .collect();

        let func_type = Type::Function {
            params: param_types,
            return_type: Box::new(Type::Unknown), // Will be inferred from body
        };

        self.declare_symbol(name, func_type.clone(), line, column, false);
        self.function_signatures.insert(name.to_string(), func_type);

        // Enter function scope
        self.enter_scope(ScopeType::Function(name.to_string()));
        self.current_function = Some(name.to_string());
        self.has_return = false;

        // Declare parameters
        for param in params {
            self.declare_symbol(&param.name, Type::Unknown, param.line, param.column, true);
        }

        // Visit function body
        for stmt in body {
            self.visit_node(stmt);
        }

        // Check if function should return a value
        if !self.has_return && self.function_should_return(body) {
            self.add_diagnostic(
                format!("Function '{}' should return a value", name),
                DiagnosticSeverity::Warning,
                DiagnosticCategory::MissingReturn,
                line,
                column,
                name.len(),
                Some("Add a return statement or change function to return void".to_string()),
            );
        }

        self.current_function = None;
        self.exit_scope();
    }

    fn visit_class_declaration(
        &mut self,
        name: &str,
        methods: &[ASTNode],
        line: usize,
        column: usize,
    ) {
        let class_type = Type::Class(name.to_string());
        self.declare_symbol(name, class_type.clone(), line, column, false);
        self.user_types.insert(name.to_string(), class_type);

        self.enter_scope(ScopeType::Class(name.to_string()));
        for method in methods {
            self.visit_node(method);
        }
        self.exit_scope();
    }

    fn visit_struct_declaration(
        &mut self,
        name: &str,
        fields: &[StructField],
        line: usize,
        column: usize,
    ) {
        let struct_type = Type::Struct(name.to_string());
        self.declare_symbol(name, struct_type.clone(), line, column, false);
        self.user_types.insert(name.to_string(), struct_type);

        // Create object type from fields
        let mut field_types = HashMap::new();
        for field in fields {
            field_types.insert(field.name.clone(), Type::Unknown); // Infer from default values
        }

        let object_type = Type::Object(field_types);
        self.user_types
            .insert(format!("{}_instance", name), object_type);
    }

    fn visit_trait_declaration(
        &mut self,
        name: &str,
        methods: &[ASTNode],
        line: usize,
        column: usize,
    ) {
        let trait_type = Type::Trait(name.to_string());
        self.declare_symbol(name, trait_type.clone(), line, column, false);
        self.user_types.insert(name.to_string(), trait_type);

        self.enter_scope(ScopeType::Trait(name.to_string()));
        for method in methods {
            self.visit_node(method);
        }
        self.exit_scope();
    }

    fn visit_impl_block(
        &mut self,
        trait_name: &Option<String>,
        type_name: &str,
        methods: &[ASTNode],
    ) {
        self.enter_scope(ScopeType::ImplBlock {
            trait_name: trait_name.clone(),
            type_name: type_name.to_string(),
        });

        for method in methods {
            self.visit_node(method);
        }

        self.exit_scope();
    }

    fn visit_variable_declaration(
        &mut self,
        name: &str,
        value: &ASTNode,
        line: usize,
        column: usize,
    ) {
        // Check for redefinition in current scope
        if self.symbol_exists_in_current_scope(name) {
            self.add_diagnostic(
                format!("Variable '{}' is already defined in this scope", name),
                DiagnosticSeverity::Error,
                DiagnosticCategory::RedefinedSymbol,
                line,
                column,
                name.len(),
                Some("Consider using a different name".to_string()),
            );
            return;
        }

        // Infer type from initial value
        let var_type = self.infer_expression_type(value);
        self.declare_symbol(name, var_type, line, column, true);

        // Visit the initialization expression
        self.visit_node(value);
    }

    fn visit_assignment(&mut self, name: &str, value: &ASTNode, line: usize, column: usize) {
        // Check if variable is defined and get its info
        let symbol_info = self
            .lookup_symbol(name)
            .map(|s| (s.is_mutable, s.symbol_type.clone()));

        if let Some((is_mutable, expected_type)) = symbol_info {
            if !is_mutable {
                self.add_diagnostic(
                    format!("Cannot assign to immutable variable '{}'", name),
                    DiagnosticSeverity::Error,
                    DiagnosticCategory::InvalidOperation,
                    line,
                    column,
                    name.len(),
                    Some("Declare the variable as mutable".to_string()),
                );
            }

            // Check type compatibility
            self.check_expression_type(value, &expected_type);

            // Mark as used
            self.mark_symbol_used(name);
        } else {
            self.add_diagnostic(
                format!("Undefined variable '{}'", name),
                DiagnosticSeverity::Error,
                DiagnosticCategory::UndefinedVariable,
                line,
                column,
                name.len(),
                Some(format!("Did you mean to declare it with 'let {}'?", name)),
            );
        }

        self.visit_node(value);
    }

    // Helper methods for scope management
    fn enter_scope(&mut self, scope_type: ScopeType) {
        let parent = Some(self.current_scope);
        let new_scope = Scope {
            symbols: HashMap::new(),
            scope_type,
            parent,
        };

        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.last() {
            if let Some(parent) = scope.parent {
                self.current_scope = parent;
            }
        }

        // Check for unused variables in this scope
        if let Some(scope) = self.scopes.pop() {
            for (name, symbol) in scope.symbols {
                if !symbol.is_used && !name.starts_with('_') {
                    self.add_diagnostic(
                        format!("Unused variable '{}'", name),
                        DiagnosticSeverity::Warning,
                        DiagnosticCategory::UnusedVariable,
                        symbol.line,
                        symbol.column,
                        name.len(),
                        Some(format!(
                            "Prefix with underscore if intentionally unused: '_{}'",
                            name
                        )),
                    );
                }
            }
        }
    }

    fn declare_symbol(
        &mut self,
        name: &str,
        symbol_type: Type,
        line: usize,
        column: usize,
        is_mutable: bool,
    ) {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type,
            line,
            column,
            is_mutable,
            is_used: false,
            scope_depth: self.current_scope,
        };

        if let Some(scope) = self.scopes.get_mut(self.current_scope) {
            scope.symbols.insert(name.to_string(), symbol);
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current = self.current_scope;
        loop {
            if let Some(scope) = self.scopes.get(current) {
                if let Some(symbol) = scope.symbols.get(name) {
                    return Some(symbol);
                }

                if let Some(parent) = scope.parent {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        None
    }

    fn symbol_exists_in_current_scope(&self, name: &str) -> bool {
        if let Some(scope) = self.scopes.get(self.current_scope) {
            scope.symbols.contains_key(name)
        } else {
            false
        }
    }

    fn mark_symbol_used(&mut self, name: &str) {
        let mut current = self.current_scope;
        loop {
            if let Some(scope) = self.scopes.get_mut(current) {
                if let Some(symbol) = scope.symbols.get_mut(name) {
                    symbol.is_used = true;
                    return;
                }

                if let Some(parent) = scope.parent {
                    current = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    // Type checking methods
    fn infer_expression_type(&self, expr: &ASTNode) -> Type {
        match expr {
            ASTNode::NumberLiteral(_) => Type::Number,
            ASTNode::StringLiteral(_) => Type::String,
            ASTNode::BooleanLiteral(_) => Type::Boolean,
            ASTNode::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    Type::Array(Box::new(Type::Unknown))
                } else {
                    let first_type = self.infer_expression_type(&elements[0]);
                    Type::Array(Box::new(first_type))
                }
            }
            ASTNode::Identifier(name) | ASTNode::IdentifierSpanned { name, .. } => {
                if let Some(symbol) = self.lookup_symbol(name) {
                    symbol.symbol_type.clone()
                } else {
                    Type::Unknown
                }
            }
            ASTNode::BinaryExpr { op, left: _, right: _ } => {
                use crate::core::token::TokenKind;
                match op {
                    TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                        Type::Number
                    }
                    TokenKind::DoubleEquals
                    | TokenKind::NotEquals
                    | TokenKind::LessThan
                    | TokenKind::LessEqual
                    | TokenKind::GreaterThan
                    | TokenKind::GreaterEqual => Type::Boolean,
                    _ => Type::Unknown,
                }
            }
            _ => Type::Unknown,
        }
    }

    fn check_expression_type(&mut self, expr: &ASTNode, expected: &Type) {
        let actual = self.infer_expression_type(expr);
        if !self.types_compatible(&actual, expected) {
            if let Some(location) = self.get_expression_location(expr) {
                self.add_diagnostic(
                    format!("Type mismatch: expected {}, found {}", expected, actual),
                    DiagnosticSeverity::Error,
                    DiagnosticCategory::TypeMismatch,
                    location.line,
                    location.column,
                    location.length,
                    None,
                );
            }
        }
    }

    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Any, _) | (_, Type::Any) => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }

    fn get_expression_location(&self, expr: &ASTNode) -> Option<SourceLocation> {
        match expr {
            ASTNode::IdentifierSpanned {
                line, column, len, ..
            } => Some(SourceLocation {
                file: self.current_file.clone(),
                line: *line,
                column: *column,
                length: *len,
            }),
            // Add more cases as needed
            _ => None,
        }
    }

    fn check_match_arm(&mut self, arm: &MatchArm, value_type: &Type) {
        // Check pattern compatibility
        let pattern_type = self.infer_expression_type(&arm.pattern);
        if !self.types_compatible(&pattern_type, value_type) {
            self.add_diagnostic(
                format!(
                    "Pattern type {} doesn't match value type {}",
                    pattern_type, value_type
                ),
                DiagnosticSeverity::Error,
                DiagnosticCategory::TypeMismatch,
                arm.line,
                arm.column,
                1,
                None,
            );
        }

        // Check guard if present
        if let Some(guard) = &arm.guard {
            self.check_expression_type(guard, &Type::Boolean);
        }

        // Visit arm body
        self.visit_node(&arm.body);
    }

    fn function_should_return(&self, body: &[ASTNode]) -> bool {
        // Simple heuristic: if any statement is a return, function should return
        body.iter().any(|stmt| matches!(stmt, ASTNode::Return(_)))
    }

    fn get_function_return_type(&self, name: &str) -> Option<Type> {
        self.function_signatures
            .get(name)
            .and_then(|sig| match sig {
                Type::Function { return_type, .. } => Some((**return_type).clone()),
                _ => None,
            })
    }

    fn visit_children(&mut self, node: &ASTNode) {
        // Generic visitor for nodes we don't handle specifically
        match node {
            ASTNode::BinaryExpr { left, right, .. } => {
                self.visit_node(left);
                self.visit_node(right);
            }
            ASTNode::UnaryExpr { expr, .. } => {
                self.visit_node(expr);
            }
            ASTNode::Call { callee, args } => {
                self.visit_node(callee);
                for arg in args {
                    self.visit_node(arg);
                }
            }
            ASTNode::ArrayLiteral(elements) => {
                for element in elements {
                    self.visit_node(element);
                }
            }
            ASTNode::ObjectLiteral(fields) => {
                for (_, value) in fields {
                    self.visit_node(value);
                }
            }
            _ => {} // No children to visit
        }
    }

    fn check_unused_symbols(&mut self) {
        // This is handled in exit_scope for now
    }

    fn check_undefined_references(&mut self) {
        // This is handled during visitation for now
    }

    fn add_diagnostic(
        &mut self,
        message: String,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        line: usize,
        column: usize,
        length: usize,
        suggestion: Option<String>,
    ) {
        let position = Position::new(self.current_file.clone(), line, column, length);
        let span = Span::single(position);

        let error_kind = match category {
            DiagnosticCategory::UndefinedVariable => ErrorKind::UndefinedVariable,
            DiagnosticCategory::TypeMismatch => ErrorKind::TypeMismatch,
            DiagnosticCategory::RedefinedSymbol => ErrorKind::RedefinedSymbol,
            DiagnosticCategory::UnusedVariable => ErrorKind::UnreachableCode, // Use as warning
            DiagnosticCategory::UnusedFunction => ErrorKind::UnreachableCode,
            DiagnosticCategory::InvalidOperation => ErrorKind::InvalidOperation,
            DiagnosticCategory::MissingReturn => ErrorKind::MissingReturn,
            DiagnosticCategory::UnreachableCode => ErrorKind::UnreachableCode,
            DiagnosticCategory::InvalidAccess => ErrorKind::InvalidAccess,
        };

        let mut compiler_error = CompilerError::new(message, error_kind, span);
        if let Some(suggestion) = suggestion {
            compiler_error = compiler_error.with_suggestion(suggestion);
        }

        self.diagnostics.push(Diagnostic {
            error: compiler_error,
            severity,
            category,
        });
    }
}

impl Default for EnhancedSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
