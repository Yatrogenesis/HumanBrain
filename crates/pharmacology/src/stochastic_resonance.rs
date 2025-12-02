//! Stochastic Resonance and Chaotic Threshold Dynamics
//! =====================================================
//!
//! Models noise-enhanced signal detection and chaotic threshold crossings:
//! - Stochastic resonance (noise amplifies subthreshold signals)
//! - Coherence resonance (noise-induced periodic behavior)
//! - Chaotic oscillators for rare event triggering
//! - Strange attractors for complex dynamics
//!
//! # Stochastic Resonance
//!
//! In nonlinear systems with a threshold, optimal noise can ENHANCE
//! signal detection rather than degrade it:
//!
//! ```text
//! Signal-to-Noise Ratio
//!        │
//!        │     ╱╲
//!        │    ╱  ╲      ← Optimal noise
//!        │   ╱    ╲
//!        │  ╱      ╲
//!        │_╱________╲___
//!        └──────────────► Noise Intensity
//! ```
//!
//! # Ontological Oscillator for Rare Events
//!
//! The user's concept: instead of simple Gaussian noise, use a chaotic
//! oscillator that occasionally produces large excursions - capturing
//! the 3-5% of cases where rare adverse events occur.
//!
//! This models:
//! - Individual variation in threshold sensitivity
//! - Rare pharmacodynamic "catastrophes"
//! - Idiosyncratic drug reactions
//! - Emergence of new behaviors from complex dynamics

use serde::{Deserialize, Serialize};
use rand::Rng;
use rand_distr::{Distribution, Normal, Uniform};

/// Type of noise/stochastic process
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NoiseType {
    /// Gaussian white noise (standard)
    Gaussian,
    /// Pink noise (1/f, more low-frequency content)
    Pink,
    /// Poisson process (discrete events)
    Poisson,
    /// Ornstein-Uhlenbeck (mean-reverting)
    OrnsteinUhlenbeck,
    /// Lévy flights (heavy tails, rare large jumps)
    LevyFlight,
    /// Chaotic (deterministic chaos)
    Chaotic,
}

/// Parameters for stochastic resonance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticResonanceParams {
    /// Signal amplitude (subthreshold)
    pub signal_amplitude: f64,
    /// Signal frequency (Hz)
    pub signal_frequency_hz: f64,
    /// Threshold for detection
    pub threshold: f64,
    /// Noise intensity (σ)
    pub noise_intensity: f64,
    /// Time constant of the bistable well (s)
    pub tau: f64,
}

impl Default for StochasticResonanceParams {
    fn default() -> Self {
        Self {
            signal_amplitude: 0.3,      // Subthreshold
            signal_frequency_hz: 1.0,
            threshold: 1.0,
            noise_intensity: 0.5,
            tau: 0.1,
        }
    }
}

/// Bistable potential for stochastic resonance
/// V(x) = -x²/2 + x⁴/4 (double-well potential)
#[derive(Debug, Clone)]
pub struct BistableSystem {
    /// Current state
    pub x: f64,
    /// Parameters
    pub params: StochasticResonanceParams,
    /// Time
    pub time: f64,
    /// Noise generator
    noise: Normal<f64>,
}

impl BistableSystem {
    pub fn new(params: StochasticResonanceParams) -> Self {
        Self {
            x: -1.0,  // Start in left well
            params,
            time: 0.0,
            noise: Normal::new(0.0, 1.0).unwrap(),
        }
    }

    /// Potential energy V(x) = -x²/2 + x⁴/4
    pub fn potential(&self, x: f64) -> f64 {
        -x.powi(2) / 2.0 + x.powi(4) / 4.0
    }

    /// Force F(x) = -dV/dx = x - x³
    pub fn force(&self, x: f64) -> f64 {
        x - x.powi(3)
    }

    /// Signal at current time
    pub fn signal(&self) -> f64 {
        self.params.signal_amplitude
            * (2.0 * std::f64::consts::PI * self.params.signal_frequency_hz * self.time).sin()
    }

    /// Update system (Euler-Maruyama integration)
    pub fn step(&mut self, dt: f64) {
        let mut rng = rand::thread_rng();

        // Deterministic part: dx = (x - x³ + signal) dt / τ
        let drift = (self.force(self.x) + self.signal()) / self.params.tau;

        // Stochastic part: σ √(2/τ) dW
        let diffusion = self.params.noise_intensity
            * (2.0 / self.params.tau).sqrt()
            * self.noise.sample(&mut rng)
            * dt.sqrt();

        self.x += drift * dt + diffusion;
        self.time += dt;
    }

    /// Check if system crossed threshold (switched wells)
    pub fn crossed_threshold(&self) -> bool {
        self.x > 0.0  // Positive well
    }

    /// Calculate Kramers escape rate (analytical)
    pub fn kramers_rate(&self) -> f64 {
        // r = (1/2πτ) exp(-ΔV/D)
        // ΔV = 0.25 (barrier height for this potential)
        // D = σ²
        let barrier = 0.25;
        let d = self.params.noise_intensity.powi(2);

        (1.0 / (2.0 * std::f64::consts::PI * self.params.tau))
            * (-barrier / d).exp()
    }

    /// Find optimal noise intensity for given signal
    pub fn find_optimal_noise(&self) -> f64 {
        // For stochastic resonance, optimal noise ~ √(ΔV)
        // where ΔV is the barrier height
        (0.25_f64).sqrt()
    }
}

/// Lorenz attractor for deterministic chaos
#[derive(Debug, Clone)]
pub struct LorenzAttractor {
    /// State (x, y, z)
    pub state: (f64, f64, f64),
    /// Lorenz parameters
    pub sigma: f64,  // Prandtl number
    pub rho: f64,    // Rayleigh number
    pub beta: f64,   // Aspect ratio
    /// Time
    pub time: f64,
}

impl Default for LorenzAttractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LorenzAttractor {
    pub fn new() -> Self {
        Self {
            state: (1.0, 1.0, 1.0),  // Initial condition
            sigma: 10.0,             // Classic parameters
            rho: 28.0,
            beta: 8.0 / 3.0,
            time: 0.0,
        }
    }

    /// Derivatives of Lorenz system
    fn derivatives(&self, state: (f64, f64, f64)) -> (f64, f64, f64) {
        let (x, y, z) = state;
        (
            self.sigma * (y - x),
            x * (self.rho - z) - y,
            x * y - self.beta * z,
        )
    }

    /// Runge-Kutta 4th order integration
    pub fn step(&mut self, dt: f64) {
        let k1 = self.derivatives(self.state);

        let state2 = (
            self.state.0 + 0.5 * dt * k1.0,
            self.state.1 + 0.5 * dt * k1.1,
            self.state.2 + 0.5 * dt * k1.2,
        );
        let k2 = self.derivatives(state2);

        let state3 = (
            self.state.0 + 0.5 * dt * k2.0,
            self.state.1 + 0.5 * dt * k2.1,
            self.state.2 + 0.5 * dt * k2.2,
        );
        let k3 = self.derivatives(state3);

        let state4 = (
            self.state.0 + dt * k3.0,
            self.state.1 + dt * k3.1,
            self.state.2 + dt * k3.2,
        );
        let k4 = self.derivatives(state4);

        self.state.0 += dt * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0) / 6.0;
        self.state.1 += dt * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1) / 6.0;
        self.state.2 += dt * (k1.2 + 2.0 * k2.2 + 2.0 * k3.2 + k4.2) / 6.0;

        self.time += dt;
    }

    /// Normalized output suitable for perturbation
    /// Maps the chaotic trajectory to [-1, 1]
    pub fn normalized_output(&self) -> f64 {
        // z typically varies from ~0 to ~50, center around 25
        (self.state.2 - 25.0) / 25.0
    }

    /// Detect when crossing a Poincaré section (rare event trigger)
    /// Returns true when z crosses the midpoint value
    pub fn rare_event_trigger(&self, threshold_z: f64) -> bool {
        self.state.2 > threshold_z
    }
}

/// Ontological Oscillator: Chaotic generator for rare events
/// This is the "chispa disparadora" (triggering spark) the user described
#[derive(Debug, Clone)]
pub struct OntologicalOscillator {
    /// Underlying chaotic system
    pub lorenz: LorenzAttractor,
    /// Threshold for rare event triggering (3-5% of time above this)
    pub rare_threshold: f64,
    /// History of output values (for statistics)
    history: Vec<f64>,
    /// Maximum history size
    max_history: usize,
    /// Event count
    pub event_count: usize,
    /// Total samples
    pub total_samples: usize,
}

impl OntologicalOscillator {
    pub fn new() -> Self {
        // Calibrated so ~3-5% of samples exceed threshold
        // For Lorenz, z > 40 is approximately 4% of the time
        Self {
            lorenz: LorenzAttractor::new(),
            rare_threshold: 40.0,
            history: Vec::with_capacity(1000),
            max_history: 1000,
            event_count: 0,
            total_samples: 0,
        }
    }

    /// Calibrate threshold for desired rare event probability
    pub fn calibrate(&mut self, target_probability: f64, n_samples: usize) {
        let mut samples: Vec<f64> = Vec::with_capacity(n_samples);

        // Burn-in
        for _ in 0..1000 {
            self.lorenz.step(0.01);
        }

        // Collect samples
        for _ in 0..n_samples {
            self.lorenz.step(0.01);
            samples.push(self.lorenz.state.2);
        }

        // Sort to find threshold
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let idx = ((1.0 - target_probability) * n_samples as f64) as usize;
        if idx < samples.len() {
            self.rare_threshold = samples[idx];
        }

        // Reset counters after calibration
        self.event_count = 0;
        self.total_samples = 0;
        self.history.clear();
    }

    /// Step the oscillator
    pub fn step(&mut self, dt: f64) {
        self.lorenz.step(dt);

        // Track statistics
        self.total_samples += 1;
        if self.is_rare_event() {
            self.event_count += 1;
        }

        // Track history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(self.lorenz.state.2);
    }

    /// Is current state a rare event?
    pub fn is_rare_event(&self) -> bool {
        self.lorenz.state.2 > self.rare_threshold
    }

    /// Get the magnitude of the rare event (for scaling effects)
    pub fn rare_event_magnitude(&self) -> f64 {
        if self.is_rare_event() {
            (self.lorenz.state.2 - self.rare_threshold) / 10.0
        } else {
            0.0
        }
    }

    /// Actual rare event frequency from history
    pub fn observed_frequency(&self) -> f64 {
        if self.total_samples == 0 {
            0.0
        } else {
            self.event_count as f64 / self.total_samples as f64
        }
    }

    /// Generate perturbation value (for modulating drug response)
    /// Returns: (normal_variation, rare_event_perturbation)
    pub fn perturbation(&self) -> (f64, f64) {
        let normal = self.lorenz.normalized_output() * 0.1;  // ±10% variation
        let rare = self.rare_event_magnitude();

        (normal, rare)
    }
}

impl Default for OntologicalOscillator {
    fn default() -> Self {
        Self::new()
    }
}

/// Lévy flight generator for heavy-tailed distributions
/// Captures rare but large jumps (idiosyncratic reactions)
#[derive(Debug, Clone)]
pub struct LevyFlightGenerator {
    /// Stability parameter (0 < α ≤ 2, α=2 is Gaussian)
    pub alpha: f64,
    /// Scale parameter
    pub scale: f64,
    /// Current position
    pub position: f64,
}

impl LevyFlightGenerator {
    pub fn new(alpha: f64, scale: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.5, 2.0),
            scale,
            position: 0.0,
        }
    }

    /// Generate a Lévy-distributed random step
    /// Uses Chambers-Mallows-Stuck method
    pub fn step(&mut self) -> f64 {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);

        let v = uniform.sample(&mut rng);
        let w = -rng.gen::<f64>().ln();  // Exponential(1)

        let step = if (self.alpha - 1.0).abs() < 0.01 {
            // Cauchy case (α = 1)
            v.tan()
        } else if (self.alpha - 2.0).abs() < 0.01 {
            // Gaussian case (α = 2)
            let normal = Normal::new(0.0, 1.0).unwrap();
            normal.sample(&mut rng)
        } else {
            // General case
            let zeta = -(std::f64::consts::FRAC_PI_2 * self.alpha / 2.0).tan();
            let term1 = (self.alpha * v).sin()
                / (v.cos()).powf(1.0 / self.alpha);
            let term2 = (v - self.alpha * v).cos() / w;
            term1 * term2.powf((1.0 - self.alpha) / self.alpha) - zeta
        };

        self.position += self.scale * step;
        self.scale * step
    }

    /// Probability of a jump larger than threshold
    pub fn tail_probability(&self, threshold: f64) -> f64 {
        // For Lévy stable: P(|X| > x) ~ x^(-α) for large x
        (threshold / self.scale).powf(-self.alpha)
    }
}

/// Neural threshold with stochastic resonance
#[derive(Debug, Clone)]
pub struct StochasticThreshold {
    /// Baseline threshold
    pub baseline: f64,
    /// Current effective threshold (fluctuating)
    pub current: f64,
    /// Noise intensity
    pub noise_sigma: f64,
    /// Mean-reversion rate
    pub kappa: f64,
    /// Chaotic modulator
    pub chaos: OntologicalOscillator,
    /// Enable chaotic modulation
    pub use_chaos: bool,
}

impl StochasticThreshold {
    pub fn new(baseline: f64, noise_sigma: f64) -> Self {
        Self {
            baseline,
            current: baseline,
            noise_sigma,
            kappa: 1.0,  // Mean-reversion rate
            chaos: OntologicalOscillator::new(),
            use_chaos: true,
        }
    }

    /// Update threshold (Ornstein-Uhlenbeck with chaotic modulation)
    pub fn step(&mut self, dt: f64) {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        // OU process: dθ = κ(μ - θ)dt + σdW
        let mean_reversion = self.kappa * (self.baseline - self.current);
        let noise = self.noise_sigma * normal.sample(&mut rng) * dt.sqrt();

        self.current += mean_reversion * dt + noise;

        // Apply chaotic modulation for rare events
        if self.use_chaos {
            self.chaos.step(dt);
            let (_, rare) = self.chaos.perturbation();

            if rare > 0.0 {
                // Rare event: temporarily lower threshold (increased sensitivity)
                // or raise threshold (decreased sensitivity)
                let direction = if rng.gen::<bool>() { 1.0 } else { -1.0 };
                self.current -= direction * rare * self.baseline * 0.2;
            }
        }

        // Ensure threshold stays positive
        self.current = self.current.max(0.1 * self.baseline);
    }

    /// Check if a signal crosses the current threshold
    pub fn is_crossed(&self, signal: f64) -> bool {
        signal > self.current
    }

    /// Get probability of crossing given signal
    pub fn crossing_probability(&self, signal: f64) -> f64 {
        // Approximate as Gaussian
        let normal = Normal::new(self.baseline, self.noise_sigma).unwrap();
        // CDF gives P(threshold < signal)
        let z = (signal - self.baseline) / self.noise_sigma;
        0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
    }
}

/// Error function approximation
fn erf(x: f64) -> f64 {
    // Horner form approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Pharmacodynamic response with stochastic resonance
#[derive(Debug, Clone)]
pub struct StochasticPDResponse {
    /// EC50 with stochastic modulation
    pub stochastic_ec50: StochasticThreshold,
    /// Hill coefficient
    pub hill: f64,
    /// Maximum effect
    pub emax: f64,
    /// Ontological oscillator for rare events
    pub oscillator: OntologicalOscillator,
}

impl StochasticPDResponse {
    pub fn new(ec50: f64, hill: f64, emax: f64) -> Self {
        Self {
            stochastic_ec50: StochasticThreshold::new(ec50, ec50 * 0.1),
            hill,
            emax,
            oscillator: OntologicalOscillator::new(),
        }
    }

    /// Calculate response with stochastic modulation
    pub fn response(&mut self, concentration: f64, dt: f64) -> PDResponseResult {
        // Update stochastic components
        self.stochastic_ec50.step(dt);
        self.oscillator.step(dt);

        // Current effective EC50
        let ec50 = self.stochastic_ec50.current;

        // Base Hill equation
        let c_n = concentration.powf(self.hill);
        let ec50_n = ec50.powf(self.hill);
        let base_effect = self.emax * c_n / (ec50_n + c_n);

        // Rare event modulation
        let (normal_var, rare_event) = self.oscillator.perturbation();
        let modulated_effect = base_effect * (1.0 + normal_var);

        // Rare adverse event
        let adverse_event = if rare_event > 0.5 {
            Some(AdverseEvent {
                severity: rare_event,
                event_type: "Idiosyncratic reaction".to_string(),
            })
        } else {
            None
        };

        PDResponseResult {
            base_effect,
            modulated_effect,
            effective_ec50: ec50,
            adverse_event,
            is_rare_event: self.oscillator.is_rare_event(),
        }
    }
}

/// Result of stochastic PD calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PDResponseResult {
    /// Effect without stochastic modulation
    pub base_effect: f64,
    /// Effect with stochastic modulation
    pub modulated_effect: f64,
    /// Current effective EC50
    pub effective_ec50: f64,
    /// Adverse event if triggered
    pub adverse_event: Option<AdverseEvent>,
    /// Whether this was a rare event moment
    pub is_rare_event: bool,
}

/// Adverse event triggered by rare dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEvent {
    /// Severity (0-1 scale)
    pub severity: f64,
    /// Type of event
    pub event_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lorenz_attractor() {
        let mut lorenz = LorenzAttractor::new();

        // Run for a while
        for _ in 0..10000 {
            lorenz.step(0.01);
        }

        // Should stay bounded (strange attractor)
        assert!(lorenz.state.0.abs() < 50.0);
        assert!(lorenz.state.1.abs() < 50.0);
        assert!(lorenz.state.2 < 60.0 && lorenz.state.2 > 0.0);
    }

    #[test]
    fn test_ontological_oscillator_frequency() {
        let mut osc = OntologicalOscillator::new();

        // Calibrate for ~4% rare events
        osc.calibrate(0.04, 10000);

        // Run and check frequency
        for _ in 0..100000 {
            osc.step(0.01);
        }

        let freq = osc.observed_frequency();
        // Should be close to 4% (allow some variance)
        assert!(freq > 0.02 && freq < 0.08,
                "Expected ~4% rare events, got {:.1}%", freq * 100.0);
    }

    #[test]
    fn test_levy_heavy_tails() {
        let mut levy = LevyFlightGenerator::new(1.5, 1.0);  // Heavy tailed

        let n_samples = 10000;
        let mut large_jumps = 0;

        for _ in 0..n_samples {
            let step = levy.step();
            if step.abs() > 3.0 {  // 3 sigma for Gaussian would be rare
                large_jumps += 1;
            }
        }

        // Should have significantly more large jumps than Gaussian
        let ratio = large_jumps as f64 / n_samples as f64;
        assert!(ratio > 0.01, "Expected heavy tails, got {}% large jumps", ratio * 100.0);
    }

    #[test]
    fn test_stochastic_resonance() {
        let params = StochasticResonanceParams::default();
        let mut system = BistableSystem::new(params);

        let mut crossings = 0;
        let n_steps = 100000;

        for _ in 0..n_steps {
            let was_positive = system.crossed_threshold();
            system.step(0.001);
            let is_positive = system.crossed_threshold();

            if !was_positive && is_positive {
                crossings += 1;
            }
        }

        // Should have some crossings with appropriate noise
        assert!(crossings > 0, "Expected threshold crossings with stochastic resonance");
    }
}
