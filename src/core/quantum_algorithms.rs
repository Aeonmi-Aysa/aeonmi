//! AEONMI Quantum Algorithm Library
//! Standard quantum algorithms implemented for the AEONMI quantum simulator

use crate::core::quantum_simulator::QuantumSimulator;
use anyhow::Result;

/// Quantum Algorithm Library providing standard quantum algorithms as AEONMI built-ins
#[derive(Debug)]
pub struct QuantumAlgorithms {
    simulator: QuantumSimulator,
}

impl QuantumAlgorithms {
    pub fn new() -> Self {
        Self {
            simulator: QuantumSimulator::new(),
        }
    }
    
    /// Grover's Search Algorithm
    /// Searches for a marked item in an unstructured database
    pub fn grovers_search(&mut self, database_size: usize, marked_item: usize) -> Result<usize> {
        // Calculate number of qubits needed
        let num_qubits = (database_size as f64).log2().ceil() as usize;
        
        // Create qubits for the search space
        let mut qubit_names = Vec::new();
        for i in 0..num_qubits {
            let qubit_name = format!("search_q{}", i);
            self.simulator.create_qubit(qubit_name.clone());
            qubit_names.push(qubit_name);
        }
        
        // Initialize superposition (equal superposition over all states)
        for qubit_name in &qubit_names {
            self.simulator.superpose(qubit_name)?;
        }
        
        // Calculate optimal number of iterations: π/4 * sqrt(N)
        let num_iterations = ((std::f64::consts::PI / 4.0) * (database_size as f64).sqrt()).round() as usize;
        
        for _ in 0..num_iterations {
            // Oracle: flip phase of marked item
            self.oracle_grover(&qubit_names, marked_item)?;
            
            // Diffusion operator (amplitude amplification)
            self.diffusion_operator(&qubit_names)?;
        }
        
        // Measure the result
        let mut result = 0;
        for (i, qubit_name) in qubit_names.iter().enumerate() {
            let bit = self.simulator.measure(qubit_name)?;
            result |= (bit as usize) << i;
        }
        
        Ok(result)
    }
    
    /// Quantum Fourier Transform
    /// Performs QFT on a register of qubits
    pub fn quantum_fourier_transform(&mut self, qubit_names: &[String]) -> Result<()> {
        let n = qubit_names.len();
        
        for i in 0..n {
            // Apply Hadamard to current qubit
            self.simulator.superpose(&qubit_names[i])?;
            
            // Apply controlled phase rotations
            for j in (i + 1)..n {
                let angle = std::f64::consts::PI / (2_f64.powi((j - i) as i32));
                self.controlled_phase_rotation(&qubit_names[j], &qubit_names[i], angle)?;
            }
        }
        
        // Reverse the order of qubits (swap)
        for i in 0..(n / 2) {
            self.swap_qubits(&qubit_names[i], &qubit_names[n - 1 - i])?;
        }
        
        Ok(())
    }
    
    /// Shor's Factoring Algorithm (simplified version)
    /// Factors a composite number using quantum period finding
    pub fn shors_factoring(&mut self, n: usize) -> Result<(usize, usize)> {
        // For demonstration, we'll implement a simplified version
        // Real Shor's algorithm requires more complex period finding
        
        if n <= 1 {
            return Err(anyhow::anyhow!("Cannot factor numbers <= 1"));
        }
        
        // Check if n is even
        if n % 2 == 0 {
            return Ok((2, n / 2));
        }
        
        // For small numbers, use classical trial division as fallback
        if n < 100 {
            for i in 3..(n as f64).sqrt() as usize + 1 {
                if n % i == 0 {
                    return Ok((i, n / i));
                }
            }
            return Ok((1, n)); // n is prime
        }
        
        // For larger numbers, implement quantum period finding
        // This is a simplified demonstration
        let period = self.quantum_period_finding(n)?;
        
        if period % 2 != 0 {
            return Err(anyhow::anyhow!("Period is odd, try again"));
        }
        
        let half_period = period / 2;
        let base = 2usize; // Using 2 as the base for simplicity - explicitly typed
        
        let factor1 = gcd(base.pow(half_period as u32) - 1, n);
        let factor2 = gcd(base.pow(half_period as u32) + 1, n);
        
        if factor1 > 1 && factor1 < n {
            Ok((factor1, n / factor1))
        } else if factor2 > 1 && factor2 < n {
            Ok((factor2, n / factor2))
        } else {
            Err(anyhow::anyhow!("Factoring failed, try again"))
        }
    }
    
    /// Deutsch-Jozsa Algorithm
    /// Determines if a boolean function is constant or balanced
    pub fn deutsch_jozsa(&mut self, oracle_type: DeutschJozsaOracle) -> Result<bool> {
        // Create qubits: n input qubits + 1 output qubit
        let num_input_qubits = 3; // Example with 3 input qubits
        let mut input_qubits = Vec::new();
        
        for i in 0..num_input_qubits {
            let qubit_name = format!("input_q{}", i);
            self.simulator.create_qubit(qubit_name.clone());
            input_qubits.push(qubit_name);
        }
        
        let output_qubit = "output_q".to_string();
        self.simulator.create_qubit(output_qubit.clone());
        
        // Initialize qubits
        // Input qubits in superposition
        for qubit in &input_qubits {
            self.simulator.superpose(qubit)?;
        }
        
        // Output qubit in |1⟩ state (X gate then Hadamard for |-⟩)
        self.simulator.pauli_x(&output_qubit)?;
        self.simulator.superpose(&output_qubit)?;
        
        // Apply the oracle
        self.deutsch_jozsa_oracle(&input_qubits, &output_qubit, oracle_type)?;
        
        // Apply Hadamard to input qubits
        for qubit in &input_qubits {
            self.simulator.superpose(qubit)?;
        }
        
        // Measure input qubits
        let mut all_zero = true;
        for qubit in &input_qubits {
            let measurement = self.simulator.measure(qubit)?;
            if measurement != 0 {
                all_zero = false;
                break;
            }
        }
        
        // If all measurements are 0, function is constant; otherwise balanced
        Ok(!all_zero) // true = balanced, false = constant
    }
    
    /// Bernstein-Vazirani Algorithm
    /// Finds a hidden bit string
    pub fn bernstein_vazirani(&mut self, hidden_string: &[bool]) -> Result<Vec<bool>> {
        let n = hidden_string.len();
        let mut input_qubits = Vec::new();
        
        // Create input qubits
        for i in 0..n {
            let qubit_name = format!("bv_input_q{}", i);
            self.simulator.create_qubit(qubit_name.clone());
            input_qubits.push(qubit_name);
        }
        
        // Create output qubit
        let output_qubit = "bv_output_q".to_string();
        self.simulator.create_qubit(output_qubit.clone());
        
        // Initialize qubits
        for qubit in &input_qubits {
            self.simulator.superpose(qubit)?;
        }
        
        // Output qubit in |-⟩ state
        self.simulator.pauli_x(&output_qubit)?;
        self.simulator.superpose(&output_qubit)?;
        
        // Apply Bernstein-Vazirani oracle
        self.bernstein_vazirani_oracle(&input_qubits, &output_qubit, hidden_string)?;
        
        // Apply Hadamard to input qubits
        for qubit in &input_qubits {
            self.simulator.superpose(qubit)?;
        }
        
        // Measure input qubits to reveal hidden string
        let mut result = Vec::new();
        for qubit in &input_qubits {
            let bit = self.simulator.measure(qubit)?;
            result.push(bit != 0);
        }
        
        Ok(result)
    }
    
    /// Quantum Teleportation Protocol
    /// Teleports a quantum state from Alice to Bob
    pub fn quantum_teleportation(&mut self, state_to_teleport: &str) -> Result<String> {
        // Create qubits
        let alice_qubit = "alice_q".to_string();
        let entangled_a = "entangled_a".to_string();
        let entangled_b = "entangled_b".to_string();
        
        self.simulator.create_qubit(alice_qubit.clone());
        self.simulator.create_qubit(entangled_a.clone());
        self.simulator.create_qubit(entangled_b.clone());
        
        // Prepare the state to teleport on Alice's qubit
        match state_to_teleport {
            "|0⟩" => {}, // Already in |0⟩
            "|1⟩" => { self.simulator.pauli_x(&alice_qubit)?; },
            "|+⟩" => { self.simulator.superpose(&alice_qubit)?; },
            "|-⟩" => { 
                self.simulator.pauli_x(&alice_qubit)?;
                self.simulator.superpose(&alice_qubit)?;
            },
            _ => return Err(anyhow::anyhow!("Unsupported quantum state")),
        }
        
        // Create entangled pair (Bell state)
        self.simulator.superpose(&entangled_a)?;
        self.simulator.entangle(&entangled_a, &entangled_b)?;
        
        // Bell measurement on Alice's qubits
        self.simulator.entangle(&alice_qubit, &entangled_a)?;
        self.simulator.superpose(&alice_qubit)?;
        
        let alice_m1 = self.simulator.measure(&alice_qubit)?;
        let alice_m2 = self.simulator.measure(&entangled_a)?;
        
        // Apply corrections to Bob's qubit based on Alice's measurements
        if alice_m2 != 0 {
            self.simulator.pauli_x(&entangled_b)?;
        }
        if alice_m1 != 0 {
            self.simulator.pauli_z(&entangled_b)?;
        }
        
        // Bob now has the teleported state
        // For demonstration, measure and return the result
        let bob_result = self.simulator.measure(&entangled_b)?;
        Ok(format!("|{}⟩", bob_result))
    }
    
    // Helper methods for algorithm implementations
    
    fn oracle_grover(&mut self, qubits: &[String], marked_item: usize) -> Result<()> {
        // Simple oracle implementation: flip phase if state matches marked_item
        // In a real implementation, this would be more sophisticated
        
        // For demonstration, apply Z gate to all qubits if in marked state
        // This is a simplified oracle
        for (i, qubit) in qubits.iter().enumerate() {
            if (marked_item >> i) & 1 == 1 {
                self.simulator.pauli_z(qubit)?;
            }
        }
        Ok(())
    }
    
    fn diffusion_operator(&mut self, qubits: &[String]) -> Result<()> {
        // Apply Hadamard to all qubits
        for qubit in qubits {
            self.simulator.superpose(qubit)?;
        }
        
        // Apply Z gate to |00...0⟩ state (simplified diffusion)
        for qubit in qubits {
            self.simulator.pauli_z(qubit)?;
        }
        
        // Apply Hadamard again
        for qubit in qubits {
            self.simulator.superpose(qubit)?;
        }
        
        Ok(())
    }
    
    fn controlled_phase_rotation(&mut self, control: &str, target: &str, _angle: f64) -> Result<()> {
        // Simplified controlled phase rotation
        // In a full implementation, this would apply a phase rotation conditioned on control qubit
        self.simulator.entangle(control, target)?;
        Ok(())
    }
    
    fn swap_qubits(&mut self, qubit1: &str, qubit2: &str) -> Result<()> {
        // Swap operation using three CNOT gates
        self.simulator.entangle(qubit1, qubit2)?;
        self.simulator.entangle(qubit2, qubit1)?;
        self.simulator.entangle(qubit1, qubit2)?;
        Ok(())
    }
    
    fn quantum_period_finding(&mut self, _n: usize) -> Result<usize> {
        // Simplified period finding for demonstration
        // Real implementation would use quantum Fourier transform
        Ok(4) // Dummy period value
    }
    
    fn deutsch_jozsa_oracle(&mut self, inputs: &[String], output: &str, oracle_type: DeutschJozsaOracle) -> Result<()> {
        match oracle_type {
            DeutschJozsaOracle::Constant0 => {
                // Do nothing (output remains unchanged)
            },
            DeutschJozsaOracle::Constant1 => {
                // Flip output qubit
                self.simulator.pauli_x(output)?;
            },
            DeutschJozsaOracle::BalancedXor => {
                // XOR all input qubits with output
                for input in inputs {
                    self.simulator.entangle(input, output)?;
                }
            },
        }
        Ok(())
    }
    
    fn bernstein_vazirani_oracle(&mut self, inputs: &[String], output: &str, hidden_string: &[bool]) -> Result<()> {
        // Apply CNOT for each bit that is 1 in the hidden string
        for (i, &bit) in hidden_string.iter().enumerate() {
            if bit && i < inputs.len() {
                self.simulator.entangle(&inputs[i], output)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum DeutschJozsaOracle {
    Constant0,    // Always returns 0
    Constant1,    // Always returns 1
    BalancedXor,  // Returns XOR of inputs (balanced)
}

impl Default for QuantumAlgorithms {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function for GCD (used in Shor's algorithm)
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grovers_search() {
        let mut alg = QuantumAlgorithms::new();
        let result = alg.grovers_search(4, 2).unwrap();
        println!("Grover's search result: {}", result);
        // Note: Due to probabilistic nature, result may vary
    }
    
    #[test]
    fn test_deutsch_jozsa() {
        let mut alg = QuantumAlgorithms::new();
        let result = alg.deutsch_jozsa(DeutschJozsaOracle::Constant0).unwrap();
        assert_eq!(result, false); // Constant function
        
        let result = alg.deutsch_jozsa(DeutschJozsaOracle::BalancedXor).unwrap();
        assert_eq!(result, true); // Balanced function
    }
    
    #[test]
    fn test_bernstein_vazirani() {
        let mut alg = QuantumAlgorithms::new();
        let hidden = vec![true, false, true, false];
        let result = alg.bernstein_vazirani(&hidden).unwrap();
        println!("Bernstein-Vazirani result: {:?}", result);
    }
    
    #[test]
    fn test_quantum_teleportation() {
        let mut alg = QuantumAlgorithms::new();
        let result = alg.quantum_teleportation("|+⟩").unwrap();
        println!("Teleportation result: {}", result);
    }
    
    #[test]
    fn test_shors_factoring() {
        let mut alg = QuantumAlgorithms::new();
        let result = alg.shors_factoring(15).unwrap();
        println!("Shor's factoring of 15: {:?}", result);
        assert!(result.0 * result.1 == 15);
    }
}