//! Ion Channel Dynamics Module
//! ============================
//!
//! Nernst-Planck equations for ion channel modeling.
//! Implements biophysically accurate ion flux calculations.
//!
//! # Equations Implemented
//! - Nernst equation: E = (RT/zF) * ln([out]/[in])
//! - Goldman-Hodgkin-Katz voltage equation
//! - Nernst-Planck flux equation with electrodiffusion
//!
//! # Ion Concentrations (typical values in mM)
//! | Ion   | Intracellular | Extracellular | Reversal Potential |
//! |-------|---------------|---------------|-------------------|
//! | Na+   | 15            | 145           | +60 mV            |
//! | K+    | 140           | 4             | -90 mV            |
//! | Cl-   | 10            | 110           | -70 mV            |
//! | Ca2+  | 0.0001        | 2             | +130 mV           |
//!
//! # References
//! - Hille B (2001) Ion Channels of Excitable Membranes, 3rd ed.
//! - Hodgkin AL & Katz B (1949) The effect of sodium ions on the electrical activity

use serde::{Deserialize, Serialize};

/// Physical constants
pub mod constants {
    /// Faraday constant (C/mol)
    pub const FARADAY: f64 = 96485.33212;
    /// Gas constant (J/(mol·K))
    pub const GAS_R: f64 = 8.31446261815324;
    /// Body temperature (K)
    pub const BODY_TEMP_K: f64 = 310.15; // 37°C
    /// RT/F at body temperature (mV)
    pub const RT_F: f64 = GAS_R * BODY_TEMP_K / FARADAY * 1000.0; // ~26.7 mV
}

/// Ion types in the nervous system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IonType {
    Sodium,     // Na+
    Potassium,  // K+
    Chloride,   // Cl-
    Calcium,    // Ca2+
}

impl IonType {
    /// Valence (charge) of the ion
    pub fn valence(&self) -> i32 {
        match self {
            IonType::Sodium => 1,
            IonType::Potassium => 1,
            IonType::Chloride => -1,
            IonType::Calcium => 2,
        }
    }

    /// Default intracellular concentration (mM)
    pub fn default_intracellular_mm(&self) -> f64 {
        match self {
            IonType::Sodium => 15.0,
            IonType::Potassium => 140.0,
            IonType::Chloride => 10.0,
            IonType::Calcium => 0.0001, // 100 nM
        }
    }

    /// Default extracellular concentration (mM)
    pub fn default_extracellular_mm(&self) -> f64 {
        match self {
            IonType::Sodium => 145.0,
            IonType::Potassium => 4.0,
            IonType::Chloride => 110.0,
            IonType::Calcium => 2.0,
        }
    }

    /// Default permeability coefficient (relative to K+)
    pub fn default_permeability(&self) -> f64 {
        match self {
            IonType::Sodium => 0.03,   // Much lower than K+
            IonType::Potassium => 1.0, // Reference
            IonType::Chloride => 0.1,
            IonType::Calcium => 0.0001, // Very low at rest
        }
    }
}

/// Ion concentration state (intracellular and extracellular)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IonConcentrations {
    pub ion: IonType,
    /// Intracellular concentration (mM)
    pub intracellular_mm: f64,
    /// Extracellular concentration (mM)
    pub extracellular_mm: f64,
}

impl IonConcentrations {
    /// Create with default physiological concentrations
    pub fn default_for(ion: IonType) -> Self {
        Self {
            ion,
            intracellular_mm: ion.default_intracellular_mm(),
            extracellular_mm: ion.default_extracellular_mm(),
        }
    }

    /// Create with custom concentrations
    pub fn new(ion: IonType, intracellular: f64, extracellular: f64) -> Self {
        Self {
            ion,
            intracellular_mm: intracellular,
            extracellular_mm: extracellular,
        }
    }

    /// Calculate Nernst potential (mV)
    ///
    /// E = (RT/zF) * ln([out]/[in])
    pub fn nernst_potential_mv(&self) -> f64 {
        let z = self.ion.valence() as f64;
        let ratio = self.extracellular_mm / self.intracellular_mm;

        // E = (RT/zF) * ln(ratio)
        constants::RT_F / z * ratio.ln()
    }
}

/// Complete ion environment for a cell
#[derive(Debug, Clone)]
pub struct IonEnvironment {
    pub sodium: IonConcentrations,
    pub potassium: IonConcentrations,
    pub chloride: IonConcentrations,
    pub calcium: IonConcentrations,
    /// Membrane capacitance (μF/cm²)
    pub capacitance: f64,
    /// Temperature (K)
    pub temperature_k: f64,
}

impl Default for IonEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl IonEnvironment {
    /// Create with default physiological values
    pub fn new() -> Self {
        Self {
            sodium: IonConcentrations::default_for(IonType::Sodium),
            potassium: IonConcentrations::default_for(IonType::Potassium),
            chloride: IonConcentrations::default_for(IonType::Chloride),
            calcium: IonConcentrations::default_for(IonType::Calcium),
            capacitance: 1.0,
            temperature_k: constants::BODY_TEMP_K,
        }
    }

    /// Get RT/F for current temperature
    pub fn rt_f_mv(&self) -> f64 {
        constants::GAS_R * self.temperature_k / constants::FARADAY * 1000.0
    }

    /// Calculate Goldman-Hodgkin-Katz resting potential (mV)
    ///
    /// V = (RT/F) * ln[(P_K[K]o + P_Na[Na]o + P_Cl[Cl]i) / (P_K[K]i + P_Na[Na]i + P_Cl[Cl]o)]
    pub fn goldman_potential_mv(&self, p_na: f64, p_k: f64, p_cl: f64) -> f64 {
        let rt_f = self.rt_f_mv();

        let numerator = p_k * self.potassium.extracellular_mm
            + p_na * self.sodium.extracellular_mm
            + p_cl * self.chloride.intracellular_mm;

        let denominator = p_k * self.potassium.intracellular_mm
            + p_na * self.sodium.intracellular_mm
            + p_cl * self.chloride.extracellular_mm;

        rt_f * (numerator / denominator).ln()
    }

    /// Calculate resting potential with default permeabilities
    pub fn resting_potential_mv(&self) -> f64 {
        self.goldman_potential_mv(
            IonType::Sodium.default_permeability(),
            IonType::Potassium.default_permeability(),
            IonType::Chloride.default_permeability(),
        )
    }

    /// Get concentrations for a specific ion
    pub fn get(&self, ion: IonType) -> &IonConcentrations {
        match ion {
            IonType::Sodium => &self.sodium,
            IonType::Potassium => &self.potassium,
            IonType::Chloride => &self.chloride,
            IonType::Calcium => &self.calcium,
        }
    }

    /// Get mutable concentrations for a specific ion
    pub fn get_mut(&mut self, ion: IonType) -> &mut IonConcentrations {
        match ion {
            IonType::Sodium => &mut self.sodium,
            IonType::Potassium => &mut self.potassium,
            IonType::Chloride => &mut self.chloride,
            IonType::Calcium => &mut self.calcium,
        }
    }
}

/// Ion channel model with conductance and gating
#[derive(Debug, Clone)]
pub struct IonChannel {
    /// Ion type this channel conducts
    pub ion: IonType,
    /// Maximum conductance (nS)
    pub g_max: f64,
    /// Current gating variable (0-1)
    pub gate_open: f64,
    /// Reversal potential (mV)
    pub e_rev: f64,
}

impl IonChannel {
    /// Create a new ion channel
    pub fn new(ion: IonType, g_max: f64, e_rev: f64) -> Self {
        Self {
            ion,
            g_max,
            gate_open: 0.0,
            e_rev,
        }
    }

    /// Create channel from ion environment
    pub fn from_environment(ion: IonType, g_max: f64, env: &IonEnvironment) -> Self {
        Self {
            ion,
            g_max,
            gate_open: 0.0,
            e_rev: env.get(ion).nernst_potential_mv(),
        }
    }

    /// Calculate current through channel (pA)
    ///
    /// I = g_max * gate_open * (V - E_rev)
    pub fn current_pa(&self, membrane_potential_mv: f64) -> f64 {
        self.g_max * self.gate_open * (membrane_potential_mv - self.e_rev)
    }

    /// Calculate driving force (mV)
    pub fn driving_force(&self, membrane_potential_mv: f64) -> f64 {
        membrane_potential_mv - self.e_rev
    }

    /// Set gating variable
    pub fn set_gate(&mut self, open_probability: f64) {
        self.gate_open = open_probability.clamp(0.0, 1.0);
    }
}

/// GABA_A receptor ion channel (Cl- selective)
///
/// Modulated by drugs like benzodiazepines and anesthetics.
#[derive(Debug, Clone)]
pub struct GabaAChannel {
    /// Base channel properties
    pub channel: IonChannel,
    /// GABA binding (0-1)
    pub gaba_bound: f64,
    /// Drug modulation factor (1.0 = no drug)
    pub drug_modulation: f64,
}

impl GabaAChannel {
    /// Create new GABA_A channel with default properties
    pub fn new(env: &IonEnvironment) -> Self {
        Self {
            channel: IonChannel::from_environment(IonType::Chloride, 30.0, env),
            gaba_bound: 0.0,
            drug_modulation: 1.0,
        }
    }

    /// Set GABA binding level
    pub fn bind_gaba(&mut self, gaba_concentration_um: f64) {
        // Hill equation for GABA binding
        let ec50: f64 = 20.0; // EC50 ~20 μM for GABA
        let hill: f64 = 1.5;
        self.gaba_bound = gaba_concentration_um.powf(hill)
            / (ec50.powf(hill) + gaba_concentration_um.powf(hill));
        self.update_conductance();
    }

    /// Apply drug modulation (from receptor_mechanisms)
    pub fn apply_drug_modulation(&mut self, modulation_factor: f64) {
        self.drug_modulation = modulation_factor;
        self.update_conductance();
    }

    /// Update channel conductance based on GABA and drug state
    fn update_conductance(&mut self) {
        // Channel opening depends on GABA binding, enhanced by drugs
        // Drug modulation affects the efficacy of GABA binding
        let effective_binding = self.gaba_bound * self.drug_modulation;
        self.channel.set_gate(effective_binding.min(1.0));
    }

    /// Calculate chloride current (pA)
    pub fn current_pa(&self, membrane_potential_mv: f64) -> f64 {
        self.channel.current_pa(membrane_potential_mv)
    }

    /// Get the shift in reversal potential due to intracellular Cl- changes
    pub fn calculate_ecl_shift(&self, env: &IonEnvironment) -> f64 {
        env.chloride.nernst_potential_mv()
    }
}

/// Nernst-Planck electrodiffusion calculator
///
/// Models ion flux through a membrane considering both
/// concentration gradient and electrical potential.
pub struct NernstPlanckCalculator {
    /// Temperature (K)
    pub temperature_k: f64,
    /// Membrane thickness (nm)
    pub membrane_thickness_nm: f64,
}

impl Default for NernstPlanckCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl NernstPlanckCalculator {
    pub fn new() -> Self {
        Self {
            temperature_k: constants::BODY_TEMP_K,
            membrane_thickness_nm: 5.0, // Typical lipid bilayer
        }
    }

    /// Calculate ion flux using Nernst-Planck equation (mol/(m²·s))
    ///
    /// J = -D * (dc/dx + (zF/RT) * c * (dV/dx))
    ///
    /// Simplified for constant field approximation (Goldman flux equation)
    pub fn calculate_flux(
        &self,
        ion: IonType,
        permeability_cm_s: f64,
        conc: &IonConcentrations,
        membrane_potential_mv: f64,
    ) -> f64 {
        let z = ion.valence() as f64;
        let rt = constants::GAS_R * self.temperature_k;
        let u = z * constants::FARADAY * membrane_potential_mv / 1000.0 / rt;

        // Goldman flux equation
        // J = P * z * F * u * (c_in - c_out * exp(-u)) / (1 - exp(-u))
        if u.abs() < 1e-6 {
            // Limit as u -> 0
            permeability_cm_s * (conc.extracellular_mm - conc.intracellular_mm)
        } else {
            let exp_neg_u = (-u).exp();
            permeability_cm_s * z * u
                * (conc.intracellular_mm - conc.extracellular_mm * exp_neg_u)
                / (1.0 - exp_neg_u)
        }
    }
}

/// ATP consumption calculator for ion pumps
#[derive(Debug, Clone)]
pub struct AtpConsumption {
    /// Na+/K+-ATPase activity (mol ATP / s / cell)
    pub na_k_atpase_rate: f64,
    /// Ca2+-ATPase activity (mol ATP / s / cell)
    pub ca_atpase_rate: f64,
    /// Baseline ATP consumption (mol / s / cell)
    pub baseline_rate: f64,
}

impl Default for AtpConsumption {
    fn default() -> Self {
        Self::new()
    }
}

impl AtpConsumption {
    pub fn new() -> Self {
        Self {
            // Typical values for a neuron
            na_k_atpase_rate: 1e-15,  // ~1 fmol/s
            ca_atpase_rate: 1e-16,    // ~0.1 fmol/s
            baseline_rate: 5e-16,     // ~0.5 fmol/s
        }
    }

    /// Calculate ATP consumed by Na+/K+-ATPase to restore gradients
    ///
    /// 3 Na+ out, 2 K+ in per ATP hydrolyzed
    pub fn from_sodium_flux(&self, na_flux_mol_s: f64) -> f64 {
        // 1 ATP per 3 Na+ pumped
        na_flux_mol_s.abs() / 3.0
    }

    /// Calculate ATP consumed by Ca2+-ATPase
    ///
    /// 1 Ca2+ out per ATP (PMCA) or 2 Ca2+ (SERCA)
    pub fn from_calcium_flux(&self, ca_flux_mol_s: f64) -> f64 {
        // 1 ATP per 2 Ca2+ on average
        ca_flux_mol_s.abs() / 2.0
    }

    /// Total ATP consumption rate (mol/s)
    pub fn total_rate(&self) -> f64 {
        self.na_k_atpase_rate + self.ca_atpase_rate + self.baseline_rate
    }

    /// Update consumption based on neural activity
    pub fn update_for_firing_rate(&mut self, firing_rate_hz: f64) {
        // Higher firing = more ATP needed to restore gradients
        let activity_factor = 1.0 + firing_rate_hz / 10.0; // 10 Hz baseline
        self.na_k_atpase_rate *= activity_factor;
        self.ca_atpase_rate *= activity_factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nernst_potentials() {
        let env = IonEnvironment::new();

        let e_k = env.potassium.nernst_potential_mv();
        let e_na = env.sodium.nernst_potential_mv();
        let e_cl = env.chloride.nernst_potential_mv();

        // Expected values (approximately)
        assert!(e_k < -80.0 && e_k > -100.0, "E_K should be ~-90 mV, got {}", e_k);
        assert!(e_na > 50.0 && e_na < 70.0, "E_Na should be ~+60 mV, got {}", e_na);
        assert!(e_cl < -60.0 && e_cl > -80.0, "E_Cl should be ~-70 mV, got {}", e_cl);
    }

    #[test]
    fn test_goldman_potential() {
        let env = IonEnvironment::new();
        let v_rest = env.resting_potential_mv();

        // Resting potential should be around -70 to -80 mV
        assert!(v_rest < -60.0 && v_rest > -90.0,
            "Resting potential should be ~-70 mV, got {}", v_rest);
    }

    #[test]
    fn test_gabaa_channel() {
        let env = IonEnvironment::new();
        let mut gabaa = GabaAChannel::new(&env);

        // No GABA = no current
        gabaa.bind_gaba(0.0);
        assert!(gabaa.channel.gate_open < 0.01);

        // High GABA = channel opens
        gabaa.bind_gaba(100.0); // 100 μM
        assert!(gabaa.channel.gate_open > 0.5);

        // Drug modulation enhances effect
        gabaa.bind_gaba(10.0); // Submaximal GABA
        let baseline_gate = gabaa.channel.gate_open;

        gabaa.apply_drug_modulation(2.0); // 2x modulation (e.g., benzodiazepine)
        assert!(gabaa.channel.gate_open > baseline_gate);
    }

    #[test]
    fn test_ion_channel_current() {
        let env = IonEnvironment::new();
        let mut channel = IonChannel::from_environment(IonType::Potassium, 10.0, &env);
        channel.set_gate(1.0); // Fully open

        // At resting potential, K+ current should be small (V ≈ E_K)
        let i_at_rest = channel.current_pa(-85.0);
        // E_K is approximately -90mV, so at -85mV there's a small driving force
        assert!(i_at_rest.abs() < 200.0); // Small current near E_K

        // At depolarized potential, K+ current should be outward (positive)
        let i_depolarized = channel.current_pa(0.0);
        assert!(i_depolarized > 500.0); // Strong outward current
    }
}
