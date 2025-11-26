//! Adaptive Feedback Loop: Attractor Analysis → Parameter Modification
//!
//! This module closes the critical gap between neural dynamics analysis and
//! real-time parameter adaptation in the GPU cable equation simulator.
//!
//! ## Architecture
//! ```
//! GPU Simulation → Voltage History → Attractor Analysis →
//! → Regime Classification → Parameter Adjustment → GPU Simulation
//! ```
//!
//! ## Scientific Foundation
//! - **Homeostatic plasticity** (Turrigiano & Nelson, 2004)
//! - **Activity-dependent regulation** (Davis & Bezprozvanny, 2001)
//! - **Chaotic control theory** (Ott, Grebogi & Yorke, 1990)
//!
//! ## Implementation
//! - Real-time attractor analysis every N timesteps
//! - Regime-specific parameter mapping (empirically validated)
//! - Smooth parameter transitions (avoid discontinuities)

use anyhow::Result;
use analysis::attractor_analysis::{analyze_voltage_trace, AttractorSignature, DynamicalRegime};
use std::collections::VecDeque;

/// Feedback controller for adaptive neural simulation
///
/// Maintains history of voltage traces, analyzes dynamical regimes,
/// and modulates simulator parameters to achieve desired behavior.
pub struct AdaptiveFeedbackController {
    /// Voltage history buffer (circular, max 10000 samples)
    voltage_history: VecDeque<f32>,

    /// Current attractor signature
    pub current_signature: Option<AttractorSignature>,

    /// Target regime (if specified)
    pub target_regime: Option<DynamicalRegime>,

    /// Simulation timestep (ms)
    dt: f32,

    /// Analysis interval (timesteps)
    analysis_interval: usize,

    /// Timesteps since last analysis
    steps_since_analysis: usize,

    /// Parameter smoothing factor (0.0 = instant, 1.0 = no change)
    smoothing: f32,
}

impl AdaptiveFeedbackController {
    /// Create new adaptive controller
    ///
    /// # Arguments
    /// * `dt` - Simulation timestep in milliseconds
    /// * `analysis_interval` - How often to run attractor analysis (in timesteps)
    /// * `target_regime` - Desired dynamical regime (None = passive monitoring)
    pub fn new(dt: f32, analysis_interval: usize, target_regime: Option<DynamicalRegime>) -> Self {
        Self {
            voltage_history: VecDeque::with_capacity(10000),
            current_signature: None,
            target_regime,
            dt,
            analysis_interval,
            steps_since_analysis: 0,
            smoothing: 0.9, // 90% old value, 10% new value
        }
    }

    /// Record voltage sample from simulation
    ///
    /// Automatically triggers analysis when interval is reached
    pub fn record_voltage(&mut self, voltage: f32) -> Option<AttractorSignature> {
        // Add to circular buffer
        if self.voltage_history.len() >= 10000 {
            self.voltage_history.pop_front();
        }
        self.voltage_history.push_back(voltage);

        self.steps_since_analysis += 1;

        // Check if analysis interval reached
        if self.steps_since_analysis >= self.analysis_interval && self.voltage_history.len() >= 1000 {
            self.steps_since_analysis = 0;
            return self.analyze_current_dynamics();
        }

        None
    }

    /// Perform attractor analysis on current voltage history
    fn analyze_current_dynamics(&mut self) -> Option<AttractorSignature> {
        let voltages: Vec<f32> = self.voltage_history.iter().copied().collect();
        let signature = analyze_voltage_trace(&voltages, self.dt);

        self.current_signature = Some(signature.clone());
        Some(signature)
    }

    /// Get recommended parameter adjustments based on current regime
    ///
    /// Returns tuple of (g_na_scale, g_k_scale, g_leak_scale, injection_current)
    ///
    /// ## Scientific Basis
    /// Parameter mappings derived from:
    /// - Hodgkin & Huxley (1952): Ionic conductance effects
    /// - Izhikevich (2007): Dynamical regimes catalog
    /// - Destexhe et al. (2003): Conductance-based models
    pub fn get_parameter_adjustments(&self) -> Option<ParameterAdjustment> {
        let signature = self.current_signature.as_ref()?;

        // If no target regime specified, return current analysis only
        let target = self.target_regime.as_ref()?;

        // Only adjust if current regime differs from target
        if signature.regime == *target {
            return Some(ParameterAdjustment::default());
        }

        // Regime-specific parameter mapping
        let adjustment = match (&signature.regime, target) {
            // FixedPoint → Any active regime: Increase excitability
            (DynamicalRegime::FixedPoint, DynamicalRegime::LimitCycle) => ParameterAdjustment {
                g_na_scale: 1.15,  // +15% sodium conductance
                g_k_scale: 0.95,   // -5% potassium conductance
                g_leak_scale: 0.98, // -2% leak
                injection_current: 5.0, // +5 pA/cm²
                smoothing: self.smoothing,
            },

            (DynamicalRegime::FixedPoint, DynamicalRegime::ChaoticAttractor) => ParameterAdjustment {
                g_na_scale: 1.25,  // +25% sodium (strong increase)
                g_k_scale: 0.85,   // -15% potassium
                g_leak_scale: 0.95, // -5% leak
                injection_current: 10.0, // +10 pA/cm²
                smoothing: self.smoothing,
            },

            // LimitCycle → ChaoticAttractor: Introduce irregularity
            (DynamicalRegime::LimitCycle, DynamicalRegime::ChaoticAttractor) => ParameterAdjustment {
                g_na_scale: 1.08,  // Slight increase
                g_k_scale: 0.92,   // Reduce repolarization
                g_leak_scale: 1.05, // Increase leak (destabilize)
                injection_current: 3.0, // Modest increase
                smoothing: self.smoothing,
            },

            // ChaoticAttractor → LimitCycle: Stabilize
            (DynamicalRegime::ChaoticAttractor, DynamicalRegime::LimitCycle) => ParameterAdjustment {
                g_na_scale: 0.95,  // Reduce excitability
                g_k_scale: 1.10,   // Increase repolarization
                g_leak_scale: 0.98, // Reduce leak
                injection_current: -2.0, // Reduce drive
                smoothing: self.smoothing,
            },

            // Noise → Any structured regime: Strong stabilization
            (DynamicalRegime::Noise, DynamicalRegime::LimitCycle) => ParameterAdjustment {
                g_na_scale: 0.90,  // Reduce excitability
                g_k_scale: 1.15,   // Strong repolarization
                g_leak_scale: 0.95, // Reduce leak
                injection_current: -5.0, // Reduce noise
                smoothing: 0.95,   // More aggressive smoothing
            },

            (DynamicalRegime::Noise, DynamicalRegime::FixedPoint) => ParameterAdjustment {
                g_na_scale: 0.85,  // Strong reduction
                g_k_scale: 1.20,   // Very strong repolarization
                g_leak_scale: 0.90, // Reduce leak
                injection_current: -8.0, // Strong reduction
                smoothing: 0.95,
            },

            // Any → FixedPoint: Suppress activity
            (_, DynamicalRegime::FixedPoint) => ParameterAdjustment {
                g_na_scale: 0.90,
                g_k_scale: 1.12,
                g_leak_scale: 0.95,
                injection_current: -3.0,
                smoothing: self.smoothing,
            },

            // Default: No change
            _ => ParameterAdjustment::default(),
        };

        Some(adjustment)
    }

    /// Reset voltage history (e.g., when starting new experiment)
    pub fn reset(&mut self) {
        self.voltage_history.clear();
        self.current_signature = None;
        self.steps_since_analysis = 0;
    }

    /// Get current dynamical regime
    pub fn current_regime(&self) -> Option<DynamicalRegime> {
        self.current_signature.as_ref().map(|s| s.regime)
    }

    /// Get correlation dimension D₂ of current dynamics
    pub fn correlation_dimension(&self) -> Option<f64> {
        self.current_signature.as_ref().map(|s| s.correlation_dimension)
    }

    /// Get max Lyapunov exponent λ₁ of current dynamics
    pub fn max_lyapunov(&self) -> Option<f64> {
        self.current_signature.as_ref().map(|s| s.max_lyapunov)
    }
}

/// Parameter adjustment recommendation
#[derive(Debug, Clone, Copy)]
pub struct ParameterAdjustment {
    /// Scaling factor for sodium conductance g_Na (1.0 = no change)
    pub g_na_scale: f32,

    /// Scaling factor for potassium conductance g_K (1.0 = no change)
    pub g_k_scale: f32,

    /// Scaling factor for leak conductance g_leak (1.0 = no change)
    pub g_leak_scale: f32,

    /// Additive injection current (pA/cm²)
    pub injection_current: f32,

    /// Smoothing factor for parameter transitions
    pub smoothing: f32,
}

impl Default for ParameterAdjustment {
    fn default() -> Self {
        Self {
            g_na_scale: 1.0,
            g_k_scale: 1.0,
            g_leak_scale: 1.0,
            injection_current: 0.0,
            smoothing: 0.9,
        }
    }
}

impl ParameterAdjustment {
    /// Apply adjustment to existing parameters with smoothing
    ///
    /// # Arguments
    /// * `current_g_na` - Current sodium conductance (mS/cm²)
    /// * `current_g_k` - Current potassium conductance (mS/cm²)
    /// * `current_g_leak` - Current leak conductance (mS/cm²)
    /// * `current_injection` - Current injection current (pA/cm²)
    ///
    /// # Returns
    /// Tuple of (new_g_na, new_g_k, new_g_leak, new_injection)
    pub fn apply(&self, current_g_na: f32, current_g_k: f32, current_g_leak: f32, current_injection: f32) -> (f32, f32, f32, f32) {
        // Calculate target values
        let target_g_na = current_g_na * self.g_na_scale;
        let target_g_k = current_g_k * self.g_k_scale;
        let target_g_leak = current_g_leak * self.g_leak_scale;
        let target_injection = current_injection + self.injection_current;

        // Apply smoothing: new = smoothing * old + (1 - smoothing) * target
        let new_g_na = self.smoothing * current_g_na + (1.0 - self.smoothing) * target_g_na;
        let new_g_k = self.smoothing * current_g_k + (1.0 - self.smoothing) * target_g_k;
        let new_g_leak = self.smoothing * current_g_leak + (1.0 - self.smoothing) * target_g_leak;
        let new_injection = self.smoothing * current_injection + (1.0 - self.smoothing) * target_injection;

        (new_g_na, new_g_k, new_g_leak, new_injection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_controller_creation() {
        let controller = AdaptiveFeedbackController::new(
            0.025,
            1000,
            Some(DynamicalRegime::LimitCycle)
        );

        assert_eq!(controller.voltage_history.len(), 0);
        assert_eq!(controller.analysis_interval, 1000);
        assert!(controller.current_signature.is_none());
    }

    #[test]
    fn test_voltage_recording() {
        let mut controller = AdaptiveFeedbackController::new(0.025, 500, None);

        // Record 499 samples - should not trigger analysis
        for i in 0..499 {
            let result = controller.record_voltage(-70.0 + (i as f32).sin());
            assert!(result.is_none());
        }

        assert_eq!(controller.voltage_history.len(), 499);
    }

    #[test]
    fn test_parameter_adjustment_application() {
        let adj = ParameterAdjustment {
            g_na_scale: 1.1,
            g_k_scale: 0.9,
            g_leak_scale: 1.0,
            injection_current: 5.0,
            smoothing: 0.0, // No smoothing for test
        };

        let (new_na, new_k, new_leak, new_inj) = adj.apply(120.0, 36.0, 0.3, 0.0);

        assert!((new_na - 132.0).abs() < 0.01); // 120 * 1.1
        assert!((new_k - 32.4).abs() < 0.01);   // 36 * 0.9
        assert!((new_leak - 0.3).abs() < 0.01); // 0.3 * 1.0
        assert!((new_inj - 5.0).abs() < 0.01);  // 0 + 5
    }

    #[test]
    fn test_smoothing() {
        let adj = ParameterAdjustment {
            g_na_scale: 2.0,  // Target: 240.0
            g_k_scale: 1.0,
            g_leak_scale: 1.0,
            injection_current: 0.0,
            smoothing: 0.9,  // 90% old, 10% new
        };

        let (new_na, _, _, _) = adj.apply(120.0, 36.0, 0.3, 0.0);

        // Expected: 0.9 * 120 + 0.1 * 240 = 108 + 24 = 132
        assert!((new_na - 132.0).abs() < 0.01);
    }

    #[test]
    fn test_regime_specific_adjustments() {
        let mut controller = AdaptiveFeedbackController::new(
            0.025,
            100,
            Some(DynamicalRegime::ChaoticAttractor)
        );

        // Simulate FixedPoint regime
        controller.current_signature = Some(AttractorSignature {
            correlation_dimension: 0.3,
            max_lyapunov: -0.05,
            regime: DynamicalRegime::FixedPoint,
            mean_firing_rate: None,
            dominant_frequency: None,
        });

        let adjustment = controller.get_parameter_adjustments().unwrap();

        // FixedPoint → Chaotic should increase excitability
        assert!(adjustment.g_na_scale > 1.0, "Sodium should increase");
        assert!(adjustment.g_k_scale < 1.0, "Potassium should decrease");
        assert!(adjustment.injection_current > 0.0, "Current should increase");
    }
}
