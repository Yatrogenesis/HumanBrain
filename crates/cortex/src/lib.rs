//! Neocortex simulation with 6-layer columnar organization.
//!
//! The neocortex is organized into 6 layers with distinct cell types and connectivity:
//! - Layer 1: Sparse neurons, mostly dendrites and axons
//! - Layer 2/3: Pyramidal neurons, cortico-cortical connections
//! - Layer 4: Granular layer, receives thalamic input
//! - Layer 5: Large pyramidal neurons, output to subcortical structures
//! - Layer 6: Corticothalamic neurons, feedback to thalamus

pub mod column;
pub mod layers;

use ndarray::Array2;
use neurons::MultiCompartmentalNeuron;
use synapses::{Synapse, SynapticNetwork, SynapseType};
use glia::{Astrocyte, Oligodendrocyte, Microglia};
use metabolism::RegionalMetabolism;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CortexError {
    #[error("Invalid cortical configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Layer error: {0}")]
    LayerError(String),
}

pub type Result<T> = std::result::Result<T, CortexError>;

pub use column::CorticalColumn;
pub use layers::{CorticalLayer, LayerType};

/// Cortical neuron types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalNeuronType {
    PyramidalL2_3,
    PyramidalL5,
    PyramidalL6,
    ParvalbuminInterneuron,  // Fast-spiking
    SomatostatinInterneuron, // Regular-spiking
    VIPInterneuron,          // Irregular-spiking
    SpinyStellate,           // L4 excitatory
}

/// Complete neocortex model
#[derive(Debug, Clone)]
pub struct Neocortex {
    /// Cortical columns
    pub columns: Vec<CorticalColumn>,

    /// Long-range connections between columns
    pub long_range_connections: Vec<(usize, usize, f64)>, // (source_col, target_col, weight)

    /// Total number of neurons
    pub total_neurons: usize,

    /// Current simulation time (ms)
    pub time: f64,

    /// Time step (ms)
    pub dt: f64,
}

impl Neocortex {
    /// Create a neocortex with specified number of columns
    pub fn new(num_columns: usize, neurons_per_column: usize, dt: f64) -> Self {
        let mut columns = Vec::with_capacity(num_columns);

        for i in 0..num_columns {
            columns.push(CorticalColumn::new(i, neurons_per_column, dt));
        }

        let total_neurons = num_columns * neurons_per_column;

        Self {
            columns,
            long_range_connections: Vec::new(),
            total_neurons,
            time: 0.0,
            dt,
        }
    }

    /// Add long-range connection between columns
    pub fn connect_columns(&mut self, source: usize, target: usize, weight: f64) {
        if source < self.columns.len() && target < self.columns.len() {
            self.long_range_connections.push((source, target, weight));
        }
    }

    /// Step the simulation forward
    pub fn step(&mut self, external_input: &Array2<f64>) -> Result<()> {
        // Update each column in parallel
        use rayon::prelude::*;

        self.columns.par_iter_mut().enumerate().for_each(|(i, column)| {
            // Extract input for this column
            let col_input = if i < external_input.ncols() {
                external_input.column(i).to_owned()
            } else {
                ndarray::Array1::zeros(column.neurons.len())
            };

            column.step(col_input.as_slice().unwrap()).ok();
        });

        // Process long-range connections
        // This would pass activity from source columns to target columns

        self.time += self.dt;
        Ok(())
    }

    /// Get total number of spikes across cortex
    pub fn total_spikes(&self) -> usize {
        self.columns
            .iter()
            .map(|col| col.get_spike_count())
            .sum()
    }

    /// Get average firing rate
    pub fn average_firing_rate(&self) -> f64 {
        if self.time > 0.0 {
            (self.total_spikes() as f64) / (self.total_neurons as f64 * self.time) * 1000.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neocortex_creation() {
        let cortex = Neocortex::new(10, 100, 0.1);
        assert_eq!(cortex.columns.len(), 10);
        assert_eq!(cortex.total_neurons, 1000);
    }

    #[test]
    fn test_cortex_simulation() {
        let mut cortex = Neocortex::new(5, 50, 0.1);
        let input = Array2::zeros((50, 5));

        cortex.step(&input).unwrap();
        assert!(cortex.time > 0.0);
    }
}
pub mod oscillations;
pub use oscillations::{OscillationBand, BrainOscillations};
