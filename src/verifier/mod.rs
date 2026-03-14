//! AEONMI Smart-Contract Verifier — Phase 5 IDEA 1
//!
//! Static analysis + quantum-assisted verification tool for `.ai` smart contracts.
//! Uses the Shard compiler's AST to emit a symbolic constraint graph and classifies
//! functions as "constant" (pure/safe) vs "balanced" (stateful/dangerous) using a
//! Deutsch-Jozsa-inspired analysis.
//!
//! CLI: `aeonmi verify <contract.ai>`

use crate::core::ast::ASTNode;
use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ── Verification Severity ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Critical => write!(f, "CRITICAL"),
            IssueSeverity::Warning  => write!(f, "WARNING"),
            IssueSeverity::Info     => write!(f, "INFO"),
        }
    }
}

// ── Verification Issue ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VerificationIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub message: String,
    pub function: Option<String>,
    pub line: Option<usize>,
}

impl VerificationIssue {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"severity":"{}","category":"{}","message":"{}","function":{},"line":{}}}"#,
            self.severity,
            self.category,
            self.message.replace('"', "\\\""),
            self.function.as_ref().map_or("null".to_string(), |f| format!("\"{}\"", f)),
            self.line.map_or("null".to_string(), |l| l.to_string()),
        )
    }
}

// ── Function Classification ──────────────────────────────────────────────────

/// Deutsch-Jozsa inspired: classify each function as "constant" (pure/safe)
/// or "balanced" (stateful/dangerous with side effects).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionPurity {
    /// Pure function — no external state mutations, safe to call repeatedly
    Constant,
    /// Stateful function — reads/writes external state, potential re-entrancy risk
    Balanced,
}

impl std::fmt::Display for FunctionPurity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionPurity::Constant => write!(f, "constant (pure)"),
            FunctionPurity::Balanced => write!(f, "balanced (stateful)"),
        }
    }
}

// ── Function Analysis ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub name: String,
    pub purity: FunctionPurity,
    pub param_count: usize,
    pub calls_external: Vec<String>,
    pub reads_state: bool,
    pub writes_state: bool,
    pub has_recursion: bool,
    pub line: usize,
}

// ── Verification Report ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub file_name: String,
    pub file_hash: String,
    pub function_count: usize,
    pub functions: Vec<FunctionAnalysis>,
    pub issues: Vec<VerificationIssue>,
    pub pure_count: usize,
    pub stateful_count: usize,
    pub risk_score: f64,
}

impl VerificationReport {
    pub fn to_json(&self) -> String {
        let funcs: Vec<String> = self.functions.iter().map(|fa| {
            format!(
                r#"    {{"name":"{}","purity":"{}","params":{},"calls":[{}],"reads_state":{},"writes_state":{},"has_recursion":{},"line":{}}}"#,
                fa.name,
                fa.purity,
                fa.param_count,
                fa.calls_external.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(","),
                fa.reads_state,
                fa.writes_state,
                fa.has_recursion,
                fa.line,
            )
        }).collect();
        let issues: Vec<String> = self.issues.iter().map(|i| format!("    {}", i.to_json())).collect();
        format!(
            r#"{{
  "file": "{}",
  "file_hash": "{}",
  "function_count": {},
  "pure_functions": {},
  "stateful_functions": {},
  "risk_score": {:.2},
  "functions": [
{}
  ],
  "issues": [
{}
  ]
}}"#,
            self.file_name,
            self.file_hash,
            self.function_count,
            self.pure_count,
            self.stateful_count,
            self.risk_score,
            funcs.join(",\n"),
            issues.join(",\n"),
        )
    }

    pub fn summary(&self) -> String {
        let mut out = String::new();
        out.push_str("╔══════════════════════════════════════════════════╗\n");
        out.push_str("║     AEONMI Smart-Contract Verifier Report       ║\n");
        out.push_str("╠══════════════════════════════════════════════════╣\n");
        out.push_str(&format!("║  File: {:<41} ║\n", truncate(&self.file_name, 41)));
        out.push_str(&format!("║  Hash: {:<41} ║\n", truncate(&self.file_hash, 41)));
        out.push_str(&format!("║  Functions: {:<36} ║\n", self.function_count));
        out.push_str(&format!("║  Pure (constant): {:<30} ║\n", self.pure_count));
        out.push_str(&format!("║  Stateful (balanced): {:<26} ║\n", self.stateful_count));
        out.push_str(&format!("║  Risk Score: {:<35} ║\n", format!("{:.2}/10.0", self.risk_score)));
        out.push_str("╠══════════════════════════════════════════════════╣\n");

        // Function details
        for fa in &self.functions {
            let icon = if fa.purity == FunctionPurity::Constant { "◈" } else { "⚠" };
            out.push_str(&format!("║  {} {}: {:<34} ║\n", icon, fa.name, fa.purity));
        }

        // Issues
        if !self.issues.is_empty() {
            out.push_str("╠══════════════════════════════════════════════════╣\n");
            out.push_str("║  Issues:                                         ║\n");
            for issue in &self.issues {
                let icon = match issue.severity {
                    IssueSeverity::Critical => "🔴",
                    IssueSeverity::Warning  => "🟡",
                    IssueSeverity::Info     => "🔵",
                };
                out.push_str(&format!("║  {} [{}] {}\n", icon, issue.severity, truncate(&issue.message, 35)));
            }
        }

        out.push_str("╚══════════════════════════════════════════════════╝\n");
        out
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let truncated: String = chars[..max - 1].iter().collect();
        format!("{}…", truncated)
    }
}

// ── Side-effect Builtins ─────────────────────────────────────────────────────

/// Built-in functions considered to have side effects (write/mutate state)
const STATE_WRITE_BUILTINS: &[&str] = &[
    "write_file", "append_file", "delete_file",
    "print", "log",
];

/// Built-in functions that read external state
const STATE_READ_BUILTINS: &[&str] = &[
    "read_file", "read_lines", "file_exists",
];

/// External call targets (network, hardware, etc.)
const EXTERNAL_BUILTINS: &[&str] = &[
    "http_listen", "http_get", "http_post",
    "submit_job", "list_devices",
    "ai_chat", "ai_complete",
];

// ── Verifier Engine ──────────────────────────────────────────────────────────

pub struct ContractVerifier {
    /// All discovered function definitions
    functions: HashMap<String, FunctionAnalysis>,
    /// Global variable assignments
    global_writes: HashSet<String>,
    /// Issues discovered during analysis
    issues: Vec<VerificationIssue>,
}

impl ContractVerifier {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            global_writes: HashSet::new(),
            issues: Vec::new(),
        }
    }

    /// Verify a `.ai` source file and produce a report.
    pub fn verify_source(&mut self, source: &str, file_name: &str) -> Result<VerificationReport, String> {
        // Parse source to AST
        let mut lexer = Lexer::from_str(source);
        let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

        // Compute file hash
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let file_hash = format!("{:x}", hasher.finalize());

        // Analyze the AST
        self.analyze_ast(&ast);

        // Run verification checks
        self.check_reentrancy();
        self.check_unchecked_external_calls();
        self.check_infinite_recursion();

        // Compile report
        let functions: Vec<FunctionAnalysis> = self.functions.values().cloned().collect();
        let pure_count = functions.iter().filter(|f| f.purity == FunctionPurity::Constant).count();
        let stateful_count = functions.iter().filter(|f| f.purity == FunctionPurity::Balanced).count();
        let function_count = functions.len();

        // Risk score: 0-10 based on issues and stateful ratio
        let critical_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
        let warning_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
        let stateful_ratio = if function_count > 0 {
            stateful_count as f64 / function_count as f64
        } else {
            0.0
        };
        let risk_score = ((critical_count as f64 * 3.0) + (warning_count as f64 * 1.0) + (stateful_ratio * 2.0))
            .min(10.0);

        Ok(VerificationReport {
            file_name: file_name.to_string(),
            file_hash,
            function_count,
            functions,
            issues: self.issues.clone(),
            pure_count,
            stateful_count,
            risk_score,
        })
    }

    /// Verify a `.ai` file from disk.
    pub fn verify_file(&mut self, path: &Path) -> Result<VerificationReport, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.ai");
        self.verify_source(&source, file_name)
    }

    // ── AST Analysis ─────────────────────────────────────────────────────────

    fn analyze_ast(&mut self, ast: &ASTNode) {
        match ast {
            ASTNode::Program(stmts) => {
                for stmt in stmts {
                    self.analyze_ast(stmt);
                }
            }
            ASTNode::Function { name, params, body, line, .. } => {
                let mut calls = Vec::new();
                let mut reads_state = false;
                let mut writes_state = false;
                let mut has_recursion = false;

                // Track local scope: parameters and locally-declared variables
                let mut locals: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();

                for stmt in body {
                    self.scan_function_body(stmt, name, &locals, &mut calls, &mut reads_state, &mut writes_state, &mut has_recursion);
                    // Track variable declarations as local
                    if let ASTNode::VariableDecl { name: vname, .. } = stmt {
                        locals.insert(vname.clone());
                    }
                }

                // Classify: a function is Balanced (stateful) if it writes state
                // or calls external/side-effecting builtins
                let has_side_effects = writes_state
                    || calls.iter().any(|c| STATE_WRITE_BUILTINS.contains(&c.as_str()))
                    || calls.iter().any(|c| EXTERNAL_BUILTINS.contains(&c.as_str()));

                let purity = if has_side_effects {
                    FunctionPurity::Balanced
                } else {
                    FunctionPurity::Constant
                };

                self.functions.insert(name.clone(), FunctionAnalysis {
                    name: name.clone(),
                    purity,
                    param_count: params.len(),
                    calls_external: calls,
                    reads_state,
                    writes_state,
                    has_recursion,
                    line: *line,
                });
            }
            ASTNode::VariableDecl { name, .. } | ASTNode::Assignment { name, .. } => {
                self.global_writes.insert(name.clone());
            }
            _ => {}
        }
    }

    fn scan_function_body(
        &self,
        node: &ASTNode,
        fn_name: &str,
        locals: &HashSet<String>,
        calls: &mut Vec<String>,
        reads_state: &mut bool,
        writes_state: &mut bool,
        has_recursion: &mut bool,
    ) {
        match node {
            ASTNode::Call { callee, args } => {
                if let Some(name) = self.extract_call_name(callee) {
                    if name == fn_name {
                        *has_recursion = true;
                    }
                    if STATE_READ_BUILTINS.contains(&name.as_str()) {
                        *reads_state = true;
                    }
                    if STATE_WRITE_BUILTINS.contains(&name.as_str()) {
                        *writes_state = true;
                    }
                    calls.push(name);
                }
                for arg in args {
                    self.scan_function_body(arg, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                }
            }
            ASTNode::Assignment { name, .. } => {
                // Only flag as state mutation if assigning to a non-local variable
                if !locals.contains(name) {
                    *writes_state = true;
                }
            }
            ASTNode::Block(stmts) => {
                for s in stmts {
                    self.scan_function_body(s, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                }
            }
            ASTNode::If { condition, then_branch, else_branch } => {
                self.scan_function_body(condition, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                self.scan_function_body(then_branch, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                if let Some(eb) = else_branch {
                    self.scan_function_body(eb, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                }
            }
            ASTNode::While { condition, body } => {
                self.scan_function_body(condition, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                self.scan_function_body(body, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::For { init, condition, increment, body } => {
                if let Some(i) = init { self.scan_function_body(i, fn_name, locals, calls, reads_state, writes_state, has_recursion); }
                if let Some(c) = condition { self.scan_function_body(c, fn_name, locals, calls, reads_state, writes_state, has_recursion); }
                if let Some(inc) = increment { self.scan_function_body(inc, fn_name, locals, calls, reads_state, writes_state, has_recursion); }
                self.scan_function_body(body, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::Return(expr) => {
                self.scan_function_body(expr, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::Log(expr) => {
                self.scan_function_body(expr, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::VariableDecl { value, .. } => {
                self.scan_function_body(value, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::BinaryExpr { left, right, .. } => {
                self.scan_function_body(left, fn_name, locals, calls, reads_state, writes_state, has_recursion);
                self.scan_function_body(right, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            ASTNode::UnaryExpr { expr, .. } => {
                self.scan_function_body(expr, fn_name, locals, calls, reads_state, writes_state, has_recursion);
            }
            _ => {}
        }
    }

    fn extract_call_name(&self, callee: &ASTNode) -> Option<String> {
        match callee {
            ASTNode::Identifier(name) => Some(name.clone()),
            ASTNode::IdentifierSpanned { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    // ── Verification Checks ──────────────────────────────────────────────────

    fn check_reentrancy(&mut self) {
        // Check for functions that call external functions AND write state
        for fa in self.functions.values() {
            let has_external = fa.calls_external.iter().any(|c| EXTERNAL_BUILTINS.contains(&c.as_str()));
            if has_external && fa.writes_state {
                self.issues.push(VerificationIssue {
                    severity: IssueSeverity::Critical,
                    category: "reentrancy".to_string(),
                    message: format!(
                        "Function '{}' writes state and calls external functions — potential re-entrancy risk",
                        fa.name
                    ),
                    function: Some(fa.name.clone()),
                    line: Some(fa.line),
                });
            }
        }
    }

    fn check_unchecked_external_calls(&mut self) {
        // Warn about functions that call external builtins without error handling
        for fa in self.functions.values() {
            for call in &fa.calls_external {
                if EXTERNAL_BUILTINS.contains(&call.as_str()) {
                    self.issues.push(VerificationIssue {
                        severity: IssueSeverity::Warning,
                        category: "unchecked-external".to_string(),
                        message: format!(
                            "Function '{}' calls '{}' — ensure error handling is in place",
                            fa.name, call
                        ),
                        function: Some(fa.name.clone()),
                        line: Some(fa.line),
                    });
                }
            }
        }
    }

    fn check_infinite_recursion(&mut self) {
        for fa in self.functions.values() {
            if fa.has_recursion {
                self.issues.push(VerificationIssue {
                    severity: IssueSeverity::Warning,
                    category: "recursion".to_string(),
                    message: format!(
                        "Function '{}' is recursive — verify termination condition exists",
                        fa.name
                    ),
                    function: Some(fa.name.clone()),
                    line: Some(fa.line),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_function_detection() {
        let src = r#"
function add(a, b) {
    return a + b;
}
"#;
        let mut v = ContractVerifier::new();
        let report = v.verify_source(src, "test.ai").unwrap();
        assert_eq!(report.function_count, 1);
        assert_eq!(report.pure_count, 1);
        assert_eq!(report.stateful_count, 0);
    }

    #[test]
    fn test_stateful_function_detection() {
        let src = r#"
function save_data(path, data) {
    write_file(path, data);
}
"#;
        let mut v = ContractVerifier::new();
        let report = v.verify_source(src, "test.ai").unwrap();
        assert_eq!(report.function_count, 1);
        assert_eq!(report.stateful_count, 1);
    }

    #[test]
    fn test_recursive_function_warning() {
        let src = r#"
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;
        let mut v = ContractVerifier::new();
        let report = v.verify_source(src, "test.ai").unwrap();
        let recursion_issues: Vec<_> = report.issues.iter()
            .filter(|i| i.category == "recursion")
            .collect();
        assert!(!recursion_issues.is_empty(), "Should detect recursion");
    }

    #[test]
    fn test_report_json_output() {
        let src = r#"
function pure_fn(x) {
    return x + 1;
}
"#;
        let mut v = ContractVerifier::new();
        let report = v.verify_source(src, "contract.ai").unwrap();
        let json = report.to_json();
        assert!(json.contains("contract.ai"));
        assert!(json.contains("pure_fn"));
    }

    #[test]
    fn test_empty_contract() {
        let src = "let x = 42;";
        let mut v = ContractVerifier::new();
        let report = v.verify_source(src, "empty.ai").unwrap();
        assert_eq!(report.function_count, 0);
        assert!(report.risk_score < 1.0);
    }
}
