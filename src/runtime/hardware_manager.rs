use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum QuantumBackend {
    Simulator,
    QiskitLocal,
    QiskitCloud,
}

/// Dispatch QASM code to the specified quantum backend and return the execution result.
pub fn dispatch_to_backend(backend: QuantumBackend, qasm_code: &str) -> Result<String, String> {
    match backend {
        QuantumBackend::Simulator => Err("Simulator backend not yet implemented".into()),
        QuantumBackend::QiskitLocal => run_qiskit_local(qasm_code),
        QuantumBackend::QiskitCloud => Err("Qiskit cloud backend not supported yet".into()),
    }
}

fn run_qiskit_local(qasm_code: &str) -> Result<String, String> {
    let tmp_path = std::env::temp_dir().join("aeonmi_circuit.qasm");
    fs::write(&tmp_path, qasm_code)
        .map_err(|e| format!("Failed to write QASM temp file: {}", e))?;

    let runner_path = locate_qiskit_runner()?;
    let python = python_command();

    let mut cmd = Command::new(&python);
    cmd.arg(&runner_path);
    cmd.arg(&tmp_path);

    if let Ok(shots) = std::env::var("AEONMI_SHOTS") {
        if !shots.trim().is_empty() {
            cmd.arg(shots);
        }
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(err) => {
            let _ = fs::remove_file(&tmp_path);
            return Err(format!(
                "Failed to invoke Qiskit runner ({}): {}",
                python, err
            ));
        }
    };

    let _ = fs::remove_file(&tmp_path);

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Qiskit execution failed: {}", err_msg.trim()));
    }

    let result_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if result_str.is_empty() {
        Err("Qiskit runner returned no data".into())
    } else {
        Ok(result_str)
    }
}

fn locate_qiskit_runner() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("AEONMI_QISKIT_RUNNER") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        } else {
            return Err(format!(
                "Qiskit runner script not found at {}",
                path.display()
            ));
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(dir) = std::env::current_dir() {
        candidates.push(dir.join("qiskit_runner.py"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("qiskit_runner.py"));
        }
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qiskit_runner.py"));

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("qiskit_runner.py not found. Set AEONMI_QISKIT_RUNNER to its location.".into())
}

fn python_command() -> String {
    std::env::var("AEONMI_PYTHON")
        .or_else(|_| std::env::var("PYTHON"))
        .unwrap_or_else(|_| "python".to_string())
}
