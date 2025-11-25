//! Hypothalamus: Homeostatic control center
//!
//! Key nuclei:
//! - Arcuate nucleus (ARC): Hunger/satiety (NPY/AgRP vs POMC/CART)
//! - Paraventricular nucleus (PVN): Stress response (CRH, oxytocin, vasopressin)
//! - Suprachiasmatic nucleus (SCN): Circadian clock [already in cognition/circadian.rs]
//! - Lateral hypothalamus (LH): Orexin neurons (arousal, reward)

use serde::{Deserialize, Serialize};

/// Arcuate nucleus: Appetite control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcuateNucleus {
    /// NPY/AgRP neurons (orexigenic - increase appetite)
    pub npy_agrp_activity: f64,  // 0-1

    /// POMC/CART neurons (anorexigenic - decrease appetite)
    pub pomc_cart_activity: f64,  // 0-1

    /// Leptin levels (satiety hormone from adipose tissue)
    pub leptin: f64,  // ng/mL

    /// Ghrelin levels (hunger hormone from stomach)
    pub ghrelin: f64,  // pg/mL
}

impl ArcuateNucleus {
    pub fn new() -> Self {
        Self {
            npy_agrp_activity: 0.3,  // Baseline hunger
            pomc_cart_activity: 0.5,  // Baseline satiety
            leptin: 10.0,  // Normal leptin
            ghrelin: 500.0,  // Fasting ghrelin
        }
    }

    /// Update based on metabolic state
    pub fn update(&mut self, dt: f64, glucose: f64, time_since_meal: f64) {
        // Ghrelin increases with time since meal (peaks ~4 hours)
        self.ghrelin = 500.0 + 300.0 * (time_since_meal / 4.0).min(1.0);

        // Leptin reflects long-term energy stores (simplified)
        // self.leptin would track adipose tissue state

        // NPY/AgRP activated by ghrelin, inhibited by leptin & glucose
        let hunger_drive = (self.ghrelin / 800.0) - (self.leptin / 20.0) - (glucose / 150.0);
        self.npy_agrp_activity += (hunger_drive - self.npy_agrp_activity) * 0.1 * dt;
        self.npy_agrp_activity = self.npy_agrp_activity.clamp(0.0, 1.0);

        // POMC/CART activated by leptin & glucose, inhibited by ghrelin
        let satiety_drive = (self.leptin / 15.0) + (glucose / 120.0) - (self.ghrelin / 1000.0);
        self.pomc_cart_activity += (satiety_drive - self.pomc_cart_activity) * 0.1 * dt;
        self.pomc_cart_activity = self.pomc_cart_activity.clamp(0.0, 1.0);
    }

    /// Get hunger signal (0-1, higher = more hungry)
    pub fn hunger_signal(&self) -> f64 {
        self.npy_agrp_activity - 0.5 * self.pomc_cart_activity
    }
}

/// Paraventricular nucleus: Stress & neuroendocrine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParaventricularNucleus {
    /// CRH neurons (stress response, activates HPA axis)
    pub crh_activity: f64,  // 0-1

    /// Oxytocin neurons (social bonding, lactation)
    pub oxytocin_release: f64,  // U/mL

    /// Vasopressin neurons (water retention, blood pressure)
    pub vasopressin_release: f64,  // pg/mL

    /// Cortisol levels (negative feedback)
    pub cortisol: f64,  // µg/dL
}

impl ParaventricularNucleus {
    pub fn new() -> Self {
        Self {
            crh_activity: 0.2,
            oxytocin_release: 5.0,
            vasopressin_release: 2.0,
            cortisol: 10.0,  // Morning baseline
        }
    }

    /// Respond to stressor
    pub fn activate_stress_response(&mut self, stressor_intensity: f64) {
        // CRH increases with stress, inhibited by cortisol (negative feedback)
        let crh_drive = stressor_intensity - (self.cortisol / 30.0);
        self.crh_activity += crh_drive * 0.2;
        self.crh_activity = self.crh_activity.clamp(0.0, 1.0);

        // CRH → ACTH → Cortisol (HPA axis, simplified)
        self.cortisol += self.crh_activity * 2.0;  // Rises over minutes
        self.cortisol = (self.cortisol * 0.99).max(5.0);  // Decay to baseline
    }
}

/// Complete hypothalamus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothalamus {
    pub arcuate: ArcuateNucleus,
    pub pvn: ParaventricularNucleus,
}

impl Hypothalamus {
    pub fn new() -> Self {
        Self {
            arcuate: ArcuateNucleus::new(),
            pvn: ParaventricularNucleus::new(),
        }
    }

    pub fn step(&mut self, dt: f64, glucose: f64, time_since_meal: f64, stress: f64) {
        self.arcuate.update(dt, glucose, time_since_meal);
        self.pvn.activate_stress_response(stress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arcuate_hunger() {
        let mut arc = ArcuateNucleus::new();

        // Simulate fasting (4 hours since meal)
        arc.update(1.0, 70.0, 4.0);  // Low glucose, long time

        assert!(arc.hunger_signal() > 0.0);  // Should feel hungry
    }

    #[test]
    fn test_pvn_stress() {
        let mut pvn = ParaventricularNucleus::new();
        let baseline_cortisol = pvn.cortisol;

        // Acute stressor
        pvn.activate_stress_response(0.8);

        assert!(pvn.cortisol > baseline_cortisol);  // Cortisol rises
        assert!(pvn.crh_activity > 0.2);  // CRH activated
    }
}
