//! Mechanistic Receptor Model
//! ==========================
//!
//! First-principles biophysical model for drug-receptor interactions.
//! Enables prediction of effects for novel compounds and drug combinations.
//!
//! # Key Concepts
//! - **BindingSite**: Physical location on receptor where drug binds
//! - **IntrinsicEfficacy**: How strongly binding translates to effect (0-1)
//! - **Cooperativity**: Hill coefficient for binding (1 = no cooperativity)
//! - **AllostericFactor**: How binding at one site affects other sites
//!
//! # References
//! - Sieghart W (1995) Structure and pharmacology of GABA_A receptors
//! - Olsen RW & Sieghart W (2008) GABA_A receptors: subtypes provide diversity
//! - PDSP Ki Database: https://pdsp.unc.edu/databases/kidb.php

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error types for receptor mechanisms
#[derive(Debug, Error)]
pub enum ReceptorError {
    #[error("Unknown drug: {0}")]
    UnknownDrug(String),
    #[error("Missing parameter: {0}")]
    MissingParameter(String),
    #[error("Invalid concentration: {0}")]
    InvalidConcentration(f64),
}

/// Physical binding sites on GABA_A receptor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingSite {
    /// Benzodiazepine site (alpha-gamma interface)
    BzSite,
    /// Propofol/etomidate site (beta subunit TM2-TM3)
    AnestheticSite,
    /// Pentobarbital binding site
    BarbituraSite,
    /// Allopregnanolone neurosteroid site
    NeurosteroidSite,
    /// Orthosteric GABA binding site
    GabaSite,
    /// Ion channel pore (antagonist site)
    PicrotoxinSite,
}

impl BindingSite {
    /// Default allosteric factor for each site
    pub fn default_allosteric_factor(&self) -> f64 {
        match self {
            BindingSite::BzSite => 1.2,
            BindingSite::AnestheticSite => 2.0,
            BindingSite::BarbituraSite => 1.8,
            BindingSite::NeurosteroidSite => 1.5,
            BindingSite::GabaSite => 1.0,
            BindingSite::PicrotoxinSite => 0.0, // Antagonist
        }
    }
}

/// Types of pharmacological effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    Sedation,
    Anxiolysis,
    Amnesia,
    MuscleRelaxation,
    Anticonvulsant,
    Anesthesia,
}

/// Molecular and pharmacological properties of a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugMolecularProfile {
    /// Drug name (lowercase)
    pub name: String,
    /// Primary binding site
    pub binding_site: BindingSite,
    /// Binding affinity in nM (lower = tighter binding)
    pub ki_nm: f64,
    /// Intrinsic efficacy (0 = antagonist, 1 = full agonist)
    pub intrinsic_efficacy: f64,
    /// Hill coefficient for cooperativity (>1 = positive cooperativity)
    pub hill_coefficient: f64,
    /// Effect on other sites when bound
    pub allosteric_factor: f64,
    /// Effect profile (effect type -> strength 0-1)
    pub effect_profile: HashMap<EffectType, f64>,
    /// Molecular weight in g/mol
    pub molecular_weight: f64,
    /// Lipophilicity (log octanol/water partition)
    pub log_p: f64,
}

impl DrugMolecularProfile {
    /// Create a new drug profile with default values
    pub fn new(name: &str, site: BindingSite, ki_nm: f64, efficacy: f64) -> Self {
        Self {
            name: name.to_lowercase(),
            binding_site: site,
            ki_nm,
            intrinsic_efficacy: efficacy,
            hill_coefficient: 1.0,
            allosteric_factor: site.default_allosteric_factor(),
            effect_profile: HashMap::new(),
            molecular_weight: 300.0,
            log_p: 2.0,
        }
    }

    /// Builder pattern: set Hill coefficient
    pub fn with_hill(mut self, hill: f64) -> Self {
        self.hill_coefficient = hill;
        self
    }

    /// Builder pattern: set allosteric factor
    pub fn with_allosteric(mut self, factor: f64) -> Self {
        self.allosteric_factor = factor;
        self
    }

    /// Builder pattern: add effect
    pub fn with_effect(mut self, effect: EffectType, strength: f64) -> Self {
        self.effect_profile.insert(effect, strength.clamp(0.0, 1.0));
        self
    }

    /// Builder pattern: set molecular weight
    pub fn with_mw(mut self, mw: f64) -> Self {
        self.molecular_weight = mw;
        self
    }

    /// Builder pattern: set logP
    pub fn with_log_p(mut self, log_p: f64) -> Self {
        self.log_p = log_p;
        self
    }
}

/// Database of validated drug profiles
pub struct DrugDatabase {
    profiles: HashMap<String, DrugMolecularProfile>,
}

impl Default for DrugDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugDatabase {
    /// Create database with all validated drug profiles
    pub fn new() -> Self {
        let mut db = Self {
            profiles: HashMap::new(),
        };
        db.init_benzodiazepines();
        db.init_anesthetics();
        db.init_barbiturates();
        db.init_z_drugs();
        db
    }

    fn init_benzodiazepines(&mut self) {
        // Diazepam - classic anxiolytic
        self.add(
            DrugMolecularProfile::new("diazepam", BindingSite::BzSite, 15.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Anxiolysis, 0.8)
                .with_effect(EffectType::Sedation, 0.5)
                .with_effect(EffectType::MuscleRelaxation, 0.6)
                .with_effect(EffectType::Anticonvulsant, 0.7)
                .with_effect(EffectType::Amnesia, 0.4)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(284.74)
                .with_log_p(2.82),
        );

        // Alprazolam - high-potency anxiolytic
        self.add(
            DrugMolecularProfile::new("alprazolam", BindingSite::BzSite, 5.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Anxiolysis, 0.9)
                .with_effect(EffectType::Sedation, 0.4)
                .with_effect(EffectType::MuscleRelaxation, 0.3)
                .with_effect(EffectType::Anticonvulsant, 0.5)
                .with_effect(EffectType::Amnesia, 0.5)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(308.77)
                .with_log_p(2.12),
        );

        // Clonazepam - anticonvulsant
        self.add(
            DrugMolecularProfile::new("clonazepam", BindingSite::BzSite, 2.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Anticonvulsant, 0.95)
                .with_effect(EffectType::Anxiolysis, 0.7)
                .with_effect(EffectType::Sedation, 0.6)
                .with_effect(EffectType::MuscleRelaxation, 0.5)
                .with_effect(EffectType::Amnesia, 0.3)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(315.71)
                .with_log_p(2.41),
        );

        // Midazolam - procedural sedation
        self.add(
            DrugMolecularProfile::new("midazolam", BindingSite::BzSite, 6.0, 0.65)
                .with_allosteric(1.3)
                .with_effect(EffectType::Amnesia, 0.95)
                .with_effect(EffectType::Sedation, 0.8)
                .with_effect(EffectType::Anxiolysis, 0.7)
                .with_effect(EffectType::Anticonvulsant, 0.6)
                .with_effect(EffectType::MuscleRelaxation, 0.3)
                .with_effect(EffectType::Anesthesia, 0.2)
                .with_mw(325.77)
                .with_log_p(3.89),
        );

        // Lorazepam - status epilepticus
        self.add(
            DrugMolecularProfile::new("lorazepam", BindingSite::BzSite, 3.0, 0.60)
                .with_allosteric(1.2)
                .with_effect(EffectType::Anxiolysis, 0.85)
                .with_effect(EffectType::Sedation, 0.7)
                .with_effect(EffectType::Amnesia, 0.8)
                .with_effect(EffectType::Anticonvulsant, 0.9)
                .with_effect(EffectType::MuscleRelaxation, 0.5)
                .with_effect(EffectType::Anesthesia, 0.15)
                .with_mw(321.16)
                .with_log_p(2.39),
        );

        // Bromazepam
        self.add(
            DrugMolecularProfile::new("bromazepam", BindingSite::BzSite, 20.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Anxiolysis, 0.85)
                .with_effect(EffectType::Sedation, 0.5)
                .with_effect(EffectType::MuscleRelaxation, 0.5)
                .with_effect(EffectType::Anticonvulsant, 0.4)
                .with_effect(EffectType::Amnesia, 0.3)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(316.15)
                .with_log_p(2.05),
        );

        // Triazolam - hypnotic
        self.add(
            DrugMolecularProfile::new("triazolam", BindingSite::BzSite, 4.0, 0.60)
                .with_allosteric(1.25)
                .with_effect(EffectType::Sedation, 0.9)
                .with_effect(EffectType::Amnesia, 0.85)
                .with_effect(EffectType::Anxiolysis, 0.6)
                .with_effect(EffectType::Anticonvulsant, 0.4)
                .with_effect(EffectType::MuscleRelaxation, 0.3)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(343.22)
                .with_log_p(2.42),
        );

        // Temazepam - hypnotic
        self.add(
            DrugMolecularProfile::new("temazepam", BindingSite::BzSite, 8.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Sedation, 0.85)
                .with_effect(EffectType::Anxiolysis, 0.6)
                .with_effect(EffectType::Amnesia, 0.5)
                .with_effect(EffectType::Anticonvulsant, 0.3)
                .with_effect(EffectType::MuscleRelaxation, 0.4)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(300.75)
                .with_log_p(2.19),
        );
    }

    fn init_anesthetics(&mut self) {
        // Propofol - IV anesthetic
        self.add(
            DrugMolecularProfile::new("propofol", BindingSite::AnestheticSite, 3500.0, 0.95)
                .with_hill(1.2)
                .with_allosteric(2.0)
                .with_effect(EffectType::Anesthesia, 0.95)
                .with_effect(EffectType::Sedation, 0.9)
                .with_effect(EffectType::Amnesia, 0.85)
                .with_effect(EffectType::Anticonvulsant, 0.6)
                .with_effect(EffectType::Anxiolysis, 0.3)
                .with_effect(EffectType::MuscleRelaxation, 0.2)
                .with_mw(178.27)
                .with_log_p(3.79),
        );

        // Etomidate - hemodynamically stable anesthetic
        self.add(
            DrugMolecularProfile::new("etomidate", BindingSite::AnestheticSite, 2000.0, 0.90)
                .with_hill(1.1)
                .with_allosteric(1.8)
                .with_effect(EffectType::Anesthesia, 0.90)
                .with_effect(EffectType::Sedation, 0.85)
                .with_effect(EffectType::Amnesia, 0.80)
                .with_effect(EffectType::Anticonvulsant, 0.5)
                .with_effect(EffectType::Anxiolysis, 0.2)
                .with_effect(EffectType::MuscleRelaxation, 0.1)
                .with_mw(244.29)
                .with_log_p(2.49),
        );

        // Sevoflurane - volatile anesthetic
        self.add(
            DrugMolecularProfile::new("sevoflurane", BindingSite::AnestheticSite, 260000.0, 0.90)
                .with_allosteric(2.0)
                .with_effect(EffectType::Anesthesia, 0.95)
                .with_effect(EffectType::Sedation, 0.95)
                .with_effect(EffectType::Amnesia, 0.9)
                .with_effect(EffectType::Anticonvulsant, 0.5)
                .with_effect(EffectType::MuscleRelaxation, 0.3)
                .with_effect(EffectType::Anxiolysis, 0.2)
                .with_mw(200.05)
                .with_log_p(2.42),
        );

        // Isoflurane - volatile anesthetic
        self.add(
            DrugMolecularProfile::new("isoflurane", BindingSite::AnestheticSite, 270000.0, 0.90)
                .with_allosteric(2.0)
                .with_effect(EffectType::Anesthesia, 0.95)
                .with_effect(EffectType::Sedation, 0.95)
                .with_effect(EffectType::Amnesia, 0.85)
                .with_effect(EffectType::Anticonvulsant, 0.4)
                .with_effect(EffectType::MuscleRelaxation, 0.3)
                .with_effect(EffectType::Anxiolysis, 0.2)
                .with_mw(184.49)
                .with_log_p(2.35),
        );

        // Desflurane - volatile anesthetic
        self.add(
            DrugMolecularProfile::new("desflurane", BindingSite::AnestheticSite, 380000.0, 0.85)
                .with_allosteric(1.9)
                .with_effect(EffectType::Anesthesia, 0.90)
                .with_effect(EffectType::Sedation, 0.90)
                .with_effect(EffectType::Amnesia, 0.8)
                .with_effect(EffectType::Anticonvulsant, 0.35)
                .with_effect(EffectType::MuscleRelaxation, 0.25)
                .with_effect(EffectType::Anxiolysis, 0.15)
                .with_mw(168.04)
                .with_log_p(2.08),
        );
    }

    fn init_barbiturates(&mut self) {
        // Thiopental - IV anesthetic (rapid induction)
        self.add(
            DrugMolecularProfile::new("thiopental", BindingSite::BarbituraSite, 5000.0, 0.95)
                .with_hill(1.1)
                .with_allosteric(2.2)
                .with_effect(EffectType::Anesthesia, 0.95)
                .with_effect(EffectType::Sedation, 0.95)
                .with_effect(EffectType::Amnesia, 0.85)
                .with_effect(EffectType::Anticonvulsant, 0.8)
                .with_effect(EffectType::MuscleRelaxation, 0.2)
                .with_effect(EffectType::Anxiolysis, 0.3)
                .with_mw(242.34)
                .with_log_p(2.85),
        );

        // Phenobarbital - anticonvulsant
        self.add(
            DrugMolecularProfile::new("phenobarbital", BindingSite::BarbituraSite, 15000.0, 0.70)
                .with_allosteric(1.8)
                .with_effect(EffectType::Anticonvulsant, 0.95)
                .with_effect(EffectType::Sedation, 0.7)
                .with_effect(EffectType::Anesthesia, 0.4)
                .with_effect(EffectType::Amnesia, 0.4)
                .with_effect(EffectType::Anxiolysis, 0.5)
                .with_effect(EffectType::MuscleRelaxation, 0.2)
                .with_mw(232.24)
                .with_log_p(1.47),
        );
    }

    fn init_z_drugs(&mut self) {
        // Zolpidem - alpha1-selective hypnotic
        self.add(
            DrugMolecularProfile::new("zolpidem", BindingSite::BzSite, 10.0, 0.60)
                .with_allosteric(1.3)
                .with_effect(EffectType::Sedation, 0.95)
                .with_effect(EffectType::Amnesia, 0.7)
                .with_effect(EffectType::Anxiolysis, 0.3)
                .with_effect(EffectType::Anticonvulsant, 0.2)
                .with_effect(EffectType::MuscleRelaxation, 0.2)
                .with_effect(EffectType::Anesthesia, 0.1)
                .with_mw(307.39)
                .with_log_p(3.0),
        );

        // Zaleplon - ultra-short-acting hypnotic
        self.add(
            DrugMolecularProfile::new("zaleplon", BindingSite::BzSite, 12.0, 0.55)
                .with_allosteric(1.2)
                .with_effect(EffectType::Sedation, 0.9)
                .with_effect(EffectType::Amnesia, 0.6)
                .with_effect(EffectType::Anxiolysis, 0.25)
                .with_effect(EffectType::Anticonvulsant, 0.15)
                .with_effect(EffectType::MuscleRelaxation, 0.15)
                .with_effect(EffectType::Anesthesia, 0.05)
                .with_mw(305.34)
                .with_log_p(1.23),
        );
    }

    /// Add a drug profile to the database
    pub fn add(&mut self, profile: DrugMolecularProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Get a drug profile by name
    pub fn get(&self, name: &str) -> Option<&DrugMolecularProfile> {
        self.profiles.get(&name.to_lowercase())
    }

    /// Check if drug exists in database
    pub fn contains(&self, name: &str) -> bool {
        self.profiles.contains_key(&name.to_lowercase())
    }

    /// Get all drug names
    pub fn drug_names(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    /// Get all profiles
    pub fn all_profiles(&self) -> &HashMap<String, DrugMolecularProfile> {
        &self.profiles
    }
}

/// Simulation result from receptor model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub mode: String,
    pub drug: String,
    pub concentration_um: f64,
    pub binding_site: BindingSite,
    pub ki_nm: f64,
    pub efficacy: f64,
    pub occupancy: f64,
    pub modulation: f64,
    pub beta_increase_pct: f64,
    pub sedation_pct: f64,
    pub effects: HashMap<EffectType, f64>,
}

/// Operating mode for the unified model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelMode {
    /// Use validated drug data from database
    Database,
    /// First-principles prediction from molecular properties
    Mechanistic,
}

/// First-principles GABA_A receptor model
///
/// Models the receptor as having multiple binding sites that can
/// independently bind drugs, with allosteric interactions between sites.
#[derive(Debug)]
pub struct MechanisticGabaAReceptor {
    /// Binding state for each site (0-1 occupancy)
    site_occupancy: HashMap<BindingSite, f64>,
    /// Drug currently bound at each site
    site_drugs: HashMap<BindingSite, Option<String>>,
    /// Combined receptor modulation factor
    total_modulation: f64,
    /// Effect outputs (0-1)
    effect_outputs: HashMap<EffectType, f64>,
}

impl Default for MechanisticGabaAReceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl MechanisticGabaAReceptor {
    /// Create a new receptor in unbound state
    pub fn new() -> Self {
        let mut site_occupancy = HashMap::new();
        let mut site_drugs = HashMap::new();
        let mut effect_outputs = HashMap::new();

        // Initialize all sites
        for site in [
            BindingSite::BzSite,
            BindingSite::AnestheticSite,
            BindingSite::BarbituraSite,
            BindingSite::NeurosteroidSite,
            BindingSite::GabaSite,
            BindingSite::PicrotoxinSite,
        ] {
            site_occupancy.insert(site, 0.0);
            site_drugs.insert(site, None);
        }

        // Initialize all effect types
        for effect in [
            EffectType::Sedation,
            EffectType::Anxiolysis,
            EffectType::Amnesia,
            EffectType::MuscleRelaxation,
            EffectType::Anticonvulsant,
            EffectType::Anesthesia,
        ] {
            effect_outputs.insert(effect, 0.0);
        }

        Self {
            site_occupancy,
            site_drugs,
            total_modulation: 1.0,
            effect_outputs,
        }
    }

    /// Reset receptor to unbound state
    pub fn reset(&mut self) {
        for occ in self.site_occupancy.values_mut() {
            *occ = 0.0;
        }
        for drug in self.site_drugs.values_mut() {
            *drug = None;
        }
        for effect in self.effect_outputs.values_mut() {
            *effect = 0.0;
        }
        self.total_modulation = 1.0;
    }

    /// Bind a drug to the receptor and calculate modulation
    ///
    /// # Arguments
    /// * `profile` - Drug molecular profile
    /// * `concentration_um` - Drug concentration in micromolar
    ///
    /// # Returns
    /// Modulation factor (>1 = enhancement of GABA effect)
    pub fn bind_drug(&mut self, profile: &DrugMolecularProfile, concentration_um: f64) -> f64 {
        let site = profile.binding_site;

        // Convert Ki (nM) to IC50 (uM) - rough approximation
        let ic50_um = profile.ki_nm / 1000.0;

        // Hill equation for binding
        let hill = profile.hill_coefficient;
        let occupancy = concentration_um.powf(hill) / (ic50_um.powf(hill) + concentration_um.powf(hill));

        // Update site state
        self.site_occupancy.insert(site, occupancy);
        self.site_drugs.insert(site, Some(profile.name.clone()));

        // Calculate modulation based on efficacy
        // Modulation = 1 + (efficacy * occupancy * allosteric_factor)
        let modulation = 1.0 + (profile.intrinsic_efficacy * occupancy * profile.allosteric_factor);

        self.total_modulation = modulation;

        // Calculate effect outputs
        for (effect_type, strength) in &profile.effect_profile {
            self.effect_outputs.insert(*effect_type, *strength * occupancy);
        }

        modulation
    }

    /// Bind multiple drugs and calculate combined effect
    ///
    /// # Arguments
    /// * `drugs` - Slice of (profile, concentration) tuples
    ///
    /// # Returns
    /// Combined modulation factor accounting for synergy/competition
    pub fn bind_multiple_drugs(&mut self, drugs: &[(&DrugMolecularProfile, f64)]) -> f64 {
        self.reset();

        let mut total_modulation = 1.0;
        let mut combined_effects: HashMap<EffectType, f64> = HashMap::new();
        let mut sites_seen: Vec<BindingSite> = Vec::new();

        for (profile, concentration) in drugs {
            let site = profile.binding_site;

            // Calculate binding
            let ic50_um = profile.ki_nm / 1000.0;
            let hill = profile.hill_coefficient;
            let occupancy = concentration.powf(hill) / (ic50_um.powf(hill) + concentration.powf(hill));

            // Store occupancy
            self.site_occupancy.insert(site, occupancy);
            self.site_drugs.insert(site, Some(profile.name.clone()));

            // Calculate modulation contribution
            let site_mod = 1.0 + (profile.intrinsic_efficacy * occupancy * profile.allosteric_factor);

            // Check for synergy (same site = compete, different site = synergy)
            if sites_seen.contains(&site) {
                // Competition - weighted average (subadditive)
                total_modulation = (total_modulation + site_mod) / 2.0;
            } else {
                // Synergy - multiplicative (superadditive)
                total_modulation *= site_mod;
            }

            sites_seen.push(site);

            // Combine effects (take maximum)
            for (effect_type, strength) in &profile.effect_profile {
                let current = combined_effects.get(effect_type).copied().unwrap_or(0.0);
                combined_effects.insert(*effect_type, current.max(*strength * occupancy));
            }
        }

        self.total_modulation = total_modulation;
        self.effect_outputs = combined_effects;

        total_modulation
    }

    /// Get the strength of a specific effect (0-1)
    pub fn get_effect(&self, effect_type: EffectType) -> f64 {
        *self.effect_outputs.get(&effect_type).unwrap_or(&0.0)
    }

    /// Calculate EEG beta power increase percentage
    ///
    /// Beta increase correlates with GABAergic enhancement
    pub fn get_beta_power_increase(&self) -> f64 {
        100.0 * (self.total_modulation - 1.0)
    }

    /// Calculate sedation/EEG suppression percentage
    ///
    /// Uses sigmoid relationship: higher modulation = more suppression
    pub fn get_sedation_percentage(&self) -> f64 {
        let mod_val = self.total_modulation;
        if mod_val <= 1.0 {
            return 0.0;
        }

        // Sigmoid parameters calibrated to clinical data
        let half_max = 1.8; // Modulation at 50% sedation
        let steepness = 3.0;

        let suppression = 100.0 / (1.0 + (-steepness * (mod_val - half_max)).exp());
        suppression.min(100.0)
    }

    /// Predict effect of a novel compound based on its properties
    ///
    /// This is the key function for predicting unknown drugs:
    /// Given molecular properties, predict the receptor effect.
    pub fn predict_novel_drug(
        &self,
        ki_nm: f64,
        site: BindingSite,
        efficacy: f64,
        concentration_um: f64,
    ) -> SimulationResult {
        // Calculate binding
        let ic50_um = ki_nm / 1000.0;
        let occupancy = concentration_um / (ic50_um + concentration_um);

        // Get allosteric factor for this site
        let allosteric = site.default_allosteric_factor();

        // Calculate modulation
        let modulation = 1.0 + (efficacy * occupancy * allosteric);

        // Estimate sedation
        let sedation_pct = if modulation <= 1.0 {
            0.0
        } else {
            let half_max = 1.8;
            let steepness = 3.0;
            (100.0 / (1.0 + (-steepness * (modulation - half_max)).exp())).min(100.0)
        };

        SimulationResult {
            mode: "MECHANISTIC".to_string(),
            drug: "novel_compound".to_string(),
            concentration_um,
            binding_site: site,
            ki_nm,
            efficacy,
            occupancy,
            modulation,
            beta_increase_pct: 100.0 * (modulation - 1.0),
            sedation_pct,
            effects: HashMap::new(),
        }
    }

    /// Get current total modulation
    pub fn total_modulation(&self) -> f64 {
        self.total_modulation
    }

    /// Get occupancy at a specific site
    pub fn site_occupancy(&self, site: BindingSite) -> f64 {
        *self.site_occupancy.get(&site).unwrap_or(&0.0)
    }
}

/// Dual-mode GABA_A receptor model
///
/// DATABASE mode: Uses validated drug profiles from DrugDatabase
/// MECHANISTIC mode: Predicts from first principles (Ki, efficacy, site)
pub struct UnifiedGabaAModel {
    mode: ModelMode,
    receptor: MechanisticGabaAReceptor,
    database: DrugDatabase,
}

impl UnifiedGabaAModel {
    /// Create a new model with specified mode
    pub fn new(mode: ModelMode) -> Self {
        Self {
            mode,
            receptor: MechanisticGabaAReceptor::new(),
            database: DrugDatabase::new(),
        }
    }

    /// Create model in DATABASE mode
    pub fn database_mode() -> Self {
        Self::new(ModelMode::Database)
    }

    /// Create model in MECHANISTIC mode
    pub fn mechanistic_mode() -> Self {
        Self::new(ModelMode::Mechanistic)
    }

    /// Simulate drug effect in DATABASE mode
    pub fn simulate_drug(
        &mut self,
        drug_name: &str,
        concentration_um: f64,
    ) -> Result<SimulationResult, ReceptorError> {
        let profile = self
            .database
            .get(drug_name)
            .ok_or_else(|| ReceptorError::UnknownDrug(drug_name.to_string()))?
            .clone();

        self.receptor.reset();
        let modulation = self.receptor.bind_drug(&profile, concentration_um);

        Ok(SimulationResult {
            mode: "DATABASE".to_string(),
            drug: drug_name.to_string(),
            concentration_um,
            binding_site: profile.binding_site,
            ki_nm: profile.ki_nm,
            efficacy: profile.intrinsic_efficacy,
            occupancy: self.receptor.site_occupancy(profile.binding_site),
            modulation,
            beta_increase_pct: self.receptor.get_beta_power_increase(),
            sedation_pct: self.receptor.get_sedation_percentage(),
            effects: self.receptor.effect_outputs.clone(),
        })
    }

    /// Simulate novel drug in MECHANISTIC mode
    pub fn simulate_novel(
        &mut self,
        ki_nm: f64,
        efficacy: f64,
        binding_site: BindingSite,
        concentration_um: f64,
    ) -> SimulationResult {
        self.receptor.predict_novel_drug(ki_nm, binding_site, efficacy, concentration_um)
    }

    /// Simulate drug-drug interaction
    pub fn simulate_interaction(
        &mut self,
        drugs: &[(&str, f64)],
    ) -> Result<InteractionResult, ReceptorError> {
        let mut profiles_and_concs: Vec<(&DrugMolecularProfile, f64)> = Vec::new();

        for (name, conc) in drugs {
            let profile = self
                .database
                .get(name)
                .ok_or_else(|| ReceptorError::UnknownDrug(name.to_string()))?;
            profiles_and_concs.push((profile, *conc));
        }

        self.receptor.reset();
        let combined_mod = self.receptor.bind_multiple_drugs(&profiles_and_concs);

        // Determine interaction type
        let sites: Vec<BindingSite> = profiles_and_concs.iter().map(|(p, _)| p.binding_site).collect();
        let unique_sites: std::collections::HashSet<_> = sites.iter().collect();
        let interaction_type = if unique_sites.len() > 1 {
            InteractionType::Synergy
        } else {
            InteractionType::Competition
        };

        Ok(InteractionResult {
            drugs: drugs.iter().map(|(n, c)| (n.to_string(), *c)).collect(),
            sites_involved: unique_sites.into_iter().copied().collect(),
            interaction_type,
            combined_modulation: combined_mod,
            beta_increase_pct: self.receptor.get_beta_power_increase(),
            sedation_pct: self.receptor.get_sedation_percentage(),
            effects: self.receptor.effect_outputs.clone(),
        })
    }

    /// Get reference to drug database
    pub fn database(&self) -> &DrugDatabase {
        &self.database
    }

    /// Get mutable reference to drug database (for adding custom drugs)
    pub fn database_mut(&mut self) -> &mut DrugDatabase {
        &mut self.database
    }
}

/// Type of drug-drug interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// Different binding sites - multiplicative effect
    Synergy,
    /// Same binding site - competitive (subadditive)
    Competition,
}

/// Result of drug-drug interaction simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    pub drugs: Vec<(String, f64)>,
    pub sites_involved: Vec<BindingSite>,
    pub interaction_type: InteractionType,
    pub combined_modulation: f64,
    pub beta_increase_pct: f64,
    pub sedation_pct: f64,
    pub effects: HashMap<EffectType, f64>,
}

/// Reverse engineering: Infer drugs from observed effects
///
/// Use cases:
/// - Unknown intoxication diagnosis
/// - Forensic toxicology
/// - Drug interaction investigation
pub struct EffectReverseEngineer {
    database: DrugDatabase,
}

impl Default for EffectReverseEngineer {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectReverseEngineer {
    pub fn new() -> Self {
        Self {
            database: DrugDatabase::new(),
        }
    }

    /// Given observed sedation %, find drugs that could cause it
    ///
    /// Returns list of candidate drugs with estimated concentrations.
    pub fn infer_from_sedation(&self, sedation_pct: f64, tolerance: f64) -> Vec<DrugCandidate> {
        let mut candidates = Vec::new();

        for (drug_name, profile) in self.database.all_profiles() {
            let mut receptor = MechanisticGabaAReceptor::new();

            // Binary search for concentration that produces target sedation
            let mut conc_low = 0.001_f64;
            let mut conc_high = 100.0_f64;
            let mut best_conc = 0.0;
            let mut best_error = f64::MAX;

            for _ in 0..20 {
                let conc_mid = (conc_low + conc_high) / 2.0;
                receptor.reset();
                receptor.bind_drug(profile, conc_mid);
                let predicted = receptor.get_sedation_percentage();

                let error = (predicted - sedation_pct).abs();
                if error < best_error {
                    best_error = error;
                    best_conc = conc_mid;
                }

                if predicted < sedation_pct {
                    conc_low = conc_mid;
                } else {
                    conc_high = conc_mid;
                }
            }

            if best_error <= tolerance {
                let plausibility = if (0.01..50.0).contains(&best_conc) {
                    ClinicalPlausibility::High
                } else {
                    ClinicalPlausibility::Low
                };

                candidates.push(DrugCandidate {
                    drug: drug_name.clone(),
                    estimated_concentration_um: best_conc,
                    predicted_value: sedation_pct,
                    error_pct: best_error,
                    binding_site: profile.binding_site,
                    clinical_plausibility: plausibility,
                });
            }
        }

        // Sort by clinical plausibility then error
        candidates.sort_by(|a, b| {
            match (a.clinical_plausibility, b.clinical_plausibility) {
                (ClinicalPlausibility::High, ClinicalPlausibility::Low) => std::cmp::Ordering::Less,
                (ClinicalPlausibility::Low, ClinicalPlausibility::High) => std::cmp::Ordering::Greater,
                _ => a.error_pct.partial_cmp(&b.error_pct).unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        candidates
    }

    /// Given observed EEG beta increase %, find drugs that could cause it
    pub fn infer_from_beta_increase(&self, beta_increase_pct: f64, tolerance: f64) -> Vec<DrugCandidate> {
        let mut candidates = Vec::new();

        for (drug_name, profile) in self.database.all_profiles() {
            let mut receptor = MechanisticGabaAReceptor::new();

            let mut conc_low = 0.001_f64;
            let mut conc_high = 100.0_f64;
            let mut best_conc = 0.0;
            let mut best_error = f64::MAX;

            for _ in 0..20 {
                let conc_mid = (conc_low + conc_high) / 2.0;
                receptor.reset();
                receptor.bind_drug(profile, conc_mid);
                let predicted = receptor.get_beta_power_increase();

                let error = (predicted - beta_increase_pct).abs();
                if error < best_error {
                    best_error = error;
                    best_conc = conc_mid;
                }

                if predicted < beta_increase_pct {
                    conc_low = conc_mid;
                } else {
                    conc_high = conc_mid;
                }
            }

            if best_error <= tolerance {
                candidates.push(DrugCandidate {
                    drug: drug_name.clone(),
                    estimated_concentration_um: best_conc,
                    predicted_value: beta_increase_pct,
                    error_pct: best_error,
                    binding_site: profile.binding_site,
                    clinical_plausibility: if (0.01..50.0).contains(&best_conc) {
                        ClinicalPlausibility::High
                    } else {
                        ClinicalPlausibility::Low
                    },
                });
            }
        }

        candidates.sort_by(|a, b| a.error_pct.partial_cmp(&b.error_pct).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    /// Given a pattern of effects, find the best matching drug(s)
    pub fn infer_from_effect_pattern(&self, effects: &HashMap<EffectType, f64>) -> Vec<EffectMatch> {
        let mut candidates = Vec::new();

        for (drug_name, profile) in self.database.all_profiles() {
            let mut match_score = 0.0_f64;
            let mut total_weight = 0.0_f64;

            for (effect_type, observed_value) in effects {
                if let Some(&drug_effect) = profile.effect_profile.get(effect_type) {
                    // Score based on how close the effect is
                    let match_val = 1.0 - (drug_effect - observed_value).abs();
                    match_score += match_val.max(0.0);
                    total_weight += 1.0;
                }
            }

            if total_weight > 0.0 {
                let avg_match = match_score / total_weight;
                if avg_match > 0.5 {
                    candidates.push(EffectMatch {
                        drug: drug_name.clone(),
                        match_score: avg_match,
                        binding_site: profile.binding_site,
                        drug_effect_profile: profile.effect_profile.clone(),
                    });
                }
            }
        }

        candidates.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }
}

/// Clinical plausibility of a drug candidate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClinicalPlausibility {
    High,
    Low,
}

/// Candidate drug from reverse engineering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugCandidate {
    pub drug: String,
    pub estimated_concentration_um: f64,
    pub predicted_value: f64,
    pub error_pct: f64,
    pub binding_site: BindingSite,
    pub clinical_plausibility: ClinicalPlausibility,
}

/// Match result from effect pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectMatch {
    pub drug: String,
    pub match_score: f64,
    pub binding_site: BindingSite,
    pub drug_effect_profile: HashMap<EffectType, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drug_database() {
        let db = DrugDatabase::new();
        assert!(db.contains("diazepam"));
        assert!(db.contains("propofol"));
        assert!(db.contains("zolpidem"));
        assert!(!db.contains("unknown_drug"));

        let diazepam = db.get("diazepam").unwrap();
        assert_eq!(diazepam.binding_site, BindingSite::BzSite);
        assert!((diazepam.ki_nm - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_receptor_binding() {
        let db = DrugDatabase::new();
        let mut receptor = MechanisticGabaAReceptor::new();

        let propofol = db.get("propofol").unwrap();
        let modulation = receptor.bind_drug(propofol, 5.0); // 5 uM

        // Propofol at 5 uM should produce significant modulation
        assert!(modulation > 1.5);
        assert!(receptor.get_sedation_percentage() > 30.0);
    }

    #[test]
    fn test_unified_model_database_mode() {
        let mut model = UnifiedGabaAModel::database_mode();

        let result = model.simulate_drug("diazepam", 0.5).unwrap();
        assert_eq!(result.mode, "DATABASE");
        assert!(result.occupancy > 0.0);
        assert!(result.modulation > 1.0);
    }

    #[test]
    fn test_drug_interaction_synergy() {
        let mut model = UnifiedGabaAModel::database_mode();

        // Diazepam (BZ site) + Propofol (anesthetic site) = synergy
        let result = model.simulate_interaction(&[
            ("diazepam", 0.2),
            ("propofol", 2.0),
        ]).unwrap();

        assert_eq!(result.interaction_type, InteractionType::Synergy);
        assert!(result.combined_modulation > 1.5);
    }

    #[test]
    fn test_drug_interaction_competition() {
        let mut model = UnifiedGabaAModel::database_mode();

        // Alprazolam + Zolpidem = both BZ site = competition
        let result = model.simulate_interaction(&[
            ("alprazolam", 0.1),
            ("zolpidem", 0.5),
        ]).unwrap();

        assert_eq!(result.interaction_type, InteractionType::Competition);
    }

    #[test]
    fn test_reverse_engineering() {
        let engineer = EffectReverseEngineer::new();

        // Given ~50% sedation, should find propofol-like drugs
        let candidates = engineer.infer_from_sedation(50.0, 15.0);
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_novel_drug_prediction() {
        let mut model = UnifiedGabaAModel::mechanistic_mode();

        // Hypothetical high-affinity, high-efficacy BZ
        let result = model.simulate_novel(
            5.0,   // Ki 5 nM (high affinity)
            0.8,   // High efficacy
            BindingSite::BzSite,
            0.1,   // 0.1 uM
        );

        assert!(result.occupancy > 0.5);
        assert!(result.modulation > 1.0);
    }
}
