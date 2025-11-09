//! Intermediate Representation (IR) and Bytecode for Aeonmi
//!
//! This module provides a platform-independent bytecode representation that serves as:
//! - Target for compilation from AST
//! - Input for the Virtual Machine
//! - Basis for optimization passes
//! - Foundation for debugging and profiling

use std::collections::HashMap;
use std::fmt;

/// Bytecode instruction opcodes
#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    // Stack operations
    Push(Value), // Push value onto stack
    Pop,         // Pop value from stack
    Dup,         // Duplicate top stack value
    Swap,        // Swap top two stack values

    // Arithmetic operations
    Add, // Add two numbers
    Sub, // Subtract two numbers
    Mul, // Multiply two numbers
    Div, // Divide two numbers
    Mod, // Modulo operation
    Neg, // Negate number

    // Comparison operations
    Eq, // Equal comparison
    Ne, // Not equal comparison
    Lt, // Less than
    Le, // Less than or equal
    Gt, // Greater than
    Ge, // Greater than or equal

    // Logical operations
    And, // Logical AND
    Or,  // Logical OR
    Not, // Logical NOT

    // Variable operations
    LoadLocal(u32),      // Load local variable
    StoreLocal(u32),     // Store to local variable
    LoadGlobal(String),  // Load global variable
    StoreGlobal(String), // Store to global variable
    LoadField(String),   // Load object field
    StoreField(String),  // Store to object field

    // Control flow
    Jump(u32),               // Unconditional jump
    JumpIfFalse(u32),        // Jump if top stack is false
    JumpIfTrue(u32),         // Jump if top stack is true
    Call(u32),               // Call function with n arguments
    CallMethod(String, u32), // Call method with name and n arguments
    Return,                  // Return from function

    // Object operations
    NewObject(String), // Create new object of type
    NewArray(u32),     // Create array with n elements
    GetIndex,          // Array/object indexing
    SetIndex,          // Array/object index assignment

    // Quantum operations
    QuantumAlloc(u32),              // Allocate n qubits
    QuantumGate(QuantumGate),       // Apply quantum gate
    QuantumMeasure(u32),            // Measure qubit at index
    QuantumEntangle(u32, u32),      // Entangle two qubits
    QuantumCircuit(QuantumCircuit), // Execute quantum circuit

    // I/O operations
    Print,             // Print top stack value
    Input,             // Read input from console
    FileRead(String),  // Read file
    FileWrite(String), // Write to file

    // Advanced operations
    Match(Vec<MatchCase>), // Pattern matching
    NewClosure(u32),       // Create closure with n captured variables
    Throw(String),         // Throw exception
    TryCatch(u32, u32),    // Try-catch block (try_end, catch_end)

    // Debug operations
    Debug(String), // Debug print with message
    Breakpoint,    // Debugger breakpoint

    // No-op
    Nop, // No operation
}

/// Runtime values that can be pushed onto the stack
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Qubit(QubitState),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(FunctionRef),
    Null,
}

/// Quantum gate operations
#[derive(Debug, Clone, PartialEq)]
pub enum QuantumGate {
    H(u32),                             // Hadamard gate on qubit
    X(u32),                             // Pauli-X gate
    Y(u32),                             // Pauli-Y gate
    Z(u32),                             // Pauli-Z gate
    CNOT(u32, u32),                     // CNOT gate (control, target)
    RZ(u32, f64),                       // RZ rotation gate (qubit, angle)
    RY(u32, f64),                       // RY rotation gate
    RX(u32, f64),                       // RX rotation gate
    Phase(u32, f64),                    // Phase gate
    T(u32),                             // T gate
    S(u32),                             // S gate
    Custom(String, Vec<u32>, Vec<f64>), // Custom gate
}

/// Quantum circuit representation
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumCircuit {
    pub qubits: u32,
    pub gates: Vec<QuantumGate>,
    pub measurements: Vec<u32>, // Qubits to measure
}

/// Qubit state representation
#[derive(Debug, Clone, PartialEq)]
pub enum QubitState {
    Zero,
    One,
    Superposition(f64, f64), // (alpha, beta) coefficients
    Entangled(u32),          // Reference to entangled qubit
}

/// Function reference for callable values
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionRef {
    Bytecode(u32),            // Index into function table
    Native(String),           // Native function name
    Closure(u32, Vec<Value>), // Function index + captured variables
}

/// Pattern matching case
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub jump_target: u32,
}

/// Pattern for match expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Value),
    Variable(String),
    Wildcard,
    Guard(Box<Pattern>, u32), // Pattern with guard condition (bytecode index)
}

/// Complete bytecode instruction with metadata
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub line: usize,
    pub column: usize,
    pub file: String,
}

/// Function definition in bytecode
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: u32,  // Number of parameters
    pub locals: u32, // Number of local variables
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>, // Function-local constants
}

/// Complete bytecode program
#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub functions: Vec<Function>,
    pub main_function: u32, // Index of main function
    pub global_constants: Vec<Value>,
    pub string_table: Vec<String>,
    pub metadata: ProgramMetadata,
}

/// Program metadata for debugging and optimization
#[derive(Debug, Clone)]
pub struct ProgramMetadata {
    pub source_files: Vec<String>,
    pub line_map: HashMap<u32, (String, usize)>, // Instruction -> (file, line)
    pub optimization_level: u32,
    pub quantum_required: bool,
}

impl BytecodeProgram {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            main_function: 0,
            global_constants: Vec::new(),
            string_table: Vec::new(),
            metadata: ProgramMetadata {
                source_files: Vec::new(),
                line_map: HashMap::new(),
                optimization_level: 0,
                quantum_required: false,
            },
        }
    }

    pub fn add_function(&mut self, function: Function) -> u32 {
        let index = self.functions.len() as u32;
        self.functions.push(function);
        index
    }

    pub fn add_constant(&mut self, value: Value) -> u32 {
        let index = self.global_constants.len() as u32;
        self.global_constants.push(value);
        index
    }

    pub fn add_string(&mut self, string: String) -> u32 {
        let index = self.string_table.len() as u32;
        self.string_table.push(string);
        index
    }

    pub fn get_function(&self, index: u32) -> Option<&Function> {
        self.functions.get(index as usize)
    }

    pub fn get_main_function(&self) -> Option<&Function> {
        self.get_function(self.main_function)
    }
}

impl Function {
    pub fn new(name: String, arity: u32, locals: u32) -> Self {
        Self {
            name,
            arity,
            locals,
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn emit(&mut self, opcode: Opcode, line: usize, column: usize, file: String) {
        self.instructions.push(Instruction {
            opcode,
            line,
            column,
            file,
        });
    }

    pub fn emit_jump_placeholder(&mut self, line: usize, column: usize, file: String) -> u32 {
        let index = self.instructions.len() as u32;
        self.emit(Opcode::Jump(0), line, column, file); // Placeholder
        index
    }

    pub fn patch_jump(&mut self, jump_index: u32, target: u32) {
        if let Some(instruction) = self.instructions.get_mut(jump_index as usize) {
            match &mut instruction.opcode {
                Opcode::Jump(ref mut addr)
                | Opcode::JumpIfFalse(ref mut addr)
                | Opcode::JumpIfTrue(ref mut addr) => {
                    *addr = target;
                }
                _ => panic!("Attempting to patch non-jump instruction"),
            }
        }
    }

    pub fn current_address(&self) -> u32 {
        self.instructions.len() as u32
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Push(val) => write!(f, "PUSH {:?}", val),
            Opcode::Pop => write!(f, "POP"),
            Opcode::Dup => write!(f, "DUP"),
            Opcode::Swap => write!(f, "SWAP"),
            Opcode::Add => write!(f, "ADD"),
            Opcode::Sub => write!(f, "SUB"),
            Opcode::Mul => write!(f, "MUL"),
            Opcode::Div => write!(f, "DIV"),
            Opcode::Mod => write!(f, "MOD"),
            Opcode::Neg => write!(f, "NEG"),
            Opcode::Eq => write!(f, "EQ"),
            Opcode::Ne => write!(f, "NE"),
            Opcode::Lt => write!(f, "LT"),
            Opcode::Le => write!(f, "LE"),
            Opcode::Gt => write!(f, "GT"),
            Opcode::Ge => write!(f, "GE"),
            Opcode::And => write!(f, "AND"),
            Opcode::Or => write!(f, "OR"),
            Opcode::Not => write!(f, "NOT"),
            Opcode::LoadLocal(idx) => write!(f, "LOAD_LOCAL {}", idx),
            Opcode::StoreLocal(idx) => write!(f, "STORE_LOCAL {}", idx),
            Opcode::LoadGlobal(name) => write!(f, "LOAD_GLOBAL {}", name),
            Opcode::StoreGlobal(name) => write!(f, "STORE_GLOBAL {}", name),
            Opcode::LoadField(name) => write!(f, "LOAD_FIELD {}", name),
            Opcode::StoreField(name) => write!(f, "STORE_FIELD {}", name),
            Opcode::Jump(addr) => write!(f, "JUMP {}", addr),
            Opcode::JumpIfFalse(addr) => write!(f, "JUMP_IF_FALSE {}", addr),
            Opcode::JumpIfTrue(addr) => write!(f, "JUMP_IF_TRUE {}", addr),
            Opcode::Call(argc) => write!(f, "CALL {}", argc),
            Opcode::CallMethod(name, argc) => write!(f, "CALL_METHOD {} {}", name, argc),
            Opcode::Return => write!(f, "RETURN"),
            Opcode::NewObject(type_name) => write!(f, "NEW_OBJECT {}", type_name),
            Opcode::NewArray(size) => write!(f, "NEW_ARRAY {}", size),
            Opcode::GetIndex => write!(f, "GET_INDEX"),
            Opcode::SetIndex => write!(f, "SET_INDEX"),
            Opcode::QuantumAlloc(n) => write!(f, "QUANTUM_ALLOC {}", n),
            Opcode::QuantumGate(gate) => write!(f, "QUANTUM_GATE {:?}", gate),
            Opcode::QuantumMeasure(idx) => write!(f, "QUANTUM_MEASURE {}", idx),
            Opcode::QuantumEntangle(q1, q2) => write!(f, "QUANTUM_ENTANGLE {} {}", q1, q2),
            Opcode::QuantumCircuit(circuit) => write!(f, "QUANTUM_CIRCUIT {:?}", circuit),
            Opcode::Print => write!(f, "PRINT"),
            Opcode::Input => write!(f, "INPUT"),
            Opcode::FileRead(path) => write!(f, "FILE_READ {}", path),
            Opcode::FileWrite(path) => write!(f, "FILE_WRITE {}", path),
            Opcode::Match(cases) => write!(f, "MATCH {} cases", cases.len()),
            Opcode::NewClosure(captures) => write!(f, "NEW_CLOSURE {}", captures),
            Opcode::Throw(msg) => write!(f, "THROW {}", msg),
            Opcode::TryCatch(try_end, catch_end) => {
                write!(f, "TRY_CATCH {} {}", try_end, catch_end)
            }
            Opcode::Debug(msg) => write!(f, "DEBUG {}", msg),
            Opcode::Breakpoint => write!(f, "BREAKPOINT"),
            Opcode::Nop => write!(f, "NOP"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Qubit(state) => write!(f, "|{:?}⟩", state),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
            Value::Function(func_ref) => write!(f, "Function({:?})", func_ref),
            Value::Null => write!(f, "null"),
        }
    }
}

/// Bytecode disassembler for debugging
pub struct Disassembler;

impl Disassembler {
    pub fn disassemble_program(program: &BytecodeProgram) -> String {
        let mut output = String::new();

        output.push_str("=== AEONMI BYTECODE PROGRAM ===\n\n");

        // Metadata
        output.push_str(&format!(
            "Optimization level: {}\n",
            program.metadata.optimization_level
        ));
        output.push_str(&format!(
            "Quantum required: {}\n",
            program.metadata.quantum_required
        ));
        output.push_str(&format!(
            "Source files: {:?}\n\n",
            program.metadata.source_files
        ));

        // Global constants
        if !program.global_constants.is_empty() {
            output.push_str("Global Constants:\n");
            for (i, constant) in program.global_constants.iter().enumerate() {
                output.push_str(&format!("  [{}] {}\n", i, constant));
            }
            output.push_str("\n");
        }

        // Functions
        for (i, function) in program.functions.iter().enumerate() {
            if i == program.main_function as usize {
                output.push_str(&format!("Function {} (MAIN): {}\n", i, function.name));
            } else {
                output.push_str(&format!("Function {}: {}\n", i, function.name));
            }
            output.push_str(&Self::disassemble_function(function));
            output.push_str("\n");
        }

        output
    }

    pub fn disassemble_function(function: &Function) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "  Arity: {}, Locals: {}\n",
            function.arity, function.locals
        ));

        if !function.constants.is_empty() {
            output.push_str("  Constants:\n");
            for (i, constant) in function.constants.iter().enumerate() {
                output.push_str(&format!("    [{}] {}\n", i, constant));
            }
        }

        output.push_str("  Instructions:\n");
        for (i, instruction) in function.instructions.iter().enumerate() {
            output.push_str(&format!(
                "    {:04}: {} ({}:{})\n",
                i, instruction.opcode, instruction.line, instruction.column
            ));
        }

        output
    }
}
