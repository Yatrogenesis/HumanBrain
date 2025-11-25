//! Cortical layer definitions and properties.

use serde::{Deserialize, Serialize};

/// Cortical layer types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Layer1,    // Molecular layer (sparse neurons, mostly processes)
    Layer2_3,  // External granular and pyramidal layers (cortico-cortical)
    Layer4,    // Internal granular layer (thalamic input)
    Layer5,    // Internal pyramidal layer (subcortical output)
    Layer6,    // Multiform layer (cortico-thalamic feedback)
}

/// Properties of each cortical layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorticalLayer {
    pub layer_type: LayerType,
    pub thickness: f64,           // um
    pub neuron_density: f64,      // neurons per mm^3
    pub excitatory_ratio: f64,    // proportion of excitatory neurons
}

impl CorticalLayer {
    /// Get layer properties
    pub fn properties(layer_type: LayerType) -> Self {
        match layer_type {
            LayerType::Layer1 => Self {
                layer_type,
                thickness: 165.0,
                neuron_density: 5000.0,
                excitatory_ratio: 0.95,
            },
            LayerType::Layer2_3 => Self {
                layer_type,
                thickness: 450.0,
                neuron_density: 25000.0,
                excitatory_ratio: 0.80,
            },
            LayerType::Layer4 => Self {
                layer_type,
                thickness: 200.0,
                neuron_density: 40000.0, // High density granule layer
                excitatory_ratio: 0.85,
            },
            LayerType::Layer5 => Self {
                layer_type,
                thickness: 550.0,
                neuron_density: 20000.0,
                excitatory_ratio: 0.80,
            },
            LayerType::Layer6 => Self {
                layer_type,
                thickness: 650.0,
                neuron_density: 22000.0,
                excitatory_ratio: 0.80,
            },
        }
    }

    /// Calculate number of neurons in this layer for a given surface area
    pub fn neuron_count(&self, surface_area_mm2: f64) -> usize {
        let volume_mm3 = surface_area_mm2 * (self.thickness / 1000.0);
        (volume_mm3 * self.neuron_density) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_properties() {
        let layer4 = CorticalLayer::properties(LayerType::Layer4);
        assert_eq!(layer4.layer_type, LayerType::Layer4);
        assert!(layer4.thickness > 0.0);
        assert!(layer4.neuron_density > 0.0);
    }

    #[test]
    fn test_neuron_count() {
        let layer = CorticalLayer::properties(LayerType::Layer4);
        let count = layer.neuron_count(1.0); // 1 mm^2
        assert!(count > 0);
    }
}
