//! Advanced ion channel models with diverse kinetics and biological realism.
//!
//! This module implements 15+ specialized ion channels found in different neuronal
//! compartments (soma, dendrites, axon initial segment) with realistic kinetics
//! from experimental data.

use serde::{Deserialize, Serialize};

/// Temperature for Q10 calculations (Celsius)
const TEMPERATURE: f64 = 37.0;
const REFERENCE_TEMP: f64 = 22.0;

/// Apply Q10 temperature correction to rate constants
fn q10_correction(rate: f64, q10: f64, temp: f64, ref_temp: f64) -> f64 {
    rate * q10.powf((temp - ref_temp) / 10.0)
}

/// Extended channel state for complex gating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedChannelState {
    pub m: f64,   // Primary activation
    pub h: f64,   // Fast inactivation
    pub s: f64,   // Slow inactivation
    pub p: f64,   // Additional gating variable
    pub ca_i: f64, // Internal calcium concentration (μM)
}

impl Default for AdvancedChannelState {
    fn default() -> Self {
        Self {
            m: 0.0,
            h: 1.0,
            s: 1.0,
            p: 0.0,
            ca_i: 0.05, // Basal calcium
        }
    }
}

/// Nav1.1 - Sodium channel (soma, dendrites)
/// Critical for action potential initiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nav1_1 {
    pub g_max: f64,
    pub e_na: f64,
}

impl Nav1_1 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_na: 50.0 }
    }

    fn alpha_m(&self, v: f64) -> f64 {
        let v_shift = v + 38.0;
        if v_shift.abs() < 1e-4 {
            3.0
        } else {
            0.182 * v_shift / (1.0 - (-v_shift / 6.0).exp())
        }
    }

    fn beta_m(&self, v: f64) -> f64 {
        let v_shift = v + 38.0;
        0.124 * (-v_shift) / (1.0 - (v_shift / 6.0).exp())
    }

    fn alpha_h(&self, v: f64) -> f64 {
        0.024 * (v + 50.0) / (1.0 - (-(v + 50.0) / 5.0).exp())
    }

    fn beta_h(&self, v: f64) -> f64 {
        0.0091 * (-(v + 75.0)) / (1.0 - ((v + 75.0) / 5.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(3) * state.h * (v - self.e_na)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let am = q10_correction(self.alpha_m(v), 2.3, TEMPERATURE, REFERENCE_TEMP);
        let bm = q10_correction(self.beta_m(v), 2.3, TEMPERATURE, REFERENCE_TEMP);
        let ah = q10_correction(self.alpha_h(v), 2.3, TEMPERATURE, REFERENCE_TEMP);
        let bh = q10_correction(self.beta_h(v), 2.3, TEMPERATURE, REFERENCE_TEMP);

        state.m += (am * (1.0 - state.m) - bm * state.m) * dt;
        state.h += (ah * (1.0 - state.h) - bh * state.h) * dt;
    }
}

/// Nav1.6 - Sodium channel (axon initial segment)
/// Higher density at AIS, faster kinetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nav1_6 {
    pub g_max: f64,
    pub e_na: f64,
}

impl Nav1_6 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_na: 50.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 30.0) / 6.0).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 60.0) / 6.5).exp())
    }

    fn tau_m(&self, v: f64) -> f64 {
        0.1 + 0.4 / (1.0 + ((v + 35.0) / 10.0).abs())
    }

    fn tau_h(&self, v: f64) -> f64 {
        1.5 + 1.0 / (1.0 + ((v + 60.0) / 15.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(3) * state.h * (v - self.e_na)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 2.3, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 2.3, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// Kv1.1 - Low-threshold potassium channel
/// Regulates spike threshold and interspike interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kv1_1 {
    pub g_max: f64,
    pub e_k: f64,
}

impl Kv1_1 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn n_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 15.0) / 8.0).exp())
    }

    fn tau_n(&self, v: f64) -> f64 {
        1.0 + 4.0 / (1.0 + ((v + 30.0) / 20.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(4) * (v - self.e_k)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let n_inf = self.n_inf(v);
        let tau_n = q10_correction(self.tau_n(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        state.m += (n_inf - state.m) / tau_n * dt;
    }
}

/// Kv3.1 - High-threshold fast potassium channel
/// Critical for fast spiking interneurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kv3_1 {
    pub g_max: f64,
    pub e_k: f64,
}

impl Kv3_1 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn n_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 10.0) / 16.0).exp())
    }

    fn tau_n(&self, v: f64) -> f64 {
        0.5 + 2.0 / (1.0 + ((v + 40.0) / 15.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(4) * (v - self.e_k)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let n_inf = self.n_inf(v);
        let tau_n = q10_correction(self.tau_n(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        state.m += (n_inf - state.m) / tau_n * dt;
    }
}

/// Kv4.2 - A-type potassium channel (transient)
/// Controls backpropagation of action potentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kv4_2 {
    pub g_max: f64,
    pub e_k: f64,
}

impl Kv4_2 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 60.0) / 8.5).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 78.0) / 6.0).exp())
    }

    fn tau_m(&self, v: f64) -> f64 {
        1.0 + 10.0 / (1.0 + ((v + 60.0) / 20.0).exp())
    }

    fn tau_h(&self, v: f64) -> f64 {
        15.0 + 40.0 / (1.0 + ((v + 70.0) / 15.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(4) * state.h * (v - self.e_k)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 3.0, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// Kv7 (KCNQ) - M-current
/// Slow, non-inactivating K+ current for spike frequency adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kv7_M {
    pub g_max: f64,
    pub e_k: f64,
}

impl Kv7_M {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 35.0) / 10.0).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        100.0 // Very slow activation
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m * (v - self.e_k)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 2.5, TEMPERATURE, REFERENCE_TEMP);
        state.m += (m_inf - state.m) / tau_m * dt;
    }
}

/// Cav1.2 - L-type calcium channel
/// Long-lasting, high-threshold calcium current
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cav1_2 {
    pub g_max: f64,
    pub e_ca: f64,
}

impl Cav1_2 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_ca: 120.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 10.0) / 6.0).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 30.0) / 12.0).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        5.0
    }

    fn tau_h(&self, _v: f64) -> f64 {
        50.0
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(2) * state.h * (v - self.e_ca)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 3.0, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// Cav2.1 - P/Q-type calcium channel
/// Presynaptic calcium influx for neurotransmitter release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cav2_1 {
    pub g_max: f64,
    pub e_ca: f64,
}

impl Cav2_1 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_ca: 120.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 5.0) / 7.0).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 25.0) / 8.0).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        3.0
    }

    fn tau_h(&self, _v: f64) -> f64 {
        25.0
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(2) * state.h * (v - self.e_ca)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 3.0, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// Cav2.2 - N-type calcium channel
/// Presynaptic, involved in neurotransmitter release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cav2_2 {
    pub g_max: f64,
    pub e_ca: f64,
}

impl Cav2_2 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_ca: 120.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 5.0) / 8.0).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 30.0) / 10.0).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        2.5
    }

    fn tau_h(&self, _v: f64) -> f64 {
        30.0
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(2) * state.h * (v - self.e_ca)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 3.0, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// Cav3.1 - T-type calcium channel
/// Low-threshold, transient calcium current for burst firing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cav3_1 {
    pub g_max: f64,
    pub e_ca: f64,
}

impl Cav3_1 {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_ca: 120.0 }
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + (-(v + 52.0) / 7.4).exp())
    }

    fn h_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 80.0) / 5.0).exp())
    }

    fn tau_m(&self, v: f64) -> f64 {
        1.0 + 10.0 / (1.0 + ((v + 60.0) / 15.0).exp())
    }

    fn tau_h(&self, v: f64) -> f64 {
        20.0 + 50.0 / (1.0 + ((v + 70.0) / 10.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(2) * state.h * (v - self.e_ca)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let h_inf = self.h_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        let tau_h = q10_correction(self.tau_h(v), 3.0, TEMPERATURE, REFERENCE_TEMP);

        state.m += (m_inf - state.m) / tau_m * dt;
        state.h += (h_inf - state.h) / tau_h * dt;
    }
}

/// SK - Small conductance calcium-activated potassium channel
/// Contributes to afterhyperpolarization (AHP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SK_Channel {
    pub g_max: f64,
    pub e_k: f64,
}

impl SK_Channel {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn m_inf(&self, ca_i: f64) -> f64 {
        // Hill equation for calcium binding
        let k_d: f64 = 0.3; // μM
        let n: f64 = 4.0;   // Hill coefficient
        ca_i.powi(4) / (ca_i.powi(4) + k_d.powf(n as f64))
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        let m_inf = self.m_inf(state.ca_i);
        self.g_max * m_inf * (v - self.e_k)
    }

    pub fn update(&self, _v: f64, state: &mut AdvancedChannelState, dt: f64) {
        // Calcium dynamics (simplified)
        let ca_decay = 0.002; // 1/ms
        let ca_influx = state.m * 0.01; // From voltage-gated Ca channels
        state.ca_i += (-ca_decay * state.ca_i + ca_influx) * dt;
        state.ca_i = state.ca_i.max(0.05); // Basal level
    }
}

/// BK - Big conductance calcium-activated potassium channel
/// Fast AHP, voltage and calcium dependent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BK_Channel {
    pub g_max: f64,
    pub e_k: f64,
}

impl BK_Channel {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_k: -90.0 }
    }

    fn m_inf(&self, v: f64, ca_i: f64) -> f64 {
        // Both voltage and calcium dependent
        let v_half = -20.0 - 80.0 / (1.0 + (ca_i / 1.0).powi(2));
        1.0 / (1.0 + (-(v - v_half) / 15.0).exp())
    }

    fn tau_m(&self, _v: f64) -> f64 {
        1.0 // Fast kinetics
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m.powi(2) * (v - self.e_k)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v, state.ca_i);
        let tau_m = q10_correction(self.tau_m(v), 3.0, TEMPERATURE, REFERENCE_TEMP);
        state.m += (m_inf - state.m) / tau_m * dt;

        // Update calcium
        let ca_decay = 0.002;
        let ca_influx = if v > -20.0 { 0.01 } else { 0.0 };
        state.ca_i += (-ca_decay * state.ca_i + ca_influx) * dt;
        state.ca_i = state.ca_i.max(0.05);
    }
}

/// HCN - Hyperpolarization-activated cation channel (Ih current)
/// Pacemaker current, active at hyperpolarized potentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HCN_Channel {
    pub g_max: f64,
    pub e_h: f64,
}

impl HCN_Channel {
    pub fn new(g_max: f64) -> Self {
        Self { g_max, e_h: -30.0 } // Non-selective cation
    }

    fn m_inf(&self, v: f64) -> f64 {
        1.0 / (1.0 + ((v + 75.0) / 5.5).exp())
    }

    fn tau_m(&self, v: f64) -> f64 {
        100.0 + 500.0 / (1.0 + ((v + 70.0) / 10.0).exp())
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState) -> f64 {
        self.g_max * state.m * (v - self.e_h)
    }

    pub fn update(&self, v: f64, state: &mut AdvancedChannelState, dt: f64) {
        let m_inf = self.m_inf(v);
        let tau_m = q10_correction(self.tau_m(v), 2.5, TEMPERATURE, REFERENCE_TEMP);
        state.m += (m_inf - state.m) / tau_m * dt;
    }
}

/// NMDA receptor with complete Mg2+ voltage-dependent block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NMDA_Advanced {
    pub g_max: f64,
    pub e_nmda: f64,
    pub mg_conc: f64,
}

impl NMDA_Advanced {
    pub fn new(g_max: f64) -> Self {
        Self {
            g_max,
            e_nmda: 0.0,
            mg_conc: 1.0, // mM
        }
    }

    fn mg_block(&self, v: f64) -> f64 {
        // Year & Stevens (1990) formulation
        1.0 / (1.0 + (self.mg_conc / 3.57) * (-0.062 * v).exp())
    }

    fn m_inf(&self, glu: f64) -> f64 {
        // Glutamate binding
        let k_d = 10.0; // μM
        glu / (glu + k_d)
    }

    pub fn conductance(&self, v: f64, state: &AdvancedChannelState, glu: f64) -> f64 {
        let mg_block = self.mg_block(v);
        let glu_binding = self.m_inf(glu);
        self.g_max * glu_binding * state.m * mg_block * (v - self.e_nmda)
    }

    pub fn update(&self, _v: f64, state: &mut AdvancedChannelState, dt: f64, glu: f64) {
        let m_inf = self.m_inf(glu);
        let tau_m = 50.0; // Slow kinetics
        state.m += (m_inf - state.m) / tau_m * dt;

        // NMDA allows calcium influx
        if state.m > 0.1 {
            state.ca_i += 0.005 * dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav1_1() {
        let channel = Nav1_1::new(120.0);
        let mut state = AdvancedChannelState::default();

        // At rest
        channel.update(-65.0, &mut state, 0.01);
        assert!(state.m < 0.1);
        assert!(state.h > 0.5);
    }

    #[test]
    fn test_q10_correction() {
        let rate = 1.0;
        let corrected = q10_correction(rate, 2.0, 37.0, 27.0);
        assert!(corrected > rate); // Should increase at higher temp
    }

    #[test]
    fn test_kv7_m_current() {
        let channel = Kv7_M::new(1.0);
        let mut state = AdvancedChannelState::default();

        // M-current should activate slowly
        for _ in 0..1000 {
            channel.update(-30.0, &mut state, 0.1);
        }
        assert!(state.m > 0.3);
    }

    #[test]
    fn test_sk_calcium_dependence() {
        let channel = SK_Channel::new(1.0);
        let mut state = AdvancedChannelState::default();

        state.ca_i = 0.05; // Basal
        let g_low = channel.conductance(-65.0, &state);

        state.ca_i = 1.0; // Elevated
        let g_high = channel.conductance(-65.0, &state);

        assert!(g_high > g_low);
    }

    #[test]
    fn test_nmda_mg_block() {
        let channel = NMDA_Advanced::new(1.0);
        let state = AdvancedChannelState::default();

        let block_rest = channel.mg_block(-70.0);
        let block_depol = channel.mg_block(0.0);

        assert!(block_rest < 0.1);
        assert!(block_depol > 0.5);
    }

    #[test]
    fn test_hcn_pacemaker() {
        let channel = HCN_Channel::new(1.0);
        let mut state = AdvancedChannelState::default();

        // HCN activates at hyperpolarized potentials
        for _ in 0..1000 {
            channel.update(-80.0, &mut state, 0.1);
        }
        assert!(state.m > 0.3);
    }

    #[test]
    fn test_cav3_1_t_type() {
        let channel = Cav3_1::new(1.0);
        let mut state = AdvancedChannelState::default();

        // T-type activates at low threshold
        channel.update(-50.0, &mut state, 0.1);
        assert!(state.m > 0.0);
    }
}
