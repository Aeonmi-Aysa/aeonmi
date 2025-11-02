#![allow(dead_code)]

use serde::{Deserialize, Serialize};
// src/core/token.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Identifiers and literals
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    QubitLiteral(String),

    // Traditional operators (legacy compatibility)
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Equals,       // =
    DoubleEquals, // ==
    NotEquals,    // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    ColonEquals,  // :=
    Pipe,         // |
    AndAnd,       // &&
    OrOr,         // ||

    // AEONMI Quantum-Native Operators
    QuantumBind,     // ←
    QuantumIn,       // ∈
    QuantumTensor,   // ⊗
    QuantumApprox,   // ≈
    QuantumXor,      // ⊕
    QuantumOr,       // ⊖
    QuantumNot,      // ⊄
    QuantumGradient, // ∇
    QuantumGeq,      // ⪰
    QuantumLeq,      // ⪯
    QuantumImplies,  // ⇒
    QuantumLoop,     // ⟲
    QuantumModulo,   // ◊

    // Delimiters for quantum syntax
    QuantumBracketOpen,  // ⟨
    QuantumBracketClose, // ⟩
    QuantumIndexOpen,    // ⟦
    QuantumIndexClose,   // ⟧

    // Delimiters
    OpenParen,  // (
    CloseParen, // )
    OpenBrace,  // {
    CloseBrace, // }
    OpenBracket, // [
    CloseBracket, // ]
    Comma,      // ,
    Semicolon,  // ;

    // Traditional keywords (legacy compatibility)
    Function,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Log,
    Qubit,

    // AEONMI Quantum-Native Keywords
    ClassicalFunc, // ◯
    QuantumFunc,   // ⊙
    AIFunc,        // 🧠
    Learn,         // learn block
    Attempt,       // ⚡ (quantum try)
    Warning,       // ⚠️ (quantum catch)
    Success,       // ✓ (quantum success)
    TimeBlock,     // ⏰/⏱️

    // State and measurement keywords
    QuantumState,       // quantum state literals like |0⟩, |1⟩, |+⟩
    SuperpositionState, // superposition expressions

    // Comments (for parsing structured comments)
    QuantumComment, // ∴ (therefore)
    BecauseComment, // ∵ (because)
    NoteComment,    // ※ (note)

    // Quantum operations
    Superpose,
    Entangle,
    Measure,
    Dod,

    // Hieroglyphic operations
    HieroglyphicOp(String),

    // Special
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

// Implement Display for TokenKind for better error messages
impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            TokenKind::Identifier(_) => "identifier",
            TokenKind::NumberLiteral(_) => "number",
            TokenKind::StringLiteral(_) => "string",
            TokenKind::BooleanLiteral(_) => "boolean",
            TokenKind::QubitLiteral(_) => "qubit",

            // Traditional operators
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Equals => "=",
            TokenKind::DoubleEquals => "==",
            TokenKind::NotEquals => "!=",
            TokenKind::LessThan => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::GreaterThan => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::ColonEquals => ":=",
            TokenKind::Pipe => "|",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",

            // Quantum operators
            TokenKind::QuantumBind => "←",
            TokenKind::QuantumIn => "∈",
            TokenKind::QuantumTensor => "⊗",
            TokenKind::QuantumApprox => "≈",
            TokenKind::QuantumXor => "⊕",
            TokenKind::QuantumOr => "⊖",
            TokenKind::QuantumNot => "⊄",
            TokenKind::QuantumGradient => "∇",
            TokenKind::QuantumGeq => "⪰",
            TokenKind::QuantumLeq => "⪯",
            TokenKind::QuantumImplies => "⇒",
            TokenKind::QuantumLoop => "⟲",
            TokenKind::QuantumModulo => "◊",

            // Quantum delimiters
            TokenKind::QuantumBracketOpen => "⟨",
            TokenKind::QuantumBracketClose => "⟩",
            TokenKind::QuantumIndexOpen => "⟦",
            TokenKind::QuantumIndexClose => "⟧",

            // Traditional delimiters
            TokenKind::OpenParen => "(",
            TokenKind::CloseParen => ")",
            TokenKind::OpenBrace => "{",
            TokenKind::CloseBrace => "}",
            TokenKind::OpenBracket => "[",
            TokenKind::CloseBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",

            // Traditional keywords
            TokenKind::Function => "function",
            TokenKind::Let => "let",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Return => "return",
            TokenKind::Log => "log",
            TokenKind::Qubit => "qubit",

            // Quantum-native keywords
            TokenKind::ClassicalFunc => "◯",
            TokenKind::QuantumFunc => "⊙",
            TokenKind::AIFunc => "🧠",
            TokenKind::Learn => "learn",
            TokenKind::Attempt => "⚡",
            TokenKind::Warning => "⚠️",
            TokenKind::Success => "✓",
            TokenKind::TimeBlock => "⏰",
            TokenKind::QuantumState => "quantum_state",
            TokenKind::SuperpositionState => "superposition",
            TokenKind::QuantumComment => "∴",
            TokenKind::BecauseComment => "∵",
            TokenKind::NoteComment => "※",

            // Quantum operations
            TokenKind::Superpose => "superpose",
            TokenKind::Entangle => "entangle",
            TokenKind::Measure => "measure",
            TokenKind::Dod => "dod",
            TokenKind::HieroglyphicOp(_) => "hieroglyphic",
            TokenKind::EOF => "end of file",
        };
        write!(f, "{}", name)
    }
}

// Implement Display for full Token (kind plus optional lexeme snippet)
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TokenKind::Identifier(name) => {
                write!(f, "Identifier('{}') @{}:{}", name, self.line, self.column)
            }
            TokenKind::NumberLiteral(v) => {
                write!(f, "Number({}) @{}:{}", v, self.line, self.column)
            }
            TokenKind::StringLiteral(s) => {
                write!(f, "String(\"{}\") @{}:{}", s, self.line, self.column)
            }
            TokenKind::BooleanLiteral(b) => {
                write!(f, "Boolean({}) @{}:{}", b, self.line, self.column)
            }
            TokenKind::QubitLiteral(q) => write!(f, "Qubit({}) @{}:{}", q, self.line, self.column),
            TokenKind::HieroglyphicOp(sym) => {
                write!(f, "Hieroglyphic('{}') @{}:{}", sym, self.line, self.column)
            }
            other => write!(f, "{} @{}:{}", other, self.line, self.column),
        }
    }
}
