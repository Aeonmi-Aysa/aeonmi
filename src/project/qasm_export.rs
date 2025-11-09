use std::collections::HashMap;
use anyhow::Result;
use super::parser::{Program, Statement, StatementKind};

/// Export a simple parser Program to OpenQASM 2.0 format
/// Maps quantum operations to QASM gates:
/// - superpose → h (Hadamard)
/// - entangle → h + cx (Hadamard + CNOT)
/// - dod → x (Pauli-X / NOT)
/// - measure → measure
///
/// Non-representable control flow (if, while, for, function calls) is commented out
/// with a note explaining they cannot be directly represented in QASM.
pub fn export_to_qasm(program: &Program) -> Result<String> {
    let mut exporter = QasmExporter::new();
    exporter.export_program(program)?;
    Ok(exporter.output)
}

struct QasmExporter {
    output: String,
    qubit_vars: Vec<String>,   // Ordered list of qubit variable names
    qubit_indices: HashMap<String, usize>, // Map variable name to qubit index
}

impl QasmExporter {
    fn new() -> Self {
        Self {
            output: String::new(),
            qubit_vars: Vec::new(),
            qubit_indices: HashMap::new(),
        }
    }

    fn export_program(&mut self, program: &Program) -> Result<()> {
        // First pass: collect all qubit variables in deterministic order
        self.collect_qubits(program);

        // Write QASM header
        self.output.push_str("OPENQASM 2.0;\n");
        self.output.push_str("include \"qelib1.inc\";\n");
        
        if self.qubit_vars.is_empty() {
            self.output.push_str("// No quantum operations found\n");
            return Ok(());
        }

        let num_qubits = self.qubit_vars.len();
        self.output.push_str(&format!("qreg q[{}];\n", num_qubits));
        self.output.push_str(&format!("creg c[{}];\n", num_qubits));
        self.output.push_str("\n");

        // Add comment mapping variable names to indices
        self.output.push_str("// Qubit variable mapping:\n");
        for (idx, name) in self.qubit_vars.iter().enumerate() {
            self.output.push_str(&format!("// {} -> q[{}]\n", name, idx));
        }
        self.output.push_str("\n");

        // Second pass: emit QASM operations from all functions
        for function in program.functions() {
            self.output.push_str(&format!("// Function: {}\n", function.name));
            for stmt in &function.statements {
                self.emit_statement(stmt)?;
            }
            self.output.push_str("\n");
        }

        Ok(())
    }

    fn collect_qubits(&mut self, program: &Program) {
        // Collect all qubit variables in deterministic order (by first appearance)
        for function in program.functions() {
            for stmt in &function.statements {
                self.collect_qubits_from_statement(stmt);
            }
        }
    }

    fn collect_qubits_from_statement(&mut self, stmt: &Statement) {
        match &stmt.kind {
            StatementKind::Superpose(var) | StatementKind::Dod(var) | StatementKind::Measure(var) => {
                self.ensure_qubit_registered(var);
            }
            StatementKind::Entangle(control, target) => {
                self.ensure_qubit_registered(control);
                self.ensure_qubit_registered(target);
            }
            _ => {}
        }
    }

    fn ensure_qubit_registered(&mut self, var_name: &str) {
        if !self.qubit_indices.contains_key(var_name) {
            let idx = self.qubit_vars.len();
            self.qubit_vars.push(var_name.to_string());
            self.qubit_indices.insert(var_name.to_string(), idx);
        }
    }

    fn get_qubit_index(&self, var_name: &str) -> usize {
        *self.qubit_indices.get(var_name).expect("Qubit should be registered")
    }

    fn emit_statement(&mut self, stmt: &Statement) -> Result<()> {
        match &stmt.kind {
            StatementKind::Superpose(var) => {
                let idx = self.get_qubit_index(var);
                self.output.push_str(&format!("h q[{}];  // superpose {}\n", idx, var));
            }
            StatementKind::Entangle(control, target) => {
                let c_idx = self.get_qubit_index(control);
                let t_idx = self.get_qubit_index(target);
                self.output.push_str(&format!("h q[{}];  // entangle {} and {} (step 1: H on control)\n", c_idx, control, target));
                self.output.push_str(&format!("cx q[{}], q[{}];  // entangle {} and {} (step 2: CNOT)\n", c_idx, t_idx, control, target));
            }
            StatementKind::Dod(var) => {
                let idx = self.get_qubit_index(var);
                self.output.push_str(&format!("x q[{}];  // dod {}\n", idx, var));
            }
            StatementKind::Measure(var) => {
                let idx = self.get_qubit_index(var);
                self.output.push_str(&format!("measure q[{}] -> c[{}];  // measure {}\n", idx, idx, var));
            }
            StatementKind::Print(_) => {
                self.output.push_str(&format!(
                    "// Line {}: print statement (not representable in QASM)\n",
                    stmt.line
                ));
            }
            StatementKind::Let(name, _) | StatementKind::Set(name, _) => {
                self.output.push_str(&format!(
                    "// Line {}: variable '{}' (classical - not representable in QASM)\n",
                    stmt.line, name
                ));
            }
            StatementKind::AssertEq(_, _) => {
                self.output.push_str(&format!(
                    "// Line {}: assertion (not representable in QASM)\n",
                    stmt.line
                ));
            }
            StatementKind::Call(func) => {
                self.output.push_str(&format!(
                    "// Line {}: function call '{}' (not representable in QASM - would need inlining)\n",
                    stmt.line, func
                ));
            }
        }

        Ok(())
    }
}
