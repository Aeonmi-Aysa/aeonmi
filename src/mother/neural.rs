//! NeuralLayer — feed-forward neural layer for Mother AI pattern processing.
//! Migrated from quantum_llama_bridge/ai/neural.rs.
//! Uses plain Vec<Vec<f64>> instead of Titan Tensor (aeonmi01 doesn't have that dep).
//! Compatible with nalgebra if deeper linear-algebra is needed later.

/// Supported activation functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    ReLU,
    Sigmoid,
    Tanh,
    Linear,
}

impl Activation {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            Activation::ReLU => x.max(0.0),
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Tanh => x.tanh(),
            Activation::Linear => x,
        }
    }

    pub fn derivative(&self, output: f64) -> f64 {
        match self {
            Activation::ReLU => if output > 0.0 { 1.0 } else { 0.0 },
            Activation::Sigmoid => output * (1.0 - output),
            Activation::Tanh => 1.0 - output * output,
            Activation::Linear => 1.0,
        }
    }
}

// ─── NeuralLayer ─────────────────────────────────────────────────────────────

/// Single dense layer: weights[out][in], biases[out], activation.
#[derive(Debug, Clone)]
pub struct NeuralLayer {
    /// weights[i][j] = weight from input j to output i
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
    pub activation: Activation,
    pub input_size: usize,
    pub output_size: usize,
}

impl NeuralLayer {
    /// Create a layer with random (Xavier) initialization.
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        use std::f64::consts::PI;
        // Simple deterministic pseudo-random init using linear congruential generator
        let mut seed = (input_size * 31 + output_size * 17) as u64;
        let mut rand_f64 = move || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let u = (seed >> 33) as f64 / u32::MAX as f64;
            // Box-Muller for normal distribution
            u * 2.0 - 1.0
        };

        let xavier = (6.0_f64 / (input_size + output_size) as f64).sqrt();
        let weights: Vec<Vec<f64>> = (0..output_size)
            .map(|_| (0..input_size).map(|_| rand_f64() * xavier).collect())
            .collect();
        let biases = vec![0.0; output_size];

        Self { weights, biases, activation, input_size, output_size }
    }

    /// Create with explicit weights and biases.
    pub fn from_params(
        weights: Vec<Vec<f64>>,
        biases: Vec<f64>,
        activation: Activation,
    ) -> Result<Self, String> {
        if weights.is_empty() {
            return Err("weights must not be empty".to_string());
        }
        let output_size = weights.len();
        let input_size = weights[0].len();
        if biases.len() != output_size {
            return Err(format!(
                "bias len {} != output_size {}",
                biases.len(),
                output_size
            ));
        }
        for (i, row) in weights.iter().enumerate() {
            if row.len() != input_size {
                return Err(format!(
                    "weights row {} has {} cols, expected {}",
                    i,
                    row.len(),
                    input_size
                ));
            }
        }
        Ok(Self { weights, biases, activation, input_size, output_size })
    }

    /// Forward pass: returns output vector of length `output_size`.
    pub fn forward(&self, input: &[f64]) -> Result<Vec<f64>, String> {
        if input.len() != self.input_size {
            return Err(format!(
                "input size mismatch: expected {}, got {}",
                self.input_size,
                input.len()
            ));
        }
        let output: Vec<f64> = self.weights.iter().zip(&self.biases)
            .map(|(row, &bias)| {
                let z: f64 = row.iter().zip(input.iter()).map(|(w, x)| w * x).sum::<f64>() + bias;
                self.activation.apply(z)
            })
            .collect();
        Ok(output)
    }

    /// Simple gradient descent update (no backprop graph — single-layer use case).
    pub fn update_weights(&mut self, input: &[f64], error: &[f64], lr: f64) -> Result<(), String> {
        if input.len() != self.input_size {
            return Err("input size mismatch in update".to_string());
        }
        if error.len() != self.output_size {
            return Err("error size mismatch in update".to_string());
        }
        for (_i, (row, (&e, _b))) in self.weights.iter_mut()
            .zip(error.iter().zip(self.biases.iter_mut()))
            .enumerate()
        {
            let delta = e * self.activation.derivative(/* output */ 0.5); // approx
            for (w, &x) in row.iter_mut().zip(input.iter()) {
                *w -= lr * delta * x;
            }
        }
        // Update biases
        let output = self.forward(input)?;
        for (b, (&e, &out)) in self.biases.iter_mut().zip(error.iter().zip(output.iter())) {
            let delta = e * self.activation.derivative(out);
            *b -= lr * delta;
        }
        Ok(())
    }
}

// ─── Multi-layer network ─────────────────────────────────────────────────────

/// Stack of NeuralLayers for multi-layer inference.
pub struct NeuralNetwork {
    pub layers: Vec<NeuralLayer>,
}

impl NeuralNetwork {
    pub fn new(layers: Vec<NeuralLayer>) -> Self {
        Self { layers }
    }

    /// Build a simple feedforward network with given layer sizes.
    pub fn feedforward(sizes: &[usize], activation: Activation) -> Result<Self, String> {
        if sizes.len() < 2 {
            return Err("Need at least input and output layer sizes".to_string());
        }
        let layers = sizes.windows(2)
            .map(|pair| NeuralLayer::new(pair[0], pair[1], activation))
            .collect();
        Ok(Self { layers })
    }

    /// Forward pass through all layers.
    pub fn forward(&self, input: &[f64]) -> Result<Vec<f64>, String> {
        let mut current = input.to_vec();
        for (i, layer) in self.layers.iter().enumerate() {
            current = layer.forward(&current)
                .map_err(|e| format!("Layer {}: {}", i, e))?;
        }
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_output_size() {
        let layer = NeuralLayer::new(4, 3, Activation::ReLU);
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = layer.forward(&input).unwrap();
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn test_sigmoid_range() {
        let layer = NeuralLayer::new(3, 3, Activation::Sigmoid);
        let input = vec![1.0, -5.0, 10.0];
        let output = layer.forward(&input).unwrap();
        for &v in &output {
            assert!(v >= 0.0 && v <= 1.0, "Sigmoid output {} not in [0,1]", v);
        }
    }

    #[test]
    fn test_tanh_range() {
        let layer = NeuralLayer::new(2, 2, Activation::Tanh);
        let output = layer.forward(&[100.0, -100.0]).unwrap();
        for &v in &output {
            assert!(v >= -1.0 && v <= 1.0, "Tanh output {} not in [-1,1]", v);
        }
    }

    #[test]
    fn test_input_size_mismatch() {
        let layer = NeuralLayer::new(3, 2, Activation::ReLU);
        let result = layer.forward(&[1.0, 2.0]); // wrong size
        assert!(result.is_err());
    }

    #[test]
    fn test_from_params_validates_dims() {
        let bad = NeuralLayer::from_params(
            vec![vec![1.0, 2.0], vec![3.0]], // row 1 has 1 col, row 0 has 2
            vec![0.0, 0.0],
            Activation::Linear,
        );
        assert!(bad.is_err());
    }

    #[test]
    fn test_feedforward_network() {
        let net = NeuralNetwork::feedforward(&[4, 8, 4, 2], Activation::Tanh).unwrap();
        let output = net.forward(&[0.5, -0.5, 0.3, 0.7]).unwrap();
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_activation_relu() {
        assert_eq!(Activation::ReLU.apply(-5.0), 0.0);
        assert_eq!(Activation::ReLU.apply(3.0), 3.0);
    }
}
