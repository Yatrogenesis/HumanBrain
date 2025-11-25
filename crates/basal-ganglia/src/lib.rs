//! Complete basal ganglia with action selection and reinforcement learning.
//!
//! Implements:
//! - Striatum (D1-MSNs, D2-MSNs) - direct/indirect pathways
//! - GPe/GPi (globus pallidus external/internal)
//! - STN (subthalamic nucleus) - stop signals
//! - SNc (substantia nigra pars compacta) - dopamine neurons
//! - Actor-critic reinforcement learning
//! - Parkinsonian dynamics simulation

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MSNType { D1, D2 }  // Medium spiny neurons

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediumSpinyNeuron {
    pub id: usize,
    pub msn_type: MSNType,
    pub voltage: f64,
    pub dopamine_level: f64,
    pub synaptic_weights: Vec<f64>,
}

impl MediumSpinyNeuron {
    pub fn new(id: usize, msn_type: MSNType, num_inputs: usize) -> Self {
        Self {
            id, msn_type, voltage: -85.0,  // Very hyperpolarized at rest
            dopamine_level: 0.2,
            synaptic_weights: vec![0.1; num_inputs],
        }
    }

    pub fn step(&mut self, dt: f64, input: &[f64], dopamine: f64, current_time: f64) -> bool {
        self.dopamine_level = dopamine;

        // D1 excited by DA, D2 inhibited by DA
        let da_modulation = match self.msn_type {
            MSNType::D1 => 1.0 + 0.5 * dopamine,
            MSNType::D2 => 1.0 - 0.4 * dopamine,
        };

        let mut total_input = 0.0;
        for (i, &inp) in input.iter().enumerate() {
            if i < self.synaptic_weights.len() {
                total_input += inp * self.synaptic_weights[i];
            }
        }

        total_input *= da_modulation;

        // Up-state/down-state dynamics
        let tau = 30.0;
        let threshold = if self.voltage > -60.0 { -50.0 } else { -45.0 };  // Bistability

        self.voltage += dt * ((-85.0 - self.voltage) / tau + total_input);

        if self.voltage >= threshold {
            self.voltage = -85.0;
            true
        } else {
            false
        }
    }

    pub fn update_weights(&mut self, reward: f64, learning_rate: f64) {
        // Reward-modulated plasticity
        let delta = match self.msn_type {
            MSNType::D1 => reward * learning_rate,
            MSNType::D2 => -reward * learning_rate,
        };

        for weight in &mut self.synaptic_weights {
            *weight += delta;
            *weight = weight.clamp(0.0, 1.0);
        }
    }
}

pub struct Striatum {
    pub d1_msns: Vec<MediumSpinyNeuron>,
    pub d2_msns: Vec<MediumSpinyNeuron>,
}

impl Striatum {
    pub fn new(num_neurons: usize, num_inputs: usize) -> Self {
        let half = num_neurons / 2;
        Self {
            d1_msns: (0..half).map(|i| MediumSpinyNeuron::new(i, MSNType::D1, num_inputs)).collect(),
            d2_msns: (half..num_neurons).map(|i| MediumSpinyNeuron::new(i, MSNType::D2, num_inputs)).collect(),
        }
    }

    pub fn step(&mut self, dt: f64, cortical_input: &[f64], dopamine: f64, t: f64) -> (Vec<bool>, Vec<bool>) {
        let d1 = self.d1_msns.iter_mut().map(|n| n.step(dt, cortical_input, dopamine, t)).collect();
        let d2 = self.d2_msns.iter_mut().map(|n| n.step(dt, cortical_input, dopamine, t)).collect();
        (d1, d2)
    }

    pub fn apply_reward(&mut self, reward: f64) {
        for neuron in &mut self.d1_msns {
            neuron.update_weights(reward, 0.01);
        }
        for neuron in &mut self.d2_msns {
            neuron.update_weights(reward, 0.01);
        }
    }
}

pub struct GlobalPallidus {
    pub gpe_neurons: usize,
    pub gpi_neurons: usize,
    pub gpe_activity: Vec<f64>,
    pub gpi_activity: Vec<f64>,
}

impl GlobalPallidus {
    pub fn new(n: usize) -> Self {
        Self {
            gpe_neurons: n,
            gpi_neurons: n,
            gpe_activity: vec![0.0; n],
            gpi_activity: vec![0.0; n],
        }
    }

    pub fn step(&mut self, d1_output: &[bool], d2_output: &[bool], stn_output: &[bool]) -> Vec<f64> {
        // GPe receives inhibition from D2 pathway
        for i in 0..self.gpe_neurons.min(d2_output.len()) {
            self.gpe_activity[i] = if d2_output[i] { 0.0 } else { 1.0 };
        }

        // GPi receives inhibition from D1 and excitation from STN
        for i in 0..self.gpi_neurons.min(d1_output.len()) {
            let inhibition = if d1_output[i] { -1.0 } else { 0.0 };
            let excitation = if i < stn_output.len() && stn_output[i] { 1.0 } else { 0.0 };
            self.gpi_activity[i] = (1.0 + inhibition + excitation).max(0.0);
        }

        // GPi inhibits thalamus - output is disinhibition
        self.gpi_activity.iter().map(|&x| 1.0 - x).collect()
    }
}

pub struct SubthalamicNucleus {
    pub neurons: usize,
    pub activity: Vec<bool>,
}

impl SubthalamicNucleus {
    pub fn new(n: usize) -> Self {
        Self { neurons: n, activity: vec![false; n] }
    }

    pub fn step(&mut self, gpe_activity: &[f64], conflict_signal: f64) -> Vec<bool> {
        // STN provides stop signal during conflict
        for i in 0..self.neurons.min(gpe_activity.len()) {
            let inhibition = gpe_activity[i];
            let activation = conflict_signal * (1.0 - inhibition);
            self.activity[i] = activation > 0.5;
        }
        self.activity.clone()
    }
}

pub struct SubstantiaNigra {
    pub dopamine_neurons: usize,
    pub dopamine_level: f64,
    pub baseline_dopamine: f64,
    pub reward_history: Vec<f64>,
}

impl SubstantiaNigra {
    pub fn new(n: usize) -> Self {
        Self {
            dopamine_neurons: n,
            dopamine_level: 0.2,
            baseline_dopamine: 0.2,
            reward_history: Vec::new(),
        }
    }

    pub fn step(&mut self, reward: f64, expected_reward: f64) -> f64 {
        // Temporal difference error
        let prediction_error = reward - expected_reward;

        // Dopamine phasic response
        if prediction_error > 0.0 {
            self.dopamine_level = (self.baseline_dopamine + prediction_error).min(1.0);
        } else {
            self.dopamine_level = (self.baseline_dopamine + prediction_error * 0.5).max(0.0);
        }

        self.reward_history.push(reward);
        if self.reward_history.len() > 100 {
            self.reward_history.remove(0);
        }

        self.dopamine_level
    }

    pub fn simulate_parkinsons(&mut self, severity: f64) {
        // Reduce dopamine production
        self.baseline_dopamine *= 1.0 - severity;
        self.dopamine_level = self.baseline_dopamine;
    }
}

pub struct BasalGanglia {
    pub striatum: Striatum,
    pub gp: GlobalPallidus,
    pub stn: SubthalamicNucleus,
    pub snc: SubstantiaNigra,
}

impl BasalGanglia {
    pub fn new(num_striatal: usize, num_inputs: usize) -> Self {
        Self {
            striatum: Striatum::new(num_striatal, num_inputs),
            gp: GlobalPallidus::new(num_striatal / 2),
            stn: SubthalamicNucleus::new(num_striatal / 4),
            snc: SubstantiaNigra::new(100),
        }
    }

    pub fn step(&mut self, dt: f64, cortical_input: &[f64], reward: f64, expected_reward: f64, t: f64) -> Vec<f64> {
        let dopamine = self.snc.step(reward, expected_reward);
        let (d1_out, d2_out) = self.striatum.step(dt, cortical_input, dopamine, t);

        let conflict = if d1_out.iter().filter(|&&x| x).count() == d2_out.iter().filter(|&&x| x).count() { 1.0 } else { 0.0 };
        let stn_out = self.stn.step(&self.gp.gpe_activity, conflict);

        self.gp.step(&d1_out, &d2_out, &stn_out)
    }

    pub fn apply_learning(&mut self, reward: f64) {
        self.striatum.apply_reward(reward);
    }
}
