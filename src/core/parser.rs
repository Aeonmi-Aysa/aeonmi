//! Parser for Aeonmi/QUBE/Titan with precedence parsing + spanned errors.

use crate::core::ast::{
    ASTNode, ForInBinding, FunctionParam, QuantumBindingType, QuantumFunctionType,
};
use crate::core::token::{Token, TokenKind};
use unicode_ident::is_xid_continue;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.message, self.line, self.column)
    }
}

impl std::error::Error for ParserError {}

/* ── Parser ───────────────────────────────────────────────────────────── */

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create new parser instance; ensure trailing EOF token present
    pub fn new(mut tokens: Vec<Token>) -> Self {
        let needs_eof = match tokens.last() {
            Some(t) => !matches!(t.kind, TokenKind::EOF),
            None => true,
        };
        if needs_eof {
            tokens.push(Token {
                kind: TokenKind::EOF,
                lexeme: String::new(),
                line: 0,
                column: 0,
            });
        }
        Parser { tokens, pos: 0 }
    }

    /// Parse whole program into an AST program node
    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            nodes.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(nodes))
    }

    /* ── Statements ──────────────────────────────────────────────────── */

    fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        match self.peek().kind.clone() {
            // Module system
            TokenKind::Module => self.parse_module_decl(),
            TokenKind::Import => self.parse_import_decl(),
            TokenKind::Record | TokenKind::Struct => self.parse_record_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Type => self.parse_type_alias(),

            // Classic
            TokenKind::Let => self.parse_variable_decl(),
            TokenKind::Function | TokenKind::Fn => self.parse_function_decl(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Log => self.parse_log(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Match => self.parse_match(),
            TokenKind::OpenBrace => Ok(self.parse_block()?),

            // AEONMI Quantum-Native Syntax
            TokenKind::QuantumBracketOpen => self.parse_quantum_variable_decl(),
            TokenKind::ClassicalFunc => self.parse_quantum_function(QuantumFunctionType::Classical),
            TokenKind::QuantumFunc => self.parse_quantum_function(QuantumFunctionType::Quantum),
            TokenKind::AIFunc => {
                // 🧠 either neural var or AI function
                let next_pos = self.pos + 1;
                if next_pos < self.tokens.len() {
                    match &self.tokens[next_pos].kind {
                        TokenKind::Identifier(s) if s == "neural" => {
                            self.advance(); // 🧠
                            self.advance(); // "neural"
                            let name =
                                self.consume_identifier("Expected variable name after 'neural'")?;
                            self.consume(
                                TokenKind::Equals,
                                "Expected '=' in neural variable declaration",
                            )?;
                            let value = self.parse_expression()?;
                            let _ = self.match_token(&[TokenKind::Semicolon]);
                            Ok(ASTNode::VariableDecl {
                                name,
                                type_annotation: None,
                                value: Box::new(value),
                                line: self.peek().line,
                                column: self.peek().column,
                            })
                        }
                        _ => self.parse_quantum_function(QuantumFunctionType::AINeural),
                    }
                } else {
                    self.parse_quantum_function(QuantumFunctionType::AINeural)
                }
            }

            // Probability/loops/try-time/learn
            TokenKind::QuantumOr => self.parse_probability_branch(), // ⊖
            TokenKind::QuantumLoop => self.parse_quantum_loop(),     // ⟲
            TokenKind::Attempt => self.parse_quantum_try_catch(),    // ⚡
            TokenKind::TimeBlock => self.parse_time_block(),         // ⏰
            TokenKind::Learn => self.parse_ai_learning_block(),

            // Quantum/general ops as standalone statements like: superpose(...);
            TokenKind::Superpose | TokenKind::Entangle | TokenKind::Measure | TokenKind::Dod => {
                self.parse_quantum_op()
            }
            TokenKind::HieroglyphicOp(_) => self.parse_hieroglyphic_op(),

            // Skip comment tokens (already produced by lexer)
            TokenKind::QuantumComment | TokenKind::BecauseComment | TokenKind::NoteComment => {
                self.advance();
                self.parse_statement()
            }

            _ => {
                let expr = self.parse_expression()?;
                let _ = self.match_token(&[TokenKind::Semicolon]); // optional semicolon
                Ok(expr)
            }
        }
    }

    fn parse_block(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::OpenBrace, "Expected '{' to start block")?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after block")?;
        Ok(ASTNode::Block(stmts))
    }

    fn parse_variable_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Let, "Expected 'let'")?;
        let _ = self.match_token(&[TokenKind::Mut]);
        let name = self.consume_identifier("Expected variable name")?;
        let ident_token = self.previous().clone();

        let type_annotation = if self.match_token(&[TokenKind::Colon]) {
            Some(self.parse_type_string()?)
        } else {
            None
        };

        self.consume(TokenKind::Equals, "Expected '=' in variable declaration")?;
        let value = self.parse_expression()?;
        let _ = self.match_token(&[TokenKind::Semicolon]);

        Ok(ASTNode::new_variable_decl_typed(
            &name,
            type_annotation,
            value,
            ident_token.line,
            ident_token.column,
        ))
    }

    fn parse_function_decl(&mut self) -> Result<ASTNode, ParserError> {
        let func_tok = if self.check(&TokenKind::Fn) {
            self.advance()
        } else {
            self.consume(TokenKind::Function, "Expected 'function' or 'fn'")?
        };
        let func_line = func_tok.line;
        let func_col = func_tok.column;

        let name = self.consume_identifier("Expected function name")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let closing = TokenKind::CloseParen;
        let params = self.parse_function_param_list(&closing, func_line, func_col)?;
        self.consume(closing, "Expected ')' after parameters")?;
        let return_type = self.parse_optional_return_type()?;

        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };

        Ok(ASTNode::new_function_at(
            &name,
            func_line,
            func_col,
            params,
            body,
            return_type,
        ))
    }

    fn parse_optional_return_type(&mut self) -> Result<Option<String>, ParserError> {
        if !self.match_token(&[TokenKind::Arrow]) {
            return Ok(None);
        }

        let ty = self.collect_type_string(|kind, depth| {
            depth == 0 && matches!(kind, TokenKind::OpenBrace)
        })?;
        if ty.is_empty() {
            return Err(self.err_here("Expected return type after '->'"));
        }
        Ok(Some(ty))
    }

    fn parse_function_param_list(
        &mut self,
        closing: &TokenKind,
        func_line: usize,
        func_col: usize,
    ) -> Result<Vec<FunctionParam>, ParserError> {
        let mut params = Vec::new();
        let mut saw_variadic = false;

        // Empty parameter list
        if self.check(closing) {
            return Ok(params);
        }

        loop {
            // Trailing comma: ", )" allowed
            if self.check(closing) {
                break;
            }

            let mut is_variadic = false;
            if self.check(&TokenKind::Dot)
                && matches!(self.peek_kind(1), TokenKind::Dot)
                && matches!(self.peek_kind(2), TokenKind::Dot)
            {
                self.advance();
                self.advance();
                self.advance();
                if saw_variadic {
                    return Err(self.err_here("Only one variadic parameter is allowed"));
                }
                is_variadic = true;
                saw_variadic = true;
            }

            let pname = self.consume_identifier("Expected parameter name")?;
            let type_annotation = if self.match_token(&[TokenKind::Colon]) {
                Some(self.parse_type_string()?)
            } else {
                None
            };
            let default = if !is_variadic && self.match_token(&[TokenKind::Equals]) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            params.push(FunctionParam {
                name: pname,
                line: func_line,
                column: func_col,
                type_annotation,
                default,
                is_variadic,
            });

            // Variadic must be last
            if is_variadic {
                if !self.check(closing) {
                    return Err(self.err_here("Variadic parameter must be the final parameter"));
                }
                break;
            }

            if self.match_token(&[TokenKind::Comma]) {
                // Allow trailing comma before ')'
                if self.check(closing) {
                    break;
                }
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_function_expression(&mut self, func_tok: Token) -> Result<ASTNode, ParserError> {
        let line = func_tok.line;
        let column = func_tok.column;

        let name = if let TokenKind::Identifier(n) = self.peek().kind.clone() {
            self.advance();
            Some(n)
        } else {
            None
        };

        self.consume(TokenKind::OpenParen, "Expected '(' after function")?;
        let closing = TokenKind::CloseParen;
        let params = self.parse_function_param_list(&closing, line, column)?;
        self.consume(closing, "Expected ')' after parameters")?;
        let _ = self.parse_optional_return_type()?;

        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };

        Ok(ASTNode::new_function_expr(name, params, body, line, column))
    }

    /* ── Macros ───────────────────────────────────────────────────────── */

    fn parse_macro_invocation(&mut self, callee: ASTNode) -> Result<ASTNode, ParserError> {
        let name = Parser::identifier_name(&callee).ok_or_else(|| ParserError {
            message: "Macro name must be an identifier".into(),
            line: self.peek().line,
            column: self.peek().column,
        })?;

        match name.as_str() {
            "vec" => self.parse_vec_macro(),
            _ => Err(ParserError {
                message: format!("Unsupported macro '{}!'", name),
                line: self.peek().line,
                column: self.peek().column,
            }),
        }
    }

    fn parse_vec_macro(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::OpenBracket, "Expected '[' after vec!'")?;
        if self.match_token(&[TokenKind::CloseBracket]) {
            return Ok(ASTNode::new_array_literal(Vec::new()));
        }

        let mut elements = Vec::new();
        loop {
            let element = self.parse_expression()?;
            if self.match_token(&[TokenKind::Semicolon]) {
                return Err(ParserError {
                    message: "vec![value; count] syntax is not yet supported".into(),
                    line: self.peek().line,
                    column: self.peek().column,
                });
            }
            elements.push(element);
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            if self.check(&TokenKind::CloseBracket) {
                break;
            }
        }

        self.consume(TokenKind::CloseBracket, "Expected ']' after vec! elements")?;
        Ok(ASTNode::new_array_literal(elements))
    }

    fn parse_closure_expression(&mut self, pipe_tok: Token) -> Result<ASTNode, ParserError> {
        let mut params = Vec::new();

        if !self.match_token(&[TokenKind::Pipe]) {
            loop {
                let _ = self.match_token(&[TokenKind::Mut]);
                let param_token = self.advance().clone();
                let name = match param_token.kind {
                    TokenKind::Identifier(n) => n,
                    _ => {
                        return Err(ParserError {
                            message: "Expected parameter name in closure".into(),
                            line: param_token.line,
                            column: param_token.column,
                        })
                    }
                };

                // ✨ NEW: Optional type annotation
                let type_annotation = if self.match_token(&[TokenKind::Colon]) {
                    Some(self.parse_type_string()?)
                } else {
                    None
                };

                params.push(FunctionParam {
                    name,
                    line: param_token.line,
                    column: param_token.column,
                    type_annotation,
                    default: None,
                    is_variadic: false,
                });

                if self.match_token(&[TokenKind::Comma]) {
                    continue;
                }
                self.consume(TokenKind::Pipe, "Expected '|' after closure parameters")?;
                break;
            }
        }

        let body = if self.check(&TokenKind::OpenBrace) {
            match self.parse_block()? {
                ASTNode::Block(stmts) => stmts,
                other => vec![other],
            }
        } else {
            let expr = self.parse_expression()?;
            vec![ASTNode::new_return(expr)]
        };

        Ok(ASTNode::new_function_expr(
            None,
            params,
            body,
            pipe_tok.line,
            pipe_tok.column,
        ))
    }

    fn parse_return(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Return, "Expected 'return'")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after return value")?;
        Ok(ASTNode::new_return(value))
    }

    fn parse_log(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Log, "Expected 'log'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after log")?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after log arg")?;
        self.consume(TokenKind::Semicolon, "Expected ';' after log")?;
        Ok(ASTNode::new_log(expr))
    }

    fn parse_if(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::If, "Expected 'if'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after if")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.match_token(&[TokenKind::Else]) {
            Some(self.parse_statement()?)
        } else {
            None
        };
        Ok(ASTNode::new_if(cond, then_branch, else_branch))
    }

    fn parse_while(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::While, "Expected 'while'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after while")?;
        let cond = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        let body = self.parse_statement()?;
        Ok(ASTNode::new_while(cond, body))
    }

    fn parse_for(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::For, "Expected 'for'")?;

        if self.looks_like_c_style_for() {
            self.consume(TokenKind::OpenParen, "Expected '(' after for")?;
            let init = if !self.check(&TokenKind::Semicolon) {
                Some(self.parse_statement()?)
            } else {
                self.advance(); // ';'
                None
            };
            let condition = if !self.check(&TokenKind::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(TokenKind::Semicolon, "Expected ';' after loop condition")?;
            let increment = if !self.check(&TokenKind::CloseParen) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(TokenKind::CloseParen, "Expected ')' after for clauses")?;
            let body = self.parse_statement()?;
            Ok(ASTNode::new_for(init, condition, increment, body))
        } else {
            let binding = self.parse_for_in_binding()?;
            self.consume(TokenKind::In, "Expected 'in' in for-in loop")?;
            let iterable = self.parse_expression()?;
            let body = self.parse_statement()?;
            Ok(ASTNode::new_for_in(binding, iterable, body))
        }
    }

    fn looks_like_c_style_for(&self) -> bool {
        if !self.check(&TokenKind::OpenParen) {
            return false;
        }
        let mut depth = 0usize;
        let mut offset = 0usize;
        loop {
            let kind = self.peek_kind(offset);
            match kind {
                TokenKind::OpenParen => depth += 1,
                TokenKind::CloseParen => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Semicolon if depth == 1 => return true,
                TokenKind::In if depth == 1 => return false,
                TokenKind::EOF => break,
                _ => {}
            }
            offset += 1;
        }
        false
    }

    fn parse_for_in_binding(&mut self) -> Result<ForInBinding, ParserError> {
        let parenthesized = self.match_token(&[TokenKind::OpenParen]);
        let is_mutable = self.match_token(&[TokenKind::Mut]);

        let binding_token = self.advance().clone();
        let name = match binding_token.kind.clone() {
            TokenKind::Identifier(n) => n,
            _ => {
                return Err(ParserError {
                    message: "Expected loop binding identifier".into(),
                    line: binding_token.line,
                    column: binding_token.column,
                })
            }
        };

        let type_annotation = self.parse_optional_for_binding_type(parenthesized)?;

        if parenthesized {
            self.consume(TokenKind::CloseParen, "Expected ')' after for binding")?;
        }

        Ok(ForInBinding {
            name,
            is_mutable,
            type_annotation,
            line: binding_token.line,
            column: binding_token.column,
        })
    }

    fn parse_optional_for_binding_type(
        &mut self,
        parenthesized: bool,
    ) -> Result<Option<String>, ParserError> {
        if !self.match_token(&[TokenKind::Colon]) {
            return Ok(None);
        }

        let ty = self.collect_type_string(|kind, depth| {
            if depth > 0 {
                return false;
            }
            matches!(kind, TokenKind::In)
                || (parenthesized && matches!(kind, TokenKind::CloseParen))
        })?;

        if ty.is_empty() {
            return Err(self.err_here("Expected type annotation after ':'"));
        }

        Ok(Some(ty))
    }

    /* ── Quantum / Hieroglyphic ops ─────────────────────────────────── */

    fn parse_quantum_op(&mut self) -> Result<ASTNode, ParserError> {
        let op = self.advance().kind.clone();
        let mut qubits = Vec::new();

        if self.match_token(&[TokenKind::OpenParen]) {
            while !self.check(&TokenKind::CloseParen) {
                qubits.push(self.parse_expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume(TokenKind::CloseParen, "Expected ')' after qubits")?;
        }

        self.consume(TokenKind::Semicolon, "Expected ';' after quantum op")?;
        Ok(ASTNode::new_quantum_op(op, qubits))
    }

    fn parse_hieroglyphic_op(&mut self) -> Result<ASTNode, ParserError> {
        let symbol_token = self.advance().clone();
        let symbol = match symbol_token.kind.clone() {
            TokenKind::HieroglyphicOp(sym) => sym,
            _ => return Err(self.err_here("Expected hieroglyphic symbol")),
        };

        // If this glyph begins a quantum var decl, parse: 𓀀 name = value;
        let treat_as_quantum_variable = is_quantum_variable_symbol(&symbol)
            && matches!(self.peek().kind.clone(), TokenKind::Identifier(_));

        if treat_as_quantum_variable {
            let line = symbol_token.line;
            let column = symbol_token.column;
            let name = self.consume_identifier("Expected variable name after quantum symbol")?;
            self.consume(
                TokenKind::Equals,
                "Expected '=' in quantum variable declaration",
            )?;
            let value = self.parse_expression()?;
            self.consume(
                TokenKind::Semicolon,
                "Expected ';' after quantum variable declaration",
            )?;

            Ok(ASTNode::new_quantum_variable_decl_from_symbol(
                &name, value, &symbol, line, column,
            ))
        } else {
            // Regular hieroglyphic operation: glyph(args?);
            let mut args = Vec::new();
            if self.match_token(&[TokenKind::OpenParen]) {
                while !self.check(&TokenKind::CloseParen) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after args")?;
            }
            self.consume(TokenKind::Semicolon, "Expected ';' after hieroglyphic op")?;
            Ok(ASTNode::new_hieroglyphic_op(&symbol, args))
        }
    }

    /* ── Expressions (precedence) ────────────────────────────────────── */

    pub fn parse_expression(&mut self) -> Result<ASTNode, ParserError> {
        self.parse_logical_or()
    }

    // or: and ('||' and)*
    fn parse_logical_or(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_logical_and()?;
        while self.match_token(&[TokenKind::OrOr]) {
            let op = self.previous().kind.clone();
            let right = self.parse_logical_and()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    // and: assignment ('&&' assignment)*
    fn parse_logical_and(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_assignment()?;
        while self.match_token(&[TokenKind::AndAnd]) {
            let op = self.previous().kind.clone();
            let right = self.parse_assignment()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    // assignment: Identifier '=' assignment | equality
    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        let expr = self.parse_equality()?;
        if self.match_token(&[TokenKind::Equals]) {
            match expr {
                ASTNode::Identifier(name) => {
                    let line = self.previous().line;
                    let column = self.previous().column;
                    let value = self.parse_assignment()?;
                    Ok(ASTNode::new_assignment_at(&name, value, line, column))
                }
                ASTNode::IdentifierSpanned {
                    name,
                    line: id_line,
                    column: id_col,
                    ..
                } => {
                    let value = self.parse_assignment()?;
                    Ok(ASTNode::new_assignment_at(&name, value, id_line, id_col))
                }
                _ => Err(self.err_here("Invalid assignment target")),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_equality(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_comparison()?;
        while self.match_token(&[TokenKind::DoubleEquals, TokenKind::NotEquals]) {
            let op = self.previous().kind.clone();
            let right = self.parse_comparison()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_quantum_ops()?;
        while self.match_token(&[
            TokenKind::LessThan,
            TokenKind::LessEqual,
            TokenKind::GreaterThan,
            TokenKind::GreaterEqual,
            TokenKind::QuantumGeq,
            TokenKind::QuantumLeq,
        ]) {
            let op = self.previous().kind.clone();
            let right = self.parse_quantum_ops()?;
            expr = match op {
                TokenKind::QuantumGeq | TokenKind::QuantumLeq => {
                    ASTNode::new_quantum_binary_expr(op, expr, right)
                }
                _ => ASTNode::new_binary_expr(op, expr, right),
            };
        }
        Ok(expr)
    }

    // quantum ops precedence between comparison and additive
    fn parse_quantum_ops(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_term()?;
        while self.match_token(&[
            TokenKind::QuantumTensor,   // ⊗
            TokenKind::QuantumXor,      // ⊕
            TokenKind::QuantumModulo,   // ◊
            TokenKind::QuantumGradient, // ∇
            TokenKind::QuantumApprox,   // ≈
        ]) {
            let op = self.previous().kind.clone();
            let right = self.parse_term()?;
            expr = ASTNode::new_quantum_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_factor()?;
        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = self.previous().kind.clone();
            let right = self.parse_factor()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_unary()?;
        while self.match_token(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, ParserError> {
        if self.match_token(&[TokenKind::Minus, TokenKind::Plus]) {
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            return Ok(ASTNode::new_unary_expr(op, right));
        }
        if self.match_token(&[TokenKind::Ampersand]) {
            let _ = self.match_token(&[TokenKind::Mut]); // ignore &mut
            return self.parse_unary();
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&[TokenKind::OpenParen]) {
                // Function/method call
                let (args, _named_args) = self.parse_call_arguments()?;
                self.consume(TokenKind::CloseParen, "Expected ')' after arguments")?;

                // If we have named args, we need to store them somehow
                // For now, we'll use regular Call and you can extend it later
                expr = ASTNode::new_call(expr, args);
                // TODO: Store named_args when needed
            } else if self.check(&TokenKind::Dot) {
                // Method access: obj.method or obj.method()
                if let TokenKind::Identifier(_) = self.peek_kind(1) {
                    self.advance(); // '.'
                    let method = self.consume_identifier("Expected method name after '.'")?;

                    // Check if this is a method call with ()
                    if self.check(&TokenKind::OpenParen) {
                        self.advance(); // '('
                        let (args, named_args) = self.parse_call_arguments()?;
                        self.consume(TokenKind::CloseParen, "Expected ')' after method arguments")?;

                        // Create MethodCall node
                        expr = ASTNode::new_method_call(expr, &method, args, named_args);
                    } else {
                        // Field access without call
                        expr = ASTNode::new_field_access(expr, &method);
                    }
                } else {
                    break;
                }
            } else if self.check(&TokenKind::DoubleColon) {
                // Keep existing :: handling
                match expr {
                    ASTNode::Identifier(mut name) => {
                        while self.match_token(&[TokenKind::DoubleColon]) {
                            let component =
                                self.consume_identifier("Expected identifier after '::'")?;
                            name.push_str("::");
                            name.push_str(&component);
                        }
                        expr = ASTNode::Identifier(name);
                    }
                    ASTNode::IdentifierSpanned { name, .. } => {
                        let mut full_name = name.clone();
                        while self.match_token(&[TokenKind::DoubleColon]) {
                            let component =
                                self.consume_identifier("Expected identifier after '::'")?;
                            full_name.push_str("::");
                            full_name.push_str(&component);
                        }
                        expr = ASTNode::Identifier(full_name);
                    }
                    _ => return Err(self.err_here("Module path must start with identifier")),
                }
            } else if self.match_token(&[TokenKind::As]) {
                // Type casting: expr as Type
                let target_type = self.parse_type_string()?;
                expr = ASTNode::new_type_cast(expr, &target_type);
            } else if self.check(&TokenKind::OpenBrace) {
                // Keep existing struct literal handling
                if let Some(type_name) = Parser::identifier_name(&expr) {
                    let after_brace = self.peek_kind(1).clone();
                    let is_struct_like = matches!(after_brace, TokenKind::CloseBrace)
                        || (matches!(
                            after_brace,
                            TokenKind::Identifier(_) | TokenKind::StringLiteral(_)
                        ) && matches!(self.peek_kind(2), TokenKind::Colon));

                    if is_struct_like {
                        self.advance(); // '{'
                        let fields = self.parse_struct_literal_fields()?;
                        expr = ASTNode::new_struct_literal(type_name, fields);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else if self.match_token(&[TokenKind::Exclamation]) {
                expr = self.parse_macro_invocation(expr)?;
            } else if self.match_token(&[TokenKind::OpenBracket]) {
                let index_expr = self.parse_expression()?;
                self.consume(TokenKind::CloseBracket, "Expected ']' after index")?;
                expr = ASTNode::new_index_expr(expr, index_expr);
            } else if self.match_token(&[TokenKind::QuantumIndexOpen]) {
                let index_expr = self.parse_expression()?;
                self.consume(
                    TokenKind::QuantumIndexClose,
                    "Expected '⟧' after quantum index",
                )?;
                expr = ASTNode::new_quantum_index_access(expr, index_expr, true);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_call_arguments(
        &mut self,
    ) -> Result<(Vec<ASTNode>, Vec<(String, ASTNode)>), ParserError> {
        let mut positional = Vec::new();
        let mut named = Vec::new();
        let mut seen_named = false;

        if self.check(&TokenKind::CloseParen) {
            return Ok((positional, named));
        }

        loop {
            if let TokenKind::Identifier(name) = self.peek().kind.clone() {
                if matches!(self.peek_kind(1), TokenKind::Equals) {
                    seen_named = true;
                    self.advance(); // identifier
                    self.advance(); // '='
                    let value = self.parse_expression()?;
                    named.push((name, value));
                } else {
                    if seen_named {
                        return Err(
                            self.err_here("Positional arguments must come before named arguments")
                        );
                    }
                    positional.push(self.parse_expression()?);
                }
            } else {
                if seen_named {
                    return Err(
                        self.err_here("Positional arguments must come before named arguments")
                    );
                }
                positional.push(self.parse_expression()?);
            }

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            if self.check(&TokenKind::CloseParen) {
                break;
            }
        }

        Ok((positional, named))
    }

    fn parse_type_string(&mut self) -> Result<String, ParserError> {
        let mut type_str = String::new();

        // Handle references: &T or &mut T
        if self.match_token(&[TokenKind::Ampersand]) {
            type_str.push('&');
            if self.match_token(&[TokenKind::Mut]) {
                type_str.push_str("mut ");
            }
        }

        // Base type name (could be path like core::App)
        let mut base = self.consume_identifier("Expected type name")?;

        // Handle type paths: core::App::Builder
        while self.match_token(&[TokenKind::DoubleColon]) {
            base.push_str("::");
            base.push_str(&self.consume_identifier("Expected type component")?);
        }

        type_str.push_str(&base);

        // Handle generic parameters: Vec<T>, Option<String>
        if self.match_token(&[TokenKind::LessThan]) {
            type_str.push('<');
            let mut _depth = 1;

            loop {
                let generic_arg = self.parse_type_string()?;
                type_str.push_str(&generic_arg);

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                type_str.push_str(", ");
            }

            self.consume(
                TokenKind::GreaterThan,
                "Expected '>' after generic arguments",
            )?;
            type_str.push('>');
        }

        Ok(type_str)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::NumberLiteral(int_part) => {
                // Fuse split decimals: Number '.' Number => float literal
                if self.check(&TokenKind::Dot)
                    && matches!(self.peek_kind(1), TokenKind::NumberLiteral(_))
                {
                    self.advance(); // '.'
                    let frac_tok = self.advance().clone(); // NumberLiteral(_)

                    let mut left_str = if !tok.lexeme.is_empty() {
                        tok.lexeme.clone()
                    } else {
                        format!("{}", int_part)
                    };
                    left_str = left_str.replace('_', "").trim_end_matches('.').to_string();

                    let mut right_str = if !frac_tok.lexeme.is_empty() {
                        frac_tok.lexeme.clone()
                    } else {
                        match frac_tok.kind {
                            TokenKind::NumberLiteral(n) => format!("{}", n.trunc() as i64),
                            _ => String::new(),
                        }
                    };
                    right_str = right_str
                        .replace('_', "")
                        .trim_start_matches('0')
                        .to_string();
                    if right_str.is_empty() {
                        right_str = "0".to_string();
                    }

                    let combined = format!("{}.{}", left_str, right_str);
                    let value: f64 = combined.parse().map_err(|_| ParserError {
                        message: format!("Invalid float literal '{}'", combined),
                        line: tok.line,
                        column: tok.column,
                    })?;
                    Ok(ASTNode::NumberLiteral(value))
                } else {
                    Ok(ASTNode::NumberLiteral(int_part))
                }
            }

            TokenKind::StringLiteral(s) => Ok(ASTNode::StringLiteral(s)),
            TokenKind::BooleanLiteral(b) => Ok(ASTNode::BooleanLiteral(b)),

            TokenKind::Identifier(name) => Ok(ASTNode::new_identifier_spanned(
                &name,
                tok.line,
                tok.column,
                name.len(),
            )),

            // 'function' / 'fn' as an expression
            TokenKind::Function | TokenKind::Fn => self.parse_function_expression(tok),

            // Quantum keywords as identifiers (so they can be callee names)
            TokenKind::Superpose | TokenKind::Entangle | TokenKind::Measure | TokenKind::Dod => {
                let name = tok.lexeme.clone();
                Ok(ASTNode::new_identifier_spanned(
                    &name,
                    tok.line,
                    tok.column,
                    name.len(),
                ))
            }

            // Qubit state literal: |ψ⟩, |+⟩, etc.
            TokenKind::QubitLiteral(literal) => {
                let normalized = Parser::normalize_qubit_literal(&literal);
                if !Parser::is_valid_qubit_literal(&normalized) {
                    return Err(ParserError {
                        message: format!("Invalid qubit literal '{}'", literal),
                        line: tok.line,
                        column: tok.column,
                    });
                }
                Ok(ASTNode::new_quantum_state(&normalized, None))
            }

            // Parenthesized
            TokenKind::OpenParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::CloseParen, "Expected ')'")?;
                Ok(expr)
            }

            // Closure: |params| body
            TokenKind::Pipe => self.parse_closure_expression(tok),

            // Array literal: [ ... ]
            TokenKind::OpenBracket => {
                let mut elements = Vec::new();
                if !self.check(&TokenKind::CloseBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::CloseBracket, "Expected ']' after array elements")?;
                Ok(ASTNode::new_array_literal(elements))
            }

            // Brace literal: object {k:v,...} or quantum array { ... }
            TokenKind::OpenBrace => {
                let is_object_literal = if self.check(&TokenKind::CloseBrace) {
                    true
                } else {
                    match self.peek().kind {
                        TokenKind::Identifier(_) | TokenKind::StringLiteral(_) => {
                            matches!(self.peek_kind(1), TokenKind::Colon)
                        }
                        _ => false,
                    }
                };

                if is_object_literal {
                    let fields = self.parse_object_literal_fields()?;
                    Ok(ASTNode::new_object_literal(fields))
                } else {
                    let mut elements = Vec::new();
                    if !self.check(&TokenKind::CloseBrace) {
                        loop {
                            elements.push(self.parse_expression()?);
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::CloseBrace, "Expected '}' after array elements")?;
                    let is_superposition = elements
                        .iter()
                        .any(|e| matches!(e, ASTNode::QuantumState { .. }));
                    Ok(ASTNode::new_quantum_array(elements, is_superposition))
                }
            }

            // Quantum var access: ⟨name⟩ or ⟨name⟩⟦i⟧
            TokenKind::QuantumBracketOpen => {
                let name = self.consume_identifier("Expected variable name after '⟨'")?;
                self.consume(
                    TokenKind::QuantumBracketClose,
                    "Expected '⟩' after variable name",
                )?;
                if self.check(&TokenKind::QuantumIndexOpen) {
                    self.advance(); // ⟦
                    let index = self.parse_expression()?;
                    self.consume(TokenKind::QuantumIndexClose, "Expected '⟧' after index")?;
                    Ok(ASTNode::new_quantum_index_access(
                        ASTNode::Identifier(name),
                        index,
                        true,
                    ))
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            }

            _ => Err(ParserError {
                message: format!("Unexpected token {:?}", tok.kind),
                line: tok.line,
                column: tok.column,
            }),
        }
    }

    #[allow(dead_code)]
    fn parse_quantum_state(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Pipe, "Expected '|'")?;

        let state_content = if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            name
        } else if let TokenKind::NumberLiteral(num) = self.peek().kind.clone() {
            self.advance();
            num.to_string()
        } else {
            self.advance().lexeme.clone()
        };

        self.consume(TokenKind::GreaterThan, "Expected '⟩' after quantum state")?;

        let amplitude = if self.match_token(&[TokenKind::Star]) {
            if let ASTNode::NumberLiteral(amp) = self.parse_expression()? {
                Some(amp)
            } else {
                return Err(self.err_here("Expected amplitude value after '*'"));
            }
        } else {
            None
        };

        Ok(ASTNode::new_quantum_state(
            &format!("|{}⟩", state_content),
            amplitude,
        ))
    }

    /* ── Token utils ─────────────────────────────────────────────────── */

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }
    fn previous(&self) -> &Token {
        if self.pos == 0 {
            &self.tokens[0]
        } else {
            &self.tokens[self.pos - 1]
        }
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }
    fn peek_offset(&self, offset: usize) -> &Token {
        let idx = (self.pos + offset).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }
    fn peek_kind(&self, offset: usize) -> &TokenKind {
        &self.peek_offset(offset).kind
    }
    fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && &self.peek().kind == kind
    }
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<&Token, ParserError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(self.err_at(
                &format!("{} (found {:?})", msg, self.peek().kind),
                self.peek().line,
                self.peek().column,
            ))
        }
    }
    fn consume_identifier(&mut self, msg: &str) -> Result<String, ParserError> {
        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.err_at(
                &format!("{} (found {:?})", msg, self.peek().kind),
                self.peek().line,
                self.peek().column,
            ))
        }
    }
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }
    fn err_here(&self, msg: &str) -> ParserError {
        self.err_at(msg, self.peek().line, self.peek().column)
    }
    fn err_at(&self, msg: &str, line: usize, column: usize) -> ParserError {
        ParserError {
            message: msg.into(),
            line,
            column,
        }
    }

    fn parse_param_type_annotation(
        &mut self,
        closing: &TokenKind,
    ) -> Result<Option<String>, ParserError> {
        if !self.match_token(&[TokenKind::Colon]) {
            return Ok(None);
        }

        let ty = self.collect_type_annotation(closing)?;
        let ty_trimmed = ty.trim().to_string();
        if ty_trimmed.is_empty() {
            return Err(self.err_here("Expected type annotation after ':'"));
        }
        Ok(Some(ty_trimmed))
    }

    fn collect_type_annotation(&mut self, closing: &TokenKind) -> Result<String, ParserError> {
        let mut parts = Vec::new();
        let mut depth = 0usize;

        while !self.is_at_end() {
            let kind = self.peek().kind.clone();
            if depth == 0
                && (matches!(kind, TokenKind::Comma | TokenKind::Equals) || kind == *closing)
            {
                break;
            }

            match kind {
                TokenKind::LessThan | TokenKind::OpenParen | TokenKind::OpenBracket => {
                    depth = depth.saturating_add(1);
                }
                TokenKind::GreaterThan | TokenKind::CloseParen | TokenKind::CloseBracket => {
                    if depth > 0 {
                        depth -= 1;
                    } else if kind == *closing {
                        break;
                    }
                }
                _ => {}
            }

            let token = self.advance().clone();
            parts.push(Self::token_fragment(&token));
        }

        Ok(parts.join(""))
    }

    fn token_fragment(token: &Token) -> String {
        if !token.lexeme.is_empty() {
            return token.lexeme.clone();
        }

        match &token.kind {
            TokenKind::Identifier(name) => name.clone(),
            TokenKind::NumberLiteral(value) => value.to_string(),
            TokenKind::StringLiteral(value) => format!("\"{}\"", value),
            TokenKind::BooleanLiteral(value) => value.to_string(),
            TokenKind::LessThan => "<".into(),
            TokenKind::GreaterThan => ">".into(),
            TokenKind::OpenParen => "(".into(),
            TokenKind::CloseParen => ")".into(),
            TokenKind::OpenBracket => "[".into(),
            TokenKind::CloseBracket => "]".into(),
            TokenKind::Dot => ".".into(),
            TokenKind::Comma => ",".into(),
            TokenKind::Star => "*".into(),
            TokenKind::Percent => "%".into(),
            TokenKind::Plus => "+".into(),
            TokenKind::Minus => "-".into(),
            TokenKind::Slash => "/".into(),
            TokenKind::Colon => ":".into(),
            other => other.to_string(),
        }
    }

    fn normalize_qubit_literal(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('⟩') {
            return trimmed.to_string();
        }
        if trimmed.starts_with('|') && trimmed.ends_with('>') {
            let inner = trimmed.trim_start_matches('|').trim_end_matches('>').trim();
            return format!("|{}⟩", inner);
        }
        if trimmed.ends_with('⟩') {
            let inner = trimmed.trim_end_matches('⟩').trim();
            return format!("|{}⟩", inner);
        }
        trimmed.to_string()
    }

    fn is_valid_qubit_literal(normalized: &str) -> bool {
        let mut chars = normalized.chars();
        if chars.next() != Some('|') {
            return false;
        }
        if chars.next_back() != Some('⟩') {
            return false;
        }

        let mut saw_non_whitespace = false;
        for ch in chars {
            if !Parser::is_valid_qubit_char(ch) && !ch.is_whitespace() {
                return false;
            }
            if !ch.is_whitespace() {
                saw_non_whitespace = true;
            }
        }
        saw_non_whitespace
    }

    fn is_valid_qubit_char(ch: char) -> bool {
        matches!(ch, '+' | '-')
            || ch.is_ascii_digit()
            || is_xid_continue(ch)
            || Parser::is_numeric_glyph(ch)
    }

    fn is_numeric_glyph(ch: char) -> bool {
        (0x1D360..=0x1D369).contains(&(ch as u32))
    }

    /* ── Quantum‑native parsing ──────────────────────────────────────── */

    fn parse_quantum_variable_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumBracketOpen, "Expected '⟨'")?;
        let line = self.peek().line;
        let column = self.peek().column;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(TokenKind::QuantumBracketClose, "Expected '⟩'")?;

        // Determine binding type based on operator
        let binding_type = match self.peek().kind {
            TokenKind::QuantumBind => QuantumBindingType::Classical, // ←
            TokenKind::QuantumIn => QuantumBindingType::Superposition, // ∈
            TokenKind::QuantumTensor => QuantumBindingType::Tensor,  // ⊗
            TokenKind::QuantumApprox => QuantumBindingType::Approximation, // ≈
            _ => return Err(self.err_here("Expected quantum binding operator (←, ∈, ⊗, ≈)")),
        };

        self.advance(); // consume the binding operator
        let mut value = self.parse_expression()?;

        if matches!(binding_type, QuantumBindingType::Tensor) {
            if let ASTNode::ArrayLiteral(elements) = value {
                let is_superposition = elements
                    .iter()
                    .any(|elem| matches!(elem, ASTNode::QuantumState { .. }));
                value = ASTNode::new_quantum_array(elements, is_superposition);
            }
        }

        let _ = self.match_token(&[TokenKind::Semicolon]);

        Ok(ASTNode::new_quantum_variable_decl(
            &name,
            binding_type,
            value,
            line,
            column,
        ))
    }

    fn parse_quantum_function(
        &mut self,
        func_type: QuantumFunctionType,
    ) -> Result<ASTNode, ParserError> {
        let func_tok = self.advance(); // ◯ / ⊙ / 🧠
        let func_line = func_tok.line;
        let func_col = func_tok.column;

        let name = self.consume_identifier("Expected function name")?;

        // Params: ⟨ ... ⟩
        self.consume(
            TokenKind::QuantumBracketOpen,
            "Expected '⟨' before parameters",
        )?;
        let closing = TokenKind::QuantumBracketClose;
        let params = self.parse_function_param_list(&closing, func_line, func_col)?;
        self.consume(closing, "Expected '⟩' after parameters")?;

        // Optional return type like → T  (skipped; parse but ignore)
        if self.match_token(&[TokenKind::QuantumImplies]) {
            let _ = self.parse_expression()?;
        }

        // Body
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };

        Ok(ASTNode::new_quantum_function(
            func_type, &name, params, body, func_line, func_col,
        ))
    }

    fn parse_probability_branch(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumOr, "Expected '⊖'")?;
        let mut condition = self.parse_expression()?;

        // Optional explicit probability ≈ 0.8
        let mut probability = if self.match_token(&[TokenKind::QuantumApprox]) {
            if let ASTNode::NumberLiteral(p) = self.parse_expression()? {
                Some(p)
            } else {
                return Err(self.err_here("Expected probability value after '≈'"));
            }
        } else {
            None
        };

        // Or condition already written as (cond ≈ 0.5)
        if probability.is_none() {
            if let ASTNode::QuantumBinaryExpr { op, left, right } = condition.clone() {
                if matches!(op, TokenKind::QuantumApprox) {
                    match *right {
                        ASTNode::NumberLiteral(p) => {
                            probability = Some(p);
                            condition = *left;
                        }
                        _ => return Err(self.err_here("Probability annotation must be numeric")),
                    }
                }
            }
        }

        self.consume(TokenKind::QuantumImplies, "Expected '⇒' after condition")?;
        let then_branch = self.parse_statement()?;

        let else_branch = if self.match_token(&[TokenKind::QuantumXor]) {
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(ASTNode::new_probability_branch(
            condition,
            probability,
            then_branch,
            else_branch,
        ))
    }

    fn parse_quantum_loop(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumLoop, "Expected '⟲'")?;
        let condition = self.parse_expression()?;

        // Optional decoherence threshold: ⪰ <num>
        let decoherence_threshold = if self.match_token(&[TokenKind::QuantumGeq]) {
            if let ASTNode::NumberLiteral(threshold) = self.parse_expression()? {
                Some(threshold)
            } else {
                return Err(self.err_here("Expected threshold value after '⪰'"));
            }
        } else {
            None
        };

        self.consume(TokenKind::QuantumImplies, "Expected '⇒' after condition")?;
        let body = self.parse_statement()?;

        Ok(ASTNode::QuantumLoop {
            condition: Box::new(condition),
            body: Box::new(body),
            decoherence_threshold,
        })
    }

    fn parse_quantum_try_catch(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Attempt, "Expected '⚡'")?;

        let attempt_body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            single => vec![single],
        };

        let mut error_probability = None;
        let mut catch_body = None;
        let mut success_body = None;

        // ⚠ (optional) catch, with optional ≈ p
        if self.match_token(&[TokenKind::Warning]) {
            if self.match_token(&[TokenKind::QuantumApprox]) {
                if let ASTNode::NumberLiteral(p) = self.parse_expression()? {
                    error_probability = Some(p);
                } else {
                    return Err(self.err_here("Expected error probability after '≈'"));
                }
            }
            self.consume(
                TokenKind::QuantumImplies,
                "Expected '⇒' after error condition",
            )?;
            catch_body = Some(match self.parse_block()? {
                ASTNode::Block(stmts) => stmts,
                single => vec![single],
            });
        }

        // ✓ (optional) success
        if self.match_token(&[TokenKind::Success]) {
            success_body = Some(match self.parse_block()? {
                ASTNode::Block(stmts) => stmts,
                single => vec![single],
            });
        }

        Ok(ASTNode::QuantumTryCatch {
            attempt_body,
            error_probability,
            catch_body,
            success_body,
        })
    }

    fn parse_time_block(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::TimeBlock, "Expected time block marker")?;
        let duration = if !self.check(&TokenKind::QuantumImplies) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.consume(TokenKind::QuantumImplies, "Expected '⇒' after duration")?;

        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            single => vec![single],
        };

        Ok(ASTNode::TimeBlock { duration, body })
    }

    fn parse_ai_learning_block(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Learn, "Expected 'learn'")?;
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            single => vec![single],
        };
        Ok(ASTNode::AILearningBlock {
            data_binding: None,
            model_binding: None,
            body,
        })
    }

    /* ── Modules / Types ─────────────────────────────────────────────── */

    fn parse_module_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Module, "Expected 'module'")?;

        // Path: Nebula::Threadsmiths
        let mut path = Vec::new();
        path.push(self.consume_identifier("Expected module name")?);
        while self.match_token(&[TokenKind::DoubleColon]) {
            path.push(self.consume_identifier("Expected module name component")?);
        }

        let body = if self.check(&TokenKind::OpenBrace) {
            self.advance(); // '{'
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                stmts.push(self.parse_statement()?);
            }
            self.consume(TokenKind::CloseBrace, "Expected '}' after module body")?;
            stmts
        } else {
            // Implicit module body: rest of file
            let mut stmts = Vec::new();
            while !self.is_at_end() {
                stmts.push(self.parse_statement()?);
            }
            stmts
        };

        Ok(ASTNode::Module { path, body })
    }

    fn parse_import_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Import, "Expected 'import'")?;

        let mut path = Vec::new();
        path.push(self.consume_identifier("Expected module name")?);
        while self.match_token(&[TokenKind::DoubleColon]) {
            path.push(self.consume_identifier("Expected module name component")?);
        }

        // TODO: support import foo::bar::{item1, item2}
        let items = None;

        let _ = self.match_token(&[TokenKind::Semicolon]);
        Ok(ASTNode::Import { path, items })
    }

    fn parse_record_decl(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.advance(); // 'record' or 'struct'
        let line = tok.line;
        let column = tok.column;

        let name = self.consume_identifier("Expected record name")?;
        self.consume(TokenKind::OpenBrace, "Expected '{' after record name")?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            // Support quantum-prefixed field names like ⟨ψ⟩strand
            let field_name = if self.check(&TokenKind::QuantumBracketOpen) {
                self.advance(); // ⟨
                let mut quantum_name = String::from("⟨");
                let inner_token = self.advance().clone();
                quantum_name.push_str(&inner_token.lexeme);
                self.consume(
                    TokenKind::QuantumBracketClose,
                    "Expected '⟩' after quantum identifier",
                )?;
                quantum_name.push('⟩');
                quantum_name.push_str(
                    &self.consume_identifier("Expected field name after quantum prefix")?,
                );
                quantum_name
            } else {
                self.consume_identifier("Expected field name")?
            };

            let type_annotation = if self.match_token(&[TokenKind::Colon]) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };

            fields.push((field_name, type_annotation));
            let _ = self.match_token(&[TokenKind::Comma]); // optional separators
        }

        self.consume(TokenKind::CloseBrace, "Expected '}' after record fields")?;
        Ok(ASTNode::RecordDecl {
            name,
            fields,
            line,
            column,
        })
    }

    fn parse_enum_decl(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.consume(TokenKind::Enum, "Expected 'enum'")?;
        let line = tok.line;
        let column = tok.column;

        let name = self.consume_identifier("Expected enum name")?;
        self.consume(TokenKind::OpenBrace, "Expected '{' after enum name")?;

        let mut variants = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            variants.push(self.consume_identifier("Expected variant name")?);
            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.consume(TokenKind::CloseBrace, "Expected '}' after enum variants")?;
        Ok(ASTNode::EnumDecl {
            name,
            variants,
            line,
            column,
        })
    }

    fn parse_type_alias(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.consume(TokenKind::Type, "Expected 'type'")?;
        let line = tok.line;
        let column = tok.column;

        let name = self.consume_identifier("Expected type name")?;
        self.consume(TokenKind::Equals, "Expected '=' in type alias")?;
        let target = Box::new(self.parse_type_annotation()?);
        let _ = self.match_token(&[TokenKind::Semicolon]);

        Ok(ASTNode::TypeAlias {
            name,
            target,
            line,
            column,
        })
    }

    fn parse_type_annotation(&mut self) -> Result<ASTNode, ParserError> {
        // Simplified types: uuid, f32, Vec<Thread>, Option<uuid>, strings::Strand
        let mut name = self.consume_identifier("Expected type name")?;

        while self.match_token(&[TokenKind::DoubleColon]) {
            name.push_str("::");
            name.push_str(&self.consume_identifier("Expected type component after '::'")?);
        }

        // Generics: Vec<T, U>
        if self.match_token(&[TokenKind::LessThan]) {
            let mut generic_args = Vec::new();
            generic_args.push(self.parse_type_annotation()?);

            while self.match_token(&[TokenKind::Comma]) {
                generic_args.push(self.parse_type_annotation()?);
            }

            self.consume(
                TokenKind::GreaterThan,
                "Expected '>' after generic arguments",
            )?;

            Ok(ASTNode::Call {
                callee: Box::new(ASTNode::Identifier(name)),
                args: generic_args,
            })
        } else {
            Ok(ASTNode::Identifier(name))
        }
    }

    fn collect_type_string<F>(&mut self, stop_when: F) -> Result<String, ParserError>
    where
        F: Fn(&TokenKind, usize) -> bool,
    {
        let mut parts: Vec<String> = Vec::new();
        let mut depth = 0usize;

        while !self.is_at_end() {
            let kind = self.peek().kind.clone();
            if stop_when(&kind, depth) {
                break;
            }

            match kind {
                TokenKind::LessThan | TokenKind::OpenParen | TokenKind::OpenBracket => depth += 1,
                TokenKind::GreaterThan | TokenKind::CloseParen | TokenKind::CloseBracket => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                _ => {}
            }

            let token = self.advance().clone();
            let fragment = if !token.lexeme.is_empty() {
                token.lexeme
            } else {
                match token.kind {
                    TokenKind::Identifier(name) => name,
                    _ => token.kind.to_string(),
                }
            };
            parts.push(fragment);
        }

        Ok(parts.join(""))
    }

    /* ── Object/Struct literal helpers ───────────────────────────────── */

    fn parse_object_literal_fields(&mut self) -> Result<Vec<(String, ASTNode)>, ParserError> {
        self.parse_braced_fields(|_| "Expected '}' after object literal".to_string())
    }

    fn parse_struct_literal_fields(&mut self) -> Result<Vec<(String, ASTNode)>, ParserError> {
        let mut fields = Vec::new();

        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let key_token = self.advance().clone();
            let key = match key_token.kind {
                TokenKind::Identifier(name) => name,
                TokenKind::StringLiteral(s) => s,
                _ => {
                    return Err(ParserError {
                        message: "Struct field keys must be identifiers or strings".into(),
                        line: key_token.line,
                        column: key_token.column,
                    })
                }
            };

            self.consume(TokenKind::Colon, "Expected ':' after struct field key")?;
            let value = self.parse_expression()?;
            fields.push((key, value));

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
            if self.check(&TokenKind::CloseBrace) {
                break;
            }
        }

        self.consume(TokenKind::CloseBrace, "Expected '}' after struct fields")?;
        Ok(fields)
    }

    fn parse_braced_fields<F>(
        &mut self,
        closing_msg: F,
    ) -> Result<Vec<(String, ASTNode)>, ParserError>
    where
        F: Fn(&Parser) -> String,
    {
        let mut fields = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let key_token = self.advance().clone();
            let key = match key_token.kind {
                TokenKind::Identifier(name) => name,
                TokenKind::StringLiteral(s) => s,
                _ => {
                    return Err(ParserError {
                        message: "Object/struct keys must be identifiers or strings".into(),
                        line: key_token.line,
                        column: key_token.column,
                    })
                }
            };
            self.consume(TokenKind::Colon, "Expected ':' after object/struct key")?;
            let value = self.parse_expression()?;
            fields.push((key, value));

            if self.match_token(&[TokenKind::Comma]) {
                // Optional trailing comma before '}'
                if self.check(&TokenKind::CloseBrace) {
                    break;
                }
                continue;
            }

            // Allow loose semicolon separators
            while self.match_token(&[TokenKind::Semicolon]) {}
        }
        let message = closing_msg(self);
        self.consume(TokenKind::CloseBrace, &message)?;
        Ok(fields)
    }

    /* ── Utilities ───────────────────────────────────────────────────── */

    fn identifier_name(expr: &ASTNode) -> Option<String> {
        match expr {
            ASTNode::Identifier(name) => Some(name.clone()),
            ASTNode::IdentifierSpanned { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /* ── Match ───────────────────────────────────────────────────────── */

    fn parse_match(&mut self) -> Result<ASTNode, ParserError> {
        use crate::core::ast::MatchArm;

        self.consume(TokenKind::Match, "Expected 'match'")?;
        let value = Box::new(self.parse_expression()?);
        self.consume(TokenKind::OpenBrace, "Expected '{' after match value")?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let pattern = self.parse_expression()?;

            // Optional guard
            let guard = if self.check(&TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.consume(TokenKind::FatArrow, "Expected '=>' after match pattern")?;

            let body = if self.check(&TokenKind::OpenBrace) {
                self.parse_statement()?
            } else {
                self.parse_expression()?
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            let _ = self.match_token(&[TokenKind::Comma]); // optional trailing comma
        }

        self.consume(TokenKind::CloseBrace, "Expected '}' after match arms")?;
        Ok(ASTNode::Match { value, arms })
    }
}

/* ── Glyph helpers ───────────────────────────────────────────────────── */

/// Check if a hieroglyphic symbol represents a quantum variable declaration
fn is_quantum_variable_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "𓀀" |  // Egyptian hieroglyph A001 - quantum variable type 1
        "𓀁" |  // Egyptian hieroglyph A002 - quantum variable type 2
        "𓀂" |  // Egyptian hieroglyph A003 - quantum variable type 3
        "𓀃" |  // Egyptian hieroglyph A004 - quantum variable type 4
        "𓀄" |  // Egyptian hieroglyph A005 - quantum variable type 5
        "𓀅" |  // Egyptian hieroglyph A006 - quantum variable type 6
        "𓀆" |  // Egyptian hieroglyph A007 - quantum variable type 7
        "𓀇" |  // Egyptian hieroglyph A008 - quantum variable type 8
        "𓀈" |  // Egyptian hieroglyph A009 - quantum variable type 9
        "𓀉" // Egyptian hieroglyph A010 - quantum variable type 10
    )
}
