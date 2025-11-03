//! Simple bytecode VM (feature: bytecode)
use crate::core::bytecode::{Chunk, Constant, OpCode};
use rand::random;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug)]
struct Frame {
    return_ip: usize,
    locals: Vec<Value>,
}

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    stack: Vec<Value>,
    ip: usize,
    frames: Vec<Frame>,
    pub stack_overflow: bool,
    max_frames: usize,
    globals: Vec<Value>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        let max_frames = std::env::var("AEONMI_MAX_FRAMES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .map(|n| n.clamp(4, 65_536))
            .unwrap_or(256);
        Self {
            chunk,
            stack: Vec::new(),
            ip: 0,
            frames: vec![Frame {
                return_ip: usize::MAX,
                locals: vec![Value::Null; 64],
            }],
            stack_overflow: false,
            max_frames,
            globals: Vec::new(),
        }
    }
    pub fn run(&mut self) -> Option<Value> {
        while self.ip < self.chunk.code.len() {
            match self.chunk.code[self.ip] {
                op => {
                    self.ip += 1;
                    if !self.dispatch(op) {
                        break;
                    }
                }
            }
        }
        self.stack.pop()
    }

    fn dispatch(&mut self, op: OpCode) -> bool {
        use OpCode::*;
        match op {
            LoadConst(i) => {
                let c = &self.chunk.constants[i as usize];
                self.stack.push(match c {
                    Constant::Number(n) => Value::Number(*n),
                    Constant::String(s) => Value::String(s.clone()),
                    Constant::Bool(b) => Value::Bool(*b),
                    Constant::Null => Value::Null,
                });
            }
            LoadLocal(i) => {
                if let Some(frame) = self.frames.last() {
                    let idx = i as usize;
                    let v = frame.locals.get(idx).cloned().unwrap_or(Value::Null);
                    self.stack.push(v);
                }
            }
            StoreLocal(i) => {
                if let Some(frame) = self.frames.last_mut() {
                    let idx = i as usize;
                    if idx >= frame.locals.len() {
                        frame.locals.resize(idx + 1, Value::Null);
                    }
                    if let Some(v) = self.stack.last().cloned() {
                        frame.locals[idx] = v;
                    }
                }
            }
            LoadGlobal(i) => {
                let idx = i as usize;
                self.ensure_global(idx);
                self.stack.push(self.globals[idx].clone());
            }
            StoreGlobal(i) => {
                let idx = i as usize;
                self.ensure_global(idx);
                if let Some(v) = self.stack.last().cloned() {
                    self.globals[idx] = v;
                }
            }
            Add => add_any(self),
            Sub => bin(self, |a, b| a - b),
            Mul => bin(self, |a, b| a * b),
            Div => bin(self, |a, b| if b == 0.0 { 0.0 } else { a / b }),
            Eq | Ne | Lt | Le | Gt | Ge => cmp(self, op),
            Jump(offset) => {
                self.apply_jump(offset);
            }
            JumpIfFalse(offset) => {
                let should_jump = self.stack.last().map(|v| !is_truthy(v)).unwrap_or(true);
                if should_jump {
                    self.apply_jump(offset);
                }
            }
            JumpIfTrue(offset) => {
                let should_jump = self.stack.last().map(|v| is_truthy(v)).unwrap_or(false);
                if should_jump {
                    self.apply_jump(offset);
                }
            }
            Pop => {
                self.stack.pop();
            }
            Print(argc) => {
                self.do_print(argc);
            }
            Len => {
                self.do_len();
            }
            TimeMs => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);
                self.stack.push(Value::Number(now));
            }
            Rand => {
                self.stack.push(Value::Number(random::<f64>()));
            }
            Superpose => {
                self.consume_call_args(1);
                self.stack.push(Value::Null);
            }
            Entangle => {
                self.consume_call_args(2);
                self.stack.push(Value::Null);
            }
            Measure => {
                self.consume_call_args(1);
                let outcome = random::<bool>();
                self.stack.push(Value::Bool(outcome));
            }
            Return => {
                // Pop current frame; if no previous frame, halt.
                if let Some(frame) = self.frames.pop() {
                    if frame.return_ip == usize::MAX {
                        return false;
                    } // root frame returned => halt
                    self.ip = frame.return_ip;
                } else {
                    return false;
                }
            }
            Call(func_index, arity) => {
                if let Some(info) = self.chunk.functions.get(func_index as usize) {
                    if self.frames.len() >= self.max_frames {
                        self.stack_overflow = true;
                        return false;
                    }
                    // Pop args into temp (reverse to locals order)
                    let mut args: Vec<Value> = Vec::new();
                    for _ in 0..arity {
                        if let Some(v) = self.stack.pop() {
                            args.push(v);
                        }
                    }
                    args.reverse();
                    // Allocate new frame sized to function max locals (at least 1)
                    let mut locals = vec![Value::Null; (info.locals as usize).max(1)];
                    for (i, arg) in args.into_iter().enumerate() {
                        if i < locals.len() {
                            locals[i] = arg;
                        }
                    }
                    let ret_ip = self.ip;
                    self.frames.push(Frame {
                        return_ip: ret_ip,
                        locals,
                    });
                    self.ip = info.start;
                }
            }
        }
        true
    }
}

impl<'a> VM<'a> {
    fn ensure_global(&mut self, idx: usize) {
        if idx >= self.globals.len() {
            self.globals.resize(idx + 1, Value::Null);
        }
    }

    fn apply_jump(&mut self, offset: i16) {
        let next = (self.ip as isize) + (offset as isize);
        let max_ip = self.chunk.code.len() as isize;
        let clamped = next.clamp(0, max_ip);
        self.ip = clamped as usize;
    }

    fn do_print(&mut self, argc: u8) {
        let mut values = Vec::with_capacity(argc as usize);
        for _ in 0..argc {
            if let Some(v) = self.stack.pop() {
                values.push(v);
            }
        }
        values.reverse();
        if !values.is_empty() {
            let rendered: Vec<String> = values.iter().map(value_to_string).collect();
            println!("{}", rendered.join(" "));
        }
        self.stack.push(Value::Null);
    }

    fn do_len(&mut self) {
        let len = match self.stack.pop() {
            Some(Value::String(s)) => s.chars().count() as f64,
            Some(Value::Number(_)) => 0.0,
            Some(Value::Bool(_)) => 0.0,
            Some(Value::Null) | None => 0.0,
        };
        self.stack.push(Value::Number(len));
    }

    fn consume_call_args(&mut self, expected: usize) {
        for _ in 0..expected {
            self.stack.pop();
        }
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.fract() == 0.0 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
    }
}

fn bin(vm: &mut VM, f: impl Fn(f64, f64) -> f64) {
    if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) {
        if let (Value::Number(rb), Value::Number(lb)) = (r, l) {
            vm.stack.push(Value::Number(f(lb, rb)));
        } else {
            vm.stack.push(Value::Null);
        }
    }
}
fn cmp(vm: &mut VM, op: OpCode) {
    use OpCode::*;
    if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) {
        if let (Value::Number(rb), Value::Number(lb)) = (r, l) {
            let res = match op {
                Eq => lb == rb,
                Ne => lb != rb,
                Lt => lb < rb,
                Le => lb <= rb,
                Gt => lb > rb,
                Ge => lb >= rb,
                _ => false,
            };
            vm.stack.push(Value::Bool(res));
        } else {
            vm.stack.push(Value::Bool(false));
        }
    }
}
fn add_any(vm: &mut VM) {
    if let (Some(r), Some(l)) = (vm.stack.pop(), vm.stack.pop()) {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => vm.stack.push(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => {
                vm.stack.push(Value::String(format!("{}{}", a, b)))
            }
            (Value::String(a), Value::Number(b)) => {
                vm.stack.push(Value::String(format!("{}{}", a, b)))
            }
            (Value::Number(a), Value::String(b)) => {
                vm.stack.push(Value::String(format!("{}{}", a, b)))
            }
            (Value::Bool(a), Value::String(b)) => {
                vm.stack.push(Value::String(format!("{}{}", a, b)))
            }
            (Value::String(a), Value::Bool(b)) => {
                vm.stack.push(Value::String(format!("{}{}", a, b)))
            }
            (Value::Bool(a), Value::Bool(b)) => vm.stack.push(Value::String(format!("{}{}", a, b))),
            (_l, _r) => vm.stack.push(Value::Null),
        }
    }
}
