//! Neurotransmitter systems and neuromodulation.

use serde::{Deserialize, Serialize};

/// Neurotransmitter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Neurotransmitter {
    Glutamate,
    GABA,
    Dopamine,
    Serotonin,
    Norepinephrine,
    Acetylcholine,
    Histamine,
}

/// Neuromodulator concentrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromodulatorState {
    pub dopamine: f64,
    pub serotonin: f64,
    pub norepinephrine: f64,
    pub acetylcholine: f64,
}

impl Default for NeuromodulatorState {
    fn default() -> Self {
        Self {
            dopamine: 0.1,
            serotonin: 0.1,
            norepinephrine: 0.1,
            acetylcholine: 0.1,
        }
    }
}
