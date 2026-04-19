use tauri::{Manager, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use aeonmi_project::core::{
    vm::{Interpreter, Value},
    parser::Parser,
    lexer::Lexer,
    hardware_integration::{HardwareManager, QuantumCircuit, JobStatus},
    quantum_simulator::QuantumSimulator,
    quantum_algorithms::QuantumAlgorithms,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    amplitudes: HashMap<String, (f64, f64)>, // (real, imaginary)
    probabilities: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitGate {
    gate_type: String,
    qubits: Vec<usize>,
    position: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    name: String,
    provider: String,
    qubits: usize,
    available: bool,
    queue_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    success: bool,
    output: String,
    quantum_state: Option<QuantumState>,
    execution_time: f64,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    path: String,
    file_type: String,
    modified: u64,
}

// Application state
pub struct AppState {
    pub interpreter: Arc<Mutex<Interpreter>>,
    pub hardware_manager: Arc<Mutex<HardwareManager>>,
    pub current_file: Arc<Mutex<Option<String>>>,
    pub project_files: Arc<Mutex<Vec<FileInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            interpreter: Arc::new(Mutex::new(Interpreter::new())),
            hardware_manager: Arc::new(Mutex::new(HardwareManager::new())),
            current_file: Arc::new(Mutex::new(None)),
            project_files: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// Tauri commands for the quantum IDE

#[tauri::command]
pub async fn execute_aeonmi_code(
    code: String,
    state: State<'_, AppState>,
) -> Result<ExecutionResult, String> {
    let start_time = std::time::Instant::now();
    
    // Parse and execute the AEONMI code
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    let mut interpreter = state.interpreter.lock().unwrap();
    
    // Capture output
    let mut output = String::new();
    
    // Execute the code
    match interpreter.eval_module(&ast) {
        Ok(_) => {
            let execution_time = start_time.elapsed().as_secs_f64();
            
            // Get quantum state if available
            let quantum_state = get_quantum_state(&interpreter);
            
            Ok(ExecutionResult {
                success: true,
                output: "Code executed successfully".to_string(),
                quantum_state,
                execution_time,
                errors: vec![],
            })
        }
        Err(e) => {
            Ok(ExecutionResult {
                success: false,
                output: String::new(),
                quantum_state: None,
                execution_time: start_time.elapsed().as_secs_f64(),
                errors: vec![format!("Runtime error: {}", e)],
            })
        }
    }
}

#[tauri::command]
pub async fn get_quantum_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DeviceStatus>, String> {
    let hardware_manager = state.hardware_manager.lock().unwrap();
    let devices = hardware_manager.list_devices();
    
    let device_statuses: Vec<DeviceStatus> = devices.into_iter().map(|device| {
        DeviceStatus {
            name: device.name.clone(),
            provider: device.provider.to_string(),
            qubits: device.qubits,
            available: device.is_available,
            queue_length: device.queue_length,
        }
    }).collect();
    
    Ok(device_statuses)
}

#[tauri::command]
pub async fn submit_quantum_job(
    device_name: String,
    circuit_gates: Vec<String>,
    shots: usize,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut hardware_manager = state.hardware_manager.lock().unwrap();
    
    // Create quantum circuit from gate descriptions
    let mut circuit = QuantumCircuit::new(5); // Default 5 qubits
    
    for gate_str in circuit_gates {
        let parts: Vec<&str> = gate_str.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "h" if parts.len() >= 2 => {
                if let Ok(qubit) = parts[1].parse::<usize>() {
                    circuit.h(qubit);
                }
            }
            "x" if parts.len() >= 2 => {
                if let Ok(qubit) = parts[1].parse::<usize>() {
                    circuit.x(qubit);
                }
            }
            "y" if parts.len() >= 2 => {
                if let Ok(qubit) = parts[1].parse::<usize>() {
                    circuit.y(qubit);
                }
            }
            "z" if parts.len() >= 2 => {
                if let Ok(qubit) = parts[1].parse::<usize>() {
                    circuit.z(qubit);
                }
            }
            "cx" if parts.len() >= 3 => {
                if let (Ok(control), Ok(target)) = (parts[1].parse::<usize>(), parts[2].parse::<usize>()) {
                    circuit.cx(control, target);
                }
            }
            _ => {} // Ignore unknown gates
        }
    }
    
    circuit.measure_all();
    
    hardware_manager.submit_job(&device_name, circuit, shots)
        .map_err(|e| format!("Job submission failed: {}", e))
}

#[tauri::command]
pub async fn get_job_status(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let hardware_manager = state.hardware_manager.lock().unwrap();
    
    match hardware_manager.get_job_status(&job_id) {
        Some(status) => {
            let status_str = match status {
                JobStatus::Queued => "queued",
                JobStatus::Running => "running", 
                JobStatus::Completed => "completed",
                JobStatus::Failed(_) => "failed",
                JobStatus::Cancelled => "cancelled",
            };
            Ok(status_str.to_string())
        }
        None => Err("Job not found".to_string()),
    }
}

#[tauri::command]
pub async fn get_job_results(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let hardware_manager = state.hardware_manager.lock().unwrap();
    
    match hardware_manager.get_job_results(&job_id) {
        Some(results) => {
            let mut result_map = HashMap::new();
            
            // Convert counts
            let counts: HashMap<String, serde_json::Value> = results.counts.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::Number((*v as f64).into())))
                .collect();
            result_map.insert("counts".to_string(), serde_json::Value::Object(counts.into()));
            
            // Convert probabilities  
            let probabilities: HashMap<String, serde_json::Value> = results.probabilities.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::Number((*v).into())))
                .collect();
            result_map.insert("probabilities".to_string(), serde_json::Value::Object(probabilities.into()));
            
            result_map.insert("execution_time".to_string(), serde_json::Value::Number(results.execution_time.into()));
            result_map.insert("shots".to_string(), serde_json::Value::Number((results.raw_data.len() as f64).into()));
            
            Ok(result_map)
        }
        None => Err("Job results not available".to_string()),
    }
}

#[tauri::command]
pub async fn apply_quantum_gate(
    gate_type: String,
    qubits: Vec<usize>,
    state: State<'_, AppState>,
) -> Result<QuantumState, String> {
    let mut interpreter = state.interpreter.lock().unwrap();
    
    // Apply the gate through the quantum simulator
    match gate_type.as_str() {
        "H" => {
            if let Some(&qubit) = qubits.first() {
                let qubit_name = format!("q{}", qubit);
                if !interpreter.quantum_sim.qubits.contains_key(&qubit_name) {
                    interpreter.quantum_sim.create_qubit(qubit_name.clone());
                }
                interpreter.quantum_sim.hadamard(&qubit_name)
                    .map_err(|e| format!("Hadamard gate error: {}", e))?;
            }
        }
        "X" => {
            if let Some(&qubit) = qubits.first() {
                let qubit_name = format!("q{}", qubit);
                if !interpreter.quantum_sim.qubits.contains_key(&qubit_name) {
                    interpreter.quantum_sim.create_qubit(qubit_name.clone());
                }
                interpreter.quantum_sim.pauli_x(&qubit_name)
                    .map_err(|e| format!("Pauli-X gate error: {}", e))?;
            }
        }
        "CNOT" => {
            if qubits.len() >= 2 {
                let control_name = format!("q{}", qubits[0]);
                let target_name = format!("q{}", qubits[1]);
                
                if !interpreter.quantum_sim.qubits.contains_key(&control_name) {
                    interpreter.quantum_sim.create_qubit(control_name.clone());
                }
                if !interpreter.quantum_sim.qubits.contains_key(&target_name) {
                    interpreter.quantum_sim.create_qubit(target_name.clone());
                }
                
                interpreter.quantum_sim.cnot(&control_name, &target_name)
                    .map_err(|e| format!("CNOT gate error: {}", e))?;
            }
        }
        _ => return Err(format!("Unsupported gate type: {}", gate_type)),
    }
    
    Ok(get_quantum_state(&interpreter).unwrap_or_else(|| QuantumState {
        amplitudes: HashMap::new(),
        probabilities: HashMap::new(),
    }))
}

#[tauri::command] 
pub async fn reset_quantum_state(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut interpreter = state.interpreter.lock().unwrap();
    interpreter.quantum_sim = QuantumSimulator::new();
    Ok(())
}

#[tauri::command]
pub async fn save_file(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use std::fs;
    
    fs::write(&path, content)
        .map_err(|e| format!("Failed to save file: {}", e))?;
    
    // Update current file
    let mut current_file = state.current_file.lock().unwrap();
    *current_file = Some(path);
    
    Ok(())
}

#[tauri::command]
pub async fn load_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use std::fs;
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to load file: {}", e))?;
    
    // Update current file
    let mut current_file = state.current_file.lock().unwrap();
    *current_file = Some(path);
    
    Ok(content)
}

#[tauri::command]
pub async fn get_project_files(
    directory: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileInfo>, String> {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new(&directory);
    let mut files = Vec::new();
    
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let file_type = if path.is_dir() {
                    "directory".to_string()
                } else if name.ends_with(".ai") {
                    "aeonmi".to_string()
                } else {
                    "file".to_string()
                };
                
                let modified = entry.metadata()
                    .and_then(|m| m.modified())
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                    .unwrap_or(0);
                
                files.push(FileInfo {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    file_type,
                    modified,
                });
            }
        }
    }
    
    // Update project files
    let mut project_files = state.project_files.lock().unwrap();
    *project_files = files.clone();
    
    Ok(files)
}

// Helper function to extract quantum state from interpreter
fn get_quantum_state(interpreter: &Interpreter) -> Option<QuantumState> {
    let qubits = &interpreter.quantum_sim.qubits;
    if qubits.is_empty() {
        return None;
    }
    
    let mut amplitudes = HashMap::new();
    let mut probabilities = HashMap::new();
    
    // Generate all possible computational basis states
    let num_qubits = qubits.len();
    for i in 0..(1 << num_qubits) {
        let state_str = format!("{:0width$b}", i, width = num_qubits);
        
        // For demonstration, use simplified probability calculation
        let prob = 1.0 / (1 << num_qubits) as f64; // Equal superposition
        let amplitude = (prob.sqrt(), 0.0); // Real amplitude, zero imaginary
        
        amplitudes.insert(format!("|{}⟩", state_str), amplitude);
        probabilities.insert(format!("|{}⟩", state_str), prob);
    }
    
    Some(QuantumState {
        amplitudes,
        probabilities,
    })
}

pub fn setup_tauri_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            execute_aeonmi_code,
            get_quantum_devices,
            submit_quantum_job,
            get_job_status, 
            get_job_results,
            apply_quantum_gate,
            reset_quantum_state,
            save_file,
            load_file,
            get_project_files,
        ])
}