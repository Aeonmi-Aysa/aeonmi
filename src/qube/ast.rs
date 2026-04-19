//! QUBE AST — node types for .qube programs.

/// A parsed QUBE program is a list of statements.
#[derive(Debug, Clone)]
pub struct QubeProgram {
    pub stmts: Vec<QubeStmt>,
}

/// A single QUBE statement.
#[derive(Debug, Clone)]
pub enum QubeStmt {
    /// `state <name> = <expr>`
    StateDecl {
        name: String,
        value: QuantumStateExpr,
    },
    /// `apply <gate> → <target>` or `apply <gate> → (<q1>, <q2>)`
    GateApply {
        gate: QuantumGate,
        targets: Vec<String>,
    },
    /// `collapse <qubit> → <result_var>`
    Collapse {
        qubit: String,
        result: String,
    },
    /// `assert <expr> ∈ {values...}`
    Assert {
        variable: String,
        valid_values: Vec<AssertValue>,
    },
    /// `print <variable>`
    Print {
        variable: String,
    },
    /// `let <name> = <expr>` — classical variable
    LetBinding {
        name: String,
        value: QubeExpr,
    },
    /// `// comment`
    Comment(String),
}

/// Quantum state expression: superposition, qubit literal, or named ref.
#[derive(Debug, Clone)]
pub enum QuantumStateExpr {
    /// `α|0⟩ + β|1⟩` — superposition with amplitudes
    Superposition {
        terms: Vec<(QubeAmplitude, QubitState)>,
    },
    /// `|0⟩` or `|1⟩` or `|+⟩` or `|ψ⟩`
    QubitLiteral(QubitState),
    /// Reference to a previously declared state
    StateRef(String),
    /// Tensor product `ψ ⊗ φ`
    TensorProduct(Box<QuantumStateExpr>, Box<QuantumStateExpr>),
}

/// Qubit basis state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QubitState {
    Zero,   // |0⟩
    One,    // |1⟩
    Plus,   // |+⟩
    Minus,  // |−⟩
    Named(String), // |ψ⟩ etc.
}

impl QubitState {
    /// Parse the inner string of a qubit literal (between | and ⟩).
    pub fn from_inner(s: &str) -> Self {
        match s {
            "0" => Self::Zero,
            "1" => Self::One,
            "+" => Self::Plus,
            "-" | "−" => Self::Minus,
            other => Self::Named(other.to_string()),
        }
    }

    pub fn amplitude_pair(&self) -> (f64, f64) {
        match self {
            Self::Zero => (1.0, 0.0),
            Self::One => (0.0, 1.0),
            Self::Plus => (std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2),
            Self::Minus => (std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2),
            Self::Named(_) => (1.0, 0.0), // default to |0⟩ until runtime resolves
        }
    }
}

/// Amplitude in a superposition term: a number, a variable (α, β), or 1.0 implied.
#[derive(Debug, Clone)]
pub enum QubeAmplitude {
    Number(f64),
    Variable(String), // α, β, θ etc.
    Implied,           // no coefficient written — default 1/√N after normalisation
}

/// Quantum gates supported by QUBE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantumGate {
    H,    // Hadamard
    X,    // Pauli-X (NOT)
    Y,    // Pauli-Y
    Z,    // Pauli-Z
    S,    // Phase (√Z)
    T,    // π/8 gate
    CNOT, // controlled-NOT (2-qubit)
    CZ,   // controlled-Z (2-qubit)
    SWAP, // swap (2-qubit)
    Custom(String),
}

impl QuantumGate {
    pub fn from_str(s: &str) -> Self {
        match s {
            "H" => Self::H,
            "X" => Self::X,
            "Y" => Self::Y,
            "Z" => Self::Z,
            "S" => Self::S,
            "T" => Self::T,
            "CNOT" | "CX" => Self::CNOT,
            "CZ" => Self::CZ,
            "SWAP" => Self::SWAP,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn is_two_qubit(&self) -> bool {
        matches!(self, Self::CNOT | Self::CZ | Self::SWAP)
    }
}

/// A classical expression inside QUBE (e.g., for let bindings or assert values).
#[derive(Debug, Clone)]
pub enum QubeExpr {
    Number(f64),
    String(String),
    Bool(bool),
    Variable(String),
}

/// Value in an assert ∈ {…} set.
#[derive(Debug, Clone)]
pub enum AssertValue {
    Integer(i64),
    Float(f64),
    State(QubitState),
    Variable(String),
}
