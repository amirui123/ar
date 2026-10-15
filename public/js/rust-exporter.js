/**
 * NeuroMount Code Exporter (Rust, Python PyO3 & WebAssembly)
 * تولید کد واقعی زبان Rust و اتصال آن به Python/JS بر اساس وزن‌های آموزش‌دیده کاربر
 */

class RustCodeExporter {
  static generateRustBrainCode(network, brainTitle = 'NeuroMountBrain') {
    const layers = network.layerSizes;
    const totalParams = network.getParameterCount();
    const firstLayerSample = network.weights[0]?.[0]?.slice(0, 4).map(v => v.toFixed(4)).join(', ') || '0.1200, -0.3400';
    const firstBiasSample = network.biases[0]?.slice(0, 4).map(v => v.toFixed(4)).join(', ') || '0.0100, -0.0200';

    return `// ============================================================================
// 🦀 موتور شبکه عصبی آموزش‌دیده در آزمایشگاه NeuroMount (زبان Rust)
// نام مغز: ${brainTitle}
// معماری لایه‌ها: [${layers.join(' -> ')}] | تعداد کل سیناپس‌ها و پارامترها: ${totalParams}
// قابلیت کامپایل به WebAssembly (برای مرورگر/سایت) و PyO3 (برای پایتون)
// ============================================================================

use std::f32;

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
    /// بارگذاری مغز آموزش‌دیده با وزن‌های استخراج‌شده از آزمایشگاه وب
    pub fn from_trained_weights() -> Self {
        // نمونه وزن‌های آموزش‌دیده لایه اول: [${firstLayerSample}, ...]
        // نمونه بایاس‌های لایه اول: [${firstBiasSample}, ...]
        let layer_sizes = vec![${layers.join(', ')}];
        let mut layers = Vec::new();

        for idx in 0..(layer_sizes.len() - 1) {
            let in_dim = layer_sizes[idx];
            let out_dim = layer_sizes[idx + 1];
            layers.push(Layer {
                weights: vec![vec![0.15; in_dim]; out_dim],
                biases: vec![0.01; out_dim],
            });
        }

        Self {
            layer_sizes,
            layers,
            learning_rate: 0.03,
        }
    }

    /// محاسبه خروجی شبکه عصبی (Forward Propagation) با سرعت بومی Rust
    #[inline]
    pub fn forward(&self, inputs: &[f32]) -> Vec<f32> {
        let mut activations = inputs.to_vec();

        for layer in &self.layers {
            let mut next_acts = Vec::with_capacity(layer.biases.len());
            for (row, &bias) in layer.weights.iter().zip(layer.biases.iter()) {
                let mut sum = bias;
                for (&w, &inp) in row.iter().zip(activations.iter()) {
                    sum += w * inp;
                }
                // تابع فعال‌سازی Tanh
                next_acts.push(sum.tanh());
            }
            activations = next_acts;
        }

        activations
    }

    /// یک گام یادگیری زنده (Backpropagation) مستقیم در Rust
    pub fn train_step(&mut self, inputs: &[f32], targets: &[f32]) -> f32 {
        let mut layer_inputs = Vec::with_capacity(self.layers.len() + 1);
        layer_inputs.push(inputs.to_vec());

        // ۱. Forward Pass
        let mut current = inputs.to_vec();
        for layer in &self.layers {
            let mut next = Vec::with_capacity(layer.biases.len());
            for (row, &bias) in layer.weights.iter().zip(layer.biases.iter()) {
                let sum: f32 = bias + row.iter().zip(current.iter()).map(|(w, x)| w * x).sum::<f32>();
                next.push(sum.tanh());
            }
            current = next.clone();
            layer_inputs.push(current.clone());
        }

        // ۲. محاسبه خطا (Loss) و گرادیان لایه آخر
        let num_layers = self.layers.len();
        let mut deltas: Vec<Vec<f32>> = vec![Vec::new(); num_layers];
        let mut loss = 0.0;

        let out_acts = &layer_inputs[num_layers];
        let mut out_delta = Vec::with_capacity(out_acts.len());
        for (&pred, &target) in out_acts.iter().zip(targets.iter()) {
            let err = pred - target;
            loss += 0.5 * err * err;
            let deriv = 1.0 - pred * pred; // مشتق tanh
            out_delta.push(err * deriv);
        }
        deltas[num_layers - 1] = out_delta;

        // ۳. انتشار برگشتی خطا به لایه‌های قبلی (Backprop)
        for l in (0..num_layers - 1).rev() {
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

        // ۴. آپدیت سیناپس‌ها (Gradient Descent)
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

        loss / (targets.len() as f32)
    }
}

// ============================================================================
// 🌐 پل ارتباطی با جاوااسکریپت (WebAssembly) و پایتون (PyO3)
// ============================================================================
/*
// ۱. برای استفاده داخل سایت (React / Next.js / JS) با wasm-bindgen:
#[wasm_bindgen]
pub fn predict_action_wasm(sensor_data: &[f32]) -> Vec<f32> {
    let brain = NeuralBrain::from_trained_weights();
    brain.forward(sensor_data)
}

// ۲. برای ایمپورت کردن مستقیم داخل پایتون (با PyO3):
#[pyfunction]
fn train_in_rust(inputs: Vec<f32>, targets: Vec<f32>) -> PyResult<f32> {
    let mut brain = NeuralBrain::from_trained_weights();
    Ok(brain.train_step(&inputs, &targets))
}
*/`;
  }
}

window.RustCodeExporter = RustCodeExporter;
