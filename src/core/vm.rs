//! Aeonmi VM: tree-walk interpreter over IR with quantum simulation support.
//! Supports: literals, quantum arrays/objects, let/assign, if/while/for, fn calls/returns,
//! binary/unary ops, quantum operations, and built-ins: print, log, time_ms, rand, len.

use crate::core::ir::*;
use crate::core::quantum_simulator::QuantumSimulator;
use crate::core::quantum_algorithms::{QuantumAlgorithms, DeutschJozsaOracle};
use crate::core::hardware_integration::{HardwareManager, QuantumCircuit};
use std::collections::HashMap;
use std::rc::Rc;
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
    Function(Rc<Function>), // user-defined — Rc makes clone O(1), prevents exponential env cascade
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
    /// Module-level functions registry — fallback when a closure-captured copy is Null
    /// (happens when alphabetical pass-2 processes a function before its sibling is defined).
    pub module_fns: std::collections::HashMap<String, Value>,
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
        
        // Gate builtins — callable as H(q), X(q), CNOT(q1,q2), etc.
        // AND usable as apply_gate(q, H) since apply_gate also accepts Builtin.
        env.define("H".into(), Value::Builtin(Builtin { name: "H", arity: 1, f: gate_h }));
        env.define("X".into(), Value::Builtin(Builtin { name: "X", arity: 1, f: gate_x }));
        env.define("Y".into(), Value::Builtin(Builtin { name: "Y", arity: 1, f: gate_y }));
        env.define("Z".into(), Value::Builtin(Builtin { name: "Z", arity: 1, f: gate_z }));
        env.define("S".into(), Value::Builtin(Builtin { name: "S", arity: 1, f: gate_s }));
        env.define("T".into(), Value::Builtin(Builtin { name: "T", arity: 1, f: gate_t }));
        env.define("CNOT".into(), Value::Builtin(Builtin { name: "CNOT", arity: 2, f: gate_cnot }));
        env.define("CX".into(), Value::Builtin(Builtin { name: "CNOT", arity: 2, f: gate_cnot }));
        env.define("HADAMARD".into(), Value::Builtin(Builtin { name: "H", arity: 1, f: gate_h }));
        env.define("NOT".into(), Value::Builtin(Builtin { name: "X", arity: 1, f: gate_x }));
        
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
        

        // ── Math builtins ──────────────────────────────────────────────────────
        env.define("sqrt".into(),     Value::Builtin(Builtin { name: "sqrt",     arity: 1, f: builtin_sqrt }));
        env.define("sin".into(),      Value::Builtin(Builtin { name: "sin",      arity: 1, f: builtin_sin }));
        env.define("cos".into(),      Value::Builtin(Builtin { name: "cos",      arity: 1, f: builtin_cos }));
        env.define("tan".into(),      Value::Builtin(Builtin { name: "tan",      arity: 1, f: builtin_tan }));
        env.define("atan2".into(),    Value::Builtin(Builtin { name: "atan2",    arity: 2, f: builtin_atan2 }));
        env.define("floor".into(),    Value::Builtin(Builtin { name: "floor",    arity: 1, f: builtin_floor }));
        env.define("ceil".into(),     Value::Builtin(Builtin { name: "ceil",     arity: 1, f: builtin_ceil }));
        env.define("round".into(),    Value::Builtin(Builtin { name: "round",    arity: 1, f: builtin_round }));
        env.define("abs".into(),      Value::Builtin(Builtin { name: "abs",      arity: 1, f: builtin_abs }));
        env.define("exp".into(),      Value::Builtin(Builtin { name: "exp",      arity: 1, f: builtin_exp }));
        env.define("ln".into(),       Value::Builtin(Builtin { name: "ln",       arity: 1, f: builtin_ln }));
        env.define("log10".into(),    Value::Builtin(Builtin { name: "log10",    arity: 1, f: builtin_log10 }));
        env.define("pow".into(),      Value::Builtin(Builtin { name: "pow",      arity: 2, f: builtin_pow }));
        env.define("min".into(),      Value::Builtin(Builtin { name: "min",      arity: 2, f: builtin_min }));
        env.define("max".into(),      Value::Builtin(Builtin { name: "max",      arity: 2, f: builtin_max }));
        env.define("PI".into(),       Value::Number(std::f64::consts::PI));
        env.define("E".into(),        Value::Number(std::f64::consts::E));
        env.define("TAU".into(),      Value::Number(std::f64::consts::TAU));
        env.define("INFINITY".into(), Value::Number(f64::INFINITY));
        env.define("NAN".into(),      Value::Number(f64::NAN));

        // ── String builtins ────────────────────────────────────────────────────
        env.define("upper".into(),       Value::Builtin(Builtin { name: "upper",       arity: 1, f: builtin_upper }));
        env.define("lower".into(),       Value::Builtin(Builtin { name: "lower",       arity: 1, f: builtin_lower }));
        env.define("trim".into(),        Value::Builtin(Builtin { name: "trim",        arity: 1, f: builtin_trim }));
        env.define("split".into(),       Value::Builtin(Builtin { name: "split",       arity: 2, f: builtin_split }));
        env.define("join".into(),        Value::Builtin(Builtin { name: "join",        arity: 2, f: builtin_join }));
        env.define("replace".into(),     Value::Builtin(Builtin { name: "replace",     arity: 3, f: builtin_replace }));
        env.define("contains".into(),    Value::Builtin(Builtin { name: "contains",    arity: 2, f: builtin_contains }));
        env.define("starts_with".into(), Value::Builtin(Builtin { name: "starts_with", arity: 2, f: builtin_starts_with }));
        env.define("ends_with".into(),   Value::Builtin(Builtin { name: "ends_with",   arity: 2, f: builtin_ends_with }));
        env.define("substr".into(),      Value::Builtin(Builtin { name: "substr",      arity: usize::MAX, f: builtin_substr }));
        env.define("char_at".into(),     Value::Builtin(Builtin { name: "char_at",     arity: 2, f: builtin_char_at }));
        env.define("find".into(),        Value::Builtin(Builtin { name: "find",        arity: 2, f: builtin_find }));
        env.define("repeat".into(),      Value::Builtin(Builtin { name: "repeat",      arity: 2, f: builtin_repeat }));
        env.define("lines".into(),       Value::Builtin(Builtin { name: "lines",       arity: 1, f: builtin_lines }));
        env.define("str_len".into(),     Value::Builtin(Builtin { name: "str_len",     arity: 1, f: builtin_str_len }));
        env.define("pad_left".into(),    Value::Builtin(Builtin { name: "pad_left",    arity: 3, f: builtin_pad_left }));
        env.define("pad_right".into(),   Value::Builtin(Builtin { name: "pad_right",   arity: 3, f: builtin_pad_right }));

        // ── File I/O builtins ──────────────────────────────────────────────────
        env.define("read_file".into(),   Value::Builtin(Builtin { name: "read_file",   arity: 1, f: builtin_read_file }));
        env.define("write_file".into(),  Value::Builtin(Builtin { name: "write_file",  arity: 2, f: builtin_write_file }));
        env.define("file_exists".into(), Value::Builtin(Builtin { name: "file_exists", arity: 1, f: builtin_file_exists }));
        env.define("append_file".into(), Value::Builtin(Builtin { name: "append_file", arity: 2, f: builtin_append_file }));
        env.define("list_dir".into(),    Value::Builtin(Builtin { name: "list_dir",    arity: 1, f: builtin_list_dir }));
        env.define("delete_file".into(), Value::Builtin(Builtin { name: "delete_file", arity: 1, f: builtin_delete_file }));
        env.define("make_dir".into(),    Value::Builtin(Builtin { name: "make_dir",    arity: 1, f: builtin_make_dir }));
        env.define("input".into(),       Value::Builtin(Builtin { name: "input",       arity: usize::MAX, f: builtin_input }));

        // ── HOST: self-hosting builtins ────────────────────────────────────────
        // shell_exec(cmd) → {stdout, stderr, exit_code}
        env.define("shell_exec".into(),  Value::Builtin(Builtin { name: "shell_exec",  arity: 1, f: builtin_shell_exec }));
        // run_ai(path, args_array) → string output of executing another .ai file
        env.define("run_ai".into(),      Value::Builtin(Builtin { name: "run_ai",      arity: usize::MAX, f: builtin_run_ai }));
        // get_env(name) → string value of environment variable
        env.define("get_env".into(),     Value::Builtin(Builtin { name: "get_env",     arity: 1, f: builtin_get_env }));
        // set_env(name, value) → sets environment variable for this process
        env.define("set_env".into(),     Value::Builtin(Builtin { name: "set_env",     arity: 2, f: builtin_set_env }));

        // ── Functional builtins ────────────────────────────────────────────────
        env.define("map".into(),      Value::Builtin(Builtin { name: "map",      arity: 2, f: builtin_map }));
        env.define("filter".into(),   Value::Builtin(Builtin { name: "filter",   arity: 2, f: builtin_filter }));
        env.define("reduce".into(),   Value::Builtin(Builtin { name: "reduce",   arity: usize::MAX, f: builtin_reduce }));
        env.define("range".into(),    Value::Builtin(Builtin { name: "range",    arity: usize::MAX, f: builtin_range }));
        env.define("enumerate".into(),Value::Builtin(Builtin { name: "enumerate",arity: 1, f: builtin_enumerate }));
        env.define("zip".into(),      Value::Builtin(Builtin { name: "zip",      arity: 2, f: builtin_zip }));
        env.define("any".into(),      Value::Builtin(Builtin { name: "any",      arity: 2, f: builtin_any }));
        env.define("all".into(),      Value::Builtin(Builtin { name: "all",      arity: 2, f: builtin_all }));
        env.define("sort".into(),     Value::Builtin(Builtin { name: "sort",     arity: usize::MAX, f: builtin_sort }));
        env.define("unique".into(),   Value::Builtin(Builtin { name: "unique",   arity: 1, f: builtin_unique }));
        env.define("flatten".into(),  Value::Builtin(Builtin { name: "flatten",  arity: 1, f: builtin_flatten }));
        env.define("keys".into(),     Value::Builtin(Builtin { name: "keys",     arity: 1, f: builtin_keys }));
        env.define("values".into(),   Value::Builtin(Builtin { name: "values",   arity: 1, f: builtin_values }));
        env.define("push".into(),     Value::Builtin(Builtin { name: "push",     arity: 2, f: builtin_push }));
        env.define("pop".into(),      Value::Builtin(Builtin { name: "pop",      arity: 1, f: builtin_pop }));
        env.define("slice".into(),    Value::Builtin(Builtin { name: "slice",    arity: usize::MAX, f: builtin_slice }));
        env.define("reverse".into(),  Value::Builtin(Builtin { name: "reverse",  arity: 1, f: builtin_reverse }));
        env.define("concat".into(),   Value::Builtin(Builtin { name: "concat",   arity: 2, f: builtin_concat }));
        env.define("sum".into(),      Value::Builtin(Builtin { name: "sum",      arity: 1, f: builtin_sum }));
        env.define("product".into(),  Value::Builtin(Builtin { name: "product",  arity: 1, f: builtin_product }));

        // ── Utility builtins ───────────────────────────────────────────────────
        env.define("assert".into(),    Value::Builtin(Builtin { name: "assert",    arity: usize::MAX, f: builtin_assert }));
        env.define("assert_eq".into(), Value::Builtin(Builtin { name: "assert_eq", arity: usize::MAX, f: builtin_assert_eq }));
        env.define("exit".into(),      Value::Builtin(Builtin { name: "exit",      arity: usize::MAX, f: builtin_exit }));
        env.define("sleep".into(),     Value::Builtin(Builtin { name: "sleep",     arity: 1, f: builtin_sleep }));
        env.define("hash".into(),      Value::Builtin(Builtin { name: "hash",      arity: 1, f: builtin_hash }));
        env.define("int".into(),       Value::Builtin(Builtin { name: "int",       arity: 1, f: builtin_int }));
        env.define("float".into(),     Value::Builtin(Builtin { name: "float",     arity: 1, f: builtin_float }));
        env.define("bool".into(),      Value::Builtin(Builtin { name: "bool",      arity: 1, f: builtin_bool }));
        env.define("is_null".into(),   Value::Builtin(Builtin { name: "is_null",   arity: 1, f: builtin_is_null }));
        env.define("is_nan".into(),    Value::Builtin(Builtin { name: "is_nan",    arity: 1, f: builtin_is_nan }));
        env.define("clamp".into(),     Value::Builtin(Builtin { name: "clamp",     arity: 3, f: builtin_clamp }));
        env.define("lerp".into(),      Value::Builtin(Builtin { name: "lerp",      arity: 3, f: builtin_lerp }));
        env.define("now".into(),       Value::Builtin(Builtin { name: "now",       arity: 0, f: builtin_now }));
        env.define("parse_json".into(),Value::Builtin(Builtin { name: "parse_json",arity: 1, f: builtin_parse_json }));
        env.define("to_json".into(),   Value::Builtin(Builtin { name: "to_json",   arity: 1, f: builtin_to_json }));
        env.define("object".into(),    Value::Builtin(Builtin { name: "object",    arity: 0, f: builtin_object }));
        env.define("set_key".into(),   Value::Builtin(Builtin { name: "set_key",   arity: 3, f: builtin_set_key }));
        env.define("get_key".into(),   Value::Builtin(Builtin { name: "get_key",   arity: 2, f: builtin_get_key }));
        env.define("has_key".into(),   Value::Builtin(Builtin { name: "has_key",   arity: 2, f: builtin_has_key }));
        env.define("delete_key".into(),Value::Builtin(Builtin { name: "delete_key",arity: 2, f: builtin_delete_key }));

        // ── Index access + fmod (unlocks arr[i] syntax and fmod() calls) ──────
        env.define("__index_access".into(), Value::Builtin(Builtin { name: "__index_access", arity: 2, f: builtin_index_access }));
        env.define("__quantum_index".into(), Value::Builtin(Builtin { name: "__quantum_index", arity: 2, f: builtin_index_access }));
        env.define("fmod".into(),           Value::Builtin(Builtin { name: "fmod",           arity: 2, f: builtin_fmod }));

        // ── Quantum bridge builtins ────────────────────────────────────────────
        // quantum_run(descriptor, shots) → JSON string with counts/most_likely
        // descriptor: space-separated string "n_q n_c shots op_count op_type tgt ctrl ..."
        // OR an Aeonmi array of numbers
        env.define("quantum_run".into(),   Value::Builtin(Builtin { name: "quantum_run",   arity: usize::MAX, f: builtin_quantum_run }));
        env.define("quantum_check".into(), Value::Builtin(Builtin { name: "quantum_check", arity: 0,          f: builtin_quantum_check }));

        Self {
            env,
            quantum_sim: QuantumSimulator::new(),
            quantum_alg: QuantumAlgorithms::new(),
            hardware_mgr: HardwareManager::new(),
            base_dir: None,
            imported: std::collections::HashSet::new(),
            module_fns: std::collections::HashMap::new(),
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

        // Load declarations: two-pass so sibling functions can call each other
        // Pass 1: pre-register all fn names as null so closures see full sibling set
        let fn_decls: Vec<&crate::core::ir::FnDecl> = module.decls.iter()
            .filter_map(|d| if let Decl::Fn(f) = d { if f.name != "main" { Some(f) } else { None } } else { None })
            .collect();
        for f in &fn_decls {
            self.env.define(f.name.clone(), Value::Null);
        }
        // Handle non-Fn decls
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
                Decl::Fn(_) => {} // handled in pass 2
            }
        }
        // Pass 2: all siblings in env — capture correct closures
        for f in &fn_decls {
            let func = Value::Function(Rc::new(Function {
                params: f.params.clone(),
                body: f.body.clone(),
                env: self.env.clone(),
            }));
            self.env.define(f.name.clone(), func);
        }
        // Register all module-level functions in the fallback registry so that
        // call_ident can find them even when a closure captured a Null placeholder.
        for f in &fn_decls {
            if let Some(v) = self.env.get(&f.name) {
                self.module_fns.insert(f.name.clone(), v);
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

        // Load top-level decls: two-pass so sibling functions can call each other
        // Pass 1: pre-register all fn names
        let fn_decls_mod: Vec<&crate::core::ir::FnDecl> = m.decls.iter()
            .filter_map(|d| if let Decl::Fn(f) = d { Some(f) } else { None })
            .collect();
        for f in &fn_decls_mod {
            self.env.define(f.name.clone(), Value::Null);
        }
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
                Decl::Fn(_) => {} // handled in pass 2
            }
        }
        // Pass 2: capture closures with all siblings in env
        for f in &fn_decls_mod {
            debug_log!("vm: load fn '{}'", f.name);
            let func = Value::Function(Rc::new(Function {
                params: f.params.clone(),
                body: f.body.clone(),
                env: self.env.clone(),
            }));
            self.env.define(f.name.clone(), func);
        }
        // Register module-level functions in fallback registry
        for f in &fn_decls_mod {
            if let Some(v) = self.env.get(&f.name) {
                self.module_fns.insert(f.name.clone(), v);
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
        // Primary lookup: current env chain
        let callee = self.env.get(name);
        // If not found, or found as Null (pass-1 placeholder not yet replaced), fall
        // back to the module-level function registry populated by resolve_import /
        // run_module — this handles sibling calls across alphabetical ordering gaps.
        let callee = match callee {
            Some(Value::Null) | None => self.module_fns.get(name).cloned(),
            other => other,
        };
        if callee.is_none() {
            return Err(err(format!("Undefined function `{}`", name)));
        }
        self.call_value(callee.unwrap(), args)
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
            Block(stmts) => {
                // Flat execution — no scope push/pop so Let bindings are visible
                // to subsequent stmts in the same enclosing scope.
                for s in stmts {
                    match self.exec_stmt(s) {
                        ControlFlow::Ok => {}
                        other => return other,
                    }
                }
                ControlFlow::Ok
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
                } else if let crate::core::ir::Expr::Member { object, .. } = target {
                    // this.field = value — evaluate value (for side effects) but
                    // discard the result; object identity tracking is Phase 5d.
                    if let crate::core::ir::Expr::Ident(obj_name) = object.as_ref() {
                        if obj_name == "this" || obj_name == "self" {
                            let _ = self.eval_expr(value); // eval for side effects
                            return ControlFlow::Ok;
                        }
                    }
                    ControlFlow::Err(err("Only simple identifier or this.field assignment supported".into()))
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
                // 'this' and 'self' resolve to an empty object when not explicitly set.
                // Constructor/method bodies use them for field access; the runtime
                // doesn't yet track object identity, so we return Null gracefully
                // instead of crashing. Phase 5d (genesis memory) will wire real 'this'.
                if (s == "this" || s == "self") && self.env.get(s).is_none() {
                    return Ok(Value::Null);
                }
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
                // Short-circuit evaluation for && and || — critical for safe range checks
                match op {
                    BinOp::And => {
                        let l = self.eval_expr(left)?;
                        if !self.truthy(&l) {
                            return Ok(Value::Bool(false));
                        }
                        let r = self.eval_expr(right)?;
                        return Ok(Value::Bool(self.truthy(&r)));
                    }
                    BinOp::Or => {
                        let l = self.eval_expr(left)?;
                        if self.truthy(&l) {
                            return Ok(Value::Bool(true));
                        }
                        let r = self.eval_expr(right)?;
                        return Ok(Value::Bool(self.truthy(&r)));
                    }
                    _ => {
                        let l = self.eval_expr(left)?;
                        let r = self.eval_expr(right)?;
                        self.eval_binop(op, l, r)?
                    }
                }
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
        // String lexicographic comparison — converts result to the numeric relation
        (Value::String(a), Value::String(b)) => {
            let ord = a.cmp(&b);
            // map ord to a numeric pair and apply f
            let (na, nb): (f64, f64) = match ord {
                std::cmp::Ordering::Less    => (-1.0, 0.0),
                std::cmp::Ordering::Equal   => (0.0,  0.0),
                std::cmp::Ordering::Greater => (1.0,  0.0),
            };
            Ok(Value::Bool(f(na, nb)))
        }
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

fn builtin_index_access(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(err(format!("__index_access: expected 2 args, got {}", args.len())));
    }
    match (&args[0], &args[1]) {
        (Value::Array(arr), Value::Number(idx)) => {
            let i = *idx as usize;
            arr.get(i).cloned().ok_or_else(|| err(format!("index {} out of bounds (array len {})", i, arr.len())))
        }
        (Value::String(s), Value::Number(idx)) => {
            let i = *idx as usize;
            let ch = s.chars().nth(i)
                .ok_or_else(|| err(format!("string index {} out of bounds (len {})", i, s.chars().count())))?;
            Ok(Value::String(ch.to_string()))
        }
        (Value::Object(map), Value::String(key)) => {
            Ok(map.get(key.as_str()).cloned().unwrap_or(Value::Null))
        }
        _ => Err(err(format!("cannot index value of this type with given key"))),
    }
}

fn builtin_fmod(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = to_f64(&args[0], "fmod")?;
    let y = to_f64(&args[1], "fmod")?;
    if y == 0.0 { return Err(err("fmod: division by zero".to_string())); }
    Ok(Value::Number(x % y))
}


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
        // Accept gate builtins: apply_gate(q, H) where H is now a Builtin
        Value::Builtin(b) => b.name.to_uppercase(),
        _ => return Err(err("apply_gate second argument must be a gate name or gate builtin (H, X, Y, Z, CNOT)".into())),
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
        "CNOT" => {
            return Err(err("CNOT requires 2 qubits: use entangle(q1, q2) instead".into()));
        }
        other => {
            return Err(err(format!("Unknown gate '{}'. Supported: H, X, Y, Z, CNOT", other)));
        }
    }
    
    Ok(Value::QubitReference(qubit_name))
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

// ─── Gate builtins — callable as H(q), X(q), CNOT(q1,q2) etc. ────────────────────────────

fn gate_qubit_name(args: &[Value], gate: &str) -> Result<String, RuntimeError> {
    match args.first() {
        Some(Value::QubitReference(n)) => Ok(n.clone()),
        Some(Value::String(n)) => Ok(n.clone()),
        _ => Err(err(format!("{} expects a qubit as first argument", gate))),
    }
}

fn gate_h(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = gate_qubit_name(&args, "H")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.superpose(&name)
        .map_err(|e| err(format!("H gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_x(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = gate_qubit_name(&args, "X")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.pauli_x(&name)
        .map_err(|e| err(format!("X gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_y(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = gate_qubit_name(&args, "Y")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.pauli_y(&name)
        .map_err(|e| err(format!("Y gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_z(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = gate_qubit_name(&args, "Z")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.pauli_z(&name)
        .map_err(|e| err(format!("Z gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_s(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    // S gate = Z^(1/2). Simulate as Z for now (approximation).
    let name = gate_qubit_name(&args, "S")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.pauli_z(&name)
        .map_err(|e| err(format!("S gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_t(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    // T gate = Z^(1/4). Simulate as Z for now (approximation).
    let name = gate_qubit_name(&args, "T")?;
    if !interp.quantum_sim.qubits.contains_key(&name) {
        interp.quantum_sim.create_qubit(name.clone());
    }
    interp.quantum_sim.pauli_z(&name)
        .map_err(|e| err(format!("T gate error: {}", e)))?;
    Ok(Value::QubitReference(name))
}

fn gate_cnot(interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() < 2 {
        return Err(err("CNOT expects 2 qubits: CNOT(control, target)".into()));
    }
    let ctrl = gate_qubit_name(&args[0..1], "CNOT")?;
    let tgt  = gate_qubit_name(&args[1..2], "CNOT")?;
    for name in [&ctrl, &tgt] {
        if !interp.quantum_sim.qubits.contains_key(name.as_str()) {
            interp.quantum_sim.create_qubit(name.clone());
        }
    }
    interp.quantum_sim.entangle(&ctrl, &tgt)
        .map_err(|e| err(format!("CNOT gate error: {}", e)))?;
    Ok(Value::Null)
}


// ═══════════════════════════════════════════════════════════════════════════════
// AEONMI STDLIB — Math, String, I/O, Functional, Utility builtins
// ═══════════════════════════════════════════════════════════════════════════════

// ── Helpers ───────────────────────────────────────────────────────────────────
fn to_f64(v: &Value, ctx: &str) -> Result<f64, RuntimeError> {
    match v {
        Value::Number(n) => Ok(*n),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::String(s) => s.parse::<f64>().map_err(|_| err(format!("{}: cannot convert {:?} to number", ctx, s))),
        _ => Err(err(format!("{}: expected number, got {:?}", ctx, v))),
    }
}
fn to_str(v: &Value, ctx: &str) -> Result<String, RuntimeError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(err(format!("{}: expected string", ctx))),
    }
}
fn to_arr(v: Value, ctx: &str) -> Result<Vec<Value>, RuntimeError> {
    match v {
        Value::Array(a) => Ok(a),
        _ => Err(err(format!("{}: expected array", ctx))),
    }
}
fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => *n != 0.0 && !n.is_nan(),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

// ── Math ──────────────────────────────────────────────────────────────────────
fn builtin_sqrt(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "sqrt")?.sqrt()))
}
fn builtin_sin(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "sin")?.sin()))
}
fn builtin_cos(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "cos")?.cos()))
}
fn builtin_tan(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "tan")?.tan()))
}
fn builtin_atan2(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "atan2")?.atan2(to_f64(&a[1], "atan2")?)))
}
fn builtin_floor(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "floor")?.floor()))
}
fn builtin_ceil(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "ceil")?.ceil()))
}
fn builtin_round(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "round")?.round()))
}
fn builtin_abs(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "abs")?.abs()))
}
fn builtin_exp(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "exp")?.exp()))
}
fn builtin_ln(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "ln")?.ln()))
}
fn builtin_log10(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "log10")?.log10()))
}
fn builtin_pow(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "pow")?.powf(to_f64(&a[1], "pow")?)))
}
fn builtin_min(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = to_f64(&a[0], "min")?; let y = to_f64(&a[1], "min")?;
    Ok(Value::Number(if x <= y { x } else { y }))
}
fn builtin_max(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = to_f64(&a[0], "max")?; let y = to_f64(&a[1], "max")?;
    Ok(Value::Number(if x >= y { x } else { y }))
}
fn builtin_clamp(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let v = to_f64(&a[0], "clamp")?; let lo = to_f64(&a[1], "clamp")?; let hi = to_f64(&a[2], "clamp")?;
    Ok(Value::Number(if v < lo { lo } else if v > hi { hi } else { v }))
}
fn builtin_lerp(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let x = to_f64(&a[0], "lerp")?; let y = to_f64(&a[1], "lerp")?; let t = to_f64(&a[2], "lerp")?;
    Ok(Value::Number(x + (y - x) * t))
}
fn builtin_is_nan(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(&a[0], Value::Number(n) if n.is_nan())))
}

// ── String ────────────────────────────────────────────────────────────────────
fn builtin_upper(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::String(to_str(&a[0], "upper")?.to_uppercase()))
}
fn builtin_lower(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::String(to_str(&a[0], "lower")?.to_lowercase()))
}
fn builtin_trim(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::String(to_str(&a[0], "trim")?.trim().to_string()))
}
fn builtin_split(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "split")?; let sep = to_str(&a[1], "split")?;
    Ok(Value::Array(s.split(sep.as_str()).map(|p| Value::String(p.to_string())).collect()))
}
fn builtin_join(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "join")?; let sep = to_str(&a[1], "join")?;
    let parts: Vec<String> = arr.iter().map(|v| display(v)).collect();
    Ok(Value::String(parts.join(&sep)))
}
fn builtin_replace(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "replace")?; let from = to_str(&a[1], "replace")?; let to = to_str(&a[2], "replace")?;
    Ok(Value::String(s.replace(from.as_str(), &to)))
}
fn builtin_contains(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "contains")?; let pat = to_str(&a[1], "contains")?;
    Ok(Value::Bool(s.contains(pat.as_str())))
}
fn builtin_starts_with(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "starts_with")?; let pat = to_str(&a[1], "starts_with")?;
    Ok(Value::Bool(s.starts_with(pat.as_str())))
}
fn builtin_ends_with(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "ends_with")?; let pat = to_str(&a[1], "ends_with")?;
    Ok(Value::Bool(s.ends_with(pat.as_str())))
}
fn builtin_substr(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if a.len() < 2 { return Err(err("substr(s, start[, len])".into())); }
    let s = to_str(&a[0], "substr")?;
    let chars: Vec<char> = s.chars().collect();
    let start = to_f64(&a[1], "substr")? as usize;
    let end = if a.len() >= 3 { (start + to_f64(&a[2], "substr")? as usize).min(chars.len()) } else { chars.len() };
    let start = start.min(chars.len());
    Ok(Value::String(chars[start..end].iter().collect()))
}
fn builtin_char_at(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "char_at")?; let i = to_f64(&a[1], "char_at")? as usize;
    let c = s.chars().nth(i).ok_or_else(|| err(format!("char_at: index {} out of bounds", i)))?;
    Ok(Value::String(c.to_string()))
}
fn builtin_find(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "find")?; let pat = to_str(&a[1], "find")?;
    match s.find(pat.as_str()) {
        Some(i) => Ok(Value::Number(s[..i].chars().count() as f64)),
        None => Ok(Value::Number(-1.0)),
    }
}
fn builtin_repeat(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "repeat")?; let n = to_f64(&a[1], "repeat")? as usize;
    Ok(Value::String(s.repeat(n)))
}
fn builtin_lines(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "lines")?;
    Ok(Value::Array(s.lines().map(|l| Value::String(l.to_string())).collect()))
}
fn builtin_str_len(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "str_len")?;
    Ok(Value::Number(s.chars().count() as f64))
}
fn builtin_pad_left(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "pad_left")?; let w = to_f64(&a[1], "pad_left")? as usize;
    let pad = to_str(&a[2], "pad_left")?;
    let ch = pad.chars().next().unwrap_or(' ');
    let len = s.chars().count();
    if len >= w { return Ok(Value::String(s)); }
    let result = std::iter::repeat(ch).take(w - len).chain(s.chars()).collect();
    Ok(Value::String(result))
}
fn builtin_pad_right(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "pad_right")?; let w = to_f64(&a[1], "pad_right")? as usize;
    let pad = to_str(&a[2], "pad_right")?;
    let ch = pad.chars().next().unwrap_or(' ');
    let len = s.chars().count();
    if len >= w { return Ok(Value::String(s)); }
    let result = s.chars().chain(std::iter::repeat(ch).take(w - len)).collect();
    Ok(Value::String(result))
}

// ── File I/O ──────────────────────────────────────────────────────────────────
fn builtin_read_file(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "read_file")?;
    match std::fs::read_to_string(&path) {
        Ok(s) => Ok(Value::String(s)),
        Err(e) => Err(err(format!("read_file: {}", e))),
    }
}
fn builtin_write_file(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "write_file")?; let content = to_str(&a[1], "write_file")?;
    std::fs::write(&path, &content).map_err(|e| err(format!("write_file: {}", e)))?;
    Ok(Value::Bool(true))
}
fn builtin_file_exists(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "file_exists")?;
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}
fn builtin_append_file(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    use std::io::Write;
    let path = to_str(&a[0], "append_file")?; let content = to_str(&a[1], "append_file")?;
    let mut f = std::fs::OpenOptions::new().append(true).create(true).open(&path)
        .map_err(|e| err(format!("append_file: {}", e)))?;
    f.write_all(content.as_bytes()).map_err(|e| err(format!("append_file write: {}", e)))?;
    Ok(Value::Bool(true))
}
fn builtin_list_dir(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "list_dir")?;
    let entries = std::fs::read_dir(&path)
        .map_err(|e| err(format!("list_dir: {}", e)))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| err(format!("list_dir entry: {}", e)))?;
        let name = entry.file_name().to_string_lossy().to_string();
        names.push(Value::String(name));
    }
    Ok(Value::Array(names))
}
fn builtin_delete_file(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "delete_file")?;
    let p = std::path::Path::new(&path);
    if p.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| err(format!("delete_file(dir): {}", e)))?;
    } else {
        std::fs::remove_file(&path).map_err(|e| err(format!("delete_file: {}", e)))?;
    }
    Ok(Value::Bool(true))
}
fn builtin_make_dir(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let path = to_str(&a[0], "make_dir")?;
    std::fs::create_dir_all(&path).map_err(|e| err(format!("make_dir: {}", e)))?;
    Ok(Value::Bool(true))
}

// ── HOST: Self-hosting builtins ───────────────────────────────────────────────
fn builtin_shell_exec(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let cmd = to_str(&a[0], "shell_exec")?;
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| err(format!("shell_exec: {}", e)))?
    };
    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .map_err(|e| err(format!("shell_exec: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1) as f64;
    let mut map = std::collections::HashMap::new();
    map.insert("stdout".to_string(),    Value::String(stdout));
    map.insert("stderr".to_string(),    Value::String(stderr));
    map.insert("exit_code".to_string(), Value::Number(exit_code));
    Ok(Value::Object(map))
}

fn builtin_run_ai(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    use crate::core::lexer::Lexer;
    use crate::core::parser::Parser as AeParser;
    use crate::core::lowering::lower_ast_to_ir;
    if a.is_empty() { return Err(err("run_ai: expected path argument".to_string())); }
    let path = to_str(&a[0], "run_ai")?;
    let source = std::fs::read_to_string(&path)
        .map_err(|e| err(format!("run_ai: cannot read '{}': {}", path, e)))?;
    let canonical = std::path::Path::new(&path)
        .canonicalize().map(|p| p.display().to_string()).unwrap_or_else(|_| path.clone());
    let mut lexer = Lexer::from_str(&source);
    let tokens = lexer.tokenize()
        .map_err(|e| err(format!("run_ai: lex error in '{}': {}", path, e)))?;
    let mut parser = AeParser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| err(format!("run_ai: parse error in '{}': {:?}", path, e)))?;
    let module = lower_ast_to_ir(&ast, &canonical)
        .map_err(|e| err(format!("run_ai: lowering error in '{}': {:?}", path, e)))?;
    // Run in a fresh child interpreter so it doesn't pollute caller's env
    let mut child = Interpreter::new();
    child.base_dir = std::path::Path::new(&path).parent().map(|p| p.to_path_buf());
    let args_val = Value::Array(a[1..].to_vec());
    child.env.define("args".into(), args_val);
    // Resolve imports relative to the child's base_dir
    for imp in &module.imports {
        child.resolve_import(&imp.path)
            .map_err(|e| err(format!("run_ai: import error in '{}': {}", path, e.message)))?;
    }
    child.run_module(&module)
        .map_err(|e| err(format!("run_ai: runtime error in '{}': {}", path, e.message)))?;
    Ok(Value::String(format!("run_ai: '{}' completed", path)))
}

fn builtin_get_env(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = to_str(&a[0], "get_env")?;
    Ok(Value::String(std::env::var(&name).unwrap_or_default()))
}

fn builtin_set_env(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let name = to_str(&a[0], "set_env")?;
    let val  = to_str(&a[1], "set_env")?;
    std::env::set_var(&name, &val);
    Ok(Value::Bool(true))
}

fn builtin_input(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if !a.is_empty() {
        if let Value::String(prompt) = &a[0] { print!("{}", prompt); }
        use std::io::Write; let _ = std::io::stdout().flush();
    }
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| err(format!("input: {}", e)))?;
    Ok(Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
}

// ── Functional ────────────────────────────────────────────────────────────────
fn builtin_map(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "map")?; let f = a[1].clone();
    let mut out = Vec::with_capacity(arr.len());
    for item in arr { out.push(interp.call_value(f.clone(), vec![item])?); }
    Ok(Value::Array(out))
}
fn builtin_filter(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "filter")?; let f = a[1].clone();
    let mut out = Vec::new();
    for item in arr {
        let result = interp.call_value(f.clone(), vec![item.clone()])?;
        if truthy(&result) { out.push(item); }
    }
    Ok(Value::Array(out))
}
fn builtin_reduce(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if a.len() < 2 { return Err(err("reduce(arr, fn[, init])".into())); }
    let arr = to_arr(a[0].clone(), "reduce")?; let f = a[1].clone();
    if arr.is_empty() {
        return if a.len() >= 3 { Ok(a[2].clone()) } else { Err(err("reduce: empty array with no init".into())) };
    }
    let (mut acc, start) = if a.len() >= 3 { (a[2].clone(), 0) } else { (arr[0].clone(), 1) };
    for item in arr.into_iter().skip(start) {
        acc = interp.call_value(f.clone(), vec![acc, item])?;
    }
    Ok(acc)
}
fn builtin_range(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let (start, end, step) = match a.len() {
        1 => (0.0, to_f64(&a[0], "range")?, 1.0),
        2 => (to_f64(&a[0], "range")?, to_f64(&a[1], "range")?, 1.0),
        _ => (to_f64(&a[0], "range")?, to_f64(&a[1], "range")?, to_f64(&a[2], "range")?),
    };
    if step == 0.0 { return Err(err("range: step cannot be 0".into())); }
    let mut out = Vec::new(); let mut cur = start;
    while (step > 0.0 && cur < end) || (step < 0.0 && cur > end) {
        out.push(Value::Number(cur)); cur += step;
        if out.len() > 1_000_000 { return Err(err("range: too many elements (>1M)".into())); }
    }
    Ok(Value::Array(out))
}
fn builtin_enumerate(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "enumerate")?;
    Ok(Value::Array(arr.into_iter().enumerate().map(|(i, v)| {
        Value::Array(vec![Value::Number(i as f64), v])
    }).collect()))
}
fn builtin_zip(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let a0 = to_arr(a[0].clone(), "zip")?; let a1 = to_arr(a[1].clone(), "zip")?;
    Ok(Value::Array(a0.into_iter().zip(a1.into_iter()).map(|(x, y)| Value::Array(vec![x, y])).collect()))
}
fn builtin_any(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "any")?; let f = a[1].clone();
    for item in arr { if truthy(&interp.call_value(f.clone(), vec![item])?) { return Ok(Value::Bool(true)); } }
    Ok(Value::Bool(false))
}
fn builtin_all(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "all")?; let f = a[1].clone();
    for item in arr { if !truthy(&interp.call_value(f.clone(), vec![item])?) { return Ok(Value::Bool(false)); } }
    Ok(Value::Bool(true))
}
fn builtin_sort(interp: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut arr = to_arr(a[0].clone(), "sort")?;
    if a.len() >= 2 {
        // sort with comparator
        let f = a[1].clone();
        let mut err_acc: Option<RuntimeError> = None;
        arr.sort_by(|x, y| {
            if err_acc.is_some() { return std::cmp::Ordering::Equal; }
            // We need interior mutability here; use a workaround with unsafe ptr
            let interp_ptr = interp as *mut Interpreter;
            match unsafe { (*interp_ptr).call_value(f.clone(), vec![x.clone(), y.clone()]) } {
                Ok(Value::Number(n)) => {
                    if n < 0.0 { std::cmp::Ordering::Less }
                    else if n > 0.0 { std::cmp::Ordering::Greater }
                    else { std::cmp::Ordering::Equal }
                },
                Ok(_) => std::cmp::Ordering::Equal,
                Err(e) => { err_acc = Some(e); std::cmp::Ordering::Equal },
            }
        });
        if let Some(e) = err_acc { return Err(e); }
    } else {
        // default sort: numbers numerically, strings lexicographically
        arr.sort_by(|x, y| match (x, y) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });
    }
    Ok(Value::Array(arr))
}
fn builtin_unique(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "unique")?;
    let mut seen = Vec::new(); let mut out = Vec::new();
    for item in arr {
        let key = display(&item);
        if !seen.contains(&key) { seen.push(key); out.push(item); }
    }
    Ok(Value::Array(out))
}
fn builtin_flatten(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "flatten")?;
    let mut out = Vec::new();
    for item in arr { match item { Value::Array(inner) => out.extend(inner), other => out.push(other) } }
    Ok(Value::Array(out))
}
fn builtin_keys(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match &a[0] {
        Value::Object(m) => { let mut ks: Vec<Value> = m.keys().map(|k| Value::String(k.clone())).collect(); ks.sort_by(|a,b| display(a).cmp(&display(b))); Ok(Value::Array(ks)) },
        _ => Err(err("keys: expected object".into())),
    }
}
fn builtin_values(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match &a[0] {
        Value::Object(m) => { let mut pairs: Vec<_> = m.iter().collect(); pairs.sort_by_key(|(k,_)| k.clone()); Ok(Value::Array(pairs.into_iter().map(|(_,v)| v.clone()).collect())) },
        _ => Err(err("values: expected object".into())),
    }
}
fn builtin_push(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut arr = to_arr(a[0].clone(), "push")?; arr.push(a[1].clone()); Ok(Value::Array(arr))
}
fn builtin_pop(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut arr = to_arr(a[0].clone(), "pop")?;
    let last = arr.pop().unwrap_or(Value::Null);
    Ok(Value::Array(vec![last, Value::Array(arr)]))
}
fn builtin_slice(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if a.len() < 2 { return Err(err("slice(arr, start[, end])".into())); }
    let arr = to_arr(a[0].clone(), "slice")?;
    let start = (to_f64(&a[1], "slice")? as isize).max(0) as usize;
    let end = if a.len() >= 3 { (to_f64(&a[2], "slice")? as isize).max(0) as usize } else { arr.len() };
    let start = start.min(arr.len()); let end = end.min(arr.len());
    Ok(Value::Array(arr[start..end].to_vec()))
}
fn builtin_reverse(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match &a[0] {
        Value::Array(arr) => { let mut v = arr.clone(); v.reverse(); Ok(Value::Array(v)) },
        Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
        _ => Err(err("reverse: expected array or string".into())),
    }
}
fn builtin_concat(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut out = to_arr(a[0].clone(), "concat")?;
    out.extend(to_arr(a[1].clone(), "concat")?);
    Ok(Value::Array(out))
}
fn builtin_sum(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "sum")?;
    let total: f64 = arr.iter().map(|v| to_f64(v, "sum").unwrap_or(0.0)).sum();
    Ok(Value::Number(total))
}
fn builtin_product(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let arr = to_arr(a[0].clone(), "product")?;
    let total: f64 = arr.iter().map(|v| to_f64(v, "product").unwrap_or(1.0)).product();
    Ok(Value::Number(total))
}

// ── Utility ───────────────────────────────────────────────────────────────────
fn builtin_assert(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if !truthy(&a[0]) {
        let msg = if a.len() >= 2 { display(&a[1]) } else { "assertion failed".to_string() };
        return Err(err(format!("AssertionError: {}", msg)));
    }
    Ok(Value::Null)
}
fn builtin_assert_eq(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    if a.len() < 2 { return Err(err("assert_eq(a, b[, msg])".into())); }
    if display(&a[0]) != display(&a[1]) {
        let msg = if a.len() >= 3 { display(&a[2]) } else { format!("expected {:?} == {:?}", display(&a[0]), display(&a[1])) };
        return Err(err(format!("AssertionError: {}", msg)));
    }
    Ok(Value::Null)
}
fn builtin_exit(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let code = if !a.is_empty() { to_f64(&a[0], "exit").unwrap_or(0.0) as i32 } else { 0 };
    std::process::exit(code);
}
fn builtin_sleep(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let ms = to_f64(&a[0], "sleep")? as u64;
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(Value::Null)
}
fn builtin_hash(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new(); display(&a[0]).hash(&mut h);
    Ok(Value::Number(h.finish() as f64))
}
fn builtin_int(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "int")?.trunc()))
}
fn builtin_float(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(to_f64(&a[0], "float")?))
}
fn builtin_bool(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(truthy(&a[0])))
}
fn builtin_is_null(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Bool(matches!(&a[0], Value::Null)))
}
fn builtin_now(_i: &mut Interpreter, _a: Vec<Value>) -> Result<Value, RuntimeError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    Ok(Value::Number(ms as f64))
}
fn builtin_parse_json(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    let s = to_str(&a[0], "parse_json")?;
    parse_json_str(&s)
}
fn parse_json_str(s: &str) -> Result<Value, RuntimeError> {
    let s = s.trim();
    if s == "null" { return Ok(Value::Null); }
    if s == "true" { return Ok(Value::Bool(true)); }
    if s == "false" { return Ok(Value::Bool(false)); }
    if let Ok(n) = s.parse::<f64>() { return Ok(Value::Number(n)); }
    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len()-1];
        return Ok(Value::String(inner.replace("\\n","\n").replace("\\t","\t").replace("\\\"","\"").replace("\\\\","\\")));
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len()-1].trim();
        if inner.is_empty() { return Ok(Value::Array(vec![])); }
        let parts = split_json_top(inner);
        let vals: Result<Vec<Value>, _> = parts.iter().map(|p| parse_json_str(p.trim())).collect();
        return Ok(Value::Array(vals?));
    }
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len()-1].trim();
        let mut map = std::collections::HashMap::new();
        if !inner.is_empty() {
            for pair in split_json_top(inner) {
                let pair = pair.trim();
                if let Some(colon) = find_json_colon(pair) {
                    let key = pair[..colon].trim().trim_matches('"').to_string();
                    let val = parse_json_str(pair[colon+1..].trim())?;
                    map.insert(key, val);
                }
            }
        }
        return Ok(Value::Object(map));
    }
    Err(err(format!("parse_json: cannot parse: {}", &s[..s.len().min(40)])))
}
fn split_json_top(s: &str) -> Vec<String> {
    let mut parts = Vec::new(); let mut depth = 0i32; let mut start = 0; let mut in_str = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' && (i == 0 || chars[i-1] != '\\') { in_str = !in_str; }
        else if !in_str {
            if c == '[' || c == '{' { depth += 1; }
            else if c == ']' || c == '}' { depth -= 1; }
            else if c == ',' && depth == 0 { parts.push(s[start..].chars().take(i-start).collect()); start = i+1; }
        }
        i += 1;
    }
    parts.push(s[start..].to_string());
    parts
}
fn find_json_colon(s: &str) -> Option<usize> {
    let mut depth = 0i32; let mut in_str = false;
    for (i, c) in s.chars().enumerate() {
        if c == '"' { in_str = !in_str; }
        else if !in_str {
            if c == '[' || c == '{' { depth += 1; }
            else if c == ']' || c == '}' { depth -= 1; }
            else if c == ':' && depth == 0 { return Some(i); }
        }
    }
    None
}
fn builtin_to_json(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::String(value_to_json(&a[0])))
}
fn value_to_json(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => { if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", *n as i64) } else { format!("{}", n) } },
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t")),
        Value::Array(a) => format!("[{}]", a.iter().map(value_to_json).collect::<Vec<_>>().join(",")),
        Value::Object(m) => { let mut pairs: Vec<_> = m.iter().collect(); pairs.sort_by_key(|(k,_)| k.clone()); format!("{{{}}}", pairs.iter().map(|(k,v)| format!("\"{}\":{}", k, value_to_json(v))).collect::<Vec<_>>().join(",")) },
        _ => format!("\"{}\"", display(v)),
    }
}
fn builtin_object(_i: &mut Interpreter, _a: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::Object(std::collections::HashMap::new()))
}
fn builtin_set_key(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match a[0].clone() {
        Value::Object(mut m) => { m.insert(to_str(&a[1], "set_key")?, a[2].clone()); Ok(Value::Object(m)) },
        _ => Err(err("set_key: expected object".into())),
    }
}
fn builtin_get_key(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match &a[0] {
        Value::Object(m) => Ok(m.get(&to_str(&a[1], "get_key")?).cloned().unwrap_or(Value::Null)),
        _ => Err(err("get_key: expected object".into())),
    }
}
fn builtin_has_key(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match &a[0] {
        Value::Object(m) => Ok(Value::Bool(m.contains_key(&to_str(&a[1], "has_key")?))),
        _ => Err(err("has_key: expected object".into())),
    }
}
fn builtin_delete_key(_i: &mut Interpreter, a: Vec<Value>) -> Result<Value, RuntimeError> {
    match a[0].clone() {
        Value::Object(mut m) => { m.remove(&to_str(&a[1], "delete_key")?); Ok(Value::Object(m)) },
        _ => Err(err("delete_key: expected object".into())),
    }
}

// ── Quantum bridge ────────────────────────────────────────────────────────────

/// quantum_run(descriptor, shots?) → JSON string
/// descriptor: space-separated string OR Aeonmi array of numbers
/// Format: "n_q n_c shots op_count [op_type tgt ctrl] ..."
/// Calls qiskit_runner.py via subprocess; falls back to dry-run if Python unavailable.
fn builtin_quantum_run(_i: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Err(err("quantum_run: expected descriptor argument".into()));
    }

    // Build the space-separated descriptor string
    let descriptor_str = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            arr.iter().map(|v| match v {
                Value::Number(n) => {
                    if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) }
                }
                other => display(other),
            }).collect::<Vec<_>>().join(" ")
        }
        other => display(other).to_string(),
    };

    // If a second arg is provided as shots, append/override
    let descriptor_with_shots = if args.len() >= 2 {
        // Shots is args[1]; the descriptor already encodes shots at position 2.
        // Patch position 2 of the descriptor with the new shots value.
        let mut parts: Vec<String> = descriptor_str.split_whitespace().map(str::to_string).collect();
        if parts.len() >= 3 {
            if let Ok(n) = to_f64(&args[1], "quantum_run") {
                parts[2] = format!("{}", n as u64);
            }
        }
        parts.join(" ")
    } else {
        descriptor_str
    };

    // Locate qiskit_runner.py — try several common locations
    let runner_path = {
        let candidates = [
            // 1. Relative to current working directory (most reliable when running via aeonmi native)
            std::env::current_dir().unwrap_or_default().join("Aeonmi_Master").join("qiskit_runner.py"),
            // 2. Relative to exe: up 3 levels (project root when built in target/debug or target/release)
            {
                let exe = std::env::current_exe().unwrap_or_default();
                let root = exe.ancestors().nth(3).unwrap_or(std::path::Path::new(".")).to_path_buf();
                root.join("Aeonmi_Master").join("qiskit_runner.py")
            },
            // 3. Sibling of current dir
            std::path::PathBuf::from("Aeonmi_Master/qiskit_runner.py"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_default().join("Aeonmi_Master").join("qiskit_runner.py")
        })
    };

    // Try python3 then python
    let python_cmds = ["python3", "python"];
    let mut last_err = String::from("Python not found");

    for python in &python_cmds {
        let result = std::process::Command::new(python)
            .arg(&runner_path)
            .args(descriptor_with_shots.split_whitespace())
            .output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() && output.status.success() {
                    // Qiskit may print warnings to stderr — that's ok
                }
                if output.status.success() && !stdout.trim().is_empty() {
                    return Ok(Value::String(stdout.trim().to_string()));
                }
                if !stdout.trim().is_empty() {
                    return Ok(Value::String(stdout.trim().to_string()));
                }
                last_err = format!("exit {}: {}", output.status.code().unwrap_or(-1), stderr.trim());
            }
            Err(e) => {
                last_err = e.to_string();
                continue;
            }
        }
    }

    // Dry-run fallback — parse descriptor manually and describe the circuit
    let parts: Vec<&str> = descriptor_with_shots.split_whitespace().collect();
    if parts.len() >= 4 {
        let n_q = parts[0];
        let n_c = parts[1];
        let shots = parts[2];
        let op_count: usize = parts[3].parse().unwrap_or(0);
        let op_names = ["H", "X", "Y", "Z", "CX", "S", "T", "MEASURE"];
        let mut ops = Vec::new();
        for i in 0..op_count {
            let base = 4 + i * 3;
            if base + 2 < parts.len() {
                let op_t: usize = parts[base].parse().unwrap_or(99);
                let tgt = parts[base + 1];
                let ctrl = parts[base + 2];
                let name = op_names.get(op_t).copied().unwrap_or("?");
                if op_t == 4 { ops.push(format!("CX(ctrl={},tgt={})", ctrl, tgt)); }
                else if op_t == 7 { ops.push(format!("MEASURE(q{}->c{})", tgt, tgt)); }
                else { ops.push(format!("{}(q{})", name, tgt)); }
            }
        }
        let dry = format!(
            r#"{{"dry_run":true,"n_qubits":{},"n_cbits":{},"shots":{},"ops":"{}" ,"note":"{}"}}"#,
            n_q, n_c, shots,
            ops.join(" -> "),
            last_err.replace('"', "'"),
        );
        return Ok(Value::String(dry));
    }

    Ok(Value::String(format!(r#"{{"error":"quantum_run failed: {}"}}"#, last_err.replace('"', "'"))))
}

/// quantum_check() → 1 if Python + Qiskit available, 0 otherwise
fn builtin_quantum_check(_i: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    let check_script = "try:\n    import qiskit; import qiskit_aer; print('1')\nexcept ImportError:\n    print('0')\n";
    for python in &["python3", "python"] {
        if let Ok(out) = std::process::Command::new(python)
            .arg("-c")
            .arg(check_script)
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.trim() == "1" { return Ok(Value::Number(1.0)); }
            if s.trim() == "0" { return Ok(Value::Number(0.0)); }
        }
    }
    Ok(Value::Number(0.0))
}
