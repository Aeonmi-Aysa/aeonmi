use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use colored::Colorize;

/// Diagnostic level for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
}

impl DiagnosticLevel {
    fn label(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "error",
        }
    }

    fn color_label(&self) -> colored::ColoredString {
        match self {
            DiagnosticLevel::Error => self.label().red().bold(),
        }
    }
}

/// A diagnostic message for parse/runtime errors
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub hint: Option<String>,
    pub source_line: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            file: None,
            line: None,
            hint: None,
            source_line: None,
        }
    }

    pub fn with_location(mut self, file: impl Into<String>, line: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Format the diagnostic with colors for terminal display
    pub fn format_pretty(&self) -> String {
        let mut output = String::new();

        // Header line: error[E001]: message
        let header = if let (Some(file), Some(line)) = (&self.file, self.line) {
            format!(
                "{}: {}\n  {} {}:{}",
                self.level.color_label(),
                self.message.bold(),
                "-->".blue().bold(),
                file,
                line
            )
        } else {
            format!("{}: {}", self.level.color_label(), self.message.bold())
        };
        output.push_str(&header);
        output.push('\n');

        // Show source line if available
        if let Some(source) = &self.source_line {
            output.push_str("   |\n");
            output.push_str(&format!("{:3} | {}\n", self.line.unwrap_or(0), source));
            output.push_str("   |\n");
        }

        // Show hint if available
        if let Some(hint) = &self.hint {
            output.push_str(&format!("   {} {}\n", "=".cyan(), hint.cyan()));
        }

        output
    }

    /// Format the diagnostic without colors for file logging
    pub fn format_plain(&self) -> String {
        let mut output = String::new();

        // Header line
        let header = if let (Some(file), Some(line)) = (&self.file, self.line) {
            format!(
                "{}: {}\n  --> {}:{}",
                self.level.label(),
                self.message,
                file,
                line
            )
        } else {
            format!("{}: {}", self.level.label(), self.message)
        };
        output.push_str(&header);
        output.push('\n');

        // Show source line if available
        if let Some(source) = &self.source_line {
            output.push_str("   |\n");
            output.push_str(&format!("{:3} | {}\n", self.line.unwrap_or(0), source));
            output.push_str("   |\n");
        }

        // Show hint if available
        if let Some(hint) = &self.hint {
            output.push_str(&format!("   = {}\n", hint));
        }

        output
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_pretty())
    }
}

/// Global diagnostic logger that writes to both stderr and output/log.txt
pub struct DiagnosticLogger {
    log_file: Option<File>,
}

impl DiagnosticLogger {
    pub fn new() -> Self {
        Self { log_file: None }
    }

    pub fn with_log_file(log_path: &Path) -> std::io::Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(Self {
            log_file: Some(log_file),
        })
    }

    pub fn emit(&mut self, diagnostic: &Diagnostic) {
        // Print to stderr with colors
        eprintln!("{}", diagnostic.format_pretty());

        // Write to log file without colors
        if let Some(ref mut file) = self.log_file {
            let _ = writeln!(file, "{}", diagnostic.format_plain());
        }
    }

    pub fn emit_error(&mut self, message: &str, file: Option<&str>, line: Option<usize>, hint: Option<&str>) {
        let mut diag = Diagnostic::error(message);
        if let (Some(f), Some(l)) = (file, line) {
            diag = diag.with_location(f, l);
        }
        if let Some(h) = hint {
            diag = diag.with_hint(h);
        }
        self.emit(&diag);
    }
}

impl Default for DiagnosticLogger {
    fn default() -> Self {
        Self::new()
    }
}
