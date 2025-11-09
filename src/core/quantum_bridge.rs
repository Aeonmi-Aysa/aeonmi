//! Quantum Integration Bridge for Aeonmi Runtime
//!
//! This module provides seamless integration between Aeonmi's quantum operations
//! and various quantum backends including simulators and real hardware.

use crate::core::bytecode_ir::{QuantumCircuit, QuantumGate};
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

/// Quantum execution bridge that manages multiple backends
#[derive(Debug)]
pub struct QuantumBridge {
    /// Available backends
    backends: HashMap<String, Box<dyn QuantumBackend>>,

    /// Default backend name
    default_backend: String,

    /// Configuration settings
    config: QuantumBridgeConfig,
}

/// Configuration for quantum bridge
#[derive(Debug, Clone)]
pub struct QuantumBridgeConfig {
    /// Enable quantum optimizations
    pub optimize_circuits: bool,

    /// Maximum qubits for local simulation
    pub max_local_qubits: u32,

    /// Measurement shot count
    pub measurement_shots: u32,

    /// Timeout for quantum operations (seconds)
    pub operation_timeout: u32,
}

/// Trait for quantum backend implementations
pub trait QuantumBackend: std::fmt::Debug + Send + Sync {
    /// Execute a quantum circuit and return measurement results
    fn execute_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumResult, QuantumError>;

    /// Get backend capabilities and information
    fn get_info(&self) -> BackendInfo;

    /// Check if backend is currently available
    fn is_available(&self) -> bool;

    /// Get current queue status (for hardware backends)
    fn get_queue_status(&self) -> Option<QueueStatus>;
}

/// Result of quantum circuit execution
#[derive(Debug, Clone)]
pub struct QuantumResult {
    /// Measurement outcomes for each measured qubit
    pub measurements: Vec<f64>,

    /// Probability distribution of outcomes
    pub probabilities: HashMap<String, f64>,

    /// Final quantum state (if available)
    pub final_state: Option<Vec<(f64, f64)>>, // Complex amplitudes

    /// Execution metadata
    pub metadata: QuantumExecutionMetadata,
}

/// Metadata about quantum execution
#[derive(Debug, Clone)]
pub struct QuantumExecutionMetadata {
    /// Backend used for execution
    pub backend_name: String,

    /// Execution time
    pub execution_time: std::time::Duration,

    /// Number of shots used
    pub shots: u32,

    /// Circuit depth
    pub circuit_depth: u32,

    /// Number of gates
    pub gate_count: u32,
}

/// Information about a quantum backend
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub name: String,
    pub backend_type: BackendType,
    pub max_qubits: u32,
    pub supported_gates: Vec<String>,
    pub connectivity: Option<Vec<(u32, u32)>>,
    pub basis_gates: Vec<String>,
    pub version: String,
}

/// Types of quantum backends
#[derive(Debug, Clone)]
pub enum BackendType {
    Simulator,
    Hardware,
    Cloud,
}

/// Queue status for hardware backends
#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub position: u32,
    pub estimated_wait_time: std::time::Duration,
    pub queue_length: u32,
}

/// Quantum execution errors
#[derive(Debug, Clone)]
pub enum QuantumError {
    BackendUnavailable(String),
    CircuitTooLarge(u32, u32), // requested qubits, max qubits
    InvalidGate(String),
    ExecutionTimeout,
    HardwareError(String),
    SimulationError(String),
    ConfigurationError(String),
}

/// Built-in local quantum simulator
#[derive(Debug)]
pub struct LocalSimulator {
    max_qubits: u32,
    precision: u32,
}

/// Qiskit backend integration
#[derive(Debug)]
pub struct QiskitBackend {
    backend_name: String,
    api_token: Option<String>,
    python_path: String,
    qiskit_script_path: String,
}

/// IBM Quantum hardware backend
#[derive(Debug)]
pub struct IBMQuantumBackend {
    backend_name: String,
    api_token: String,
    hub: Option<String>,
    group: Option<String>,
    project: Option<String>,
}

impl QuantumBridge {
    /// Create a new quantum bridge with default configuration
    pub fn new() -> Self {
        let mut bridge = Self {
            backends: HashMap::new(),
            default_backend: "local_simulator".to_string(),
            config: QuantumBridgeConfig::default(),
        };

        // Add default local simulator
        bridge.add_backend(
            "local_simulator".to_string(),
            Box::new(LocalSimulator::new(16, 64)),
        );

        bridge
    }

    /// Add a quantum backend
    pub fn add_backend(&mut self, name: String, backend: Box<dyn QuantumBackend>) {
        self.backends.insert(name, backend);
    }

    /// Set the default backend
    pub fn set_default_backend(&mut self, name: String) -> Result<(), QuantumError> {
        if !self.backends.contains_key(&name) {
            return Err(QuantumError::BackendUnavailable(name));
        }
        self.default_backend = name;
        Ok(())
    }

    /// Execute a quantum circuit on the default backend
    pub fn execute(&self, circuit: &QuantumCircuit) -> Result<QuantumResult, QuantumError> {
        self.execute_on_backend(circuit, &self.default_backend)
    }

    /// Execute a quantum circuit on a specific backend
    pub fn execute_on_backend(
        &self,
        circuit: &QuantumCircuit,
        backend_name: &str,
    ) -> Result<QuantumResult, QuantumError> {
        let backend = self
            .backends
            .get(backend_name)
            .ok_or_else(|| QuantumError::BackendUnavailable(backend_name.to_string()))?;

        if !backend.is_available() {
            return Err(QuantumError::BackendUnavailable(backend_name.to_string()));
        }

        // Validate circuit against backend capabilities
        let info = backend.get_info();
        if circuit.qubits > info.max_qubits {
            return Err(QuantumError::CircuitTooLarge(
                circuit.qubits,
                info.max_qubits,
            ));
        }

        // Optimize circuit if enabled
        let optimized_circuit = if self.config.optimize_circuits {
            self.optimize_circuit(circuit)?
        } else {
            circuit.clone()
        };

        // Execute the circuit
        backend.execute_circuit(&optimized_circuit)
    }

    /// Optimize a quantum circuit
    fn optimize_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit, QuantumError> {
        // Simple optimization: remove identity gates and combine adjacent gates
        let mut optimized_gates = Vec::new();

        for gate in &circuit.gates {
            match gate {
                // Skip identity-like operations
                QuantumGate::RZ(_, angle) if angle.abs() < 1e-10 => continue,
                QuantumGate::RY(_, angle) if angle.abs() < 1e-10 => continue,
                QuantumGate::RX(_, angle) if angle.abs() < 1e-10 => continue,
                _ => optimized_gates.push(gate.clone()),
            }
        }

        Ok(QuantumCircuit {
            qubits: circuit.qubits,
            gates: optimized_gates,
            measurements: circuit.measurements.clone(),
        })
    }

    /// Get information about all available backends
    pub fn list_backends(&self) -> Vec<BackendInfo> {
        self.backends
            .values()
            .map(|backend| backend.get_info())
            .collect()
    }

    /// Configure Qiskit integration
    pub fn setup_qiskit(
        &mut self,
        api_token: Option<String>,
        backend_name: String,
    ) -> Result<(), QuantumError> {
        let qiskit_backend = QiskitBackend::new(backend_name.clone(), api_token)?;
        self.add_backend(format!("qiskit_{}", backend_name), Box::new(qiskit_backend));
        Ok(())
    }

    /// Configure IBM Quantum hardware access
    pub fn setup_ibm_quantum(
        &mut self,
        api_token: String,
        backend_name: String,
    ) -> Result<(), QuantumError> {
        let ibm_backend =
            IBMQuantumBackend::new(backend_name.clone(), api_token, None, None, None)?;
        self.add_backend(format!("ibm_{}", backend_name), Box::new(ibm_backend));
        Ok(())
    }
}

impl LocalSimulator {
    pub fn new(max_qubits: u32, precision: u32) -> Self {
        Self {
            max_qubits,
            precision,
        }
    }
}

impl QuantumBackend for LocalSimulator {
    fn execute_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumResult, QuantumError> {
        let start_time = std::time::Instant::now();

        // Initialize quantum state vector
        let mut state_vector = vec![(0.0, 0.0); 1 << circuit.qubits];
        state_vector[0] = (1.0, 0.0); // |00...0⟩ state

        // Apply quantum gates
        for gate in &circuit.gates {
            self.apply_gate(&mut state_vector, gate, circuit.qubits)?;
        }

        // Perform measurements
        let mut measurements = Vec::new();
        for &qubit_index in &circuit.measurements {
            let measurement = self.measure_qubit(&state_vector, qubit_index, circuit.qubits);
            measurements.push(measurement);
        }

        // Calculate probabilities
        let mut probabilities = HashMap::new();
        for i in 0..state_vector.len() {
            let prob =
                state_vector[i].0 * state_vector[i].0 + state_vector[i].1 * state_vector[i].1;
            if prob > 1e-10 {
                let bitstring = format!("{:0width$b}", i, width = circuit.qubits as usize);
                probabilities.insert(bitstring, prob);
            }
        }

        Ok(QuantumResult {
            measurements,
            probabilities,
            final_state: Some(state_vector),
            metadata: QuantumExecutionMetadata {
                backend_name: "local_simulator".to_string(),
                execution_time: start_time.elapsed(),
                shots: 1, // Single shot for simulator
                circuit_depth: circuit.gates.len() as u32,
                gate_count: circuit.gates.len() as u32,
            },
        })
    }

    fn get_info(&self) -> BackendInfo {
        BackendInfo {
            name: "local_simulator".to_string(),
            backend_type: BackendType::Simulator,
            max_qubits: self.max_qubits,
            supported_gates: vec![
                "H".to_string(),
                "X".to_string(),
                "Y".to_string(),
                "Z".to_string(),
                "CNOT".to_string(),
                "RZ".to_string(),
                "RY".to_string(),
                "RX".to_string(),
                "Phase".to_string(),
                "T".to_string(),
                "S".to_string(),
            ],
            connectivity: None, // Fully connected
            basis_gates: vec!["H".to_string(), "CNOT".to_string(), "RZ".to_string()],
            version: "1.0.0".to_string(),
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get_queue_status(&self) -> Option<QueueStatus> {
        None // Simulators don't have queues
    }
}

impl LocalSimulator {
    fn apply_gate(
        &self,
        state: &mut Vec<(f64, f64)>,
        gate: &QuantumGate,
        num_qubits: u32,
    ) -> Result<(), QuantumError> {
        match gate {
            QuantumGate::H(qubit) => self.apply_hadamard(state, *qubit, num_qubits),
            QuantumGate::X(qubit) => self.apply_pauli_x(state, *qubit, num_qubits),
            QuantumGate::Y(qubit) => self.apply_pauli_y(state, *qubit, num_qubits),
            QuantumGate::Z(qubit) => self.apply_pauli_z(state, *qubit, num_qubits),
            QuantumGate::CNOT(control, target) => {
                self.apply_cnot(state, *control, *target, num_qubits)
            }
            QuantumGate::RZ(qubit, angle) => self.apply_rz(state, *qubit, *angle, num_qubits),
            _ => Err(QuantumError::InvalidGate(format!("{:?}", gate))),
        }
    }

    fn apply_hadamard(
        &self,
        state: &mut Vec<(f64, f64)>,
        qubit: u32,
        num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let n = 1 << num_qubits;
        let qubit_mask = 1 << qubit;
        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;

        for i in 0..n {
            let j = i ^ qubit_mask;
            if i < j {
                let (a_real, a_imag) = state[i];
                let (b_real, b_imag) = state[j];

                state[i] = (inv_sqrt2 * (a_real + b_real), inv_sqrt2 * (a_imag + b_imag));
                state[j] = (inv_sqrt2 * (a_real - b_real), inv_sqrt2 * (a_imag - b_imag));
            }
        }

        Ok(())
    }

    fn apply_pauli_x(
        &self,
        state: &mut Vec<(f64, f64)>,
        qubit: u32,
        num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let n = 1 << num_qubits;
        let qubit_mask = 1 << qubit;

        for i in 0..n {
            let j = i ^ qubit_mask;
            if i < j {
                state.swap(i, j);
            }
        }

        Ok(())
    }

    fn apply_pauli_y(
        &self,
        state: &mut Vec<(f64, f64)>,
        qubit: u32,
        num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let n = 1 << num_qubits;
        let qubit_mask = 1 << qubit;

        for i in 0..n {
            let j = i ^ qubit_mask;
            if i < j {
                let (a_real, a_imag) = state[i];
                let (b_real, b_imag) = state[j];

                if (i & qubit_mask) == 0 {
                    // |0⟩ -> i|1⟩
                    state[i] = (b_imag, -b_real);
                    // |1⟩ -> -i|0⟩
                    state[j] = (-a_imag, a_real);
                }
            }
        }

        Ok(())
    }

    fn apply_pauli_z(
        &self,
        state: &mut Vec<(f64, f64)>,
        qubit: u32,
        _num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let qubit_mask = 1 << qubit;

        for i in 0..state.len() {
            if (i & qubit_mask) != 0 {
                state[i].0 = -state[i].0;
                state[i].1 = -state[i].1;
            }
        }

        Ok(())
    }

    fn apply_cnot(
        &self,
        state: &mut Vec<(f64, f64)>,
        control: u32,
        target: u32,
        num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let n = 1 << num_qubits;
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        for i in 0..n {
            if (i & control_mask) != 0 {
                let j = i ^ target_mask;
                if i != j {
                    state.swap(i, j);
                }
            }
        }

        Ok(())
    }

    fn apply_rz(
        &self,
        state: &mut Vec<(f64, f64)>,
        qubit: u32,
        angle: f64,
        _num_qubits: u32,
    ) -> Result<(), QuantumError> {
        let qubit_mask = 1 << qubit;
        let cos_half = (angle / 2.0).cos();
        let sin_half = (angle / 2.0).sin();

        for i in 0..state.len() {
            if (i & qubit_mask) != 0 {
                let (real, imag) = state[i];
                state[i] = (
                    real * cos_half + imag * sin_half,
                    imag * cos_half - real * sin_half,
                );
            }
        }

        Ok(())
    }

    fn measure_qubit(&self, state: &Vec<(f64, f64)>, qubit: u32, _num_qubits: u32) -> f64 {
        let qubit_mask = 1 << qubit;
        let mut prob_one = 0.0;

        for i in 0..state.len() {
            if (i & qubit_mask) != 0 {
                let amplitude_sq = state[i].0 * state[i].0 + state[i].1 * state[i].1;
                prob_one += amplitude_sq;
            }
        }

        // Simulate measurement
        if rand::random::<f64>() < prob_one {
            1.0
        } else {
            0.0
        }
    }
}

impl QiskitBackend {
    pub fn new(backend_name: String, api_token: Option<String>) -> Result<Self, QuantumError> {
        // Check if Python and Qiskit are available
        let python_path = Self::find_python_with_qiskit()?;
        let script_path = Self::create_qiskit_script()?;

        Ok(Self {
            backend_name,
            api_token,
            python_path,
            qiskit_script_path: script_path,
        })
    }

    fn find_python_with_qiskit() -> Result<String, QuantumError> {
        // Try common Python installations
        let python_commands = ["python3", "python", "py"];

        for cmd in &python_commands {
            if let Ok(output) = Command::new(cmd)
                .arg("-c")
                .arg("import qiskit; print('ok')")
                .output()
            {
                if output.status.success() {
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(QuantumError::ConfigurationError(
            "Qiskit not found".to_string(),
        ))
    }

    fn create_qiskit_script() -> Result<String, QuantumError> {
        let script_content = r#"
import sys
import json
import qiskit
from qiskit import QuantumCircuit, transpile, assemble
from qiskit.providers.aer import AerSimulator

def execute_circuit(circuit_json, backend_name, shots=1024):
    # Parse circuit from JSON
    circuit_data = json.loads(circuit_json)
    
    # Create quantum circuit
    qc = QuantumCircuit(circuit_data['qubits'], circuit_data['qubits'])
    
    # Add gates
    for gate in circuit_data['gates']:
        gate_type = gate['type']
        qubits = gate['qubits']
        
        if gate_type == 'H':
            qc.h(qubits[0])
        elif gate_type == 'X':
            qc.x(qubits[0])
        elif gate_type == 'Y':
            qc.y(qubits[0])
        elif gate_type == 'Z':
            qc.z(qubits[0])
        elif gate_type == 'CNOT':
            qc.cx(qubits[0], qubits[1])
        elif gate_type == 'RZ':
            qc.rz(gate['angle'], qubits[0])
        # Add more gates as needed
    
    # Add measurements
    for qubit in circuit_data['measurements']:
        qc.measure(qubit, qubit)
    
    # Execute circuit
    if backend_name == 'qasm_simulator':
        backend = AerSimulator()
    else:
        # For hardware backends, would need IBMQ provider setup
        backend = AerSimulator()
    
    transpiled = transpile(qc, backend)
    job = backend.run(transpiled, shots=shots)
    result = job.result()
    counts = result.get_counts()
    
    # Return results as JSON
    return json.dumps({
        'counts': counts,
        'shots': shots,
        'backend': backend_name
    })

if __name__ == '__main__':
    circuit_json = sys.argv[1]
    backend_name = sys.argv[2] if len(sys.argv) > 2 else 'qasm_simulator'
    
    result = execute_circuit(circuit_json, backend_name)
    print(result)
"#;

        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join("aeonmi_qiskit_bridge.py");

        std::fs::write(&script_path, script_content).map_err(|e| {
            QuantumError::ConfigurationError(format!("Failed to create Qiskit script: {}", e))
        })?;

        Ok(script_path.to_string_lossy().to_string())
    }
}

impl QuantumBackend for QiskitBackend {
    fn execute_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumResult, QuantumError> {
        let start_time = std::time::Instant::now();

        // Convert circuit to JSON for Python script
        let circuit_json = serde_json::json!({
            "qubits": circuit.qubits,
            "gates": circuit.gates.iter().map(|gate| match gate {
                QuantumGate::H(q) => json!({"type": "H", "qubits": [q]}),
                QuantumGate::X(q) => json!({"type": "X", "qubits": [q]}),
                QuantumGate::Y(q) => json!({"type": "Y", "qubits": [q]}),
                QuantumGate::Z(q) => json!({"type": "Z", "qubits": [q]}),
                QuantumGate::CNOT(c, t) => json!({"type": "CNOT", "qubits": [c, t]}),
                QuantumGate::RZ(q, angle) => json!({"type": "RZ", "qubits": [q], "angle": angle}),
                _ => json!({"type": "UNKNOWN", "qubits": []}),
            }).collect::<Vec<_>>(),
            "measurements": circuit.measurements
        });

        // Execute Python script
        let output = Command::new(&self.python_path)
            .arg(&self.qiskit_script_path)
            .arg(circuit_json.to_string())
            .arg(&self.backend_name)
            .output()
            .map_err(|e| QuantumError::SimulationError(format!("Failed to run Qiskit: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(QuantumError::SimulationError(format!(
                "Qiskit error: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result_json: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| {
            QuantumError::SimulationError(format!("Failed to parse Qiskit result: {}", e))
        })?;

        // Convert results back to Aeonmi format
        let counts = result_json["counts"]
            .as_object()
            .ok_or_else(|| QuantumError::SimulationError("Invalid result format".to_string()))?;

        let mut probabilities = HashMap::new();
        let total_shots = result_json["shots"].as_u64().unwrap_or(1024) as f64;

        for (bitstring, count) in counts {
            let prob = count.as_u64().unwrap_or(0) as f64 / total_shots;
            probabilities.insert(bitstring.clone(), prob);
        }

        // Extract measurements (simplified - would need more sophisticated parsing)
        let measurements: Vec<f64> = circuit
            .measurements
            .iter()
            .map(|_| {
                probabilities
                    .keys()
                    .next()
                    .map(|s| s.chars().next().unwrap().to_digit(2).unwrap() as f64)
                    .unwrap_or(0.0)
            })
            .collect();

        Ok(QuantumResult {
            measurements,
            probabilities,
            final_state: None, // Qiskit doesn't return state vector for hardware
            metadata: QuantumExecutionMetadata {
                backend_name: self.backend_name.clone(),
                execution_time: start_time.elapsed(),
                shots: total_shots as u32,
                circuit_depth: circuit.gates.len() as u32,
                gate_count: circuit.gates.len() as u32,
            },
        })
    }

    fn get_info(&self) -> BackendInfo {
        BackendInfo {
            name: format!("qiskit_{}", self.backend_name),
            backend_type: if self.backend_name.contains("simulator") {
                BackendType::Simulator
            } else {
                BackendType::Hardware
            },
            max_qubits: 32, // Default for simulators
            supported_gates: vec![
                "H".to_string(),
                "X".to_string(),
                "Y".to_string(),
                "Z".to_string(),
                "CNOT".to_string(),
                "RZ".to_string(),
                "RY".to_string(),
                "RX".to_string(),
            ],
            connectivity: None,
            basis_gates: vec![
                "rz".to_string(),
                "sx".to_string(),
                "x".to_string(),
                "cx".to_string(),
            ],
            version: "1.0.0".to_string(),
        }
    }

    fn is_available(&self) -> bool {
        // Check if Qiskit is accessible
        Command::new(&self.python_path)
            .arg("-c")
            .arg("import qiskit; print('ok')")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn get_queue_status(&self) -> Option<QueueStatus> {
        // Would need to query IBM Quantum for real queue status
        None
    }
}

impl IBMQuantumBackend {
    pub fn new(
        backend_name: String,
        api_token: String,
        hub: Option<String>,
        group: Option<String>,
        project: Option<String>,
    ) -> Result<Self, QuantumError> {
        Ok(Self {
            backend_name,
            api_token,
            hub,
            group,
            project,
        })
    }
}

impl QuantumBackend for IBMQuantumBackend {
    fn execute_circuit(&self, _circuit: &QuantumCircuit) -> Result<QuantumResult, QuantumError> {
        // This would implement IBM Quantum API integration
        // For now, fall back to simulator
        Err(QuantumError::ConfigurationError(
            "IBM Quantum hardware not implemented yet".to_string(),
        ))
    }

    fn get_info(&self) -> BackendInfo {
        BackendInfo {
            name: format!("ibm_{}", self.backend_name),
            backend_type: BackendType::Hardware,
            max_qubits: 127, // Example for IBM hardware
            supported_gates: vec![
                "rz".to_string(),
                "sx".to_string(),
                "x".to_string(),
                "cx".to_string(),
            ],
            connectivity: Some(vec![(0, 1), (1, 2), (2, 3)]), // Simplified
            basis_gates: vec![
                "rz".to_string(),
                "sx".to_string(),
                "x".to_string(),
                "cx".to_string(),
            ],
            version: "1.0.0".to_string(),
        }
    }

    fn is_available(&self) -> bool {
        // Would check IBM Quantum API status
        false
    }

    fn get_queue_status(&self) -> Option<QueueStatus> {
        // Would query IBM Quantum queue
        Some(QueueStatus {
            position: 5,
            estimated_wait_time: std::time::Duration::from_secs(300),
            queue_length: 10,
        })
    }
}

impl Default for QuantumBridgeConfig {
    fn default() -> Self {
        Self {
            optimize_circuits: true,
            max_local_qubits: 16,
            measurement_shots: 1024,
            operation_timeout: 60,
        }
    }
}

impl std::fmt::Display for QuantumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantumError::BackendUnavailable(name) => {
                write!(f, "Quantum backend '{}' is unavailable", name)
            }
            QuantumError::CircuitTooLarge(req, max) => write!(
                f,
                "Circuit requires {} qubits but backend supports only {}",
                req, max
            ),
            QuantumError::InvalidGate(gate) => write!(f, "Invalid quantum gate: {}", gate),
            QuantumError::ExecutionTimeout => write!(f, "Quantum execution timed out"),
            QuantumError::HardwareError(msg) => write!(f, "Quantum hardware error: {}", msg),
            QuantumError::SimulationError(msg) => write!(f, "Quantum simulation error: {}", msg),
            QuantumError::ConfigurationError(msg) => {
                write!(f, "Quantum configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for QuantumError {}
