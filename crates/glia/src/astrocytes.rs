//! Astrocyte models.
//!
//! Astrocytes perform multiple critical functions:
//! - Clear glutamate from synaptic cleft
//! - Buffer extracellular K+ to prevent hyperexcitability
//! - Regulate blood flow (neurovascular coupling)
//! - Provide metabolic support (lactate shuttle)

use serde::{Deserialize, Serialize};

/// Astrocyte cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Astrocyte {
    /// Astrocyte ID
    pub id: usize,

    /// Position in tissue
    pub position: [f64; 3],

    /// Intracellular calcium concentration (mM)
    pub calcium: f64,

    /// IP3 concentration (mM)
    pub ip3: f64,

    /// Glutamate uptake rate (mM/ms)
    pub glutamate_uptake_rate: f64,

    /// Extracellular K+ concentration in domain (mM)
    pub extracellular_k: f64,

    /// K+ buffering capacity
    pub k_buffering_capacity: f64,

    /// Glycogen stores (mM)
    pub glycogen: f64,

    /// Lactate release rate (mM/ms)
    pub lactate_release_rate: f64,

    /// Number of synapses in astrocyte domain
    pub num_synapses_covered: usize,
}

impl Astrocyte {
    pub fn new(id: usize, position: [f64; 3]) -> Self {
        Self {
            id,
            position,
            calcium: 0.0001,           // Resting Ca2+
            ip3: 0.0001,               // Resting IP3
            glutamate_uptake_rate: 0.0,
            extracellular_k: 3.5,      // Normal extracellular K+ (mM)
            k_buffering_capacity: 100.0,
            glycogen: 10.0,            // Glycogen stores
            lactate_release_rate: 0.0,
            num_synapses_covered: 0,
        }
    }

    /// Update astrocyte state
    pub fn step(&mut self, dt: f64, synaptic_activity: f64) {
        // Glutamate uptake (proportional to synaptic activity)
        self.glutamate_uptake_rate = 0.1 * synaptic_activity;

        // Glutamate triggers calcium signaling via mGluR
        let ca_influx = self.glutamate_uptake_rate * 0.01;
        self.calcium += ca_influx * dt;

        // Calcium decay
        let ca_decay_rate = 0.05;
        self.calcium *= (-ca_decay_rate * dt).exp();
        self.calcium = self.calcium.max(0.0001); // Maintain basal level

        // IP3 dynamics (simplified)
        self.ip3 += ca_influx * 0.5 * dt;
        self.ip3 *= (-0.1 * dt).exp();

        // K+ buffering
        // Neural activity increases extracellular K+
        let k_influx = synaptic_activity * 0.01;
        self.extracellular_k += k_influx * dt;

        // Astrocyte buffers excess K+
        let k_buffer_rate = (self.extracellular_k - 3.5) * 0.1; // Return to baseline
        self.extracellular_k -= k_buffer_rate * dt;
        self.extracellular_k = self.extracellular_k.clamp(3.0, 10.0);

        // Lactate shuttle (astrocyte-neuron lactate shuttle)
        // Astrocytes produce lactate from glycogen during high activity
        if synaptic_activity > 0.5 {
            let glycogen_consumption = 0.01 * synaptic_activity * dt;
            self.glycogen -= glycogen_consumption;
            self.glycogen = self.glycogen.max(0.0);

            self.lactate_release_rate = glycogen_consumption * 2.0; // 1 glucose -> 2 lactate
        } else {
            // Replenish glycogen during low activity
            self.glycogen += 0.001 * dt;
            self.glycogen = self.glycogen.min(15.0);
            self.lactate_release_rate = 0.0;
        }
    }

    /// Get calcium wave propagation signal
    pub fn calcium_wave_signal(&self) -> f64 {
        // High calcium can trigger waves to neighboring astrocytes
        if self.calcium > 0.001 {
            self.calcium * 10.0
        } else {
            0.0
        }
    }

    /// Influence on blood flow (neurovascular coupling)
    pub fn blood_flow_signal(&self) -> f64 {
        // High calcium causes release of vasoactive substances
        self.calcium * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astrocyte_creation() {
        let astro = Astrocyte::new(0, [0.0, 0.0, 0.0]);
        assert_eq!(astro.id, 0);
        assert!(astro.calcium > 0.0);
    }

    #[test]
    fn test_glutamate_uptake() {
        let mut astro = Astrocyte::new(0, [0.0, 0.0, 0.0]);

        // High synaptic activity
        astro.step(0.1, 1.0);
        assert!(astro.glutamate_uptake_rate > 0.0);
        assert!(astro.calcium > 0.0001); // Should increase from baseline
    }

    #[test]
    fn test_k_buffering() {
        let mut astro = Astrocyte::new(0, [0.0, 0.0, 0.0]);
        let initial_k = astro.extracellular_k;

        // Simulate activity that increases K+
        for _ in 0..10 {
            astro.step(0.1, 1.0);
        }

        // K+ should increase then be buffered back toward baseline
        for _ in 0..100 {
            astro.step(0.1, 0.0);
        }

        assert!((astro.extracellular_k - initial_k).abs() < 1.0);
    }
}
