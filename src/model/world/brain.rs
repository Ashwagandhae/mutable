// NEAT algorithm implementation

#[derive(Debug, Clone)]
pub struct Neuron {
    pub inputs: Vec<f64>,
    pub weights: Vec<f64>,
    pub bias: f64,
    pub output: f64,
}

impl Neuron {
    pub fn new(inputs: usize) -> Self {
        Neuron {
            inputs: vec![0.0; inputs],
            weights: vec![0.0; inputs],
            bias: 0.0,
            output: 0.0,
        }
    }
    pub fn activate(&mut self) {
        let mut sum = 0.0;
        for i in 0..self.inputs.len() {
            sum += self.inputs[i] * self.weights[i];
        }
        sum += self.bias;
        // self.output = sigmoid(sum);
    }
}

#[derive(Debug, Clone)]
pub struct Brain {
    pub neurons: Vec<Neuron>,
    pub outputs: Vec<f64>,
}

impl Brain {
    pub fn new(inputs: usize, outputs: usize) -> Self {
        let mut neurons = Vec::new();
        for _ in 0..outputs {
            neurons.push(Neuron::new(inputs));
        }
        Brain {
            neurons,
            outputs: vec![0.0; outputs],
        }
    }
    pub fn activate(&mut self, inputs: &[f64]) {
        for i in 0..self.neurons.len() {
            self.neurons[i].inputs.copy_from_slice(inputs);
            self.neurons[i].activate();
            self.outputs[i] = self.neurons[i].output;
        }
    }
}
