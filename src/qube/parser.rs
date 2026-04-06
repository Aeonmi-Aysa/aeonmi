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
                // Original symbolic-syntax keywords
                QubeTok::KwState    => stmts.push(self.parse_state_decl()?),
                QubeTok::KwApply    => stmts.push(self.parse_gate_apply()?),
                QubeTok::KwCollapse => stmts.push(self.parse_collapse()?),
                QubeTok::KwAssert   => stmts.push(self.parse_assert()?),
                QubeTok::KwPrint    => stmts.push(self.parse_print()?),
                QubeTok::KwLet      => stmts.push(self.parse_let()?),
                // Circuit-syntax keywords
                QubeTok::KwCircuit  => stmts.push(self.parse_circuit_def()?),
                QubeTok::KwMeta     => stmts.push(self.parse_meta_block()?),
                QubeTok::KwExecute  => stmts.push(self.parse_execute_block()?),
                QubeTok::KwExpected => stmts.push(self.parse_expected_block()?),
                // Skip unknown tokens at top level (semicolons, etc.)
                QubeTok::Semicolon  => { self.advance(); }
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

    // ──────────────────────────────────────────────────────────────────────────
    // ── Circuit-syntax parsing ────────────────────────────────────────────────
    // ──────────────────────────────────────────────────────────────────────────

    // ── circuit <name> { body } ───────────────────────────────────────────────

    fn parse_circuit_def(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `circuit`
        let name = self.expect_ident()?;
        self.skip_newlines();
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' after circuit name, got {:?}", other)),
        }
        let body = self.parse_circuit_body()?;
        Ok(QubeStmt::CircuitDef { name, body })
    }

    /// Parse statements inside a `circuit { }` block until `}`.
    fn parse_circuit_body(&mut self) -> Result<Vec<CircuitStmt>, String> {
        let mut stmts = Vec::new();
        loop {
            // skip whitespace/newlines
            while matches!(self.peek(), QubeTok::Newline | QubeTok::Semicolon) {
                self.advance();
            }
            match self.peek().clone() {
                QubeTok::RBrace | QubeTok::Eof => {
                    self.advance();
                    break;
                }
                QubeTok::Comment(c) => {
                    let c = c.clone();
                    self.advance();
                    stmts.push(CircuitStmt::Comment(c));
                }
                QubeTok::KwQubit => {
                    self.advance();
                    let name = self.expect_ident()?;
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::QubitDecl(name));
                }
                QubeTok::KwBit => {
                    self.advance();
                    let name = self.expect_ident()?;
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::BitDecl(name));
                }
                QubeTok::KwQreg => {
                    self.advance();
                    let name = self.expect_ident()?;
                    let size = self.parse_bracket_int()?;
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::QregDecl { name, size });
                }
                QubeTok::KwCreg => {
                    self.advance();
                    let name = self.expect_ident()?;
                    let size = self.parse_bracket_int()?;
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::CregDecl { name, size });
                }
                QubeTok::KwMeasure => {
                    stmts.push(self.parse_measure_stmt()?);
                }
                QubeTok::KwIf => {
                    stmts.push(self.parse_if_stmt()?);
                }
                QubeTok::KwReset => {
                    self.advance();
                    let q = self.expect_ident()?;
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::Reset(q));
                }
                QubeTok::KwBarrier => {
                    self.advance();
                    let mut qubits = Vec::new();
                    while matches!(self.peek(), QubeTok::Ident(_)) {
                        qubits.push(self.expect_ident()?);
                    }
                    self.skip_opt_semicolon();
                    stmts.push(CircuitStmt::Barrier(qubits));
                }
                // Gate application or built-in algorithm — starts with an Ident
                QubeTok::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    // Built-in algorithms: grover(n, t), qft(n), shor(n), teleport(...)
                    if matches!(self.peek(), QubeTok::LParen) {
                        let args = self.parse_float_arg_list()?;
                        self.skip_opt_semicolon();
                        stmts.push(CircuitStmt::BuiltinAlgo { name, args });
                    } else {
                        // Gate application: possibly parameterised Rx(theta)
                        let gate = QuantumGate::from_str(&name);
                        // Check for optional rotation param: Rx(1.5708)
                        let param = if matches!(self.peek(), QubeTok::LParen) {
                            Some(self.parse_single_float_arg()?)
                        } else {
                            None
                        };
                        // Read target qubits (one or two identifiers)
                        let mut targets = Vec::new();
                        while matches!(self.peek(), QubeTok::Ident(_)) {
                            targets.push(self.expect_ident()?);
                        }
                        self.skip_opt_semicolon();
                        stmts.push(CircuitStmt::GateApply { gate, param, targets });
                    }
                }
                other => {
                    return Err(format!(
                        "Unexpected token in circuit body: {:?}", other
                    ));
                }
            }
        }
        Ok(stmts)
    }

    /// Parse `measure <qubit> -> <classical>;`
    fn parse_measure_stmt(&mut self) -> Result<CircuitStmt, String> {
        self.advance(); // consume `measure`
        let qubit = self.expect_ident()?;
        // Expect `->`
        match self.advance() {
            QubeTok::DashArrow => {}
            QubeTok::Arrow => {} // accept → too
            other => return Err(format!("Expected '->' in measure, got {:?}", other)),
        }
        let classical = self.expect_ident()?;
        self.skip_opt_semicolon();
        Ok(CircuitStmt::Measure { qubit, classical })
    }

    /// Parse `if <cond> { <body> }`
    fn parse_if_stmt(&mut self) -> Result<CircuitStmt, String> {
        self.advance(); // consume `if`
        let condition = self.expect_ident()?;
        self.skip_newlines();
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' in if, got {:?}", other)),
        }
        let body = self.parse_circuit_body()?;
        Ok(CircuitStmt::IfClassical { condition, body })
    }

    /// Parse `[n]` integer bracket (for qreg/creg).
    fn parse_bracket_int(&mut self) -> Result<usize, String> {
        match self.advance() {
            QubeTok::LBracket => {}
            other => return Err(format!("Expected '[', got {:?}", other)),
        }
        let n = match self.advance() {
            QubeTok::Number(n) => *n as usize,
            other => return Err(format!("Expected integer, got {:?}", other)),
        };
        match self.advance() {
            QubeTok::RBracket => {}
            other => return Err(format!("Expected ']', got {:?}", other)),
        }
        Ok(n)
    }

    /// Parse `(f1, f2, ...)` returning Vec<f64>.
    fn parse_float_arg_list(&mut self) -> Result<Vec<f64>, String> {
        match self.advance() {
            QubeTok::LParen => {}
            other => return Err(format!("Expected '(', got {:?}", other)),
        }
        let mut args = Vec::new();
        while !matches!(self.peek(), QubeTok::RParen | QubeTok::Eof) {
            match self.peek().clone() {
                QubeTok::Number(n) => { args.push(n); self.advance(); }
                QubeTok::Comma => { self.advance(); }
                _ => { self.advance(); } // skip unexpected
            }
        }
        match self.advance() {
            QubeTok::RParen => {}
            other => return Err(format!("Expected ')', got {:?}", other)),
        }
        Ok(args)
    }

    /// Parse `(f)` returning f64 (for single param gates like Rx).
    fn parse_single_float_arg(&mut self) -> Result<f64, String> {
        let args = self.parse_float_arg_list()?;
        Ok(args.into_iter().next().unwrap_or(0.0))
    }

    /// Skip an optional semicolon.
    fn skip_opt_semicolon(&mut self) {
        if matches!(self.peek(), QubeTok::Semicolon) {
            self.advance();
        }
    }

    // ── meta { key: "value", ... } ────────────────────────────────────────────

    fn parse_meta_block(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `meta`
        self.skip_newlines();
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' after meta, got {:?}", other)),
        }
        let entries = self.parse_kv_block()?;
        Ok(QubeStmt::MetaBlock { entries })
    }

    // ── execute { run <name>; ... } ───────────────────────────────────────────

    fn parse_execute_block(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `execute`
        self.skip_newlines();
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' after execute, got {:?}", other)),
        }
        let mut steps = Vec::new();
        loop {
            while matches!(self.peek(), QubeTok::Newline | QubeTok::Semicolon) { self.advance(); }
            match self.peek().clone() {
                QubeTok::RBrace | QubeTok::Eof => { self.advance(); break; }
                QubeTok::Comment(_) => { self.advance(); }
                QubeTok::KwRun => {
                    self.advance(); // consume `run`
                    let name = self.expect_ident()?;
                    self.skip_opt_semicolon();
                    steps.push(name);
                }
                _ => { self.advance(); } // skip unknown tokens inside execute
            }
        }
        Ok(QubeStmt::ExecuteBlock { steps })
    }

    // ── expected { Name: { key: val, ... }, ... } ─────────────────────────────

    fn parse_expected_block(&mut self) -> Result<QubeStmt, String> {
        self.advance(); // consume `expected`
        self.skip_newlines();
        match self.advance() {
            QubeTok::LBrace => {}
            other => return Err(format!("Expected '{{' after expected, got {:?}", other)),
        }
        let mut results = Vec::new();
        loop {
            while matches!(self.peek(), QubeTok::Newline | QubeTok::Semicolon | QubeTok::Comma) {
                self.advance();
            }
            match self.peek().clone() {
                QubeTok::RBrace | QubeTok::Eof => { self.advance(); break; }
                QubeTok::Comment(c) => { let _ = c; self.advance(); }
                QubeTok::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    // Consume `:` then `{`
                    if matches!(self.peek(), QubeTok::Colon) { self.advance(); }
                    self.skip_newlines();
                    if matches!(self.peek(), QubeTok::LBrace) {
                        self.advance();
                        let entries = self.parse_kv_block()?;
                        results.push((name, entries));
                    }
                }
                _ => { self.advance(); }
            }
        }
        Ok(QubeStmt::ExpectedBlock { results })
    }

    /// Parse `key: value, key: value }` key-value pairs until `}`.
    fn parse_kv_block(&mut self) -> Result<Vec<(String, QubeExpr)>, String> {
        let mut entries = Vec::new();
        loop {
            while matches!(self.peek(), QubeTok::Newline | QubeTok::Comma) { self.advance(); }
            match self.peek().clone() {
                QubeTok::RBrace | QubeTok::Eof => { self.advance(); break; }
                QubeTok::Comment(_) => { self.advance(); }
                QubeTok::Ident(k) => {
                    let k = k.clone();
                    self.advance();
                    // consume `:` if present
                    if matches!(self.peek(), QubeTok::Colon) { self.advance(); }
                    // parse value
                    let val = self.parse_kv_value()?;
                    entries.push((k, val));
                }
                _ => { self.advance(); }
            }
        }
        Ok(entries)
    }

    fn parse_kv_value(&mut self) -> Result<QubeExpr, String> {
        match self.peek().clone() {
            QubeTok::Number(n) => {
                let n = n;
                self.advance();
                // Check for array [n, m] after number
                Ok(QubeExpr::Number(n))
            }
            QubeTok::Ident(s) => {
                let s = s.clone();
                self.advance();
                // true / false
                if s == "true" { return Ok(QubeExpr::Bool(true)); }
                if s == "false" { return Ok(QubeExpr::Bool(false)); }
                Ok(QubeExpr::Variable(s))
            }
            QubeTok::LBracket => {
                // Array literal [a, b, ...]
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), QubeTok::RBracket | QubeTok::Eof) {
                    match self.peek().clone() {
                        QubeTok::Number(n) => { elems.push(QubeExpr::Number(n)); self.advance(); }
                        QubeTok::Comma => { self.advance(); }
                        QubeTok::Ident(s) => { elems.push(QubeExpr::Variable(s.clone())); self.advance(); }
                        _ => { self.advance(); }
                    }
                }
                if matches!(self.peek(), QubeTok::RBracket) { self.advance(); }
                Ok(QubeExpr::Array(elems))
            }
            // String literal — look for content between quotes
            // (quoted strings are not tokenized currently, handle Ident fallback)
            _ => {
                // Skip until comma or newline or '}'
                while !matches!(self.peek(),
                    QubeTok::Comma | QubeTok::Newline | QubeTok::RBrace | QubeTok::Eof) {
                    self.advance();
                }
                Ok(QubeExpr::Variable("_".to_string()))
            }
        }
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
