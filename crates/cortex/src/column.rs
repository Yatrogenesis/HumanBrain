//! Cortical column implementation.
//!
//! A cortical column is the fundamental computational unit of the neocortex,
//! containing ~100,000 neurons arranged in 6 layers.

use crate::{CorticalNeuronType, layers::*, Result};
use neurons::{MultiCompartmentalNeuron, compartmental::ChannelStates};
use synapses::{Synapse, SynapticNetwork, SynapseType};
use glia::{Astrocyte, Oligodendrocyte, Microglia};
use metabolism::RegionalMetabolism;
use serde::{Deserialize, Serialize};
use rand::Rng;

/// A single cortical column
#[derive(Debug, Clone)]
pub struct CorticalColumn {
    /// Column ID
    pub id: usize,

    /// All neurons in the column
    pub neurons: Vec<MultiCompartmentalNeuron>,

    /// Channel states for all neurons
    pub channel_states: Vec<Vec<ChannelStates>>,

    /// Neuron types
    pub neuron_types: Vec<CorticalNeuronType>,

    /// Layer assignments
    pub neuron_layers: Vec<LayerType>,

    /// Synaptic network
    pub synaptic_network: SynapticNetwork,

    /// Astrocytes
    pub astrocytes: Vec<Astrocyte>,

    /// Oligodendrocytes
    pub oligodendrocytes: Vec<Oligodendrocyte>,

    /// Microglia
    pub microglia: Vec<Microglia>,

    /// Metabolic state
    pub metabolism: RegionalMetabolism,

    /// Spike history (for analysis)
    pub spike_count: usize,

    /// Time step (ms)
    pub dt: f64,

    /// Current time (ms)
    pub time: f64,
}

impl CorticalColumn {
    /// Create a new cortical column
    pub fn new(id: usize, num_neurons: usize, dt: f64) -> Self {
        let mut rng = rand::thread_rng();

        // Create neurons distributed across layers
        let mut neurons = Vec::with_capacity(num_neurons);
        let mut channel_states = Vec::with_capacity(num_neurons);
        let mut neuron_types = Vec::with_capacity(num_neurons);
        let mut neuron_layers = Vec::with_capacity(num_neurons);

        // Layer distribution (approximating cortical proportions)
        let layer_proportions = [
            (LayerType::Layer1, 0.05),
            (LayerType::Layer2_3, 0.25),
            (LayerType::Layer4, 0.20),
            (LayerType::Layer5, 0.25),
            (LayerType::Layer6, 0.25),
        ];

        let mut neuron_id = 0;
        for (layer, proportion) in layer_proportions {
            let layer_size = (num_neurons as f64 * proportion) as usize;

            for _ in 0..layer_size {
                // 80% excitatory, 20% inhibitory
                let is_excitatory = rng.gen::<f64>() < 0.8;

                let neuron_type = if is_excitatory {
                    match layer {
                        LayerType::Layer2_3 => CorticalNeuronType::PyramidalL2_3,
                        LayerType::Layer4 => CorticalNeuronType::SpinyStellate,
                        LayerType::Layer5 => CorticalNeuronType::PyramidalL5,
                        LayerType::Layer6 => CorticalNeuronType::PyramidalL6,
                        _ => CorticalNeuronType::PyramidalL2_3,
                    }
                } else {
                    // Randomly assign interneuron type
                    match rng.gen_range(0..3) {
                        0 => CorticalNeuronType::ParvalbuminInterneuron,
                        1 => CorticalNeuronType::SomatostatinInterneuron,
                        _ => CorticalNeuronType::VIPInterneuron,
                    }
                };

                // Create neuron based on type
                let neuron = match neuron_type {
                    CorticalNeuronType::PyramidalL2_3
                    | CorticalNeuronType::PyramidalL5
                    | CorticalNeuronType::PyramidalL6 => {
                        MultiCompartmentalNeuron::new_pyramidal(neuron_id, dt)
                    }
                    _ => MultiCompartmentalNeuron::new(neuron_id, 20, dt),
                };

                // Initialize channel states for all compartments
                let num_compartments = neuron.compartments.len();
                let states = vec![ChannelStates::default(); num_compartments];

                neurons.push(neuron);
                channel_states.push(states);
                neuron_types.push(neuron_type);
                neuron_layers.push(layer);

                neuron_id += 1;
            }
        }

        // Create synaptic network
        let mut synaptic_network = SynapticNetwork::new(neurons.len());

        // Add intra-columnar connections
        Self::create_columnar_connections(
            &mut synaptic_network,
            &neuron_types,
            &neuron_layers,
            &mut rng,
        );

        // Create glial cells (approximately 1:1 ratio with neurons)
        let num_glial = num_neurons;
        let astrocytes = (0..num_glial / 3)
            .map(|i| Astrocyte::new(i, [rng.gen(), rng.gen(), rng.gen()]))
            .collect();

        let oligodendrocytes = (0..num_glial / 3)
            .map(|i| Oligodendrocyte::new(i, [rng.gen(), rng.gen(), rng.gen()]))
            .collect();

        let microglia = (0..num_glial / 3)
            .map(|i| Microglia::new(i, [rng.gen(), rng.gen(), rng.gen()]))
            .collect();

        // Initialize metabolism
        let metabolism = RegionalMetabolism::new(neurons.len());

        Self {
            id,
            neurons,
            channel_states,
            neuron_types,
            neuron_layers,
            synaptic_network,
            astrocytes,
            oligodendrocytes,
            microglia,
            metabolism,
            spike_count: 0,
            dt,
            time: 0.0,
        }
    }

    /// Create layer-specific connections within the column
    fn create_columnar_connections(
        network: &mut SynapticNetwork,
        neuron_types: &[CorticalNeuronType],
        neuron_layers: &[LayerType],
        rng: &mut impl Rng,
    ) {
        let num_neurons = neuron_types.len();

        for pre_id in 0..num_neurons {
            let pre_type = neuron_types[pre_id];
            let pre_layer = neuron_layers[pre_id];

            // Determine connection probability and targets based on layer
            let targets = Self::get_connection_targets(pre_layer, neuron_layers);

            for &post_id in &targets {
                if pre_id == post_id {
                    continue;
                }

                // Connection probability
                let prob = Self::connection_probability(pre_type, neuron_types[post_id]);

                if rng.gen::<f64>() < prob {
                    // Determine synapse type
                    let syn_type = if Self::is_excitatory(pre_type) {
                        if rng.gen::<f64>() < 0.8 {
                            SynapseType::AMPA
                        } else {
                            SynapseType::NMDA
                        }
                    } else {
                        SynapseType::GABAA
                    };

                    // Random initial weight
                    let weight = rng.gen_range(0.5..1.5);

                    let synapse = Synapse::new(
                        network.synapses.len(),
                        pre_id,
                        post_id,
                        syn_type,
                        weight,
                    );

                    network.add_synapse(synapse);
                }
            }
        }
    }

    /// Get potential connection targets based on source layer
    fn get_connection_targets(source_layer: LayerType, all_layers: &[LayerType]) -> Vec<usize> {
        all_layers
            .iter()
            .enumerate()
            .filter(|(_, &layer)| Self::layers_connect(source_layer, layer))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Determine if two layers have connections (simplified connectivity)
    fn layers_connect(source: LayerType, target: LayerType) -> bool {
        match (source, target) {
            // L2/3 -> L5
            (LayerType::Layer2_3, LayerType::Layer5) => true,
            // L4 -> L2/3
            (LayerType::Layer4, LayerType::Layer2_3) => true,
            // L5 -> L6
            (LayerType::Layer5, LayerType::Layer6) => true,
            // L6 -> L4
            (LayerType::Layer6, LayerType::Layer4) => true,
            // Within-layer connections
            _ if source == target => true,
            _ => false,
        }
    }

    /// Connection probability between neuron types
    fn connection_probability(pre: CorticalNeuronType, post: CorticalNeuronType) -> f64 {
        // Simplified connection probabilities
        match (Self::is_excitatory(pre), Self::is_excitatory(post)) {
            (true, true) => 0.1,   // E->E
            (true, false) => 0.2,  // E->I
            (false, true) => 0.3,  // I->E
            (false, false) => 0.1, // I->I
        }
    }

    /// Check if neuron type is excitatory
    fn is_excitatory(neuron_type: CorticalNeuronType) -> bool {
        matches!(
            neuron_type,
            CorticalNeuronType::PyramidalL2_3
                | CorticalNeuronType::PyramidalL5
                | CorticalNeuronType::PyramidalL6
                | CorticalNeuronType::SpinyStellate
        )
    }

    /// Step the column simulation
    pub fn step(&mut self, external_input: &[f64]) -> Result<()> {
        // Apply external input
        for (i, &input) in external_input.iter().enumerate() {
            if i < self.neurons.len() {
                self.neurons[i].inject_current(0, input);
            }
        }

        // Update neurons
        let mut spikes = vec![false; self.neurons.len()];
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            neuron.step(&mut self.channel_states[i]);
            spikes[i] = neuron.is_spiking;

            if spikes[i] {
                self.spike_count += 1;
            }
        }

        // Update synapses
        self.synaptic_network.step(self.dt, &spikes, self.time);

        // Calculate synaptic currents and inject into neurons
        for (post_id, neuron) in self.neurons.iter_mut().enumerate() {
            let incoming = self.synaptic_network.get_incoming_synapses(post_id);
            let total_current: f64 = incoming
                .iter()
                .map(|syn| syn.current(neuron.get_soma_voltage()))
                .sum();

            neuron.synaptic_current[0] = total_current;
        }

        // Update glial cells
        let avg_activity = spikes.iter().filter(|&&s| s).count() as f64 / self.neurons.len() as f64;

        for astro in &mut self.astrocytes {
            astro.step(self.dt, avg_activity);
        }

        for microglia in &mut self.microglia {
            microglia.step(self.dt, 0.0, &vec![avg_activity; 100]);
        }

        // Update metabolism
        let synaptic_events: Vec<usize> = (0..self.neurons.len())
            .map(|i| self.synaptic_network.get_incoming_synapses(i).len())
            .collect();

        self.metabolism.step(self.dt, &spikes, &synaptic_events).ok();

        self.time += self.dt;
        Ok(())
    }

    /// Get spike count
    pub fn get_spike_count(&self) -> usize {
        self.spike_count
    }

    /// Get average voltage
    pub fn get_average_voltage(&self) -> f64 {
        self.neurons
            .iter()
            .map(|n| n.get_soma_voltage())
            .sum::<f64>()
            / self.neurons.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let column = CorticalColumn::new(0, 1000, 0.1);
        assert_eq!(column.neurons.len(), 1000);
        assert!(column.synaptic_network.synapses.len() > 0);
    }

    #[test]
    fn test_column_simulation() {
        let mut column = CorticalColumn::new(0, 100, 0.1);
        let input = vec![0.0; 100];

        column.step(&input).unwrap();
        assert!(column.time > 0.0);
    }
}
