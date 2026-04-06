//! QUBE Lexer — tokenizes .qube source files.
//!
//! Supports both the original symbolic syntax (state/apply/collapse) and the
//! circuit-based syntax (circuit { }, qubit, H, CNOT, measure, execute { run }).

#[derive(Debug, Clone, PartialEq)]
pub enum QubeTok {
    // ── Original symbolic-syntax keywords ──
    KwState,    // state
    KwApply,    // apply
    KwCollapse, // collapse
    KwAssert,   // assert
    KwPrint,    // print (symbolic syntax)
    KwLet,      // let

    // ── Circuit-syntax keywords ──
    KwCircuit,  // circuit
    KwMeta,     // meta
    KwExecute,  // execute
    KwExpected, // expected
    KwRun,      // run
    KwQubit,    // qubit
    KwBit,      // bit
    KwMeasure,  // measure
    KwIf,       // if
    KwReset,    // reset
    KwBarrier,  // barrier
    KwQreg,     // qreg
    KwCreg,     // creg

    // ── Arrows / membership ──
    Arrow,      // →  (Unicode U+2192)
    DashArrow,  // ->  (ASCII dash-gt)
    Member,     // ∈

    // ── Delimiters ──
    Pipe,       // |
    RAngle,     // ⟩
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Colon,      // :
    Semicolon,  // ;
    Equals,     // =
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    TensorOp,   // ⊗

    // ── Literals ──
    Number(f64),
    Ident(String),      // identifiers, gate names, Greek letters
    QubitInner(String), // the content inside |…⟩

    // ── Misc ──
    Comment(String),
    Newline,
    Eof,
    Unknown(char),
}

#[derive(Debug, Clone)]
pub struct QubeLexer {
    src: Vec<char>,
    pos: usize,
    pub line: usize,
    pub col: usize,
}

impl QubeLexer {
    pub fn new(src: &str) -> Self {
        Self {
            src: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<(QubeTok, usize, usize)> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let line = self.line;
            let col = self.col;
            let done = tok == QubeTok::Eof;
            tokens.push((tok, line, col));
            if done { break; }
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.src.get(self.pos).copied();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) -> Option<QubeTok> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => { self.advance(); }
                Some('\n') => { self.advance(); return Some(QubeTok::Newline); }
                Some('/') if self.src.get(self.pos + 1) == Some(&'/') => {
                    // line comment
                    let mut comment = String::new();
                    self.advance(); self.advance(); // consume //
                    while let Some(c) = self.peek() {
                        if c == '\n' { break; }
                        comment.push(c);
                        self.advance();
                    }
                    return Some(QubeTok::Comment(comment.trim().to_string()));
                }
                _ => return None,
            }
        }
    }

    fn read_ident_or_keyword(&mut self) -> QubeTok {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_'
                // Greek letters and other Unicode identifiers
                || ('\u{0370}'..='\u{03FF}').contains(&c)
                || ('\u{1F00}'..='\u{1FFF}').contains(&c)
            {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            // Original symbolic-syntax keywords
            "state"    => QubeTok::KwState,
            "apply"    => QubeTok::KwApply,
            "collapse" => QubeTok::KwCollapse,
            "assert"   => QubeTok::KwAssert,
            "print"    => QubeTok::KwPrint,
            "let"      => QubeTok::KwLet,
            // Circuit-syntax keywords
            "circuit"  => QubeTok::KwCircuit,
            "meta"     => QubeTok::KwMeta,
            "execute"  => QubeTok::KwExecute,
            "expected" => QubeTok::KwExpected,
            "run"      => QubeTok::KwRun,
            "qubit"    => QubeTok::KwQubit,
            "bit"      => QubeTok::KwBit,
            "measure"  => QubeTok::KwMeasure,
            "if"       => QubeTok::KwIf,
            "reset"    => QubeTok::KwReset,
            "barrier"  => QubeTok::KwBarrier,
            "qreg"     => QubeTok::KwQreg,
            "creg"     => QubeTok::KwCreg,
            _ => QubeTok::Ident(s),
        }
    }

    fn read_number(&mut self) -> QubeTok {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        QubeTok::Number(s.parse().unwrap_or(0.0))
    }

    /// Read qubit literal: |…⟩ — we're already past the leading |
    fn read_qubit_inner(&mut self) -> QubeTok {
        let mut inner = String::new();
        while let Some(c) = self.peek() {
            if c == '\u{27E9}' { // ⟩
                self.advance();
                break;
            }
            inner.push(c);
            self.advance();
        }
        QubeTok::QubitInner(inner)
    }

    pub fn next_token(&mut self) -> QubeTok {
        if let Some(ws_tok) = self.skip_whitespace_and_comments() {
            return ws_tok;
        }

        let c = match self.peek() {
            None => return QubeTok::Eof,
            Some(c) => c,
        };

        match c {
            // Multi-char Unicode first
            '→' => { self.advance(); QubeTok::Arrow }
            '∈' => { self.advance(); QubeTok::Member }
            '⟩' => { self.advance(); QubeTok::RAngle }
            '⊗' => { self.advance(); QubeTok::TensorOp }

            '|' => {
                self.advance();
                // Is this a qubit literal? peek for digits or letters followed by ⟩
                if matches!(self.peek(), Some('0') | Some('1') | Some('+') | Some('-') | Some('ψ') | Some('φ') | Some('Φ') | Some('Ω')) {
                    self.read_qubit_inner()
                } else {
                    QubeTok::Pipe
                }
            }

            '{' => { self.advance(); QubeTok::LBrace }
            '}' => { self.advance(); QubeTok::RBrace }
            '(' => { self.advance(); QubeTok::LParen }
            ')' => { self.advance(); QubeTok::RParen }
            '[' => { self.advance(); QubeTok::LBracket }
            ']' => { self.advance(); QubeTok::RBracket }
            ',' => { self.advance(); QubeTok::Comma }
            ':' => { self.advance(); QubeTok::Colon }
            ';' => { self.advance(); QubeTok::Semicolon }
            '=' => { self.advance(); QubeTok::Equals }
            '+' => { self.advance(); QubeTok::Plus }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    QubeTok::DashArrow
                } else {
                    QubeTok::Minus
                }
            }
            '*' => { self.advance(); QubeTok::Star }
            '/' => { self.advance(); QubeTok::Slash }

            c if c.is_ascii_digit() => self.read_number(),

            c if c.is_alphabetic()
                || ('\u{0370}'..='\u{03FF}').contains(&c) // Greek
                || ('\u{1F00}'..='\u{1FFF}').contains(&c) // Greek extended
            => {
                self.read_ident_or_keyword()
            }

            other => { self.advance(); QubeTok::Unknown(other) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok_kinds(src: &str) -> Vec<QubeTok> {
        let mut lex = QubeLexer::new(src);
        lex.tokenize().into_iter().map(|(t, _, _)| t).collect()
    }

    #[test]
    fn test_keywords() {
        let toks = tok_kinds("state apply collapse assert print let");
        assert!(toks.contains(&QubeTok::KwState));
        assert!(toks.contains(&QubeTok::KwApply));
        assert!(toks.contains(&QubeTok::KwCollapse));
    }

    #[test]
    fn test_qubit_literal() {
        let toks = tok_kinds("|0⟩");
        assert!(toks.iter().any(|t| matches!(t, QubeTok::QubitInner(s) if s == "0")));
    }

    #[test]
    fn test_arrow_and_member() {
        let toks = tok_kinds("→ ∈");
        assert!(toks.contains(&QubeTok::Arrow));
        assert!(toks.contains(&QubeTok::Member));
    }

    #[test]
    fn test_number() {
        let toks = tok_kinds("0.707");
        assert!(toks.iter().any(|t| matches!(t, QubeTok::Number(n) if (n - 0.707).abs() < 1e-9)));
    }
}
