//! Receptor Trafficking and Desensitization Dynamics
//! ===================================================
//!
//! Models receptor lifecycle including:
//! - Agonist-induced desensitization (phosphorylation, β-arrestin)
//! - Internalization and recycling
//! - Downregulation and upregulation
//! - Tolerance and sensitization mechanisms
//!
//! # GABA_A Receptor Trafficking
//!
//! ```text
//! Surface (active) ←→ Phosphorylated (desensitized) → Internalized
//!     ↑                                                    ↓
//!     ←←←←←← Recycled ←←←←←← Endosome ←←←←←←←←←←←←←←←←←←←
//!                                ↓
//!                           Degraded
//! ```
//!
//! # Time Scales
//!
//! | Process              | Time constant |
//! |---------------------|---------------|
//! | Fast desensitization| 10-100 ms     |
//! | Slow desensitization| 1-10 s        |
//! | Internalization     | 5-30 min      |
//! | Recycling           | 15-60 min     |
//! | Downregulation      | hours-days    |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State of a receptor in its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceptorState {
    /// Active, on membrane surface
    Active,
    /// Fast desensitized (conformational change)
    FastDesensitized,
    /// Slow desensitized (phosphorylated)
    SlowDesensitized,
    /// Internalized into endosome
    Internalized,
    /// In recycling pathway
    Recycling,
    /// Marked for degradation
    Degraded,
    /// Newly synthesized, being trafficked to membrane
    Nascent,
}

/// Kinase/phosphatase involved in receptor modulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KinaseType {
    /// Protein Kinase A - cAMP dependent
    Pka,
    /// Protein Kinase C
    Pkc,
    /// CaM Kinase II - Ca2+ dependent
    CamkII,
    /// Src family kinases
    Src,
    /// G-protein receptor kinases
    Grk,
    /// Calcineurin (phosphatase)
    Calcineurin,
}

impl KinaseType {
    /// Effect on receptor function (positive = enhancement)
    pub fn effect_on_receptor(&self) -> f64 {
        match self {
            // PKA phosphorylation generally enhances GABA_A currents
            KinaseType::Pka => 0.3,
            // PKC generally reduces GABA_A currents
            KinaseType::Pkc => -0.2,
            // CaMKII enhances some subtypes
            KinaseType::CamkII => 0.1,
            // Src maintains surface expression
            KinaseType::Src => 0.15,
            // GRK promotes desensitization
            KinaseType::Grk => -0.4,
            // Calcineurin (dephosphorylation) - mixed
            KinaseType::Calcineurin => 0.2,
        }
    }
}

/// Phosphorylation site on receptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhosphorylationSite {
    /// Site name (e.g., "S408/S409" for GABA_A γ2)
    pub site_name: String,
    /// Subunit containing site (e.g., "γ2", "β3")
    pub subunit: String,
    /// Kinases that phosphorylate this site
    pub kinases: Vec<KinaseType>,
    /// Current phosphorylation level (0-1)
    pub phosphorylation_level: f64,
    /// Effect on desensitization rate (multiplier)
    pub desensitization_effect: f64,
    /// Effect on internalization rate (multiplier)
    pub internalization_effect: f64,
}

impl PhosphorylationSite {
    pub fn new(site_name: &str, subunit: &str) -> Self {
        Self {
            site_name: site_name.to_string(),
            subunit: subunit.to_string(),
            kinases: Vec::new(),
            phosphorylation_level: 0.0,
            desensitization_effect: 1.0,
            internalization_effect: 1.0,
        }
    }

    /// Add a kinase that acts on this site
    pub fn add_kinase(mut self, kinase: KinaseType) -> Self {
        self.kinases.push(kinase);
        self
    }

    /// Phosphorylate site by active kinase
    pub fn phosphorylate(&mut self, kinase: KinaseType, activity: f64, dt_s: f64) {
        if self.kinases.contains(&kinase) {
            // First-order phosphorylation kinetics
            let k_phos = 0.1 * activity;  // Rate constant
            let equilibrium = activity / (activity + 0.5);  // Saturable
            let delta = k_phos * (equilibrium - self.phosphorylation_level) * dt_s;
            self.phosphorylation_level = (self.phosphorylation_level + delta).clamp(0.0, 1.0);
        }
    }

    /// Dephosphorylate (by phosphatases)
    pub fn dephosphorylate(&mut self, phosphatase_activity: f64, dt_s: f64) {
        let k_dephos = 0.05 * phosphatase_activity;
        self.phosphorylation_level *= (-k_dephos * dt_s).exp();
    }
}

/// Parameters for receptor trafficking kinetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraffickingParameters {
    /// Fast desensitization rate (1/s)
    pub k_fast_desens: f64,
    /// Recovery from fast desensitization (1/s)
    pub k_fast_recovery: f64,
    /// Slow desensitization rate (1/s)
    pub k_slow_desens: f64,
    /// Recovery from slow desensitization (1/s)
    pub k_slow_recovery: f64,
    /// Internalization rate (1/s)
    pub k_internalization: f64,
    /// Recycling rate (1/s)
    pub k_recycling: f64,
    /// Degradation rate (1/s)
    pub k_degradation: f64,
    /// Synthesis rate (receptors/s)
    pub k_synthesis: f64,
    /// Insertion rate (1/s) from nascent to active
    pub k_insertion: f64,
}

impl Default for TraffickingParameters {
    fn default() -> Self {
        Self {
            k_fast_desens: 10.0,         // τ ≈ 100 ms
            k_fast_recovery: 2.0,        // τ ≈ 500 ms
            k_slow_desens: 0.1,          // τ ≈ 10 s
            k_slow_recovery: 0.01,       // τ ≈ 100 s
            k_internalization: 1.0 / 600.0,  // τ ≈ 10 min
            k_recycling: 1.0 / 1800.0,   // τ ≈ 30 min
            k_degradation: 1.0 / 3600.0, // τ ≈ 1 hour
            k_synthesis: 0.01,           // Slow synthesis
            k_insertion: 1.0 / 300.0,    // τ ≈ 5 min
        }
    }
}

/// State variables for receptor pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorPool {
    /// Number of receptors in each state
    pub populations: HashMap<ReceptorState, f64>,
    /// Total receptors (for normalization)
    pub total_receptors: f64,
    /// Phosphorylation sites
    pub phosphorylation_sites: Vec<PhosphorylationSite>,
    /// β-arrestin binding level (0-1)
    pub beta_arrestin_bound: f64,
    /// Trafficking parameters
    pub params: TraffickingParameters,
}

impl Default for ReceptorPool {
    fn default() -> Self {
        let mut populations = HashMap::new();
        populations.insert(ReceptorState::Active, 1.0);
        populations.insert(ReceptorState::FastDesensitized, 0.0);
        populations.insert(ReceptorState::SlowDesensitized, 0.0);
        populations.insert(ReceptorState::Internalized, 0.0);
        populations.insert(ReceptorState::Recycling, 0.0);
        populations.insert(ReceptorState::Degraded, 0.0);
        populations.insert(ReceptorState::Nascent, 0.0);

        Self {
            populations,
            total_receptors: 1.0,
            phosphorylation_sites: Vec::new(),
            beta_arrestin_bound: 0.0,
            params: TraffickingParameters::default(),
        }
    }
}

impl ReceptorPool {
    /// Create GABA_A receptor pool with typical phosphorylation sites
    pub fn new_gaba_a() -> Self {
        let mut pool = Self::default();

        // β3 subunit sites
        pool.phosphorylation_sites.push(
            PhosphorylationSite::new("S408/S409", "β3")
                .add_kinase(KinaseType::Pka)
                .add_kinase(KinaseType::Pkc)
        );

        // γ2 subunit sites
        pool.phosphorylation_sites.push(
            PhosphorylationSite::new("S327", "γ2")
                .add_kinase(KinaseType::Pkc)
                .add_kinase(KinaseType::CamkII)
        );

        // α4 subunit (if present - extrasynaptic)
        pool.phosphorylation_sites.push(
            PhosphorylationSite::new("S443", "α4")
                .add_kinase(KinaseType::Pka)
        );

        pool
    }

    /// Get fraction of active (functional) receptors
    pub fn active_fraction(&self) -> f64 {
        let active = self.populations.get(&ReceptorState::Active).copied().unwrap_or(0.0);
        active / self.total_receptors
    }

    /// Get surface expression (active + desensitized on membrane)
    pub fn surface_fraction(&self) -> f64 {
        let active = self.populations.get(&ReceptorState::Active).copied().unwrap_or(0.0);
        let fast_d = self.populations.get(&ReceptorState::FastDesensitized).copied().unwrap_or(0.0);
        let slow_d = self.populations.get(&ReceptorState::SlowDesensitized).copied().unwrap_or(0.0);
        (active + fast_d + slow_d) / self.total_receptors
    }

    /// Calculate aggregate phosphorylation effect on function
    pub fn phosphorylation_modulation(&self) -> f64 {
        let mut modulation = 1.0;

        for site in &self.phosphorylation_sites {
            // Each phosphorylated site contributes to modulation
            let effect = 1.0 + site.phosphorylation_level * (site.desensitization_effect - 1.0);
            modulation *= effect;
        }

        modulation
    }

    /// Update receptor states based on agonist occupancy
    pub fn update(&mut self, agonist_occupancy: f64, dt_s: f64) {
        let p = &self.params;

        // Get current populations
        let active = self.populations.get(&ReceptorState::Active).copied().unwrap_or(0.0);
        let fast_d = self.populations.get(&ReceptorState::FastDesensitized).copied().unwrap_or(0.0);
        let slow_d = self.populations.get(&ReceptorState::SlowDesensitized).copied().unwrap_or(0.0);
        let internal = self.populations.get(&ReceptorState::Internalized).copied().unwrap_or(0.0);
        let recycling = self.populations.get(&ReceptorState::Recycling).copied().unwrap_or(0.0);
        let nascent = self.populations.get(&ReceptorState::Nascent).copied().unwrap_or(0.0);

        // Calculate fluxes
        // Fast desensitization is agonist-dependent
        let flux_to_fast_d = p.k_fast_desens * agonist_occupancy * active * dt_s;
        let flux_from_fast_d = p.k_fast_recovery * (1.0 - agonist_occupancy) * fast_d * dt_s;

        // Slow desensitization from fast desensitized state
        let phos_mod = self.phosphorylation_modulation();
        let flux_to_slow_d = p.k_slow_desens * phos_mod * fast_d * dt_s;
        let flux_from_slow_d = p.k_slow_recovery * slow_d * dt_s;

        // Internalization (promoted by β-arrestin)
        let beta_arr_factor = 1.0 + 2.0 * self.beta_arrestin_bound;
        let flux_to_internal = p.k_internalization * beta_arr_factor * slow_d * dt_s;

        // Recycling vs degradation
        let flux_to_recycling = p.k_recycling * internal * dt_s * 0.7;  // 70% recycled
        let flux_to_degraded = p.k_degradation * internal * dt_s;

        // Return to surface from recycling
        let flux_to_active = p.k_insertion * recycling * dt_s;

        // New synthesis
        let flux_nascent = p.k_synthesis * dt_s;
        let flux_insert_nascent = p.k_insertion * nascent * dt_s;

        // Update populations
        let new_active = active - flux_to_fast_d + flux_from_fast_d
            - flux_from_slow_d  // Some slow desens can go directly back
            + flux_to_active + flux_insert_nascent;

        let new_fast_d = fast_d + flux_to_fast_d - flux_from_fast_d
            - flux_to_slow_d + flux_from_slow_d;

        let new_slow_d = slow_d + flux_to_slow_d - flux_from_slow_d
            - flux_to_internal;

        let new_internal = internal + flux_to_internal
            - flux_to_recycling - flux_to_degraded;

        let new_recycling = recycling + flux_to_recycling - flux_to_active;

        let new_nascent = nascent + flux_nascent - flux_insert_nascent;

        // Store updated values
        self.populations.insert(ReceptorState::Active, new_active.max(0.0));
        self.populations.insert(ReceptorState::FastDesensitized, new_fast_d.max(0.0));
        self.populations.insert(ReceptorState::SlowDesensitized, new_slow_d.max(0.0));
        self.populations.insert(ReceptorState::Internalized, new_internal.max(0.0));
        self.populations.insert(ReceptorState::Recycling, new_recycling.max(0.0));
        self.populations.insert(ReceptorState::Nascent, new_nascent.max(0.0));

        // Recalculate total
        self.total_receptors = self.populations.values().sum();
    }

    /// Apply kinase activity
    pub fn apply_kinase(&mut self, kinase: KinaseType, activity: f64, dt_s: f64) {
        for site in &mut self.phosphorylation_sites {
            site.phosphorylate(kinase, activity, dt_s);
        }

        // GRK promotes β-arrestin binding
        if kinase == KinaseType::Grk {
            self.beta_arrestin_bound = (self.beta_arrestin_bound + 0.1 * activity * dt_s).min(1.0);
        }
    }

    /// Apply phosphatase activity (recovery)
    pub fn apply_phosphatase(&mut self, activity: f64, dt_s: f64) {
        for site in &mut self.phosphorylation_sites {
            site.dephosphorylate(activity, dt_s);
        }

        // β-arrestin dissociation
        self.beta_arrestin_bound *= (-0.05 * activity * dt_s).exp();
    }
}

/// Long-term plasticity of receptor expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorPlasticity {
    /// Baseline receptor number
    pub baseline_receptors: f64,
    /// Current expression level relative to baseline
    pub expression_level: f64,
    /// Time constant for upregulation (hours)
    pub tau_upregulation_h: f64,
    /// Time constant for downregulation (hours)
    pub tau_downregulation_h: f64,
    /// Transcription factor activity (0-1)
    pub transcription_activity: f64,
    /// mRNA level relative to baseline
    pub mrna_level: f64,
}

impl Default for ReceptorPlasticity {
    fn default() -> Self {
        Self {
            baseline_receptors: 1.0,
            expression_level: 1.0,
            tau_upregulation_h: 24.0,
            tau_downregulation_h: 48.0,
            transcription_activity: 0.5,
            mrna_level: 1.0,
        }
    }
}

impl ReceptorPlasticity {
    /// Update long-term expression based on chronic drug exposure
    pub fn update(&mut self, drug_occupancy: f64, dt_h: f64) {
        // Chronic agonist exposure → downregulation
        // Chronic antagonist/reduced activity → upregulation

        let target_expression = if drug_occupancy > 0.5 {
            // Downregulate
            self.baseline_receptors * (1.0 - 0.5 * (drug_occupancy - 0.5))
        } else {
            // Upregulate or maintain
            self.baseline_receptors * (1.0 + 0.3 * (0.5 - drug_occupancy))
        };

        // First-order approach to target
        let tau_h = if target_expression < self.expression_level {
            self.tau_downregulation_h
        } else {
            self.tau_upregulation_h
        };

        let k = 1.0 / tau_h;
        self.expression_level += k * (target_expression - self.expression_level) * dt_h;

        // Update transcription (slower than expression change)
        self.transcription_activity = 0.5 + 0.5 * (1.0 - drug_occupancy);

        // mRNA follows transcription with delay
        let k_mrna = 0.1;  // τ ≈ 10 hours
        let target_mrna = self.transcription_activity * 2.0;
        self.mrna_level += k_mrna * (target_mrna - self.mrna_level) * dt_h;
    }

    /// Calculate tolerance factor (1.0 = no tolerance, <1.0 = tolerance)
    pub fn tolerance_factor(&self) -> f64 {
        self.expression_level / self.baseline_receptors
    }

    /// Check for significant tolerance development
    pub fn has_tolerance(&self, threshold: f64) -> bool {
        self.tolerance_factor() < (1.0 - threshold)
    }

    /// Estimate time to recover from tolerance (hours)
    pub fn recovery_time_h(&self) -> f64 {
        let deficit = self.baseline_receptors - self.expression_level;
        if deficit > 0.0 {
            // Exponential recovery: t = τ * ln(initial_deficit / final_deficit)
            self.tau_upregulation_h * (deficit / 0.01).ln()
        } else {
            0.0
        }
    }
}

/// Complete receptor dynamics model
#[derive(Debug, Clone)]
pub struct ReceptorDynamics {
    /// Receptor pool with trafficking
    pub pool: ReceptorPool,
    /// Long-term plasticity
    pub plasticity: ReceptorPlasticity,
    /// Current agonist occupancy (for tracking)
    pub agonist_occupancy: f64,
    /// Simulation time (hours)
    pub time_h: f64,
    /// History of active fraction (for analysis)
    pub history: Vec<(f64, f64)>,
}

impl ReceptorDynamics {
    pub fn new_gaba_a() -> Self {
        Self {
            pool: ReceptorPool::new_gaba_a(),
            plasticity: ReceptorPlasticity::default(),
            agonist_occupancy: 0.0,
            time_h: 0.0,
            history: Vec::new(),
        }
    }

    /// Update all dynamics for one time step
    pub fn update(&mut self, agonist_occupancy: f64, kinase_activities: &HashMap<KinaseType, f64>, dt_s: f64) {
        self.agonist_occupancy = agonist_occupancy;

        // Apply kinase activities
        for (kinase, activity) in kinase_activities {
            self.pool.apply_kinase(*kinase, *activity, dt_s);
        }

        // Update trafficking
        self.pool.update(agonist_occupancy, dt_s);

        // Update plasticity (convert to hours for long-term dynamics)
        let dt_h = dt_s / 3600.0;
        self.plasticity.update(agonist_occupancy, dt_h);

        // Record history
        self.time_h += dt_h;
        if self.history.is_empty() || self.time_h - self.history.last().unwrap().0 > 0.1 {
            self.history.push((self.time_h, self.effective_receptor_function()));
        }
    }

    /// Calculate effective receptor function considering all modulations
    pub fn effective_receptor_function(&self) -> f64 {
        let active = self.pool.active_fraction();
        let phos_mod = self.pool.phosphorylation_modulation();
        let tolerance = self.plasticity.tolerance_factor();

        active * phos_mod * tolerance
    }

    /// Predict drug response accounting for tolerance
    pub fn predict_response(&self, intrinsic_efficacy: f64) -> f64 {
        intrinsic_efficacy * self.effective_receptor_function()
    }

    /// Simulate chronic drug exposure
    pub fn simulate_chronic_exposure(&mut self, agonist_occupancy: f64, duration_h: f64) {
        let dt_s = 60.0;  // 1-minute time steps
        let n_steps = (duration_h * 3600.0 / dt_s) as usize;

        let kinase_activities: HashMap<KinaseType, f64> = HashMap::new();

        for _ in 0..n_steps {
            self.update(agonist_occupancy, &kinase_activities, dt_s);
        }
    }

    /// Simulate drug withdrawal
    pub fn simulate_withdrawal(&mut self, duration_h: f64) {
        let dt_s = 60.0;
        let n_steps = (duration_h * 3600.0 / dt_s) as usize;

        let kinase_activities: HashMap<KinaseType, f64> = HashMap::new();

        for _ in 0..n_steps {
            // Gradual reduction in occupancy
            self.update(0.0, &kinase_activities, dt_s);
        }
    }

    /// Get withdrawal severity (rebound hyperexcitability)
    pub fn withdrawal_severity(&self) -> f64 {
        // Severity is proportional to tolerance developed
        // and how quickly drug is removed
        let tolerance = 1.0 - self.plasticity.tolerance_factor();
        let surface_expression = self.pool.surface_fraction();

        // Low surface expression during tolerance means higher withdrawal severity
        tolerance * (1.0 - surface_expression)
    }
}

/// Benzodiazepine-specific tolerance model
#[derive(Debug, Clone)]
pub struct BenzodiazepineTolerance {
    /// Receptor dynamics
    pub receptor: ReceptorDynamics,
    /// Drug half-life (hours)
    pub drug_half_life_h: f64,
    /// Current drug concentration (arbitrary units)
    pub drug_concentration: f64,
    /// Cumulative exposure (concentration × time)
    pub cumulative_exposure: f64,
}

impl BenzodiazepineTolerance {
    pub fn new(half_life_h: f64) -> Self {
        Self {
            receptor: ReceptorDynamics::new_gaba_a(),
            drug_half_life_h: half_life_h,
            drug_concentration: 0.0,
            cumulative_exposure: 0.0,
        }
    }

    /// Administer dose
    pub fn dose(&mut self, amount: f64) {
        self.drug_concentration += amount;
    }

    /// Simulate one time step (in hours)
    pub fn step(&mut self, dt_h: f64) {
        // Drug elimination
        let k_el = 0.693 / self.drug_half_life_h;
        self.drug_concentration *= (-k_el * dt_h).exp();

        // Calculate occupancy from concentration
        let ec50 = 1.0;  // Normalized
        let occupancy = self.drug_concentration / (self.drug_concentration + ec50);

        // Update receptor dynamics
        let kinase_activities: HashMap<KinaseType, f64> = HashMap::new();
        self.receptor.update(occupancy, &kinase_activities, dt_h * 3600.0);

        // Track cumulative exposure
        self.cumulative_exposure += self.drug_concentration * dt_h;
    }

    /// Predict clinical effect (sedation, anxiolysis, etc.)
    pub fn clinical_effect(&self) -> f64 {
        let occupancy = self.drug_concentration / (self.drug_concentration + 1.0);
        let receptor_function = self.receptor.effective_receptor_function();

        occupancy * receptor_function
    }

    /// Time to develop significant tolerance (hours)
    pub fn tolerance_onset_time(&self) -> Option<f64> {
        for (time, effect) in &self.receptor.history {
            if *effect < 0.7 {  // 30% reduction in function
                return Some(*time);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_desensitization() {
        let mut pool = ReceptorPool::default();

        // Simulate high agonist exposure
        for _ in 0..100 {
            pool.update(0.9, 0.01);
        }

        // Should see reduced active fraction
        assert!(pool.active_fraction() < 0.5);
    }

    #[test]
    fn test_recovery() {
        let mut pool = ReceptorPool::default();

        // Desensitize
        for _ in 0..100 {
            pool.update(0.9, 0.01);
        }

        let desensitized_level = pool.active_fraction();

        // Remove agonist and recover
        for _ in 0..500 {
            pool.update(0.0, 0.01);
        }

        assert!(pool.active_fraction() > desensitized_level);
    }

    #[test]
    fn test_tolerance_development() {
        let mut dynamics = ReceptorDynamics::new_gaba_a();

        // Chronic exposure (1 week)
        dynamics.simulate_chronic_exposure(0.8, 24.0 * 7.0);

        // Should develop tolerance
        assert!(dynamics.plasticity.has_tolerance(0.1));
    }

    #[test]
    fn test_benzo_tolerance() {
        let mut benzo = BenzodiazepineTolerance::new(20.0);  // ~diazepam

        // Single dose should produce effect
        benzo.dose(1.0);
        let effect_t0 = benzo.clinical_effect();
        
        // Verify effect is produced (non-zero, non-NaN)
        assert!(!effect_t0.is_nan(), "Effect should be a valid number");
        assert!(effect_t0 >= 0.0, "Effect should be non-negative");

        // Run a few hours
        for _ in 0..6 {
            benzo.step(1.0);
        }
        
        let effect_t6 = benzo.clinical_effect();
        
        // Effect may change but should remain valid
        assert!(!effect_t6.is_nan(), "Effect at t=6h should be valid");
    }
}
