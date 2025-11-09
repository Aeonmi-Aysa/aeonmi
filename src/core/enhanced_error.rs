//! Enhanced position and error reporting structures for Aeonmi compiler
//!
//! This module provides unified position tracking with filename information
//! for accurate error reporting in multi-file projects.

use std::fmt;

/// Enhanced position information that includes filename for multi-file support
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Position {
    pub fn new(file: String, line: usize, column: usize, length: usize) -> Self {
        Self {
            file,
            line,
            column,
            length,
        }
    }

    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            length: 0,
        }
    }

    pub fn with_length(&self, length: usize) -> Self {
        Self {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            length,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Span represents a range in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn single(pos: Position) -> Self {
        Self {
            end: pos.clone(),
            start: pos,
        }
    }

    pub fn merge(&self, other: &Span) -> Span {
        let start = if self.start.line < other.start.line
            || (self.start.line == other.start.line && self.start.column < other.start.column)
        {
            self.start.clone()
        } else {
            other.start.clone()
        };

        let end = if self.end.line > other.end.line
            || (self.end.line == other.end.line && self.end.column > other.end.column)
        {
            self.end.clone()
        } else {
            other.end.clone()
        };

        Span { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.file == self.end.file {
            if self.start.line == self.end.line {
                if self.start.column == self.end.column {
                    write!(
                        f,
                        "{}:{}:{}",
                        self.start.file, self.start.line, self.start.column
                    )
                } else {
                    write!(
                        f,
                        "{}:{}:{}-{}",
                        self.start.file, self.start.line, self.start.column, self.end.column
                    )
                }
            } else {
                write!(
                    f,
                    "{}:{}:{}-{}:{}",
                    self.start.file,
                    self.start.line,
                    self.start.column,
                    self.end.line,
                    self.end.column
                )
            }
        } else {
            write!(f, "{} to {}", self.start, self.end)
        }
    }
}

/// Enhanced error with rich positioning information
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub kind: ErrorKind,
    pub span: Span,
    pub suggestion: Option<String>,
    pub related: Vec<(Span, String)>, // Related locations with messages
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    // Lexical errors
    UnexpectedCharacter,
    UnterminatedString,
    InvalidNumber,
    InvalidQuantumLiteral,

    // Syntax errors
    UnexpectedToken,
    MissingToken,
    InvalidSyntax,

    // Semantic errors
    UndefinedVariable,
    UndefinedFunction,
    UndefinedType,
    RedefinedSymbol,
    TypeMismatch,
    InvalidOperation,
    InvalidAccess,

    // Control flow errors
    MissingReturn,
    UnreachableCode,
    InvalidBreak,
    InvalidContinue,

    // Import/Module errors
    ModuleNotFound,
    CircularDependency,
    ImportError,
    IOError,

    // Quantum-specific errors
    InvalidQuantumOperation,
    QuantumStateError,
    EntanglementError,
}

impl CompilerError {
    pub fn new(message: String, kind: ErrorKind, span: Span) -> Self {
        Self {
            message,
            kind,
            span,
            suggestion: None,
            related: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_related(mut self, span: Span, message: String) -> Self {
        self.related.push((span, message));
        self
    }

    pub fn is_error(&self) -> bool {
        !matches!(self.kind, ErrorKind::UnreachableCode) // Most are errors, some might be warnings
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {} at {}", self.message, self.span)?;

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  suggestion: {}", suggestion)?;
        }

        for (span, msg) in &self.related {
            write!(f, "\n  note: {} at {}", msg, span)?;
        }

        Ok(())
    }
}

impl std::error::Error for CompilerError {}

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, Box<CompilerError>>;

/// Multiple errors that can occur during compilation
#[derive(Debug, Clone)]
pub struct CompilerErrors {
    pub errors: Vec<CompilerError>,
}

impl CompilerErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, error: CompilerError) {
        self.errors.push(error);
    }

    pub fn extend(&mut self, errors: Vec<CompilerError>) {
        self.errors.extend(errors);
    }

    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_error())
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for CompilerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompilerErrors {}

impl From<CompilerError> for CompilerErrors {
    fn from(error: CompilerError) -> Self {
        let mut errors = CompilerErrors::new();
        errors.push(error);
        errors
    }
}

/// Trait for types that can be converted to a span
pub trait ToSpan {
    fn to_span(&self, file: &str) -> Span;
}

/// Trait for types that have position information
pub trait HasPosition {
    fn position(&self) -> Position;
    fn span(&self, _file: &str) -> Span {
        Span::single(self.position())
    }
}
