//! Complete amygdala implementation for emotion processing and fear conditioning.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmygdalaNucleus {
    pub neurons: Vec<f64>,  // Activation levels
}

impl AmygdalaNucleus {
    pub fn new(n: usize) -> Self {
        Self { neurons: vec![0.0; n] }
    }

    pub fn step(&mut self, input: &[f64], modulation: f64) {
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let inp = if i < input.len() { input[i] } else { 0.0 };
            *neuron = (*neuron * 0.9 + inp * modulation).clamp(0.0, 1.0);
        }
    }
}

pub struct Amygdala {
    pub lateral: AmygdalaNucleus,   // Sensory input
    pub basal: AmygdalaNucleus,     // Associations
    pub central: AmygdalaNucleus,   // Output
    pub fear_weights: Vec<Vec<f64>>, // CS-US associations
}

impl Amygdala {
    pub fn new(size: usize) -> Self {
        Self {
            lateral: AmygdalaNucleus::new(size),
            basal: AmygdalaNucleus::new(size),
            central: AmygdalaNucleus::new(size / 2),
            fear_weights: vec![vec![0.1; size]; size],
        }
    }

    pub fn step(&mut self, sensory_input: &[f64], us_present: bool) -> Vec<f64> {
        self.lateral.step(sensory_input, 1.0);

        let mut basal_input = vec![0.0; self.basal.neurons.len()];
        for (i, &lat) in self.lateral.neurons.iter().enumerate() {
            for j in 0..basal_input.len().min(self.fear_weights[i].len()) {
                basal_input[j] += lat * self.fear_weights[i][j];
            }
        }
        self.basal.step(&basal_input, 1.0);

        let central_input: Vec<f64> = self.basal.neurons.iter().take(self.central.neurons.len()).cloned().collect();
        self.central.step(&central_input, 1.0);

        if us_present {
            self.fear_conditioning(0.01);
        }

        self.central.neurons.clone()
    }

    fn fear_conditioning(&mut self, learning_rate: f64) {
        for i in 0..self.lateral.neurons.len() {
            for j in 0..self.basal.neurons.len().min(self.fear_weights[i].len()) {
                if self.lateral.neurons[i] > 0.3 && self.basal.neurons[j] > 0.3 {
                    self.fear_weights[i][j] += learning_rate;
                    self.fear_weights[i][j] = self.fear_weights[i][j].min(1.0);
                }
            }
        }
    }

    pub fn fear_response(&self) -> f64 {
        self.central.neurons.iter().sum::<f64>() / self.central.neurons.len() as f64
    }
}
