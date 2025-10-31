//! Parser for Aeonmi/QUBE/Titan with precedence parsing + spanned errors.

use crate::core::ast::{ASTNode, FunctionParam, QuantumBindingType, QuantumFunctionType};
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
        let line = self.peek().line;
        let column = self.peek().column;
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(TokenKind::Equals, "Expected '=' in variable declaration")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration")?;
        Ok(ASTNode::new_variable_decl_at(&name, value, line, column))
    }

    fn parse_function_decl(&mut self) -> Result<ASTNode, ParserError> {
    let func_tok = self.consume(TokenKind::Function, "Expected 'function'")?;
    let func_line = func_tok.line; let func_col = func_tok.column;
    let name = self.consume_identifier("Expected function name")?;
    self.consume(TokenKind::OpenParen, "Expected '(' after function name")?;
        let mut params: Vec<FunctionParam> = Vec::new();
        if !self.check(&TokenKind::CloseParen) {
            loop {
                let pname = self.consume_identifier("Expected parameter name")?;
                // For now, param spans reuse function token line/col (could refine with lexer spans)
                params.push(FunctionParam { name: pname, line: func_line, column: func_col });
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenKind::CloseParen, "Expected ')' after parameters")?;
        let body = match self.parse_block()? {
            ASTNode::Block(stmts) => stmts,
            _ => return Err(self.err_here("Function body must be a block")),
        };
    Ok(ASTNode::new_function_at(&name, func_line, func_col, params, body))
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
        self.consume(TokenKind::OpenParen, "Expected '(' after for")?;
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
        let increment = if !self.check(&TokenKind::CloseParen) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(TokenKind::CloseParen, "Expected ')' after for clauses")?;
        let body = self.parse_statement()?;
        Ok(ASTNode::new_for(init, condition, increment, body))
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

    // assignment: Identifier '=' assignment | equality
    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        let expr = self.parse_equality()?;
        if self.match_token(&[TokenKind::Equals]) {
            match expr {
                ASTNode::Identifier(name) => {
                    let line = self.previous().line; let column = self.previous().column; let value = self.parse_assignment()?; Ok(ASTNode::new_assignment_at(&name, value, line, column))
                }
                ASTNode::IdentifierSpanned { name, line: id_line, column: id_col, .. } => {
                    let value = self.parse_assignment()?; Ok(ASTNode::new_assignment_at(&name, value, id_line, id_col))
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
        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = self.previous().kind.clone();
            let right = self.parse_factor()?;
            expr = ASTNode::new_binary_expr(op, expr, right);
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_unary()?;
        while self.match_token(&[TokenKind::Star, TokenKind::Slash]) {
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
        self.parse_call()
    }

    // support simple calls: primary ('(' args? ')')*
    fn parse_call(&mut self) -> Result<ASTNode, ParserError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_token(&[TokenKind::OpenParen]) {
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
            
            // Traditional parentheses
            TokenKind::OpenParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenKind::CloseParen, "Expected ')'")?;
                Ok(expr)
            }
            
            // AEONMI Quantum-Native Constructs
            
            // Quantum arrays: [element1, element2, ...] (using traditional brackets for now)
            TokenKind::OpenBrace => {
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
            
            _ => Err(ParserError {
                message: format!("Unexpected token {:?}", tok.kind),
                line: tok.line,
                column: tok.column,
            }),
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
                let pname = self.consume_identifier("Expected parameter name")?;
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
}

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
