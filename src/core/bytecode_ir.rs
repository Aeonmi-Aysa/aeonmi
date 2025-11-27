//! Intermediate Representation (IR) and Bytecode for Aeonmi
//!
//! This module provides a platform-independent bytecode representation that serves as:
//! - Target for compilation from AST
//! - Input for the Virtual Machine
//! - Basis for optimization passes
//! - Foundation for debugging and profiling

use std::fmt;

/// Bytecode instruction opcodes
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    // Stack operations
    Push(Value), // Push value onto stack

    // Control flow
    Return,                  // Return from function
}

/// Runtime values that can be pushed onto the stack
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Numeric value
    Number(f64),
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Push(val) => write!(f, "PUSH {:?}", val),
            Opcode::Return => write!(f, "RETURN"),
        }
    }
}
