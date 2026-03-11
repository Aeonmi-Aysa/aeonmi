//! QUBE parser — reuses Aeonmi lexer tokens, own grammar rules.

use crate::core::token::{Token, TokenKind};
use crate::core::qube_ast::{AssertOp, QubeExpr, QubeNode, SuperpositionTerm};

#[derive(Debug)]
pub struct QubeParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for QubeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QUBE parse error at {}:{}: {}", self.line, self.column, self.message)
    }
}

pub struct QubeParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl QubeParser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        if !matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::EOF)) {
            tokens.push(Token {
                kind: TokenKind::EOF,
                lexeme: String::new(),
                line: 0,
                column: 0,
            });
        }
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<QubeNode, QubeParseError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            // skip blank lines / newlines represented as EOF lookahead
            if matches!(self.peek().kind, TokenKind::EOF) {
                break;
            }
            // skip comment tokens
            if matches!(
                self.peek().kind,
                TokenKind::QuantumComment | TokenKind::BecauseComment | TokenKind::NoteComment
            ) {
                // consume until newline (lexer already consumed the comment content)
                self.advance();
                continue;
            }
            stmts.push(self.parse_statement()?);
        }
        Ok(QubeNode::Program(stmts))
    }

    fn parse_statement(&mut self) -> Result<QubeNode, QubeParseError> {
        match self.peek().kind.clone() {
            // "state" keyword — but lexer produces Identifier("state")
            TokenKind::Identifier(ref s) if s == "state" => {
                self.advance();
                self.parse_state_decl()
            }
            // "apply" keyword
            TokenKind::Identifier(ref s) if s == "apply" => {
                self.advance();
                self.parse_gate_apply()
            }
            // "collapse" keyword
            TokenKind::Identifier(ref s) if s == "collapse" => {
                self.advance();
                self.parse_collapse()
            }
            // "assert" keyword
            TokenKind::Identifier(ref s) if s == "assert" => {
                self.advance();
                self.parse_assert()
            }
            // log / print
            TokenKind::Log | TokenKind::Identifier(ref _s) => {
                // handle both TokenKind::Log and Identifier("print"/"log")
                self.advance();
                self.parse_log()
            }
            _ => {
                let tok = self.advance().clone();
                Err(self.err_at(
                    &format!("Unexpected token {:?}", tok.kind),
                    tok.line,
                    tok.column,
                ))
            }
        }
    }

    /// state <name> = <expr>
    fn parse_state_decl(&mut self) -> Result<QubeNode, QubeParseError> {
        let name = self.consume_ident("Expected state name after 'state'")?;
        self.consume_eq("Expected '=' after state name")?;
        let value = self.parse_qube_expr()?;
        let _ = self.match_tok(&[TokenKind::Semicolon]);
        Ok(QubeNode::StateDecl { name, value })
    }

    /// apply <Gate> -> <target>
    /// apply <Gate>(<control>, <target>)
    fn parse_gate_apply(&mut self) -> Result<QubeNode, QubeParseError> {
        let gate = self.consume_ident("Expected gate name after 'apply'")?;

        // Optional angle for rotation gates: apply Rx(1.5708) -> q
        // or two-qubit: apply CNOT(q0, q1)
        let mut angle: Option<f64> = None;
        let mut targets: Vec<String> = Vec::new();

        if self.match_tok(&[TokenKind::OpenParen]) {
            // parse comma-separated idents or a single number
            loop {
                match self.peek().kind.clone() {
                    TokenKind::NumberLiteral(n) => {
                        angle = Some(n);
                        self.advance();
                    }
                    TokenKind::Identifier(name) => {
                        targets.push(name);
                        self.advance();
                    }
                    _ => break,
                }
                if !self.match_tok(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume_close_paren()?;
        }

        // -> target (single-qubit form)
        if self.match_tok(&[TokenKind::Arrow]) {
            let target = self.consume_ident("Expected target qubit after '->'")?;
            targets.push(target);
        }

        let _ = self.match_tok(&[TokenKind::Semicolon]);
        Ok(QubeNode::GateApply { gate, targets, angle })
    }

    /// collapse <state> -> <result>
    fn parse_collapse(&mut self) -> Result<QubeNode, QubeParseError> {
        let state = self.consume_ident("Expected state name after 'collapse'")?;
        self.consume_arrow("Expected '->' after state name in collapse")?;
        let result = self.consume_ident("Expected result variable name")?;
        let _ = self.match_tok(&[TokenKind::Semicolon]);
        Ok(QubeNode::Collapse { state, result })
    }

    /// assert <state> ∈ {0, 1}
    /// assert <state> == <value>
    fn parse_assert(&mut self) -> Result<QubeNode, QubeParseError> {
        let state = self.consume_ident("Expected variable name after 'assert'")?;

        let op = match self.peek().kind.clone() {
            TokenKind::QuantumIn => {
                self.advance();
                AssertOp::In
            }
            TokenKind::DoubleEquals => {
                self.advance();
                AssertOp::Eq
            }
            other => {
                return Err(self.err_here(&format!(
                    "Expected '∈' or '==' in assert, got {:?}",
                    other
                )))
            }
        };

        let values = match op {
            AssertOp::In => {
                self.consume_open_brace()?;
                let mut vals = Vec::new();
                while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                    vals.push(self.parse_qube_expr()?);
                    if !self.match_tok(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume_close_brace()?;
                vals
            }
            AssertOp::Eq => {
                vec![self.parse_qube_expr()?]
            }
        };

        let _ = self.match_tok(&[TokenKind::Semicolon]);
        Ok(QubeNode::Assert { state, op, values })
    }

    /// log(<expr>)
    fn parse_log(&mut self) -> Result<QubeNode, QubeParseError> {
        // 'log' keyword or 'print' ident already consumed by caller
        if self.match_tok(&[TokenKind::OpenParen]) {
            let expr = self.parse_qube_expr()?;
            self.consume_close_paren()?;
            let _ = self.match_tok(&[TokenKind::Semicolon]);
            Ok(QubeNode::Log(expr))
        } else {
            // bare: log expr
            let expr = self.parse_qube_expr()?;
            let _ = self.match_tok(&[TokenKind::Semicolon]);
            Ok(QubeNode::Log(expr))
        }
    }

    /// Parse a QUBE expression:
    /// - qubit literal: |0⟩ |1⟩ |+⟩ |ψ⟩
    /// - superposition: 0.707|0⟩ + 0.707|1⟩
    /// - identifier
    /// - number
    /// - string
    fn parse_qube_expr(&mut self) -> Result<QubeExpr, QubeParseError> {
        // Check for number-prefixed superposition term: 0.707|0⟩ + ...
        if let TokenKind::NumberLiteral(amp) = self.peek().kind.clone() {
            let next_pos = self.pos + 1;
            if next_pos < self.tokens.len() {
                if let TokenKind::QubitLiteral(_) = &self.tokens[next_pos].kind {
                    return self.parse_superposition_expr();
                }
            }
            self.advance();
            return Ok(QubeExpr::Number(amp));
        }

        match self.peek().kind.clone() {
            TokenKind::QubitLiteral(lit) => {
                self.advance();
                // Could be start of superposition: |0⟩ + 0.707|1⟩
                if self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
                    // Build superposition starting from this term
                    let mut terms = vec![SuperpositionTerm { amplitude: 1.0, state: lit }];
                    while self.match_tok(&[TokenKind::Plus]) || self.match_tok(&[TokenKind::Minus]) {
                        let sign = if self.previous().kind == TokenKind::Minus { -1.0 } else { 1.0 };
                        let amp = if let TokenKind::NumberLiteral(n) = self.peek().kind.clone() {
                            self.advance();
                            n * sign
                        } else {
                            sign
                        };
                        if let TokenKind::QubitLiteral(s) = self.peek().kind.clone() {
                            self.advance();
                            terms.push(SuperpositionTerm { amplitude: amp, state: s });
                        }
                    }
                    Ok(QubeExpr::Superposition(terms))
                } else {
                    Ok(QubeExpr::QubitLiteral(lit))
                }
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(QubeExpr::Ident(name))
            }
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(QubeExpr::Str(s))
            }
            other => Err(self.err_here(&format!("Expected QUBE expression, got {:?}", other))),
        }
    }

    fn parse_superposition_expr(&mut self) -> Result<QubeExpr, QubeParseError> {
        let mut terms = Vec::new();
        loop {
            let amp = if let TokenKind::NumberLiteral(n) = self.peek().kind.clone() {
                self.advance();
                n
            } else {
                1.0
            };
            if let TokenKind::QubitLiteral(s) = self.peek().kind.clone() {
                self.advance();
                terms.push(SuperpositionTerm { amplitude: amp, state: s });
            } else {
                break;
            }
            if !self.match_tok(&[TokenKind::Plus]) {
                break;
            }
        }
        Ok(QubeExpr::Superposition(terms))
    }

    // ── Token utilities ──────────────────────────────────────────

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.pos += 1; }
        self.previous()
    }

    fn previous(&self) -> &Token {
        if self.pos == 0 { &self.tokens[0] } else { &self.tokens[self.pos - 1] }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && &self.peek().kind == kind
    }

    fn match_tok(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.check(k) { self.advance(); return true; }
        }
        false
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }

    fn consume_ident(&mut self, msg: &str) -> Result<String, QubeParseError> {
        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.err_here(msg))
        }
    }

    fn consume_eq(&mut self, msg: &str) -> Result<(), QubeParseError> {
        if self.match_tok(&[TokenKind::Equals]) { Ok(()) } else { Err(self.err_here(msg)) }
    }

    fn consume_arrow(&mut self, msg: &str) -> Result<(), QubeParseError> {
        if self.match_tok(&[TokenKind::Arrow]) { Ok(()) } else { Err(self.err_here(msg)) }
    }

    fn consume_close_paren(&mut self) -> Result<(), QubeParseError> {
        if self.match_tok(&[TokenKind::CloseParen]) { Ok(()) } else { Err(self.err_here("Expected ')'")) }
    }

    fn consume_open_brace(&mut self) -> Result<(), QubeParseError> {
        if self.match_tok(&[TokenKind::OpenBrace]) { Ok(()) } else { Err(self.err_here("Expected '{'")) }
    }

    fn consume_close_brace(&mut self) -> Result<(), QubeParseError> {
        if self.match_tok(&[TokenKind::CloseBrace]) { Ok(()) } else { Err(self.err_here("Expected '}'")) }
    }

    fn err_here(&self, msg: &str) -> QubeParseError {
        let tok = self.peek();
        self.err_at(msg, tok.line, tok.column)
    }

    fn err_at(&self, msg: &str, line: usize, column: usize) -> QubeParseError {
        QubeParseError { message: msg.into(), line, column }
    }
}
