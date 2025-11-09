//! Module System and Multi-file Support for Aeonmi
//!
//! This module provides:
//! - Module resolution and dependency tracking
//! - Cross-file type checking and symbol resolution
//! - Import/export validation
//! - Circular dependency detection

use crate::core::ast::ASTNode;
use crate::core::enhanced_error::*;
use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Represents a compiled module with its dependencies
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub ast: ASTNode,
    pub exports: HashSet<String>,
    pub imports: HashMap<String, ModuleImport>,
    pub dependencies: HashSet<String>,
}

/// Represents an import statement in a module
#[derive(Debug, Clone)]
pub struct ModuleImport {
    pub module_path: Vec<String>,
    pub items: Option<Vec<String>>, // None means import everything
    pub alias: Option<String>,
}

/// Module resolver handles multi-file compilation
#[derive(Debug)]
pub struct ModuleResolver {
    modules: HashMap<String, Module>,
    module_paths: HashMap<String, PathBuf>,
    root_path: PathBuf,
    search_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            modules: HashMap::new(),
            module_paths: HashMap::new(),
            root_path: root_path.clone(),
            search_paths: vec![root_path],
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Resolve and compile a module from a file path
    pub fn resolve_module(&mut self, module_name: &str) -> CompilerResult<&Module> {
        if self.modules.contains_key(module_name) {
            return Ok(self.modules.get(module_name).unwrap());
        }

        let module_path = self.find_module_file(module_name)?;
        let source = fs::read_to_string(&module_path).map_err(|e| {
            CompilerError::new(
                format!("Failed to read module '{}': {}", module_name, e),
                ErrorKind::ModuleNotFound,
                Span::single(Position::unknown()),
            )
        })?;

        // Parse the module
        let mut lexer = Lexer::new(&source, false); // Set AI access to false for module parsing
        let tokens = lexer.tokenize().map_err(|e| {
            CompilerError::new(
                format!("Lexer error in module '{}': {:?}", module_name, e),
                ErrorKind::InvalidSyntax,
                Span::single(Position::unknown()),
            )
        })?;

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| {
            CompilerError::new(
                format!("Parser error in module '{}': {:?}", module_name, e),
                ErrorKind::InvalidSyntax,
                Span::single(Position::unknown()),
            )
        })?;

        // Extract imports and exports
        let (imports, exports, dependencies) = self.analyze_module(&ast)?;

        let module = Module {
            name: module_name.to_string(),
            path: module_path.clone(),
            ast,
            exports,
            imports,
            dependencies,
        };

        self.module_paths
            .insert(module_name.to_string(), module_path);
        self.modules.insert(module_name.to_string(), module);

        Ok(self.modules.get(module_name).unwrap())
    }

    /// Find a module file in the search paths
    fn find_module_file(&self, module_name: &str) -> CompilerResult<PathBuf> {
        let possible_extensions = ["aeon", "aeonmi", "qube"];

        for search_path in &self.search_paths {
            for ext in &possible_extensions {
                let candidate = search_path.join(format!("{}.{}", module_name, ext));
                if candidate.exists() {
                    return Ok(candidate);
                }

                // Also check for index files in directories
                let dir_candidate = search_path.join(module_name).join(format!("index.{}", ext));
                if dir_candidate.exists() {
                    return Ok(dir_candidate);
                }
            }
        }

        Err(Box::new(CompilerError::new(
            format!("Module '{}' not found in search paths", module_name),
            ErrorKind::ModuleNotFound,
            Span::single(Position::unknown()),
        )))
    }

    /// Analyze a module AST to extract imports, exports, and dependencies
    fn analyze_module(
        &self,
        ast: &ASTNode,
    ) -> CompilerResult<(
        HashMap<String, ModuleImport>,
        HashSet<String>,
        HashSet<String>,
    )> {
        let mut imports = HashMap::new();
        let mut exports = HashSet::new();
        let mut dependencies = HashSet::new();

        self.analyze_node(ast, &mut imports, &mut exports, &mut dependencies)?;

        Ok((imports, exports, dependencies))
    }

    fn analyze_node(
        &self,
        node: &ASTNode,
        imports: &mut HashMap<String, ModuleImport>,
        exports: &mut HashSet<String>,
        dependencies: &mut HashSet<String>,
    ) -> CompilerResult<()> {
        match node {
            ASTNode::Program(statements) => {
                for stmt in statements {
                    self.analyze_node(stmt, imports, exports, dependencies)?;
                }
            }

            ASTNode::Import { path, items } => {
                let module_name = path.join("::");
                dependencies.insert(module_name.clone());

                let import = ModuleImport {
                    module_path: path.clone(),
                    items: items.clone(),
                    alias: None,
                };

                let key = if let Some(items) = items {
                    items.join(",")
                } else {
                    module_name.clone()
                };

                imports.insert(key, import);
            }

            ASTNode::Export { items, path: _ } => {
                for item in items {
                    exports.insert(item.clone());
                }
            }

            ASTNode::Function { name, .. } => {
                // Functions are exportable by default
                exports.insert(name.clone());
            }

            ASTNode::ClassDecl { name, .. } => {
                exports.insert(name.clone());
            }

            ASTNode::StructDecl { name, .. } => {
                exports.insert(name.clone());
            }

            ASTNode::TraitDecl { name, .. } => {
                exports.insert(name.clone());
            }

            ASTNode::EnumDecl { name, .. } => {
                exports.insert(name.clone());
            }

            // Recursively analyze other nodes
            ASTNode::Module { body, .. } => {
                for stmt in body {
                    self.analyze_node(stmt, imports, exports, dependencies)?;
                }
            }

            _ => {
                // For other node types, continue recursion if they have children
                // This is a simplified approach - in a full implementation,
                // we'd need to handle all node types that can contain other nodes
            }
        }

        Ok(())
    }

    /// Resolve all dependencies for a module recursively
    pub fn resolve_dependencies(&mut self, module_name: &str) -> CompilerResult<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut resolving = HashSet::new();

        self.resolve_dependencies_recursive(
            module_name,
            &mut resolved,
            &mut visited,
            &mut resolving,
        )?;

        Ok(resolved)
    }

    fn resolve_dependencies_recursive(
        &mut self,
        module_name: &str,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
        resolving: &mut HashSet<String>,
    ) -> CompilerResult<()> {
        if visited.contains(module_name) {
            return Ok(());
        }

        if resolving.contains(module_name) {
            return Err(Box::new(CompilerError::new(
                format!(
                    "Circular dependency detected involving module '{}'",
                    module_name
                ),
                ErrorKind::CircularDependency,
                Span::single(Position::unknown()),
            )));
        }

        resolving.insert(module_name.to_string());

        // First resolve the module itself
        let module = self.resolve_module(module_name)?.clone();

        // Then resolve all its dependencies
        for dep in &module.dependencies {
            self.resolve_dependencies_recursive(dep, resolved, visited, resolving)?;
        }

        resolving.remove(module_name);
        visited.insert(module_name.to_string());
        resolved.push(module_name.to_string());

        Ok(())
    }

    /// Get all modules in dependency order
    pub fn get_compilation_order(&mut self, entry_module: &str) -> CompilerResult<Vec<String>> {
        self.resolve_dependencies(entry_module)
    }

    /// Check if an import is valid (the target module exports the requested items)
    pub fn validate_import(
        &self,
        _importing_module: &str,
        import: &ModuleImport,
    ) -> CompilerResult<()> {
        let target_module_name = import.module_path.join("::");

        let target_module = self.modules.get(&target_module_name).ok_or_else(|| {
            CompilerError::new(
                format!("Module '{}' not found", target_module_name),
                ErrorKind::ModuleNotFound,
                Span::single(Position::unknown()),
            )
        })?;

        if let Some(items) = &import.items {
            for item in items {
                if !target_module.exports.contains(item) {
                    return Err(Box::new(CompilerError::new(
                        format!("Module '{}' does not export '{}'", target_module_name, item),
                        ErrorKind::ImportError,
                        Span::single(Position::unknown()),
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate all imports in all loaded modules
    pub fn validate_all_imports(&self) -> CompilerResult<()> {
        for (module_name, module) in &self.modules {
            for import in module.imports.values() {
                self.validate_import(module_name, import)?;
            }
        }
        Ok(())
    }

    /// Get the module containing a specific symbol
    pub fn find_symbol_module(&self, symbol: &str) -> Option<&Module> {
        for module in self.modules.values() {
            if module.exports.contains(symbol) {
                return Some(module);
            }
        }
        None
    }
}

/// Project-wide compilation context
#[derive(Debug)]
pub struct CompilationContext {
    pub resolver: ModuleResolver,
    pub entry_module: String,
    pub compilation_order: Vec<String>,
}

impl CompilationContext {
    pub fn new(root_path: PathBuf, entry_module: String) -> Self {
        Self {
            resolver: ModuleResolver::new(root_path),
            entry_module,
            compilation_order: Vec::new(),
        }
    }

    pub fn compile_project(&mut self) -> CompilerResult<()> {
        // Resolve all dependencies
        self.compilation_order = self.resolver.get_compilation_order(&self.entry_module)?;

        // Validate all imports
        self.resolver.validate_all_imports()?;

        Ok(())
    }

    pub fn get_modules(&self) -> &HashMap<String, Module> {
        &self.resolver.modules
    }
}
