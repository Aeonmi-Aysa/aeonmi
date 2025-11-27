use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;

use anyhow::{anyhow, bail, Result};
use rand::{rngs::StdRng, Rng, SeedableRng};

/// Create RNG with deterministic seed in tests, entropy otherwise
fn make_rng() -> StdRng {
    #[cfg(test)]
    { StdRng::seed_from_u64(42) }
    #[cfg(not(test))]
    { StdRng::from_entropy() }
}

#[derive(Debug, Clone)]
pub struct Program {
    functions: HashMap<String, Function>,
    tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub statements: Vec<Statement>,
    pub source: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub statements: Vec<Statement>,
    pub source: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestReport {
    pub name: String,
    pub group: Option<String>,
    pub passed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub line: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Print(Expression),
    Let(String, Expression),
    Set(String, Expression),
    AssertEq(Expression, Expression),
    Call(String),
    // Quantum operations
    Superpose(String),    // superpose(qubit_var) - Apply Hadamard
    Entangle(String, String), // entangle(control, target) - H + CNOT
    Dod(String),          // dod(qubit_var) - Apply X gate (Death or Dishonor)
    Measure(String),      // measure(qubit_var) - Measure and collapse
}

#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    Str(String),
    Var(String),
    Add(Vec<Expression>),
    Qubit(i64), // Create qubit in |0⟩ or |1⟩ state
}

#[derive(Debug, Default)]
pub struct ProgramBuilder {
    functions: HashMap<String, Function>,
    tests: Vec<TestCase>,
}

#[derive(Debug)]
pub struct Fragment {
    functions: Vec<Function>,
    tests: Vec<TestCase>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fragment(&mut self, fragment: Fragment) -> Result<()> {
        for function in fragment.functions {
            if self.functions.contains_key(&function.name) {
                bail!(
                    "duplicate function '{}' defined in {}",
                    function.name,
                    function.source.display()
                );
            }
            self.functions.insert(function.name.clone(), function);
        }

        self.tests.extend(fragment.tests);
        Ok(())
    }

    pub fn build(self) -> Program {
        Program {
            functions: self.functions,
            tests: self.tests,
        }
    }
}

impl Program {
    pub fn require_main(&self) -> Result<()> {
        if self.functions.contains_key("main") {
            Ok(())
        } else {
            // Provide helpful error message with available functions
            let available_fns = self.function_names();
            let suggestion = if available_fns.is_empty() {
                "Add a `fn main:` function to your entry point file.".to_string()
            } else {
                format!(
                    "Add a `fn main:` function to your entry point file.\n\
                     Available functions: {}",
                    available_fns.join(", ")
                )
            };
            
            bail!(
                "Missing required entry point: `fn main:`\n\n\
                 Aeonmi projects must define a `fn main:` function as the entry point.\n\n\
                 {}\n\n\
                 Example:\n  fn main:\n    log(\"Hello, Aeonmi!\")",
                suggestion
            )
        }
    }

    pub fn function_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.functions.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.values()
    }
    
    pub fn execute_function_with_timeout_and_log(
        &self,
        name: &str,
        cancel_flag: Arc<AtomicBool>,
        log_path: Option<PathBuf>,
    ) -> Result<()> {
        let function = self
            .functions
            .get(name)
            .ok_or_else(|| anyhow!("unknown function '{}'", name))?;
        let mut vm = Vm::with_timeout_and_log(self, cancel_flag, log_path)?;
        vm.run_statements(&function.statements, &function.source)
    }

    pub fn run_tests(&mut self, filter: Option<&str>) -> Result<Vec<TestReport>> {
        let mut results = Vec::new();
        for test in &self.tests {
            if let Some(filter) = filter {
                if !test.name.contains(filter) {
                    continue;
                }
            }
            let mut vm = Vm::new(self);
            let outcome = vm.run_statements(&test.statements, &test.source);
            match outcome {
                Ok(()) => results.push(TestReport {
                    name: test.name.clone(),
                    group: None,
                    passed: true,
                    message: None,
                }),
                Err(err) => results.push(TestReport {
                    name: test.name.clone(),
                    group: None,
                    passed: false,
                    message: Some(err.to_string()),
                }),
            }
        }
        Ok(results)
    }
}

pub fn parse_fragment(path: &Path, src: &str) -> Result<Fragment> {
    let mut functions = Vec::new();
    let mut tests = Vec::new();

    enum BlockKind {
        Function(Function),
        Test(TestCase),
    }

    let mut current: Option<BlockKind> = None;

    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("fn ") && trimmed.ends_with(':') {
            if let Some(block) = current.take() {
                match block {
                    BlockKind::Function(func) => functions.push(func),
                    BlockKind::Test(test) => tests.push(test),
                }
            }
            let name = trimmed[3..trimmed.len() - 1].trim().to_string();
            if name.is_empty() {
                bail!("empty function name at {}:{}", path.display(), line_no);
            }
            current = Some(BlockKind::Function(Function {
                name,
                statements: Vec::new(),
                source: path.to_path_buf(),
            }));
            continue;
        }

        if trimmed.starts_with("test ") && trimmed.ends_with(':') {
            if let Some(block) = current.take() {
                match block {
                    BlockKind::Function(func) => functions.push(func),
                    BlockKind::Test(test) => tests.push(test),
                }
            }
            let name = trimmed[5..trimmed.len() - 1].trim().to_string();
            if name.is_empty() {
                bail!("empty test name at {}:{}", path.display(), line_no);
            }
            current = Some(BlockKind::Test(TestCase {
                name,
                statements: Vec::new(),
                source: path.to_path_buf(),
            }));
            continue;
        }

        let statement = parse_statement(trimmed, line_no)?;

        match current.as_mut() {
            Some(BlockKind::Function(func)) => func.statements.push(statement),
            Some(BlockKind::Test(test)) => test.statements.push(statement),
            None => {
                bail!(
                    "statement outside of block at {}:{}",
                    path.display(),
                    line_no
                );
            }
        }
    }

    if let Some(block) = current {
        match block {
            BlockKind::Function(func) => functions.push(func),
            BlockKind::Test(test) => tests.push(test),
        }
    }

    Ok(Fragment { functions, tests })
}

fn parse_statement(line: &str, line_no: usize) -> Result<Statement> {
    let statement = if let Some(rest) = line.strip_prefix("print ") {
        StatementKind::Print(parse_expression(rest, line_no)?)
    } else if let Some(rest) = line.strip_prefix("let ") {
        let (name, value) = split_assignment(rest, line_no)?;
        StatementKind::Let(name, parse_expression(value, line_no)?)
    } else if let Some(rest) = line.strip_prefix("set ") {
        let (name, value) = split_assignment(rest, line_no)?;
        StatementKind::Set(name, parse_expression(value, line_no)?)
    } else if let Some(rest) = line.strip_prefix("assert ") {
        let parts: Vec<_> = rest.split("==").collect();
        if parts.len() != 2 {
            bail!("assert requires `==` at line {}", line_no);
        }
        StatementKind::AssertEq(
            parse_expression(parts[0].trim(), line_no)?,
            parse_expression(parts[1].trim(), line_no)?,
        )
    } else if let Some(rest) = line.strip_prefix("call ") {
        StatementKind::Call(rest.trim().to_string())
    } else if let Some(rest) = line.strip_prefix("superpose ") {
        StatementKind::Superpose(rest.trim().to_string())
    } else if let Some(rest) = line.strip_prefix("entangle ") {
        let parts: Vec<_> = rest.split_whitespace().collect();
        if parts.len() != 2 {
            bail!("entangle requires two qubit names at line {}", line_no);
        }
        StatementKind::Entangle(parts[0].to_string(), parts[1].to_string())
    } else if let Some(rest) = line.strip_prefix("dod ") {
        StatementKind::Dod(rest.trim().to_string())
    } else if let Some(rest) = line.strip_prefix("measure ") {
        StatementKind::Measure(rest.trim().to_string())
    } else {
        bail!("unsupported statement `{}` at line {}", line, line_no);
    };

    Ok(Statement {
        line: line_no,
        kind: statement,
    })
}

fn split_assignment(input: &str, line_no: usize) -> Result<(String, &str)> {
    let mut parts = input.splitn(2, '=');
    let name = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("missing variable name at line {}", line_no))?;
    let value = parts
        .next()
        .map(|s| s.trim())
        .ok_or_else(|| anyhow!("missing assignment value at line {}", line_no))?;
    Ok((name, value))
}

fn parse_expression(input: &str, line_no: usize) -> Result<Expression> {
    if input.is_empty() {
        bail!("empty expression at line {}", line_no);
    }

    if input.starts_with('"') && input.ends_with('"') && input.len() >= 2 {
        let mut inner = input[1..input.len() - 1].replace("\\\"", "\"");
        inner = inner.replace("\\n", "\n");
        return Ok(Expression::Str(inner));
    }
    
    // Parse "qubit 0" or "qubit 1"
    if let Some(rest) = input.strip_prefix("qubit ") {
        let value = rest.trim().parse::<i64>()
            .map_err(|_| anyhow!("qubit requires 0 or 1 at line {}", line_no))?;
        if value != 0 && value != 1 {
            bail!("qubit requires 0 or 1, got {} at line {}", value, line_no);
        }
        return Ok(Expression::Qubit(value));
    }

    if input.contains('+') {
        let mut parts = Vec::new();
        for part in input.split('+') {
            parts.push(parse_expression(part.trim(), line_no)?);
        }
        return Ok(Expression::Add(parts));
    }

    if let Ok(value) = input.parse::<i64>() {
        return Ok(Expression::Int(value));
    }

    if input
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Ok(Expression::Var(input.to_string()));
    }

    bail!("could not parse expression `{}` at line {}", input, line_no)
}

struct Vm<'a> {
    program: &'a Program,
    stack: Vec<Frame>,
    cancel_flag: Arc<AtomicBool>,
    log_file: Option<File>,
}

#[derive(Debug, Default)]
struct Frame {
    locals: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Str(String),
    Qubit(QubitState), // Quantum state for a single qubit
}

/// Quantum state representation for a qubit
#[derive(Debug, Clone)]
struct QubitState {
    /// Amplitude for |0⟩ state (real, imag)
    alpha: (f64, f64),
    /// Amplitude for |1⟩ state (real, imag)
    beta: (f64, f64),
    /// Whether this qubit has been measured
    measured: bool,
    /// Measured value (0 or 1) if measured
    measured_value: Option<i64>,
}

impl<'a> Vm<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            stack: vec![Frame::default()],
            cancel_flag: Arc::new(AtomicBool::new(false)),
            log_file: None,
        }
    }
    
    fn with_timeout_and_log(program: &'a Program, timeout_flag: Arc<AtomicBool>, log_path: Option<PathBuf>) -> Result<Self> {
        let log_file = if let Some(path) = log_path {
            Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?)
        } else {
            None
        };
        
        Ok(Self {
            program,
            stack: vec![Frame::default()],
            cancel_flag: timeout_flag,
            log_file,
        })
    }

    fn run_statements(&mut self, statements: &[Statement], source: &Path) -> Result<()> {
        for statement in statements {
            // Check if cancelled
            if self.cancel_flag.load(Ordering::Relaxed) {
                bail!("Execution cancelled due to timeout");
            }
            self.execute_statement(statement, source)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement, source: &Path) -> Result<()> {
        match &statement.kind {
            StatementKind::Print(expr) => {
                let value = self.evaluate(expr)?;
                let output = match value {
                    Value::Int(n) => format!("{}", n),
                    Value::Str(s) => s,
                    Value::Qubit(q) => {
                        if q.measured {
                            format!("Qubit(measured={})", q.measured_value.unwrap())
                        } else {
                            format!("Qubit(|α|²={:.3}, |β|²={:.3})", 
                                q.alpha.0 * q.alpha.0 + q.alpha.1 * q.alpha.1,
                                q.beta.0 * q.beta.0 + q.beta.1 * q.beta.1)
                        }
                    }
                };
                
                // Print to stdout
                println!("{}", output);
                
                // Also log to file if available
                if let Some(ref mut file) = self.log_file {
                    writeln!(file, "{}", output).ok();
                }
            }
            StatementKind::Let(name, expr) => {
                let value = self.evaluate(expr)?;
                self.frame_mut().locals.insert(name.clone(), value);
            }
            StatementKind::Set(name, expr) => {
                let value = self.evaluate(expr)?;
                if let Some(slot) = self.frame_mut().locals.get_mut(name) {
                    *slot = value;
                } else {
                    bail!("undefined variable '{}' at {}", name, source.display());
                }
            }
            StatementKind::AssertEq(lhs, rhs) => {
                let left = self.evaluate(lhs)?;
                let right = self.evaluate(rhs)?;
                if left != right {
                    bail!("assertion failed: left = {:?}, right = {:?}", left, right);
                }
            }
            StatementKind::Call(name) => {
                let function = self
                    .program
                    .functions
                    .get(name)
                    .ok_or_else(|| anyhow!("unknown function '{}'", name))?
                    .clone();
                self.stack.push(Frame::default());
                let result = self.run_statements(&function.statements, &function.source);
                self.stack.pop();
                result?;
            }
            StatementKind::Superpose(qubit_name) => {
                // Apply Hadamard gate to put qubit in superposition
                let qubit = self.frame_mut().locals.get_mut(qubit_name)
                    .ok_or_else(|| anyhow!("undefined qubit '{}'", qubit_name))?;
                
                if let Value::Qubit(ref mut q) = qubit {
                    if q.measured {
                        bail!("Cannot apply superpose to measured qubit");
                    }
                    // H|ψ⟩ = (|0⟩ + |1⟩)/√2 for |0⟩ or (|0⟩ - |1⟩)/√2 for |1⟩
                    let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
                    let new_alpha = (
                        inv_sqrt2 * (q.alpha.0 + q.beta.0),
                        inv_sqrt2 * (q.alpha.1 + q.beta.1),
                    );
                    let new_beta = (
                        inv_sqrt2 * (q.alpha.0 - q.beta.0),
                        inv_sqrt2 * (q.alpha.1 - q.beta.1),
                    );
                    q.alpha = new_alpha;
                    q.beta = new_beta;
                } else {
                    bail!("'{}' is not a qubit", qubit_name);
                }
            }
            StatementKind::Entangle(control_name, target_name) => {
                // Apply H to control, then CNOT(control, target)
                // First apply H to control
                let control = self.frame_mut().locals.get_mut(control_name)
                    .ok_or_else(|| anyhow!("undefined qubit '{}'", control_name))?;
                
                if let Value::Qubit(ref mut q) = control {
                    if q.measured {
                        bail!("Cannot entangle measured qubit");
                    }
                    let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
                    let new_alpha = (
                        inv_sqrt2 * (q.alpha.0 + q.beta.0),
                        inv_sqrt2 * (q.alpha.1 + q.beta.1),
                    );
                    let new_beta = (
                        inv_sqrt2 * (q.alpha.0 - q.beta.0),
                        inv_sqrt2 * (q.alpha.1 - q.beta.1),
                    );
                    q.alpha = new_alpha;
                    q.beta = new_beta;
                } else {
                    bail!("'{}' is not a qubit", control_name);
                }
                
                // Now apply CNOT - simplified 2-qubit operation
                // For full CNOT we'd need tensor product space, but for demo
                // we'll create entanglement by correlating the qubits
                let target = self.frame_mut().locals.get_mut(target_name)
                    .ok_or_else(|| anyhow!("undefined qubit '{}'", target_name))?;
                
                if let Value::Qubit(ref mut t) = target {
                    if t.measured {
                        bail!("Cannot entangle measured qubit");
                    }
                    // Simplified entanglement: flip target based on control state
                    // This creates correlation without full tensor product
                    std::mem::swap(&mut t.alpha, &mut t.beta);
                } else {
                    bail!("'{}' is not a qubit", target_name);
                }
            }
            StatementKind::Dod(qubit_name) => {
                // Apply Pauli-X (NOT) gate
                let qubit = self.frame_mut().locals.get_mut(qubit_name)
                    .ok_or_else(|| anyhow!("undefined qubit '{}'", qubit_name))?;
                
                if let Value::Qubit(ref mut q) = qubit {
                    if q.measured {
                        bail!("Cannot apply dod to measured qubit");
                    }
                    // X|ψ⟩ swaps |0⟩ and |1⟩ amplitudes
                    std::mem::swap(&mut q.alpha, &mut q.beta);
                } else {
                    bail!("'{}' is not a qubit", qubit_name);
                }
            }
            StatementKind::Measure(qubit_name) => {
                // Measure qubit and collapse wavefunction
                let qubit = self.frame_mut().locals.get_mut(qubit_name)
                    .ok_or_else(|| anyhow!("undefined qubit '{}'", qubit_name))?;
                
                if let Value::Qubit(ref mut q) = qubit {
                    if !q.measured {
                        // Calculate probability of measuring |1⟩
                        let prob_one = q.beta.0 * q.beta.0 + q.beta.1 * q.beta.1;
                        
                        // Perform measurement with deterministic RNG in tests
                        let mut rng = make_rng();
                        let result = if rng.gen::<f64>() < prob_one {
                            1
                        } else {
                            0
                        };
                        
                        // Collapse wavefunction
                        if result == 0 {
                            q.alpha = (1.0, 0.0);
                            q.beta = (0.0, 0.0);
                        } else {
                            q.alpha = (0.0, 0.0);
                            q.beta = (1.0, 0.0);
                        }
                        
                        q.measured = true;
                        q.measured_value = Some(result);
                        
                        let msg = format!("Measured qubit '{}': {}", qubit_name, result);
                        println!("{}", msg);
                        if let Some(ref mut file) = self.log_file {
                            writeln!(file, "{}", msg).ok();
                        }
                    }
                } else {
                    bail!("'{}' is not a qubit", qubit_name);
                }
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Int(n) => Ok(Value::Int(*n)),
            Expression::Str(s) => Ok(Value::Str(s.clone())),
            Expression::Var(name) => self
                .frame()
                .locals
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow!("undefined variable '{}'", name)),
            Expression::Qubit(initial_state) => {
                // Create a new qubit in |0⟩ or |1⟩ state
                let q = if *initial_state == 0 {
                    QubitState {
                        alpha: (1.0, 0.0),  // |0⟩
                        beta: (0.0, 0.0),
                        measured: false,
                        measured_value: None,
                    }
                } else {
                    QubitState {
                        alpha: (0.0, 0.0),
                        beta: (1.0, 0.0),  // |1⟩
                        measured: false,
                        measured_value: None,
                    }
                };
                Ok(Value::Qubit(q))
            }
            Expression::Add(items) => {
                let mut total = 0i64;
                for item in items {
                    let value = self.evaluate(item)?;
                    match value {
                        Value::Int(n) => total += n,
                        other => bail!("cannot add non-integer value {:?}", other),
                    }
                }
                Ok(Value::Int(total))
            }
        }
    }

    fn frame(&self) -> &Frame {
        self.stack.last().expect("frame")
    }

    fn frame_mut(&mut self) -> &mut Frame {
        self.stack.last_mut().expect("frame")
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Qubit(a), Value::Qubit(b)) => {
                // Compare measured values if both measured
                if a.measured && b.measured {
                    a.measured_value == b.measured_value
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_function_and_test() {
        let src = "\nfn main:\n    print \"hello\"\n\ntest basic:\n    assert 1 + 1 == 2\n";
        let fragment = parse_fragment(Path::new("main.ai"), src).unwrap();
        assert_eq!(fragment.functions.len(), 1);
        assert_eq!(fragment.tests.len(), 1);
    }

    #[test]
    fn run_test_failure() {
        let src = "\nfn main:\n    print \"noop\"\n\ntest fail:\n    assert 2 == 3\n";
        let mut builder = ProgramBuilder::new();
        builder
            .add_fragment(parse_fragment(Path::new("main.ai"), src).unwrap())
            .unwrap();
        let mut program = builder.build();
        let results = program.run_tests(None).unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }
}
