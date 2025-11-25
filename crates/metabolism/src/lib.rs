//! Metabolic constraints and energy dynamics.
//!
//! This crate models ATP production, glucose metabolism, and oxygen consumption
//! to impose realistic energy budgets on neural activity.

use ndarray::Array1;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetabolismError {
    #[error("Energy constraint violated: {0}")]
    EnergyConstraint(String),

    #[error("Invalid metabolic state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, MetabolismError>;

/// Metabolic state for a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronMetabolism {
    /// ATP concentration (mM)
    pub atp: f64,

    /// ADP concentration (mM)
    pub adp: f64,

    /// Glucose concentration (mM)
    pub glucose: f64,

    /// Oxygen concentration (mM)
    pub oxygen: f64,

    /// Lactate concentration (mM)
    pub lactate: f64,

    /// ATP production rate (mM/ms)
    pub atp_production_rate: f64,

    /// ATP consumption rate (mM/ms)
    pub atp_consumption_rate: f64,

    /// Baseline ATP cost (mM/ms)
    pub baseline_cost: f64,

    /// ATP cost per spike
    pub spike_cost: f64,

    /// ATP cost per synaptic event
    pub synaptic_cost: f64,
}

impl NeuronMetabolism {
    /// Create new metabolism instance
    pub fn new() -> Self {
        Self {
            atp: 2.0,              // Typical intracellular ATP (mM)
            adp: 0.1,              // Typical ADP
            glucose: 5.0,          // Blood glucose level
            oxygen: 0.1,           // Dissolved O2
            lactate: 1.0,          // Lactate level
            atp_production_rate: 0.0,
            atp_consumption_rate: 0.0,
            baseline_cost: 0.001,  // Baseline maintenance
            spike_cost: 0.01,      // ATP per action potential
            synaptic_cost: 0.001,  // ATP per synaptic event
        }
    }

    /// Update metabolic state
    pub fn step(&mut self, dt: f64, spiking: bool, num_synaptic_events: usize) -> Result<()> {
        // Calculate ATP consumption
        self.atp_consumption_rate = self.baseline_cost;

        if spiking {
            self.atp_consumption_rate += self.spike_cost;
        }

        self.atp_consumption_rate += (num_synaptic_events as f64) * self.synaptic_cost;

        // Calculate ATP production (oxidative phosphorylation)
        // Simplified: depends on glucose and oxygen availability
        let oxidative_rate = self.calculate_oxidative_phosphorylation();

        // Glycolysis (anaerobic)
        let glycolytic_rate = self.calculate_glycolysis();

        self.atp_production_rate = oxidative_rate + glycolytic_rate;

        // Update ATP
        let delta_atp = (self.atp_production_rate - self.atp_consumption_rate) * dt;
        self.atp += delta_atp;

        // ATP cannot go negative or exceed capacity
        if self.atp < 0.1 {
            return Err(MetabolismError::EnergyConstraint(
                "ATP depleted - neuron cannot fire".to_string(),
            ));
        }

        self.atp = self.atp.min(3.0); // Maximum ATP concentration

        // Update glucose (consumed during glycolysis and oxidation)
        self.glucose -= 0.001 * dt;
        self.glucose = self.glucose.max(0.0);

        // Update oxygen (consumed during oxidation)
        self.oxygen -= oxidative_rate * 0.1 * dt;
        self.oxygen = self.oxygen.max(0.0);

        // Update lactate (produced during glycolysis)
        self.lactate += glycolytic_rate * 0.5 * dt;

        Ok(())
    }

    /// Calculate oxidative phosphorylation rate
    fn calculate_oxidative_phosphorylation(&self) -> f64 {
        // Requires both glucose and oxygen
        let glucose_factor = self.glucose / (self.glucose + 1.0); // Michaelis-Menten
        let oxygen_factor = self.oxygen / (self.oxygen + 0.01);

        0.1 * glucose_factor * oxygen_factor // Base rate
    }

    /// Calculate glycolysis rate
    fn calculate_glycolysis(&self) -> f64 {
        // Can occur without oxygen (anaerobic)
        let glucose_factor = self.glucose / (self.glucose + 1.0);
        0.05 * glucose_factor // Less efficient than oxidation
    }

    /// Check if neuron can afford to spike
    pub fn can_spike(&self) -> bool {
        self.atp > self.spike_cost
    }

    /// Check if synapse can release
    pub fn can_release(&self) -> bool {
        self.atp > self.synaptic_cost
    }

    /// Supply glucose and oxygen (from blood flow)
    pub fn supply_nutrients(&mut self, glucose: f64, oxygen: f64) {
        self.glucose += glucose;
        self.oxygen += oxygen;

        // Clamp to physiological values
        self.glucose = self.glucose.min(10.0);
        self.oxygen = self.oxygen.min(0.2);
    }
}

impl Default for NeuronMetabolism {
    fn default() -> Self {
        Self::new()
    }
}

/// Regional blood flow and nutrient delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodFlow {
    /// Regional cerebral blood flow (ml/100g/min)
    pub flow_rate: f64,

    /// Glucose delivery rate (mM/ms)
    pub glucose_delivery: f64,

    /// Oxygen delivery rate (mM/ms)
    pub oxygen_delivery: f64,

    /// Metabolic activity factor (scales with neural activity)
    pub activity_factor: f64,
}

impl BloodFlow {
    pub fn new() -> Self {
        Self {
            flow_rate: 50.0,        // Normal resting flow
            glucose_delivery: 0.01,
            oxygen_delivery: 0.001,
            activity_factor: 1.0,
        }
    }

    /// Update blood flow based on neural activity (neurovascular coupling)
    pub fn update_from_activity(&mut self, activity_level: f64) {
        // Activity increases blood flow with delay
        let target_factor = 1.0 + activity_level * 0.5;
        self.activity_factor += (target_factor - self.activity_factor) * 0.01;

        self.flow_rate = 50.0 * self.activity_factor;
        self.glucose_delivery = 0.01 * self.activity_factor;
        self.oxygen_delivery = 0.001 * self.activity_factor;
    }
}

impl Default for BloodFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Energy budget for a brain region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalMetabolism {
    /// Neurons in this region
    pub num_neurons: usize,

    /// Per-neuron metabolism
    pub neuron_metabolism: Vec<NeuronMetabolism>,

    /// Blood flow to this region
    pub blood_flow: BloodFlow,

    /// Total ATP consumption rate
    pub total_atp_consumption: f64,

    /// Total ATP production rate
    pub total_atp_production: f64,
}

impl RegionalMetabolism {
    pub fn new(num_neurons: usize) -> Self {
        Self {
            num_neurons,
            neuron_metabolism: vec![NeuronMetabolism::new(); num_neurons],
            blood_flow: BloodFlow::new(),
            total_atp_consumption: 0.0,
            total_atp_production: 0.0,
        }
    }

    /// Update all neurons' metabolism
    pub fn step(
        &mut self,
        dt: f64,
        spikes: &[bool],
        synaptic_events: &[usize],
    ) -> Result<()> {
        // Update blood flow based on activity
        let activity_level = spikes.iter().filter(|&&s| s).count() as f64 / self.num_neurons as f64;
        self.blood_flow.update_from_activity(activity_level);

        // Update each neuron
        self.total_atp_consumption = 0.0;
        self.total_atp_production = 0.0;

        for i in 0..self.num_neurons {
            // Supply nutrients from blood
            self.neuron_metabolism[i].supply_nutrients(
                self.blood_flow.glucose_delivery * dt,
                self.blood_flow.oxygen_delivery * dt,
            );

            // Update metabolism
            self.neuron_metabolism[i].step(dt, spikes[i], synaptic_events[i])?;

            self.total_atp_consumption += self.neuron_metabolism[i].atp_consumption_rate;
            self.total_atp_production += self.neuron_metabolism[i].atp_production_rate;
        }

        Ok(())
    }

    /// Get average ATP level
    pub fn average_atp(&self) -> f64 {
        self.neuron_metabolism.iter().map(|m| m.atp).sum::<f64>() / self.num_neurons as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_metabolism() {
        let mut metab = NeuronMetabolism::new();
        let initial_atp = metab.atp;

        // Simulate without activity - should maintain ATP
        metab.step(0.1, false, 0).unwrap();
        assert!(metab.atp > 0.0);

        // Simulate with activity - should consume ATP
        metab.atp = initial_atp;
        for _ in 0..100 {
            metab.step(0.1, true, 10).unwrap();
        }

        // ATP should equilibrate based on production/consumption
        assert!(metab.atp > 0.1);
        assert!(metab.atp < 3.0);
    }

    #[test]
    fn test_blood_flow() {
        let mut flow = BloodFlow::new();
        let initial_rate = flow.flow_rate;

        // High activity should increase blood flow
        flow.update_from_activity(1.0);
        assert!(flow.flow_rate > initial_rate);

        // Low activity should decrease blood flow
        for _ in 0..1000 {
            flow.update_from_activity(0.0);
        }
        assert!(flow.flow_rate < initial_rate * 1.1);
    }

    #[test]
    fn test_regional_metabolism() {
        let mut region = RegionalMetabolism::new(100);
        let spikes = vec![false; 100];
        let synaptic_events = vec![0; 100];

        region.step(0.1, &spikes, &synaptic_events).unwrap();

        let avg_atp = region.average_atp();
        assert!(avg_atp > 0.0);
    }
}
