use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

#[derive(Debug, Clone)]
pub struct Program {
    functions: HashMap<String, Function>,
    tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    statements: Vec<Statement>,
    source: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    statements: Vec<Statement>,
    source: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestReport {
    pub name: String,
    pub group: Option<String>,
    pub passed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
struct Statement {
    line: usize,
    kind: StatementKind,
}

#[derive(Debug, Clone)]
enum StatementKind {
    Print(Expression),
    Let(String, Expression),
    Set(String, Expression),
    AssertEq(Expression, Expression),
    Call(String),
}

#[derive(Debug, Clone)]
enum Expression {
    Int(i64),
    Str(String),
    Var(String),
    Add(Vec<Expression>),
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
            bail!("Aeonmi project requires a `fn main:` entry point")
        }
    }

    pub fn function_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.functions.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn execute_main(&self) -> Result<()> {
        self.execute_function("main")
    }

    pub fn execute_function(&self, name: &str) -> Result<()> {
        let function = self
            .functions
            .get(name)
            .ok_or_else(|| anyhow!("unknown function '{}'", name))?;
        let mut vm = Vm::new(self);
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
}

#[derive(Debug, Default)]
struct Frame {
    locals: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Str(String),
    Unit,
}

impl<'a> Vm<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            stack: vec![Frame::default()],
        }
    }

    fn run_statements(&mut self, statements: &[Statement], source: &Path) -> Result<()> {
        for statement in statements {
            self.execute_statement(statement, source)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement, source: &Path) -> Result<()> {
        match &statement.kind {
            StatementKind::Print(expr) => {
                let value = self.evaluate(expr)?;
                match value {
                    Value::Int(n) => println!("{}", n),
                    Value::Str(s) => println!("{}", s),
                    Value::Unit => println!("()"),
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
            (Value::Unit, Value::Unit) => true,
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
