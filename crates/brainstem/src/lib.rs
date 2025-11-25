//! Brainstem monoaminergic nuclei
//!
//! - Raphe nuclei: Serotonin (5-HT)
//! - Locus coeruleus: Norepinephrine (NE)
//! - Ventral tegmental area: Dopamine (DA) [in basal-ganglia, refine here]

use serde::{Deserialize, Serialize};

/// Raphe nuclei: Serotonergic system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RapheNuclei {
    /// 5-HT neuron firing rate (Hz)
    pub firing_rate: f64,

    /// Serotonin concentration (nM)
    pub serotonin: f64,

    /// 5-HT1A autoreceptor activation (negative feedback)
    pub autoreceptor_activation: f64,  // 0-1
}

impl RapheNuclei {
    pub fn new() -> Self {
        Self {
            firing_rate: 0.5,  // Baseline firing during normal state
            serotonin: 50.0,
            autoreceptor_activation: 0.3,
        }
    }

    /// Update firing based on arousal state
    pub fn update(&mut self, dt: f64, arousal: f64) {
        // Firing increases with arousal, decreases during sleep
        // Autoreceptors provide negative feedback (multiplicative)
        let drive = arousal * (1.0 - 0.5 * self.autoreceptor_activation);
        self.firing_rate += (drive - self.firing_rate) * 0.1 * dt;
        self.firing_rate = self.firing_rate.clamp(0.0, 3.0);

        // Serotonin release proportional to firing
        self.serotonin += self.firing_rate * 10.0 * dt;
        self.serotonin *= 0.95;  // Reuptake/degradation

        // Autoreceptor activation by serotonin
        self.autoreceptor_activation = 1.0 / (1.0 + ((-0.05 * (self.serotonin - 50.0)).exp()));
    }
}

/// Locus coeruleus: Noradrenergic system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocusCoeruleus {
    /// NE neuron firing mode
    pub firing_mode: FiringMode,

    /// Norepinephrine concentration (nM)
    pub norepinephrine: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiringMode {
    /// Low tonic firing (~1 Hz) - drowsy/inattentive
    Tonic,

    /// Phasic bursts (5-10 Hz) - alert, responding to salient stimuli
    Phasic,
}

impl LocusCoeruleus {
    pub fn new() -> Self {
        Self {
            firing_mode: FiringMode::Tonic,
            norepinephrine: 100.0,
        }
    }

    /// Switch to phasic mode in response to salient stimulus
    pub fn respond_to_stimulus(&mut self, salience: f64) {
        if salience > 0.6 {
            self.firing_mode = FiringMode::Phasic;
            self.norepinephrine += 50.0;  // Burst release
        }
    }

    pub fn update(&mut self, _dt: f64) {
        // Return to tonic mode over time
        if self.firing_mode == FiringMode::Phasic {
            self.firing_mode = FiringMode::Tonic;  // Simplified: instant return
        }

        self.norepinephrine *= 0.9;  // Decay
        self.norepinephrine = self.norepinephrine.max(50.0);  // Baseline
    }
}

/// Complete brainstem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brainstem {
    pub raphe: RapheNuclei,
    pub locus_coeruleus: LocusCoeruleus,
}

impl Brainstem {
    pub fn new() -> Self {
        Self {
            raphe: RapheNuclei::new(),
            locus_coeruleus: LocusCoeruleus::new(),
        }
    }

    pub fn step(&mut self, dt: f64, arousal: f64, stimulus_salience: f64) {
        self.raphe.update(dt, arousal);

        if stimulus_salience > 0.6 {
            self.locus_coeruleus.respond_to_stimulus(stimulus_salience);
        }
        self.locus_coeruleus.update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raphe_serotonin() {
        let raphe = RapheNuclei::new();

        // Test high arousal vs low arousal
        let mut raphe_high = raphe.clone();
        let mut raphe_low = raphe.clone();

        // High arousal
        for _ in 0..50 {
            raphe_high.update(1.0, 0.9);
        }

        // Low arousal
        for _ in 0..50 {
            raphe_low.update(1.0, 0.1);
        }

        // High arousal should lead to higher firing than low arousal
        assert!(raphe_high.firing_rate > raphe_low.firing_rate);
        // Both should maintain positive serotonin
        assert!(raphe_high.serotonin > 0.0);
        assert!(raphe_low.serotonin > 0.0);
    }

    #[test]
    fn test_lc_phasic() {
        let mut lc = LocusCoeruleus::new();
        let baseline = lc.norepinephrine;

        lc.respond_to_stimulus(0.8);

        assert_eq!(lc.firing_mode, FiringMode::Phasic);
        assert!(lc.norepinephrine > baseline);
    }
}
