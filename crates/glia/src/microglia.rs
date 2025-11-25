//! Microglia models.
//!
//! Microglia are the brain's immune cells and also prune synapses.

use serde::{Deserialize, Serialize};

/// Microglia activation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicrogliaState {
    Resting,
    Activated,
    Phagocytic,
}

/// Microglia cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microglia {
    /// Microglia ID
    pub id: usize,

    /// Position
    pub position: [f64; 3],

    /// Activation state
    pub state: MicrogliaState,

    /// Activation level (0-1)
    pub activation_level: f64,

    /// Synaptic pruning rate
    pub pruning_rate: f64,

    /// Number of synapses monitored
    pub num_synapses_monitored: usize,
}

impl Microglia {
    pub fn new(id: usize, position: [f64; 3]) -> Self {
        Self {
            id,
            position,
            state: MicrogliaState::Resting,
            activation_level: 0.0,
            pruning_rate: 0.0,
            num_synapses_monitored: 0,
        }
    }

    /// Update microglia state
    pub fn step(&mut self, dt: f64, injury_signal: f64, synapse_activity: &[f64]) {
        // Activation by injury/inflammatory signals
        if injury_signal > 0.1 {
            self.activation_level += 0.1 * dt;
        } else {
            self.activation_level -= 0.01 * dt;
        }

        self.activation_level = self.activation_level.clamp(0.0, 1.0);

        // Update state based on activation
        self.state = if self.activation_level > 0.7 {
            MicrogliaState::Phagocytic
        } else if self.activation_level > 0.3 {
            MicrogliaState::Activated
        } else {
            MicrogliaState::Resting
        };

        // Synaptic pruning (removes weak/inactive synapses)
        if matches!(self.state, MicrogliaState::Resting) {
            self.pruning_rate = self.calculate_pruning_rate(synapse_activity);
        } else {
            self.pruning_rate = 0.0;
        }
    }

    /// Calculate which synapses to prune
    fn calculate_pruning_rate(&self, synapse_activity: &[f64]) -> f64 {
        // Prune synapses with very low activity
        let weak_synapses = synapse_activity
            .iter()
            .filter(|&&activity| activity < 0.1)
            .count();

        (weak_synapses as f64) * 0.001 // Slow pruning rate
    }

    /// Check if synapse should be pruned
    pub fn should_prune_synapse(&self, synapse_activity: f64) -> bool {
        matches!(self.state, MicrogliaState::Resting)
            && synapse_activity < 0.05
            && rand::random::<f64>() < 0.001
    }
}
