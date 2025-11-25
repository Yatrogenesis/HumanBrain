//! Complete cerebellum for motor learning and coordination.

use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GranuleCell {
    pub id: usize,
    pub voltage: f64,
}

impl GranuleCell {
    pub fn new(id: usize) -> Self {
        Self { id, voltage: -65.0 }
    }

    pub fn step(&mut self, input: f64) -> bool {
        self.voltage += (-65.0 - self.voltage) * 0.1 + input;
        if self.voltage > -50.0 {
            self.voltage = -65.0;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurkinjeCell {
    pub id: usize,
    pub voltage: f64,
    pub parallel_fiber_weights: Vec<f64>,
    pub climbing_fiber_input: f64,
}

impl PurkinjeCell {
    pub fn new(id: usize, num_parallel_fibers: usize) -> Self {
        Self {
            id, voltage: -65.0,
            parallel_fiber_weights: vec![0.5; num_parallel_fibers],
            climbing_fiber_input: 0.0,
        }
    }

    pub fn step(&mut self, parallel_fiber_activity: &[bool], climbing_fiber: f64) -> bool {
        self.climbing_fiber_input = climbing_fiber;

        let mut pf_input = 0.0;
        for (i, &active) in parallel_fiber_activity.iter().enumerate() {
            if active && i < self.parallel_fiber_weights.len() {
                pf_input += self.parallel_fiber_weights[i];
            }
        }

        self.voltage += (-65.0 - self.voltage) * 0.1 + pf_input + climbing_fiber * 10.0;

        let spiked = self.voltage > -50.0;
        if spiked {
            self.voltage = -65.0;
        }

        // LTD: Climbing fiber causes depression of active parallel fibers
        if climbing_fiber > 0.5 {
            for (i, &active) in parallel_fiber_activity.iter().enumerate() {
                if active && i < self.parallel_fiber_weights.len() {
                    self.parallel_fiber_weights[i] *= 0.98; // Depression
                }
            }
        }

        spiked
    }

    pub fn complex_spike(&self) -> bool {
        self.climbing_fiber_input > 0.5
    }
}

pub struct DeepCerebellarNuclei {
    pub neurons: Vec<f64>,
}

impl DeepCerebellarNuclei {
    pub fn new(n: usize) -> Self {
        Self { neurons: vec![0.5; n] }
    }

    pub fn step(&mut self, purkinje_inhibition: &[bool]) {
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let inhibited = i < purkinje_inhibition.len() && purkinje_inhibition[i];
            *neuron = if inhibited { 0.0 } else { 1.0 };
        }
    }
}

pub struct Cerebellum {
    pub granule_cells: Vec<GranuleCell>,
    pub purkinje_cells: Vec<PurkinjeCell>,
    pub deep_nuclei: DeepCerebellarNuclei,
}

impl Cerebellum {
    pub fn new(num_granule: usize, num_purkinje: usize) -> Self {
        Self {
            granule_cells: (0..num_granule).map(|i| GranuleCell::new(i)).collect(),
            purkinje_cells: (0..num_purkinje).map(|i| PurkinjeCell::new(i, num_granule)).collect(),
            deep_nuclei: DeepCerebellarNuclei::new(num_purkinje / 2),
        }
    }

    pub fn step(&mut self, mossy_fiber_input: &[f64], climbing_fiber_input: &[f64]) -> Vec<f64> {
        // Granule cells receive mossy fiber input
        let parallel_fiber_activity: Vec<bool> = self.granule_cells.iter_mut().enumerate().map(|(i, gc)| {
            let input = if i < mossy_fiber_input.len() { mossy_fiber_input[i] } else { 0.0 };
            gc.step(input)
        }).collect();

        // Purkinje cells integrate parallel and climbing fibers
        let purkinje_output: Vec<bool> = self.purkinje_cells.iter_mut().enumerate().map(|(i, pc)| {
            let cf = if i < climbing_fiber_input.len() { climbing_fiber_input[i] } else { 0.0 };
            pc.step(&parallel_fiber_activity, cf)
        }).collect();

        // Deep nuclei receive Purkinje inhibition
        self.deep_nuclei.step(&purkinje_output);

        self.deep_nuclei.neurons.clone()
    }
}
