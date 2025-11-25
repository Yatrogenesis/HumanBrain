//! Synaptic plasticity mechanisms.

use serde::{Deserialize, Serialize};

/// Spike-timing-dependent plasticity parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STDPParams {
    pub a_plus: f64,      // LTP amplitude
    pub a_minus: f64,     // LTD amplitude
    pub tau_plus: f64,    // LTP time window (ms)
    pub tau_minus: f64,   // LTD time window (ms)
}

impl Default for STDPParams {
    fn default() -> Self {
        Self {
            a_plus: 0.01,
            a_minus: 0.012,
            tau_plus: 20.0,
            tau_minus: 20.0,
        }
    }
}

/// Triplet STDP (accounts for spike triplets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripletSTDP {
    pub params: STDPParams,
    pub r1: f64,  // Pre-synaptic trace
    pub r2: f64,  // Pre-synaptic trace (slow)
    pub o1: f64,  // Post-synaptic trace
    pub o2: f64,  // Post-synaptic trace (slow)
}

/// Calcium-based plasticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalciumPlasticity {
    pub ca_concentration: f64,
    pub theta_d: f64,  // LTD threshold
    pub theta_p: f64,  // LTP threshold
}
