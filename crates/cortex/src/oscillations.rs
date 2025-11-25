//! Brain oscillations (delta, theta, alpha, beta, gamma) implementation.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillationBand {
    pub name: String,
    pub frequency_range: (f64, f64),  // Hz
    pub power: f64,
    pub phase: f64,
}

impl OscillationBand {
    pub fn new(name: &str, freq_min: f64, freq_max: f64) -> Self {
        Self {
            name: name.to_string(),
            frequency_range: (freq_min, freq_max),
            power: 0.0,
            phase: 0.0,
        }
    }

    pub fn step(&mut self, dt: f64, driving_input: f64) {
        let center_freq = (self.frequency_range.0 + self.frequency_range.1) / 2.0;
        self.phase += 2.0 * PI * center_freq * dt / 1000.0;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        self.power = 0.9 * self.power + 0.1 * driving_input;
    }

    pub fn modulation(&self) -> f64 {
        self.power * (0.5 + 0.5 * self.phase.cos())
    }
}

pub struct BrainOscillations {
    pub delta: OscillationBand,    // 0.5-4 Hz - deep sleep
    pub theta: OscillationBand,    // 4-8 Hz - memory encoding
    pub alpha: OscillationBand,    // 8-13 Hz - relaxed wakefulness
    pub beta: OscillationBand,     // 13-30 Hz - active thinking
    pub gamma: OscillationBand,    // 30-100 Hz - binding, attention
}

impl BrainOscillations {
    pub fn new() -> Self {
        Self {
            delta: OscillationBand::new("delta", 0.5, 4.0),
            theta: OscillationBand::new("theta", 4.0, 8.0),
            alpha: OscillationBand::new("alpha", 8.0, 13.0),
            beta: OscillationBand::new("beta", 13.0, 30.0),
            gamma: OscillationBand::new("gamma", 30.0, 100.0),
        }
    }

    pub fn step(&mut self, dt: f64, arousal: f64, attention: f64, sleep_depth: f64) {
        self.delta.step(dt, sleep_depth);
        self.theta.step(dt, 1.0 - sleep_depth);
        self.alpha.step(dt, (1.0 - arousal) * (1.0 - sleep_depth));
        self.beta.step(dt, arousal * (1.0 - sleep_depth));
        self.gamma.step(dt, attention * arousal);
    }

    pub fn total_modulation(&self) -> f64 {
        self.delta.modulation() + self.theta.modulation() +
        self.alpha.modulation() + self.beta.modulation() + self.gamma.modulation()
    }
}
