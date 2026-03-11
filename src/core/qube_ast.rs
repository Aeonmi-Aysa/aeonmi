//! QUBE AST nodes — Quantum Universal Base Engine

#[derive(Debug, Clone)]
pub enum QubeNode {
    Program(Vec<QubeNode>),
    StateDecl {
        name: String,
        value: QubeExpr,
    },
    GateApply {
        gate: String,
        targets: Vec<String>,
        angle: Option<f64>,
    },
    Collapse {
        state: String,
        result: String,
    },
    Assert {
        state: String,
        op: AssertOp,
        values: Vec<QubeExpr>,
    },
    Log(QubeExpr),
}

#[derive(Debug, Clone)]
pub enum AssertOp {
    In,   // ∈ {set}
    Eq,   // == value
}

#[derive(Debug, Clone)]
pub enum QubeExpr {
    Ident(String),
    Number(f64),
    QubitLiteral(String),
    Superposition(Vec<SuperpositionTerm>),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct SuperpositionTerm {
    pub amplitude: f64,
    pub state: String,  // e.g. "|0⟩"
}
