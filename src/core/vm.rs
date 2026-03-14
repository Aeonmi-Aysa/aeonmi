//! Aeonmi VM: tree-walk interpreter over IR with quantum simulation support.
//! Supports: literals, quantum arrays/objects, let/assign, if/while/for, fn calls/returns,
//! binary/unary ops, quantum operations, and built-ins: print, log, time_ms, rand, len.

use crate::core::ir::*;
use crate::core::quantum_simulator::QuantumSimulator;
use crate::core::quantum_algorithms::{QuantumAlgorithms, DeutschJozsaOracle};
use crate::core::hardware_integration::{HardwareManager, QuantumCircuit};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(Function), // user-defined
    Builtin(Builtin),
    
    // AEONMI Quantum-Native Values
    QuantumArray(Vec<Value>, bool), // elements, is_superposition
    QuantumState(String, Option<f64>), // state, amplitude
    QubitReference(String), // reference to qubit in simulator
}

#[derive(Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Block,
    pub env: Env, // closure (shallow copy at def time)
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("params", &self.params)
            .field("body_len", &self.body.stmts.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub arity: usize, // use usize::MAX for variadic
    pub f: fn(&mut Interpreter, Vec<Value>) -> Result<Value, RuntimeError>,
}

impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builtin").field("name", &self.name).finish()
    }
}

#[derive(Clone, Debug)]
pub struct Env {
    frames: Vec<HashMap<String, Value>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
        }
    }
    pub fn push(&mut self) {
        self.frames.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.frames.pop();
    }
    pub fn define(&mut self, k: String, v: Value) {
        self.frames.last_mut().unwrap().insert(k, v);
    }

    pub fn assign(&mut self, k: &str, v: Value) -> bool {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(k) {
                frame.insert(k.to_string(), v);
                return true;
            }
        }
        false
    }

    pub fn get(&self, k: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(k) {
                return Some(v.clone());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub env: Env,
    pub quantum_sim: QuantumSimulator, // Quantum simulator for quantum operations
    pub quantum_alg: QuantumAlgorithms, // Quantum algorithms library
    pub hardware_mgr: HardwareManager, // Real quantum hardware integration
    pub base_dir: Option<std::path::PathBuf>, // Directory of the executing .ai file (for imports)
    imported: std::collections::HashSet<String>, // Track already-imported paths to avoid cycles
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();
        // Builtins
        env.define(
            "print".into(),
            Value::Builtin(Builtin {
                name: "print",
                arity: usize::MAX,
                f: builtin_print,
            }),
        );
        env.define(
            "log".into(),
            Value::Builtin(Builtin {
                name: "log",
                arity: usize::MAX,
                f: builtin_print,
            }),
        );
        env.define(
            "time_ms".into(),
            Value::Builtin(Builtin {
                name: "time_ms",
                arity: 0,
                f: builtin_time_ms,
            }),
        );
        env.define(
            "rand".into(),
            Value::Builtin(Builtin {
                name: "rand",
                arity: 0,
                f: builtin_rand,
            }),
        );
        env.define(
            "len".into(),
            Value::Builtin(Builtin {
                name: "len",
                arity: 1,
                f: builtin_len,
            }),
        );
        
        // Add quantum built-ins
        env.define(
            "superpose".into(),
            Value::Builtin(Builtin {
                name: "superpose",
                arity: 1,
                f: builtin_superpose,
            }),
        );
        env.define(
            "measure".into(),
            Value::Builtin(Builtin {
                name: "measure",
                arity: 1,
                f: builtin_measure,
            }),
        );
        env.define(
            "entangle".into(),
            Value::Builtin(Builtin {
                name: "entangle",
                arity: 2,
                f: builtin_entangle,
            }),
        );
        env.define(
            "apply_gate".into(),
            Value::Builtin(Builtin {
                name: "apply_gate",
                arity: 2,
                f: builtin_apply_gate,
            }),
        );
        
        // Add quantum algorithm built-ins
        env.define(
            "grovers_search".into(),
            Value::Builtin(Builtin {
                name: "grovers_search",
                arity: 2,
                f: builtin_grovers_search,
            }),
        );
        env.define(
            "quantum_fourier_transform".into(),
            Value::Builtin(Builtin {
                name: "quantum_fourier_transform",
                arity: 1,
                f: builtin_qft,
            }),
        );
        env.define(
            "shors_factoring".into(),
            Value::Builtin(Builtin {
                name: "shors_factoring",
                arity: 1,
                f: builtin_shors,
            }),
        );
        env.define(
            "deutsch_jozsa".into(),
            Value::Builtin(Builtin {
                name: "deutsch_jozsa",
                arity: 1,
                f: builtin_deutsch_jozsa,
            }),
        );
        env.define(
            "bernstein_vazirani".into(),
            Value::Builtin(Builtin {
                name: "bernstein_vazirani",
                arity: 1,
                f: builtin_bernstein_vazirani,
            }),
        );
        env.define(
            "quantum_teleportation".into(),
            Value::Builtin(Builtin {
                name: "quantum_teleportation",
                arity: 1,
                f: builtin_quantum_teleportation,
            }),
        );
        
        // Add hardware integration built-ins
        // Quantum circuit control builtins
        env.define(
            "__quantum_circuit_begin".into(),
            Value::Builtin(Builtin {
                name: "__quantum_circuit_begin",
                arity: 1,
                f: |_i, args| {
                    if let Some(Value::String(name)) = args.first() {
                        eprintln!("[circuit] begin: {}", name);
                    }
                    Ok(Value::Null)
                },
            }),
        );
        env.define(
            "__quantum_circuit_end".into(),
            Value::Builtin(Builtin {
                name: "__quantum_circuit_end",
                arity: 1,
                f: |_i, args| {
                    if let Some(Value::String(name)) = args.first() {
                        eprintln!("[circuit] end: {}", name);
                    }
                    Ok(Value::Null)
                },
            }),
        );
        env.define(
            "__quantum_circuit_run".into(),
            Value::Builtin(Builtin {
                name: "__quantum_circuit_run",
                arity: usize::MAX,
                f: |_i, args| {
                    // Args are already evaluated (side effects already happened)
                    Ok(args.last().cloned().unwrap_or(Value::Null))
                },
            }),
        );
        env.define(
            "list_devices".into(),
            Value::Builtin(Builtin {
                name: "list_devices",
                arity: 0,
                f: builtin_list_devices,
            }),
        );
        env.define(
            "submit_job".into(),
            Value::Builtin(Builtin {
                name: "submit_job",
                arity: 3,
                f: builtin_submit_job,
            }),
        );
        env.define(
            "job_status".into(),
            Value::Builtin(Builtin {
                name: "job_status",
                arity: 1,
                f: builtin_job_status,
            }),
        );
        env.define(
            "job_results".into(),
            Value::Builtin(Builtin {
                name: "job_results",
                arity: 1,
                f: builtin_job_results,
            }),
        );
        
        // Quantum gate builtins — callable as H(q), X(q), CNOT(q1, q2) or apply_gate(q, H)
        env.define("H".into(), Value::Builtin(Builtin { name: "H", arity: 1, f: gate_h }));
        env.define("X".into(), Value::Builtin(Builtin { name: "X", arity: 1, f: gate_x }));
        env.define("Y".into(), Value::Builtin(Builtin { name: "Y", arity: 1, f: gate_y }));
        env.define("Z".into(), Value::Builtin(Builtin { name: "Z", arity: 1, f: gate_z }));
        env.define("S".into(), Value::Builtin(Builtin { name: "S", arity: 1, f: gate_s }));
        env.define("T".into(), Value::Builtin(Builtin { name: "T", arity: 1, f: gate_t }));
        env.define("CNOT".into(), Value::Builtin(Builtin { name: "CNOT", arity: 2, f: gate_cnot }));
        env.define("CX".into(),   Value::Builtin(Builtin { name: "CX",   arity: 2, f: gate_cnot }));
        env.define("HADAMARD".into(), Value::Builtin(Builtin { name: "H", arity: 1, f: gate_h }));
        env.define("NOT".into(),      Value::Builtin(Builtin { name: "X", arity: 1, f: gate_x }));
        
        // Hieroglyphic glyph operations — called when scripts use Unicode symbols like 𓀀(x, y)
        env.define(
            "__glyph".into(),
            Value::Builtin(Builtin {
                name: "__glyph",
                arity: usize::MAX, // variadic: first arg is symbol name, rest are user args
                f: builtin_glyph,
            }),
        );
        
        // Type/toString utility
        env.define(
            "typeof".into(),
            Value::Builtin(Builtin {
                name: "typeof",
                arity: 1,
                f: builtin_typeof,
            }),
        );
        env.define(
            "toString".into(),
            Value::Builtin(Builtin {
                name: "toString",
                arity: 1,
                f: builtin_to_string,
            }),
        );
        env.define(
            "toNumber".into(),
            Value::Builtin(Builtin {
                name: "toNumber",
                arity: 1,
                f: builtin_to_number,
            }),
        );
        env.define(
            "read_file".into(),
            Value::Builtin(Builtin {
                name: "read_file",
                arity: 1,
                f: builtin_read_file,
            }),
        );
        env.define(
            "write_file".into(),
            Value::Builtin(Builtin {
                name: "write_file",
                arity: 2,
                f: builtin_write_file,
            }),
        );
        env.define(
            "file_exists".into(),
            Value::Builtin(Builtin {
                name: "file_exists",
                arity: 1,
                f: builtin_file_exists,
            }),
        );
        env.define(
            "__index_access".into(),
            Value::Builtin(Builtin {
                name: "__index_access",
                arity: 2,
                f: |_i, args| {
                    let idx = match args.get(1) {
                        Some(Value::Number(n)) => *n as usize,
                        _ => return Ok(Value::Null),
                    };
                    match args.into_iter().next().unwrap_or(Value::Null) {
                        Value::Array(a) => Ok(a.into_iter().nth(idx).unwrap_or(Value::Null)),
                        Value::String(s) => Ok(s.chars().nth(idx)
                            .map(|c| Value::String(c.to_string()))
                            .unwrap_or(Value::Null)),
                        _ => Ok(Value::Null),
                    }
                },
            }),
        );
        env.define(
            "__quantum_index".into(),
            Value::Builtin(Builtin {
                name: "__quantum_index",
                arity: 2,
                f: |_i, args| {
                    let idx = match args.get(1) {
                        Some(Value::Number(n)) => *n as usize,
                        _ => return Ok(Value::Null),
                    };
                    match args.into_iter().next().unwrap_or(Value::Null) {
                        Value::Array(a) | Value::QuantumArray(a, _) =>
                            Ok(a.into_iter().nth(idx).unwrap_or(Value::Null)),
                        _ => Ok(Value::Null),
                    }
                },
            }),
        );
        env.define(
            "__spread".into(),
            Value::Builtin(Builtin {
                name: "__spread",
                arity: 1,
                f: |_i, args| {
                    // __spread wraps a value for spread context; returns it as-is
                    // Actual spreading is handled by concat / array construction
                    Ok(args.into_iter().next().unwrap_or(Value::Null))
                },
            }),
        );
        
        Self { 
            env,
            quantum_sim: QuantumSimulator::new(),
            quantum_alg: QuantumAlgorithms::new(),
            hardware_mgr: HardwareManager::new(),
            base_dir: None,
            imported: std::collections::HashSet::new(),
        }
    }

    /// Resolve and execute imports from another .ai file.
    /// Reads the file, lexes/parses/lowers it, then loads its top-level
    /// declarations (functions, consts, lets) into the current environment.
    fn resolve_import(&mut self, import_path: &str) -> Result<(), RuntimeError> {
        use crate::core::lexer::Lexer;
        use crate::core::parser::Parser as AeParser;
        use crate::core::lowering::lower_ast_to_ir;

        // Resolve path relative to base_dir
        let resolved = if let Some(ref base) = self.base_dir {
            let mut p = base.clone();
            // Strip leading "./" if present
            let clean = import_path.trim_start_matches("./").trim_start_matches(".\\" );
            p.push(clean);
            // Add .ai extension if missing
            if p.extension().is_none() {
                p.set_extension("ai");
            }
            p
        } else {
            let mut p = std::path::PathBuf::from(import_path);
            if p.extension().is_none() {
                p.set_extension("ai");
            }
            p
        };

        let canonical = resolved.display().to_string();
        if self.imported.contains(&canonical) {
            return Ok(()); // already loaded — skip cycle
        }
        self.imported.insert(canonical.clone());

        let source = std::fs::read_to_string(&resolved)
            .map_err(|e| err(format!("import '{}': {}", resolved.display(), e)))?;

        let mut lexer = Lexer::from_str(&source);
        let tokens = lexer.tokenize()
            .map_err(|e| err(format!("import '{}' lex error: {}", resolved.display(), e)))?;

        let mut parser = AeParser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| err(format!("import '{}' parse error: {}", resolved.display(), e)))?;

        let module = lower_ast_to_ir(&ast, &canonical)
            .map_err(|e| err(format!("import '{}' lowering error: {}", resolved.display(), e)))?;

        // Recursively resolve this module's imports first
        for imp in &module.imports {
            self.resolve_import(&imp.path)?;
        }

        // Load declarations into the current environment (functions, lets, consts)
        // but do NOT auto-call main — imported modules should not run their main.
        for d in &module.decls {
            match d {
                Decl::Const(c) => {
                    let v = self.eval_expr(&c.value)?;
                    self.env.define(c.name.clone(), v);
                }
                Decl::Let(l) => {
                    let v = if let Some(e) = &l.value {
                        self.eval_expr(e)?
                    } else {
                        Value::Null
                    };
                    self.env.define(l.name.clone(), v);
                }
                Decl::Fn(f) => {
                    // Skip the imported module's auto-generated main()
                    if f.name == "main" { continue; }
                    let func = Value::Function(Function {
                        params: f.params.clone(),
                        body: f.body.clone(),
                        env: self.env.clone(),
                    });
                    self.env.define(f.name.clone(), func);
                }
            }
        }

        Ok(())
    }

    pub fn run_module(&mut self, m: &Module) -> Result<(), RuntimeError> {
        debug_log!("vm: run_module decls={} imports={}", m.decls.len(), m.imports.len());

        // Phase 1: resolve imports — load their declarations into env
        for imp in &m.imports {
            self.resolve_import(&imp.path)?;
        }

        // Load top-level decls
        for d in &m.decls {
            debug_log!("vm: processing decl: {:?}", d);
            match d {
                Decl::Const(c) => {
                    let v = self.eval_expr(&c.value)?;
                    self.env.define(c.name.clone(), v);
                }
                Decl::Let(l) => {
                    let v = if let Some(e) = &l.value {
                        self.eval_expr(e)?
                    } else {
                        Value::Null
                    };
                    self.env.define(l.name.clone(), v);
                }
                Decl::Fn(f) => {
                    debug_log!("vm: load fn '{}'", f.name);
                    let func = Value::Function(Function {
                        params: f.params.clone(),
                        body: f.body.clone(),
                        env: self.env.clone(),
                    });
                    self.env.define(f.name.clone(), func);
                }
            }
        }
        // If there is a `main` fn with zero params, run it.
        if let Some(Value::Function(_)) = self.env.get("main") {
            debug_log!("vm: calling main()");
            let _ = self.call_ident("main", vec![])?;
        } else {
            debug_log!("vm: no main() found");
        }
        Ok(())
    }

    fn call_ident(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let callee = self
            .env
            .get(name)
            .ok_or_else(|| err(format!("Undefined function `{}`", name)))?;
        self.call_value(callee, args)
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match callee {
            Value::Builtin(b) => {
                if b.arity != usize::MAX && b.arity != args.len() {
                    return Err(err(format!(
                        "builtin `{}` expected {} args, got {}",
                        b.name,
                        b.arity,
                        args.len()
                    )));
                }
                (b.f)(self, args)
            }
            Value::Function(fun) => {
                if fun.params.len() != args.len() {
                    return Err(err(format!(
                        "function expected {} args, got {}",
                        fun.params.len(),
                        args.len()
                    )));
                }
                // New scope with closure base
                let saved = self.env.clone();
                self.env = fun.env.clone();
                self.env.push();
                for (p, v) in fun.params.iter().zip(args.into_iter()) {
                    self.env.define(p.clone(), v);
                }
                // Execute - don't create another scope in exec_block for function bodies
                let ret = self.exec_function_block(&fun.body);
                // Restore
                let out = match ret {
                    ControlFlow::Ok => Ok(Value::Null),
                    ControlFlow::Return(v) => Ok(v.unwrap_or(Value::Null)),
                    ControlFlow::Err(e) => Err(e),
                };
                self.env = saved;
                out
            }
            other => Err(err(format!("callee is not callable: {:?}", other))),
        }
    }

    fn exec_block(&mut self, b: &Block) -> ControlFlow {
        self.env.push();
        for s in &b.stmts {
            match self.exec_stmt(s) {
                ControlFlow::Ok => {}
                other => {
                    self.env.pop();
                    return other;
                }
            }
        }
        self.env.pop();
        ControlFlow::Ok
    }

    fn exec_function_block(&mut self, b: &Block) -> ControlFlow {
        debug_log!("vm: exec_function_block");
        // Don't create an additional scope - function call already created one
        for s in &b.stmts {
            match self.exec_stmt(s) {
                ControlFlow::Ok => {}
                other => {
                    return other;
                }
            }
        }
        ControlFlow::Ok
    }

    fn exec_stmt(&mut self, s: &Stmt) -> ControlFlow {
        use Stmt::*;
        match s {
            Expr(e) => {
                if let Err(e) = self.eval_expr(e) {
                    return ControlFlow::Err(e);
                }
                ControlFlow::Ok
            }
            Return(None) => ControlFlow::Return(None),
            Return(Some(e)) => {
                let v = match self.eval_expr(e) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };
                ControlFlow::Return(Some(v))
            }
            If {
                cond,
                then_block,
                else_block,
            } => {
                let c = match self.eval_expr(cond) {
                    Ok(v) => self.truthy(&v),
                    Err(e) => return ControlFlow::Err(e),
                };
                if c {
                    self.exec_block(then_block)
                } else if let Some(e) = else_block {
                    self.exec_block(e)
                } else {
                    ControlFlow::Ok
                }
            }
            While { cond, body } => {
                loop {
                    let c = match self.eval_expr(cond) {
                        Ok(v) => self.truthy(&v),
                        Err(e) => return ControlFlow::Err(e),
                    };
                    if !c {
                        break;
                    }
                    match self.exec_block(body) {
                        ControlFlow::Ok => {}
                        other => return other,
                    }
                }
                ControlFlow::Ok
            }
            For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(i) = init {
                    if let ControlFlow::Err(e) = self.exec_stmt(i) {
                        return ControlFlow::Err(e);
                    }
                }
                loop {
                    if let Some(c) = cond {
                        let ok = match self.eval_expr(c) {
                            Ok(v) => self.truthy(&v),
                            Err(e) => return ControlFlow::Err(e),
                        };
                        if !ok {
                            break;
                        }
                    }
                    match self.exec_block(body) {
                        ControlFlow::Ok => {}
                        other => return other,
                    }
                    if let Some(st) = step {
                        match self.exec_stmt(st) {
                            ControlFlow::Ok => {}
                            other => return other,
                        }
                    }
                }
                ControlFlow::Ok
            }
            ForIn { var, iterable, body } => {
                let coll = match self.eval_expr(iterable) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };
                let items: Vec<Value> = match coll {
                    Value::Array(a) => a,
                    Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                    other => vec![other], // single value — iterate once
                };
                for item in items {
                    self.env.push();
                    self.env.define(var.clone(), item);
                    match self.exec_block(body) {
                        ControlFlow::Ok => {}
                        other => { self.env.pop(); return other; }
                    }
                    self.env.pop();
                }
                ControlFlow::Ok
            }
            Let { name, value } => {
                let v = if let Some(e) = value {
                    match self.eval_expr(e) {
                        Ok(v) => v,
                        Err(e) => return ControlFlow::Err(e),
                    }
                } else {
                    Value::Null
                };
                debug_log!("vm: let {} = {:?}", name, v);
                self.env.define(name.clone(), v);
                ControlFlow::Ok
            }
            Assign { target, value } => {
                // Only Ident target in v0
                if let crate::core::ir::Expr::Ident(name) = target {
                    let v = match self.eval_expr(value) {
                        Ok(v) => v,
                        Err(e) => return ControlFlow::Err(e),
                    };
                    if !self.env.assign(name, v) {
                        return ControlFlow::Err(err(format!("Undefined variable `{}`", name)));
                    }
                    ControlFlow::Ok
                } else {
                    ControlFlow::Err(err(
                        "Only simple identifier assignment supported in v0".into()
                    ))
                }
            }
        }
    }

    fn eval_expr(&mut self, e: &Expr) -> Result<Value, RuntimeError> {
        use Expr::*;
        Ok(match e {
            Lit(l) => match l {
                crate::core::ir::Lit::Null => Value::Null,
                crate::core::ir::Lit::Bool(b) => Value::Bool(*b),
                crate::core::ir::Lit::Number(n) => Value::Number(*n),
                crate::core::ir::Lit::String(s) => Value::String(s.clone()),
            },
            Ident(s) => {
                debug_log!("vm: lookup '{}'", s);
                let result = self
                    .env
                    .get(s)
                    .ok_or_else(|| err(format!("Undefined identifier `{}`", s)))?;
                debug_log!("vm: found '{}' -> {:?}", s, result);
                result
            }
            Call { callee, args } => {
                // Fast path: direct ident call
                if let Expr::Ident(name) = &**callee {
                    let argv = collect_vals(self, args)?;
                    self.call_ident(name, argv)?
                } else if let Expr::Member { object, property } = &**callee {
                    // obj.method(args) — look up method as a global fn named Object_method,
                    // or resolve from the object's field map.
                    let obj_val = self.eval_expr(object)?;
                    let argv = collect_vals(self, args)?;
                    match obj_val {
                        Value::Object(ref map) => {
                            if let Some(func) = map.get(property.as_str()).cloned() {
                                self.call_value(func, argv)?
                            } else {
                                // Try global lookup: TypeName_method pattern
                                if let Expr::Ident(type_name) = &**object {
                                    let fn_name = format!("{}_{}", type_name, property);
                                    if self.env.get(&fn_name).is_some() {
                                        self.call_ident(&fn_name, argv)?
                                    } else {
                                        return Err(err(format!("Method '{}' not found on object", property)));
                                    }
                                } else {
                                    return Err(err(format!("Method '{}' not found on object", property)));
                                }
                            }
                        }
                        Value::Array(_) => {
                            // Array built-in methods: push, pop, slice, join, concat, indexOf
                            // For mutating methods (push/pop), try to update the binding in env.
                            let arr_ident: Option<String> = if let Expr::Ident(n) = &**object {
                                Some(n.clone())
                            } else {
                                None
                            };
                            match property.as_str() {
                                "push" => {
                                    // Mutate array in env; return new length.
                                    if let Some(ref var_name) = arr_ident {
                                        let mut current = match self.env.get(var_name) {
                                            Some(Value::Array(a)) => a,
                                            _ => return Err(err(format!("push: '{}' is not an array", var_name))),
                                        };
                                        for v in argv {
                                            current.push(v);
                                        }
                                        let new_len = current.len() as f64;
                                        self.env.assign(var_name, Value::Array(current));
                                        Value::Number(new_len)
                                    } else {
                                        // No ident — return extended array (non-mutating fallback)
                                        match obj_val {
                                            Value::Array(mut a) => {
                                                for v in argv { a.push(v); }
                                                let len = a.len() as f64;
                                                let _ = a; // can't assign back without ident
                                                Value::Number(len)
                                            }
                                            _ => Value::Null,
                                        }
                                    }
                                }
                                "pop" => {
                                    if let Some(ref var_name) = arr_ident {
                                        let mut current = match self.env.get(var_name) {
                                            Some(Value::Array(a)) => a,
                                            _ => return Err(err(format!("pop: '{}' is not an array", var_name))),
                                        };
                                        let popped = current.pop().unwrap_or(Value::Null);
                                        self.env.assign(var_name, Value::Array(current));
                                        popped
                                    } else {
                                        match obj_val {
                                            Value::Array(mut a) => a.pop().unwrap_or(Value::Null),
                                            _ => Value::Null,
                                        }
                                    }
                                }
                                "length" => {
                                    match obj_val {
                                        Value::Array(ref a) => Value::Number(a.len() as f64),
                                        _ => Value::Null,
                                    }
                                }
                                "join" => {
                                    let sep = match argv.first() {
                                        Some(Value::String(s)) => s.clone(),
                                        _ => ",".to_string(),
                                    };
                                    match obj_val {
                                        Value::Array(ref a) => {
                                            let s = a.iter().map(display).collect::<Vec<_>>().join(&sep);
                                            Value::String(s)
                                        }
                                        _ => Value::Null,
                                    }
                                }
                                "indexOf" => {
                                    let target = argv.into_iter().next().unwrap_or(Value::Null);
                                    match obj_val {
                                        Value::Array(ref a) => {
                                            let idx = a.iter().position(|v| eq_val(v, &target));
                                            Value::Number(idx.map(|i| i as f64).unwrap_or(-1.0))
                                        }
                                        _ => Value::Number(-1.0),
                                    }
                                }
                                "slice" => {
                                    let start = match argv.first() {
                                        Some(Value::Number(n)) => *n as usize,
                                        _ => 0,
                                    };
                                    let end_default = match &obj_val {
                                        Value::Array(a) => a.len(),
                                        _ => 0,
                                    };
                                    let end = match argv.get(1) {
                                        Some(Value::Number(n)) => *n as usize,
                                        _ => end_default,
                                    };
                                    match obj_val {
                                        Value::Array(ref a) => {
                                            let sliced = a[start.min(a.len())..end.min(a.len())].to_vec();
                                            Value::Array(sliced)
                                        }
                                        _ => Value::Null,
                                    }
                                }
                                "concat" => {
                                    let mut base = match obj_val {
                                        Value::Array(a) => a,
                                        _ => vec![],
                                    };
                                    for v in argv {
                                        match v {
                                            Value::Array(other) => base.extend(other),
                                            other => base.push(other),
                                        }
                                    }
                                    Value::Array(base)
                                }
                                "is_empty" | "isEmpty" => {
                                    match obj_val {
                                        Value::Array(ref a) => Value::Bool(a.is_empty()),
                                        _ => Value::Bool(true),
                                    }
                                }
                                other_method => {
                                    return Err(err(format!("Array has no method '{}'", other_method)));
                                }
                            }
                        }
                        Value::String(_) => {
                            // String built-in methods
                            match property.as_str() {
                                "length" => match obj_val {
                                    Value::String(ref s) => Value::Number(s.chars().count() as f64),
                                    _ => Value::Null,
                                },
                                "toUpperCase" | "to_upper_case" => match obj_val {
                                    Value::String(s) => Value::String(s.to_uppercase()),
                                    _ => Value::Null,
                                },
                                "toLowerCase" | "to_lower_case" => match obj_val {
                                    Value::String(s) => Value::String(s.to_lowercase()),
                                    _ => Value::Null,
                                },
                                "trim" => match obj_val {
                                    Value::String(s) => Value::String(s.trim().to_string()),
                                    _ => Value::Null,
                                },
                                "includes" | "contains" => {
                                    let needle = match argv.first() {
                                        Some(Value::String(s)) => s.clone(),
                                        _ => String::new(),
                                    };
                                    match obj_val {
                                        Value::String(s) => Value::Bool(s.contains(&needle)),
                                        _ => Value::Bool(false),
                                    }
                                }
                                "split" => {
                                    let sep = match argv.first() {
                                        Some(Value::String(s)) => s.clone(),
                                        _ => String::new(),
                                    };
                                    match obj_val {
                                        Value::String(s) => {
                                            let parts: Vec<Value> = s.split(&*sep)
                                                .map(|p| Value::String(p.to_string()))
                                                .collect();
                                            Value::Array(parts)
                                        }
                                        _ => Value::Null,
                                    }
                                }
                                other_method => {
                                    return Err(err(format!("String has no method '{}'", other_method)));
                                }
                            }
                        }
                        _ => {
                            // For non-object callees, try global fn as Type_method
                            if let Expr::Ident(type_name) = &**object {
                                let fn_name = format!("{}_{}", type_name, property);
                                if self.env.get(&fn_name).is_some() {
                                    self.call_ident(&fn_name, argv)?
                                } else {
                                    return Err(err(format!("Cannot call method '{}' on value", property)));
                                }
                            } else {
                                return Err(err(format!("Cannot call method '{}' on value", property)));
                            }
                        }
                    }
                } else {
                    let callee_v = self.eval_expr(callee)?;
                    let argv = collect_vals(self, args)?;
                    self.call_value(callee_v, argv)?
                }
            }
            Unary { op, expr } => {
                let v = self.eval_expr(expr)?;
                match op {
                    UnOp::Neg => match v {
                        Value::Number(n) => Value::Number(-n),
                        other => return Err(err(format!("Unary `-` on non-number: {:?}", other))),
                    },
                    UnOp::Not => Value::Bool(!self.truthy(&v)),
                }
            }
            Binary { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binop(op, l, r)?
            }
            Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(self.eval_expr(it)?);
                }
                Value::Array(out)
            }
            Object(kvs) => {
                let mut map = HashMap::with_capacity(kvs.len());
                for (k, v) in kvs {
                    map.insert(k.clone(), self.eval_expr(v)?);
                }
                Value::Object(map)
            }
            Member { object, property } => {
                let obj_val = self.eval_expr(object)?;
                match obj_val {
                    Value::Object(ref map) => {
                        map.get(property.as_str()).cloned().unwrap_or(Value::Null)
                    }
                    Value::String(ref s) => {
                        // built-in string properties
                        match property.as_str() {
                            "length" => Value::Number(s.chars().count() as f64),
                            _ => Value::Null,
                        }
                    }
                    Value::Array(ref a) => {
                        match property.as_str() {
                            "length" => Value::Number(a.len() as f64),
                            _ => Value::Null,
                        }
                    }
                    other => return Err(err(format!("Cannot access property '{}' on {:?}", property, other))),
                }
            }
        })
    }

    fn eval_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        use BinOp::*;
        match op {
            Add => match (l, r) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, display(&b)))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", display(&a), b))),
                (a, b) => Err(err(format!("`+` on incompatible types: {:?}, {:?}", a, b))),
            },
            Sub => num2(l, r, |a, b| a - b),
            Mul => num2(l, r, |a, b| a * b),
            Div => num2(l, r, |a, b| a / b),
            Mod => num2(l, r, |a, b| a % b),
            Eq => Ok(Value::Bool(eq_val(&l, &r))),
            Ne => Ok(Value::Bool(!eq_val(&l, &r))),
            Lt => cmp2(l, r, |a, b| a < b),
            Le => cmp2(l, r, |a, b| a <= b),
            Gt => cmp2(l, r, |a, b| a > b),
            Ge => cmp2(l, r, |a, b| a >= b),
            And => Ok(Value::Bool(self.truthy(&l) && self.truthy(&r))),
            Or => Ok(Value::Bool(self.truthy(&l) || self.truthy(&r))),
        }
    }

    fn truthy(&self, v: &Value) -> bool {
        match v {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function(_) | Value::Builtin(_) => true,
            
            // Quantum values
            Value::QuantumArray(a, _) => !a.is_empty(),
            Value::QuantumState(state, _) => !state.is_empty(),
            Value::QubitReference(_) => true, // Qubit references are always truthy
        }
    }
}

enum ControlFlow {
    Ok,
    Return(Option<Value>),
    Err(RuntimeError),
}

impl From<RuntimeError> for ControlFlow {
    fn from(e: RuntimeError) -> Self {
        ControlFlow::Err(e)
    }
}

fn err(msg: String) -> RuntimeError {
    RuntimeError { message: msg }
}

fn collect_vals(i: &mut Interpreter, es: &[Expr]) -> Result<Vec<Value>, RuntimeError> {
    let mut out = Vec::with_capacity(es.len());
    for e in es {
        out.push(i.eval_expr(e)?);
    }
    Ok(out)
}

fn num2(l: Value, r: Value, f: fn(f64, f64) -> f64) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        (a, b) => Err(err(format!("numeric op on non-numbers: {:?}, {:?}", a, b))),
    }
}

fn cmp2(l: Value, r: Value, f: fn(f64, f64) -> bool) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
        (a, b) => Err(err(format!("comparison on non-numbers: {:?}, {:?}", a, b))),
    }
}

fn eq_val(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Null, Null) => true,
        (Bool(x), Bool(y)) => x == y,
        (Number(x), Number(y)) => x == y,
        (String(x), String(y)) => x == y,

        (Array(x), Array(y)) => {
            if x.len() != y.len() {
                return false;
            }
            for (lx, ry) in x.iter().zip(y.iter()) {
                if !eq_val(lx, ry) {
                    return false;
                }
            }
            true
        }

        (Object(x), Object(y)) => {
            if x.len() != y.len() {
                return false;
            }
            for (k, vx) in x.iter() {
                match y.get(k) {
                    Some(vy) if eq_val(vx, vy) => {}
                    _ => return false,
                }
            }
            true
        }

        // Functions/builtins: not comparable for now
        (Function(_), Function(_)) => false,
        (Builtin(_), Builtin(_)) => false,

        _ => false,
    }
}

// ---------- Builtins ----------

fn builtin_print(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let parts: Vec<String> = args.iter().map(display).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Null)
}

fn builtin_time_ms(_i: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    Ok(Value::Number(now.as_millis() as f64))
}

static GLOBAL_SEED: AtomicU64 = AtomicU64::new(0);
static INIT_SEED: Once = Once::new();

fn init_seed_once() {
    INIT_SEED.call_once(|| {
        // Order of precedence:
        // 1. AEONMI_SEED env var (u64 parse)
        // 2. Time-based fallback (nanos lower 32 bits)
        let from_env = std::env::var("AEONMI_SEED")
            .ok()
            .and_then(|s| s.parse::<u64>().ok());
        let seed = from_env.unwrap_or_else(|| {
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                & 0xFFFF_FFFF) as u64
        });
        // Avoid zero seed (LCG degenerate cycles shorter sometimes)
        let seed = if seed == 0 { 1 } else { seed };
        GLOBAL_SEED.store(seed, Ordering::Relaxed);
    });
}

fn lcg_next() -> u64 {
    init_seed_once();
    // Parameters from Numerical Recipes LCG (same as original placeholder constants)
    let mut x = GLOBAL_SEED.load(Ordering::Relaxed);
    x = x.wrapping_mul(1664525).wrapping_add(1013904223);
    GLOBAL_SEED.store(x, Ordering::Relaxed);
    x
}

fn builtin_rand(_i: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = lcg_next();
    Ok(Value::Number(((x >> 8) as f64) / (u32::MAX as f64)))
}

fn builtin_len(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err(format!(
            "len expects exactly 1 argument, got {}",
            args.len()
        )));
    }

    match args.into_iter().next().unwrap() {
        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
        Value::Array(items) => Ok(Value::Number(items.len() as f64)),
        Value::Object(map) => Ok(Value::Number(map.len() as f64)),
        Value::Null => Ok(Value::Number(0.0)),
        other => Err(err(format!("len unsupported for value: {:?}", other))),
    }
}

fn display(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(display).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Object(o) => {
            let mut parts: Vec<(String, String)> =
                o.iter().map(|(k, v)| (k.clone(), display(v))).collect();
            parts.sort_by(|a, b| a.0.cmp(&b.0));
            let s = parts
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", s)
        }
        Value::Function(_) => "<fn>".to_string(),
        Value::Builtin(b) => format!("<builtin:{}>", b.name),
        
        // Quantum values
        Value::QuantumArray(a, is_superposition) => {
            let parts: Vec<String> = a.iter().map(display).collect();
            let prefix = if *is_superposition { "⊗" } else { "" };
            format!("{}[{}]", prefix, parts.join(", "))
        }
        Value::QuantumState(state, amplitude) => {
            match amplitude {
                Some(amp) => format!("{}*{}", state, amp),
                None => state.clone(),
            }
        }
        Value::QubitReference(name) => format!("⟨{}⟩", name),
    }
}

// AEONMI Quantum Built-in Functions

fn builtin_superpose(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("superpose expects 1 argument".into()));
    }
    
    match &args[0] {
        Value::QubitReference(qubit_name) => {
            interp.quantum_sim.superpose(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::QubitReference(qubit_name.clone()))
        }
        Value::String(qubit_name) => {
            // Create qubit if it doesn't exist
            if !interp.quantum_sim.qubits.contains_key(qubit_name) {
                interp.quantum_sim.create_qubit(qubit_name.clone());
            }
            interp.quantum_sim.superpose(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::QubitReference(qubit_name.clone()))
        }
        Value::Array(elements) => {
            // superpose([1.0, 0.0]) — normalize the state vector and create an anonymous qubit
            use crate::core::titan::quantum_superposition::create_superposition;
            let state_vec: Vec<f64> = elements.iter().map(|v| match v {
                Value::Number(n) => *n,
                _ => 0.0,
            }).collect();
            let normalized = create_superposition(&state_vec);
            let result: Vec<Value> = normalized.into_iter().map(Value::Number).collect();
            Ok(Value::Array(result))
        }
        _ => Err(err("superpose expects a qubit reference, name, or state vector array".into())),
    }
}

fn builtin_measure(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("measure expects 1 argument".into()));
    }
    
    match &args[0] {
        Value::QubitReference(qubit_name) => {
            let result = interp.quantum_sim.measure(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::Number(result as f64))
        }
        Value::String(qubit_name) => {
            let result = interp.quantum_sim.measure(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::Number(result as f64))
        }
        _ => Err(err("measure expects a qubit reference or name".into())),
    }
}

fn builtin_entangle(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("entangle expects 2 arguments".into()));
    }
    
    let qubit1_name = match &args[0] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("entangle expects qubit references or names".into())),
    };
    
    let qubit2_name = match &args[1] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("entangle expects qubit references or names".into())),
    };
    
    // Create qubits if they don't exist
    if !interp.quantum_sim.qubits.contains_key(&qubit1_name) {
        interp.quantum_sim.create_qubit(qubit1_name.clone());
    }
    if !interp.quantum_sim.qubits.contains_key(&qubit2_name) {
        interp.quantum_sim.create_qubit(qubit2_name.clone());
    }
    
    interp.quantum_sim.entangle(&qubit1_name, &qubit2_name)
        .map_err(|e| err(format!("Quantum error: {}", e)))?;
    
    Ok(Value::Null)
}

fn builtin_apply_gate(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("apply_gate expects 2 arguments: qubit_name, gate_name".into()));
    }
    
    let qubit_name = match &args[0] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("apply_gate first argument must be a qubit reference or name".into())),
    };
    
    let gate_name = match &args[1] {
        Value::String(name) => name.to_uppercase(),
        Value::Builtin(b) => b.name.to_uppercase(),
        _ => return Err(err("apply_gate second argument must be a gate name string or gate builtin (H, X, Y, Z, S, T, CNOT)".into())),
    };
    
    // Create qubit if it doesn't exist
    if !interp.quantum_sim.qubits.contains_key(&qubit_name) {
        interp.quantum_sim.create_qubit(qubit_name.clone());
    }
    
    match gate_name.as_str() {
        "H" | "HADAMARD" => {
            interp.quantum_sim.superpose(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying H: {}", e)))?;
        }
        "X" | "PAULI_X" | "NOT" => {
            interp.quantum_sim.pauli_x(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying X: {}", e)))?;
        }
        "Z" | "PAULI_Z" => {
            interp.quantum_sim.pauli_z(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying Z: {}", e)))?;
        }
        "Y" | "PAULI_Y" => {
            interp.quantum_sim.pauli_y(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying Y: {}", e)))?;
        }
        "S" => {
            interp.quantum_sim.phase_s(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying S: {}", e)))?;
        }
        "T" => {
            interp.quantum_sim.phase_t(&qubit_name)
                .map_err(|e| err(format!("Quantum error applying T: {}", e)))?;
        }
        "CNOT" | "CX" => {
            return Err(err("CNOT/CX requires 2 qubits: use CNOT(control, target) or entangle(q1, q2)".into()));
        }
        other => {
            return Err(err(format!("Unknown gate '{}'. Supported: H, X, Y, Z, S, T, CNOT, CX", other)));
        }
    }
    
    Ok(Value::QubitReference(qubit_name))
}

// ── Individual gate built-in functions ──────────────────────────────────────
// These allow scripts to write: H(q), X(q), CNOT(q1, q2), etc.

/// Helper: resolve qubit name from a Value
fn resolve_qubit_name(v: &Value) -> Result<String, RuntimeError> {
    match v {
        Value::QubitReference(n) | Value::String(n) => Ok(n.clone()),
        _ => Err(err("Gate argument must be a qubit reference or name".into())),
    }
}

/// Helper: ensure qubit exists in simulator, creating it if necessary
fn ensure_qubit(interp: &mut Interpreter, name: &str) {
    if !interp.quantum_sim.qubits.contains_key(name) {
        interp.quantum_sim.create_qubit(name.to_string());
    }
}

fn gate_h(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("H expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.superpose(&q).map_err(|e| err(format!("H gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_x(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("X expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.pauli_x(&q).map_err(|e| err(format!("X gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_y(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("Y expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.pauli_y(&q).map_err(|e| err(format!("Y gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_z(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("Z expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.pauli_z(&q).map_err(|e| err(format!("Z gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_s(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("S expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.phase_s(&q).map_err(|e| err(format!("S gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_t(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 { return Err(err("T expects 1 qubit argument".into())); }
    let q = resolve_qubit_name(&args[0])?;
    ensure_qubit(interp, &q);
    interp.quantum_sim.phase_t(&q).map_err(|e| err(format!("T gate error: {}", e)))?;
    Ok(Value::QubitReference(q))
}

fn gate_cnot(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 { return Err(err("CNOT/CX expects 2 qubit arguments: CNOT(control, target)".into())); }
    let ctrl = resolve_qubit_name(&args[0])?;
    let tgt  = resolve_qubit_name(&args[1])?;
    ensure_qubit(interp, &ctrl);
    ensure_qubit(interp, &tgt);
    interp.quantum_sim.apply_cnot(&ctrl, &tgt)
        .map_err(|e| err(format!("CNOT gate error: {}", e)))?;
    Ok(Value::QubitReference(ctrl))
}

// AEONMI Quantum Algorithm Built-in Functions

fn builtin_grovers_search(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("grovers_search expects 2 arguments: database_size, marked_item".into()));
    }
    
    let database_size = match &args[0] {
        Value::Number(n) => *n as usize,
        _ => return Err(err("Database size must be a number".into())),
    };
    
    let marked_item = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => return Err(err("Marked item must be a number".into())),
    };
    
    if marked_item >= database_size {
        return Err(err("Marked item index must be less than database size".into()));
    }
    
    match interp.quantum_alg.grovers_search(database_size, marked_item) {
        Ok(result) => Ok(Value::Number(result as f64)),
        Err(e) => Err(err(format!("Grover's search failed: {}", e))),
    }
}

fn builtin_qft(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("quantum_fourier_transform expects 1 argument: array of qubit names".into()));
    }
    
    let qubit_names = match &args[0] {
        Value::Array(arr) => {
            let mut names = Vec::new();
            for val in arr {
                match val {
                    Value::String(name) => names.push(name.clone()),
                    Value::QubitReference(name) => names.push(name.clone()),
                    _ => return Err(err("QFT expects array of qubit names".into())),
                }
            }
            names
        },
        _ => return Err(err("QFT expects an array of qubit names".into())),
    };
    
    match interp.quantum_alg.quantum_fourier_transform(&qubit_names) {
        Ok(_) => Ok(Value::Null),
        Err(e) => Err(err(format!("QFT failed: {}", e))),
    }
}

fn builtin_shors(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("shors_factoring expects 1 argument: number to factor".into()));
    }
    
    let n = match &args[0] {
        Value::Number(num) => *num as usize,
        _ => return Err(err("Number to factor must be a number".into())),
    };
    
    match interp.quantum_alg.shors_factoring(n) {
        Ok((factor1, factor2)) => {
            let result = vec![Value::Number(factor1 as f64), Value::Number(factor2 as f64)];
            Ok(Value::Array(result))
        },
        Err(e) => Err(err(format!("Shor's factoring failed: {}", e))),
    }
}

fn builtin_deutsch_jozsa(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("deutsch_jozsa expects 1 argument: oracle type".into()));
    }
    
    let oracle_type = match &args[0] {
        Value::String(s) => match s.as_str() {
            "constant0" => DeutschJozsaOracle::Constant0,
            "constant1" => DeutschJozsaOracle::Constant1,
            "balanced" => DeutschJozsaOracle::BalancedXor,
            _ => return Err(err("Oracle type must be 'constant0', 'constant1', or 'balanced'".into())),
        },
        _ => return Err(err("Oracle type must be a string".into())),
    };
    
    match interp.quantum_alg.deutsch_jozsa(oracle_type) {
        Ok(is_balanced) => Ok(Value::Bool(is_balanced)),
        Err(e) => Err(err(format!("Deutsch-Jozsa failed: {}", e))),
    }
}

fn builtin_bernstein_vazirani(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("bernstein_vazirani expects 1 argument: hidden bit string".into()));
    }
    
    let hidden_string = match &args[0] {
        Value::Array(arr) => {
            let mut bits = Vec::new();
            for val in arr {
                match val {
                    Value::Bool(b) => bits.push(*b),
                    Value::Number(n) => bits.push(*n != 0.0),
                    _ => return Err(err("Hidden string must be array of booleans or numbers".into())),
                }
            }
            bits
        },
        Value::String(s) => {
            s.chars().map(|c| c == '1').collect()
        },
        _ => return Err(err("Hidden string must be array or string".into())),
    };
    
    match interp.quantum_alg.bernstein_vazirani(&hidden_string) {
        Ok(result) => {
            let result_values: Vec<Value> = result.into_iter().map(|b| Value::Bool(b)).collect();
            Ok(Value::Array(result_values))
        },
        Err(e) => Err(err(format!("Bernstein-Vazirani failed: {}", e))),
    }
}

fn builtin_quantum_teleportation(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("quantum_teleportation expects 1 argument: quantum state to teleport".into()));
    }
    
    let state = match &args[0] {
        Value::String(s) => s.clone(),
        Value::QuantumState(state, _) => state.clone(),
        _ => return Err(err("State to teleport must be a quantum state string".into())),
    };
    
    match interp.quantum_alg.quantum_teleportation(&state) {
        Ok(result_state) => Ok(Value::String(result_state)),
        Err(e) => Err(err(format!("Quantum teleportation failed: {}", e))),
    }
}

// AEONMI Hardware Integration Built-in Functions

fn builtin_list_devices(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(err("list_devices expects no arguments".into()));
    }
    
    let devices = interp.hardware_mgr.list_devices();
    let device_list: Vec<Value> = devices.into_iter().map(|device| {
        let mut device_info = std::collections::HashMap::new();
        device_info.insert("name".to_string(), Value::String(device.name.clone()));
        device_info.insert("provider".to_string(), Value::String(device.provider.to_string()));
        device_info.insert("qubits".to_string(), Value::Number(device.qubits as f64));
        device_info.insert("available".to_string(), Value::Bool(device.is_available));
        device_info.insert("queue_length".to_string(), Value::Number(device.queue_length as f64));
        Value::Object(device_info)
    }).collect();
    
    Ok(Value::Array(device_list))
}

fn builtin_submit_job(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(err("submit_job expects 3 arguments: device_name, circuit_gates, shots".into()));
    }
    
    let device_name = match &args[0] {
        Value::String(name) => name.clone(),
        _ => return Err(err("Device name must be a string".into())),
    };
    
    let shots = match &args[2] {
        Value::Number(n) => *n as usize,
        _ => return Err(err("Shots must be a number".into())),
    };
    
    // Parse circuit gates from array or object
    let mut circuit = QuantumCircuit::new(2); // Default 2 qubits for now
    
    match &args[1] {
        Value::Array(gates) => {
            for gate in gates {
                match gate {
                    Value::String(gate_str) => {
                        // Simple gate parsing: "h 0", "cx 0 1", etc.
                        let parts: Vec<&str> = gate_str.split_whitespace().collect();
                        if parts.is_empty() {
                            continue;
                        }
                        
                        match parts[0] {
                            "h" if parts.len() == 2 => {
                                if let Ok(qubit) = parts[1].parse::<usize>() {
                                    circuit.h(qubit);
                                }
                            }
                            "x" if parts.len() == 2 => {
                                if let Ok(qubit) = parts[1].parse::<usize>() {
                                    circuit.x(qubit);
                                }
                            }
                            "cx" if parts.len() == 3 => {
                                if let (Ok(control), Ok(target)) = (parts[1].parse::<usize>(), parts[2].parse::<usize>()) {
                                    circuit.cx(control, target);
                                }
                            }
                            _ => {} // Ignore unknown gates
                        }
                    }
                    _ => {} // Ignore non-string gate specifications
                }
            }
        }
        _ => return Err(err("Circuit gates must be an array of gate strings".into())),
    }
    
    circuit.measure_all();
    
    match interp.hardware_mgr.submit_job(&device_name, circuit, shots) {
        Ok(job_id) => Ok(Value::String(job_id)),
        Err(e) => Err(err(format!("Job submission failed: {}", e))),
    }
}

fn builtin_job_status(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("job_status expects 1 argument: job_id".into()));
    }
    
    let job_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => return Err(err("Job ID must be a string".into())),
    };
    
    match interp.hardware_mgr.get_job_status(&job_id) {
        Some(status) => {
            let status_str = match status {
                crate::core::hardware_integration::JobStatus::Queued => "queued",
                crate::core::hardware_integration::JobStatus::Running => "running",
                crate::core::hardware_integration::JobStatus::Completed => "completed",
                crate::core::hardware_integration::JobStatus::Failed(_) => "failed",
                crate::core::hardware_integration::JobStatus::Cancelled => "cancelled",
            };
            Ok(Value::String(status_str.to_string()))
        }
        None => Err(err("Job not found".into())),
    }
}

fn builtin_job_results(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("job_results expects 1 argument: job_id".into()));
    }
    
    let job_id = match &args[0] {
        Value::String(id) => id.clone(),
        _ => return Err(err("Job ID must be a string".into())),
    };
    
    match interp.hardware_mgr.get_job_results(&job_id) {
        Some(results) => {
            let mut result_obj = std::collections::HashMap::new();
            
            // Convert counts to AEONMI value format
            let counts: std::collections::HashMap<String, Value> = results.counts.iter()
                .map(|(k, v)| (k.clone(), Value::Number(*v as f64)))
                .collect();
            result_obj.insert("counts".to_string(), Value::Object(counts));
            
            // Convert probabilities to AEONMI value format  
            let probabilities: std::collections::HashMap<String, Value> = results.probabilities.iter()
                .map(|(k, v)| (k.clone(), Value::Number(*v)))
                .collect();
            result_obj.insert("probabilities".to_string(), Value::Object(probabilities));
            
            result_obj.insert("execution_time".to_string(), Value::Number(results.execution_time));
            result_obj.insert("shots".to_string(), Value::Number(results.raw_data.len() as f64));
            
            Ok(Value::Object(result_obj))
        }
        None => Err(err("Job results not available (job may not be completed)".into())),
    }
}

// AEONMI Hieroglyphic & Utility Built-in Functions

fn builtin_glyph(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    // __glyph(symbol, arg1, arg2, ...)
    // Executes a hieroglyphic operation. For now: print the invocation and return null.
    // Future: dispatch to glyph-specific logic based on the symbol.
    let symbol = match args.first() {
        Some(Value::String(s)) => s.clone(),
        _ => "unknown".to_string(),
    };
    let glyph_args: Vec<String> = args.iter().skip(1).map(display).collect();
    println!("[glyph] {} ({})", symbol, glyph_args.join(", "));
    Ok(Value::Null)
}

fn builtin_typeof(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("typeof expects 1 argument".into()));
    }
    let type_name = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Function(_) => "function",
        Value::Builtin(_) => "builtin",
        Value::QuantumArray(_, _) => "quantum_array",
        Value::QuantumState(_, _) => "quantum_state",
        Value::QubitReference(_) => "qubit",
    };
    Ok(Value::String(type_name.to_string()))
}

fn builtin_to_string(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("toString expects 1 argument".into()));
    }
    Ok(Value::String(display(&args[0])))
}

fn builtin_to_number(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("toNumber expects 1 argument".into()));
    }
    let n = match &args[0] {
        Value::Number(n) => *n,
        Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
        Value::Null => 0.0,
        _ => f64::NAN,
    };
    Ok(Value::Number(n))
}

fn builtin_read_file(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("read_file expects 1 argument: path".into()));
    }
    let path = match &args[0] {
        Value::String(s) => s.clone(),
        other => display(other),
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Value::String(content)),
        Err(e) => Err(err(format!("read_file: cannot read '{}': {}", path, e))),
    }
}

fn builtin_write_file(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("write_file expects 2 arguments: path, content".into()));
    }
    let path = match &args[0] {
        Value::String(s) => s.clone(),
        other => display(other),
    };
    let content = match &args[1] {
        Value::String(s) => s.clone(),
        other => display(other),
    };
    match std::fs::write(&path, &content) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(e) => Err(err(format!("write_file: cannot write '{}': {}", path, e))),
    }
}

fn builtin_file_exists(_interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("file_exists expects 1 argument: path".into()));
    }
    let path = match &args[0] {
        Value::String(s) => s.clone(),
        other => display(other),
    };
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}
