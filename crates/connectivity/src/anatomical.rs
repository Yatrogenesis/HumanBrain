//! Anatomically realistic cortical connectivity.
//!
//! Based on experimental data from:
//! - Binzegger et al. (2004): Layer-specific connectivity
//! - Markram et al. (2015): Blue Brain Project
//! - Thomson & Bannister (2003): Neocortical circuitry

use ndarray::Array2;
use serde::{Deserialize, Serialize};

/// Cortical layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorticalLayer {
    Layer1,
    Layer23,
    Layer4,
    Layer5,
    Layer6,
}

/// Connection probability based on distance and layers
pub struct AnatomicalConnectivity {
    /// Layer-to-layer connection matrix (probabilities)
    pub layer_matrix: Array2<f64>,

    /// Horizontal connection decay length (µm)
    pub horizontal_lambda: f64,

    /// Vertical connection decay length (µm)
    pub vertical_lambda: f64,
}

impl AnatomicalConnectivity {
    /// Create realistic cortical connectivity matrix
    pub fn new_cortical() -> Self {
        // Connection probabilities from Binzegger et al. 2004
        // Rows: source layer, Cols: target layer
        // Order: L1, L2/3, L4, L5, L6

        #[rustfmt::skip]
        let layer_matrix = Array2::from_shape_vec(
            (5, 5),
            vec![
                // To:  L1    L2/3   L4     L5     L6
                0.05,  0.10,  0.05,  0.05,  0.05,  // From L1
                0.15,  0.25,  0.10,  0.15,  0.10,  // From L2/3
                0.05,  0.30,  0.20,  0.15,  0.10,  // From L4
                0.10,  0.20,  0.10,  0.20,  0.15,  // From L5
                0.05,  0.15,  0.15,  0.15,  0.20,  // From L6
            ],
        ).unwrap();

        Self {
            layer_matrix,
            horizontal_lambda: 100.0,  // µm (exponential decay)
            vertical_lambda: 300.0,     // µm (larger for vertical)
        }
    }

    /// Calculate connection probability between two neurons
    ///
    /// P(connection) = P_layer × P_distance
    ///
    /// where:
    /// - P_layer: base probability from layer matrix
    /// - P_distance: exp(-d_horizontal/λ_h) × exp(-d_vertical/λ_v)
    pub fn connection_probability(
        &self,
        source_layer: CorticalLayer,
        target_layer: CorticalLayer,
        horizontal_distance: f64,  // µm
        vertical_distance: f64,    // µm
    ) -> f64 {
        let source_idx = Self::layer_to_index(source_layer);
        let target_idx = Self::layer_to_index(target_layer);

        let p_layer = self.layer_matrix[[source_idx, target_idx]];

        let p_horizontal = (-horizontal_distance / self.horizontal_lambda).exp();
        let p_vertical = (-vertical_distance / self.vertical_lambda).exp();

        p_layer * p_horizontal * p_vertical
    }

    /// Calculate axonal delay based on distance
    ///
    /// delay = distance / conduction_velocity
    ///
    /// Conduction velocities (Swadlow et al. 1978):
    /// - Unmyelinated: 0.5 m/s
    /// - Myelinated: 3-5 m/s
    /// Using average 2 m/s = 2000 µm/ms
    pub fn axonal_delay(&self, distance_um: f64) -> f64 {
        const CONDUCTION_VELOCITY: f64 = 2000.0; // µm/ms
        distance_um / CONDUCTION_VELOCITY
    }

    fn layer_to_index(layer: CorticalLayer) -> usize {
        match layer {
            CorticalLayer::Layer1 => 0,
            CorticalLayer::Layer23 => 1,
            CorticalLayer::Layer4 => 2,
            CorticalLayer::Layer5 => 3,
            CorticalLayer::Layer6 => 4,
        }
    }
}

/// Gap junction connectivity (electrical synapses)
///
/// Found primarily between:
/// - Parvalbumin+ interneurons (10% within 100µm)
/// - Layer 6 pyramidal cells (5% within 50µm)
pub struct GapJunctionConnectivity {
    /// Connection probability at zero distance
    pub p_zero: f64,

    /// Decay length (µm)
    pub lambda: f64,
}

impl GapJunctionConnectivity {
    /// Create gap junction connectivity for PV+ interneurons
    pub fn parvalbumin_interneurons() -> Self {
        Self {
            p_zero: 0.10,  // 10% probability at zero distance
            lambda: 100.0,  // Drops to ~37% at 100 µm
        }
    }

    /// Calculate gap junction probability
    pub fn probability(&self, distance_um: f64) -> f64 {
        self.p_zero * (-distance_um / self.lambda).exp()
    }

    /// Gap junction conductance (nS)
    ///
    /// Typical: 0.1-1 nS (Bennett & Zukin 2004)
    pub fn conductance(&self) -> f64 {
        0.5  // nS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_connectivity() {
        let conn = AnatomicalConnectivity::new_cortical();

        // Test L4 → L2/3 (strong feedforward)
        let p = conn.connection_probability(
            CorticalLayer::Layer4,
            CorticalLayer::Layer23,
            0.0,  // same column
            0.0,
        );
        assert!(p > 0.25); // Should be ~0.30

        // Test distance decay
        let p_near = conn.connection_probability(
            CorticalLayer::Layer23,
            CorticalLayer::Layer23,
            10.0,
            0.0,
        );
        let p_far = conn.connection_probability(
            CorticalLayer::Layer23,
            CorticalLayer::Layer23,
            200.0,
            0.0,
        );
        assert!(p_near > p_far);
    }

    #[test]
    fn test_axonal_delay() {
        let conn = AnatomicalConnectivity::new_cortical();

        let delay_100um = conn.axonal_delay(100.0);
        assert!((delay_100um - 0.05).abs() < 0.01); // ~0.05 ms

        let delay_1mm = conn.axonal_delay(1000.0);
        assert!((delay_1mm - 0.5).abs() < 0.05); // ~0.5 ms
    }

    #[test]
    fn test_gap_junctions() {
        let gap = GapJunctionConnectivity::parvalbumin_interneurons();

        let p_near = gap.probability(50.0);
        let p_far = gap.probability(200.0);

        assert!(p_near > p_far);
        assert!(p_near < 0.10); // Less than max
    }
}
