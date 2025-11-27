//! Simple bytecode VM (feature: bytecode)
use crate::core::bytecode::{Chunk, Constant, OpCode};
use rand::random;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Quantum(Rc<RefCell<QuantumState>>),
}

#[derive(Debug)]
pub struct QuantumState {
    prob: f64,
    ent_group: Option<Rc<RefCell<EntanglementGroup>>>,
    collapsed: Option<bool>,
}

#[derive(Debug)]
struct EntanglementGroup {
    members: Vec<Rc<RefCell<QuantumState>>>,
    correlated: bool,
    collapsed: bool,
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
                if let Some(arg) = self.stack.pop() {
                    let state = self.into_quantum(arg);
                    Self::detach_from_group(&state);
                    {
                        let mut q = state.borrow_mut();
                        q.prob = 0.5;
                        q.collapsed = None;
                    }
                    self.stack.push(Value::Quantum(state));
                } else {
                    self.stack.push(Value::Null);
                }
            }
            Entangle => {
                let q2 = self.stack.pop().unwrap_or(Value::Null);
                let q1 = self.stack.pop().unwrap_or(Value::Null);

                let state1 = self.into_quantum(q1);
                let state2 = self.into_quantum(q2);

                Self::detach_from_group(&state1);
                Self::detach_from_group(&state2);

                let correlated = {
                    let a = state1.borrow().collapsed;
                    let b = state2.borrow().collapsed;
                    if let (Some(av), Some(bv)) = (a, b) {
                        av == bv
                    } else {
                        true
                    }
                };

                let group = Rc::new(RefCell::new(EntanglementGroup {
                    members: vec![Rc::clone(&state1), Rc::clone(&state2)],
                    correlated,
                    collapsed: false,
                }));

                {
                    let mut s1 = state1.borrow_mut();
                    s1.ent_group = Some(Rc::clone(&group));
                    s1.prob = 0.5;
                    s1.collapsed = None;
                }
                {
                    let mut s2 = state2.borrow_mut();
                    s2.ent_group = Some(Rc::clone(&group));
                    s2.prob = 0.5;
                    s2.collapsed = None;
                }

                self.stack.push(Value::Null);
            }
            Measure => {
                let value = self.stack.pop().unwrap_or(Value::Null);
                let outcome = match value {
                    Value::Quantum(state) => self.measure_quantum(state),
                    other => Self::coerce_bool(&other),
                };
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
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        self.stack.push(Value::Null);
    }

    fn do_len(&mut self) {
        let len = match self.stack.pop() {
            Some(Value::String(s)) => s.chars().count() as f64,
            Some(Value::Number(_)) => 0.0,
            Some(Value::Bool(_)) => 0.0,
            Some(Value::Quantum(_)) => 0.0,
            Some(Value::Null) | None => 0.0,
        };
        self.stack.push(Value::Number(len));
    }

    fn into_quantum(&self, value: Value) -> Rc<RefCell<QuantumState>> {
        match value {
            Value::Quantum(state) => state,
            Value::Bool(b) => Self::new_quantum_state(if b { 1.0 } else { 0.0 }, Some(b)),
            Value::Number(n) => {
                let collapsed = n != 0.0;
                Self::new_quantum_state(if collapsed { 1.0 } else { 0.0 }, Some(collapsed))
            }
            Value::String(s) => {
                let collapsed = !s.is_empty();
                Self::new_quantum_state(if collapsed { 1.0 } else { 0.0 }, Some(collapsed))
            }
            Value::Null => Self::new_quantum_state(0.5, None),
        }
    }

    fn new_quantum_state(prob: f64, collapsed: Option<bool>) -> Rc<RefCell<QuantumState>> {
        Rc::new(RefCell::new(QuantumState {
            prob: prob.clamp(0.0, 1.0),
            ent_group: None,
            collapsed,
        }))
    }

    fn detach_from_group(state: &Rc<RefCell<QuantumState>>) {
        let group_opt = { state.borrow().ent_group.clone() };
        if let Some(group_rc) = group_opt {
            {
                let mut group = group_rc.borrow_mut();
                group.members.retain(|member| !Rc::ptr_eq(member, state));
            }
            state.borrow_mut().ent_group = None;
        }
    }

    fn measure_quantum(&mut self, state: Rc<RefCell<QuantumState>>) -> bool {
        if let Some(val) = state.borrow().collapsed {
            return val;
        }

        let ent_group = { state.borrow().ent_group.clone() };
        if let Some(group_rc) = ent_group {
            let mut group = group_rc.borrow_mut();
            if group.collapsed {
                if let Some(val) = state.borrow().collapsed {
                    return val;
                }
            }

            let base_outcome = {
                let mut known = None;
                for member in &group.members {
                    if let Some(val) = member.borrow().collapsed {
                        known = Some(val);
                        break;
                    }
                }
                known.unwrap_or_else(|| {
                    let prob = state.borrow().prob.clamp(0.0, 1.0);
                    random::<f64>() < prob
                })
            };

            group.collapsed = true;

            for member in &group.members {
                let mut qubit = member.borrow_mut();
                let value = if group.correlated {
                    base_outcome
                } else if Rc::ptr_eq(member, &state) {
                    base_outcome
                } else {
                    !base_outcome
                };
                qubit.collapsed = Some(value);
                qubit.prob = if value { 1.0 } else { 0.0 };
            }

            return base_outcome;
        }

        let mut state_ref = state.borrow_mut();
        if let Some(val) = state_ref.collapsed {
            return val;
        }
        let prob = state_ref.prob.clamp(0.0, 1.0);
        let outcome = random::<f64>() < prob;
        state_ref.collapsed = Some(outcome);
        state_ref.prob = if outcome { 1.0 } else { 0.0 };
        outcome
    }

    fn coerce_bool(value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Quantum(state) => {
                let state = state.borrow();
                state
                    .collapsed
                    .unwrap_or_else(|| state.prob.clamp(0.0, 1.0) >= 0.5)
            }
        }
    }
}

fn is_truthy(value: &Value) -> bool {
    VM::coerce_bool(value)
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
        Value::Quantum(state) => {
            let state = state.borrow();
            let collapsed = match state.collapsed {
                Some(true) => "1",
                Some(false) => "0",
                None => "?",
            };
            format!("qubit[p={:.2}, collapsed={}]", state.prob, collapsed)
        }
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
        let result = match op {
            Eq | Ne => {
                let eq_result = match (&l, &r) {
                    (Value::Number(lb), Value::Number(rb)) => lb == rb,
                    (Value::Bool(lb), Value::Bool(rb)) => lb == rb,
                    (Value::String(ls), Value::String(rs)) => ls == rs,
                    (Value::Null, Value::Null) => true,
                    (Value::Quantum(a), Value::Quantum(b)) => {
                        if Rc::ptr_eq(a, b) {
                            true
                        } else {
                            let a_state = a.borrow();
                            let b_state = b.borrow();
                            a_state.collapsed == b_state.collapsed
                                && (a_state.prob - b_state.prob).abs() < f64::EPSILON
                        }
                    }
                    _ => false,
                };
                if matches!(op, Eq) {
                    eq_result
                } else {
                    !eq_result
                }
            }
            Lt | Le | Gt | Ge => {
                if let (Value::Number(lb), Value::Number(rb)) = (&l, &r) {
                    match op {
                        Lt => lb < rb,
                        Le => lb <= rb,
                        Gt => lb > rb,
                        Ge => lb >= rb,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        };
        vm.stack.push(Value::Bool(result));
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
