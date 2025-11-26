//! Attractor Analysis for Neural Time Series
//!
//! Adapts chaotic attractor compression algorithms for neural dynamics:
//! - Input: Voltage traces or spike trains from GPU simulation
//! - Output: Dynamic signature (D₂, λ₁, regime classification)
//!
//! Based on PP25-CHAOTIC_ATTRACTOR_COMPRESSION algorithms

use serde::{Deserialize, Serialize};

/// Dynamical regime classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DynamicalRegime {
    /// Fixed point (quiescent)
    FixedPoint,
    /// Limit cycle (periodic spiking)
    LimitCycle,
    /// Quasiperiodic (multiple frequencies)
    Quasiperiodic,
    /// Chaotic attractor (irregular but deterministic)
    ChaoticAttractor,
    /// High-dimensional noise
    Noise,
}

/// Attractor signature extracted from neural activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorSignature {
    /// Correlation dimension D₂ (Grassberger-Procaccia)
    pub correlation_dimension: f64,

    /// Max Lyapunov exponent λ₁
    pub max_lyapunov: f64,

    /// Classified dynamical regime
    pub regime: DynamicalRegime,

    /// Mean firing rate (Hz) if spike train
    pub mean_firing_rate: Option<f64>,

    /// Dominant frequency (Hz) if periodic
    pub dominant_frequency: Option<f64>,
}

impl AttractorSignature {
    /// Classify dynamical regime based on D₂ and λ₁
    pub fn classify_regime(d2: f64, lambda: f64) -> DynamicalRegime {
        if d2 < 0.5 {
            DynamicalRegime::FixedPoint
        } else if lambda < -0.01 {
            // Negative Lyapunov → stable periodic
            if d2 < 1.5 {
                DynamicalRegime::LimitCycle
            } else {
                DynamicalRegime::Quasiperiodic
            }
        } else if lambda > 0.01 && d2 < 5.0 {
            // Positive Lyapunov + low dimension → chaos
            DynamicalRegime::ChaoticAttractor
        } else {
            // High dimension + unstable → noise
            DynamicalRegime::Noise
        }
    }
}

/// Analyze voltage trace from neural simulation
pub fn analyze_voltage_trace(voltages: &[f32], dt: f32) -> AttractorSignature {
    println!("🔬 Analyzing voltage trace ({} samples, dt={:.2}ms)", voltages.len(), dt);

    let d2 = correlation_dimension(voltages);
    let lambda = max_lyapunov_exponent(voltages, dt);
    let regime = AttractorSignature::classify_regime(d2, lambda);

    println!("   D₂ = {:.4}, λ₁ = {:.4}, Regime: {:?}", d2, lambda, regime);

    AttractorSignature {
        correlation_dimension: d2,
        max_lyapunov: lambda,
        regime,
        mean_firing_rate: None,
        dominant_frequency: estimate_dominant_frequency(voltages, dt),
    }
}

/// Analyze spike train (array of spike times in ms)
pub fn analyze_spike_train(spike_times: &[f32], duration_ms: f32) -> AttractorSignature {
    // Convert spike train to ISI (inter-spike intervals)
    let mut isis: Vec<f32> = Vec::new();
    for i in 1..spike_times.len() {
        isis.push(spike_times[i] - spike_times[i-1]);
    }

    let mean_rate = (spike_times.len() as f64) / (duration_ms as f64 / 1000.0);

    if isis.len() < 50 {
        // Not enough spikes for analysis
        return AttractorSignature {
            correlation_dimension: 0.0,
            max_lyapunov: 0.0,
            regime: DynamicalRegime::FixedPoint,
            mean_firing_rate: Some(mean_rate),
            dominant_frequency: None,
        };
    }

    let d2 = correlation_dimension(&isis);
    let lambda = max_lyapunov_exponent(&isis, 1.0); // ISIs are in ms
    let regime = AttractorSignature::classify_regime(d2, lambda);

    AttractorSignature {
        correlation_dimension: d2,
        max_lyapunov: lambda,
        regime,
        mean_firing_rate: Some(mean_rate),
        dominant_frequency: None,
    }
}

/// Correlation Dimension D₂ (Grassberger-Procaccia algorithm)
///
/// Adapted from PP25-CHAOTIC_ATTRACTOR_COMPRESSION/code/src/attractor_analysis.rs
pub fn correlation_dimension(time_series: &[f32]) -> f64 {
    let n = time_series.len().min(2000);
    if n < 100 {
        return time_series.len() as f64; // Not enough data
    }

    let series = &time_series[..n];

    // Calculate pairwise distances
    let mut distances = Vec::new();
    for i in 0..n {
        for j in (i+1)..n {
            let dist = (series[i] - series[j]).abs() as f64;
            distances.push(dist);
        }
    }

    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Logarithmic radii
    let min_r = distances[distances.len() / 100].max(1e-6);
    let max_r = distances[distances.len() * 99 / 100];

    let num_radii = 20;
    let mut radii = Vec::new();
    let mut correlation_sums = Vec::new();

    for i in 0..num_radii {
        let log_r = min_r.ln() + (max_r.ln() - min_r.ln()) * (i as f64) / (num_radii as f64 - 1.0);
        let r = log_r.exp();
        radii.push(r);

        let count = distances.iter().filter(|&&d| d < r).count();
        let c_r = (count as f64 / distances.len() as f64).max(1e-10);
        correlation_sums.push(c_r);
    }

    // Linear regression in log-log space
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_xy = 0.0;
    let n_points = num_radii as f64;

    for i in 0..num_radii {
        let x = radii[i].ln();
        let y = correlation_sums[i].ln();
        sum_x += x;
        sum_y += y;
        sum_xx += x * x;
        sum_xy += x * y;
    }

    // Slope = D₂
    let d2 = (n_points * sum_xy - sum_x * sum_y) / (n_points * sum_xx - sum_x * sum_x);
    d2.max(0.0)
}

/// Max Lyapunov Exponent λ₁
///
/// Measures exponential divergence of nearby trajectories
pub fn max_lyapunov_exponent(time_series: &[f32], dt: f32) -> f64 {
    let n = time_series.len().min(1000);
    if n < 100 {
        return 0.0;
    }

    let num_pairs = 50;
    let mut lyapunov_estimates = Vec::new();

    for i in (0..(n-20)).step_by(n / num_pairs) {
        // Find nearest neighbor
        let mut min_dist = f64::INFINITY;
        let mut nearest_idx = i + 1;

        for j in (i+1)..(n-20) {
            let dist = (time_series[i] - time_series[j]).abs() as f64;
            if dist > 1e-6 && dist < min_dist {
                min_dist = dist;
                nearest_idx = j;
            }
        }

        if min_dist == f64::INFINITY {
            continue;
        }

        // Track divergence over time
        let max_steps = 20.min(n - i.max(nearest_idx));
        let mut log_divergence_sum = 0.0;
        let mut valid_steps = 0;

        for t in 1..max_steps {
            if i + t >= n || nearest_idx + t >= n {
                break;
            }

            let d0 = min_dist;
            let dt_val = (time_series[i + t] - time_series[nearest_idx + t]).abs() as f64;

            if dt_val > 1e-6 && d0 > 1e-6 {
                log_divergence_sum += (dt_val / d0).ln();
                valid_steps += 1;
            }
        }

        if valid_steps > 0 {
            let lambda = log_divergence_sum / (valid_steps as f64 * dt as f64);
            lyapunov_estimates.push(lambda);
        }
    }

    if lyapunov_estimates.is_empty() {
        return 0.0;
    }

    lyapunov_estimates.iter().sum::<f64>() / lyapunov_estimates.len() as f64
}

/// Estimate dominant frequency via simple peak detection in autocorrelation
fn estimate_dominant_frequency(time_series: &[f32], dt: f32) -> Option<f64> {
    if time_series.len() < 100 {
        return None;
    }

    // Simple autocorrelation at lag 1-50
    let max_lag = 50.min(time_series.len() / 2);
    let mut autocorr: Vec<f64> = Vec::new();

    let mean: f64 = time_series.iter().map(|&x| x as f64).sum::<f64>() / time_series.len() as f64;

    for lag in 1..max_lag {
        let mut sum = 0.0;
        for i in 0..(time_series.len() - lag) {
            sum += ((time_series[i] as f64 - mean) * (time_series[i + lag] as f64 - mean));
        }
        autocorr.push(sum / (time_series.len() - lag) as f64);
    }

    // Find first peak
    for i in 1..(autocorr.len()-1) {
        if autocorr[i] > autocorr[i-1] && autocorr[i] > autocorr[i+1] && autocorr[i] > 0.0 {
            let period_ms = (i as f64) * (dt as f64);
            return Some(1000.0 / period_ms); // Hz
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_dimension() {
        // Low-dimensional sine wave
        let mut sine: Vec<f32> = Vec::new();
        for i in 0..1000 {
            sine.push((i as f32 * 0.1).sin());
        }

        let d2 = correlation_dimension(&sine);
        assert!(d2 < 2.0, "Sine wave should have D2 < 2");
    }

    #[test]
    fn test_lyapunov() {
        // Periodic signal should have negative Lyapunov
        let sine: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin()).collect();
        let lambda = max_lyapunov_exponent(&sine, 1.0);

        // Periodic signals have λ < 0 (converging)
        println!("Sine wave λ = {:.4}", lambda);
    }

    #[test]
    fn test_regime_classification() {
        let regime1 = AttractorSignature::classify_regime(0.3, -0.05);
        assert_eq!(regime1, DynamicalRegime::FixedPoint);

        let regime2 = AttractorSignature::classify_regime(1.2, -0.02);
        assert_eq!(regime2, DynamicalRegime::LimitCycle);

        let regime3 = AttractorSignature::classify_regime(2.4, 0.08);
        assert_eq!(regime3, DynamicalRegime::ChaoticAttractor);
    }
}
