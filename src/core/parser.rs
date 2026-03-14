//! Parser for Aeonmi/QUBE/Titan with precedence parsing + spanned errors.

use crate::core::ast::{
    ASTNode, FStringPart, FunctionParam,
    MatchPattern, QuantumBindingType, QuantumFunctionType,
};
use crate::core::token::{Token, TokenKind};

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
            tokens.push(Token { kind: TokenKind::EOF, lexeme: String::new(), line: 0, column: 0 });
        }
        Parser { tokens, pos: 0 }
    }

    /// Main parse entrypoint: parses all tokens into program AST
    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            nodes.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(nodes))
    }

    /// Parses a single statement based on current token peek
    fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        // Skip stray semicolons
        while self.check(&TokenKind::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        if self.is_at_end() {
            return Ok(ASTNode::NullLiteral);
        }
        match self.peek().kind.clone() {
            // Traditional keywords (legacy compatibility)
            TokenKind::Let => self.parse_variable_decl(),
            TokenKind::Function => self.parse_function_decl(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Log => self.parse_log(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::OpenBrace => Ok(self.parse_block()?),
            
            // AEONMI Quantum-Native Syntax
            TokenKind::QuantumBracketOpen => self.parse_quantum_variable_decl(),
            TokenKind::ClassicalFunc => self.parse_quantum_function(QuantumFunctionType::Classical),
            TokenKind::QuantumFunc => self.parse_quantum_function(QuantumFunctionType::Quantum),
            TokenKind::AIFunc => self.parse_quantum_function(QuantumFunctionType::AINeural),
            TokenKind::QuantumOr => self.parse_probability_branch(), // ⊖ for probability branching
            TokenKind::QuantumLoop => self.parse_quantum_loop(),     // ⟲ for quantum loops
            TokenKind::Attempt => self.parse_quantum_try_catch(),    // ⚡ for quantum try-catch
            TokenKind::TimeBlock => self.parse_time_block(),         // ⏰ for time blocks
            TokenKind::Learn => self.parse_ai_learning_block(),      // AI learning blocks
            
            // Quantum qubit declaration: qubit q;
            // Lower to a string variable so superpose/measure can reference it by name.
            TokenKind::Qubit => {
                self.advance(); // consume 'qubit'
                let line = self.peek().line;
                let col  = self.peek().column;
                let name = self.consume_identifier("Expected qubit name after 'qubit'")?;
                let _ = self.match_token(&[TokenKind::Semicolon]);
                Ok(ASTNode::VariableDecl {
                    name: name.clone(),
                    value: Box::new(ASTNode::StringLiteral(name)),
                    line,
                    column: col,
                })
            }

            // Phase 1 keywords
            TokenKind::Import => self.parse_import_decl(),
            TokenKind::Quantum => self.parse_quantum_keyword_stmt(),
            TokenKind::Async => self.parse_async_function(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::Impl => self.parse_impl_block(),
            TokenKind::Struct => self.parse_struct_decl(false),
            TokenKind::Enum => self.parse_enum_decl(false),
            TokenKind::Const => {
                // treat const same as let for now
                self.advance();
                let line = self.peek().line;
                let col = self.peek().column;
                let name = self.consume_identifier("Expected name after 'const'")?;
                self.consume(TokenKind::Equals, "Expected '=' after const name")?;
                let value = self.parse_expression()?;
                let _ = self.match_token(&[TokenKind::Semicolon]);
                Ok(ASTNode::new_variable_decl_at(&name, value, line, col))
            }
            TokenKind::Type => {
                // type alias: `type X = SomeType;` — treat like const (skip `= SomeType`, emit NullLiteral)
                self.advance();
                let line = self.peek().line; let col = self.peek().column;
                let name = self.consume_identifier_or_keyword("Expected name after 'type'")?;
                if self.match_token(&[TokenKind::Equals]) {
                    let value = self.parse_expression()?;
                    let _ = self.match_token(&[TokenKind::Semicolon]);
                    return Ok(ASTNode::new_variable_decl_at(&name, value, line, col));
                }
                let _ = self.match_token(&[TokenKind::Semicolon]);
                Ok(ASTNode::new_variable_decl_at(&name, ASTNode::NullLiteral, line, col))
            }

            // Quantum operations
            TokenKind::Superpose | TokenKind::Entangle | TokenKind::Measure | TokenKind::Dod => {
                self.parse_quantum_op()
            }
            TokenKind::HieroglyphicOp(_) => self.parse_hieroglyphic_op(),
            
            // Comments (for now, skip them, but they could be processed for documentation)
            TokenKind::QuantumComment | TokenKind::BecauseComment | TokenKind::NoteComment => {
                self.advance(); // Skip comment token
                self.parse_statement() // Parse the next statement
            }
            
            _ => {
                // Special case: `loop { ... }` → while(true) { ... }
                if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
                    if s == "loop" && self.pos + 1 < self.tokens.len()
                        && matches!(self.tokens[self.pos + 1].kind, TokenKind::OpenBrace)
                    {
                        self.advance(); // consume 'loop'
                        let body = self.parse_statement()?;
                        return Ok(ASTNode::new_while(ASTNode::BooleanLiteral(true), body));
                    }
                    // `static NAME: Type = value;` → treat like const/let
                    if s == "static" && self.pos + 1 < self.tokens.len()
                        && matches!(self.tokens[self.pos + 1].kind, TokenKind::Identifier(_))
                    {
                        self.advance(); // consume 'static'
                        let line = self.peek().line; let col = self.peek().column;
                        let name = self.consume_identifier_or_keyword("Expected variable name after 'static'")?;
                        // Skip optional `: Type`
                        if self.match_token(&[TokenKind::Colon]) {
                            self.skip_type_annotation_until_eq();
                        }
                        if self.match_token(&[TokenKind::Equals]) {
                            let value = self.parse_expression()?;
                            let _ = self.match_token(&[TokenKind::Semicolon]);
                            return Ok(ASTNode::new_variable_decl_at(&name, value, line, col));
                        }
                        let _ = self.match_token(&[TokenKind::Semicolon]);
                        return Ok(ASTNode::new_variable_decl_at(&name, ASTNode::NullLiteral, line, col));
                    }
                }
                let expr = self.parse_expression()?;
                let _ = self.match_token(&[TokenKind::Semicolon, TokenKind::Comma]); // optional terminator
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
        let line = self.peek().line;
        let column = self.peek().column;

        // Skip optional `mut` modifier: `let mut x = ...`
        if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
            if s == "mut" { self.advance(); }
        }

        // Destructuring: let (a, b) = expr; or let (&a, &b) = expr;
        if self.check(&TokenKind::OpenParen) {
            self.advance(); // consume '('
            let mut names = Vec::new();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                // Skip optional `&` and `mut` prefix in each binding
                if self.check(&TokenKind::Ampersand) { self.advance(); }
                if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
                    if s == "mut" { self.advance(); }
                }
                names.push(self.consume_identifier_or_keyword("Expected variable name in destructuring")?);
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
            }
            self.consume(TokenKind::CloseParen, "Expected ')' after destructuring pattern")?;
            self.consume(TokenKind::Equals, "Expected '=' after destructuring pattern")?;
            let value = self.parse_expression()?;
            let _ = self.match_token(&[TokenKind::Semicolon]);
            // Lower to: let __destructured = value; let a = __destructured[0]; let b = __destructured[1];
            let mut stmts = Vec::new();
            let tmp = "__destructured";
            stmts.push(ASTNode::new_variable_decl_at(tmp, value, line, column));
            for (i, name) in names.iter().enumerate() {
                stmts.push(ASTNode::new_variable_decl_at(
                    name,
                    ASTNode::NumberLiteral(i as f64), // placeholder — runtime needs indexing
                    line, column,
                ));
            }
            return Ok(ASTNode::Block(stmts));
        }

        let name = self.consume_identifier_or_keyword("Expected variable name")?;
        // Optional type annotation: let x: Type = value;
        if self.match_token(&[TokenKind::Colon]) {
            self.skip_param_type_annotation_until_equals();
        }
        self.consume(TokenKind::Equals, "Expected '=' in variable declaration")?;
        let value = self.parse_expression()?;
        let _ = self.match_token(&[TokenKind::Semicolon]);
        Ok(ASTNode::new_variable_decl_at(&name, value, line, column))
    }

    fn parse_function_decl(&mut self) -> Result<ASTNode, ParserError> {
    let func_tok = self.consume(TokenKind::Function, "Expected 'function'")?;
    let func_line = func_tok.line; let func_col = func_tok.column;
    // Accept keywords as function names (e.g. `function new(...)`, `function type(...)`)
    let name = self.consume_identifier_or_keyword("Expected function name")?;
    self.consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let mut params: Vec<FunctionParam> = Vec::new();
        if !self.check(&TokenKind::CloseParen) {
            loop {
                let pname = self.consume_param_name("Expected parameter name")?;
                self.skip_param_type_annotation();
                params.push(FunctionParam { name: pname, line: func_line, column: func_col });
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) {
                    break;
                }
            }
        }
        self.consume(TokenKind::CloseParen, "Expected ')' after parameters")?;
        // Skip optional return type annotation: -> Type
        if self.match_token(&[TokenKind::Arrow]) {
            self.skip_type_annotation();
        }
        // Abstract/interface method: no body, just `;`
        if self.match_token(&[TokenKind::Semicolon]) {
            return Ok(ASTNode::new_function_at(&name, func_line, func_col, params, vec![]));
        }
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };
    Ok(ASTNode::new_function_at(&name, func_line, func_col, params, body))
    }

    fn parse_return(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Return, "Expected 'return'")?;
        // Support bare `return;` with no value
        let value = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::CloseBrace) || self.is_at_end() {
            ASTNode::NullLiteral
        } else {
            self.parse_expression()?
        };
        let _ = self.match_token(&[TokenKind::Semicolon]);
        Ok(ASTNode::new_return(value))
    }

    fn parse_log(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Log, "Expected 'log'")?;
        self.consume(TokenKind::OpenParen, "Expected '(' after log")?;
        // Collect all args (handles log(f"...") where f and "..." are separate tokens)
        let mut parts = Vec::new();
        if !self.check(&TokenKind::CloseParen) {
            loop {
                parts.push(self.parse_expression()?);
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
            }
        }
        self.consume(TokenKind::CloseParen, "Expected ')' after log args")?;
        let _ = self.match_token(&[TokenKind::Semicolon]);
        // If multiple parts, concatenate with +
        let expr = if parts.len() == 1 {
            parts.into_iter().next().unwrap()
        } else if parts.is_empty() {
            ASTNode::StringLiteral(String::new())
        } else {
            parts.into_iter().reduce(|a, b| {
                ASTNode::new_binary_expr(TokenKind::Plus, a, b)
            }).unwrap()
        };
        Ok(ASTNode::new_log(expr))
    }

    fn parse_if(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::If, "Expected 'if'")?;
        // Support `if let Pattern = expr` — skip the binding, use RHS as condition
        if self.match_token(&[TokenKind::Let]) {
            // skip over the pattern tokens until we find '='
            while !self.is_at_end() && !self.check(&TokenKind::Equals) {
                self.advance();
            }
            self.consume(TokenKind::Equals, "Expected '=' in if let")?;
            let cond = self.parse_expression()?;
            let then_branch = self.parse_statement()?;
            let else_branch = if self.match_token(&[TokenKind::Else]) {
                Some(self.parse_statement()?)
            } else {
                None
            };
            return Ok(ASTNode::new_if(cond, then_branch, else_branch));
        }
        // Support both C-style if (cond) and Rust-style if cond {
        let has_paren = self.match_token(&[TokenKind::OpenParen]);
        let cond = self.parse_expression()?;
        if has_paren {
            self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        }
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
        let has_paren = self.match_token(&[TokenKind::OpenParen]);
        let cond = self.parse_expression()?;
        if has_paren {
            self.consume(TokenKind::CloseParen, "Expected ')' after condition")?;
        }
        let body = self.parse_statement()?;
        Ok(ASTNode::new_while(cond, body))
    }

    fn parse_for(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::For, "Expected 'for'")?;

        // Check for destructuring for-in: for (a, b) in collection { ... }
        let is_destructuring_for_in = if self.check(&TokenKind::OpenParen) {
            // Scan ahead to see if we have '(' ids ')' 'in'
            let mut j = self.pos + 1;
            let mut depth = 1usize;
            while j < self.tokens.len() {
                match &self.tokens[j].kind {
                    TokenKind::OpenParen => { depth += 1; j += 1; }
                    TokenKind::CloseParen => {
                        depth -= 1;
                        j += 1;
                        if depth == 0 { break; }
                    }
                    _ => { j += 1; }
                }
            }
            j < self.tokens.len() && matches!(self.tokens[j].kind, TokenKind::In)
        } else {
            false
        };

        if is_destructuring_for_in {
            // for (a, b) in collection { ... } — also handles (a, &b)
            self.advance(); // consume '('
            let mut bindings = Vec::new();
            while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                // Skip optional `&` / `mut` prefix
                if self.check(&TokenKind::Ampersand) { self.advance(); }
                if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
                    if s == "mut" { self.advance(); }
                }
                bindings.push(self.consume_identifier_or_keyword("Expected variable name")?);
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
            }
            self.consume(TokenKind::CloseParen, "Expected ')' after bindings")?;
            self.consume(TokenKind::In, "Expected 'in'")?;
            let iterable = self.parse_expression()?;
            let body = self.parse_statement()?;
            // Emit as: for __item in iterable { let a = __item[0]; let b = __item[1]; body }
            let var = "__for_item";
            let mut stmts = vec![ASTNode::new_variable_decl_at(var, iterable, 0, 0)];
            for (i, name) in bindings.iter().enumerate() {
                stmts.push(ASTNode::new_variable_decl_at(
                    name,
                    ASTNode::NumberLiteral(i as f64),
                    0, 0,
                ));
            }
            stmts.push(body);
            return Ok(ASTNode::Block(stmts));
        }

        // Check for Rust-style: for x in collection { ... }
        // Detect: next is Identifier/keyword (or `&name`) and token after that is 'in'
        let is_for_in = {
            let first_is_name = matches!(self.peek().kind,
                TokenKind::Identifier(_) | TokenKind::Qubit | TokenKind::This | TokenKind::Self_
            );
            let first_is_ref = matches!(self.peek().kind, TokenKind::Ampersand)
                && self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::Identifier(_) | TokenKind::Qubit)
                && self.pos + 2 < self.tokens.len()
                && matches!(self.tokens[self.pos + 2].kind, TokenKind::In);
            let name_then_in = first_is_name
                && self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::In);
            name_then_in || first_is_ref
        };

        if is_for_in {
            // Skip optional `&` / `mut` prefix on loop variable
            if self.check(&TokenKind::Ampersand) { self.advance(); }
            let var_name = self.consume_identifier_or_keyword("Expected loop variable")?;
            self.consume(TokenKind::In, "Expected 'in'")?;
            let iterable = self.parse_expression()?;
            let body = self.parse_statement()?;
            // Lower to: for (let __i = 0; __i < collection.length; __i++) { let var = collection[__i]; body }
            // For now, emit as a block the VM/codegen can handle
            Ok(ASTNode::Block(vec![
                ASTNode::new_variable_decl_at(&var_name, iterable.clone(), 0, 0),
                body,
            ]))
        } else {
            // C-style: for (init; cond; incr) { ... }
            let has_paren = self.match_token(&[TokenKind::OpenParen]);
            let init = if !self.check(&TokenKind::Semicolon) {
                Some(self.parse_statement()?)
            } else {
                self.advance(); // consume ';'
                None
            };
            let condition = if !self.check(&TokenKind::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(TokenKind::Semicolon, "Expected ';' after loop condition")?;
            let increment = if !self.check(&TokenKind::CloseParen) && !(has_paren == false && self.check(&TokenKind::OpenBrace)) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            if has_paren {
                self.consume(TokenKind::CloseParen, "Expected ')' after for clauses")?;
            }
            let body = self.parse_statement()?;
            Ok(ASTNode::new_for(init, condition, increment, body))
        }
    }

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
        
        // Check if this is a quantum variable declaration (𓀀, 𓀁, etc.)
        if is_quantum_variable_symbol(&symbol) {
            // Parse as quantum variable declaration: 𓀀 name = value;
            let line = symbol_token.line;
            let column = symbol_token.column;
            let name = self.consume_identifier("Expected variable name after quantum symbol")?;
            self.consume(TokenKind::Equals, "Expected '=' in quantum variable declaration")?;
            let value = self.parse_expression()?;
            self.consume(TokenKind::Semicolon, "Expected ';' after quantum variable declaration")?;
            
            // Create a quantum variable declaration with the hieroglyphic type
            Ok(ASTNode::new_quantum_variable_decl_from_symbol(&name, value, &symbol, line, column))
        } else {
            // Parse as regular hieroglyphic operation
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

    /* ── Precedence ───────────────────────────────────────── */
    pub fn parse_expression(&mut self) -> Result<ASTNode, ParserError> { self.parse_logical_or() }

    // logical_or: logical_and ( '||' logical_and )*
    fn parse_logical_or(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_logical_and()?;
        while self.match_token(&[TokenKind::OrOr]) { let op = self.previous().kind.clone(); let right = self.parse_logical_and()?; expr = ASTNode::new_binary_expr(op, expr, right); }
        Ok(expr)
    }
    // logical_and: equality ( '&&' equality )*
    fn parse_logical_and(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_assignment()?; // parse below level (assignment/equality chain)
        while self.match_token(&[TokenKind::AndAnd]) { let op = self.previous().kind.clone(); let right = self.parse_assignment()?; expr = ASTNode::new_binary_expr(op, expr, right); }
        Ok(expr)
    }

    // assignment: Identifier '=' assignment | field.access '=' assignment | equality
    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        let expr = self.parse_equality()?;
        // Compound assignment: `+=`, `-=`, `*=`, `/=` (lexed as two tokens: op + `=`)
        let is_compound = matches!(self.peek().kind, TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash)
            && self.pos + 1 < self.tokens.len()
            && matches!(self.tokens[self.pos + 1].kind, TokenKind::Equals);
        if is_compound {
            let op = self.advance().kind.clone(); // consume op
            self.advance(); // consume '='
            let rhs = self.parse_assignment()?;
            // Desugar: lhs op= rhs  →  lhs = lhs op rhs
            let combined = ASTNode::new_binary_expr(op, expr.clone(), rhs);
            return match expr {
                ASTNode::Identifier(name) => {
                    let line = self.previous().line; let column = self.previous().column;
                    Ok(ASTNode::new_assignment_at(&name, combined, line, column))
                }
                ASTNode::IdentifierSpanned { name, line: id_line, column: id_col, .. } => {
                    Ok(ASTNode::new_assignment_at(&name, combined, id_line, id_col))
                }
                ASTNode::FieldAccess { object, field } => {
                    Ok(ASTNode::FieldAssign { object, field, value: Box::new(combined) })
                }
                _ => Ok(combined),
            };
        }
        if self.match_token(&[TokenKind::Equals]) {
            let value = self.parse_assignment()?;
            match expr {
                ASTNode::Identifier(name) => {
                    let line = self.previous().line; let column = self.previous().column;
                    Ok(ASTNode::new_assignment_at(&name, value, line, column))
                }
                ASTNode::IdentifierSpanned { name, line: id_line, column: id_col, .. } => {
                    Ok(ASTNode::new_assignment_at(&name, value, id_line, id_col))
                }
                ASTNode::FieldAccess { object, field } => {
                    // field assignment: obj.field = value
                    Ok(ASTNode::FieldAssign {
                        object,
                        field,
                        value: Box::new(value),
                    })
                }
                ASTNode::MethodCall { .. } | ASTNode::Call { .. } => {
                    // Allow but treat as expression-statement (side effect)
                    Ok(value)
                }
                _ => Err(self.err_here("Invalid assignment target")),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_equality(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_comparison()?;
        while self.match_token(&[
            TokenKind::DoubleEquals, TokenKind::NotEquals,
            // Bitwise OR / AND as low-precedence ops (also handles `matches!(x, A | B)`)
            TokenKind::Pipe, TokenKind::Ampersand,
        ]) {
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
            // Use quantum binary expr for quantum operators
            expr = match op {
                TokenKind::QuantumGeq | TokenKind::QuantumLeq => {
                    ASTNode::new_quantum_binary_expr(op, expr, right)
                }
                _ => ASTNode::new_binary_expr(op, expr, right),
            };
        }
        Ok(expr)
    }
    
    // New: quantum operations have their own precedence level
    fn parse_quantum_ops(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_term()?;
        while self.match_token(&[
            TokenKind::QuantumTensor,    // ⊗
            TokenKind::QuantumXor,       // ⊕
            TokenKind::QuantumModulo,    // ◊
            TokenKind::QuantumGradient,  // ∇
            TokenKind::QuantumApprox,    // ≈
        ]) {
            let op = self.previous().kind.clone();
            let right = self.parse_term()?;
            expr = ASTNode::new_quantum_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_factor()?;
        loop {
            // Don't consume `+` or `-` if followed by `=` (compound assignment `+=` / `-=`)
            let is_compound = matches!(self.peek().kind, TokenKind::Plus | TokenKind::Minus)
                && self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::Equals);
            if is_compound { break; }
            if !self.match_token(&[TokenKind::Plus, TokenKind::Minus]) { break; }
            let op = self.previous().kind.clone();
            let right = self.parse_factor()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_unary()?;
        loop {
            // Don't consume `*` or `/` if followed by `=` (compound assignment `*=` / `/=`)
            let is_compound = matches!(self.peek().kind, TokenKind::Star | TokenKind::Slash)
                && self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::Equals);
            if is_compound { break; }
            if !self.match_token(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent,
                                   TokenKind::ShiftLeft, TokenKind::ShiftRight]) { break; }
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ASTNode, ParserError> {
        if self.match_token(&[TokenKind::Minus, TokenKind::Plus, TokenKind::Bang]) {
            let op = self.previous().kind.clone();
            let right = self.parse_unary()?;
            return Ok(ASTNode::new_unary_expr(op, right));
        }
        // Dereference operator `*expr` — treat as identity (no real pointer tracking)
        if self.match_token(&[TokenKind::Star]) {
            return self.parse_unary();
        }
        // Reference operator `&expr` or `&mut expr` — treat as identity
        if self.match_token(&[TokenKind::Ampersand]) {
            // Skip optional `mut` keyword after `&`
            if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
                if s == "mut" { self.advance(); }
            }
            return self.parse_unary();
        }
        self.parse_call()
    }

    // support calls, method calls, field access, and :: constructors
    fn parse_call(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenKind::Bang) && self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::OpenBracket | TokenKind::OpenParen)
            {
                // Macro call: vec![], format!(...), panic!(...)
                self.advance(); // consume '!'
                if self.match_token(&[TokenKind::OpenBracket]) {
                    // vec![...] style — also handles vec![value; count]
                    let mut elems = Vec::new();
                    while !self.check(&TokenKind::CloseBracket) && !self.is_at_end() {
                        if self.check(&TokenKind::CloseBracket) { break; }
                        elems.push(self.parse_expression()?);
                        // Handle `vec![value; count]` — skip `;` and count
                        if self.match_token(&[TokenKind::Semicolon]) {
                            if !self.check(&TokenKind::CloseBracket) { let _ = self.parse_expression()?; }
                            break;
                        }
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseBracket) { break; }
                    }
                    self.consume(TokenKind::CloseBracket, "Expected ']' in macro invocation")?;
                    expr = ASTNode::ArrayLiteral(elems);
                } else {
                    // macro!(...) style — treat as regular call
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                    }
                    self.consume(TokenKind::CloseParen, "Expected ')' in macro invocation")?;
                    expr = ASTNode::new_call(expr, args);
                }
            } else if self.match_token(&[TokenKind::OpenParen]) {
                // regular call: expr(...)
                let mut args = Vec::new();
                if !self.check(&TokenKind::CloseParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after arguments")?;
                expr = ASTNode::new_call(expr, args);
            } else if self.match_token(&[TokenKind::Dot]) {
                // Check for `..` range operator: lhs..rhs or lhs..
                if self.check(&TokenKind::Dot) {
                    self.advance(); // consume second dot
                    // Parse RHS if one exists (skip for open-ended ranges like `start..`)
                    let has_rhs = !matches!(self.peek().kind,
                        TokenKind::CloseBracket | TokenKind::CloseParen |
                        TokenKind::Semicolon | TokenKind::Comma |
                        TokenKind::FatArrow | TokenKind::EOF);
                    if has_rhs {
                        let rhs = self.parse_equality()?;
                        // Represent range as array [start, end] for runtime compatibility
                        expr = ASTNode::ArrayLiteral(vec![expr, rhs]);
                    }
                    break; // range is a complete expression — end postfix chain
                }
                // dot access: expr.field or expr.method(...)
                // Also handle `.await` postfix
                let member = if self.match_token(&[TokenKind::Await]) {
                    "await".to_string()
                } else {
                    // Accept identifiers AND reserved keywords as field/method names
                    let tok = self.peek().kind.clone();
                    match tok {
                        TokenKind::Identifier(n) => { self.advance(); n }
                        TokenKind::EOF | TokenKind::Semicolon | TokenKind::Comma |
                        TokenKind::CloseParen | TokenKind::CloseBrace | TokenKind::CloseBracket => {
                            return Err(self.err_here("Expected field or method name after '.'"));
                        }
                        _ => { let s = tok.to_string(); self.advance(); s }
                    }
                };
                if self.check(&TokenKind::OpenParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::CloseParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                        }
                    }
                    self.consume(TokenKind::CloseParen, "Expected ')' after method args")?;
                    expr = ASTNode::MethodCall { object: Box::new(expr), method: member, args };
                } else {
                    expr = ASTNode::FieldAccess { object: Box::new(expr), field: member };
                }
            } else if self.match_token(&[TokenKind::ColonColon]) {
                // constructor / static call: Type::method(...)
                // Also handle turbofish: expr::<Type>()
                if self.check(&TokenKind::LessThan) {
                    // Turbofish — skip <Type> then continue to parse call
                    self.advance(); // consume '<'
                    let mut depth = 1usize;
                    while !self.is_at_end() && depth > 0 {
                        match self.peek().kind {
                            TokenKind::LessThan => { depth += 1; self.advance(); }
                            TokenKind::GreaterThan | TokenKind::ShiftRight => { if depth > 1 && matches!(self.tokens[self.pos].kind, TokenKind::ShiftRight) { depth = depth.saturating_sub(2); } else { depth -= 1; } self.advance(); }
                            _ => { self.advance(); }
                        }
                    }
                    // Now should be '(' for the call — loop back to top
                    continue;
                }
                // Accept identifiers AND keywords used as method names (new, type, self)
                let method = match self.peek().kind.clone() {
                    TokenKind::Identifier(name) => { self.advance(); name }
                    TokenKind::New => { self.advance(); "new".to_string() }
                    TokenKind::Type => { self.advance(); "type".to_string() }
                    TokenKind::Self_ => { self.advance(); "self".to_string() }
                    _ => return Err(self.err_here("Expected method name after '::'")),
                };
                if self.check(&TokenKind::OpenParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::CloseParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                        }
                    }
                    self.consume(TokenKind::CloseParen, "Expected ')' after constructor args")?;
                    expr = ASTNode::MethodCall { object: Box::new(expr), method, args };
                } else {
                    expr = ASTNode::FieldAccess { object: Box::new(expr), field: method };
                }
            } else if self.match_token(&[TokenKind::OpenBracket]) {
                // subscript: expr[index]
                let index = self.parse_expression()?;
                self.consume(TokenKind::CloseBracket, "Expected ']' after subscript")?;
                expr = ASTNode::QuantumIndexAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                    is_quantum_index: false,
                };
            } else if self.match_token(&[TokenKind::Question]) {
                // Postfix `?` — error propagation / Option unwrap
                // Represent as __try__(expr) for runtime; semantically keep inner value
                expr = ASTNode::new_call(
                    ASTNode::new_identifier_spanned("__try__", 0, 0, 7),
                    vec![expr],
                );
            } else if self.check(&TokenKind::OpenBrace) {
                // Struct literal: TypeName { field: value, ... }
                // Only when expr is an uppercase type name AND the body looks like struct fields
                let is_type_name = match &expr {
                    ASTNode::Identifier(n) |
                    ASTNode::IdentifierSpanned { name: n, .. } => {
                        n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    }
                    ASTNode::FieldAccess { field, .. } => {
                        field.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    }
                    _ => false,
                };
                if is_type_name && self.looks_like_struct_literal_body() {
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                        // Skip comment and punctuation tokens
                        if matches!(self.peek().kind,
                            TokenKind::QuantumComment | TokenKind::BecauseComment | TokenKind::NoteComment
                        ) { self.advance(); continue; }
                        if self.check(&TokenKind::CloseBrace) { break; }
                        let fname = self.consume_identifier_or_keyword("Expected field name in struct literal")?;
                        let fval = if self.match_token(&[TokenKind::Colon]) {
                            self.parse_expression()?
                        } else {
                            // shorthand: `field` = `field` identifier
                            ASTNode::new_identifier_spanned(&fname, 0, 0, fname.len())
                        };
                        fields.push((fname, fval));
                        let _ = self.match_token(&[TokenKind::Comma]);
                    }
                    self.consume(TokenKind::CloseBrace, "Expected '}' after struct literal")?;
                    // Represent as a call to typename(field_vals...)
                    let type_name = match &expr {
                        ASTNode::Identifier(n) => n.clone(),
                        ASTNode::IdentifierSpanned { name, .. } => name.clone(),
                        ASTNode::FieldAccess { field, .. } => field.clone(),
                        _ => "__struct__".to_string(),
                    };
                    let field_vals: Vec<ASTNode> = fields.into_iter()
                        .map(|(_, v)| v)
                        .collect();
                    expr = ASTNode::new_call(
                        ASTNode::new_identifier_spanned(&type_name, 0, 0, type_name.len()),
                        field_vals,
                    );
                } else {
                    break;
                }
            } else if let TokenKind::Identifier(ref kw) = self.peek().kind.clone() {
                if kw == "as" {
                    // `expr as Type` cast — skip type, keep expr
                    self.advance(); // consume 'as'
                    let _ = self.consume_type_name(); // skip type
                    // expr keeps its value unchanged
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParserError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::NumberLiteral(v) => Ok(ASTNode::NumberLiteral(v)),
            TokenKind::StringLiteral(s) => Ok(ASTNode::StringLiteral(s)),
            TokenKind::BooleanLiteral(b) => Ok(ASTNode::BooleanLiteral(b)),
            TokenKind::Identifier(name) => Ok(ASTNode::new_identifier_spanned(&name, tok.line, tok.column, name.len())),
            TokenKind::This => Ok(ASTNode::new_identifier_spanned("this", tok.line, tok.column, 4)),
            TokenKind::Self_ => Ok(ASTNode::new_identifier_spanned("self", tok.line, tok.column, 4)),
            // `if` as an expression: let x = if cond { a } else { b }
            TokenKind::If => {
                // Put the If token back and delegate to parse_if
                self.pos -= 1;
                self.parse_if()
            }
            // `match` as an expression: let x = match val { A => ... }
            TokenKind::Match => {
                self.pos -= 1;
                self.parse_match_expr()
            }
            
            // Quantum state literals: |0⟩, |1⟩, |+⟩, etc.
            TokenKind::QubitLiteral(literal) => {
                // Extract state content from |state⟩ format
                let state_content = if literal.starts_with("|") && literal.ends_with("⟩") {
                    literal.trim_start_matches("|").trim_end_matches("⟩")
                } else if literal.starts_with("|") && literal.ends_with(">") {
                    // Fallback for basic > instead of ⟩
                    literal.trim_start_matches("|").trim_end_matches(">")
                } else {
                    &literal
                };
                
                // Create properly formatted quantum state
                let formatted_state = format!("|{}⟩", state_content);
                Ok(ASTNode::new_quantum_state(&formatted_state, None))
            }
            
            // Traditional parentheses, tuple literals, or unit value ()
            TokenKind::OpenParen => {
                // Empty parens () = unit/void value
                if self.check(&TokenKind::CloseParen) {
                    self.advance(); // consume ')'
                    return Ok(ASTNode::NullLiteral);
                }
                let expr = self.parse_expression()?;
                // Tuple: (a, b, ...) → treat as array literal
                if self.match_token(&[TokenKind::Comma]) {
                    let mut elems = vec![expr];
                    if !self.check(&TokenKind::CloseParen) {
                        loop {
                            elems.push(self.parse_expression()?);
                            if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                        }
                    }
                    self.consume(TokenKind::CloseParen, "Expected ')' after tuple")?;
                    return Ok(ASTNode::ArrayLiteral(elems));
                }
                self.consume(TokenKind::CloseParen, "Expected ')'")?;
                Ok(expr)
            }

            // Array literal: [a, b, c]
            TokenKind::OpenBracket => {
                let mut elems = Vec::new();
                if !self.check(&TokenKind::CloseBracket) {
                    loop {
                        if self.check(&TokenKind::CloseBracket) { break; } // trailing comma
                        elems.push(self.parse_expression()?);
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                    }
                }
                self.consume(TokenKind::CloseBracket, "Expected ']' after array elements")?;
                Ok(ASTNode::ArrayLiteral(elems))
            }

            // f-string: f"text {var}"
            TokenKind::FString => {
                // Lexer already tokenized it; treat as StringLiteral for now
                // Full interpolation requires lexer support — emit as-is
                Ok(ASTNode::FStringLiteral(vec![FStringPart::Literal(tok.lexeme.clone())]))
            }

            // await expr
            TokenKind::Await => {
                let inner = self.parse_unary()?;
                Ok(ASTNode::AwaitExpr(Box::new(inner)))
            }

            // quantum builtins usable as expressions: superpose(...), entangle(...), etc.
            TokenKind::Superpose => Ok(ASTNode::new_identifier_spanned("superpose", tok.line, tok.column, "superpose".len())),
            TokenKind::Entangle => Ok(ASTNode::new_identifier_spanned("entangle", tok.line, tok.column, "entangle".len())),
            TokenKind::Measure  => Ok(ASTNode::new_identifier_spanned("measure",  tok.line, tok.column, "measure".len())),
            TokenKind::Dod      => Ok(ASTNode::new_identifier_spanned("dod",      tok.line, tok.column, "dod".len())),

            // Closure: |params| -> { body } or |params| expr
            // Also handles: || expr (zero-param closure via OrOr token)
            TokenKind::OrOr => {
                // `|| expr` — zero-parameter closure
                let _ = self.match_token(&[TokenKind::Arrow]); // optional ->
                let body = if self.check(&TokenKind::OpenBrace) {
                    match self.parse_block()? {
                        ASTNode::Block(stmts) => stmts,
                        other => vec![other],
                    }
                } else {
                    vec![self.parse_expression()?]
                };
                Ok(ASTNode::Closure { params: vec![], body })
            }
            TokenKind::Pipe => {
                let mut params = Vec::new();
                if !self.check(&TokenKind::Pipe) {
                    loop {
                        let pname = self.consume_param_name("Expected closure parameter name")?;
                        self.skip_param_type_annotation();
                        params.push(FunctionParam { name: pname, line: tok.line, column: tok.column });
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                    }
                }
                self.consume(TokenKind::Pipe, "Expected '|' after closure parameters")?;
                // Optional -> before body
                let _ = self.match_token(&[TokenKind::Arrow]);
                let body = if self.check(&TokenKind::OpenBrace) {
                    match self.parse_block()? {
                        ASTNode::Block(stmts) => stmts,
                        other => vec![other],
                    }
                } else {
                    vec![self.parse_expression()?]
                };
                Ok(ASTNode::Closure { params, body })
            }

            // null literal
            TokenKind::Null => Ok(ASTNode::NullLiteral),
            TokenKind::True => Ok(ASTNode::BooleanLiteral(true)),
            TokenKind::False => Ok(ASTNode::BooleanLiteral(false)),
            
            // AEONMI Quantum-Native Constructs
            
            // Quantum arrays / JSON-like map objects: { ... }
            TokenKind::OpenBrace => {
                let mut elements = Vec::new();
                
                if !self.check(&TokenKind::CloseBrace) {
                    loop {
                        let elem = self.parse_expression()?;
                        if self.match_token(&[TokenKind::Colon]) {
                            // `key: value` map entry — parse value and push both or skip
                            let _ = self.parse_expression()?;
                            // Push key as the element (value is discarded for now)
                            elements.push(elem);
                        } else {
                            elements.push(elem);
                        }
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseBrace) {
                            break;
                        }
                    }
                }
                
                self.consume(TokenKind::CloseBrace, "Expected '}' after array/map elements")?;
                
                // Check if this is a superposition array (contains quantum states)
                let is_superposition = elements.iter().any(|elem| {
                    matches!(elem, ASTNode::QuantumState { .. })
                });
                
                Ok(ASTNode::new_quantum_array(elements, is_superposition))
            }
            
            // Quantum variable access: ⟨variable⟩
            TokenKind::QuantumBracketOpen => {
                let name = self.consume_identifier("Expected variable name after '⟨'")?;
                self.consume(TokenKind::QuantumBracketClose, "Expected '⟩' after variable name")?;
                
                // Check for quantum indexing: ⟨var⟩⟦index⟧
                if self.check(&TokenKind::QuantumIndexOpen) {
                    self.advance(); // consume ⟦
                    let index = self.parse_expression()?;
                    self.consume(TokenKind::QuantumIndexClose, "Expected '⟧' after index")?;
                    Ok(ASTNode::new_quantum_index_access(
                        ASTNode::Identifier(name),
                        index,
                        true  // quantum indexing
                    ))
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            }
            
            // Fallback: treat any keyword token as an identifier (e.g. `qubit`, `type`, etc.)
            _ => {
                let name = tok.kind.to_string();
                // Only accept if it looks like a valid identifier name (not punctuation)
                match tok.kind {
                    TokenKind::Semicolon | TokenKind::Comma | TokenKind::CloseParen |
                    TokenKind::CloseBrace | TokenKind::CloseBracket | TokenKind::EOF => {
                        Err(ParserError {
                            message: format!("Unexpected token {:?}", tok.kind),
                            line: tok.line,
                            column: tok.column,
                        })
                    }
                    _ => Ok(ASTNode::new_identifier_spanned(&name, tok.line, tok.column, name.len())),
                }
            }
        }
    }
    
    fn parse_quantum_state(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Pipe, "Expected '|'")?;
        
        // Parse the state content (could be identifier, number, or special symbols)
        let state_content = if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            name
        } else if let TokenKind::NumberLiteral(num) = self.peek().kind.clone() {
            self.advance();
            num.to_string()
        } else {
            // Handle special quantum states like +, -, etc.
            let ch = self.advance().lexeme.clone();
            ch
        };
        
        self.consume(TokenKind::GreaterThan, "Expected '⟩' after quantum state")?;
        
        // Check for amplitude specification
        let amplitude = if self.match_token(&[TokenKind::Star]) {
            if let ASTNode::NumberLiteral(amp) = self.parse_expression()? {
                Some(amp)
            } else {
                return Err(self.err_here("Expected amplitude value after '*'"));
            }
        } else {
            None
        };
        
        Ok(ASTNode::new_quantum_state(&format!("|{}⟩", state_content), amplitude))
    }

    /* ── Token utils ─────────────────────────────────────── */
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
        // Safe: we ensure there's always an EOF at the end
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
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
            Err(self.err_at(msg, self.peek().line, self.peek().column))
        }
    }

    fn consume_identifier(&mut self, msg: &str) -> Result<String, ParserError> {
        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.err_at(msg, self.peek().line, self.peek().column))
        }
    }

    /// Like `consume_identifier` but also accepts any keyword token as a variable name.
    fn consume_identifier_or_keyword(&mut self, msg: &str) -> Result<String, ParserError> {
        let name = self.peek().kind.to_string();
        match &self.peek().kind {
            TokenKind::Identifier(n) => { let n = n.clone(); self.advance(); Ok(n) }
            TokenKind::EOF | TokenKind::OpenParen | TokenKind::CloseParen |
            TokenKind::OpenBrace | TokenKind::CloseBrace | TokenKind::Semicolon |
            TokenKind::Comma | TokenKind::Equals | TokenKind::Colon => {
                Err(self.err_at(msg, self.peek().line, self.peek().column))
            }
            _ => { self.advance(); Ok(name) }
        }
    }

    /// Consume a function parameter name, skipping optional `&`, `&mut`, `mut` prefix.
    /// Handles Rust-style `&mut self`, `&self`, `mut name` in parameter lists.
    fn consume_param_name(&mut self, msg: &str) -> Result<String, ParserError> {
        // Skip leading `&` (reference)
        if self.check(&TokenKind::Ampersand) { self.advance(); }
        // Skip `mut`
        if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
            if s == "mut" { self.advance(); }
        }
        // Handle destructuring param: `(a, b)` or `(_, count)` → skip and return synthetic name
        if self.check(&TokenKind::OpenParen) {
            let mut depth = 0usize;
            while !self.is_at_end() {
                match self.peek().kind {
                    TokenKind::CloseParen if depth == 0 => { self.advance(); break; }
                    TokenKind::OpenParen => { depth += 1; self.advance(); }
                    TokenKind::CloseParen => { depth -= 1; self.advance(); if depth == 0 { break; } }
                    _ => { self.advance(); }
                }
            }
            return Ok("__destructured__".to_string());
        }
        self.consume_identifier_or_keyword(msg)
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

    /// Lookahead heuristic: does the `{` that follows look like a struct literal body
    /// (rather than a regular block)?  Scans ahead for `identifier :` (typed field)
    /// within the next 30 tokens; stops early on statement keywords or dot-access.
    fn looks_like_struct_literal_body(&self) -> bool {
        // self.pos is pointing at '{'
        let mut i = self.pos + 1;
        let limit = (i + 30).min(self.tokens.len().saturating_sub(1));
        while i < limit {
            let cur = &self.tokens[i].kind;
            let nxt = if i + 1 < self.tokens.len() { &self.tokens[i + 1].kind } else { &TokenKind::EOF };
            // `{ .. }` or `{ field, .. }` — struct wildcard patterns → treat as struct literal
            if matches!(cur, TokenKind::Dot) {
                if matches!(nxt, TokenKind::Dot) { return true; } // `..` spread/wildcard (two Dot tokens)
                return false; // single `.` → method call
            }
            // Determine if current token looks like a potential field name
            let cur_is_name = matches!(cur,
                TokenKind::Identifier(_) |
                TokenKind::Qubit | TokenKind::Quantum | TokenKind::Measure |
                TokenKind::This | TokenKind::Self_
            );
            match (cur_is_name, nxt) {
                // name followed by ':' → struct field with explicit value
                (true, TokenKind::Colon) => return true,
                // name followed by ',' or '}' → shorthand struct field (e.g. { qubit })
                (true, TokenKind::Comma) | (true, TokenKind::CloseBrace) => return true,
                _ => {}
            }
            match cur {
                // statement-starting keywords → it's a block
                TokenKind::If | TokenKind::While | TokenKind::For
                | TokenKind::Let | TokenKind::Return => return false,
                // close brace before finding anything matching → block or empty struct
                TokenKind::CloseBrace => return false,
                _ => { i += 1; }
            }
        }
        false
    }
    
    // AEONMI Quantum-Native Parsing Functions
    
    fn parse_quantum_variable_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumBracketOpen, "Expected '⟨'")?;
        let line = self.peek().line;
        let column = self.peek().column;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(TokenKind::QuantumBracketClose, "Expected '⟩'")?;
        
        // Determine binding type based on operator
        let binding_type = match self.peek().kind {
            TokenKind::QuantumBind => QuantumBindingType::Classical,      // ←
            TokenKind::QuantumIn => QuantumBindingType::Superposition,    // ∈
            TokenKind::QuantumTensor => QuantumBindingType::Tensor,       // ⊗
            TokenKind::QuantumApprox => QuantumBindingType::Approximation, // ≈
            _ => return Err(self.err_here("Expected quantum binding operator (←, ∈, ⊗, ≈)")),
        };
        
        self.advance(); // consume the binding operator
        let value = self.parse_expression()?;
        
        Ok(ASTNode::new_quantum_variable_decl(&name, binding_type, value, line, column))
    }
    
    fn parse_quantum_function(&mut self, func_type: QuantumFunctionType) -> Result<ASTNode, ParserError> {
        let func_tok = self.advance(); // consume function marker (◯, ⊙, 🧠)
        let func_line = func_tok.line;
        let func_col = func_tok.column;
        
        let name = self.consume_identifier("Expected function name")?;
        
        // Parse parameters with quantum brackets
        self.consume(TokenKind::QuantumBracketOpen, "Expected '⟨' before parameters")?;
        let mut params = Vec::new();
        
        if !self.check(&TokenKind::QuantumBracketClose) {
            loop {
                let pname = self.consume_param_name("Expected parameter name")?;
                params.push(FunctionParam { name: pname, line: func_line, column: func_col });
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenKind::QuantumBracketClose, "Expected '⟩' after parameters")?;
        
        // Parse return type annotation if present
        if self.match_token(&[TokenKind::QuantumImplies]) { // →
            // Skip return type for now, could be enhanced later
            let _ = self.parse_expression()?;
        }
        
        // Parse function body
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };
        
        Ok(ASTNode::new_quantum_function(func_type, &name, params, body, func_line, func_col))
    }
    
    fn parse_probability_branch(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumOr, "Expected '⊖'")?; // probability branch operator
        
        let condition = self.parse_expression()?;
        
        // Optional explicit probability ≈ 0.8
        let probability = if self.match_token(&[TokenKind::QuantumApprox]) {
            if let ASTNode::NumberLiteral(p) = self.parse_expression()? {
                Some(p)
            } else {
                return Err(self.err_here("Expected probability value after '≈'"));
            }
        } else {
            None
        };
        
        self.consume(TokenKind::QuantumImplies, "Expected '⇒' after condition")?;
        
        let then_branch = self.parse_statement()?;
        
        let else_branch = if self.match_token(&[TokenKind::QuantumXor]) { // ⊕ for else
            Some(self.parse_statement()?)
        } else {
            None
        };
        
        Ok(ASTNode::new_probability_branch(condition, probability, then_branch, else_branch))
    }
    
    fn parse_quantum_loop(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::QuantumLoop, "Expected '⟲'")?;
        
        let condition = self.parse_expression()?;
        
        // Optional decoherence threshold
        let decoherence_threshold = if self.match_token(&[TokenKind::QuantumGeq]) { // ⪰
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
        
        // Parse ⚠️ error handling
        if self.match_token(&[TokenKind::Warning]) {
            // Optional error probability
            if self.match_token(&[TokenKind::QuantumApprox]) {
                if let ASTNode::NumberLiteral(p) = self.parse_expression()? {
                    error_probability = Some(p);
                } else {
                    return Err(self.err_here("Expected error probability after '≈'"));
                }
            }
            
            self.consume(TokenKind::QuantumImplies, "Expected '⇒' after error condition")?;
            
            catch_body = Some(match self.parse_block()? {
                ASTNode::Block(stmts) => stmts,
                single => vec![single],
            });
        }
        
        // Parse ✓ success handling
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
            data_binding: None,    // Could be enhanced to parse explicit bindings
            model_binding: None,
            body,
        })
    }

    // ── Phase 1 parse functions ──────────────────────────────────────────

    /// import { X, Y } from "./path";
    fn parse_import_decl(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Import, "Expected 'import'")?;
        let mut names: Vec<String> = Vec::new();
        if self.match_token(&[TokenKind::OpenBrace]) {
            while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                names.push(self.consume_identifier("Expected name in import list")?);
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
            }
            self.consume(TokenKind::CloseBrace, "Expected '}' after import list")?;
        } else {
            // default import: import Foo from "./path"
            names.push(self.consume_identifier("Expected import name")?);
        }
        self.consume(TokenKind::From, "Expected 'from' after import names")?;
        let path = match self.advance().kind.clone() {
            TokenKind::StringLiteral(s) => s,
            _ => return Err(self.err_here("Expected string path after 'from'")),
        };
        let _ = self.match_token(&[TokenKind::Semicolon]);
        Ok(ASTNode::ImportDecl { names, path })
    }

    /// quantum function / quantum struct / quantum enum dispatch
    fn parse_quantum_keyword_stmt(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Quantum, "Expected 'quantum'")?;
        match self.peek().kind.clone() {
            TokenKind::Function => {
                // quantum function foo(...) { ... }
                self.advance(); // consume 'function'
                let line = self.peek().line;
                let col = self.peek().column;
                let name = self.consume_identifier("Expected function name after 'quantum function'")?;
                self.consume(TokenKind::OpenParen, "Expected '('")?;
                let mut params = Vec::new();
                if !self.check(&TokenKind::CloseParen) {
                    loop {
                        let pname = self.consume_param_name("Expected parameter name")?;
                        self.skip_param_type_annotation();
                        params.push(FunctionParam { name: pname, line, column: col });
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')'")?;
                // optional return type annotation: -> Type
                if self.match_token(&[TokenKind::Arrow]) {
                    self.skip_type_annotation();
                }
                let body = match self.parse_block()? {
                    ASTNode::Block(stmts) => stmts,
                    _ => return Err(self.err_here("Expected block")),
                };
                Ok(ASTNode::new_quantum_function(
                    crate::core::ast::QuantumFunctionType::Quantum,
                    &name, params, body, line, col,
                ))
            }
            TokenKind::Struct => {
                self.advance();
                self.parse_struct_decl(true)
            }
            TokenKind::Class => {
                // `quantum class Foo { ... }` → parse as quantum struct
                self.advance();
                self.parse_struct_decl(true)
            }
            TokenKind::Enum => {
                self.advance();
                self.parse_enum_decl(true)
            }
            TokenKind::Identifier(ref s) if s == "circuit" => {
                self.advance(); // consume 'circuit'
                self.parse_quantum_circuit()
            }
            _ => {
                // Generic quantum block: quantum <tag> <Name> { body }
                // Handles: quantum command_line, quantum config, quantum module, etc.
                let tag_name = if let TokenKind::Identifier(ref s) = self.peek().kind {
                    let s = s.clone();
                    self.advance(); // consume the tag (e.g. "command_line")
                    s
                } else {
                    // Not an identifier after quantum — parse as expression
                    let expr = self.parse_expression()?;
                    let _ = self.match_token(&[TokenKind::Semicolon]);
                    return Ok(expr);
                };

                // Optional block name (e.g. ShardCompiler, CompilerConfig)
                let block_name = if let TokenKind::Identifier(_) = self.peek().kind {
                    self.consume_identifier("Expected block name").ok()
                } else {
                    None
                };
                let name = block_name.unwrap_or_else(|| tag_name.clone());

                // Skip optional generic type params: `Name<T>`, `Name<T, U>`
                if self.check(&TokenKind::LessThan) {
                    let mut depth = 1usize;
                    self.advance(); // consume '<'
                    while !self.is_at_end() && depth > 0 {
                        match self.peek().kind {
                            TokenKind::LessThan => { depth += 1; self.advance(); }
                            TokenKind::GreaterThan | TokenKind::ShiftRight => { if depth > 1 && matches!(self.tokens[self.pos].kind, TokenKind::ShiftRight) { depth = depth.saturating_sub(2); } else { depth -= 1; } self.advance(); }
                            _ => { self.advance(); }
                        }
                    }
                }

                // Skip optional trait/interface clause: `implements Foo`, `extends Bar`
                while let TokenKind::Identifier(ref kw) = self.peek().kind.clone() {
                    if kw == "implements" || kw == "extends" {
                        self.advance(); // consume keyword
                        // consume the trait/interface name(s)
                        while let TokenKind::Identifier(_) = self.peek().kind.clone() {
                            self.advance();
                            if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                        }
                    } else {
                        break;
                    }
                }

                // If next token is '{', parse as quantum block body
                if self.check(&TokenKind::OpenBrace) {
                    self.advance(); // consume '{'
                    let mut body = Vec::new();
                    while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
                        // Peek: if Identifier followed by Colon, parse as field decl
                        let is_field = if let TokenKind::Identifier(_) = self.peek().kind {
                            // Look ahead for ':'
                            self.pos + 1 < self.tokens.len()
                                && matches!(self.tokens[self.pos + 1].kind, TokenKind::Colon)
                        } else {
                            false
                        };
                        // Peek: if Identifier followed by '(', parse as method/constructor decl
                        let is_method = if let TokenKind::Identifier(_) = self.peek().kind {
                            self.pos + 1 < self.tokens.len()
                                && matches!(self.tokens[self.pos + 1].kind, TokenKind::OpenParen)
                        } else {
                            false
                        };
                        if is_field {
                            let line = self.peek().line;
                            let col = self.peek().column;
                            let fname = self.consume_identifier("field name")?;
                            self.consume(TokenKind::Colon, "Expected ':' after field name")?;
                            // Skip type annotation until '=' or ';' or '{'
                            // Type can be complex: QuantumBackend[], Vec<T>, etc.
                            let has_default = self.skip_until_equals_or_semi();
                            let value = if has_default {
                                self.parse_expression()?
                            } else {
                                ASTNode::NullLiteral
                            };
                            let _ = self.match_token(&[TokenKind::Semicolon]);
                            body.push(ASTNode::new_variable_decl_at(&fname, value, line, col));
                        } else if is_method {
                            // Parse constructor(params) { body } or method(params) { body }
                            let line = self.peek().line;
                            let col = self.peek().column;
                            let mname = self.consume_identifier("method name")?;
                            self.consume(TokenKind::OpenParen, "Expected '('")?;
                            let mut params = Vec::new();
                            if !self.check(&TokenKind::CloseParen) {
                                loop {
                                    let pname = self.consume_param_name("Expected parameter name")?;
                                    self.skip_param_type_annotation();
                                    params.push(FunctionParam { name: pname, line, column: col });
                                    if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                                }
                            }
                            self.consume(TokenKind::CloseParen, "Expected ')' after method params")?;
                            if self.match_token(&[TokenKind::Arrow]) { self.skip_type_annotation(); }
                            let mbody = match self.parse_block()? {
                                ASTNode::Block(stmts) => stmts,
                                _ => return Err(self.err_here("Expected method body block")),
                            };
                            body.push(ASTNode::new_function_at(&mname, line, col, params, mbody));
                        } else {
                            body.push(self.parse_statement()?);
                        }
                    }
                    self.consume(TokenKind::CloseBrace, "Expected '}' after quantum block")?;
                    // Emit as a struct with the parsed body as its "constructor" block
                    Ok(ASTNode::Block(body))
                } else {
                    // No brace — treat quantum <tag> as expression
                    let _ = self.match_token(&[TokenKind::Semicolon]);
                    Ok(ASTNode::Identifier(name))
                }
            }
        }
    }

    /// quantum circuit Bell { H(q); CNOT(q, r); measure(q); }
    fn parse_quantum_circuit(&mut self) -> Result<ASTNode, ParserError> {
        let name = if let TokenKind::Identifier(_) = self.peek().kind.clone() {
            self.consume_identifier("Expected circuit name")?  
        } else {
            "anon".to_string()
        };
        self.consume(TokenKind::OpenBrace, "Expected '{' after quantum circuit name")?;
        let mut gates: Vec<ASTNode> = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            gates.push(self.parse_statement()?);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after quantum circuit body")?;
        Ok(ASTNode::QuantumCircuit { name, gates })
    }

    /// struct Foo { field: Type, ... }
    fn parse_struct_decl(&mut self, is_quantum: bool) -> Result<ASTNode, ParserError> {
        if !is_quantum {
            self.consume(TokenKind::Struct, "Expected 'struct'")?;
        }
        let name = self.consume_identifier_or_keyword("Expected struct name")?;
        // Skip optional generic params: `struct Foo<T, U>`
        if self.check(&TokenKind::LessThan) {
            let mut depth = 1usize;
            self.advance(); // consume '<'
            while !self.is_at_end() && depth > 0 {
                match self.peek().kind {
                    TokenKind::LessThan => { depth += 1; self.advance(); }
                    TokenKind::GreaterThan | TokenKind::ShiftRight => { if depth > 1 && matches!(self.tokens[self.pos].kind, TokenKind::ShiftRight) { depth = depth.saturating_sub(2); } else { depth -= 1; } self.advance(); }
                    _ => { self.advance(); }
                }
            }
        }
        self.consume(TokenKind::OpenBrace, "Expected '{' in struct")?;
        let mut fields = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            // Skip `function` method declarations inside structs
            if self.check(&TokenKind::Function) {
                let _ = self.parse_function_decl()?;
                continue;
            }
            // Skip `constructor(...)` / `methodName(...)` method-like declarations
            let is_method = if let TokenKind::Identifier(_) = self.peek().kind {
                self.pos + 1 < self.tokens.len()
                    && matches!(self.tokens[self.pos + 1].kind, TokenKind::OpenParen)
            } else {
                false
            };
            if is_method {
                let line = self.peek().line;
                let col = self.peek().column;
                let mname = self.consume_identifier("method name")?;
                self.consume(TokenKind::OpenParen, "Expected '('")?;
                let mut params = Vec::new();
                if !self.check(&TokenKind::CloseParen) {
                    loop {
                        let pname = self.consume_param_name("Expected parameter name")?;
                        self.skip_param_type_annotation();
                        params.push(FunctionParam { name: pname, line, column: col });
                        if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after method params")?;
                if self.match_token(&[TokenKind::Arrow]) { self.skip_type_annotation(); }
                let _ = self.parse_block()?; // parse and discard method body
                continue;
            }
            let fname = self.consume_identifier("Expected field name")?;
            let ftype = if self.match_token(&[TokenKind::Colon]) {
                self.consume_type_name()
            } else {
                "Any".to_string()
            };
            // Skip optional default value: `field: Type = default`
            if self.match_token(&[TokenKind::Equals]) {
                let _ = self.parse_expression()?;
            }
            fields.push(crate::core::ast::FieldDecl { name: fname, type_name: ftype });
            let _ = self.match_token(&[TokenKind::Comma, TokenKind::Semicolon]);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after struct fields")?;
        Ok(ASTNode::StructDecl { name, fields, is_quantum })
    }

    /// enum Foo { A, B(T), ... }
    fn parse_enum_decl(&mut self, is_quantum: bool) -> Result<ASTNode, ParserError> {
        if !is_quantum {
            self.consume(TokenKind::Enum, "Expected 'enum'")?;
        }
        let name = self.consume_identifier("Expected enum name")?;
        self.consume(TokenKind::OpenBrace, "Expected '{' in enum")?;
        let mut variants = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let vname = self.consume_identifier_or_keyword("Expected variant name")?;
            let payload = if self.match_token(&[TokenKind::OpenParen]) {
                let t = self.consume_type_name();
                // Skip additional comma-separated types (multi-field tuple variants)
                while self.match_token(&[TokenKind::Comma]) {
                    if self.check(&TokenKind::CloseParen) { break; }
                    let _ = self.consume_type_name();
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after variant type")?;
                Some(t)
            } else if self.check(&TokenKind::OpenBrace) {
                // Struct-like enum variant: Variant { field: Type, ... } — skip the body
                let mut depth = 0usize;
                loop {
                    match self.peek().kind {
                        TokenKind::OpenBrace => { depth += 1; self.advance(); }
                        TokenKind::CloseBrace => {
                            if depth == 1 { self.advance(); break; }
                            depth -= 1; self.advance();
                        }
                        TokenKind::EOF => break,
                        _ => { self.advance(); }
                    }
                }
                None
            } else {
                None
            };
            variants.push(crate::core::ast::EnumVariant { name: vname, payload });
            let _ = self.match_token(&[TokenKind::Comma]);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after enum variants")?;
        Ok(ASTNode::EnumDecl { name, variants, is_quantum })
    }

    /// impl Foo { function bar(...) { ... } ... }
    /// Also handles: impl Trait for Type { ... }
    fn parse_impl_block(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Impl, "Expected 'impl'")?;
        // Skip optional generic params: impl<T>
        if self.check(&TokenKind::LessThan) {
            let mut d = 1usize; self.advance();
            while !self.is_at_end() && d > 0 {
                match self.peek().kind {
                    TokenKind::LessThan => { d += 1; self.advance(); }
                    TokenKind::GreaterThan => { d -= 1; self.advance(); }
                    TokenKind::ShiftRight => { d = d.saturating_sub(2); self.advance(); }
                    _ => { self.advance(); }
                }
            }
        }
        // Consume type name (may be a path: std::fmt::Display)
        let target = self.consume_type_name();
        // Skip `for TypeName` clause (impl Trait for Type)
        // `for` may be TokenKind::For (keyword) or Identifier("for")
        let is_for = matches!(self.peek().kind, TokenKind::For)
            || matches!(&self.peek().kind, TokenKind::Identifier(kw) if kw == "for");
        if is_for {
            self.advance(); // consume 'for'
            let _ = self.consume_type_name(); // consume target type
        }
        self.consume(TokenKind::OpenBrace, "Expected '{' in impl block")?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            // allow pub prefix — skip it
            if self.check(&TokenKind::Pub) { self.advance(); }
            if self.check(&TokenKind::Function) {
                methods.push(self.parse_function_decl()?);
            } else if self.check(&TokenKind::Async) {
                methods.push(self.parse_async_function()?);
            } else if self.check(&TokenKind::Quantum) {
                methods.push(self.parse_quantum_keyword_stmt()?);
            } else {
                // skip unknown tokens inside impl to avoid hard crash
                self.advance();
            }
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after impl block")?;
        Ok(ASTNode::ImplBlock { target, methods })
    }

    /// async function foo(...) { ... }
    fn parse_async_function(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Async, "Expected 'async'")?;
        self.consume(TokenKind::Function, "Expected 'function' after 'async'")?;
        let line = self.peek().line;
        let col = self.peek().column;
        let name = self.consume_identifier("Expected async function name")?;
        self.consume(TokenKind::OpenParen, "Expected '('")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::CloseParen) {
            loop {
                let pname = self.consume_param_name("Expected parameter name")?;
                self.skip_param_type_annotation();
                params.push(FunctionParam { name: pname, line, column: col });
                if !self.match_token(&[TokenKind::Comma]) || self.check(&TokenKind::CloseParen) { break; }
            }
        }
        self.consume(TokenKind::CloseParen, "Expected ')'")?;
        if self.match_token(&[TokenKind::Arrow]) { self.skip_type_annotation(); }
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Expected block")),
        };
        Ok(ASTNode::AsyncFunction { name, params, body, line, column: col })
    }

    /// match value { pattern => body, ... }
    fn parse_match_expr(&mut self) -> Result<ASTNode, ParserError> {
        self.consume(TokenKind::Match, "Expected 'match'")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::OpenBrace, "Expected '{' after match value")?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let pattern = self.parse_match_pattern()?;
            // Consume OR alternatives: Pattern | Pattern | Pattern
            while self.check(&TokenKind::Pipe) {
                self.advance(); // consume '|'
                // If next is '=>', this was a trailing pipe — stop
                if self.check(&TokenKind::FatArrow) { break; }
                let _ = self.parse_match_pattern()?; // parse and discard extra patterns
            }
            // Optional match guard: pattern if condition => body
            let guard = if self.match_token(&[TokenKind::If]) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            self.consume(TokenKind::FatArrow, "Expected '=>' after match pattern")?;
            let body = self.parse_statement()?;
            let _ = self.match_token(&[TokenKind::Comma]);
            arms.push(crate::core::ast::MatchArm { pattern, guard, body: Box::new(body) });
        }
        self.consume(TokenKind::CloseBrace, "Expected '}' after match arms")?;
        Ok(ASTNode::MatchExpr { value: Box::new(value), arms })
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, ParserError> {
        match self.peek().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                // Could be EnumVariant::Binding, Variant(binding), or just an identifier
                if self.match_token(&[TokenKind::ColonColon]) {
                    // Consume variant name — may be any identifier or keyword token
                    let variant_name: Option<String> = match self.peek().kind.clone() {
                        TokenKind::Identifier(vn) => { self.advance(); Some(vn) }
                        TokenKind::EOF | TokenKind::FatArrow | TokenKind::Pipe | TokenKind::CloseBrace => None,
                        _ => { let s = self.peek().kind.to_string(); self.advance(); Some(s) }
                    };
                    let variant = variant_name.unwrap_or(name.clone());
                    let binding = if self.check(&TokenKind::OpenParen) {
                        self.advance();
                        // Depth-scan to handle nested patterns like Enum::Variant { .. }
                        if self.check(&TokenKind::CloseParen) {
                            self.advance();
                            None
                        } else {
                            let mut first_name = String::new();
                            let mut depth = 0usize;
                            while !self.is_at_end() {
                                match self.peek().kind.clone() {
                                    TokenKind::CloseParen if depth == 0 => break,
                                    TokenKind::OpenParen | TokenKind::OpenBrace => { depth += 1; self.advance(); }
                                    TokenKind::CloseParen | TokenKind::CloseBrace => { depth -= 1; self.advance(); }
                                    TokenKind::Identifier(ref n) if first_name.is_empty() => {
                                        first_name = n.clone(); self.advance();
                                    }
                                    _ => { self.advance(); }
                                }
                            }
                            self.consume(TokenKind::CloseParen, "Expected ')'")?;
                            if first_name.is_empty() { None } else { Some(first_name) }
                        }
                    } else if self.check(&TokenKind::OpenBrace) {
                        // Struct pattern: Variant::Name { field, .. } — skip the body
                        self.skip_brace_block();
                        None
                    } else { None };
                    Ok(MatchPattern::EnumVariant { name: variant, binding })
                } else if self.check(&TokenKind::OpenParen) {
                    // Tuple-style variant without `::`: Ok(binding), Err(msg), Some(x)
                    // Also handles nested patterns: Statement::Expression(Expression::Gate { .. })
                    self.advance(); // consume '('
                    let binding = if self.check(&TokenKind::CloseParen) {
                        self.advance();
                        None
                    } else {
                        // Scan until matching ')' to skip nested patterns of any depth
                        let mut first_name = String::new();
                        let mut depth = 0usize;
                        while !self.is_at_end() {
                            match self.peek().kind.clone() {
                                TokenKind::CloseParen if depth == 0 => break,
                                TokenKind::OpenParen | TokenKind::OpenBrace => { depth += 1; self.advance(); }
                                TokenKind::CloseParen | TokenKind::CloseBrace => { depth -= 1; self.advance(); }
                                TokenKind::Identifier(ref n) if first_name.is_empty() => {
                                    first_name = n.clone(); self.advance();
                                }
                                _ => { self.advance(); }
                            }
                        }
                        self.consume(TokenKind::CloseParen, "Expected ')'")?;
                        if first_name.is_empty() { None } else { Some(first_name) }
                    };
                    Ok(MatchPattern::EnumVariant { name, binding })
                } else if self.check(&TokenKind::OpenBrace) {
                    // Struct pattern: Name { field, .. } — skip the body
                    self.skip_brace_block();
                    Ok(MatchPattern::Identifier(name))
                } else {
                    Ok(MatchPattern::Identifier(name))
                }
            }
            TokenKind::NumberLiteral(_) | TokenKind::StringLiteral(_) | TokenKind::BooleanLiteral(_) => {
                let node = self.parse_primary()?;
                Ok(MatchPattern::Literal(node))
            }
            TokenKind::OpenParen => {
                // Tuple pattern: (pat1, pat2, ...) or (false, false)
                self.advance(); // consume '('
                let mut depth = 0usize;
                while !self.is_at_end() {
                    match self.peek().kind {
                        TokenKind::CloseParen if depth == 0 => break,
                        TokenKind::OpenParen => { depth += 1; self.advance(); }
                        TokenKind::CloseParen => { depth -= 1; self.advance(); }
                        _ => { self.advance(); }
                    }
                }
                self.consume(TokenKind::CloseParen, "Expected ')' after tuple pattern")?;
                Ok(MatchPattern::Wildcard)
            }
            TokenKind::Star => {
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
            _ => {
                // fallback: treat as wildcard
                self.advance();
                Ok(MatchPattern::Wildcard)
            }
        }
    }

    /// Skip a balanced `{ ... }` block without parsing contents.
    fn skip_brace_block(&mut self) {
        if !self.check(&TokenKind::OpenBrace) { return; }
        let mut depth = 0usize;
        loop {
            match self.peek().kind {
                TokenKind::OpenBrace => { depth += 1; self.advance(); }
                TokenKind::CloseBrace => {
                    depth -= 1;
                    self.advance();
                    if depth == 0 { break; }
                }
                TokenKind::EOF => break,
                _ => { self.advance(); }
            }
        }
    }

    /// Consume a simple type name (identifier, possibly with generics like Vec<T>)
    /// Returns the type as a string — we don't build a full type AST yet.
    fn consume_type_name(&mut self) -> String {
        let mut ty = if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            self.advance();
            name
        } else if let TokenKind::Ampersand = self.peek().kind {
            self.advance();
            // optional `mut` after `&`
            if let TokenKind::Identifier(ref s) = self.peek().kind.clone() {
                if s == "mut" { self.advance(); }
            }
            let inner = self.consume_type_name();
            return format!("&{}", inner);
        } else {
            // Accept keyword tokens as type names (e.g. `usize`, `str`)
            let s = self.peek().kind.to_string().to_string();
            match &self.peek().kind {
                TokenKind::EOF | TokenKind::Semicolon | TokenKind::Comma
                | TokenKind::Equals | TokenKind::CloseParen | TokenKind::CloseBrace => {
                    return "Any".to_string();
                }
                _ => { self.advance(); s }
            }
        };
        // Handle path types: std::time::SystemTime, serde_json::Value, etc.
        while self.check(&TokenKind::ColonColon) {
            self.advance(); // consume '::'
            let seg = match self.peek().kind.clone() {
                TokenKind::Identifier(n) => { self.advance(); n }
                TokenKind::EOF | TokenKind::Semicolon | TokenKind::Comma
                | TokenKind::CloseParen | TokenKind::CloseBrace => break,
                _ => { let s = self.peek().kind.to_string(); self.advance(); s }
            };
            ty.push_str("::");
            ty.push_str(&seg);
        }
        // Generic params: Vec<T>, Option<String>, Result<(), E>
        if self.check(&TokenKind::LessThan) {
            self.advance();
            let mut depth = 1usize;
            ty.push('<');
            while !self.is_at_end() && depth > 0 {
                let tok = self.advance();
                match &tok.kind {
                    TokenKind::LessThan => { depth += 1; ty.push('<'); }
                    TokenKind::GreaterThan => { depth -= 1; if depth > 0 { ty.push('>'); } else { ty.push('>'); } }
                    TokenKind::ShiftRight => { 
                        // `>>` closes two depth levels (e.g. HashMap<K, Vec<V>>)
                        if depth >= 2 { depth -= 2; ty.push_str(">>"); } 
                        else if depth == 1 { depth = 0; ty.push('>'); }
                    }
                    TokenKind::Identifier(n) => ty.push_str(n),
                    TokenKind::Comma => ty.push_str(", "),
                    _ => {}
                }
            }
        }
        // Array type suffix: Token[] or Token[][]
        while self.check(&TokenKind::OpenBracket) {
            self.advance(); // consume '['
            if self.check(&TokenKind::CloseBracket) { self.advance(); } // consume ']'
            ty.push_str("[]");
        }
        ty
    }

    /// Skip type annotation in a let statement: `let x: Type = value`
    /// Stops before `=` without consuming it.
    fn skip_param_type_annotation_until_equals(&mut self) {
        let mut depth = 0usize;
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::Equals if depth == 0 => break,
                TokenKind::Semicolon if depth == 0 => break,
                TokenKind::LessThan => { depth += 1; self.advance(); }
                TokenKind::GreaterThan if depth > 0 => { depth -= 1; self.advance(); }
                TokenKind::ShiftRight if depth > 0 => { depth = depth.saturating_sub(2); self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

    /// Skip optional type annotation on a function parameter: `name: Type[]`
    fn skip_param_type_annotation(&mut self) {
        if self.match_token(&[TokenKind::Colon]) {
            let mut depth = 0usize;
            while !self.is_at_end() {
                match self.peek().kind {
                    TokenKind::Comma if depth == 0 => break,
                    TokenKind::CloseParen if depth == 0 => break,
                    TokenKind::LessThan => { depth += 1; self.advance(); }
                    TokenKind::GreaterThan if depth > 0 => { depth -= 1; self.advance(); }
                    TokenKind::ShiftRight if depth > 0 => { depth = depth.saturating_sub(2); self.advance(); }
                    _ => { self.advance(); }
                }
            }
        }
    }

    /// Skip type annotation tokens until `=` (used for `static NAME: Type = value`)
    fn skip_type_annotation_until_eq(&mut self) {
        let mut depth = 0usize;
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::Equals if depth == 0 => break,
                TokenKind::Semicolon | TokenKind::OpenBrace if depth == 0 => break,
                TokenKind::LessThan => { depth += 1; self.advance(); }
                TokenKind::GreaterThan if depth > 0 => { depth -= 1; self.advance(); }
                TokenKind::ShiftRight if depth > 0 => { depth = depth.saturating_sub(2); self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

    /// Skip type tokens in a field declaration until `=` or `;` or `}`.
    /// Returns true if `=` was found and consumed (meaning a default value follows).
    fn skip_until_equals_or_semi(&mut self) -> bool {
        let mut depth = 0usize; // track <> nesting
        loop {
            match self.peek().kind {
                TokenKind::Equals if depth == 0 => {
                    self.advance(); // consume '='
                    return true;
                }
                TokenKind::Semicolon | TokenKind::CloseBrace if depth == 0 => {
                    return false; // no default value
                }
                TokenKind::LessThan => { depth += 1; self.advance(); }
                TokenKind::GreaterThan if depth > 0 => { depth -= 1; self.advance(); }
                TokenKind::ShiftRight if depth > 0 => { depth = depth.saturating_sub(2); self.advance(); }
                TokenKind::EOF => return false,
                _ => { self.advance(); }
            }
        }
    }

    /// Skip a return type annotation after `->` (e.g., `-> Result<()>`)
    fn skip_type_annotation(&mut self) {
        // consume tokens until we hit '{', ';', or EOF
        let mut depth = 0usize;
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::OpenBrace if depth == 0 => break,
                TokenKind::Semicolon if depth == 0 => break,
                TokenKind::LessThan => { depth += 1; self.advance(); }
                TokenKind::GreaterThan if depth > 0 => { depth -= 1; self.advance(); }
                TokenKind::ShiftRight if depth > 0 => { depth = depth.saturating_sub(2); self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

} // end impl Parser (Phase 1 functions above)

/// Check if a hieroglyphic symbol represents a quantum variable declaration
fn is_quantum_variable_symbol(symbol: &str) -> bool {
    matches!(symbol, 
        "𓀀" |  // Egyptian hieroglyph A001 - quantum variable type 1
        "𓀁" |  // Egyptian hieroglyph A002 - quantum variable type 2
        "𓀂" |  // Egyptian hieroglyph A003 - quantum variable type 3
        "𓀃" |  // Egyptian hieroglyph A004 - quantum variable type 4
        "𓀄" |  // Egyptian hieroglyph A005 - quantum variable type 5
        "𓀅" |  // Egyptian hieroglyph A006 - quantum variable type 6
        "𓀆" |  // Egyptian hieroglyph A007 - quantum variable type 7
        "𓀇" |  // Egyptian hieroglyph A008 - quantum variable type 8
        "𓀈" |  // Egyptian hieroglyph A009 - quantum variable type 9
        "𓀉"    // Egyptian hieroglyph A010 - quantum variable type 10
    )
}
