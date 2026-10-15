/**
 * NeuroMount Core Neural Network Engine
 * پشتیبانی از Forward Pass، Backpropagation واقعی (گرادیان کاهشی + Momentum)
 * و Neuroevolution (جهش و ترکیب ژنتیکی برای سوار کردن روی ایجنت‌ها)
 */

class NeuralNetwork {
  constructor(layerSizes = [7, 12, 8, 2], options = {}) {
    this.layerSizes = [...layerSizes];
    this.activation = options.activation || 'tanh';
    this.outputActivation = options.outputActivation || 'tanh';
    this.momentum = options.momentum ?? 0.85;

    this.weights = [];      // [layer][outNeuron][inNeuron]
    this.biases = [];       // [layer][outNeuron]
    this.weightVel = [];    // Momentum velocities for weights
    this.biasVel = [];      // Momentum velocities for biases
    this.lastActivations = layerSizes.map(size => new Array(size).fill(0));
    this.lastGradients = [];
    this.totalTrainSteps = 0;

    this.initWeights();
  }

  initWeights() {
    this.weights = [];
    this.biases = [];
    this.weightVel = [];
    this.biasVel = [];

    for (let l = 0; l < this.layerSizes.length - 1; l++) {
      const inSize = this.layerSizes[l];
      const outSize = this.layerSizes[l + 1];
      // Xavier / Glorot initialization
      const scale = Math.sqrt(2.0 / (inSize + outSize));

      const layerW = [];
      const layerWVel = [];
      const layerB = [];
      const layerBVel = [];

      for (let j = 0; j < outSize; j++) {
        const row = [];
        const rowVel = [];
        for (let i = 0; i < inSize; i++) {
          row.push((Math.random() * 2 - 1) * scale);
          rowVel.push(0);
        }
        layerW.push(row);
        layerWVel.push(rowVel);
        layerB.push((Math.random() * 2 - 1) * 0.1);
        layerBVel.push(0);
      }

      this.weights.push(layerW);
      this.weightVel.push(layerWVel);
      this.biases.push(layerB);
      this.biasVel.push(layerBVel);
    }
  }

  activate(x, type) {
    switch (type) {
      case 'relu':
        return x > 0 ? x : 0;
      case 'leaky_relu':
        return x > 0 ? x : 0.08 * x;
      case 'sigmoid':
        return 1 / (1 + Math.exp(-Math.max(-20, Math.min(20, x))));
      case 'linear':
        return x;
      case 'tanh':
      default:
        return Math.tanh(x);
    }
  }

  activateDerivative(activatedVal, type) {
    switch (type) {
      case 'relu':
        return activatedVal > 0 ? 1 : 0;
      case 'leaky_relu':
        return activatedVal > 0 ? 1 : 0.08;
      case 'sigmoid':
        return activatedVal * (1 - activatedVal);
      case 'linear':
        return 1;
      case 'tanh':
      default:
        return 1 - activatedVal * activatedVal;
    }
  }

  forward(inputs) {
    let current = inputs.slice(0, this.layerSizes[0]);
    while (current.length < this.layerSizes[0]) current.push(0);

    this.lastActivations = [current.slice()];

    const numLayers = this.weights.length;
    for (let l = 0; l < numLayers; l++) {
      const W = this.weights[l];
      const B = this.biases[l];
      const next = new Array(W.length);
      const isOutput = l === numLayers - 1;
      const actType = isOutput ? this.outputActivation : this.activation;

      for (let j = 0; j < W.length; j++) {
        let sum = B[j];
        const row = W[j];
        for (let i = 0; i < current.length; i++) {
          sum += row[i] * current[i];
        }
        next[j] = this.activate(sum, actType);
      }
      current = next;
      this.lastActivations.push(current.slice());
    }

    return current;
  }

  /**
   * Softmax با پارامتر دما (Temperature) برای مدل زبانی
   */
  static softmax(logits, temperature = 1.0) {
    const temp = Math.max(0.05, temperature);
    let maxLogit = -Infinity;
    for (let i = 0; i < logits.length; i++) {
      if (logits[i] > maxLogit) maxLogit = logits[i];
    }
    const exps = new Array(logits.length);
    let sum = 0;
    for (let i = 0; i < logits.length; i++) {
      const e = Math.exp((logits[i] - maxLogit) / temp);
      exps[i] = e;
      sum += e;
    }
    if (sum === 0) return exps.fill(1 / logits.length);
    for (let i = 0; i < exps.length; i++) {
      exps[i] /= sum;
    }
    return exps;
  }

  /**
   * آموزش یک گام با Backpropagation واقعی
   * بازگشت: مقدار خطا (Loss)
   */
  trainStep(inputs, targets, learningRate = 0.03) {
    const outputs = this.forward(inputs);
    const numLayers = this.weights.length;

    // محاسبه خطای لایه خروجی
    const deltas = new Array(numLayers);
    const outIdx = numLayers - 1;
    const outSize = this.layerSizes[numLayers];
    deltas[outIdx] = new Array(outSize);

    let loss = 0;
    for (let j = 0; j < outSize; j++) {
      const target = targets[j] ?? 0;
      const err = outputs[j] - target;
      loss += 0.5 * err * err;
      const deriv = this.activateDerivative(outputs[j], this.outputActivation);
      deltas[outIdx][j] = Math.max(-2, Math.min(2, err * deriv));
    }
    loss /= outSize;

    // انتشار برگشتی خطا به لایه‌های پنهان (Backpropagation)
    for (let l = outIdx - 1; l >= 0; l--) {
      const layerSize = this.layerSizes[l + 1];
      const nextW = this.weights[l + 1];
      const nextDeltas = deltas[l + 1];
      const layerActs = this.lastActivations[l + 1];
      deltas[l] = new Array(layerSize);

      for (let i = 0; i < layerSize; i++) {
        let sumErr = 0;
        for (let k = 0; k < nextW.length; k++) {
          sumErr += nextW[k][i] * nextDeltas[k];
        }
        const deriv = this.activateDerivative(layerActs[i], this.activation);
        deltas[l][i] = Math.max(-2, Math.min(2, sumErr * deriv));
      }
    }

    // آپدیت وزن‌ها و بایاس‌ها با Momentum
    for (let l = 0; l < numLayers; l++) {
      const W = this.weights[l];
      const B = this.biases[l];
      const WVel = this.weightVel[l];
      const BVel = this.biasVel[l];
      const prevActs = this.lastActivations[l];
      const layerDeltas = deltas[l];

      for (let j = 0; j < W.length; j++) {
        const d = layerDeltas[j];
        for (let i = 0; i < prevActs.length; i++) {
          const grad = d * prevActs[i];
          WVel[j][i] = this.momentum * WVel[j][i] - learningRate * grad;
          W[j][i] += WVel[j][i];
        }
        BVel[j] = this.momentum * BVel[j] - learningRate * d;
        B[j] += BVel[j];
      }
    }

    this.lastGradients = deltas;
    this.totalTrainSteps++;
    return loss;
  }

  /**
   * کپی کردن کامل شبکه عصبی
   */
  clone() {
    const copy = new NeuralNetwork(this.layerSizes, {
      activation: this.activation,
      outputActivation: this.outputActivation,
      momentum: this.momentum
    });
    copy.weights = this.weights.map(layer => layer.map(row => row.slice()));
    copy.biases = this.biases.map(layer => layer.slice());
    copy.totalTrainSteps = this.totalTrainSteps;
    return copy;
  }

  /**
   * جهش ژنتیکی (Mutation) برای یادگیری تقویتی و تکاملی
   */
  mutate(rate = 0.15, strength = 0.35) {
    for (let l = 0; l < this.weights.length; l++) {
      for (let j = 0; j < this.weights[l].length; j++) {
        if (Math.random() < rate) {
          this.biases[l][j] += this.gaussianRandom() * strength;
        }
        for (let i = 0; i < this.weights[l][j].length; i++) {
          if (Math.random() < rate) {
            this.weights[l][j][i] += this.gaussianRandom() * strength;
          }
        }
      }
    }
  }

  /**
   * ترکیب دو مغز برتر (Crossover)
   */
  static crossover(parentA, parentB) {
    const child = parentA.clone();
    for (let l = 0; l < child.weights.length; l++) {
      for (let j = 0; j < child.weights[l].length; j++) {
        if (Math.random() < 0.5) {
          child.biases[l][j] = parentB.biases[l][j];
        }
        for (let i = 0; i < child.weights[l][j].length; i++) {
          if (Math.random() < 0.5) {
            child.weights[l][j][i] = parentB.weights[l][j][i];
          }
        }
      }
    }
    return child;
  }

  gaussianRandom() {
    let u = 0, v = 0;
    while (u === 0) u = Math.random();
    while (v === 0) v = Math.random();
    return Math.sqrt(-2.0 * Math.log(u)) * Math.cos(2.0 * Math.PI * v);
  }

  getParameterCount() {
    let count = 0;
    for (let l = 0; l < this.weights.length; l++) {
      count += this.biases[l].length;
      for (let j = 0; j < this.weights[l].length; j++) {
        count += this.weights[l][j].length;
      }
    }
    return count;
  }

  toJSON() {
    return {
      layerSizes: this.layerSizes,
      activation: this.activation,
      outputActivation: this.outputActivation,
      weights: this.weights,
      biases: this.biases,
      totalTrainSteps: this.totalTrainSteps
    };
  }

  static fromJSON(data) {
    const net = new NeuralNetwork(data.layerSizes, {
      activation: data.activation,
      outputActivation: data.outputActivation
    });
    net.weights = data.weights.map(layer => layer.map(row => row.slice()));
    net.biases = data.biases.map(layer => layer.slice());
    net.totalTrainSteps = data.totalTrainSteps || 0;
    return net;
  }
}

window.NeuralNetwork = NeuralNetwork;
