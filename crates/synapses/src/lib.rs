//! Advanced synaptic dynamics for realistic brain simulation.
//!
//! This crate implements various synaptic models including:
//! - Short-term plasticity (facilitation and depression)
//! - Long-term plasticity (LTP/LTD, STDP)
//! - Multiple neurotransmitter systems
//! - Neuromodulation

pub mod plasticity;
pub mod neurotransmitters;

use serde::{Deserialize, Serialize};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynapseError {
    #[error("Invalid synapse configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Plasticity error: {0}")]
    PlasticityError(String),
}

pub type Result<T> = std::result::Result<T, SynapseError>;

/// Type of synaptic connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynapseType {
    AMPA,      // Fast excitatory
    NMDA,      // Slow excitatory, voltage-dependent
    GABAA,     // Fast inhibitory
    GABAB,     // Slow inhibitory
    Glycine,   // Inhibitory
    Dopamine,  // Neuromodulatory
    Serotonin, // Neuromodulatory
}

/// Synapse model with realistic dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Synapse ID
    pub id: usize,

    /// Pre-synaptic neuron ID
    pub pre_neuron_id: usize,

    /// Post-synaptic neuron ID
    pub post_neuron_id: usize,

    /// Synapse type
    pub synapse_type: SynapseType,

    /// Synaptic weight (strength)
    pub weight: f64,

    /// Maximum synaptic conductance (nS)
    pub g_max: f64,

    /// Rise time constant (ms)
    pub tau_rise: f64,

    /// Decay time constant (ms)
    pub tau_decay: f64,

    /// Synaptic conductance state
    pub conductance: f64,

    /// Gating variable (0-1)
    pub gating: f64,

    /// Release probability
    pub release_probability: f64,

    /// Available neurotransmitter resources (0-1)
    pub resources: f64,

    /// Short-term facilitation
    pub facilitation: f64,

    /// Last pre-synaptic spike time (ms)
    pub last_pre_spike: f64,

    /// Last post-synaptic spike time (ms)
    pub last_post_spike: f64,

    /// Calcium concentration at synapse (mM)
    pub calcium: f64,
}

impl Synapse {
    /// Create a new synapse
    pub fn new(
        id: usize,
        pre_neuron_id: usize,
        post_neuron_id: usize,
        synapse_type: SynapseType,
        weight: f64,
    ) -> Self {
        let (tau_rise, tau_decay, g_max) = match synapse_type {
            SynapseType::AMPA => (0.2, 2.0, 1.0),
            SynapseType::NMDA => (2.0, 50.0, 0.5),
            SynapseType::GABAA => (0.5, 5.0, 1.5),
            SynapseType::GABAB => (50.0, 200.0, 0.8),
            SynapseType::Glycine => (0.5, 4.0, 1.2),
            SynapseType::Dopamine => (10.0, 100.0, 0.1),
            SynapseType::Serotonin => (20.0, 150.0, 0.1),
        };

        Self {
            id,
            pre_neuron_id,
            post_neuron_id,
            synapse_type,
            weight,
            g_max,
            tau_rise,
            tau_decay,
            conductance: 0.0,
            gating: 0.0,
            release_probability: 0.5,
            resources: 1.0,
            facilitation: 1.0,
            last_pre_spike: -1000.0,
            last_post_spike: -1000.0,
            calcium: 0.0,
        }
    }

    /// Update synapse state
    pub fn step(&mut self, dt: f64, pre_spike: bool, post_spike: bool, current_time: f64) {
        // Update gating variable (neurotransmitter in cleft)
        let decay_rate = 1.0 / self.tau_decay;
        self.gating *= (-decay_rate * dt).exp();

        // Handle pre-synaptic spike
        if pre_spike {
            self.handle_presynaptic_spike(current_time);
        }

        // Update resources (recovery from depletion)
        let recovery_rate = 0.01; // Recovery time constant
        self.resources += (1.0 - self.resources) * recovery_rate * dt;
        self.resources = self.resources.min(1.0);

        // Update facilitation (decay back to baseline)
        let fac_decay = 0.002;
        self.facilitation -= (self.facilitation - 1.0) * fac_decay * dt;

        // Update calcium concentration
        if pre_spike {
            self.calcium += 0.1; // Calcium influx during spike
        }
        let ca_decay = 0.01;
        self.calcium *= (-ca_decay * dt).exp();

        // Calculate conductance
        self.conductance = self.g_max * self.weight * self.gating * self.facilitation;

        // Store spike times for plasticity
        if pre_spike {
            self.last_pre_spike = current_time;
        }
        if post_spike {
            self.last_post_spike = current_time;
        }
    }

    /// Handle pre-synaptic spike event
    fn handle_presynaptic_spike(&mut self, _current_time: f64) {
        // Stochastic release
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() < self.release_probability * self.resources {
            // Neurotransmitter release
            self.gating += 0.5 * self.resources;
            self.gating = self.gating.min(1.0);

            // Resource depletion
            self.resources *= 0.5; // Use half of available resources

            // Facilitation increase
            self.facilitation += 0.1;
            self.facilitation = self.facilitation.min(3.0);
        }
    }

    /// Calculate synaptic current (pA)
    pub fn current(&self, v_post: f64) -> f64 {
        let e_rev = match self.synapse_type {
            SynapseType::AMPA | SynapseType::NMDA => 0.0,  // Excitatory
            SynapseType::GABAA | SynapseType::GABAB | SynapseType::Glycine => -70.0, // Inhibitory
            SynapseType::Dopamine | SynapseType::Serotonin => 0.0, // Modulatory
        };

        // NMDA voltage-dependent Mg2+ block
        let mg_block = if matches!(self.synapse_type, SynapseType::NMDA) {
            1.0 / (1.0 + 0.3 * (-0.062 * v_post).exp())
        } else {
            1.0
        };

        self.conductance * (v_post - e_rev) * mg_block
    }

    /// Apply spike-timing-dependent plasticity (STDP)
    pub fn apply_stdp(&mut self, dt_spike: f64) {
        // dt_spike = t_post - t_pre
        let a_plus = 0.01;  // LTP magnitude
        let a_minus = 0.012; // LTD magnitude
        let tau_plus = 20.0; // LTP time window (ms)
        let tau_minus = 20.0; // LTD time window (ms)

        if dt_spike > 0.0 {
            // Post after pre - LTP
            let dw = a_plus * (-dt_spike / tau_plus).exp();
            self.weight += dw * (1.0 - self.weight); // Soft bound at 1.0
        } else {
            // Pre after post - LTD
            let dw = -a_minus * (dt_spike / tau_minus).exp();
            self.weight += dw * self.weight; // Soft bound at 0.0
        }

        // Clamp weights
        self.weight = self.weight.clamp(0.0, 2.0);
    }

    /// Apply homeostatic plasticity
    pub fn apply_homeostatic_plasticity(&mut self, target_rate: f64, actual_rate: f64, dt: f64) {
        let scaling_rate = 0.0001; // Very slow
        let error = target_rate - actual_rate;
        self.weight += scaling_rate * error * dt;
        self.weight = self.weight.clamp(0.0, 2.0);
    }
}

/// Synaptic connection matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapticNetwork {
    /// All synapses
    pub synapses: Vec<Synapse>,

    /// Connection matrix: pre_neuron -> list of synapse indices
    pub pre_to_synapses: Vec<Vec<usize>>,

    /// Connection matrix: post_neuron -> list of synapse indices
    pub post_to_synapses: Vec<Vec<usize>>,
}

impl SynapticNetwork {
    /// Create a new synaptic network
    pub fn new(num_neurons: usize) -> Self {
        Self {
            synapses: Vec::new(),
            pre_to_synapses: vec![Vec::new(); num_neurons],
            post_to_synapses: vec![Vec::new(); num_neurons],
        }
    }

    /// Add a synapse to the network
    pub fn add_synapse(&mut self, synapse: Synapse) {
        let syn_idx = self.synapses.len();
        let pre_id = synapse.pre_neuron_id;
        let post_id = synapse.post_neuron_id;

        self.synapses.push(synapse);
        self.pre_to_synapses[pre_id].push(syn_idx);
        self.post_to_synapses[post_id].push(syn_idx);
    }

    /// Get all synapses from a pre-synaptic neuron
    pub fn get_outgoing_synapses(&self, pre_neuron_id: usize) -> Vec<&Synapse> {
        self.pre_to_synapses[pre_neuron_id]
            .iter()
            .map(|&idx| &self.synapses[idx])
            .collect()
    }

    /// Get all synapses to a post-synaptic neuron
    pub fn get_incoming_synapses(&self, post_neuron_id: usize) -> Vec<&Synapse> {
        self.post_to_synapses[post_neuron_id]
            .iter()
            .map(|&idx| &self.synapses[idx])
            .collect()
    }

    /// Update all synapses
    pub fn step(&mut self, dt: f64, spikes: &[bool], current_time: f64) {
        for synapse in &mut self.synapses {
            let pre_spike = spikes[synapse.pre_neuron_id];
            let post_spike = spikes[synapse.post_neuron_id];
            synapse.step(dt, pre_spike, post_spike, current_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synapse_creation() {
        let syn = Synapse::new(0, 0, 1, SynapseType::AMPA, 1.0);
        assert_eq!(syn.pre_neuron_id, 0);
        assert_eq!(syn.post_neuron_id, 1);
        assert_eq!(syn.synapse_type, SynapseType::AMPA);
    }

    #[test]
    fn test_synapse_dynamics() {
        let mut syn = Synapse::new(0, 0, 1, SynapseType::AMPA, 1.0);
        syn.release_probability = 1.0; // Deterministic for testing

        // Simulate pre-synaptic spike
        syn.step(0.1, true, false, 0.0);
        assert!(syn.gating > 0.0);

        // Decay
        for _ in 0..100 {
            syn.step(0.1, false, false, 1.0);
        }
        assert!(syn.gating < 0.1);
    }

    #[test]
    fn test_stdp() {
        let mut syn = Synapse::new(0, 0, 1, SynapseType::AMPA, 0.5);
        let initial_weight = syn.weight;

        // Pre before post - LTP
        syn.apply_stdp(10.0);
        assert!(syn.weight > initial_weight);

        // Post before pre - LTD
        syn.weight = 0.5;
        syn.apply_stdp(-10.0);
        assert!(syn.weight < 0.5);
    }

    #[test]
    fn test_network() {
        let mut network = SynapticNetwork::new(10);

        let syn = Synapse::new(0, 0, 1, SynapseType::AMPA, 1.0);
        network.add_synapse(syn);

        assert_eq!(network.synapses.len(), 1);
        assert_eq!(network.get_outgoing_synapses(0).len(), 1);
        assert_eq!(network.get_incoming_synapses(1).len(), 1);
    }
}
