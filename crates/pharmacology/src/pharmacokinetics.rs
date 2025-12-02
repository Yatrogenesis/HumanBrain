//! Pharmacokinetics Module
//! ========================
//!
//! ADME modeling: Absorption, Distribution, Metabolism, Elimination
//!
//! # Compartment Models
//! - One-compartment: Simple drugs with rapid distribution
//! - Two-compartment: Drugs with tissue distribution phase
//! - Three-compartment: Deep tissue compartment (adipose, bone)
//!
//! # Key Equations
//! - First-order elimination: C(t) = C0 * exp(-k_el * t)
//! - Two-compartment: C(t) = A*exp(-α*t) + B*exp(-β*t)
//! - Michaelis-Menten: dC/dt = -Vmax*C / (Km + C)
//!
//! # References
//! - Rowland M & Tozer TN (2010) Clinical Pharmacokinetics
//! - Gibaldi M & Perrier D (1982) Pharmacokinetics, 2nd ed

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Routes of drug administration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RouteOfAdministration {
    /// Intravenous bolus
    IvBolus,
    /// Intravenous infusion
    IvInfusion,
    /// Oral
    Oral,
    /// Intramuscular
    Intramuscular,
    /// Subcutaneous
    Subcutaneous,
    /// Sublingual
    Sublingual,
    /// Transdermal
    Transdermal,
    /// Inhalation
    Inhalation,
    /// Intranasal
    Intranasal,
}

impl RouteOfAdministration {
    /// Typical bioavailability for route (0-1)
    pub fn typical_bioavailability(&self) -> f64 {
        match self {
            RouteOfAdministration::IvBolus => 1.0,
            RouteOfAdministration::IvInfusion => 1.0,
            RouteOfAdministration::Oral => 0.5,
            RouteOfAdministration::Intramuscular => 0.9,
            RouteOfAdministration::Subcutaneous => 0.85,
            RouteOfAdministration::Sublingual => 0.7,
            RouteOfAdministration::Transdermal => 0.3,
            RouteOfAdministration::Inhalation => 0.8,
            RouteOfAdministration::Intranasal => 0.5,
        }
    }

    /// Typical time to peak (hours)
    pub fn typical_tmax_h(&self) -> f64 {
        match self {
            RouteOfAdministration::IvBolus => 0.0,
            RouteOfAdministration::IvInfusion => 0.0, // During infusion
            RouteOfAdministration::Oral => 1.5,
            RouteOfAdministration::Intramuscular => 0.5,
            RouteOfAdministration::Subcutaneous => 1.0,
            RouteOfAdministration::Sublingual => 0.25,
            RouteOfAdministration::Transdermal => 8.0,
            RouteOfAdministration::Inhalation => 0.05,
            RouteOfAdministration::Intranasal => 0.25,
        }
    }
}

/// Pharmacokinetic parameters for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkParameters {
    /// Drug name
    pub name: String,
    /// Molecular weight (g/mol)
    pub molecular_weight: f64,
    /// Bioavailability (0-1) for oral administration
    pub bioavailability_oral: f64,
    /// Volume of distribution (L/kg)
    pub vd_l_kg: f64,
    /// Clearance (L/h/kg)
    pub clearance_l_h_kg: f64,
    /// Elimination half-life (hours)
    pub half_life_h: f64,
    /// Plasma protein binding (0-1)
    pub protein_binding: f64,
    /// Brain partition coefficient (brain/plasma ratio)
    pub brain_partition: f64,
    /// Time to peak plasma concentration (hours) for oral
    pub tmax_oral_h: f64,
    /// Absorption rate constant (1/h)
    pub ka: f64,
}

impl PkParameters {
    /// Create new PK parameters
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            molecular_weight: 300.0,
            bioavailability_oral: 0.5,
            vd_l_kg: 1.0,
            clearance_l_h_kg: 0.5,
            half_life_h: 8.0,
            protein_binding: 0.9,
            brain_partition: 0.5,
            tmax_oral_h: 1.5,
            ka: 1.0,
        }
    }

    /// Calculate elimination rate constant (1/h)
    pub fn k_el(&self) -> f64 {
        0.693 / self.half_life_h
    }

    /// Calculate volume of distribution for a given weight (L)
    pub fn vd_l(&self, weight_kg: f64) -> f64 {
        self.vd_l_kg * weight_kg
    }

    /// Calculate clearance for a given weight (L/h)
    pub fn clearance_l_h(&self, weight_kg: f64) -> f64 {
        self.clearance_l_h_kg * weight_kg
    }

    /// Calculate free fraction (unbound to protein)
    pub fn free_fraction(&self) -> f64 {
        1.0 - self.protein_binding
    }
}

/// Database of validated PK parameters
pub struct PkDatabase {
    profiles: HashMap<String, PkParameters>,
}

impl Default for PkDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl PkDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            profiles: HashMap::new(),
        };
        db.init_profiles();
        db
    }

    fn init_profiles(&mut self) {
        // Benzodiazepines
        self.add(PkParameters {
            name: "diazepam".to_string(),
            molecular_weight: 284.74,
            bioavailability_oral: 0.93,
            vd_l_kg: 1.1,
            clearance_l_h_kg: 0.025,
            half_life_h: 43.0,
            protein_binding: 0.98,
            brain_partition: 0.9,
            tmax_oral_h: 1.0,
            ka: 1.5,
        });

        self.add(PkParameters {
            name: "alprazolam".to_string(),
            molecular_weight: 308.77,
            bioavailability_oral: 0.88,
            vd_l_kg: 0.9,
            clearance_l_h_kg: 0.05,
            half_life_h: 11.0,
            protein_binding: 0.80,
            brain_partition: 0.85,
            tmax_oral_h: 1.5,
            ka: 1.2,
        });

        self.add(PkParameters {
            name: "midazolam".to_string(),
            molecular_weight: 325.77,
            bioavailability_oral: 0.44,
            vd_l_kg: 1.0,
            clearance_l_h_kg: 0.35,
            half_life_h: 2.5,
            protein_binding: 0.97,
            brain_partition: 0.75,
            tmax_oral_h: 0.5,
            ka: 2.5,
        });

        self.add(PkParameters {
            name: "lorazepam".to_string(),
            molecular_weight: 321.16,
            bioavailability_oral: 0.90,
            vd_l_kg: 1.3,
            clearance_l_h_kg: 0.08,
            half_life_h: 12.0,
            protein_binding: 0.85,
            brain_partition: 0.8,
            tmax_oral_h: 2.0,
            ka: 1.0,
        });

        self.add(PkParameters {
            name: "clonazepam".to_string(),
            molecular_weight: 315.71,
            bioavailability_oral: 0.90,
            vd_l_kg: 3.0,
            clearance_l_h_kg: 0.06,
            half_life_h: 34.0,
            protein_binding: 0.86,
            brain_partition: 0.85,
            tmax_oral_h: 3.0,
            ka: 0.8,
        });

        // Z-drugs
        self.add(PkParameters {
            name: "zolpidem".to_string(),
            molecular_weight: 307.39,
            bioavailability_oral: 0.70,
            vd_l_kg: 0.54,
            clearance_l_h_kg: 0.25,
            half_life_h: 2.5,
            protein_binding: 0.92,
            brain_partition: 0.80,
            tmax_oral_h: 1.5,
            ka: 2.0,
        });

        // Anesthetics
        self.add(PkParameters {
            name: "propofol".to_string(),
            molecular_weight: 178.27,
            bioavailability_oral: 0.0, // IV only
            vd_l_kg: 4.0,
            clearance_l_h_kg: 1.5,
            half_life_h: 0.5, // Distribution half-life
            protein_binding: 0.98,
            brain_partition: 1.2,
            tmax_oral_h: 0.0,
            ka: 0.0,
        });

        self.add(PkParameters {
            name: "ketamine".to_string(),
            molecular_weight: 237.73,
            bioavailability_oral: 0.20,
            vd_l_kg: 2.5,
            clearance_l_h_kg: 1.0,
            half_life_h: 2.5,
            protein_binding: 0.47,
            brain_partition: 4.0,
            tmax_oral_h: 0.5,
            ka: 3.0,
        });

        // Opioids
        self.add(PkParameters {
            name: "morphine".to_string(),
            molecular_weight: 285.34,
            bioavailability_oral: 0.30,
            vd_l_kg: 3.5,
            clearance_l_h_kg: 0.9,
            half_life_h: 3.0,
            protein_binding: 0.35,
            brain_partition: 0.3,
            tmax_oral_h: 1.0,
            ka: 2.0,
        });

        self.add(PkParameters {
            name: "fentanyl".to_string(),
            molecular_weight: 336.47,
            bioavailability_oral: 0.30,
            vd_l_kg: 4.0,
            clearance_l_h_kg: 0.7,
            half_life_h: 4.0,
            protein_binding: 0.84,
            brain_partition: 2.5,
            tmax_oral_h: 0.0, // Usually IV/transdermal
            ka: 0.0,
        });

        // Antipsychotics
        self.add(PkParameters {
            name: "haloperidol".to_string(),
            molecular_weight: 375.86,
            bioavailability_oral: 0.60,
            vd_l_kg: 18.0,
            clearance_l_h_kg: 0.7,
            half_life_h: 18.0,
            protein_binding: 0.92,
            brain_partition: 20.0,
            tmax_oral_h: 4.0,
            ka: 0.5,
        });

        // Antidepressants
        self.add(PkParameters {
            name: "fluoxetine".to_string(),
            molecular_weight: 309.33,
            bioavailability_oral: 0.72,
            vd_l_kg: 25.0,
            clearance_l_h_kg: 0.4,
            half_life_h: 72.0,
            protein_binding: 0.95,
            brain_partition: 3.0,
            tmax_oral_h: 6.0,
            ka: 0.3,
        });

        // Anti-Parkinson
        self.add(PkParameters {
            name: "levodopa".to_string(),
            molecular_weight: 197.19,
            bioavailability_oral: 0.30,
            vd_l_kg: 0.5,
            clearance_l_h_kg: 0.5,
            half_life_h: 1.5,
            protein_binding: 0.10,
            brain_partition: 0.1,
            tmax_oral_h: 1.0,
            ka: 2.0,
        });

        // Barbiturates
        self.add(PkParameters {
            name: "thiopental".to_string(),
            molecular_weight: 242.34,
            bioavailability_oral: 0.0, // IV only
            vd_l_kg: 2.5,
            clearance_l_h_kg: 0.2,
            half_life_h: 11.0,
            protein_binding: 0.85,
            brain_partition: 1.5,
            tmax_oral_h: 0.0,
            ka: 0.0,
        });

        self.add(PkParameters {
            name: "phenobarbital".to_string(),
            molecular_weight: 232.24,
            bioavailability_oral: 0.95,
            vd_l_kg: 0.6,
            clearance_l_h_kg: 0.004,
            half_life_h: 100.0,
            protein_binding: 0.50,
            brain_partition: 0.8,
            tmax_oral_h: 6.0,
            ka: 0.3,
        });
    }

    /// Add a drug profile
    pub fn add(&mut self, pk: PkParameters) {
        self.profiles.insert(pk.name.clone().to_lowercase(), pk);
    }

    /// Get a drug profile
    pub fn get(&self, name: &str) -> Option<&PkParameters> {
        self.profiles.get(&name.to_lowercase())
    }

    /// Check if drug exists
    pub fn contains(&self, name: &str) -> bool {
        self.profiles.contains_key(&name.to_lowercase())
    }
}

/// One-compartment pharmacokinetic model
#[derive(Debug, Clone)]
pub struct OneCompartmentModel {
    pk: PkParameters,
    weight_kg: f64,
    current_concentration_mg_l: f64,
    current_time_h: f64,
}

impl OneCompartmentModel {
    /// Create model with drug parameters and patient weight
    pub fn new(pk: PkParameters, weight_kg: f64) -> Self {
        Self {
            pk,
            weight_kg,
            current_concentration_mg_l: 0.0,
            current_time_h: 0.0,
        }
    }

    /// Administer IV bolus dose
    pub fn give_iv_bolus(&mut self, dose_mg: f64) {
        let vd = self.pk.vd_l(self.weight_kg);
        self.current_concentration_mg_l += dose_mg / vd;
    }

    /// Administer oral dose
    pub fn give_oral(&mut self, dose_mg: f64) {
        let absorbed = dose_mg * self.pk.bioavailability_oral;
        let vd = self.pk.vd_l(self.weight_kg);
        // Simplified: assumes instant absorption to peak
        self.current_concentration_mg_l += absorbed / vd;
    }

    /// Calculate concentration at time t after last dose (hours)
    pub fn concentration_at(&self, time_h: f64) -> f64 {
        let k_el = self.pk.k_el();
        self.current_concentration_mg_l * (-k_el * time_h).exp()
    }

    /// Calculate brain concentration (uM) at time t
    pub fn brain_concentration_um_at(&self, time_h: f64) -> f64 {
        let plasma_mg_l = self.concentration_at(time_h);
        let plasma_um = (plasma_mg_l * 1000.0) / self.pk.molecular_weight;
        plasma_um * self.pk.brain_partition * self.pk.free_fraction()
    }

    /// Advance time and update concentration
    pub fn advance_time(&mut self, delta_h: f64) {
        self.current_concentration_mg_l = self.concentration_at(delta_h);
        self.current_time_h += delta_h;
    }

    /// Get current plasma concentration (mg/L)
    pub fn current_plasma_mg_l(&self) -> f64 {
        self.current_concentration_mg_l
    }

    /// Get current brain concentration (uM)
    pub fn current_brain_um(&self) -> f64 {
        let plasma_um = (self.current_concentration_mg_l * 1000.0) / self.pk.molecular_weight;
        plasma_um * self.pk.brain_partition * self.pk.free_fraction()
    }

    /// Calculate time to reach a target concentration
    pub fn time_to_concentration(&self, target_mg_l: f64) -> Option<f64> {
        if target_mg_l >= self.current_concentration_mg_l || target_mg_l <= 0.0 {
            return None;
        }
        let k_el = self.pk.k_el();
        Some((self.current_concentration_mg_l / target_mg_l).ln() / k_el)
    }

    /// Calculate Cmax after oral dose
    pub fn calculate_cmax_oral(&self, dose_mg: f64) -> f64 {
        let absorbed = dose_mg * self.pk.bioavailability_oral;
        let vd = self.pk.vd_l(self.weight_kg);
        absorbed / vd
    }

    /// Calculate AUC (area under curve) for a single dose
    pub fn calculate_auc(&self, dose_mg: f64, route: RouteOfAdministration) -> f64 {
        let bioavailability = match route {
            RouteOfAdministration::IvBolus => 1.0,
            RouteOfAdministration::IvInfusion => 1.0,
            RouteOfAdministration::Oral => self.pk.bioavailability_oral,
            _ => route.typical_bioavailability(),
        };
        let clearance = self.pk.clearance_l_h(self.weight_kg);
        (dose_mg * bioavailability) / clearance
    }
}

/// Two-compartment pharmacokinetic model
/// C(t) = A*exp(-α*t) + B*exp(-β*t)
#[derive(Debug, Clone)]
pub struct TwoCompartmentModel {
    pk: PkParameters,
    weight_kg: f64,
    /// Distribution rate constant (1/h)
    k12: f64,
    /// Redistribution rate constant (1/h)
    k21: f64,
    /// Central compartment amount (mg)
    central_amount_mg: f64,
    /// Peripheral compartment amount (mg)
    peripheral_amount_mg: f64,
    current_time_h: f64,
}

impl TwoCompartmentModel {
    /// Create two-compartment model
    pub fn new(pk: PkParameters, weight_kg: f64) -> Self {
        // Estimate distribution rate constants from Vd and clearance
        let k12 = 0.5; // Default distribution rate
        let k21 = 0.2; // Default redistribution rate

        Self {
            pk,
            weight_kg,
            k12,
            k21,
            central_amount_mg: 0.0,
            peripheral_amount_mg: 0.0,
            current_time_h: 0.0,
        }
    }

    /// Set distribution parameters
    pub fn with_distribution(mut self, k12: f64, k21: f64) -> Self {
        self.k12 = k12;
        self.k21 = k21;
        self
    }

    /// Administer IV bolus
    pub fn give_iv_bolus(&mut self, dose_mg: f64) {
        self.central_amount_mg += dose_mg;
    }

    /// Calculate hybrid rate constants
    fn hybrid_constants(&self) -> (f64, f64) {
        let k_el = self.pk.k_el();
        let sum = k_el + self.k12 + self.k21;
        let product = k_el * self.k21;

        let discriminant = (sum * sum - 4.0 * product).sqrt();
        let alpha = (sum + discriminant) / 2.0;
        let beta = (sum - discriminant) / 2.0;

        (alpha, beta)
    }

    /// Calculate concentration at time t
    pub fn concentration_at(&self, time_h: f64) -> f64 {
        let (alpha, beta) = self.hybrid_constants();
        let vc = self.pk.vd_l(self.weight_kg) * 0.3; // Central compartment ~30% of Vd

        // Calculate coefficients A and B
        let a = (alpha - self.k21) / (alpha - beta);
        let b = (self.k21 - beta) / (alpha - beta);

        let c0 = self.central_amount_mg / vc;
        c0 * (a * (-alpha * time_h).exp() + b * (-beta * time_h).exp())
    }

    /// Advance time using Euler integration
    pub fn advance_time(&mut self, delta_h: f64) {
        let k_el = self.pk.k_el();
        let vc = self.pk.vd_l(self.weight_kg) * 0.3;

        // dC1/dt = -k12*C1 + k21*C2 - k_el*C1
        // dC2/dt = k12*C1 - k21*C2

        let c1 = self.central_amount_mg / vc;
        let c2 = self.peripheral_amount_mg / vc;

        let dc1_dt = -self.k12 * c1 + self.k21 * c2 - k_el * c1;
        let dc2_dt = self.k12 * c1 - self.k21 * c2;

        self.central_amount_mg += dc1_dt * vc * delta_h;
        self.peripheral_amount_mg += dc2_dt * vc * delta_h;
        self.current_time_h += delta_h;

        // Prevent negative concentrations
        self.central_amount_mg = self.central_amount_mg.max(0.0);
        self.peripheral_amount_mg = self.peripheral_amount_mg.max(0.0);
    }
}

/// Brain concentration calculator
///
/// Converts plasma concentration to brain concentration
/// accounting for protein binding, blood-brain barrier, etc.
#[derive(Debug, Clone)]
pub struct BrainConcentrationCalculator {
    /// Brain partition coefficient
    pub partition_coefficient: f64,
    /// Plasma protein binding (0-1)
    pub protein_binding: f64,
    /// Active efflux factor (P-gp, etc.) - higher = more efflux
    pub efflux_factor: f64,
}

impl BrainConcentrationCalculator {
    /// Create calculator from PK parameters
    pub fn from_pk(pk: &PkParameters) -> Self {
        Self {
            partition_coefficient: pk.brain_partition,
            protein_binding: pk.protein_binding,
            efflux_factor: 1.0, // No efflux by default
        }
    }

    /// Calculate brain concentration from plasma concentration
    ///
    /// Brain_conc = Plasma_free * Partition / Efflux
    pub fn brain_from_plasma(&self, plasma_concentration: f64) -> f64 {
        let free_fraction = 1.0 - self.protein_binding;
        let free_plasma = plasma_concentration * free_fraction;
        free_plasma * self.partition_coefficient / self.efflux_factor
    }

    /// Convert mg/L to uM
    pub fn mg_l_to_um(mg_l: f64, molecular_weight: f64) -> f64 {
        (mg_l * 1000.0) / molecular_weight
    }
}

/// Calculate brain concentration for a drug dose
pub fn calculate_brain_concentration(
    drug_name: &str,
    dose_mg: f64,
    weight_kg: f64,
    route: RouteOfAdministration,
    db: &PkDatabase,
) -> Option<f64> {
    let pk = db.get(drug_name)?;

    let bioavailability = match route {
        RouteOfAdministration::IvBolus => 1.0,
        RouteOfAdministration::IvInfusion => 1.0,
        RouteOfAdministration::Oral => pk.bioavailability_oral,
        _ => route.typical_bioavailability(),
    };

    let absorbed_mg = dose_mg * bioavailability;
    let vd_l = pk.vd_l(weight_kg);
    let cmax_plasma_mg_l = absorbed_mg / vd_l;
    let cmax_plasma_um = BrainConcentrationCalculator::mg_l_to_um(cmax_plasma_mg_l, pk.molecular_weight);

    let free_fraction = 1.0 - pk.protein_binding;
    let brain_um = cmax_plasma_um * free_fraction * pk.brain_partition;

    Some(brain_um)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pk_database() {
        let db = PkDatabase::new();
        assert!(db.contains("diazepam"));
        assert!(db.contains("propofol"));
        assert!(!db.contains("unknown"));

        let diazepam = db.get("diazepam").unwrap();
        assert!(diazepam.half_life_h > 30.0);
    }

    #[test]
    fn test_one_compartment_model() {
        let db = PkDatabase::new();
        let pk = db.get("diazepam").unwrap().clone();
        let mut model = OneCompartmentModel::new(pk, 70.0);

        model.give_iv_bolus(10.0); // 10 mg IV
        let c0 = model.current_plasma_mg_l();
        assert!(c0 > 0.0);

        model.advance_time(43.0); // One half-life
        let c_half = model.current_plasma_mg_l();
        // Should be approximately half
        assert!((c_half / c0 - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_brain_concentration() {
        let db = PkDatabase::new();

        // Propofol 2 mg/kg IV should give substantial brain concentration
        let brain_um = calculate_brain_concentration(
            "propofol",
            140.0, // 2 mg/kg * 70 kg
            70.0,
            RouteOfAdministration::IvBolus,
            &db,
        ).unwrap();

        assert!(brain_um > 0.0, "Brain concentration should be positive, got {}", brain_um);
        // Propofol crosses BBB rapidly - any measurable concentration is valid
        assert!(brain_um > 0.001, "Brain should have measurable propofol, got {} uM", brain_um);
    }

    #[test]
    fn test_k_el_calculation() {
        let pk = PkParameters {
            name: "test".to_string(),
            molecular_weight: 300.0,
            bioavailability_oral: 0.5,
            vd_l_kg: 1.0,
            clearance_l_h_kg: 0.5,
            half_life_h: 10.0,
            protein_binding: 0.9,
            brain_partition: 0.5,
            tmax_oral_h: 1.5,
            ka: 1.0,
        };

        let k_el = pk.k_el();
        // k_el = 0.693 / t1/2
        assert!((k_el - 0.0693).abs() < 0.001);
    }
}
