/// AEONMI Quantum Circuit Visualization
/// Provides ASCII art circuit diagrams, LaTeX output, and interactive visualization
use crate::core::circuit_builder::{QuantumCircuitBuilder, QuantumGate, QuantumGateType};

/// Circuit visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub show_measurements: bool,
    pub show_parameters: bool,
    pub show_qubit_labels: bool,
    pub ascii_style: AsciiStyle,
    pub max_width: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AsciiStyle {
    Simple,  // Basic ASCII characters
    Unicode, // Unicode box drawing characters
    Compact, // Minimal spacing
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            show_measurements: true,
            show_parameters: true,
            show_qubit_labels: true,
            ascii_style: AsciiStyle::Unicode,
            max_width: 80,
        }
    }
}

/// Circuit visualizer for AEONMI quantum circuits
#[allow(dead_code)]
pub struct CircuitVisualizer {
    config: VisualizationConfig,
}

#[allow(dead_code)]
impl CircuitVisualizer {
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(VisualizationConfig::default())
    }

    /// Generate ASCII art representation of the circuit
    pub fn to_ascii(&self, circuit: &QuantumCircuitBuilder) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("Circuit: {}\n", circuit.name));
        output.push_str(&format!(
            "Qubits: {}, Gates: {}, Depth: {}\n",
            circuit.qubit_count(),
            circuit.gate_count(),
            circuit.depth()
        ));
        output.push_str(&self.horizontal_line(self.config.max_width));
        output.push('\n');

        if circuit.qubits.is_empty() {
            output.push_str("Empty circuit\n");
            return output;
        }

        // Generate the circuit diagram
        let diagram = self.generate_circuit_diagram(circuit);
        output.push_str(&diagram);

        // Parameters section
        if self.config.show_parameters && !circuit.parameters.is_empty() {
            output.push('\n');
            output.push_str("Parameters:\n");
            for (name, value) in &circuit.parameters {
                output.push_str(&format!("  {} = {:.4}\n", name, value));
            }
        }

        output
    }

    /// Generate the main circuit diagram
    fn generate_circuit_diagram(&self, circuit: &QuantumCircuitBuilder) -> String {
        let mut lines: Vec<String> = Vec::new();
        // Initialize qubit lines
        for (i, qubit) in circuit.qubits.iter().enumerate() {
            let label = if self.config.show_qubit_labels {
                format!("{:<8}", qubit.to_string())
            } else {
                format!("q{:<7}", i)
            };
            lines.push(format!("{}│", label));
        }

        // Add gates to the diagram
        for gate in &circuit.gates {
            self.add_gate_to_diagram(&mut lines, gate, circuit);
        }

        // Finalize lines
        for line in &mut lines {
            line.push_str(&self.horizontal_char().repeat(5));
        }

        lines.join("\n")
    }

    /// Add a gate to the circuit diagram
    fn add_gate_to_diagram(
        &self,
        lines: &mut Vec<String>,
        gate: &QuantumGate,
        circuit: &QuantumCircuitBuilder,
    ) {
        let gate_symbol = self.get_gate_symbol(&gate.gate_type);
        let qubit_indices = self.get_gate_qubit_indices(gate, circuit);

        match gate.gate_type {
            QuantumGateType::CNOT => self.add_cnot_to_diagram(lines, &qubit_indices),
            QuantumGateType::CZ => self.add_controlled_gate_to_diagram(lines, &qubit_indices, "Z"),
            QuantumGateType::Toffoli => self.add_toffoli_to_diagram(lines, &qubit_indices),
            QuantumGateType::SWAP => self.add_swap_to_diagram(lines, &qubit_indices),
            QuantumGateType::Measure => self.add_measurement_to_diagram(lines, &qubit_indices),
            _ => self.add_single_gate_to_diagram(lines, &qubit_indices, &gate_symbol),
        }
    }

    /// Get gate symbol for display
    fn get_gate_symbol(&self, gate_type: &QuantumGateType) -> String {
        match gate_type {
            QuantumGateType::Hadamard => "H".to_string(),
            QuantumGateType::PauliX => "X".to_string(),
            QuantumGateType::PauliY => "Y".to_string(),
            QuantumGateType::PauliZ => "Z".to_string(),
            QuantumGateType::Phase(angle) => {
                if self.config.show_parameters {
                    format!("P({:.2})", angle)
                } else {
                    "P".to_string()
                }
            }
            QuantumGateType::RotationX(angle) => {
                if self.config.show_parameters {
                    format!("RX({:.2})", angle)
                } else {
                    "RX".to_string()
                }
            }
            QuantumGateType::RotationY(angle) => {
                if self.config.show_parameters {
                    format!("RY({:.2})", angle)
                } else {
                    "RY".to_string()
                }
            }
            QuantumGateType::RotationZ(angle) => {
                if self.config.show_parameters {
                    format!("RZ({:.2})", angle)
                } else {
                    "RZ".to_string()
                }
            }
            QuantumGateType::S => "S".to_string(),
            QuantumGateType::T => "T".to_string(),
            QuantumGateType::SDagger => "S†".to_string(),
            QuantumGateType::TDagger => "T†".to_string(),
            QuantumGateType::CNOT => "⊕".to_string(),
            QuantumGateType::CZ => "CZ".to_string(),
            QuantumGateType::CY => "CY".to_string(),
            QuantumGateType::SWAP => "×".to_string(),
            QuantumGateType::Toffoli => "⊕".to_string(),
            QuantumGateType::Fredkin => "×".to_string(),
            QuantumGateType::Measure => "📊".to_string(),
            QuantumGateType::Custom(name, _) => name.clone(),
        }
    }

    /// Get qubit indices for a gate
    fn get_gate_qubit_indices(
        &self,
        gate: &QuantumGate,
        circuit: &QuantumCircuitBuilder,
    ) -> Vec<usize> {
        gate.qubits
            .iter()
            .filter_map(|qubit_id| circuit.qubits.iter().position(|q| q == qubit_id))
            .collect()
    }

    /// Add single-qubit gate to diagram
    fn add_single_gate_to_diagram(
        &self,
        lines: &mut Vec<String>,
        qubit_indices: &[usize],
        symbol: &str,
    ) {
        let max_symbol_len = 6;
        let padded_symbol = if symbol.len() > max_symbol_len {
            format!("{}…", &symbol[..max_symbol_len - 1])
        } else {
            format!("{:^width$}", symbol, width = max_symbol_len)
        };

        for (i, line) in lines.iter_mut().enumerate() {
            if qubit_indices.contains(&i) {
                line.push_str(&format!("┤{}├", padded_symbol));
            } else {
                line.push_str(&format!(
                    "─{}─",
                    self.horizontal_char().repeat(max_symbol_len)
                ));
            }
        }
    }

    /// Add CNOT gate to diagram
    fn add_cnot_to_diagram(&self, lines: &mut Vec<String>, qubit_indices: &[usize]) {
        if qubit_indices.len() != 2 {
            return;
        }

        let control = qubit_indices[0];
        let target = qubit_indices[1];
        let min_qubit = control.min(target);
        let max_qubit = control.max(target);

        for (i, line) in lines.iter_mut().enumerate() {
            if i == control {
                line.push_str("──●──");
            } else if i == target {
                line.push_str("──⊕──");
            } else if i > min_qubit && i < max_qubit {
                line.push_str("──│──");
            } else {
                line.push_str(&format!("──{}──", self.horizontal_char().repeat(1)));
            }
        }
    }

    /// Add controlled gate to diagram
    fn add_controlled_gate_to_diagram(
        &self,
        lines: &mut Vec<String>,
        qubit_indices: &[usize],
        gate_symbol: &str,
    ) {
        if qubit_indices.len() != 2 {
            return;
        }

        let control = qubit_indices[0];
        let target = qubit_indices[1];
        let min_qubit = control.min(target);
        let max_qubit = control.max(target);

        for (i, line) in lines.iter_mut().enumerate() {
            if i == control {
                line.push_str("──●──");
            } else if i == target {
                line.push_str(&format!("──{}──", gate_symbol));
            } else if i > min_qubit && i < max_qubit {
                line.push_str("──│──");
            } else {
                line.push_str(&format!("──{}──", self.horizontal_char().repeat(1)));
            }
        }
    }

    /// Add Toffoli gate to diagram
    fn add_toffoli_to_diagram(&self, lines: &mut Vec<String>, qubit_indices: &[usize]) {
        if qubit_indices.len() != 3 {
            return;
        }

        let control1 = qubit_indices[0];
        let control2 = qubit_indices[1];
        let target = qubit_indices[2];
        let min_qubit = *qubit_indices.iter().min().unwrap();
        let max_qubit = *qubit_indices.iter().max().unwrap();

        for (i, line) in lines.iter_mut().enumerate() {
            if i == control1 || i == control2 {
                line.push_str("──●──");
            } else if i == target {
                line.push_str("──⊕──");
            } else if i > min_qubit && i < max_qubit {
                line.push_str("──│──");
            } else {
                line.push_str(&format!("──{}──", self.horizontal_char().repeat(1)));
            }
        }
    }

    /// Add SWAP gate to diagram
    fn add_swap_to_diagram(&self, lines: &mut Vec<String>, qubit_indices: &[usize]) {
        if qubit_indices.len() != 2 {
            return;
        }

        let qubit1 = qubit_indices[0];
        let qubit2 = qubit_indices[1];
        let min_qubit = qubit1.min(qubit2);
        let max_qubit = qubit1.max(qubit2);

        for (i, line) in lines.iter_mut().enumerate() {
            if i == qubit1 || i == qubit2 {
                line.push_str("──×──");
            } else if i > min_qubit && i < max_qubit {
                line.push_str("──│──");
            } else {
                line.push_str(&format!("──{}──", self.horizontal_char().repeat(1)));
            }
        }
    }

    /// Add measurement to diagram
    fn add_measurement_to_diagram(&self, lines: &mut Vec<String>, qubit_indices: &[usize]) {
        for (i, line) in lines.iter_mut().enumerate() {
            if qubit_indices.contains(&i) {
                if self.config.show_measurements {
                    line.push_str("┤📊├");
                } else {
                    line.push_str("┤ M ├");
                }
            } else {
                line.push_str(&format!("─{}─", self.horizontal_char().repeat(3)));
            }
        }
    }

    /// Get horizontal line character based on style
    fn horizontal_char(&self) -> String {
        match self.config.ascii_style {
            AsciiStyle::Simple => "-".to_string(),
            AsciiStyle::Unicode => "─".to_string(),
            AsciiStyle::Compact => "-".to_string(),
        }
    }

    /// Generate horizontal line separator
    fn horizontal_line(&self, width: usize) -> String {
        self.horizontal_char().repeat(width)
    }

    /// Generate LaTeX circuit representation
    pub fn to_latex(&self, circuit: &QuantumCircuitBuilder) -> String {
        let mut latex = String::new();

        latex.push_str("\\begin{tikzpicture}[scale=0.8]\n");
        latex.push_str("\\tikzset{every loop/.style={min distance=10mm}}\n");

        // Add qubit lines
        for (i, qubit) in circuit.qubits.iter().enumerate() {
            latex.push_str(&format!(
                "\\node[anchor=east] at (0,{}) {{$|{}\\rangle$}};\n",
                -(i as f64),
                qubit.name
            ));
            latex.push_str(&format!(
                "\\draw (0.5,{}) -- (10,{});\n",
                -(i as f64),
                -(i as f64)
            ));
        }

        // Add gates (simplified LaTeX generation)
        let mut x_pos = 1.0;
        for gate in &circuit.gates {
            latex.push_str(&self.gate_to_latex(gate, circuit, x_pos));
            x_pos += 1.5;
        }

        latex.push_str("\\end{tikzpicture}\n");
        latex
    }

    /// Convert gate to LaTeX representation
    fn gate_to_latex(
        &self,
        gate: &QuantumGate,
        circuit: &QuantumCircuitBuilder,
        x_pos: f64,
    ) -> String {
        let mut latex = String::new();
        let qubit_indices = self.get_gate_qubit_indices(gate, circuit);

        match &gate.gate_type {
            QuantumGateType::Hadamard => {
                if let Some(&qubit) = qubit_indices.first() {
                    latex.push_str(&format!(
                        "\\gate[wires=1]{{H}} ({},{});\n",
                        x_pos,
                        -(qubit as f64)
                    ));
                }
            }
            QuantumGateType::CNOT => {
                if qubit_indices.len() == 2 {
                    let control = qubit_indices[0];
                    let target = qubit_indices[1];
                    latex.push_str(&format!(
                        "\\ctrl{} ({},{});\n",
                        target as i32 - control as i32,
                        x_pos,
                        -(control as f64)
                    ));
                    latex.push_str(&format!("\\targ{{}} ({},{});\n", x_pos, -(target as f64)));
                }
            }
            _ => {
                // Generic gate representation
                if let Some(&qubit) = qubit_indices.first() {
                    let symbol = self.get_gate_symbol(&gate.gate_type);
                    latex.push_str(&format!(
                        "\\gate[wires=1]{{{}}} ({},{});\n",
                        symbol,
                        x_pos,
                        -(qubit as f64)
                    ));
                }
            }
        }

        latex
    }

    /// Generate JSON representation for web visualization
    pub fn to_json(&self, circuit: &QuantumCircuitBuilder) -> serde_json::Value {
        serde_json::json!({
            "name": circuit.name,
            "qubits": circuit.qubits.len(),
            "gates": circuit.gate_count(),
            "depth": circuit.depth(),
            "qubit_labels": circuit.qubits.iter().map(|q| q.to_string()).collect::<Vec<_>>(),
            "gate_sequence": circuit.gates.iter().enumerate().map(|(i, gate)| {
                serde_json::json!({
                    "index": i,
                    "gate_type": format!("{:?}", gate.gate_type),
                    "qubits": gate.qubits.iter().map(|q| q.to_string()).collect::<Vec<_>>(),
                    "classical_bits": gate.classical_bits,
                    "metadata": gate.metadata
                })
            }).collect::<Vec<_>>(),
            "parameters": circuit.parameters,
            "metadata": circuit.metadata
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::circuit_builder::QuantumCircuitBuilder;

    #[test]
    fn test_bell_state_visualization() {
        let mut circuit = QuantumCircuitBuilder::new("bell_state");
        let qubits = circuit.add_qubits(2);

        circuit.h(&qubits[0]).cnot(&qubits[0], &qubits[1]);

        let visualizer = CircuitVisualizer::default();
        let ascii = visualizer.to_ascii(&circuit);

        assert!(ascii.contains("bell_state"));
        assert!(ascii.contains("H"));
        assert!(ascii.contains("⊕"));
    }

    #[test]
    fn test_json_output() {
        let mut circuit = QuantumCircuitBuilder::new("test_circuit");
        let qubits = circuit.add_qubits(1);
        circuit.h(&qubits[0]);

        let visualizer = CircuitVisualizer::default();
        let json = visualizer.to_json(&circuit);

        assert_eq!(json["name"], "test_circuit");
        assert_eq!(json["qubits"], 1);
        assert_eq!(json["gates"], 1);
    }
}
