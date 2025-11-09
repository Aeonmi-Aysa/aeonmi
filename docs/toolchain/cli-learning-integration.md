# Aeonmi CLI Learning Mode Integration

Implementation of learning mode commands and verbose output for the `aeon` CLI.

## CLI Commands for Learning Mode

### `aeon explain` Command

```rust
// src/cli/explain.rs

use clap::{Args, Subcommand};
use crate::compiler::{CompilerOptions, QuantumCompiler};
use crate::analysis::{CircuitAnalyzer, ComplexityAnalyzer};
use crate::visualization::{CircuitDiagramGenerator, StateVisualizer};

#[derive(Args)]
pub struct ExplainArgs {
    /// File to analyze and explain
    pub file: String,
    
    /// Explanation verbosity level
    #[arg(short, long, value_enum, default_value = "detailed")]
    pub verbosity: VerbosityLevel,
    
    /// Show circuit diagrams
    #[arg(long)]
    pub show_circuits: bool,
    
    /// Include mathematical details
    #[arg(long)]
    pub show_math: bool,
    
    /// Compare with classical algorithms
    #[arg(long)]
    pub compare_classical: bool,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "console")]
    pub format: OutputFormat,
    
    #[command(subcommand)]
    pub command: Option<ExplainCommand>,
}

#[derive(Subcommand)]
pub enum ExplainCommand {
    /// Explain the quantum circuit generated from code
    Circuit {
        /// Show optimization passes
        #[arg(long)]
        show_optimizations: bool,
        
        /// Include gate-level details
        #[arg(long)]
        gate_details: bool,
    },
    
    /// Explain quantum gates and operations
    Gates {
        /// Specific gate to explain
        gate: Option<String>,
        
        /// Show mathematical representation
        #[arg(long)]
        show_matrix: bool,
    },
    
    /// Analyze algorithmic complexity
    Complexity {
        /// Include quantum advantage analysis
        #[arg(long)]
        quantum_advantage: bool,
    },
    
    /// Show compilation process step-by-step
    Compilation {
        /// Stop at specific phase
        #[arg(long)]
        stop_at: Option<CompilationPhase>,
    },
    
    /// Explain quantum state evolution
    States {
        /// Show probability amplitudes
        #[arg(long)]
        show_amplitudes: bool,
        
        /// Animate state changes
        #[arg(long)]
        animate: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum VerbosityLevel {
    Basic,
    Detailed,
    Expert,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Console,
    Markdown,
    Html,
    Json,
    Slides,
}

#[derive(Clone, ValueEnum)]
pub enum CompilationPhase {
    Lexing,
    Parsing,
    Semantic,
    Circuit,
    Optimization,
    Codegen,
}

pub fn handle_explain_command(args: ExplainArgs) -> Result<(), Box<dyn std::error::Error>> {
    let learning_config = LearningConfig {
        verbosity: args.verbosity,
        show_circuits: args.show_circuits,
        show_math: args.show_math,
        compare_classical: args.compare_classical,
        output_format: args.format,
    };
    
    match args.command {
        Some(ExplainCommand::Circuit { show_optimizations, gate_details }) => {
            explain_circuit(&args.file, &learning_config, show_optimizations, gate_details)
        },
        Some(ExplainCommand::Gates { gate, show_matrix }) => {
            explain_gates(gate.as_deref(), &learning_config, show_matrix)
        },
        Some(ExplainCommand::Complexity { quantum_advantage }) => {
            explain_complexity(&args.file, &learning_config, quantum_advantage)
        },
        Some(ExplainCommand::Compilation { stop_at }) => {
            explain_compilation(&args.file, &learning_config, stop_at)
        },
        Some(ExplainCommand::States { show_amplitudes, animate }) => {
            explain_states(&args.file, &learning_config, show_amplitudes, animate)
        },
        None => {
            // General explanation of the file
            explain_general(&args.file, &learning_config)
        },
    }
}

pub struct LearningConfig {
    pub verbosity: VerbosityLevel,
    pub show_circuits: bool,
    pub show_math: bool,
    pub compare_classical: bool,
    pub output_format: OutputFormat,
}

fn explain_circuit(
    file: &str, 
    config: &LearningConfig, 
    show_optimizations: bool, 
    gate_details: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 CIRCUIT ANALYSIS: {}", file);
    println!("====================================");
    
    // Compile with learning mode enabled
    let mut compiler_options = CompilerOptions::default();
    compiler_options.learning_mode = true;
    compiler_options.verbosity = config.verbosity.clone();
    
    let compiler = QuantumCompiler::new(compiler_options);
    let compilation_result = compiler.compile_with_learning(file)?;
    
    // Analyze the generated circuit
    let circuit_analyzer = CircuitAnalyzer::new();
    let analysis = circuit_analyzer.analyze(&compilation_result.circuit)?;
    
    match config.verbosity {
        VerbosityLevel::Basic => print_basic_circuit_info(&analysis),
        VerbosityLevel::Detailed => print_detailed_circuit_info(&analysis, gate_details),
        VerbosityLevel::Expert => print_expert_circuit_info(&analysis, gate_details, show_optimizations),
    }
    
    if config.show_circuits {
        print_circuit_diagram(&compilation_result.circuit, &config.output_format)?;
    }
    
    if config.show_math {
        print_circuit_mathematics(&analysis)?;
    }
    
    if show_optimizations {
        print_optimization_details(&compilation_result.optimizations)?;
    }
    
    Ok(())
}

fn explain_gates(
    gate: Option<&str>, 
    config: &LearningConfig, 
    show_matrix: bool
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(gate_name) = gate {
        explain_specific_gate(gate_name, config, show_matrix)
    } else {
        explain_all_gates(config, show_matrix)
    }
}

fn explain_specific_gate(
    gate_name: &str, 
    config: &LearningConfig, 
    show_matrix: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚪 QUANTUM GATE: {}", gate_name.to_uppercase());
    println!("==============================");
    
    let gate_info = match gate_name.to_lowercase().as_str() {
        "h" | "hadamard" => GateInfo {
            name: "Hadamard".to_string(),
            description: "Creates superposition: |0⟩ → (|0⟩ + |1⟩)/√2".to_string(),
            matrix: Some(vec![
                vec![1.0/2.0_f64.sqrt(), 1.0/2.0_f64.sqrt()],
                vec![1.0/2.0_f64.sqrt(), -1.0/2.0_f64.sqrt()],
            ]),
            common_uses: vec![
                "Creating superposition states".to_string(),
                "Quantum algorithm initialization".to_string(),
                "Bell state preparation".to_string(),
            ],
            quantum_effect: "Rotates qubit state around X+Z axis by π radians".to_string(),
        },
        "x" | "pauli-x" | "not" => GateInfo {
            name: "Pauli-X (NOT)".to_string(),
            description: "Bit flip: |0⟩ → |1⟩, |1⟩ → |0⟩".to_string(),
            matrix: Some(vec![
                vec![0.0, 1.0],
                vec![1.0, 0.0],
            ]),
            common_uses: vec![
                "Classical NOT operation".to_string(),
                "Qubit state initialization".to_string(),
                "Error correction".to_string(),
            ],
            quantum_effect: "180° rotation around X-axis".to_string(),
        },
        "cnot" | "cx" => GateInfo {
            name: "CNOT (Controlled-X)".to_string(),
            description: "Flips target if control is |1⟩: |00⟩→|00⟩, |10⟩→|11⟩".to_string(),
            matrix: Some(vec![
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 1.0],
                vec![0.0, 0.0, 1.0, 0.0],
            ]),
            common_uses: vec![
                "Creating entanglement".to_string(),
                "Quantum algorithm building block".to_string(),
                "Bell state preparation".to_string(),
            ],
            quantum_effect: "Creates entanglement between control and target qubits".to_string(),
        },
        _ => {
            println!("❌ Unknown gate: {}", gate_name);
            println!("Available gates: hadamard, pauli-x, pauli-y, pauli-z, cnot, toffoli, phase, t, s");
            return Ok(());
        }
    };
    
    print_gate_info(&gate_info, config, show_matrix);
    
    Ok(())
}

struct GateInfo {
    name: String,
    description: String,
    matrix: Option<Vec<Vec<f64>>>,
    common_uses: Vec<String>,
    quantum_effect: String,
}

fn print_gate_info(gate: &GateInfo, config: &LearningConfig, show_matrix: bool) {
    println!("📖 Description: {}", gate.description);
    println!("⚡ Quantum Effect: {}", gate.quantum_effect);
    
    println!("\n🎯 Common Uses:");
    for (i, use_case) in gate.common_uses.iter().enumerate() {
        println!("   {}. {}", i + 1, use_case);
    }
    
    if show_matrix && gate.matrix.is_some() {
        println!("\n🔢 Matrix Representation:");
        print_matrix(gate.matrix.as_ref().unwrap());
    }
    
    match config.verbosity {
        VerbosityLevel::Expert => {
            println!("\n🧮 Advanced Details:");
            print_advanced_gate_details(&gate.name);
        },
        _ => {}
    }
}

fn print_matrix(matrix: &Vec<Vec<f64>>) {
    println!("   ┌{:^10}┐", " ".repeat(matrix[0].len() * 8));
    for row in matrix {
        print!("   │");
        for &val in row {
            print!("{:>7.3} ", val);
        }
        println!("│");
    }
    println!("   └{:^10}┘", " ".repeat(matrix[0].len() * 8));
}

fn explain_complexity(
    file: &str, 
    config: &LearningConfig, 
    quantum_advantage: bool
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 COMPLEXITY ANALYSIS: {}", file);
    println!("=====================================");
    
    let complexity_analyzer = ComplexityAnalyzer::new();
    let analysis = complexity_analyzer.analyze_file(file)?;
    
    println!("🔢 Circuit Metrics:");
    println!("   Qubits required: {}", analysis.qubit_count);
    println!("   Total gates: {}", analysis.gate_count);
    println!("   Circuit depth: {}", analysis.circuit_depth);
    println!("   Quantum volume: {}", analysis.quantum_volume);
    
    println!("\n⏱️ Complexity Classes:");
    println!("   Time complexity: {}", analysis.time_complexity);
    println!("   Space complexity: {}", analysis.space_complexity);
    
    if quantum_advantage {
        println!("\n🚀 Quantum Advantage Analysis:");
        if let Some(classical_complexity) = analysis.classical_equivalent {
            println!("   Classical best known: {}", classical_complexity);
            println!("   Quantum algorithm: {}", analysis.time_complexity);
            
            if analysis.has_quantum_advantage {
                println!("   ✅ Quantum advantage: YES");
                println!("   📈 Speedup factor: {}", analysis.speedup_factor);
            } else {
                println!("   ❌ Quantum advantage: NO");
                println!("   💡 Consider: This may be better suited for classical computation");
            }
        }
    }
    
    Ok(())
}

fn explain_compilation(
    file: &str, 
    config: &LearningConfig, 
    stop_at: Option<CompilationPhase>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 COMPILATION PROCESS: {}", file);
    println!("======================================");
    
    let mut compiler_options = CompilerOptions::default();
    compiler_options.learning_mode = true;
    compiler_options.verbosity = config.verbosity.clone();
    
    if let Some(phase) = stop_at {
        compiler_options.stop_at_phase = Some(phase);
    }
    
    let compiler = QuantumCompiler::new(compiler_options);
    let result = compiler.compile_with_detailed_logging(file)?;
    
    // Print compilation phases with explanations
    for phase in &result.phases {
        print_compilation_phase(phase, &config.verbosity);
    }
    
    if result.completed {
        println!("\n✅ Compilation completed successfully!");
        
        if config.show_circuits && result.final_circuit.is_some() {
            println!("\n🎨 Final Circuit:");
            print_circuit_diagram(result.final_circuit.as_ref().unwrap(), &config.output_format)?;
        }
    } else {
        println!("\n⏸️ Compilation stopped at requested phase");
    }
    
    Ok(())
}
```

This implementation provides comprehensive learning mode features through the CLI, allowing users to:

1. **Explain circuits** with varying levels of detail
2. **Understand quantum gates** with mathematical representations
3. **Analyze complexity** and quantum advantage
4. **Follow compilation** step-by-step
5. **Visualize quantum states** and their evolution

The learning mode integrates seamlessly with the existing CLI structure and provides educational value for users at all levels.