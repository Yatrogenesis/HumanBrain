//! Neuromodulation effects on circuit dynamics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromodulatorLevels {
    pub dopamine: f64,
    pub serotonin: f64,
    pub acetylcholine: f64,
    pub norepinephrine: f64,
}

impl NeuromodulatorLevels {
    pub fn new() -> Self {
        Self { dopamine: 0.2, serotonin: 0.3, acetylcholine: 0.3, norepinephrine: 0.2 }
    }
}

pub struct NeuromodulationEffects;

impl NeuromodulationEffects {
    pub fn dopamine_effects(level: f64) -> ModulationFactors {
        ModulationFactors {
            excitability: 1.0 + 0.5 * level,
            learning_rate: 1.0 + level,
            working_memory_gain: 1.0 + 0.3 * level,
            motor_vigor: 1.0 + 0.4 * level,
        }
    }

    pub fn serotonin_effects(level: f64) -> ModulationFactors {
        ModulationFactors {
            excitability: 1.0 - 0.2 * level,
            learning_rate: 1.0,
            working_memory_gain: 1.0,
            motor_vigor: 1.0 - 0.2 * level,
        }
    }

    pub fn acetylcholine_effects(level: f64) -> ModulationFactors {
        ModulationFactors {
            excitability: 1.0 + 0.3 * level,
            learning_rate: 1.0 + 0.6 * level,
            working_memory_gain: 1.0 + 0.5 * level,
            motor_vigor: 1.0,
        }
    }

    pub fn norepinephrine_effects(level: f64) -> ModulationFactors {
        ModulationFactors {
            excitability: 1.0 + 0.4 * level,
            learning_rate: 1.0 + 0.3 * level,
            working_memory_gain: 1.0 + 0.4 * level,
            motor_vigor: 1.0 + 0.3 * level,
        }
    }

    pub fn combined_effects(levels: &NeuromodulatorLevels) -> ModulationFactors {
        let da = Self::dopamine_effects(levels.dopamine);
        let se = Self::serotonin_effects(levels.serotonin);
        let ac = Self::acetylcholine_effects(levels.acetylcholine);
        let ne = Self::norepinephrine_effects(levels.norepinephrine);

        ModulationFactors {
            excitability: (da.excitability + se.excitability + ac.excitability + ne.excitability) / 4.0,
            learning_rate: (da.learning_rate + se.learning_rate + ac.learning_rate + ne.learning_rate) / 4.0,
            working_memory_gain: (da.working_memory_gain + se.working_memory_gain + ac.working_memory_gain + ne.working_memory_gain) / 4.0,
            motor_vigor: (da.motor_vigor + se.motor_vigor + ac.motor_vigor + ne.motor_vigor) / 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModulationFactors {
    pub excitability: f64,
    pub learning_rate: f64,
    pub working_memory_gain: f64,
    pub motor_vigor: f64,
}
