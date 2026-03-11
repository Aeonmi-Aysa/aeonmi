//! QUBE Parser — converts tokens into a QubeProgram AST.

use crate::qube::ast::*;
use crate::qube::lexer::{QubeLexer, QubeTok};

pub struct QubeParser {
    tokens: Vec<(QubeTok, usize, usize)>,
    pos: usize,
}

impl QubeParser {
    pub fn new(tokens: Vec<(QubeTok, usize, usize)>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn from_str(src: &str) -> Self {
        let mut lex = QubeLexer::new(src);
        Self::new(lex.tokenize())
    }

    fn peek(&self) -> &QubeTok {
        self.tokens.get(self.pos).map(|(t, _, _)| t).unwrap_or(&QubeTok::Eof)
    }

    fn peek_line(&self) -> usize {
        self.tokens.get(self.pos).map(|(_, l, _)| *l).unwrap_or(0)
    }

    fn advance(&mut self) -> &QubeTok {
        let tok = self.tokens.get(self.pos).map(|(t, _, _)| t).unwrap_or(&QubeTok::Eof);
        self.pos += 1;
        tok
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), QubeTok::Newline) {
            self.advance();
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance().clone() {
            QubeTok::Ident(s) => Ok(s),
            other => Err(format!("Expected identifier, got {:?}", other)),
        }
    }

    pub fn parse(&mut self) -> Result<QubeProgram, String> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !matches!(self.peek(), QubeTok::Eof) {
            match self.peek().clone() {
                QubeTok::Newline => { self.advance(); }
                QubeTok::Comment(c) => {
                    let c = c.clone();
                    self.advance();
                    stmts.push(QubeStmt::Comment(c));
                }
                QubeTok::KwState    => stmts.push(self.parse_state_decl()?),
                QubeTok::KwApply    => stmts.push(self.parse_gate_apply()?),
                QubeTok::KwCollapse => stmts.push(self.parse_collapse()?),
                QubeTok::KwAssert   => stmts.push(self.parse_assert()?),
                QubeTok::KwPrint    => stmts.push(self.parse_print()?),
                QubeTok::KwLet      => stmts.push(self.parse_let()?),
                other => {
                    return Err(format!(
                        "Unexpected token in QUBE program: {:?}",
                        other
                    ));
                }
            }
            self.skip_newlines();
        }

        Ok(QubeProgram { stmts })
    }

    // ── state <name> = <expr> ─────────────────────────────────────────────

    fn parse_state_decl(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `state`
        let name = self.expect_ident()?;
        match self.advance() {
            QubeTok::Equals => {}
            other => return Err(format!("Expected '=' after state name, got {:?}", other)),
        }
        let value = self.parse_state_expr()?;
        Ok(QubeStmt::StateDecl { name, value })
    }

    fn parse_state_expr(&mut self) -> Result<QuantumStateExpr, String> {
        // Could be: |q⟩  OR  α|q⟩ + β|q⟩  OR  name  OR  name ⊗ name
        let first = self.parse_single_state_term()?;

        if matches!(self.peek(), QubeTok::Plus | QubeTok::Minus) {
            // Superposition
            let mut terms = self.extract_term_from_expr(first)?;
            while matches!(self.peek(), QubeTok::Plus | QubeTok::Minus) {
                let sign = if matches!(self.peek(), QubeTok::Minus) { -1.0 } else { 1.0 };
                self.advance();
                let next = self.parse_single_state_term()?;
                let mut next_terms = self.extract_term_from_expr(next)?;
                if sign < 0.0 {
                    // Negate first amplitude
                    for (amp, _) in &mut next_terms {
                        if let QubeAmplitude::Number(n) = amp {
                            *n *= -1.0;
                        }
                    }
                }
                terms.extend(next_terms);
            }
            return Ok(QuantumStateExpr::Superposition { terms });
        }

        if matches!(self.peek(), QubeTok::TensorOp) {
            self.advance(); // consume ⊗
            let right = self.parse_single_state_term()?;
            return Ok(QuantumStateExpr::TensorProduct(
                Box::new(first),
                Box::new(right),
            ));
        }

        Ok(first)
    }

    fn extract_term_from_expr(
        &self,
        expr: QuantumStateExpr,
    ) -> Result<Vec<(QubeAmplitude, QubitState)>, String> {
        match expr {
            QuantumStateExpr::QubitLiteral(q) => Ok(vec![(QubeAmplitude::Implied, q)]),
            QuantumStateExpr::Superposition { terms } => Ok(terms),
            QuantumStateExpr::StateRef(name) => {
                Ok(vec![(QubeAmplitude::Variable(name), QubitState::Zero)])
            }
            _ => Err("Cannot extract term from complex expression".to_string()),
        }
    }

    fn parse_single_state_term(&mut self) -> Result<QuantumStateExpr, String> {
        match self.peek().clone() {
            // Qubit literal |q⟩ — already tokenized as QubitInner
            QubeTok::QubitInner(inner) => {
                let inner = inner.clone();
                self.advance();
                Ok(QuantumStateExpr::QubitLiteral(QubitState::from_inner(&inner)))
            }
            // Pipe: might be start of |q⟩ that wasn't fully tokenized
            QubeTok::Pipe => {
                self.advance();
                let inner = match self.peek().clone() {
                    QubeTok::QubitInner(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    QubeTok::Ident(s) => {
                        let s = s.clone();
                        self.advance();
                        // consume ⟩ if present
                        if matches!(self.peek(), QubeTok::RAngle) { self.advance(); }
                        s
                    }
                    QubeTok::Number(n) => {
                        let s = if n as i64 == 0 { "0" } else { "1" }.to_string();
                        self.advance();
                        if matches!(self.peek(), QubeTok::RAngle) { self.advance(); }
                        s
                    }
                    _ => "0".to_string(),
                };
                Ok(QuantumStateExpr::QubitLiteral(QubitState::from_inner(&inner)))
            }
            // Number coefficient: 0.707|0⟩
            QubeTok::Number(n) => {
                self.advance();
                let state = match self.peek().clone() {
                    QubeTok::QubitInner(inner) => {
                        let inner = inner.clone();
                        self.advance();
                        QubitState::from_inner(&inner)
                    }
                    QubeTok::Pipe => {
                        self.advance();
                        let inner = match self.peek().clone() {
                            QubeTok::Ident(s) => { let s = s.clone(); self.advance(); s }
                            QubeTok::Number(v) => {
                                let s = if v as i64 == 0 { "0" } else { "1" }.to_string();
                                self.advance();
                                s
                            }
                            _ => "0".to_string(),
                        };
                        if matches!(self.peek(), QubeTok::RAngle) { self.advance(); }
                        QubitState::from_inner(&inner)
                    }
                    _ => QubitState::Zero,
                };
                Ok(QuantumStateExpr::Superposition {
                    terms: vec![(QubeAmplitude::Number(n), state)],
                })
            }
            // Identifier: could be α (variable) or a state reference
            QubeTok::Ident(name) => {
                let name = name.clone();
                self.advance();
                // Is it followed by a qubit literal? Then it's a variable coefficient.
                match self.peek().clone() {
                    QubeTok::QubitInner(inner) => {
                        let inner = inner.clone();
                        self.advance();
                        Ok(QuantumStateExpr::Superposition {
                            terms: vec![(QubeAmplitude::Variable(name), QubitState::from_inner(&inner))],
                        })
                    }
                    QubeTok::Pipe => {
                        // variable * |q⟩
                        self.advance();
                        let inner = match self.peek().clone() {
                            QubeTok::Ident(s) => { let s = s.clone(); self.advance(); s }
                            _ => "0".to_string(),
                        };
                        if matches!(self.peek(), QubeTok::RAngle) { self.advance(); }
                        Ok(QuantumStateExpr::Superposition {
                            terms: vec![(QubeAmplitude::Variable(name), QubitState::from_inner(&inner))],
                        })
                    }
                    _ => Ok(QuantumStateExpr::StateRef(name)),
                }
            }
            other => Err(format!("Cannot parse quantum state expression starting with {:?}", other)),
        }
    }

    // ── apply <gate> → <target(s)> ───────────────────────────────────────────

    fn parse_gate_apply(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `apply`
        let gate_name = self.expect_ident()?;
        let gate = QuantumGate::from_str(&gate_name);

        // consume →
        match self.advance() {
            QubeTok::Arrow => {}
            other => return Err(format!("Expected '→' after gate name, got {:?}", other)),
        }

        // Targets: single ident or (q1, q2)
        let targets = if matches!(self.peek(), QubeTok::LParen) {
            self.advance(); // (
            let mut ts = Vec::new();
            ts.push(self.expect_ident()?);
            while matches!(self.peek(), QubeTok::Comma) {
                self.advance();
                ts.push(self.expect_ident()?);
            }
            match self.advance() {
                QubeTok::RParen => {}
                other => return Err(format!("Expected ')' after gate targets, got {:?}", other)),
            }
            ts
        } else {
            vec![self.expect_ident()?]
        };

        Ok(QubeStmt::GateApply { gate, targets })
    }

    // ── collapse <qubit> → <result> ───────────────────────────────────────────

    fn parse_collapse(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `collapse`
        let qubit = self.expect_ident()?;
        match self.advance() {
            QubeTok::Arrow => {}
            other => return Err(format!("Expected '→' in collapse, got {:?}", other)),
        }
        let result = self.expect_ident()?;
        Ok(QubeStmt::Collapse { qubit, result })
    }

    // ── assert <var> ∈ {values} ──────────────────────────────────────────────

    fn parse_assert(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `assert`
        let variable = self.expect_ident()?;
        match self.advance() {
            QubeTok::Member => {}
            other => return Err(format!("Expected '∈' in assert, got {:?}", other)),
        }
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' in assert set, got {:?}", other)),
        }
        let mut values = Vec::new();
        while !matches!(self.peek(), QubeTok::RBrace | QubeTok::Eof) {
            match self.peek().clone() {
                QubeTok::Number(n) => {
                    self.advance();
                    if n.fract() == 0.0 {
                        values.push(AssertValue::Integer(n as i64));
                    } else {
                        values.push(AssertValue::Float(n));
                    }
                }
                QubeTok::QubitInner(inner) => {
                    let inner = inner.clone();
                    self.advance();
                    values.push(AssertValue::State(QubitState::from_inner(&inner)));
                }
                QubeTok::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    values.push(AssertValue::Variable(name));
                }
                QubeTok::Comma => { self.advance(); }
                _ => { self.advance(); } // skip unexpected
            }
        }
        match self.advance() {
            QubeTok::RBrace => {}
            other => return Err(format!("Expected '}}' to close assert set, got {:?}", other)),
        }
        Ok(QubeStmt::Assert { variable, valid_values: values })
    }

    // ── print <var> ──────────────────────────────────────────────────────────

    fn parse_print(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `print`
        let variable = self.expect_ident()?;
        Ok(QubeStmt::Print { variable })
    }

    // ── let <name> = <expr> ──────────────────────────────────────────────────

    fn parse_let(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `let`
        let name = self.expect_ident()?;
        match self.advance() {
            QubeTok::Equals => {}
            other => return Err(format!("Expected '=' in let, got {:?}", other)),
        }
        let value = match self.peek().clone() {
            QubeTok::Number(n) => {
                self.advance();
                QubeExpr::Number(n)
            }
            QubeTok::Ident(s) => {
                let s = s.clone();
                self.advance();
                QubeExpr::Variable(s)
            }
            other => return Err(format!("Cannot parse let value from {:?}", other)),
        };
        Ok(QubeStmt::LetBinding { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_decl() {
        let mut p = QubeParser::from_str("state ψ = |0⟩");
        let prog = p.parse().unwrap();
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0] {
            QubeStmt::StateDecl { name, .. } => assert_eq!(name, "ψ"),
            _ => panic!("Expected StateDecl"),
        }
    }

    #[test]
    fn test_parse_gate_apply_single() {
        let mut p = QubeParser::from_str("apply H → ψ");
        let prog = p.parse().unwrap();
        match &prog.stmts[0] {
            QubeStmt::GateApply { gate, targets } => {
                assert_eq!(*gate, QuantumGate::H);
                assert_eq!(targets, &vec!["ψ".to_string()]);
            }
            _ => panic!("Expected GateApply"),
        }
    }

    #[test]
    fn test_parse_gate_apply_two_qubit() {
        let mut p = QubeParser::from_str("apply CNOT → (ψ, φ)");
        let prog = p.parse().unwrap();
        match &prog.stmts[0] {
            QubeStmt::GateApply { gate, targets } => {
                assert_eq!(*gate, QuantumGate::CNOT);
                assert_eq!(targets.len(), 2);
            }
            _ => panic!("Expected GateApply"),
        }
    }

    #[test]
    fn test_parse_collapse() {
        let mut p = QubeParser::from_str("collapse ψ → result");
        let prog = p.parse().unwrap();
        match &prog.stmts[0] {
            QubeStmt::Collapse { qubit, result } => {
                assert_eq!(qubit, "ψ");
                assert_eq!(result, "result");
            }
            _ => panic!("Expected Collapse"),
        }
    }

    #[test]
    fn test_parse_assert() {
        let mut p = QubeParser::from_str("assert result ∈ {0, 1}");
        let prog = p.parse().unwrap();
        match &prog.stmts[0] {
            QubeStmt::Assert { variable, valid_values } => {
                assert_eq!(variable, "result");
                assert_eq!(valid_values.len(), 2);
            }
            _ => panic!("Expected Assert"),
        }
    }

    #[test]
    fn test_parse_multi_stmt_program() {
        let src = "state ψ = |0⟩\napply H → ψ\ncollapse ψ → r\nassert r ∈ {0, 1}\nprint r";
        let mut p = QubeParser::from_str(src);
        let prog = p.parse().unwrap();
        assert_eq!(prog.stmts.len(), 5);
    }
}
