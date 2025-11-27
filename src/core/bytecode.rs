use std::collections::HashMap;

use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

/// An entry in the constant pool for the bytecode chunk.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

/// All opcodes emitted by the bytecode compiler. The VM backend will
/// interpret these instructions in a subsequent change.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    LoadConst(u16),
    Pop,
    LoadLocal(u16),
    StoreLocal(u16),
    LoadGlobal(u16),
    StoreGlobal(u16),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Jump(i16),
    JumpIfFalse(i16),
    JumpIfTrue(i16),
    Call(u16, u8),
    Return,
    Print(u8),
    Len,
    TimeMs,
    Rand,
    Superpose,
    Entangle,
    Measure,
}

/// Basic optimization statistics gathered while compiling.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct OptStats {
    pub const_folds: u32,
    pub chain_folds: u32,
    pub dce_if: u32,
    pub dce_while: u32,
    pub dce_for: u32,
    pub pops_eliminated: u32,
}

/// Metadata describing a compiled function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionInfo {
    pub start: usize,
    pub locals: u16,
    pub name: String,
}

/// Aggregated bytecode result for a compilation unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
    pub functions: Vec<FunctionInfo>,
    pub opt_stats: OptStats,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            opt_stats: OptStats::default(),
        }
    }
}

/// Bytecode compiler that lowers AST nodes into `Chunk` instructions.
pub struct BytecodeCompiler {
    chunk: Chunk,
    global_index: HashMap<String, u16>,
    func_index: HashMap<String, u16>,
    local_map: HashMap<String, u16>,
    next_local_slot: u16,
    scope_stack: Vec<Vec<String>>,
    loop_stack: Vec<LoopContext>, // Track loop breaks/continues for patching
}

/// Context for tracking break/continue jumps in loops
#[derive(Debug, Clone)]
struct LoopContext {
    breaks: Vec<usize>,     // Positions of break jumps to patch later
    continues: Vec<usize>,  // Positions of continue jumps to patch later (for for-loops)
}

impl Default for BytecodeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            global_index: HashMap::new(),
            func_index: HashMap::new(),
            local_map: HashMap::new(),
            next_local_slot: 0,
            scope_stack: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Compile the provided AST and return a filled `Chunk`.
    pub fn compile(mut self, ast: &ASTNode) -> Chunk {
        let items = match ast {
            ASTNode::Program(items) => items.clone(),
            other => vec![other.clone()],
        };

        // Forward declare functions so calls resolve regardless of order.
        for item in &items {
            if let ASTNode::Function { name, .. } = item {
                self.register_function(name.clone());
            }
        }

        for (idx, item) in items.iter().enumerate() {
            let is_last = idx == items.len() - 1;
            self.compile_stmt(item, is_last, true);
        }

        self.chunk.code.push(OpCode::Return);
        self.chunk
    }

    fn register_function(&mut self, name: String) -> u16 {
        if let Some(&idx) = self.func_index.get(&name) {
            return idx;
        }
        let idx = self.chunk.functions.len() as u16;
        self.chunk.functions.push(FunctionInfo {
            start: 0,
            locals: 0,
            name: name.clone(),
        });
        self.func_index.insert(name, idx);
        idx
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    fn exit_scope(&mut self) {
        if let Some(vars) = self.scope_stack.pop() {
            for name in vars {
                self.local_map.remove(&name);
            }
        }
    }

    fn add_constant(&mut self, value: Constant) -> u16 {
        if let Some((idx, _)) = self
            .chunk
            .constants
            .iter()
            .enumerate()
            .find(|(_, existing)| **existing == value)
        {
            idx as u16
        } else {
            let idx = self.chunk.constants.len() as u16;
            self.chunk.constants.push(value);
            idx
        }
    }

    fn add_global(&mut self, name: String) -> u16 {
        if let Some(&idx) = self.global_index.get(&name) {
            idx
        } else {
            let idx = self.global_index.len() as u16;
            self.global_index.insert(name, idx);
            idx
        }
    }

    fn add_local(&mut self, name: String) -> u16 {
        let slot = self.next_local_slot;
        self.next_local_slot += 1;
        self.local_map.insert(name.clone(), slot);
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.push(name);
        }
        slot
    }

    fn compile_stmt(&mut self, node: &ASTNode, _is_last: bool, top_level: bool) {
        match node {
            ASTNode::Program(items) => {
                for (idx, item) in items.iter().enumerate() {
                    let is_last = idx == items.len() - 1;
                    self.compile_stmt(item, is_last, top_level);
                }
            }
            ASTNode::Block(items) => {
                self.enter_scope();
                for item in items {
                    self.compile_stmt(item, false, top_level);
                    if !matches!(item, ASTNode::Function { .. }) {
                        self.chunk.code.push(OpCode::Pop);
                    }
                }
                self.exit_scope();
            }
            ASTNode::QuantumOp { op, qubits } => {
                for qubit in qubits {
                    if let Some(c) = self.compile_expr(qubit) {
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                }

                match op {
                    TokenKind::Superpose => self.chunk.code.push(OpCode::Superpose),
                    TokenKind::Entangle | TokenKind::Dod => {
                        self.chunk.code.push(OpCode::Entangle)
                    }
                    TokenKind::Measure => self.chunk.code.push(OpCode::Measure),
                    _ => {}
                }
            }
            ASTNode::VariableDecl { name, value, .. } => {
                let const_val = self.compile_expr(value.as_ref());
                if top_level {
                    let idx = self.add_global(name.clone());
                    if let Some(c) = const_val {
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                    self.chunk.code.push(OpCode::StoreGlobal(idx));
                } else {
                    let slot = if let Some(&slot) = self.local_map.get(name) {
                        slot
                    } else {
                        self.add_local(name.clone())
                    };
                    if let Some(c) = const_val {
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                    self.chunk.code.push(OpCode::StoreLocal(slot));
                }
                self.chunk.code.push(OpCode::Pop);
            }
            ASTNode::Assignment { target, value, .. } => {
                if let ASTNode::Identifier(name) = &**target {
                    let const_val = self.compile_expr(value.as_ref());
                    if let Some(&slot) = self.local_map.get(name) {
                        if let Some(c) = const_val {
                            let ci = self.add_constant(c);
                            self.chunk.code.push(OpCode::LoadConst(ci));
                        }
                        self.chunk.code.push(OpCode::StoreLocal(slot));
                    } else {
                        let idx = self.add_global(name.clone());
                        if let Some(c) = const_val {
                            let ci = self.add_constant(c);
                            self.chunk.code.push(OpCode::LoadConst(ci));
                        }
                        self.chunk.code.push(OpCode::StoreGlobal(idx));
                    }
                }
                self.chunk.code.push(OpCode::Pop);
            }
            ASTNode::Log(expr) => {
                let const_val = self.compile_expr(expr.as_ref());
                if let Some(c) = const_val {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                self.chunk.code.push(OpCode::Print(1));
                self.chunk.code.push(OpCode::Pop);
            }
            ASTNode::Return(expr) => {
                if let Some(c) = self.compile_expr(expr.as_ref()) {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                self.chunk.code.push(OpCode::Return);
            }
            ASTNode::Break => {
                // Emit a placeholder jump; will be patched when loop ends
                let jump_pos = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.breaks.push(jump_pos);
                }
                // Note: If not in a loop, semantic analysis should have caught this
            }
            ASTNode::Continue => {
                // Emit a placeholder jump; will be patched when loop ends
                let jump_pos = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.continues.push(jump_pos);
                }
                // Note: If not in a loop, semantic analysis should have caught this
            }
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let saved_len = self.chunk.code.len();
                let const_eval = self.compile_expr(condition.as_ref());
                self.chunk.code.truncate(saved_len);

                if let Some(Constant::Bool(c)) = const_eval {
                    if c {
                        self.chunk.opt_stats.dce_if += 1;
                        self.compile_stmt(then_branch.as_ref(), false, top_level);
                    } else {
                        self.chunk.opt_stats.dce_if += 1;
                        if let Some(else_branch) = else_branch {
                            self.compile_stmt(else_branch.as_ref(), false, top_level);
                        }
                    }
                    return;
                }

                let cond_const = self.compile_expr(condition.as_ref());
                if let Some(c) = cond_const {
                    let truthy = self.is_truthy(&c);
                    let ci = self.add_constant(Constant::Bool(truthy));
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }

                let jump_to_else = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);
                self.compile_stmt(then_branch.as_ref(), false, top_level);
                let jump_to_end = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));
                let else_index = self.chunk.code.len();
                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_else] {
                    *off = else_index as i16 - (jump_to_else as i16 + 1);
                }
                self.chunk.code.push(OpCode::Pop);
                if let Some(else_branch) = else_branch {
                    self.compile_stmt(else_branch.as_ref(), false, top_level);
                }
                let end_index = self.chunk.code.len();
                if let OpCode::Jump(ref mut off) = self.chunk.code[jump_to_end] {
                    *off = end_index as i16 - (jump_to_end as i16 + 1);
                }
            }
            ASTNode::ProbabilityBranch {
                probability,
                then_branch,
                else_branch,
                ..
            } => {
                let prob = probability.unwrap_or(0.5).clamp(0.0, 1.0);
                let prob_idx = self.add_constant(Constant::Number(prob));
                self.chunk.code.push(OpCode::Rand);
                self.chunk.code.push(OpCode::LoadConst(prob_idx));
                self.chunk.code.push(OpCode::Lt);

                let jump_to_else = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);
                self.compile_stmt(then_branch.as_ref(), false, top_level);
                let jump_to_end = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));
                let else_index = self.chunk.code.len();
                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_else] {
                    *off = else_index as i16 - (jump_to_else as i16 + 1);
                }
                self.chunk.code.push(OpCode::Pop);
                if let Some(else_branch) = else_branch {
                    self.compile_stmt(else_branch.as_ref(), false, top_level);
                }
                let end_index = self.chunk.code.len();
                if let OpCode::Jump(ref mut off) = self.chunk.code[jump_to_end] {
                    *off = end_index as i16 - (jump_to_end as i16 + 1);
                }
                let null_idx = self.add_constant(Constant::Null);
                self.chunk.code.push(OpCode::LoadConst(null_idx));
            }
            ASTNode::QuantumLoop {
                condition,
                body,
                decoherence_threshold,
            } => {
                self.enter_scope();
                let counter_name = format!("__ql_counter_{}", self.loop_stack.len());
                let counter_slot = self.add_local(counter_name);
                let zero_idx = self.add_constant(Constant::Number(0.0));
                self.chunk.code.push(OpCode::LoadConst(zero_idx));
                self.chunk.code.push(OpCode::StoreLocal(counter_slot));
                self.chunk.code.push(OpCode::Pop);

                let loop_start = self.chunk.code.len();
                self.loop_stack.push(LoopContext {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });

                let threshold_jump = if let Some(threshold) = decoherence_threshold {
                    let threshold_idx = self.add_constant(Constant::Number(*threshold));
                    self.chunk.code.push(OpCode::LoadLocal(counter_slot));
                    self.chunk.code.push(OpCode::LoadConst(threshold_idx));
                    self.chunk.code.push(OpCode::Ge);
                    let jump_idx = self.chunk.code.len();
                    self.chunk.code.push(OpCode::JumpIfTrue(0));
                    self.chunk.code.push(OpCode::Pop);
                    Some(jump_idx)
                } else {
                    None
                };

                if let Some(c) = self.compile_expr(condition.as_ref()) {
                    let truthy = self.is_truthy(&c);
                    let ci = self.add_constant(Constant::Bool(truthy));
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                let jump_to_end = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);

                self.enter_scope();
                self.compile_stmt(body.as_ref(), false, top_level);
                self.exit_scope();

                let continue_target = self.chunk.code.len();
                let one_idx = self.add_constant(Constant::Number(1.0));
                self.chunk.code.push(OpCode::LoadLocal(counter_slot));
                self.chunk.code.push(OpCode::LoadConst(one_idx));
                self.chunk.code.push(OpCode::Add);
                self.chunk.code.push(OpCode::StoreLocal(counter_slot));
                self.chunk.code.push(OpCode::Pop);

                let back_offset = loop_start as i16 - (self.chunk.code.len() as i16 + 1);
                self.chunk.code.push(OpCode::Jump(back_offset));
                let end_index = self.chunk.code.len();
                self.chunk.code.push(OpCode::Pop);

                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_end] {
                    *off = end_index as i16 - (jump_to_end as i16 + 1);
                }
                if let Some(jump_idx) = threshold_jump {
                    if let OpCode::JumpIfTrue(ref mut off) = self.chunk.code[jump_idx] {
                        *off = end_index as i16 - (jump_idx as i16 + 1);
                    }
                }

                if let Some(ctx) = self.loop_stack.pop() {
                    for break_pos in ctx.breaks {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[break_pos] {
                            *off = end_index as i16 - (break_pos as i16 + 1);
                        }
                    }
                    for continue_pos in ctx.continues {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[continue_pos] {
                            *off = continue_target as i16 - (continue_pos as i16 + 1);
                        }
                    }
                }

                self.exit_scope();
                let null_idx = self.add_constant(Constant::Null);
                self.chunk.code.push(OpCode::LoadConst(null_idx));
            }
            ASTNode::QuantumTryCatch {
                attempt_body,
                error_probability,
                catch_body,
                success_body,
            } => {
                self.enter_scope();
                for stmt in attempt_body {
                    self.compile_stmt(stmt, false, top_level);
                    if !matches!(stmt, ASTNode::Function { .. }) {
                        self.chunk.code.push(OpCode::Pop);
                    }
                }
                self.exit_scope();

                let failure_flag = error_probability.unwrap_or(0.0).clamp(0.0, 1.0);
                if failure_flag > 0.0 {
                    let prob_idx = self.add_constant(Constant::Number(failure_flag));
                    self.chunk.code.push(OpCode::Rand);
                    self.chunk.code.push(OpCode::LoadConst(prob_idx));
                    self.chunk.code.push(OpCode::Lt);
                } else {
                    let false_idx = self.add_constant(Constant::Bool(false));
                    self.chunk.code.push(OpCode::LoadConst(false_idx));
                }

                let jump_to_success = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);

                if let Some(catch_body) = catch_body {
                    self.enter_scope();
                    for stmt in catch_body {
                        self.compile_stmt(stmt, false, top_level);
                        if !matches!(stmt, ASTNode::Function { .. }) {
                            self.chunk.code.push(OpCode::Pop);
                        }
                    }
                    self.exit_scope();
                }

                let jump_over_success = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));

                let success_index = self.chunk.code.len();
                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_success] {
                    *off = success_index as i16 - (jump_to_success as i16 + 1);
                }
                self.chunk.code.push(OpCode::Pop);

                if let Some(success_body) = success_body {
                    self.enter_scope();
                    for stmt in success_body {
                        self.compile_stmt(stmt, false, top_level);
                        if !matches!(stmt, ASTNode::Function { .. }) {
                            self.chunk.code.push(OpCode::Pop);
                        }
                    }
                    self.exit_scope();
                }

                let end_index = self.chunk.code.len();
                if let OpCode::Jump(ref mut off) = self.chunk.code[jump_over_success] {
                    *off = end_index as i16 - (jump_over_success as i16 + 1);
                }

                let null_idx = self.add_constant(Constant::Null);
                self.chunk.code.push(OpCode::LoadConst(null_idx));
            }
            ASTNode::While { condition, body } => {
                let saved_len = self.chunk.code.len();
                let const_eval = self.compile_expr(condition.as_ref());
                self.chunk.code.truncate(saved_len);

                if let Some(Constant::Bool(false)) = const_eval {
                    self.chunk.opt_stats.dce_while += 1;
                    return;
                }

                let loop_start = self.chunk.code.len();
                
                // Push loop context for break/continue tracking
                self.loop_stack.push(LoopContext {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                
                let cond_const = self.compile_expr(condition.as_ref());
                if let Some(c) = cond_const {
                    let truthy = self.is_truthy(&c);
                    let ci = self.add_constant(Constant::Bool(truthy));
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                let jump_to_end = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);
                self.enter_scope();
                self.compile_stmt(body.as_ref(), false, top_level);
                self.exit_scope();
                let back_offset = loop_start as i16 - (self.chunk.code.len() as i16 + 1);
                self.chunk.code.push(OpCode::Jump(back_offset));
                let end_index = self.chunk.code.len();
                self.chunk.code.push(OpCode::Pop);
                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_end] {
                    *off = end_index as i16 - (jump_to_end as i16 + 1);
                }
                
                // Patch all break/continue jumps
                if let Some(ctx) = self.loop_stack.pop() {
                    // Patch all break jumps to jump to end_index
                    for break_pos in ctx.breaks {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[break_pos] {
                            *off = end_index as i16 - (break_pos as i16 + 1);
                        }
                    }
                    // Patch all continue jumps to jump back to loop_start
                    for continue_pos in ctx.continues {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[continue_pos] {
                            *off = loop_start as i16 - (continue_pos as i16 + 1);
                        }
                    }
                }
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                self.enter_scope();
                if let Some(init) = init {
                    self.compile_stmt(init.as_ref(), false, top_level);
                    self.chunk.code.push(OpCode::Pop);
                }
                let loop_start = self.chunk.code.len();
                
                // Push loop context - we'll update continue target later
                self.loop_stack.push(LoopContext {
                    breaks: Vec::new(),
                    continues: Vec::new(),
                });
                
                if let Some(cond) = condition {
                    let saved_len = self.chunk.code.len();
                    let const_eval = self.compile_expr(cond.as_ref());
                    self.chunk.code.truncate(saved_len);

                    if let Some(Constant::Bool(false)) = const_eval {
                        self.chunk.opt_stats.dce_for += 1;
                        self.loop_stack.pop(); // Clean up loop context
                        self.exit_scope();
                        return;
                    }
                    if let Some(c) = self.compile_expr(cond.as_ref()) {
                        let truthy = self.is_truthy(&c);
                        let ci = self.add_constant(Constant::Bool(truthy));
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                } else {
                    let ci = self.add_constant(Constant::Bool(true));
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                let jump_to_end = self.chunk.code.len();
                self.chunk.code.push(OpCode::JumpIfFalse(0));
                self.chunk.code.push(OpCode::Pop);
                self.compile_stmt(body.as_ref(), false, top_level);
                
                // Continue should jump here (before increment)
                let continue_target = self.chunk.code.len();
                
                if let Some(incr) = increment {
                    self.compile_expr(incr.as_ref());
                    self.chunk.code.push(OpCode::Pop);
                }
                let back_offset = loop_start as i16 - (self.chunk.code.len() as i16 + 1);
                self.chunk.code.push(OpCode::Jump(back_offset));
                let end_index = self.chunk.code.len();
                self.chunk.code.push(OpCode::Pop);
                if let OpCode::JumpIfFalse(ref mut off) = self.chunk.code[jump_to_end] {
                    *off = end_index as i16 - (jump_to_end as i16 + 1);
                }
                
                // Patch all break/continue jumps
                if let Some(ctx) = self.loop_stack.pop() {
                    // Patch all break jumps to jump to end_index
                    for break_pos in ctx.breaks {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[break_pos] {
                            *off = end_index as i16 - (break_pos as i16 + 1);
                        }
                    }
                    // Patch all continue jumps to jump to continue_target (increment)
                    for continue_pos in ctx.continues {
                        if let OpCode::Jump(ref mut off) = self.chunk.code[continue_pos] {
                            *off = continue_target as i16 - (continue_pos as i16 + 1);
                        }
                    }
                }
                
                self.exit_scope();
            }
            ASTNode::ForIn { iterable, body, .. } => {
                if let Some(c) = self.compile_expr(iterable.as_ref()) {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                self.chunk.code.push(OpCode::Pop);
                self.enter_scope();
                self.compile_stmt(body.as_ref(), false, top_level);
                self.exit_scope();
            }
            ASTNode::Function {
                name, params, body, ..
            } => {
                let skip_jump_idx = self.chunk.code.len();
                self.chunk.code.push(OpCode::Jump(0));

                let f_idx = if let Some(&idx) = self.func_index.get(name) {
                    idx
                } else {
                    self.register_function(name.clone())
                };

                let saved_local = std::mem::take(&mut self.local_map);
                let saved_slot = self.next_local_slot;
                let saved_scope = std::mem::take(&mut self.scope_stack);

                self.enter_scope();
                self.next_local_slot = 0;
                for param in params {
                    self.add_local(param.name.clone());
                }

                let func_start = self.chunk.code.len();
                for stmt in body {
                    self.compile_stmt(stmt, false, false);
                }
                let null_idx = self.add_constant(Constant::Null);
                self.chunk.code.push(OpCode::LoadConst(null_idx));
                self.chunk.code.push(OpCode::Return);

                let locals = self.next_local_slot.max(1);
                if let Some(info) = self.chunk.functions.get_mut(f_idx as usize) {
                    info.start = func_start;
                    info.locals = locals;
                    info.name = name.clone();
                }

                self.scope_stack = saved_scope;
                self.local_map = saved_local;
                self.next_local_slot = saved_slot;

                let after_func = self.chunk.code.len();
                if let OpCode::Jump(ref mut offset) = self.chunk.code[skip_jump_idx] {
                    *offset = after_func as i16 - (skip_jump_idx as i16 + 1);
                }
            }
            ASTNode::StructDecl { name, .. } => {
                // For bytecode: register struct as a "constructor" function
                // Skip the actual struct definition in bytecode (it's a compile-time construct)
                // Store the struct name in globals so it can be referenced
                let idx = self.add_global(name.clone());
                // Create a simple marker constant
                let const_idx = self.add_constant(Constant::String(format!("struct:{}", name)));
                self.chunk.code.push(OpCode::LoadConst(const_idx));
                self.chunk.code.push(OpCode::StoreGlobal(idx));
                self.chunk.code.push(OpCode::Pop);
                
                // TODO: In future, could compile a constructor function that creates objects
                // For now, the JS backend handles struct instantiation
            }
            ASTNode::EnumDecl { name, variants, .. } => {
                // For bytecode: register enum variants as globals
                // Each variant gets a unique constant value
                for variant in variants {
                    let full_name = format!("{}.{}", name, variant);
                    let idx = self.add_global(full_name.clone());
                    let const_idx = self.add_constant(Constant::String(format!("enum:{}:{}", name, variant)));
                    self.chunk.code.push(OpCode::LoadConst(const_idx));
                    self.chunk.code.push(OpCode::StoreGlobal(idx));
                    self.chunk.code.push(OpCode::Pop);
                }
            }
            ASTNode::Call { callee, args } => {
                if let Some(func_name) = self.extract_ident(callee.as_ref()) {
                    if let Some(&idx) = self.func_index.get(&func_name) {
                        for arg in args {
                            if let Some(c) = self.compile_expr(arg) {
                                let ci = self.add_constant(c);
                                self.chunk.code.push(OpCode::LoadConst(ci));
                            }
                        }
                        self.chunk.code.push(OpCode::Call(idx, args.len() as u8));
                        return;
                    }
                    match func_name.as_str() {
                        "log" | "print" => {
                            for arg in args {
                                if let Some(c) = self.compile_expr(arg) {
                                    let ci = self.add_constant(c);
                                    self.chunk.code.push(OpCode::LoadConst(ci));
                                }
                            }
                            self.chunk.code.push(OpCode::Print(args.len() as u8));
                            self.chunk.code.push(OpCode::Pop);
                            return;
                        }
                        "len" => {
                            if let Some(arg) = args.first() {
                                if let Some(c) = self.compile_expr(arg) {
                                    let ci = self.add_constant(c);
                                    self.chunk.code.push(OpCode::LoadConst(ci));
                                }
                                self.chunk.code.push(OpCode::Len);
                            }
                            return;
                        }
                        "time_ms" => {
                            self.chunk.code.push(OpCode::TimeMs);
                            return;
                        }
                        "rand" => {
                            self.chunk.code.push(OpCode::Rand);
                            return;
                        }
                        "superpose" => {
                            for arg in args {
                                if let Some(c) = self.compile_expr(arg) {
                                    let ci = self.add_constant(c);
                                    self.chunk.code.push(OpCode::LoadConst(ci));
                                }
                            }
                            self.chunk.code.push(OpCode::Superpose);
                            return;
                        }
                        "entangle" => {
                            for arg in args {
                                if let Some(c) = self.compile_expr(arg) {
                                    let ci = self.add_constant(c);
                                    self.chunk.code.push(OpCode::LoadConst(ci));
                                }
                            }
                            self.chunk.code.push(OpCode::Entangle);
                            return;
                        }
                        "measure" => {
                            for arg in args {
                                if let Some(c) = self.compile_expr(arg) {
                                    let ci = self.add_constant(c);
                                    self.chunk.code.push(OpCode::LoadConst(ci));
                                }
                            }
                            self.chunk.code.push(OpCode::Measure);
                            return;
                        }
                        _ => {}
                    }
                }
                if let Some(c) = self.compile_expr(callee.as_ref()) {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                self.chunk.code.push(OpCode::Pop);
                for arg in args {
                    if let Some(c) = self.compile_expr(arg) {
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                    self.chunk.code.push(OpCode::Pop);
                }
                let ci = self.add_constant(Constant::Null);
                self.chunk.code.push(OpCode::LoadConst(ci));
            }
            ASTNode::NumberLiteral(n) => {
                let ci = self.add_constant(Constant::Number(*n));
                self.chunk.code.push(OpCode::LoadConst(ci));
            }
            ASTNode::StringLiteral(s) => {
                let ci = self.add_constant(Constant::String(s.clone()));
                self.chunk.code.push(OpCode::LoadConst(ci));
            }
            ASTNode::BooleanLiteral(b) => {
                let ci = self.add_constant(Constant::Bool(*b));
                self.chunk.code.push(OpCode::LoadConst(ci));
            }
            ASTNode::Identifier(name) => {
                self.load_identifier(name);
            }
            ASTNode::IdentifierSpanned { name, .. } => {
                self.load_identifier(name);
            }
            ASTNode::BinaryExpr { .. } | ASTNode::UnaryExpr { .. } => {
                let const_val = self.compile_expr(node);
                if let Some(c) = const_val {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
            }
            _ => {}
        }
    }

    fn load_identifier(&mut self, name: &str) {
        if let Some(&slot) = self.local_map.get(name) {
            self.chunk.code.push(OpCode::LoadLocal(slot));
        } else {
            let idx = self.add_global(name.to_string());
            self.chunk.code.push(OpCode::LoadGlobal(idx));
        }
    }

    fn compile_expr(&mut self, node: &ASTNode) -> Option<Constant> {
        match node {
            ASTNode::NumberLiteral(n) => Some(Constant::Number(*n)),
            ASTNode::StringLiteral(s) => Some(Constant::String(s.clone())),
            ASTNode::BooleanLiteral(b) => Some(Constant::Bool(*b)),
            ASTNode::Identifier(name) => {
                self.load_identifier(name);
                None
            }
            ASTNode::IdentifierSpanned { name, .. } => {
                self.load_identifier(name);
                None
            }
            ASTNode::BinaryExpr { op, left, right } => {
                let op_str = match op {
                    TokenKind::Plus => "+",
                    TokenKind::Minus => "-",
                    TokenKind::Star => "*",
                    TokenKind::Slash => "/",
                    TokenKind::DoubleEquals => "==",
                    TokenKind::NotEquals => "!=",
                    TokenKind::LessThan => "<",
                    TokenKind::LessEqual => "<=",
                    TokenKind::GreaterThan => ">",
                    TokenKind::GreaterEqual => ">=",
                    TokenKind::AndAnd => "&&",
                    TokenKind::OrOr => "||",
                    _ => "",
                };

                if op_str == "&&" || op_str == "||" {
                    let left_const = self.compile_expr(left.as_ref());
                    if let Some(c) = left_const.clone() {
                        let left_truthy = self.is_truthy(&c);
                        if op_str == "&&" {
                            if !left_truthy {
                                self.chunk.opt_stats.dce_if += 1;
                                return Some(Constant::Bool(false));
                            }
                        } else if left_truthy {
                            self.chunk.opt_stats.dce_if += 1;
                            return Some(Constant::Bool(true));
                        }
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }

                    let jump_index = self.chunk.code.len();
                    if op_str == "&&" {
                        self.chunk.code.push(OpCode::JumpIfFalse(0));
                    } else {
                        self.chunk.code.push(OpCode::JumpIfTrue(0));
                    }
                    self.chunk.code.push(OpCode::Pop);

                    let right_const = self.compile_expr(right.as_ref());
                    if let Some(c) = right_const {
                        let ci = self.add_constant(c);
                        self.chunk.code.push(OpCode::LoadConst(ci));
                    }
                    let end_index = self.chunk.code.len();
                    match self.chunk.code.get_mut(jump_index) {
                        Some(OpCode::JumpIfFalse(ref mut offset))
                        | Some(OpCode::JumpIfTrue(ref mut offset)) => {
                            *offset = end_index as i16 - (jump_index as i16 + 1);
                        }
                        _ => {}
                    }
                    return None;
                }

                let left_const = self.compile_expr(left.as_ref());
                let right_const = self.compile_expr(right.as_ref());

                if let (Some(lc), Some(rc)) = (&left_const, &right_const) {
                    if let Some(folded) = self.fold_binary(op_str, lc, rc) {
                        self.chunk.opt_stats.const_folds += 1;
                        if matches!(left.as_ref(), ASTNode::BinaryExpr { .. }) {
                            self.chunk.opt_stats.chain_folds += 1;
                        }
                        if matches!(right.as_ref(), ASTNode::BinaryExpr { .. }) {
                            self.chunk.opt_stats.chain_folds += 1;
                        }
                        return Some(folded);
                    }
                }

                if let Some(c) = left_const {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }
                if let Some(c) = right_const {
                    let ci = self.add_constant(c);
                    self.chunk.code.push(OpCode::LoadConst(ci));
                }

                match op_str {
                    "+" => self.chunk.code.push(OpCode::Add),
                    "-" => self.chunk.code.push(OpCode::Sub),
                    "*" => self.chunk.code.push(OpCode::Mul),
                    "/" => self.chunk.code.push(OpCode::Div),
                    "==" => self.chunk.code.push(OpCode::Eq),
                    "!=" => self.chunk.code.push(OpCode::Ne),
                    "<" => self.chunk.code.push(OpCode::Lt),
                    "<=" => self.chunk.code.push(OpCode::Le),
                    ">" => self.chunk.code.push(OpCode::Gt),
                    ">=" => self.chunk.code.push(OpCode::Ge),
                    _ => {}
                }

                None
            }
            ASTNode::UnaryExpr { op, expr } => {
                let op_str = match op {
                    TokenKind::Minus => "-",
                    TokenKind::QuantumNot => "!",
                    _ => "",
                };

                let const_val = self.compile_expr(expr.as_ref());
                if let Some(c) = const_val {
                    return match (op_str, &c) {
                        ("-", Constant::Number(n)) => Some(Constant::Number(-n)),
                        ("!", Constant::Bool(b)) => Some(Constant::Bool(!b)),
                        _ => Some(c),
                    };
                }

                match op_str {
                    "-" => {
                        let ci = self.add_constant(Constant::Number(-1.0));
                        self.chunk.code.push(OpCode::LoadConst(ci));
                        self.chunk.code.push(OpCode::Mul);
                    }
                    "!" => {
                        let ci = self.add_constant(Constant::Bool(false));
                        self.chunk.code.push(OpCode::LoadConst(ci));
                        self.chunk.code.push(OpCode::Eq);
                    }
                    _ => {}
                }
                None
            }
            ASTNode::Call { .. }
            | ASTNode::Assignment { .. }
            | ASTNode::VariableDecl { .. }
            | ASTNode::Block(_)
            | ASTNode::Program(_)
            | ASTNode::If { .. }
            | ASTNode::While { .. }
            | ASTNode::For { .. }
            | ASTNode::ForIn { .. }
            | ASTNode::Function { .. }
            | ASTNode::Log(_)
            | ASTNode::Return(_) => {
                self.compile_stmt(node, false, false);
                None
            }
            _ => None,
        }
    }

    fn fold_binary(&self, op: &str, left: &Constant, right: &Constant) -> Option<Constant> {
        use Constant::*;
        match op {
            "+" => match (left, right) {
                (Number(a), Number(b)) => Some(Number(a + b)),
                (String(a), String(b)) => Some(String(format!("{}{}", a, b))),
                (String(a), other) => Some(String(format!("{}{}", a, self.display_const(other)))),
                (other, String(b)) => Some(String(format!("{}{}", self.display_const(other), b))),
                _ => None,
            },
            "-" => match (left, right) {
                (Number(a), Number(b)) => Some(Number(a - b)),
                _ => None,
            },
            "*" => match (left, right) {
                (Number(a), Number(b)) => Some(Number(a * b)),
                _ => None,
            },
            "/" => match (left, right) {
                (Number(a), Number(b)) => Some(Number(a / b)),
                _ => None,
            },
            "==" | "!=" | "<" | "<=" | ">" | ">=" => {
                Some(Constant::Bool(self.cmp_constants(left, right, op)))
            }
            _ => None,
        }
    }

    fn display_const(&self, c: &Constant) -> String {
        match c {
            Constant::Number(n) => n.to_string(),
            Constant::String(s) => s.clone(),
            Constant::Bool(b) => b.to_string(),
            Constant::Null => "null".into(),
        }
    }

    fn cmp_constants(&self, left: &Constant, right: &Constant, op: &str) -> bool {
        use Constant::*;
        match (left, right) {
            (Number(a), Number(b)) => match op {
                "==" => a == b,
                "!=" => a != b,
                "<" => a < b,
                "<=" => a <= b,
                ">" => a > b,
                ">=" => a >= b,
                _ => false,
            },
            (Bool(a), Bool(b)) => match op {
                "==" => a == b,
                "!=" => a != b,
                "<" => (*a as i32) < (*b as i32),
                "<=" => (*a as i32) <= (*b as i32),
                ">" => (*a as i32) > (*b as i32),
                ">=" => (*a as i32) >= (*b as i32),
                _ => false,
            },
            (String(a), String(b)) => match op {
                "==" => a == b,
                "!=" => a != b,
                "<" => a < b,
                "<=" => a <= b,
                ">" => a > b,
                ">=" => a >= b,
                _ => false,
            },
            _ => match op {
                "==" => false,
                "!=" => true,
                _ => false,
            },
        }
    }

    fn is_truthy(&self, value: &Constant) -> bool {
        match value {
            Constant::Null => false,
            Constant::Bool(b) => *b,
            Constant::Number(n) => *n != 0.0,
            Constant::String(s) => !s.is_empty(),
        }
    }

    fn extract_ident(&self, node: &ASTNode) -> Option<String> {
        match node {
            ASTNode::Identifier(name) => Some(name.clone()),
            ASTNode::IdentifierSpanned { name, .. } => Some(name.clone()),
            _ => None,
        }
    }
}

/// Produce a debug disassembly of the chunk for compiler-oriented tests.
pub fn disassemble(chunk: &Chunk) -> String {
    let mut out = String::new();
    out.push_str("== Bytecode Disassembly ==\n");
    for (i, instr) in chunk.code.iter().enumerate() {
        out.push_str(&format!("{:04} {:?}\n", i, instr));
    }
    out.push_str("Constants:\n");
    for (i, constant) in chunk.constants.iter().enumerate() {
        out.push_str(&format!("  [{}] {:?}\n", i, constant));
    }
    out.push_str("Functions:\n");
    for (i, info) in chunk.functions.iter().enumerate() {
        out.push_str(&format!(
            "  [{}] start={} locals={} name={}\n",
            i, info.start, info.locals, info.name
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lexer::Lexer;
    use crate::core::parser::Parser;

    fn compile_source(src: &str) -> Chunk {
        let mut lexer = Lexer::new(src, false);
        let tokens = lexer.tokenize().expect("lexing failed");
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().expect("parse failed");
        BytecodeCompiler::new().compile(&ast)
    }

    #[test]
    fn arithmetic_const_folding() {
        let chunk = compile_source("let x = 2 * (3 + 4);\n");
        assert!(chunk.opt_stats.const_folds >= 1);
        assert!(chunk.opt_stats.chain_folds >= 1);
        let constants = chunk.constants;
        assert!(constants
            .iter()
            .any(|c| matches!(c, Constant::Number(14.0))));
    }

    #[test]
    fn dead_code_elimination_if() {
        let chunk = compile_source("if (false) {\n  log(1);\n} else {\n  log(2);\n}\n");
        assert!(chunk.opt_stats.dce_if >= 1);
    }

    #[test]
    fn dead_code_elimination_while() {
        let chunk = compile_source("while (false) { log(1); }\n");
        assert!(chunk.opt_stats.dce_while >= 1);
    }

    #[test]
    fn disassembly_lists_functions() {
        let chunk = compile_source("function add(a, b) {\n  return a + b;\n}\nadd(1, 2);\n");
        let text = disassemble(&chunk);
        assert!(text.contains("add"));
        assert!(text.contains("Call"));
    }
}
