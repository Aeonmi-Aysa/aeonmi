//! Aeonmi VM: tree-walk interpreter over IR with quantum simulation support.
//! Supports: literals, quantum arrays/objects, let/assign, if/while/for, fn calls/returns,
//! binary/unary ops, quantum operations, and built-ins: print, log, time_ms, rand, len.

#![allow(dead_code)]

use crate::core::hardware_integration::{HardwareManager, QuantumCircuit};
use crate::core::ir::*;
use crate::core::quantum_algorithms::{DeutschJozsaOracle, QuantumAlgorithms};
use crate::core::quantum_simulator::{Complex, QuantumSimulator};
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
    QuantumArray(Vec<Value>, bool),    // elements, is_superposition
    QuantumState(String, Option<f64>), // state, amplitude
    QubitReference(String),            // reference to qubit in simulator
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

    pub fn define_global(&mut self, k: String, v: Value) {
        if let Some(global) = self.frames.first_mut() {
            global.insert(k, v);
        } else {
            self.define(k, v);
        }
    }

    pub fn extend_global<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        if let Some(global) = self.frames.first_mut() {
            for (k, v) in iter {
                global.insert(k, v);
            }
        } else {
            let mut frame = HashMap::new();
            for (k, v) in iter {
                frame.insert(k, v);
            }
            self.frames.push(frame);
        }
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

    pub fn merge_from_prefix(&mut self, other: &Env) {
        let len = self.frames.len().min(other.frames.len());
        for idx in 0..len {
            let src = &other.frames[idx];
            let dest = &mut self.frames[idx];
            for (k, v) in src {
                dest.insert(k.clone(), v.clone());
            }
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub env: Env,
    pub quantum_sim: QuantumSimulator, // Quantum simulator for quantum operations
    pub quantum_alg: QuantumAlgorithms, // Quantum algorithms library
    pub hardware_mgr: HardwareManager, // Real quantum hardware integration
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
            "is_entangled".into(),
            Value::Builtin(Builtin {
                name: "is_entangled",
                arity: 2,
                f: builtin_is_entangled,
            }),
        );

        env.define(
            "apply_matrix".into(),
            Value::Builtin(Builtin {
                name: "apply_matrix",
                arity: 2,
                f: builtin_apply_matrix,
            }),
        );

        env.define(
            "__quantum_index".into(),
            Value::Builtin(Builtin {
                name: "__quantum_index",
                arity: 2,
                f: builtin_quantum_index_internal,
            }),
        );

        env.define(
            "__quantum_protect_loop".into(),
            Value::Builtin(Builtin {
                name: "__quantum_protect_loop",
                arity: 1,
                f: builtin_quantum_protect_loop_internal,
            }),
        );

        env.define(
            "__quantum_try_catch".into(),
            Value::Builtin(Builtin {
                name: "__quantum_try_catch",
                arity: 3,
                f: builtin_quantum_try_catch_internal,
            }),
        );

        env.define(
            "__quantum_probability_branch".into(),
            Value::Builtin(Builtin {
                name: "__quantum_probability_branch",
                arity: usize::MAX,
                f: builtin_quantum_probability_branch_internal,
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

        Self {
            env,
            quantum_sim: QuantumSimulator::new(),
            quantum_alg: QuantumAlgorithms::new(),
            hardware_mgr: HardwareManager::new(),
        }
    }

    pub fn run_module(&mut self, m: &Module) -> Result<(), RuntimeError> {
        debug_log!("vm: run_module decls={} ", m.decls.len());
        // Load top-level decls
        for d in &m.decls {
            debug_log!("vm: processing decl: {:?}", d);
            match d {
                Decl::Const(c) => {
                    let v = self.eval_expr(&c.value)?;
                    self.env.define(c.name.clone(), v);
                }
                Decl::Let(l) => {
                    let value = if let Some(e) = &l.value {
                        self.eval_expr(e)?
                    } else {
                        Value::Null
                    };
                    self.env.define(l.name.clone(), value);
                }
                Decl::QuantumLet(q) => {
                    let evaluated = if let Some(expr) = &q.value {
                        self.eval_expr(expr)?
                    } else {
                        Value::Null
                    };
                    let bound = self.initialize_quantum_binding(&q.name, q.binding, evaluated)?;
                    self.env.define_global(q.name.clone(), bound);
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
                let mut saved = self.env.clone();
                self.env = fun.env.clone();
                self.env.push();
                for (p, v) in fun.params.iter().zip(args.into_iter()) {
                    self.env.define(p.clone(), v);
                }
                // Execute - don't create another scope in exec_block for function bodies
                let ret = self.exec_function_block(&fun.body);
                let mut post_call_env = self.env.clone();
                let extra_frame = if post_call_env.frames.len() > fun.env.frames.len() {
                    post_call_env.frames.pop()
                } else {
                    None
                };
                saved.merge_from_prefix(&post_call_env);
                if let Some(frame) = extra_frame {
                    saved.extend_global(frame.into_iter());
                }
                // Restore
                self.env = saved;
                match ret {
                    ControlFlow::Ok => Ok(Value::Null),
                    ControlFlow::Return(v) => Ok(v.unwrap_or(Value::Null)),
                    ControlFlow::Err(e) => Err(e),
                }
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

    fn exec_stmt(&mut self, stmt: &Stmt) -> ControlFlow {
        use Stmt::*;
        match stmt {
            Expr(expr) => match self.eval_expr(expr) {
                Ok(_) => ControlFlow::Ok,
                Err(e) => ControlFlow::Err(e),
            },
            Return(Some(e)) => {
                let v = match self.eval_expr(e) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };
                ControlFlow::Return(Some(v))
            }
            Return(None) => ControlFlow::Return(None),
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
                        if let Err(e) = self.eval_expr(st) {
                            return ControlFlow::Err(e);
                        }
                    }
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
            QuantumLet {
                name,
                binding,
                value,
            } => {
                let evaluated = match self.eval_expr(value) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };
                match self.initialize_quantum_binding(name, *binding, evaluated) {
                    Ok(bound) => {
                        self.env.define_global(name.clone(), bound);
                        ControlFlow::Ok
                    }
                    Err(e) => ControlFlow::Err(e),
                }
            }
            ProbabilityBranch {
                condition,
                probability,
                then_block,
                else_block,
            } => {
                let condition_value = match self.eval_expr(condition) {
                    Ok(v) => v,
                    Err(e) => return ControlFlow::Err(e),
                };

                let mut weight = match probability {
                    Some(p) => *p,
                    None => match self.value_to_probability(condition_value) {
                        Ok(p) => p,
                        Err(e) => return ControlFlow::Err(e),
                    },
                };

                if !weight.is_finite() {
                    return ControlFlow::Err(err("Probability must be finite".into()));
                }
                if weight.is_nan() {
                    weight = 0.0;
                }
                if weight < 0.0 {
                    weight = 0.0;
                }
                if weight > 1.0 {
                    weight = 1.0;
                }

                let roll = lcg_unit();
                if roll < weight {
                    self.exec_block(then_block)
                } else if let Some(else_blk) = else_block {
                    self.exec_block(else_blk)
                } else {
                    ControlFlow::Ok
                }
            }
            Assign { target, value } => {
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
            QuantumState { label, amplitude } => Value::QuantumState(label.clone(), *amplitude),
            QuantumArray {
                elements,
                is_superposition,
            } => {
                let mut values = Vec::with_capacity(elements.len());
                for el in elements {
                    values.push(self.eval_expr(el)?);
                }
                Value::QuantumArray(values, *is_superposition)
            }
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
                // Fast path: direct ident call (avoids allocating callee Value if builtin/func)
                if let Expr::Ident(name) = &**callee {
                    let argv = collect_vals(self, args)?;
                    self.call_ident(name, argv)?
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
            Index { target, index } => {
                let target_val = self.eval_expr(target)?;
                let index_val = self.eval_expr(index)?;
                match target_val {
                    Value::Array(items) => {
                        let idx = match index_val {
                            Value::Number(n) if n.fract() == 0.0 => n as usize,
                            Value::Number(_) => {
                                return Err(err("Array index must be an integer".into()))
                            }
                            other => {
                                return Err(err(format!(
                                    "Array index must be a number, got {:?}",
                                    other
                                )))
                            }
                        };
                        items
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| err(format!("Index {} out of bounds", idx)))?
                    }
                    Value::QuantumArray(items, _) => {
                        let idx = match index_val {
                            Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => n as usize,
                            _ => return Err(err("Index must be a non-negative integer".into())),
                        };
                        items
                            .get(idx)
                            .cloned()
                            .ok_or_else(|| err(format!("Quantum index {} out of bounds", idx)))?
                    }
                    Value::Object(map) => {
                        let key = match index_val {
                            Value::String(s) => s,
                            Value::Number(n) if n.fract() == 0.0 => n.to_string(),
                            other => {
                                return Err(err(format!(
                                    "Object index must be string or integer-like, got {:?}",
                                    other
                                )))
                            }
                        };
                        map.get(&key).cloned().unwrap_or(Value::Null)
                    }
                    Value::String(s) => {
                        let idx = match index_val {
                            Value::Number(n) if n.fract() == 0.0 => n as usize,
                            _ => return Err(err("String index must be an integer number".into())),
                        };
                        s.chars()
                            .nth(idx)
                            .map(|c| Value::String(c.to_string()))
                            .unwrap_or(Value::Null)
                    }
                    other => {
                        return Err(err(format!("Indexing not supported on value {:?}", other)))
                    }
                }
            }
            Object(kvs) => {
                let mut map = HashMap::with_capacity(kvs.len());
                for (k, v) in kvs {
                    map.insert(k.clone(), self.eval_expr(v)?);
                }
                Value::Object(map)
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
                (Value::QuantumState(label_a, amp_a), Value::QuantumState(label_b, amp_b)) => {
                    Ok(Value::QuantumArray(
                        vec![
                            Value::QuantumState(label_a, amp_a),
                            Value::QuantumState(label_b, amp_b),
                        ],
                        true,
                    ))
                }
                (Value::QuantumArray(mut items, _), Value::QuantumState(label, amp)) => {
                    items.push(Value::QuantumState(label, amp));
                    Ok(Value::QuantumArray(items, true))
                }
                (Value::QuantumState(label, amp), Value::QuantumArray(mut items, _)) => {
                    let mut new_items = vec![Value::QuantumState(label, amp)];
                    new_items.append(&mut items);
                    Ok(Value::QuantumArray(new_items, true))
                }
                (
                    Value::QuantumArray(mut left, left_super),
                    Value::QuantumArray(mut right, right_super),
                ) => {
                    let mut result = Vec::with_capacity(left.len() + right.len());
                    result.append(&mut left);
                    result.append(&mut right);
                    let has_multi = result.len() > 1;
                    Ok(Value::QuantumArray(
                        result,
                        left_super || right_super || has_multi,
                    ))
                }
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

    fn initialize_quantum_binding(
        &mut self,
        name: &str,
        binding: QuantumBinding,
        value: Value,
    ) -> Result<Value, RuntimeError> {
        use QuantumBinding::*;
        match binding {
            Classical => self.prepare_single_qubit(name, value, false),
            Superposition => self.prepare_single_qubit(name, value, true),
            Approximation => self.prepare_single_qubit(name, value, true),
            Tensor => self.prepare_tensor(name, value),
        }
    }

    fn prepare_single_qubit(
        &mut self,
        name: &str,
        value: Value,
        allow_superposition: bool,
    ) -> Result<Value, RuntimeError> {
        match value {
            Value::QuantumState(label, amplitude) => {
                self.quantum_sim.create_qubit(name.to_string());
                self.quantum_sim
                    .prepare_named_state(name, &label, amplitude)
                    .map_err(|e| err(format!("Quantum state error: {}", e)))?;
                Ok(Value::QubitReference(name.to_string()))
            }
            Value::QuantumArray(elements, is_superposition) => {
                if !allow_superposition && (is_superposition || elements.len() > 1) {
                    return Err(err(format!(
                        "Quantum binding `{}` expects classical state but received superposition",
                        name
                    )));
                }
                let components = collect_state_components(elements)?;
                self.quantum_sim.create_qubit(name.to_string());
                self.quantum_sim
                    .prepare_state_from_components(name, &components)
                    .map_err(|e| err(format!("Quantum state error: {}", e)))?;
                Ok(Value::QubitReference(name.to_string()))
            }
            Value::Array(elements) => {
                let is_superposition = allow_superposition && elements.len() > 1;
                self.prepare_single_qubit(
                    name,
                    Value::QuantumArray(elements, is_superposition),
                    allow_superposition,
                )
            }
            Value::String(label) if label.trim().starts_with('|') => self.prepare_single_qubit(
                name,
                Value::QuantumState(label, None),
                allow_superposition,
            ),
            Value::QubitReference(existing) => {
                if existing == name {
                    if !self.quantum_sim.qubits.contains_key(&existing) {
                        self.quantum_sim.create_qubit(existing.clone());
                    }
                    Ok(Value::QubitReference(existing))
                } else {
                    Err(err(format!(
                        "Cannot alias existing qubit `{}` into `{}` in this revision",
                        existing, name
                    )))
                }
            }
            other => Err(err(format!(
                "Invalid quantum initializer for `{}`: {:?}",
                name, other
            ))),
        }
    }

    fn prepare_tensor(&mut self, name: &str, value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::QuantumArray(elements, is_superposition) => {
                let mut refs = Vec::with_capacity(elements.len());
                let mut aggregated_super = is_superposition;
                for (idx, element) in elements.into_iter().enumerate() {
                    let element_super = match &element {
                        Value::QuantumArray(inner, flag) => *flag || inner.len() > 1,
                        Value::Array(inner) => inner.len() > 1,
                        _ => false,
                    };
                    aggregated_super |= element_super;
                    let element_name = format!("{}[{}]", name, idx);
                    let reference = self.prepare_single_qubit(&element_name, element, true)?;
                    refs.push(reference);
                }
                Ok(Value::QuantumArray(refs, aggregated_super))
            }
            Value::Array(elements) => {
                self.prepare_tensor(name, Value::QuantumArray(elements, false))
            }
            other => Err(err(format!(
                "Tensor binding `{}` requires quantum array literal, got {:?}",
                name, other
            ))),
        }
    }

    fn value_to_probability(&self, value: Value) -> Result<f64, RuntimeError> {
        match value {
            Value::Number(n) => Ok(n.clamp(0.0, 1.0)),
            Value::Bool(b) => Ok(if b { 1.0 } else { 0.0 }),
            Value::QuantumState(label, amplitude) => {
                let components = vec![(label, amplitude)];
                self.components_probability(&components)
            }
            Value::QuantumArray(elements, _) => {
                let components = collect_state_components(elements)?;
                self.components_probability(&components)
            }
            Value::Array(elements) => {
                let components = collect_state_components(elements)?;
                self.components_probability(&components)
            }
            Value::QubitReference(name) => {
                let zero_prob = self
                    .quantum_sim
                    .get_zero_probability(&name)
                    .map_err(|e| err(format!("Quantum probability error: {}", e)))?;
                Ok((1.0 - zero_prob).clamp(0.0, 1.0))
            }
            other => Err(err(format!(
                "Cannot derive probability from value {:?}",
                other
            ))),
        }
    }

    fn components_probability(
        &self,
        components: &[(String, Option<f64>)],
    ) -> Result<f64, RuntimeError> {
        self.quantum_sim
            .probability_from_components(components)
            .map_err(|e| err(format!("Quantum probability error: {}", e)))
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

fn collect_state_components(values: Vec<Value>) -> Result<Vec<(String, Option<f64>)>, RuntimeError> {
    let mut components = Vec::new();
    for value in values {
        match value {
            Value::QuantumState(label, amplitude) => components.push((label, amplitude)),
            Value::String(s) if s.trim().starts_with('|') => components.push((s, None)),
            Value::QuantumArray(inner, _) => components.extend(collect_state_components(inner)?),
            Value::Array(inner) => components.extend(collect_state_components(inner)?),
            other => {
                return Err(err(format!(
                    "Expected quantum state component, got {:?}",
                    other
                )));
            }
        }
    }

    if components.is_empty() {
        return Err(err(
            "Quantum state definition requires at least one component".into(),
        ));
    }

    Ok(components)
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

        (QuantumArray(x, xs), QuantumArray(y, ys)) => {
            if xs != ys || x.len() != y.len() {
                return false;
            }
            x.iter().zip(y.iter()).all(|(lx, ry)| eq_val(lx, ry))
        }
        (QuantumState(ax, ay), QuantumState(bx, by)) => ax == bx && ay == by,
        (QubitReference(aq), QubitReference(bq)) => aq == bq,

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

fn lcg_unit() -> f64 {
    let x = lcg_next();
    let mantissa = x >> 11; // keep 53 bits for double precision
    (mantissa as f64) / ((1u64 << 53) as f64)
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
        Value::QuantumArray(items, _) => Ok(Value::Number(items.len() as f64)),
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
        Value::QuantumState(state, amplitude) => match amplitude {
            Some(amp) => format!("{}*{}", state, amp),
            None => state.clone(),
        },
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
            interp
                .quantum_sim
                .superpose(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::Null)
        }
        Value::String(qubit_name) => {
            // Create qubit if it doesn't exist
            if !interp.quantum_sim.qubits.contains_key(qubit_name) {
                interp.quantum_sim.create_qubit(qubit_name.clone());
            }
            interp
                .quantum_sim
                .superpose(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::QubitReference(qubit_name.clone()))
        }
        _ => Err(err("superpose expects a qubit reference or name".into())),
    }
}

fn builtin_measure(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("measure expects 1 argument".into()));
    }

    match &args[0] {
        Value::QubitReference(qubit_name) => {
            let result = interp
                .quantum_sim
                .measure(qubit_name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::Number(result as f64))
        }
        Value::String(qubit_name) => {
            let result = interp
                .quantum_sim
                .measure(qubit_name)
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

    interp
        .quantum_sim
        .entangle(&qubit1_name, &qubit2_name)
        .map_err(|e| err(format!("Quantum error: {}", e)))?;

    Ok(Value::Null)
}

fn builtin_is_entangled(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("is_entangled expects 2 arguments".into()));
    }

    let name_a = match &args[0] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("is_entangled expects qubit references or names".into())),
    };

    let name_b = match &args[1] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("is_entangled expects qubit references or names".into())),
    };

    let entangled = interp
        .quantum_sim
        .entangled_systems
        .iter()
        .any(|group| group.contains(&name_a) && group.contains(&name_b));

    Ok(Value::Bool(entangled))
}

fn builtin_apply_matrix(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("apply_matrix expects 2 arguments".into()));
    }

    let qubit_name = match &args[0] {
        Value::QubitReference(name) => name.clone(),
        Value::String(name) => name.clone(),
        _ => return Err(err("apply_matrix expects a qubit reference or name".into())),
    };

    let matrix = match &args[1] {
        Value::Array(rows) if rows.len() == 2 => {
            let mut parsed = [[0.0_f64; 2]; 2];
            for (row_idx, row_val) in rows.iter().enumerate() {
                match row_val {
                    Value::Array(cols) if cols.len() == 2 => {
                        for (col_idx, col_val) in cols.iter().enumerate() {
                            parsed[row_idx][col_idx] = match col_val {
                                Value::Number(n) => *n,
                                _ => {
                                    return Err(err(
                                        "apply_matrix expects numeric entries in matrix".into(),
                                    ))
                                }
                            };
                        }
                    }
                    _ => return Err(err("apply_matrix expects a 2x2 numeric matrix".into())),
                }
            }
            parsed
        }
        _ => return Err(err("apply_matrix expects a 2x2 numeric matrix".into())),
    };

    if !interp.quantum_sim.qubits.contains_key(&qubit_name) {
        interp.quantum_sim.create_qubit(qubit_name.clone());
    }

    if let Some(state) = interp.quantum_sim.qubits.get_mut(&qubit_name) {
        if state.amplitudes.len() < 2 {
            state.amplitudes.resize(2, Complex::new(0.0, 0.0));
            state.num_qubits = 1;
        }

        let amp0 = state.amplitudes[0];
        let amp1 = state.amplitudes[1];

        let a = Complex::new(matrix[0][0], 0.0);
        let b = Complex::new(matrix[0][1], 0.0);
        let c = Complex::new(matrix[1][0], 0.0);
        let d = Complex::new(matrix[1][1], 0.0);

        state.amplitudes[0] = a * amp0 + b * amp1;
        state.amplitudes[1] = c * amp0 + d * amp1;
        state.normalize();
    }

    Ok(Value::Null)
}

fn builtin_quantum_index_internal(
    _interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err("__quantum_index expects 2 arguments".into()));
    }

    let mut iter = args.into_iter();
    let container = iter.next().unwrap();
    let index = iter.next().unwrap();

    let idx = match index {
        Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => n as usize,
        _ => return Err(err("Index must be a non-negative integer".into())),
    };

    match container {
        Value::QuantumArray(items, _) => items
            .get(idx)
            .cloned()
            .ok_or_else(|| err("Quantum index out of bounds".into())),
        Value::Array(items) => items
            .get(idx)
            .cloned()
            .ok_or_else(|| err("Index out of bounds".into())),
        _ => Err(err(
            "__quantum_index expects an array or quantum array as the first argument".into(),
        )),
    }
}

fn builtin_quantum_protect_loop_internal(
    _interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("__quantum_protect_loop expects 1 argument".into()));
    }
    Ok(args.into_iter().next().unwrap())
}

fn builtin_quantum_try_catch_internal(
    _interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(err("__quantum_try_catch expects 3 arguments".into()));
    }
    Ok(Value::Bool(true))
}

fn builtin_quantum_probability_branch_internal(
    interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(err(
            "__quantum_probability_branch expects at least 1 argument".into(),
        ));
    }

    let mut iter = args.into_iter();
    let condition = iter.next().unwrap();
    let probability = iter.next().and_then(|v| match v {
        Value::Number(n) => Some(n.clamp(0.0, 1.0)),
        _ => None,
    });

    if let Some(p) = probability {
        let random = (lcg_next() as f64) / (u64::MAX as f64);
        return Ok(Value::Bool(random < p));
    }

    match condition {
        Value::QubitReference(name) => {
            let measured = interp
                .quantum_sim
                .measure(&name)
                .map_err(|e| err(format!("Quantum error: {}", e)))?;
            Ok(Value::Bool(measured != 0))
        }
        other => Ok(Value::Bool(interp.truthy(&other))),
    }
}

// AEONMI Quantum Algorithm Built-in Functions

fn builtin_grovers_search(
    interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err(
            "grovers_search expects 2 arguments: database_size, marked_item".into(),
        ));
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
        return Err(err(
            "Marked item index must be less than database size".into()
        ));
    }

    match interp
        .quantum_alg
        .grovers_search(database_size, marked_item)
    {
        Ok(result) => Ok(Value::Number(result as f64)),
        Err(e) => Err(err(format!("Grover's search failed: {}", e))),
    }
}

fn builtin_qft(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err(
            "quantum_fourier_transform expects 1 argument: array of qubit names".into(),
        ));
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
        }
        _ => return Err(err("QFT expects an array of qubit names".into())),
    };

    match interp.quantum_alg.quantum_fourier_transform(&qubit_names) {
        Ok(_) => Ok(Value::Null),
        Err(e) => Err(err(format!("QFT failed: {}", e))),
    }
}

fn builtin_shors(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err(
            "shors_factoring expects 1 argument: number to factor".into()
        ));
    }

    let n = match &args[0] {
        Value::Number(num) => *num as usize,
        _ => return Err(err("Number to factor must be a number".into())),
    };

    match interp.quantum_alg.shors_factoring(n) {
        Ok((factor1, factor2)) => {
            let result = vec![Value::Number(factor1 as f64), Value::Number(factor2 as f64)];
            Ok(Value::Array(result))
        }
        Err(e) => Err(err(format!("Shor's factoring failed: {}", e))),
    }
}

fn builtin_deutsch_jozsa(
    interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err("deutsch_jozsa expects 1 argument: oracle type".into()));
    }

    let oracle_type = match &args[0] {
        Value::String(s) => match s.as_str() {
            "constant0" => DeutschJozsaOracle::Constant0,
            "constant1" => DeutschJozsaOracle::Constant1,
            "balanced" => DeutschJozsaOracle::BalancedXor,
            _ => {
                return Err(err(
                    "Oracle type must be 'constant0', 'constant1', or 'balanced'".into(),
                ))
            }
        },
        _ => return Err(err("Oracle type must be a string".into())),
    };

    match interp.quantum_alg.deutsch_jozsa(oracle_type) {
        Ok(is_balanced) => Ok(Value::Bool(is_balanced)),
        Err(e) => Err(err(format!("Deutsch-Jozsa failed: {}", e))),
    }
}

fn builtin_bernstein_vazirani(
    interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err(
            "bernstein_vazirani expects 1 argument: hidden bit string".into(),
        ));
    }

    let hidden_string = match &args[0] {
        Value::Array(arr) => {
            let mut bits = Vec::new();
            for val in arr {
                match val {
                    Value::Bool(b) => bits.push(*b),
                    Value::Number(n) => bits.push(*n != 0.0),
                    _ => {
                        return Err(err(
                            "Hidden string must be array of booleans or numbers".into()
                        ))
                    }
                }
            }
            bits
        }
        Value::String(s) => s.chars().map(|c| c == '1').collect(),
        _ => return Err(err("Hidden string must be array or string".into())),
    };

    match interp.quantum_alg.bernstein_vazirani(&hidden_string) {
        Ok(result) => {
            let result_values: Vec<Value> = result.into_iter().map(|b| Value::Bool(b)).collect();
            Ok(Value::Array(result_values))
        }
        Err(e) => Err(err(format!("Bernstein-Vazirani failed: {}", e))),
    }
}

fn builtin_quantum_teleportation(
    interp: &mut Interpreter,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(err(
            "quantum_teleportation expects 1 argument: quantum state to teleport".into(),
        ));
    }

    let state = match &args[0] {
        Value::String(s) => s.clone(),
        Value::QuantumState(state, _) => state.clone(),
        _ => {
            return Err(err(
                "State to teleport must be a quantum state string".into()
            ))
        }
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
    let device_list: Vec<Value> = devices
        .into_iter()
        .map(|device| {
            let mut device_info = std::collections::HashMap::new();
            device_info.insert("name".to_string(), Value::String(device.name.clone()));
            device_info.insert(
                "provider".to_string(),
                Value::String(device.provider.to_string()),
            );
            device_info.insert("qubits".to_string(), Value::Number(device.qubits as f64));
            device_info.insert("available".to_string(), Value::Bool(device.is_available));
            device_info.insert(
                "queue_length".to_string(),
                Value::Number(device.queue_length as f64),
            );
            Value::Object(device_info)
        })
        .collect();

    Ok(Value::Array(device_list))
}

fn builtin_submit_job(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(err(
            "submit_job expects 3 arguments: device_name, circuit_gates, shots".into(),
        ));
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
                                if let (Ok(control), Ok(target)) =
                                    (parts[1].parse::<usize>(), parts[2].parse::<usize>())
                                {
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
            let counts: std::collections::HashMap<String, Value> = results
                .counts
                .iter()
                .map(|(k, v)| (k.clone(), Value::Number(*v as f64)))
                .collect();
            result_obj.insert("counts".to_string(), Value::Object(counts));

            // Convert probabilities to AEONMI value format
            let probabilities: std::collections::HashMap<String, Value> = results
                .probabilities
                .iter()
                .map(|(k, v)| (k.clone(), Value::Number(*v)))
                .collect();
            result_obj.insert("probabilities".to_string(), Value::Object(probabilities));

            result_obj.insert(
                "execution_time".to_string(),
                Value::Number(results.execution_time),
            );
            result_obj.insert(
                "shots".to_string(),
                Value::Number(results.raw_data.len() as f64),
            );

            Ok(Value::Object(result_obj))
        }
        None => Err(err(
            "Job results not available (job may not be completed)".into()
        )),
    }
}
