#![cfg(feature = "quantum")]

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::compiler::qasm_exporter::export_to_qasm;
use crate::core::{lexer::Lexer, parser::Parser};
use crate::runtime::hardware_manager::{dispatch_to_backend, QuantumBackend};

pub fn main(file: PathBuf, shots: Option<usize>, backend: &str) -> Result<()> {
    quantum_run(file, backend, shots)
}

pub fn quantum_run(file: PathBuf, backend: &str, shots: Option<usize>) -> Result<()> {
    let source = std::fs::read_to_string(&file)
        .map_err(|e| anyhow!("Failed to read file {}: {}", file.display(), e))?;

    let mut lexer = Lexer::from_str(&source);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow!("Lexing failed: {}", e))?;

    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|e| anyhow!("Parsing failed: {}", e))?;

    let qasm_code = export_to_qasm(&ast);
    if qasm_code.is_empty() {
        return Err(anyhow!("No quantum operations found to execute"));
    }

    let backend_enum = match backend.to_lowercase().as_str() {
        "titan" | "simulator" => QuantumBackend::Simulator,
        "aer" | "qiskit" => QuantumBackend::QiskitLocal,
        "ibmq" | "cloud" => QuantumBackend::QiskitCloud,
        other => return Err(anyhow!("Unknown backend '{}'", other)),
    };

    let previous_shots = std::env::var("AEONMI_SHOTS").ok();
    if let Some(n) = shots {
        std::env::set_var("AEONMI_SHOTS", n.to_string());
    }

    let execution = dispatch_to_backend(backend_enum, &qasm_code)
        .map_err(|e| anyhow!("Quantum execution error: {}", e));

    if shots.is_some() {
        if let Some(value) = previous_shots {
            std::env::set_var("AEONMI_SHOTS", value);
        } else {
            std::env::remove_var("AEONMI_SHOTS");
        }
    }

    let result = execution?;
    println!("{}", result);

    Ok(())
}
