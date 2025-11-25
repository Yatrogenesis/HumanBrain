//! Pharmacology - drug effects on brain function.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEffect {
    pub name: String,
    pub concentration: f64,
    pub half_life: f64,  // Hours
}

impl DrugEffect {
    pub fn new(name: &str, concentration: f64, half_life: f64) -> Self {
        Self { name: name.to_string(), concentration, half_life }
    }

    pub fn decay(&mut self, dt_hours: f64) {
        let decay_rate = 0.693 / self.half_life;
        self.concentration *= (-decay_rate * dt_hours).exp();
    }
}

pub struct Pharmacology;

impl Pharmacology {
    pub fn benzodiazepine_effects(concentration: f64) -> DrugEffects {
        DrugEffects {
            gaba_a_potentiation: 1.0 + 2.0 * concentration,
            anxiolytic: concentration,
            sedation: concentration,
            amnesia: 0.5 * concentration,
        }
    }

    pub fn ssri_effects(concentration: f64) -> DrugEffects {
        DrugEffects {
            serotonin_level: 1.0 + 1.5 * concentration,
            mood_elevation: concentration * 0.7,
            anxiety_reduction: concentration * 0.5,
            sleep_disturbance: concentration * 0.3,
        }
    }

    pub fn amphetamine_effects(concentration: f64) -> DrugEffects {
        DrugEffects {
            dopamine_release: 2.0 + 3.0 * concentration,
            norepinephrine_release: 1.5 + 2.0 * concentration,
            arousal: concentration,
            focus: concentration * 0.8,
        }
    }

    pub fn caffeine_effects(concentration: f64) -> DrugEffects {
        DrugEffects {
            adenosine_antagonism: concentration,
            alertness: concentration * 0.7,
            arousal: concentration * 0.5,
            jitteriness: if concentration > 0.7 { concentration - 0.7 } else { 0.0 },
        }
    }

    pub fn psychedelic_effects(concentration: f64) -> DrugEffects {
        DrugEffects {
            ht2a_agonism: concentration,
            perceptual_distortion: concentration,
            cognitive_flexibility: concentration * 0.8,
            default_mode_disruption: concentration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrugEffects {
    pub gaba_a_potentiation: f64,
    pub anxiolytic: f64,
    pub sedation: f64,
    pub amnesia: f64,
    pub serotonin_level: f64,
    pub mood_elevation: f64,
    pub anxiety_reduction: f64,
    pub sleep_disturbance: f64,
    pub dopamine_release: f64,
    pub norepinephrine_release: f64,
    pub arousal: f64,
    pub focus: f64,
    pub adenosine_antagonism: f64,
    pub alertness: f64,
    pub jitteriness: f64,
    pub ht2a_agonism: f64,
    pub perceptual_distortion: f64,
    pub cognitive_flexibility: f64,
    pub default_mode_disruption: f64,
}

impl Default for DrugEffects {
    fn default() -> Self {
        Self {
            gaba_a_potentiation: 1.0, anxiolytic: 0.0, sedation: 0.0, amnesia: 0.0,
            serotonin_level: 1.0, mood_elevation: 0.0, anxiety_reduction: 0.0, sleep_disturbance: 0.0,
            dopamine_release: 1.0, norepinephrine_release: 1.0, arousal: 0.0, focus: 0.0,
            adenosine_antagonism: 0.0, alertness: 0.0, jitteriness: 0.0,
            ht2a_agonism: 0.0, perceptual_distortion: 0.0, cognitive_flexibility: 0.0, default_mode_disruption: 0.0,
        }
    }
}
