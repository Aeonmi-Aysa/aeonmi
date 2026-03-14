//! Abstract Syntax Tree (AST) definitions for Aeonmi/QUBE/Titan.
//! Includes Assignment and Call nodes to support expression statements.

use crate::core::token::TokenKind;

/// Represents nodes in the Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Many variants used only in experimental passes / future features
pub enum ASTNode {
    // Program root
    Program(Vec<ASTNode>),
    // Declarations
    Function {
        name: String,
        line: usize,
        column: usize,
        params: Vec<FunctionParam>,
        body: Vec<ASTNode>,
    },
    VariableDecl {
        name: String,
        value: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    // Statements / simple stmt-like exprs
    Block(Vec<ASTNode>),
    Return(Box<ASTNode>),
    Log(Box<ASTNode>),
    // Control flow
    If {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    For {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Box<ASTNode>,
    },
    ForIn {
        var: String,
        iterable: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    // Expressions
    Assignment {
        name: String,
        value: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    Call {
        callee: Box<ASTNode>,
        args: Vec<ASTNode>,
    },
    BinaryExpr {
        op: TokenKind,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    UnaryExpr {
        op: TokenKind,
        expr: Box<ASTNode>,
    },
    #[allow(dead_code)]
    Identifier(String),
    IdentifierSpanned { name: String, line: usize, column: usize, len: usize },
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    NullLiteral,
    // Quantum & Hieroglyphic
    QuantumOp {
        op: TokenKind,
        qubits: Vec<ASTNode>,
    },
    HieroglyphicOp {
        symbol: String,
        args: Vec<ASTNode>,
    },
    
    // AEONMI Quantum-Native Constructs
    QuantumArray {
        elements: Vec<ASTNode>,
        dimensions: Option<Vec<usize>>, // Multi-dimensional support
        is_superposition: bool,
    },
    QuantumIndexAccess {
        array: Box<ASTNode>,
        index: Box<ASTNode>,
        is_quantum_index: bool, // true if using ⟦⟧ quantum indexing
    },
    QuantumVariableDecl {
        name: String,
        binding_type: QuantumBindingType,
        value: Box<ASTNode>,
        line: usize,
        column: usize,
    },
    QuantumBinaryExpr {
        op: TokenKind, // ⊕, ⊗, ◊, ∇, etc.
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ProbabilityBranch {
        condition: Box<ASTNode>,
        probability: Option<f64>, // Optional explicit probability
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    QuantumLoop {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
        decoherence_threshold: Option<f64>,
    },
    SuperpositionSwitch {
        value: Box<ASTNode>,
        cases: Vec<SuperpositionCase>,
    },
    QuantumFunction {
        func_type: QuantumFunctionType,
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    QuantumState {
        state: String, // |0⟩, |1⟩, |+⟩, etc.
        amplitude: Option<f64>,
    },
    AILearningBlock {
        data_binding: Option<String>,
        model_binding: Option<String>,
        body: Vec<ASTNode>,
    },
    TimeBlock {
        duration: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
    },
    QuantumTryCatch {
        attempt_body: Vec<ASTNode>,
        error_probability: Option<f64>,
        catch_body: Option<Vec<ASTNode>>,
        success_body: Option<Vec<ASTNode>>,
    },
    
    // Phase 1 constructs
    /// import { X, Y } from "./path"
    ImportDecl {
        names: Vec<String>,   // imported names (empty = wildcard/default)
        path: String,
    },
    /// struct / quantum struct Foo { field: Type, ... }
    StructDecl {
        name: String,
        fields: Vec<FieldDecl>,
        is_quantum: bool,
    },
    /// enum / quantum enum Foo { A, B(T), ... }
    EnumDecl {
        name: String,
        variants: Vec<EnumVariant>,
        is_quantum: bool,
    },
    /// impl Foo { fn method(...) { ... } }
    ImplBlock {
        target: String,
        methods: Vec<ASTNode>, // Function nodes
    },
    /// match value { pattern => expr, ... }
    MatchExpr {
        value: Box<ASTNode>,
        arms: Vec<MatchArm>,
    },
    /// obj.method(args)
    MethodCall {
        object: Box<ASTNode>,
        method: String,
        args: Vec<ASTNode>,
    },
    /// obj.field
    FieldAccess {
        object: Box<ASTNode>,
        field: String,
    },
    /// obj.field = value
    FieldAssign {
        object: Box<ASTNode>,
        field: String,
        value: Box<ASTNode>,
    },
    /// f"text {var} more"
    FStringLiteral(Vec<FStringPart>),
    /// async function foo() { ... }
    AsyncFunction {
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<ASTNode>,
        line: usize,
        column: usize,
    },
    /// await expr
    AwaitExpr(Box<ASTNode>),
    /// Type annotation wrapper (let x: i32 = 0) — stores inner expr
    TypeAnnotated {
        expr: Box<ASTNode>,
        type_name: String,
    },
    /// Array literal [1, 2, 3]
    ArrayLiteral(Vec<ASTNode>),
    
    /// quantum circuit Bell { H(q); CNOT(q, r); measure(q); }
    QuantumCircuit {
        name: String,
        gates: Vec<ASTNode>, // gate calls as statements
    },
    /// Closure: |params| -> { body } or |params| expr
    Closure {
        params: Vec<FunctionParam>,
        body: Vec<ASTNode>,
    },
    
    // Special
    #[allow(dead_code)]
    Error(String),

    // Phase 1.5 — Genesis Glyphs
    /// ⧉expr‥expr‥expr⧉  — glyph array literal
    GlyphArray(Vec<ASTNode>),
    /// …expr  — spread operator
    SpreadExpr(Box<ASTNode>),
    /// expr⟨low‥high⟩  — zero-copy slice
    SliceExpr {
        array: Box<ASTNode>,
        low: Option<Box<ASTNode>>,
        high: Option<Box<ASTNode>>,
    },
    /// name ↦ expr  — binding/projection
    BindingProjection {
        name: String,
        expr: Box<ASTNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<String>, // optional type payload
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub guard: Option<Box<ASTNode>>,  // match guard: pattern if condition => body
    pub body: Box<ASTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Literal(ASTNode),
    Identifier(String),
    Wildcard,
    EnumVariant { name: String, binding: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Literal(String),
    Expr(ASTNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantumBindingType {
    Classical,      // ⟨x⟩ ← value
    Superposition,  // ⟨x⟩ ∈ |0⟩ + |1⟩
    Tensor,         // ⟨x⟩ ⊗ value
    Approximation,  // ⟨x⟩ ≈ value
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantumFunctionType {
    Classical,      // ◯
    Quantum,        // ⊙
    AINeural,       // 🧠
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuperpositionCase {
    pub pattern: String,     // |0⟩, |1⟩, |+⟩, or "*" for wildcard
    pub body: Vec<ASTNode>,
}

impl ASTNode {
    // Utility constructors
    #[allow(dead_code)]
    pub fn new_function(name: &str, params: Vec<&str>, body: Vec<ASTNode>) -> Self {
        Self::Function {
            name: name.to_string(),
            line: 0,
            column: 0,
            params: params.into_iter().map(|p| FunctionParam { name: p.to_string(), line: 0, column: 0 }).collect(),
            body,
        }
    }
    pub fn new_function_at(name: &str, line: usize, column: usize, params: Vec<FunctionParam>, body: Vec<ASTNode>) -> Self {
        Self::Function { name: name.to_string(), line, column, params, body }
    }
    #[allow(dead_code)]
    pub fn new_variable_decl(name: &str, value: ASTNode) -> Self {
        Self::VariableDecl { name: name.to_string(), value: Box::new(value), line: 0, column: 0 }
    }
    pub fn new_variable_decl_at(name: &str, value: ASTNode, line: usize, column: usize) -> Self {
        Self::VariableDecl { name: name.to_string(), value: Box::new(value), line, column }
    }
    
    // AEONMI Quantum-Native Constructors
    pub fn new_quantum_array(elements: Vec<ASTNode>, is_superposition: bool) -> Self {
        Self::QuantumArray {
            elements,
            dimensions: None,
            is_superposition,
        }
    }
    
    pub fn new_quantum_array_multidim(elements: Vec<ASTNode>, dimensions: Vec<usize>) -> Self {
        Self::QuantumArray {
            elements,
            dimensions: Some(dimensions),
            is_superposition: false,
        }
    }
    
    pub fn new_quantum_variable_decl(name: &str, binding_type: QuantumBindingType, value: ASTNode, line: usize, column: usize) -> Self {
        Self::QuantumVariableDecl {
            name: name.to_string(),
            binding_type,
            value: Box::new(value),
            line,
            column,
        }
    }
    
    /// Create a quantum variable declaration from hieroglyphic symbol
    pub fn new_quantum_variable_decl_from_symbol(name: &str, value: ASTNode, symbol: &str, line: usize, column: usize) -> Self {
        let binding_type = match symbol {
            "𓀀" => QuantumBindingType::Classical,      // Basic quantum variable
            "𓀁" => QuantumBindingType::Superposition,  // Superposition state
            "𓀂" => QuantumBindingType::Tensor,         // Tensor product
            "𓀃" => QuantumBindingType::Approximation,  // Quantum approximation
            "𓀄" => QuantumBindingType::Classical,      // Alternative classical
            "𓀅" => QuantumBindingType::Superposition,  // Alternative superposition
            "𓀆" => QuantumBindingType::Tensor,         // Alternative tensor
            "𓀇" => QuantumBindingType::Approximation,  // Alternative approximation
            "𓀈" => QuantumBindingType::Classical,      // Extended classical
            "𓀉" => QuantumBindingType::Superposition,  // Extended superposition
            _ => QuantumBindingType::Classical,         // Default fallback
        };
        
        Self::QuantumVariableDecl {
            name: name.to_string(),
            binding_type,
            value: Box::new(value),
            line,
            column,
        }
    }
    
    pub fn new_quantum_binary_expr(op: TokenKind, left: ASTNode, right: ASTNode) -> Self {
        Self::QuantumBinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    
    pub fn new_probability_branch(condition: ASTNode, probability: Option<f64>, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Self {
        Self::ProbabilityBranch {
            condition: Box::new(condition),
            probability,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
    
    pub fn new_quantum_function(func_type: QuantumFunctionType, name: &str, params: Vec<FunctionParam>, body: Vec<ASTNode>, line: usize, column: usize) -> Self {
        Self::QuantumFunction {
            func_type,
            name: name.to_string(),
            params,
            body,
            line,
            column,
        }
    }
    
    pub fn new_quantum_state(state: &str, amplitude: Option<f64>) -> Self {
        Self::QuantumState {
            state: state.to_string(),
            amplitude,
        }
    }
    
    pub fn new_quantum_index_access(array: ASTNode, index: ASTNode, is_quantum: bool) -> Self {
        Self::QuantumIndexAccess {
            array: Box::new(array),
            index: Box::new(index),
            is_quantum_index: is_quantum,
        }
    }
    #[allow(dead_code)]
    pub fn new_assignment(name: &str, value: ASTNode) -> Self {
        Self::Assignment { name: name.to_string(), value: Box::new(value), line: 0, column: 0 }
    }
    pub fn new_assignment_at(name: &str, value: ASTNode, line: usize, column: usize) -> Self {
        Self::Assignment { name: name.to_string(), value: Box::new(value), line, column }
    }
    pub fn new_call(callee: ASTNode, args: Vec<ASTNode>) -> Self {
        Self::Call {
            callee: Box::new(callee),
            args,
        }
    }
    pub fn new_binary_expr(op: TokenKind, left: ASTNode, right: ASTNode) -> Self {
        Self::BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn new_unary_expr(op: TokenKind, expr: ASTNode) -> Self {
        Self::UnaryExpr {
            op,
            expr: Box::new(expr),
        }
    }
    pub fn new_identifier_spanned(name: &str, line: usize, column: usize, len: usize) -> Self { Self::IdentifierSpanned { name: name.into(), line, column, len } }
    pub fn new_if(cond: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Self {
        Self::If {
            condition: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
    pub fn new_while(cond: ASTNode, body: ASTNode) -> Self {
        Self::While {
            condition: Box::new(cond),
            body: Box::new(body),
        }
    }
    pub fn new_for(
        init: Option<ASTNode>,
        condition: Option<ASTNode>,
        increment: Option<ASTNode>,
        body: ASTNode,
    ) -> Self {
        Self::For {
            init: init.map(Box::new),
            condition: condition.map(Box::new),
            increment: increment.map(Box::new),
            body: Box::new(body),
        }
    }
    pub fn new_log(expr: ASTNode) -> Self {
        Self::Log(Box::new(expr))
    }
    pub fn new_return(expr: ASTNode) -> Self {
        Self::Return(Box::new(expr))
    }
    pub fn new_quantum_op(op: TokenKind, qubits: Vec<ASTNode>) -> Self {
        Self::QuantumOp { op, qubits }
    }
    pub fn new_hieroglyphic_op(symbol: &str, args: Vec<ASTNode>) -> Self {
        Self::HieroglyphicOp {
            symbol: symbol.to_string(),
            args,
        }
    }
}

// Unit tests for ASTNode types – works directly with your TokenKind
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::token::TokenKind;

    #[test]
    fn test_if_else_node() {
        let cond = ASTNode::BooleanLiteral(true);
        let then_b = ASTNode::NumberLiteral(1.0);
        let else_b = ASTNode::NumberLiteral(0.0);
        let node = ASTNode::new_if(cond.clone(), then_b.clone(), Some(else_b.clone()));
        if let ASTNode::If {
            condition,
            then_branch,
            else_branch,
        } = node
        {
            assert_eq!(*condition, cond);
            assert_eq!(*then_branch, then_b);
            assert_eq!(*else_branch.unwrap(), else_b);
        } else {
            panic!("Expected If node");
        }
    }

    #[test]
    fn test_quantum_op_node() {
        let qop = ASTNode::new_quantum_op(TokenKind::Superpose, vec![ASTNode::Identifier("q1".into())]);
        if let ASTNode::QuantumOp { op, qubits } = qop {
            assert_eq!(op, TokenKind::Superpose);
            assert_eq!(qubits[0], ASTNode::Identifier("q1".into()));
        } else {
            panic!("Expected QuantumOp node");
        }
    }

    #[test]
    fn test_assignment_and_call_nodes() {
        let call = ASTNode::new_call(
            ASTNode::Identifier("f".into()),
            vec![ASTNode::NumberLiteral(1.0)],
        );
        let asn = ASTNode::new_assignment("x", call);
    let ASTNode::Assignment { name, value, .. } = asn else {
            panic!("Expected Assignment")
        };
        assert_eq!(name, "x");
    let ASTNode::Call { callee, args } = *value else {
            panic!("Expected Call")
        };
        assert_eq!(*callee, ASTNode::Identifier("f".into()));
        assert_eq!(args[0], ASTNode::NumberLiteral(1.0));
    }
}
