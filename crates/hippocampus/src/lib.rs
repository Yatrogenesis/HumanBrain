//! Complete hippocampus implementation with DG, CA3, CA1, and realistic dynamics.

use serde::{Deserialize, Serialize};
use rand::Rng;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HippocampalNeuronType {
    GranuleCell, MossyCell, PyramidalCA3, PyramidalCA1,
    InterneuronPV, InterneuronSST, InterneuronCCK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HippocampalNeuron {
    pub id: usize, pub neuron_type: HippocampalNeuronType,
    pub voltage: f64, pub threshold: f64, pub spike_times: Vec<f64>,
    pub position: [f64; 2], pub place_field_center: Option<[f64; 2]>,
    pub place_field_size: f64, pub firing_rate: f64, pub adaptation: f64,
    pub last_spike_time: f64, pub refractory_period: f64,
}

impl HippocampalNeuron {
    pub fn new(id: usize, neuron_type: HippocampalNeuronType) -> Self {
        let (threshold, refractory) = match neuron_type {
            HippocampalNeuronType::GranuleCell => (-50.0, 2.0),
            HippocampalNeuronType::PyramidalCA3 => (-54.0, 2.0),
            HippocampalNeuronType::PyramidalCA1 => (-54.0, 2.0),
            _ => (-52.0, 1.5),
        };
        Self { id, neuron_type, voltage: -65.0, threshold, spike_times: Vec::new(),
            position: [0.0, 0.0], place_field_center: None, place_field_size: 30.0,
            firing_rate: 0.0, adaptation: 0.0, last_spike_time: -1000.0,
            refractory_period: refractory }
    }
    pub fn step(&mut self, dt: f64, current: f64, current_time: f64) -> bool {
        if current_time - self.last_spike_time < self.refractory_period { return false; }
        self.voltage += dt * ((-65.0 - self.voltage) / 20.0 + 100.0 * current - self.adaptation);
        self.adaptation *= (-dt / 100.0).exp();
        if self.voltage >= self.threshold {
            self.voltage = -65.0; self.last_spike_time = current_time;
            self.spike_times.push(current_time); self.adaptation += 0.5; true
        } else { false }
    }
    pub fn place_cell_rate(&self, pos: [f64; 2]) -> f64 {
        if let Some(c) = self.place_field_center {
            let d = ((pos[0]-c[0]).powi(2) + (pos[1]-c[1]).powi(2)).sqrt();
            20.0 * (-d*d / (2.0*self.place_field_size*self.place_field_size)).exp()
        } else { 0.0 }
    }
}

pub struct DentateGyrus {
    pub granule_cells: Vec<HippocampalNeuron>,
}
impl DentateGyrus {
    pub fn new(n: usize) -> Self {
        Self { granule_cells: (0..n).map(|i| HippocampalNeuron::new(i, HippocampalNeuronType::GranuleCell)).collect() }
    }
    pub fn step(&mut self, dt: f64, input: &[f64], t: f64) -> Vec<bool> {
        let inh = input.iter().sum::<f64>() * 0.8;
        self.granule_cells.iter_mut().enumerate().map(|(i,n)| {
            let ex = if i < input.len() { input[i] } else { 0.0 };
            n.step(dt, ex - inh, t)
        }).collect()
    }
}

pub struct CA3Region {
    pub pyramidal_cells: Vec<HippocampalNeuron>,
    pub recurrent_weights: Vec<Vec<f64>>,
}
impl CA3Region {
    pub fn new(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let rw = (0..n).map(|i| (0..n).map(|j|
            if i != j && rng.gen::<f64>() < 0.1 { rng.gen::<f64>() * 0.1 } else { 0.0 }
        ).collect()).collect();
        Self { pyramidal_cells: (0..n).map(|i| HippocampalNeuron::new(i, HippocampalNeuronType::PyramidalCA3)).collect(), recurrent_weights: rw }
    }
    pub fn step(&mut self, dt: f64, dg: &[bool], t: f64) -> Vec<bool> {
        let n = self.pyramidal_cells.len();
        let mut ri = vec![0.0; n];
        for i in 0..n { for j in 0..n {
            if self.pyramidal_cells[j].voltage > -60.0 { ri[i] += self.recurrent_weights[j][i]; }
        }}
        self.pyramidal_cells.iter_mut().enumerate().map(|(i,neu)| {
            let dg_i = if i < dg.len() && dg[i] { 10.0 } else { 0.0 };
            neu.step(dt, dg_i + ri[i], t)
        }).collect()
    }
}

pub struct CA1Region {
    pub pyramidal_cells: Vec<HippocampalNeuron>,
    pub theta_phase: f64, pub theta_frequency: f64,
}
impl CA1Region {
    pub fn new(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = Vec::new();
        for i in 0..n {
            let mut neu = HippocampalNeuron::new(i, HippocampalNeuronType::PyramidalCA1);
            neu.place_field_center = Some([rng.gen::<f64>() * 100.0, rng.gen::<f64>() * 100.0]);
            neu.place_field_size = 20.0 + rng.gen::<f64>() * 20.0;
            cells.push(neu);
        }
        Self { pyramidal_cells: cells, theta_phase: 0.0, theta_frequency: 7.0 }
    }
    pub fn step(&mut self, dt: f64, ca3: &[bool], pos: [f64; 2], t: f64) -> Vec<bool> {
        self.theta_phase += 2.0 * PI * self.theta_frequency * dt / 1000.0;
        if self.theta_phase > 2.0 * PI { self.theta_phase -= 2.0 * PI; }
        let tm = 0.5 + 0.5 * self.theta_phase.cos();
        self.pyramidal_cells.iter_mut().enumerate().map(|(i,n)| {
            let ca3_i = if i < ca3.len() && ca3[i] { 5.0 } else { 0.0 };
            let pi = n.place_cell_rate(pos);
            n.step(dt, (ca3_i + pi) * tm, t)
        }).collect()
    }
}

pub struct Hippocampus {
    pub dentate_gyrus: DentateGyrus,
    pub ca3: CA3Region,
    pub ca1: CA1Region,
}
impl Hippocampus {
    pub fn new(scale: f64) -> Self {
        Self {
            dentate_gyrus: DentateGyrus::new((1000.0 * scale) as usize),
            ca3: CA3Region::new((300.0 * scale) as usize),
            ca1: CA1Region::new((400.0 * scale) as usize),
        }
    }
    pub fn step(&mut self, dt: f64, input: &[f64], pos: [f64; 2], t: f64) -> Vec<bool> {
        let dg = self.dentate_gyrus.step(dt, input, t);
        let ca3 = self.ca3.step(dt, &dg, t);
        self.ca1.step(dt, &ca3, pos, t)
    }
}
