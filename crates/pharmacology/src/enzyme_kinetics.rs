//! Enzyme Kinetics with Saturation Detection
//! ==========================================
//!
//! First-principles Michaelis-Menten kinetics with:
//! - Substrate saturation detection
//! - Competitive, non-competitive, and uncompetitive inhibition
//! - Allosteric modulation (Hill kinetics)
//! - Product inhibition
//! - Time-dependent enzyme inactivation
//!
//! # Mathematical Foundation
//!
//! Standard Michaelis-Menten:
//! ```text
//! v = Vmax * [S] / (Km + [S])
//! ```
//!
//! With inhibition:
//! ```text
//! v = Vmax * [S] / (Km * (1 + [I]/Ki) + [S])  // Competitive
//! v = Vmax * [S] / ((Km + [S]) * (1 + [I]/Ki))  // Non-competitive
//! v = Vmax * [S] / (Km + [S] * (1 + [I]/Ki))  // Uncompetitive
//! ```
//!
//! Hill equation for cooperativity:
//! ```text
//! v = Vmax * [S]^n / (K0.5^n + [S]^n)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Physical constants for enzyme kinetics
pub mod constants {
    /// Boltzmann constant (J/K)
    pub const K_B: f64 = 1.380649e-23;
    /// Planck constant (J·s)
    pub const H_PLANCK: f64 = 6.62607015e-34;
    /// Avogadro number (mol⁻¹)
    pub const N_A: f64 = 6.02214076e23;
    /// Gas constant (J/(mol·K))
    pub const R_GAS: f64 = 8.314462618;
    /// Body temperature (K)
    pub const BODY_TEMP: f64 = 310.15; // 37°C
}

/// Saturation regime of an enzymatic reaction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SaturationRegime {
    /// [S] << Km: First-order kinetics, v ≈ (Vmax/Km)*[S]
    FirstOrder {
        /// Fraction of Km: [S]/Km
        saturation_fraction: f64,
    },
    /// [S] ≈ Km: Mixed kinetics
    Transition {
        /// Fraction of Vmax achieved
        vmax_fraction: f64,
    },
    /// [S] >> Km: Zero-order kinetics, v ≈ Vmax
    Saturated {
        /// How many times Km is [S]
        fold_over_km: f64,
    },
}

impl SaturationRegime {
    /// Determine saturation regime from substrate concentration
    pub fn from_concentration(substrate_um: f64, km_um: f64) -> Self {
        let ratio = substrate_um / km_um;

        if ratio < 0.1 {
            SaturationRegime::FirstOrder {
                saturation_fraction: ratio,
            }
        } else if ratio > 10.0 {
            SaturationRegime::Saturated {
                fold_over_km: ratio,
            }
        } else {
            let vmax_fraction = substrate_um / (km_um + substrate_um);
            SaturationRegime::Transition { vmax_fraction }
        }
    }

    /// Warning message if approaching saturation
    pub fn warning(&self) -> Option<String> {
        match self {
            SaturationRegime::Saturated { fold_over_km } => {
                Some(format!(
                    "SATURATION WARNING: Substrate at {:.1}x Km. \
                     Enzyme operating at near-maximum velocity. \
                     Accumulation risk!",
                    fold_over_km
                ))
            }
            SaturationRegime::Transition { vmax_fraction } if *vmax_fraction > 0.8 => {
                Some(format!(
                    "Approaching saturation: {:.0}% of Vmax reached",
                    vmax_fraction * 100.0
                ))
            }
            _ => None,
        }
    }
}

/// Type of enzyme inhibition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InhibitionType {
    /// Inhibitor binds only to free enzyme (competes with substrate)
    /// Increases apparent Km, Vmax unchanged
    Competitive,
    /// Inhibitor binds to enzyme regardless of substrate binding
    /// Decreases apparent Vmax, Km unchanged
    NonCompetitive,
    /// Inhibitor binds only to ES complex
    /// Decreases both apparent Km and Vmax
    Uncompetitive,
    /// Inhibitor binds preferentially to one form
    /// Both Km and Vmax affected
    Mixed {
        /// α factor: ratio of Ki(ES)/Ki(E)
        alpha: f64,
    },
    /// Irreversible covalent modification
    Irreversible {
        /// Inactivation rate constant (1/s)
        k_inact: f64,
    },
}

/// Parameters for an enzymatic reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnzymeParameters {
    /// Enzyme identifier
    pub name: String,
    /// Maximum velocity (µM/s) at standard enzyme concentration
    pub vmax_um_per_s: f64,
    /// Michaelis constant (µM)
    pub km_um: f64,
    /// Hill coefficient for cooperativity (1.0 = no cooperativity)
    pub hill_coefficient: f64,
    /// Product inhibition constant (µM), if applicable
    pub ki_product_um: Option<f64>,
    /// Catalytic efficiency kcat/Km (1/(µM·s))
    pub catalytic_efficiency: f64,
    /// Turnover number kcat (1/s)
    pub kcat: f64,
}

impl EnzymeParameters {
    /// Create enzyme parameters from kinetic constants
    pub fn new(
        name: &str,
        vmax_um_per_s: f64,
        km_um: f64,
        enzyme_concentration_um: f64,
    ) -> Self {
        let kcat = vmax_um_per_s / enzyme_concentration_um;
        let catalytic_efficiency = kcat / km_um;

        Self {
            name: name.to_string(),
            vmax_um_per_s,
            km_um,
            hill_coefficient: 1.0,
            ki_product_um: None,
            catalytic_efficiency,
            kcat,
        }
    }

    /// Set cooperativity (Hill coefficient)
    pub fn with_cooperativity(mut self, n: f64) -> Self {
        self.hill_coefficient = n;
        self
    }

    /// Set product inhibition
    pub fn with_product_inhibition(mut self, ki_um: f64) -> Self {
        self.ki_product_um = Some(ki_um);
        self
    }
}

/// An inhibitor affecting an enzyme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inhibitor {
    /// Inhibitor name
    pub name: String,
    /// Inhibition constant Ki (µM)
    pub ki_um: f64,
    /// Type of inhibition
    pub inhibition_type: InhibitionType,
    /// Current concentration (µM)
    pub concentration_um: f64,
}

impl Inhibitor {
    pub fn new(name: &str, ki_um: f64, inhibition_type: InhibitionType) -> Self {
        Self {
            name: name.to_string(),
            ki_um,
            inhibition_type,
            concentration_um: 0.0,
        }
    }

    pub fn at_concentration(mut self, conc_um: f64) -> Self {
        self.concentration_um = conc_um;
        self
    }
}

/// State of enzyme during reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnzymeState {
    /// Active enzyme fraction (0-1)
    pub active_fraction: f64,
    /// Time-dependent inactivation accumulated
    pub inactivation_accumulated: f64,
    /// Phosphorylation state (affects activity)
    pub phosphorylation_level: f64,
}

impl Default for EnzymeState {
    fn default() -> Self {
        Self {
            active_fraction: 1.0,
            inactivation_accumulated: 0.0,
            phosphorylation_level: 0.0,
        }
    }
}

/// Complete enzyme kinetics calculator
#[derive(Debug, Clone)]
pub struct EnzymeKinetics {
    /// Enzyme parameters
    pub params: EnzymeParameters,
    /// Active inhibitors
    pub inhibitors: Vec<Inhibitor>,
    /// Enzyme state
    pub state: EnzymeState,
}

impl EnzymeKinetics {
    pub fn new(params: EnzymeParameters) -> Self {
        Self {
            params,
            inhibitors: Vec::new(),
            state: EnzymeState::default(),
        }
    }

    /// Add an inhibitor
    pub fn add_inhibitor(&mut self, inhibitor: Inhibitor) {
        self.inhibitors.push(inhibitor);
    }

    /// Calculate reaction velocity with all modifiers
    pub fn velocity(&self, substrate_um: f64, product_um: f64) -> VelocityResult {
        // Base parameters
        let mut apparent_vmax = self.params.vmax_um_per_s * self.state.active_fraction;
        let mut apparent_km = self.params.km_um;
        let n = self.params.hill_coefficient;

        // Apply inhibitor effects
        for inhibitor in &self.inhibitors {
            let i_over_ki = inhibitor.concentration_um / inhibitor.ki_um;

            match inhibitor.inhibition_type {
                InhibitionType::Competitive => {
                    // Increases apparent Km
                    apparent_km *= 1.0 + i_over_ki;
                }
                InhibitionType::NonCompetitive => {
                    // Decreases apparent Vmax
                    apparent_vmax /= 1.0 + i_over_ki;
                }
                InhibitionType::Uncompetitive => {
                    // Decreases both
                    let factor = 1.0 + i_over_ki;
                    apparent_km /= factor;
                    apparent_vmax /= factor;
                }
                InhibitionType::Mixed { alpha } => {
                    // Mixed effects
                    apparent_km *= (1.0 + i_over_ki) / (1.0 + i_over_ki / alpha);
                    apparent_vmax /= 1.0 + i_over_ki / alpha;
                }
                InhibitionType::Irreversible { .. } => {
                    // Handled separately through enzyme state
                }
            }
        }

        // Apply product inhibition if defined
        if let Some(ki_p) = self.params.ki_product_um {
            let p_over_ki = product_um / ki_p;
            // Product inhibition is typically competitive
            apparent_km *= 1.0 + p_over_ki;
        }

        // Calculate velocity using Hill equation
        let s_n = substrate_um.powf(n);
        let km_n = apparent_km.powf(n);

        let velocity = apparent_vmax * s_n / (km_n + s_n);

        // Determine saturation regime
        let regime = SaturationRegime::from_concentration(substrate_um, apparent_km);

        VelocityResult {
            velocity_um_per_s: velocity,
            apparent_km_um: apparent_km,
            apparent_vmax_um_per_s: apparent_vmax,
            saturation_regime: regime,
            substrate_um,
            product_um,
        }
    }

    /// Update enzyme state over time (for time-dependent effects)
    pub fn update_state(&mut self, dt_s: f64) {
        for inhibitor in &self.inhibitors {
            if let InhibitionType::Irreversible { k_inact } = inhibitor.inhibition_type {
                // First-order inactivation: d[E]/dt = -k_inact * [I] * [E]
                let inactivation_rate = k_inact * inhibitor.concentration_um;
                self.state.active_fraction *= (-inactivation_rate * dt_s).exp();
                self.state.inactivation_accumulated += inactivation_rate * dt_s;
            }
        }
    }

    /// Estimate time to reach steady state
    pub fn time_to_steady_state_s(&self, substrate_um: f64) -> f64 {
        // Approximate as 5 * Km / Vmax for near-complete approach
        let velocity = self.velocity(substrate_um, 0.0).velocity_um_per_s;
        if velocity > 0.0 {
            5.0 * substrate_um / velocity
        } else {
            f64::INFINITY
        }
    }
}

/// Result of velocity calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityResult {
    /// Reaction velocity (µM/s)
    pub velocity_um_per_s: f64,
    /// Apparent Km after inhibitor effects (µM)
    pub apparent_km_um: f64,
    /// Apparent Vmax after inhibitor effects (µM/s)
    pub apparent_vmax_um_per_s: f64,
    /// Saturation regime
    pub saturation_regime: SaturationRegime,
    /// Substrate concentration used (µM)
    pub substrate_um: f64,
    /// Product concentration used (µM)
    pub product_um: f64,
}

/// CYP450 enzyme with drug metabolism specifics
#[derive(Debug, Clone)]
pub struct Cyp450Enzyme {
    /// Enzyme isoform (e.g., "CYP3A4", "CYP2D6")
    pub isoform: String,
    /// Base kinetics
    pub kinetics: EnzymeKinetics,
    /// Substrate specificity map: drug -> relative affinity
    pub substrate_specificity: HashMap<String, f64>,
    /// Inducers: drug -> fold induction
    pub inducers: HashMap<String, f64>,
    /// Current expression level relative to baseline
    pub expression_level: f64,
}

impl Cyp450Enzyme {
    /// Create a new CYP450 isoform
    pub fn new(isoform: &str, vmax: f64, km: f64) -> Self {
        let params = EnzymeParameters::new(isoform, vmax, km, 1.0);
        Self {
            isoform: isoform.to_string(),
            kinetics: EnzymeKinetics::new(params),
            substrate_specificity: HashMap::new(),
            inducers: HashMap::new(),
            expression_level: 1.0,
        }
    }

    /// Add substrate with relative affinity
    pub fn add_substrate(&mut self, drug: &str, relative_km: f64) {
        self.substrate_specificity.insert(drug.to_string(), relative_km);
    }

    /// Calculate metabolism rate for a specific drug
    pub fn metabolize(&self, drug: &str, concentration_um: f64) -> f64 {
        let specificity = self.substrate_specificity.get(drug).unwrap_or(&1.0);
        let adjusted_km = self.kinetics.params.km_um * specificity;

        // Adjusted Vmax by expression level
        let vmax = self.kinetics.params.vmax_um_per_s * self.expression_level;

        // Standard MM with adjusted parameters
        vmax * concentration_um / (adjusted_km + concentration_um)
    }

    /// Apply enzyme induction
    pub fn induce(&mut self, inducer: &str, fold: f64) {
        self.inducers.insert(inducer.to_string(), fold);
        // Recalculate expression level from all inducers
        self.expression_level = self.inducers.values().product();
    }
}

/// Metabolic pathway with multiple sequential reactions
#[derive(Debug, Clone)]
pub struct MetabolicPathway {
    /// Pathway name
    pub name: String,
    /// Enzymes in order
    pub enzymes: Vec<EnzymeKinetics>,
    /// Intermediate concentrations (µM)
    pub intermediates: Vec<f64>,
    /// Rate-limiting step index
    pub rate_limiting_step: Option<usize>,
}

impl MetabolicPathway {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enzymes: Vec::new(),
            intermediates: Vec::new(),
            rate_limiting_step: None,
        }
    }

    /// Add enzyme to pathway
    pub fn add_step(&mut self, enzyme: EnzymeKinetics) {
        self.enzymes.push(enzyme);
        self.intermediates.push(0.0);
    }

    /// Simulate pathway for one time step
    pub fn step(&mut self, substrate_um: f64, dt_s: f64) -> PathwayFlux {
        if self.enzymes.is_empty() {
            return PathwayFlux::default();
        }

        let mut fluxes = Vec::with_capacity(self.enzymes.len());
        let mut min_flux = f64::MAX;
        let mut rate_limiting = 0;

        // First enzyme uses external substrate
        let mut current_substrate = substrate_um;

        for (i, enzyme) in self.enzymes.iter().enumerate() {
            let product = if i + 1 < self.intermediates.len() {
                self.intermediates[i + 1]
            } else {
                0.0
            };

            let result = enzyme.velocity(current_substrate, product);
            let flux = result.velocity_um_per_s;

            if flux < min_flux {
                min_flux = flux;
                rate_limiting = i;
            }

            fluxes.push(result);

            // Next enzyme uses this product
            current_substrate = self.intermediates.get(i).copied().unwrap_or(0.0);
        }

        // Update intermediate concentrations
        for i in 0..self.enzymes.len() {
            let production = fluxes.get(i).map(|f| f.velocity_um_per_s).unwrap_or(0.0);
            let consumption = fluxes.get(i + 1).map(|f| f.velocity_um_per_s).unwrap_or(0.0);

            if i < self.intermediates.len() {
                self.intermediates[i] += (production - consumption) * dt_s;
                // Prevent negative concentrations
                self.intermediates[i] = self.intermediates[i].max(0.0);
            }
        }

        self.rate_limiting_step = Some(rate_limiting);

        PathwayFlux {
            step_fluxes: fluxes,
            rate_limiting_step: rate_limiting,
            overall_flux_um_per_s: min_flux,
            intermediate_concentrations: self.intermediates.clone(),
        }
    }

    /// Find bottlenecks in the pathway
    pub fn identify_bottlenecks(&self, substrate_um: f64) -> Vec<(usize, String, SaturationRegime)> {
        let mut bottlenecks = Vec::new();
        let mut current_s = substrate_um;

        for (i, enzyme) in self.enzymes.iter().enumerate() {
            let result = enzyme.velocity(current_s, 0.0);

            if matches!(result.saturation_regime, SaturationRegime::Saturated { .. }) {
                bottlenecks.push((
                    i,
                    enzyme.params.name.clone(),
                    result.saturation_regime,
                ));
            }

            current_s = self.intermediates.get(i).copied().unwrap_or(0.0);
        }

        bottlenecks
    }
}

/// Flux through a metabolic pathway
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PathwayFlux {
    /// Flux at each step
    pub step_fluxes: Vec<VelocityResult>,
    /// Index of rate-limiting step
    pub rate_limiting_step: usize,
    /// Overall pathway flux (limited by slowest step)
    pub overall_flux_um_per_s: f64,
    /// Current intermediate concentrations
    pub intermediate_concentrations: Vec<f64>,
}

/// Database of CYP450 isoforms with typical parameters
#[derive(Debug, Clone)]
pub struct Cyp450Database {
    pub enzymes: HashMap<String, Cyp450Enzyme>,
}

impl Default for Cyp450Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Cyp450Database {
    pub fn new() -> Self {
        let mut enzymes = HashMap::new();

        // CYP3A4 - Most abundant, metabolizes ~50% of drugs
        let mut cyp3a4 = Cyp450Enzyme::new("CYP3A4", 10.0, 5.0);
        cyp3a4.add_substrate("midazolam", 1.0);
        cyp3a4.add_substrate("diazepam", 1.2);
        cyp3a4.add_substrate("alprazolam", 0.8);
        cyp3a4.add_substrate("triazolam", 0.9);
        cyp3a4.add_substrate("fentanyl", 1.5);
        cyp3a4.add_substrate("carbamazepine", 2.0);
        enzymes.insert("CYP3A4".to_string(), cyp3a4);

        // CYP2D6 - Highly polymorphic
        let mut cyp2d6 = Cyp450Enzyme::new("CYP2D6", 5.0, 10.0);
        cyp2d6.add_substrate("codeine", 1.0);
        cyp2d6.add_substrate("tramadol", 1.2);
        cyp2d6.add_substrate("haloperidol", 0.8);
        cyp2d6.add_substrate("fluoxetine", 2.0);
        cyp2d6.add_substrate("paroxetine", 1.5);
        enzymes.insert("CYP2D6".to_string(), cyp2d6);

        // CYP2C19 - Important for PPIs and some benzos
        let mut cyp2c19 = Cyp450Enzyme::new("CYP2C19", 8.0, 15.0);
        cyp2c19.add_substrate("diazepam", 0.8);
        cyp2c19.add_substrate("clobazam", 1.0);
        cyp2c19.add_substrate("clopidogrel", 1.2);
        enzymes.insert("CYP2C19".to_string(), cyp2c19);

        // CYP2C9 - Warfarin, phenytoin
        let mut cyp2c9 = Cyp450Enzyme::new("CYP2C9", 6.0, 8.0);
        cyp2c9.add_substrate("phenytoin", 1.0);
        cyp2c9.add_substrate("valproate", 1.5);
        cyp2c9.add_substrate("ibuprofen", 0.7);
        enzymes.insert("CYP2C9".to_string(), cyp2c9);

        // CYP1A2 - Caffeine, theophylline
        let mut cyp1a2 = Cyp450Enzyme::new("CYP1A2", 4.0, 20.0);
        cyp1a2.add_substrate("caffeine", 1.0);
        cyp1a2.add_substrate("theophylline", 1.2);
        cyp1a2.add_substrate("clozapine", 0.8);
        enzymes.insert("CYP1A2".to_string(), cyp1a2);

        // CYP2B6 - Bupropion, methadone
        let mut cyp2b6 = Cyp450Enzyme::new("CYP2B6", 3.0, 25.0);
        cyp2b6.add_substrate("bupropion", 1.0);
        cyp2b6.add_substrate("ketamine", 1.5);
        cyp2b6.add_substrate("propofol", 0.6);
        enzymes.insert("CYP2B6".to_string(), cyp2b6);

        // CYP2E1 - Ethanol, acetaminophen
        let mut cyp2e1 = Cyp450Enzyme::new("CYP2E1", 2.0, 50.0);
        cyp2e1.add_substrate("ethanol", 1.0);
        cyp2e1.add_substrate("acetaminophen", 2.0);
        cyp2e1.add_substrate("isoflurane", 0.5);
        enzymes.insert("CYP2E1".to_string(), cyp2e1);

        Self { enzymes }
    }

    /// Get enzyme by isoform name
    pub fn get(&self, isoform: &str) -> Option<&Cyp450Enzyme> {
        self.enzymes.get(isoform)
    }

    /// Calculate total metabolism rate across all isoforms
    pub fn total_metabolism(&self, drug: &str, concentration_um: f64) -> f64 {
        self.enzymes
            .values()
            .map(|e| e.metabolize(drug, concentration_um))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_michaelis_menten_basic() {
        let params = EnzymeParameters::new("TestEnzyme", 10.0, 5.0, 1.0);
        let kinetics = EnzymeKinetics::new(params);

        // At Km, velocity should be Vmax/2
        let result = kinetics.velocity(5.0, 0.0);
        assert!((result.velocity_um_per_s - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_saturation_detection() {
        let regime = SaturationRegime::from_concentration(100.0, 5.0);
        assert!(matches!(regime, SaturationRegime::Saturated { .. }));

        let regime = SaturationRegime::from_concentration(0.1, 5.0);
        assert!(matches!(regime, SaturationRegime::FirstOrder { .. }));
    }

    #[test]
    fn test_competitive_inhibition() {
        let params = EnzymeParameters::new("TestEnzyme", 10.0, 5.0, 1.0);
        let mut kinetics = EnzymeKinetics::new(params);

        // Add competitive inhibitor at Ki concentration
        let inhibitor = Inhibitor::new("CompInhib", 10.0, InhibitionType::Competitive)
            .at_concentration(10.0);
        kinetics.add_inhibitor(inhibitor);

        // Apparent Km should double
        let result = kinetics.velocity(5.0, 0.0);
        assert!((result.apparent_km_um - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_cyp450_metabolism() {
        let db = Cyp450Database::new();

        // Test midazolam metabolism by CYP3A4
        let rate = db.get("CYP3A4").unwrap().metabolize("midazolam", 1.0);
        assert!(rate > 0.0);
    }
}
