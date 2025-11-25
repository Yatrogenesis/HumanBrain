//! Ion channel models for realistic neuronal dynamics.
//!
//! This module implements various voltage-gated and ligand-gated ion channels
//! including Hodgkin-Huxley type channels, calcium channels, and NMDA receptors.

use serde::{Deserialize, Serialize};

/// Trait for ion channels
pub trait IonChannel {
    /// Calculate channel conductance given voltage and state variables
    fn conductance(&self, voltage: f64, state: &ChannelState) -> f64;

    /// Update channel state variables
    fn update_state(&self, voltage: f64, state: &mut ChannelState, dt: f64);

    /// Reversal potential (mV)
    fn reversal_potential(&self) -> f64;
}

/// Channel state variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub m: f64,  // Activation variable
    pub h: f64,  // Inactivation variable
    pub n: f64,  // Secondary activation (for K+ channels)
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            m: 0.05,
            h: 0.6,
            n: 0.32,
        }
    }
}

/// Hodgkin-Huxley Sodium Channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HodgkinHuxleyNa {
    /// Maximum conductance (nS)
    pub g_max: f64,
    /// Reversal potential (mV)
    pub e_na: f64,
}

impl HodgkinHuxleyNa {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_na: 50.0,
        }
    }

    fn alpha_m(&self, v: f64) -> f64 {
        0.1 * (v + 40.0) / (1.0 - (-0.1 * (v + 40.0)).exp())
    }

    fn beta_m(&self, v: f64) -> f64 {
        4.0 * (-0.0556 * (v + 65.0)).exp()
    }

    fn alpha_h(&self, v: f64) -> f64 {
        0.07 * (-0.05 * (v + 65.0)).exp()
    }

    fn beta_h(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-0.1 * (v + 35.0)).exp())
    }
}

impl IonChannel for HodgkinHuxleyNa {
    fn conductance(&self, _voltage: f64, state: &ChannelState) -> f64 {
        self.g_max * state.m.powi(3) * state.h
    }

    fn update_state(&self, voltage: f64, state: &mut ChannelState, dt: f64) {
        let am = self.alpha_m(voltage);
        let bm = self.beta_m(voltage);
        let ah = self.alpha_h(voltage);
        let bh = self.beta_h(voltage);

        state.m += (am * (1.0 - state.m) - bm * state.m) * dt;
        state.h += (ah * (1.0 - state.h) - bh * state.h) * dt;
    }

    fn reversal_potential(&self) -> f64 {
        self.e_na
    }
}

/// Hodgkin-Huxley Potassium Channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HodgkinHuxleyK {
    /// Maximum conductance (nS)
    pub g_max: f64,
    /// Reversal potential (mV)
    pub e_k: f64,
}

impl HodgkinHuxleyK {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_k: -90.0,
        }
    }

    fn alpha_n(&self, v: f64) -> f64 {
        0.01 * (v + 55.0) / (1.0 - (-0.1 * (v + 55.0)).exp())
    }

    fn beta_n(&self, v: f64) -> f64 {
        0.125 * (-0.0125 * (v + 65.0)).exp()
    }
}

impl IonChannel for HodgkinHuxleyK {
    fn conductance(&self, _voltage: f64, state: &ChannelState) -> f64 {
        self.g_max * state.n.powi(4)
    }

    fn update_state(&self, voltage: f64, state: &mut ChannelState, dt: f64) {
        let an = self.alpha_n(voltage);
        let bn = self.beta_n(voltage);

        state.n += (an * (1.0 - state.n) - bn * state.n) * dt;
    }

    fn reversal_potential(&self) -> f64 {
        self.e_k
    }
}

/// Calcium Channel (L-type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalciumChannel {
    /// Maximum conductance (nS)
    pub g_max: f64,
    /// Reversal potential (mV) - depends on Ca2+ concentration
    pub e_ca: f64,
}

impl CalciumChannel {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_ca: 120.0,
        }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-0.15 * (v + 20.0)).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (0.2 * (v + 50.0)).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        0.5  // ms
    }

    fn tau_h(&self, _v: f64) -> f64 {
        20.0  // ms
    }
}

impl IonChannel for CalciumChannel {
    fn conductance(&self, _voltage: f64, state: &ChannelState) -> f64 {
        self.g_max * state.m.powi(2) * state.h
    }

    fn update_state(&self, voltage: f64, state: &mut ChannelState, dt: f64) {
        let m_inf = self.m_inf(voltage);
        let h_inf = self.h_inf(voltage);
        let tau_m = self.tau_m(voltage);
        let tau_h = self.tau_h(voltage);

        state.m += ((m_inf - state.m) / tau_m) * dt;
        state.h += ((h_inf - state.h) / tau_h) * dt;
    }

    fn reversal_potential(&self) -> f64 {
        self.e_ca
    }
}

/// NMDA receptor channel
///
/// Voltage-dependent Mg2+ block and glutamate binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NMDAChannel {
    /// Maximum conductance (nS)
    pub g_max: f64,
    /// Reversal potential (mV)
    pub e_nmda: f64,
    /// Mg2+ concentration (mM)
    pub mg_concentration: f64,
}

impl NMDAChannel {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_nmda: 0.0,  // Non-selective cation channel
            mg_concentration: 1.0,  // Typical extracellular Mg2+
        }
    }

    /// Voltage-dependent Mg2+ block
    fn mg_block(&self, v: f64) -> f64 {
        1.0 / (1.0 + (self.mg_concentration / 3.57) * (-0.062 * v).exp())
    }

    /// Glutamate binding dynamics (simplified)
    fn glu_binding(&self, state: &ChannelState) -> f64 {
        state.m  // Assumes m represents glutamate binding
    }
}

impl IonChannel for NMDAChannel {
    fn conductance(&self, voltage: f64, state: &ChannelState) -> f64 {
        let mg_block = self.mg_block(voltage);
        let glu_bound = self.glu_binding(state);
        self.g_max * glu_bound * mg_block
    }

    fn update_state(&self, _voltage: f64, state: &mut ChannelState, dt: f64) {
        // Glutamate binding kinetics
        let alpha = 0.5;  // Binding rate
        let beta = 0.05;  // Unbinding rate

        state.m += (alpha * (1.0 - state.m) - beta * state.m) * dt;
    }

    fn reversal_potential(&self) -> f64 {
        self.e_nmda
    }
}

/// A-type Potassium Channel (fast inactivating)
///
/// Important for dendritic excitability and backpropagating action potentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KAChannel {
    pub g_max: f64,
    pub e_k: f64,
}

impl KAChannel {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_k: -90.0,
        }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((-0.143 * (v + 50.0)).exp()))
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((0.111 * (v + 80.0)).exp()))
    }

    fn tau_m(&self, v: f64) -> f64 {
        0.34 + 0.92 * ((-0.091 * (v + 66.0)).exp())
    }

    fn tau_h(&self, v: f64) -> f64 {
        8.0 + 49.0 / (1.0 + ((0.1 * (v + 70.0)).exp()))
    }
}

impl IonChannel for KAChannel {
    fn conductance(&self, _voltage: f64, state: &ChannelState) -> f64 {
        self.g_max * state.m.powi(3) * state.h
    }

    fn update_state(&self, voltage: f64, state: &mut ChannelState, dt: f64) {
        let m_inf = self.m_inf(voltage);
        let h_inf = self.h_inf(voltage);
        let tau_m = self.tau_m(voltage);
        let tau_h = self.tau_h(voltage);

        state.m += ((m_inf - state.m) / tau_m) * dt;
        state.h += ((h_inf - state.h) / tau_h) * dt;
    }

    fn reversal_potential(&self) -> f64 {
        self.e_k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_na_channel() {
        let channel = HodgkinHuxleyNa::new(120.0);
        let mut state = ChannelState::default();

        // Test conductance calculation
        let g = channel.conductance(-65.0, &state);
        assert!(g >= 0.0);

        // Test state update
        channel.update_state(-65.0, &mut state, 0.01);
        assert!(state.m >= 0.0 && state.m <= 1.0);
        assert!(state.h >= 0.0 && state.h <= 1.0);
    }

    #[test]
    fn test_nmda_mg_block() {
        let channel = NMDAChannel::new(1.0);

        // At rest, Mg2+ block should be strong
        let block_rest = channel.mg_block(-70.0);
        assert!(block_rest < 0.1);

        // At depolarized potentials, Mg2+ block should be relieved
        let block_depol = channel.mg_block(0.0);
        assert!(block_depol > 0.5);
    }
}
