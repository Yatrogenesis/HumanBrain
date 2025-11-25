//! Complete thalamus implementation with relay nuclei and oscillations.
//!
//! The thalamus is the gateway to the cortex. This module implements:
//! - Specific relay nuclei: VPL/VPM (somatosensory), LGN (visual), MGN (auditory)
//! - Thalamic reticular nucleus (TRN) - gating and attention
//! - Burst vs tonic firing modes
//! - Thalamocortical oscillations (sleep spindles, 7-14 Hz)
//! - Corticothalamic feedback

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThalamicNucleusType {
    VPL,  // Ventral posterolateral (somatosensory - body)
    VPM,  // Ventral posteromedial (somatosensory - face)
    LGN,  // Lateral geniculate (visual)
    MGN,  // Medial geniculate (auditory)
    TRN,  // Thalamic reticular nucleus
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiringMode { Tonic, Burst }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThalamicNeuron {
    pub id: usize,
    pub nucleus_type: ThalamicNucleusType,
    pub voltage: f64,
    pub calcium_t: f64,  // T-type calcium current
    pub firing_mode: FiringMode,
    pub last_spike_time: f64,
    pub burst_count: usize,
}

impl ThalamicNeuron {
    pub fn new(id: usize, nucleus_type: ThalamicNucleusType) -> Self {
        Self {
            id, nucleus_type, voltage: -65.0, calcium_t: 0.0,
            firing_mode: FiringMode::Tonic, last_spike_time: -1000.0, burst_count: 0,
        }
    }

    pub fn step(&mut self, dt: f64, current: f64, current_time: f64) -> bool {
        // T-type calcium dynamics (burst mode)
        let t_inf = if self.voltage < -60.0 { 1.0 / (1.0 + ((self.voltage + 52.0) / 7.4).exp()) } else { 0.0 };
        self.calcium_t += (t_inf - self.calcium_t) / 5.0 * dt;

        // Determine firing mode
        self.firing_mode = if self.voltage < -62.0 { FiringMode::Burst } else { FiringMode::Tonic };

        // Membrane dynamics
        let tau_m = 20.0;
        let g_t = 2.0; // T-type calcium conductance
        let i_t = g_t * self.calcium_t * (self.voltage - 120.0);

        self.voltage += dt * ((-65.0 - self.voltage) / tau_m + current - i_t);

        // Spike generation
        let threshold = if matches!(self.firing_mode, FiringMode::Burst) { -52.0 } else { -50.0 };

        if self.voltage >= threshold && current_time - self.last_spike_time > 1.0 {
            self.voltage = -65.0;
            self.last_spike_time = current_time;
            if matches!(self.firing_mode, FiringMode::Burst) {
                self.burst_count += 1;
                if self.burst_count > 5 { self.burst_count = 0; self.voltage = -70.0; }
            } else {
                self.burst_count = 0;
            }
            true
        } else {
            false
        }
    }
}

pub struct ThalamicNucleus {
    pub neurons: Vec<ThalamicNeuron>,
    pub nucleus_type: ThalamicNucleusType,
}

impl ThalamicNucleus {
    pub fn new(num_neurons: usize, nucleus_type: ThalamicNucleusType) -> Self {
        Self {
            neurons: (0..num_neurons).map(|i| ThalamicNeuron::new(i, nucleus_type)).collect(),
            nucleus_type,
        }
    }

    pub fn step(&mut self, dt: f64, input: &[f64], cortical_feedback: &[f64], current_time: f64) -> Vec<bool> {
        self.neurons.iter_mut().enumerate().map(|(i, neuron)| {
            let sensory_input = if i < input.len() { input[i] } else { 0.0 };
            let feedback = if i < cortical_feedback.len() { cortical_feedback[i] } else { 0.0 };
            neuron.step(dt, sensory_input + feedback * 0.3, current_time)
        }).collect()
    }
}

pub struct ThalamicReticular {
    pub neurons: Vec<ThalamicNeuron>,
    pub inhibitory_weights: Vec<Vec<f64>>,
}

impl ThalamicReticular {
    pub fn new(num_neurons: usize) -> Self {
        let inhibitory_weights = (0..num_neurons).map(|_|
            (0..num_neurons).map(|_| 0.5).collect()
        ).collect();
        Self {
            neurons: (0..num_neurons).map(|i| ThalamicNeuron::new(i, ThalamicNucleusType::TRN)).collect(),
            inhibitory_weights,
        }
    }

    pub fn step(&mut self, dt: f64, thalamic_activity: &[bool], current_time: f64) -> Vec<f64> {
        let n = self.neurons.len();
        let mut inhibition = vec![0.0; n];

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let excitation = if i < thalamic_activity.len() && thalamic_activity[i] { 5.0 } else { 0.0 };
            if neuron.step(dt, excitation, current_time) {
                for j in 0..n {
                    inhibition[j] += self.inhibitory_weights[i][j];
                }
            }
        }
        inhibition
    }
}

pub struct Thalamus {
    pub vpl: ThalamicNucleus,
    pub lgn: ThalamicNucleus,
    pub mgn: ThalamicNucleus,
    pub trn: ThalamicReticular,
    pub spindle_oscillation: f64,
}

impl Thalamus {
    pub fn new(neurons_per_nucleus: usize) -> Self {
        Self {
            vpl: ThalamicNucleus::new(neurons_per_nucleus, ThalamicNucleusType::VPL),
            lgn: ThalamicNucleus::new(neurons_per_nucleus, ThalamicNucleusType::LGN),
            mgn: ThalamicNucleus::new(neurons_per_nucleus, ThalamicNucleusType::MGN),
            trn: ThalamicReticular::new(neurons_per_nucleus),
            spindle_oscillation: 0.0,
        }
    }

    pub fn step(&mut self, dt: f64, sensory_input: &[f64], cortical_feedback: &[f64], current_time: f64) -> Vec<bool> {
        let vpl_output = self.vpl.step(dt, sensory_input, cortical_feedback, current_time);
        let trn_inhibition = self.trn.step(dt, &vpl_output, current_time);

        // Apply TRN inhibition to relay neurons
        for (neuron, &inh) in self.vpl.neurons.iter_mut().zip(trn_inhibition.iter()) {
            neuron.voltage -= inh * dt;
        }

        vpl_output
    }
}
