// src/core/diagnostics.rs
//! Pretty, colored, file+line diagnostics with quantum-specific error messages and suggestions.

use colored::Colorize;

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize, // underline length (use 1 if unknown)
}

impl Span {
    pub fn single(line: usize, col: usize) -> Self {
        Self { line, col, len: 1 }
    }
}

/// Enhanced error with contextual suggestions
#[derive(Debug)]
pub struct QuantumDiagnostic {
    pub title: String,
    pub span: Span,
    pub suggestion: Option<String>,
    pub help: Option<String>,
    pub quantum_context: Option<String>,
}

impl QuantumDiagnostic {
    pub fn new(title: String, span: Span) -> Self {
        Self {
            title,
            span,
            suggestion: None,
            help: None,
            quantum_context: None,
        }
    }
    
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
    
    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }
    
    pub fn with_quantum_context(mut self, context: String) -> Self {
        self.quantum_context = Some(context);
        self
    }
}

pub fn print_error(filename: &str, source: &str, title: &str, span: Span) {
    let diag = QuantumDiagnostic::new(title.to_string(), span);
    print_quantum_diagnostic(filename, source, &diag);
}

pub fn print_quantum_diagnostic(filename: &str, source: &str, diag: &QuantumDiagnostic) {
    eprintln!("{} {}", "error:".bright_red().bold(), diag.title.bright_white());
    let (ln, col) = (diag.span.line, diag.span.col);
    let line_text = nth_line(source, ln).unwrap_or_default();

    // line number gutter
    let ln_str = format!("{:>4}", ln);
    eprintln!(
        "{} {}",
        "-->".bright_blue(),
        format!("{}:{}:{}", filename, ln, col).bright_white()
    );
    eprintln!(" {} {}", ln_str.dimmed(), "|".dimmed());
    eprintln!("{} {} {}", ln_str.dimmed(), "|".dimmed(), line_text);

    // underline with ^^^^^
    let underline = " ".repeat(col.saturating_sub(1)) + &"^".repeat(diag.span.len.max(1));
    eprintln!(
        " {} {} {}",
        " ".repeat(ln_str.len()).dimmed(),
        "|".dimmed(),
        underline.bright_red()
    );
    
    // Show suggestion if available
    if let Some(suggestion) = &diag.suggestion {
        eprintln!(" {} {} {}", " ".repeat(ln_str.len()).dimmed(), "|".dimmed(), " ");
        eprintln!("{} {}", "suggestion:".bright_yellow().bold(), suggestion.bright_white());
    }
    
    // Show help if available
    if let Some(help) = &diag.help {
        eprintln!("{} {}", "help:".bright_cyan().bold(), help.bright_white());
    }
    
    // Show quantum context if available
    if let Some(context) = &diag.quantum_context {
        eprintln!("{} {}", "quantum:".bright_magenta().bold(), context.bright_white());
    }
    
    eprintln!();
}

/// Create quantum-specific error diagnostics with helpful suggestions
pub fn quantum_syntax_error(message: &str, span: Span) -> QuantumDiagnostic {
    let diag = QuantumDiagnostic::new(message.to_string(), span);
    
    // Add context-specific suggestions based on the error message
    match message {
        msg if msg.contains("Expected quantum binding operator") => {
            diag.with_suggestion("Use ← for classical binding, ∈ for superposition, ⊗ for tensor, or ≈ for approximation".to_string())
                .with_help("Quantum variables use different binding operators than classical languages".to_string())
                .with_quantum_context("⟨variable⟩ ← value (classical) or ⟨variable⟩ ∈ |0⟩ + |1⟩ (superposition)".to_string())
        }
        msg if msg.contains("Expected '⟨'") => {
            diag.with_suggestion("Quantum variables are declared with ⟨name⟩".to_string())
                .with_help("AEONMI uses quantum brackets ⟨⟩ instead of traditional syntax".to_string())
        }
        msg if msg.contains("quantum state") => {
            diag.with_suggestion("Quantum states use |state⟩ notation, like |0⟩, |1⟩, |+⟩, |-⟩".to_string())
                .with_quantum_context("Common states: |0⟩ (zero), |1⟩ (one), |+⟩ (plus), |-⟩ (minus)".to_string())
        }
        msg if msg.contains("function") => {
            diag.with_suggestion("Use ◯ for classical functions, ⊙ for quantum functions, 🧠 for AI functions".to_string())
                .with_help("AEONMI distinguishes function types with unique symbols".to_string())
        }
        msg if msg.contains("probability") => {
            diag.with_suggestion("Use ⊖ condition ≈ 0.7 ⇒ { ... } for probability-based branching".to_string())
                .with_quantum_context("Probability values should be between 0.0 and 1.0".to_string())
        }
        _ => diag
    }
}

/// Create specific error for unsupported legacy syntax with migration suggestions
pub fn legacy_syntax_error(legacy_syntax: &str, span: Span) -> QuantumDiagnostic {
    let (title, suggestion, help) = match legacy_syntax {
        "let" => (
            "Legacy 'let' keyword detected",
            "Use quantum variable declaration: ⟨variable⟩ ← value",
            "AEONMI uses quantum-native syntax. Traditional 'let' is deprecated."
        ),
        "function" => (
            "Legacy 'function' keyword detected", 
            "Use ◯ name⟨params⟩ for classical functions or ⊙ name⟨params⟩ for quantum functions",
            "AEONMI distinguishes between classical (◯) and quantum (⊙) functions"
        ),
        "if" => (
            "Legacy 'if' statement detected",
            "Use ⊖ condition ⇒ { ... } for probability-based branching",
            "AEONMI supports quantum probability-aware control flow"
        ),
        "while" => (
            "Legacy 'while' loop detected",
            "Use ⟲ condition ⇒ { ... } for quantum loops with decoherence awareness",
            "Quantum loops can include decoherence thresholds"
        ),
        _ => (
            "Legacy syntax detected",
            "Migrate to AEONMI quantum-native syntax",
            "AEONMI provides quantum-first programming constructs"
        )
    };
    
    QuantumDiagnostic::new(title.to_string(), span)
        .with_suggestion(suggestion.to_string())
        .with_help(help.to_string())
        .with_quantum_context("See docs/AEONMI_UNIQUE_SYNTAX.md for migration guide".to_string())
}

#[derive(serde::Serialize)]
pub struct JsonDiagnostic<'a> {
    pub severity: &'a str,
    pub message: &'a str,
    pub file: &'a str,
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

/// Emit a machine-readable JSON line (prefixed) for downstream tools (GUI, editors).
pub fn emit_json_error(file: &str, title: &str, span: &Span) {
    let jd = JsonDiagnostic { severity: "error", message: title, file, line: span.line, col: span.col, len: span.len };
    if let Ok(s) = serde_json::to_string(&jd) {
        eprintln!("@@DIAG:{}", s);
    }
}

fn nth_line(src: &str, n: usize) -> Option<String> {
    src.lines().nth(n.saturating_sub(1)).map(|s| s.to_string())
}
