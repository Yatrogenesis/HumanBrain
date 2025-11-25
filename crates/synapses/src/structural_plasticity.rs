//! Structural plasticity - synaptogenesis, pruning, and spine dynamics.
//!
//! This module implements long-term structural changes in neural networks including:
//! - Synaptogenesis (formation of new synapses)
//! - Synaptic pruning (elimination of weak synapses)
//! - Dendritic spine dynamics (growth, shrinkage, stabilization)
//! - Axonal sprouting
//! - Activity-dependent structural remodeling

use serde::{Deserialize, Serialize};
use rand::Rng;

/// Dendritic spine morphology types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpineType {
    Thin,      // Small, plastic, learning-related
    Stubby,    // Intermediate
    Mushroom,  // Large, stable, memory storage
    Filopodial, // Exploratory, transient
}

/// Dendritic spine state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DendriticSpine {
    pub id: usize,
    pub spine_type: SpineType,
    pub volume: f64,          // μm³
    pub psd_area: f64,        // Postsynaptic density area (μm²)
    pub neck_length: f64,     // μm
    pub neck_diameter: f64,   // μm
    pub age: f64,             // Days since formation
    pub stability: f64,       // 0-1, higher = more stable
    pub activity_history: Vec<f64>, // Recent activity levels
}

impl DendriticSpine {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            spine_type: SpineType::Thin,
            volume: 0.02,         // Small initial volume
            psd_area: 0.04,
            neck_length: 1.0,
            neck_diameter: 0.1,
            age: 0.0,
            stability: 0.1,       // Initially unstable
            activity_history: Vec::new(),
        }
    }

    pub fn new_mushroom(id: usize) -> Self {
        Self {
            id,
            spine_type: SpineType::Mushroom,
            volume: 0.1,          // Large volume
            psd_area: 0.15,
            neck_length: 0.8,
            neck_diameter: 0.15,
            age: 100.0,           // Mature
            stability: 0.9,       // Very stable
            activity_history: Vec::new(),
        }
    }

    /// Update spine based on recent activity
    pub fn update(&mut self, dt: f64, synaptic_activity: f64, calcium: f64) {
        self.age += dt;

        // Record activity
        self.activity_history.push(synaptic_activity);
        if self.activity_history.len() > 1000 {
            self.activity_history.remove(0);
        }

        // Calculate average recent activity
        let avg_activity = if self.activity_history.is_empty() {
            0.0
        } else {
            self.activity_history.iter().sum::<f64>() / self.activity_history.len() as f64
        };

        // Volume changes based on activity and calcium
        let volume_change_rate = 0.001; // μm³/day
        let target_volume = match self.spine_type {
            SpineType::Thin => 0.01 + 0.02 * avg_activity,
            SpineType::Stubby => 0.04 + 0.03 * avg_activity,
            SpineType::Mushroom => 0.08 + 0.05 * avg_activity,
            SpineType::Filopodial => 0.005 + 0.01 * avg_activity,
        };

        self.volume += (target_volume - self.volume) * volume_change_rate * dt;
        self.volume = self.volume.clamp(0.005, 0.15);

        // PSD area scales with volume
        self.psd_area = self.volume * 1.5;

        // Stability increases with age and consistent activity
        let stability_increase_rate = 0.001; // per day
        if avg_activity > 0.3 && calcium > 0.2 {
            self.stability += stability_increase_rate * dt;
        } else if avg_activity < 0.1 {
            self.stability -= stability_increase_rate * 0.5 * dt;
        }
        self.stability = self.stability.clamp(0.0, 1.0);

        // Spine type transitions based on volume and stability
        self.update_spine_type();
    }

    fn update_spine_type(&mut self) {
        if self.volume < 0.02 {
            self.spine_type = SpineType::Thin;
        } else if self.volume < 0.05 {
            self.spine_type = SpineType::Stubby;
        } else if self.stability > 0.7 {
            self.spine_type = SpineType::Mushroom;
        }
    }

    /// Check if spine should be eliminated
    pub fn should_prune(&self) -> bool {
        // Prune if very small, very unstable, or chronically inactive
        let avg_activity = if self.activity_history.is_empty() {
            0.0
        } else {
            self.activity_history.iter().sum::<f64>() / self.activity_history.len() as f64
        };

        (self.volume < 0.008 && self.age > 10.0) ||
        (self.stability < 0.05 && self.age > 5.0) ||
        (avg_activity < 0.01 && self.age > 20.0)
    }
}

/// Structural plasticity manager for a neuron or network region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPlasticityManager {
    pub spines: Vec<DendriticSpine>,
    pub next_spine_id: usize,
    pub synaptogenesis_rate: f64,    // New synapses per day
    pub pruning_rate: f64,            // Fraction of weak synapses pruned per day
    pub critical_period: bool,        // Higher plasticity during development
}

impl StructuralPlasticityManager {
    pub fn new(initial_spines: usize) -> Self {
        let mut spines = Vec::new();
        for i in 0..initial_spines {
            // Mix of spine types
            if i % 10 == 0 {
                spines.push(DendriticSpine::new_mushroom(i));
            } else {
                spines.push(DendriticSpine::new(i));
            }
        }

        Self {
            spines,
            next_spine_id: initial_spines,
            synaptogenesis_rate: 5.0,   // 5 new synapses/day
            pruning_rate: 0.02,          // 2% pruned per day
            critical_period: false,
        }
    }

    /// Update all spines and perform structural plasticity
    pub fn step(&mut self, dt: f64, activity: &[f64], calcium: &[f64]) {
        // Update existing spines
        for (i, spine) in self.spines.iter_mut().enumerate() {
            let synaptic_activity = if i < activity.len() { activity[i] } else { 0.0 };
            let ca = if i < calcium.len() { calcium[i] } else { 0.05 };
            spine.update(dt, synaptic_activity, ca);
        }

        // Prune weak spines
        self.prune_synapses();

        // Form new synapses based on activity
        self.synaptogenesis(dt, activity);
    }

    /// Synaptogenesis - form new synapses
    fn synaptogenesis(&mut self, dt: f64, activity: &[f64]) {
        let rate_factor = if self.critical_period { 3.0 } else { 1.0 };
        let num_new = (self.synaptogenesis_rate * rate_factor * dt) as usize;

        // Higher activity promotes synapse formation
        let avg_activity = if activity.is_empty() {
            0.0
        } else {
            activity.iter().sum::<f64>() / activity.len() as f64
        };

        let activity_factor = (0.5 + 1.5 * avg_activity).min(2.0);
        let num_new_adjusted = ((num_new as f64) * activity_factor) as usize;

        for _ in 0..num_new_adjusted {
            let new_spine = DendriticSpine::new(self.next_spine_id);
            self.spines.push(new_spine);
            self.next_spine_id += 1;
        }
    }

    /// Synaptic pruning - eliminate weak synapses
    fn prune_synapses(&mut self) {
        let rate_factor = if self.critical_period { 0.5 } else { 1.0 };
        let effective_pruning_rate = self.pruning_rate * rate_factor;

        self.spines.retain(|spine| {
            let should_keep = !spine.should_prune();
            let mut rng = rand::thread_rng();
            should_keep && (rng.gen::<f64>() > effective_pruning_rate)
        });
    }

    /// Get statistics about spine population
    pub fn spine_statistics(&self) -> SpineStatistics {
        let mut stats = SpineStatistics::default();
        stats.total = self.spines.len();

        for spine in &self.spines {
            match spine.spine_type {
                SpineType::Thin => stats.thin += 1,
                SpineType::Stubby => stats.stubby += 1,
                SpineType::Mushroom => stats.mushroom += 1,
                SpineType::Filopodial => stats.filopodial += 1,
            }

            stats.avg_volume += spine.volume;
            stats.avg_stability += spine.stability;
        }

        if stats.total > 0 {
            stats.avg_volume /= stats.total as f64;
            stats.avg_stability /= stats.total as f64;
        }

        stats
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpineStatistics {
    pub total: usize,
    pub thin: usize,
    pub stubby: usize,
    pub mushroom: usize,
    pub filopodial: usize,
    pub avg_volume: f64,
    pub avg_stability: f64,
}

/// Axonal sprouting model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonalSprouting {
    pub growth_cone_active: bool,
    pub growth_rate: f64,        // μm/hour
    pub guidance_factors: Vec<f64>, // Molecular guidance cues
    pub position: [f64; 3],      // Current position
}

impl AxonalSprouting {
    pub fn new(position: [f64; 3]) -> Self {
        Self {
            growth_cone_active: true,
            growth_rate: 50.0,   // 50 μm/hour typical
            guidance_factors: vec![0.0; 10],
            position,
        }
    }

    pub fn step(&mut self, dt: f64) {
        if !self.growth_cone_active {
            return;
        }

        // Simplified growth: move in direction of guidance factors
        let mut direction = [0.0; 3];
        for (i, &factor) in self.guidance_factors.iter().enumerate().take(3) {
            direction[i] = factor;
        }

        // Normalize direction
        let magnitude = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
        if magnitude > 0.0 {
            for d in &mut direction {
                *d /= magnitude;
            }
        }

        // Move growth cone
        for i in 0..3 {
            self.position[i] += direction[i] * self.growth_rate * dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spine_creation() {
        let spine = DendriticSpine::new(0);
        assert_eq!(spine.spine_type, SpineType::Thin);
        assert!(spine.volume < 0.05);
    }

    #[test]
    fn test_spine_maturation() {
        let mut spine = DendriticSpine::new(0);

        // Simulate high activity over time
        for _ in 0..1000 {
            spine.update(0.1, 0.8, 0.5); // High activity, high calcium
        }

        // Spine should grow and stabilize
        assert!(spine.volume > 0.03);
        assert!(spine.stability > 0.3);
    }

    #[test]
    fn test_spine_pruning() {
        let mut spine = DendriticSpine::new(0);

        // Simulate low activity over long time
        for _ in 0..5000 {
            spine.update(0.1, 0.0, 0.05); // No activity
        }

        // Spine should be marked for pruning
        assert!(spine.should_prune());
    }

    #[test]
    fn test_synaptogenesis() {
        let mut manager = StructuralPlasticityManager::new(100);
        let initial_count = manager.spines.len();

        let activity = vec![0.5; 100];
        let calcium = vec![0.3; 100];

        manager.step(1.0, &activity, &calcium); // 1 day

        // Should have formed new synapses
        assert!(manager.spines.len() >= initial_count);
    }

    #[test]
    fn test_spine_type_transition() {
        let mut spine = DendriticSpine::new(0);
        spine.volume = 0.06;
        spine.stability = 0.8;

        spine.update_spine_type();
        assert_eq!(spine.spine_type, SpineType::Mushroom);
    }

    #[test]
    fn test_critical_period() {
        let mut manager = StructuralPlasticityManager::new(100);
        manager.critical_period = true;

        let activity = vec![0.5; 100];
        let calcium = vec![0.3; 100];

        manager.step(1.0, &activity, &calcium);

        // Critical period should enhance synaptogenesis
        assert!(manager.spines.len() > 100);
    }
}
