//! موتور شبکه عصبی و یادگیری ماشین NeuroMount به زبان Rust
//! قابل کامپایل به WebAssembly برای اجرا در مرورگر یا اتصال به پایتون با PyO3

#[derive(Clone, Debug)]
pub struct Layer {
    pub weights: Vec<Vec<f32>>,
    pub biases: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct NeuralBrain {
    pub layer_sizes: Vec<usize>,
    pub layers: Vec<Layer>,
    pub learning_rate: f32,
}

impl NeuralBrain {
    pub fn new(layer_sizes: Vec<usize>, learning_rate: f32) -> Self {
        let mut layers = Vec::new();
        for idx in 0..(layer_sizes.len().saturating_sub(1)) {
            let in_dim = layer_sizes[idx];
            let out_dim = layer_sizes[idx + 1];
            let scale = (2.0 / ((in_dim + out_dim) as f32)).sqrt();

            let mut weights = Vec::with_capacity(out_dim);
            for j in 0..out_dim {
                let mut row = Vec::with_capacity(in_dim);
                for i in 0..in_dim {
                    let pseudo = (((j * 31 + i * 17 + idx * 13) % 100) as f32 / 50.0) - 1.0;
                    row.push(pseudo * scale);
                }
                weights.push(row);
            }
            let biases = vec![0.01; out_dim];
            layers.push(Layer { weights, biases });
        }

        Self {
            layer_sizes,
            layers,
            learning_rate,
        }
    }

    pub fn forward(&self, inputs: &[f32]) -> Vec<f32> {
        let mut activations = inputs.to_vec();
        for layer in &self.layers {
            let mut next_acts = Vec::with_capacity(layer.biases.len());
            for (row, &bias) in layer.weights.iter().zip(layer.biases.iter()) {
                let sum: f32 = bias
                    + row
                        .iter()
                        .zip(activations.iter())
                        .map(|(&w, &x)| w * x)
                        .sum::<f32>();
                next_acts.push(sum.tanh());
            }
            activations = next_acts;
        }
        activations
    }

    pub fn train_step(&mut self, inputs: &[f32], targets: &[f32]) -> f32 {
        let mut layer_inputs = Vec::with_capacity(self.layers.len() + 1);
        layer_inputs.push(inputs.to_vec());

        let mut current = inputs.to_vec();
        for layer in &self.layers {
            let mut next = Vec::with_capacity(layer.biases.len());
            for (row, &bias) in layer.weights.iter().zip(layer.biases.iter()) {
                let sum: f32 = bias
                    + row
                        .iter()
                        .zip(current.iter())
                        .map(|(&w, &x)| w * x)
                        .sum::<f32>();
                next.push(sum.tanh());
            }
            current = next.clone();
            layer_inputs.push(current);
        }

        let num_layers = self.layers.len();
        let mut deltas: Vec<Vec<f32>> = vec![Vec::new(); num_layers];
        let mut loss = 0.0;

        let out_acts = &layer_inputs[num_layers];
        let mut out_delta = Vec::with_capacity(out_acts.len());
        for (&pred, &target) in out_acts.iter().zip(targets.iter()) {
            let err = pred - target;
            loss += 0.5 * err * err;
            out_delta.push(err * (1.0 - pred * pred));
        }
        deltas[num_layers - 1] = out_delta;

        for l in (0..num_layers.saturating_sub(1)).rev() {
            let acts = &layer_inputs[l + 1];
            let next_w = &self.layers[l + 1].weights;
            let next_d = &deltas[l + 1];
            let mut layer_delta = vec![0.0; acts.len()];

            for i in 0..acts.len() {
                let mut sum_err = 0.0;
                for k in 0..next_w.len() {
                    sum_err += next_w[k][i] * next_d[k];
                }
                layer_delta[i] = sum_err * (1.0 - acts[i] * acts[i]);
            }
            deltas[l] = layer_delta;
        }

        for l in 0..num_layers {
            let prev_acts = &layer_inputs[l];
            for j in 0..self.layers[l].biases.len() {
                let d = deltas[l][j];
                for i in 0..prev_acts.len() {
                    self.layers[l].weights[j][i] -= self.learning_rate * d * prev_acts[i];
                }
                self.layers[l].biases[j] -= self.learning_rate * d;
            }
        }

        loss / (targets.len().max(1) as f32)
    }
}
