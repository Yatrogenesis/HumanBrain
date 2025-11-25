//! Oligodendrocyte models.
//!
//! Oligodendrocytes myelinate axons to increase conduction velocity.

use serde::{Deserialize, Serialize};

/// Oligodendrocyte cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oligodendrocyte {
    /// Oligodendrocyte ID
    pub id: usize,

    /// Position
    pub position: [f64; 3],

    /// Number of axons myelinated
    pub num_axons_myelinated: usize,

    /// Myelin thickness (um)
    pub myelin_thickness: f64,

    /// Internodal length (um)
    pub internodal_length: f64,

    /// Metabolic support to axons
    pub metabolic_support_rate: f64,
}

impl Oligodendrocyte {
    pub fn new(id: usize, position: [f64; 3]) -> Self {
        Self {
            id,
            position,
            num_axons_myelinated: 0,
            myelin_thickness: 0.5,      // Typical myelin thickness
            internodal_length: 100.0,   // Distance between nodes of Ranvier
            metabolic_support_rate: 0.01,
        }
    }

    /// Add myelination to an axon segment
    pub fn myelinate_axon(&mut self) {
        if self.num_axons_myelinated < 50 {
            // One oligodendrocyte can myelinate up to 50 axon segments
            self.num_axons_myelinated += 1;
        }
    }

    /// Calculate conduction velocity increase from myelination
    pub fn conduction_velocity_factor(&self, axon_diameter: f64) -> f64 {
        // Myelinated axons conduct 50-100x faster
        let base_velocity = axon_diameter * 0.5; // Unmyelinated (m/s)
        let myelinated_velocity = axon_diameter * 6.0; // Myelinated (m/s)
        myelinated_velocity / base_velocity
    }
}
