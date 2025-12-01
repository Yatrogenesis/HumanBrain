//! Active Transport and Efflux Pump Dynamics
//! ===========================================
//!
//! Models ATP-dependent drug transport including:
//! - P-glycoprotein (MDR1/ABCB1) efflux at BBB
//! - Breast Cancer Resistance Protein (BCRP/ABCG2)
//! - Multidrug Resistance Proteins (MRP1-9)
//! - Organic Anion/Cation Transporters (OAT/OCT)
//! - Peptide transporters (PEPT1/2)
//!
//! # Blood-Brain Barrier Transport
//!
//! ```text
//!                 ┌─────────────────────────────────────┐
//!    BLOOD        │         BRAIN ENDOTHELIUM           │        BRAIN
//!                 │                                     │
//!   Drug ───────►│────► Passive Diffusion ─────────────│────► Free Drug
//!                 │                                     │
//!   Drug ◄───────│◄──── P-gp Efflux ◄──────────────────│◄──── Drug-Bound
//!                 │         (ATP)                       │
//!                 │                                     │
//!   Drug ───────►│────► SLC Influx ────────────────────│────► Drug
//!                 │     (OAT, OCT, PEPT)                │
//!                 └─────────────────────────────────────┘
//! ```
//!
//! # Michaelis-Menten for Active Transport
//!
//! ```text
//! J = Jmax * [S] / (Km + [S])
//! ```
//!
//! With ATP dependence:
//! ```text
//! J = Jmax * [S] * [ATP] / ((Km + [S]) * (Km_ATP + [ATP]))
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of membrane transporters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransporterType {
    /// P-glycoprotein (MDR1/ABCB1) - broad substrate specificity
    Pgp,
    /// Breast Cancer Resistance Protein
    Bcrp,
    /// Multidrug Resistance Protein 1
    Mrp1,
    /// Multidrug Resistance Protein 2 (canalicular)
    Mrp2,
    /// Organic Anion Transporter 1
    Oat1,
    /// Organic Anion Transporter 3
    Oat3,
    /// Organic Cation Transporter 1
    Oct1,
    /// Organic Cation Transporter 2
    Oct2,
    /// Peptide Transporter 1
    Pept1,
    /// Peptide Transporter 2
    Pept2,
    /// GABA Transporter 1 (GAT1)
    Gat1,
    /// GABA Transporter 3 (GAT3)
    Gat3,
    /// Serotonin Transporter (SERT)
    Sert,
    /// Dopamine Transporter (DAT)
    Dat,
    /// Norepinephrine Transporter (NET)
    Net,
    /// Glutamate Transporter (EAAT)
    Eaat,
    /// Glucose Transporter 1 (GLUT1)
    Glut1,
    /// Large Neutral Amino Acid Transporter (LAT1)
    Lat1,
}

impl TransporterType {
    /// Is this an efflux transporter (ATP-dependent)?
    pub fn is_efflux(&self) -> bool {
        matches!(self, TransporterType::Pgp | TransporterType::Bcrp |
                       TransporterType::Mrp1 | TransporterType::Mrp2)
    }

    /// Is this an influx transporter?
    pub fn is_influx(&self) -> bool {
        matches!(self, TransporterType::Oat1 | TransporterType::Oat3 |
                       TransporterType::Oct1 | TransporterType::Oct2 |
                       TransporterType::Pept1 | TransporterType::Pept2 |
                       TransporterType::Lat1 | TransporterType::Glut1)
    }

    /// Is this a neurotransmitter reuptake transporter?
    pub fn is_reuptake(&self) -> bool {
        matches!(self, TransporterType::Gat1 | TransporterType::Gat3 |
                       TransporterType::Sert | TransporterType::Dat |
                       TransporterType::Net | TransporterType::Eaat)
    }

    /// Typical Km (µM) for prototype substrate
    pub fn typical_km_um(&self) -> f64 {
        match self {
            TransporterType::Pgp => 1.0,
            TransporterType::Bcrp => 2.0,
            TransporterType::Mrp1 => 5.0,
            TransporterType::Mrp2 => 3.0,
            TransporterType::Oat1 => 10.0,
            TransporterType::Oat3 => 15.0,
            TransporterType::Oct1 => 50.0,
            TransporterType::Oct2 => 30.0,
            TransporterType::Pept1 => 100.0,
            TransporterType::Pept2 => 50.0,
            TransporterType::Gat1 => 10.0,
            TransporterType::Gat3 => 15.0,
            TransporterType::Sert => 0.4,    // High affinity
            TransporterType::Dat => 0.5,
            TransporterType::Net => 0.3,
            TransporterType::Eaat => 20.0,
            TransporterType::Glut1 => 5000.0,  // mM range, glucose
            TransporterType::Lat1 => 50.0,
        }
    }

    /// Typical Jmax (pmol/min/cm²)
    pub fn typical_jmax(&self) -> f64 {
        match self {
            TransporterType::Pgp => 100.0,
            TransporterType::Bcrp => 50.0,
            TransporterType::Mrp1 => 30.0,
            TransporterType::Mrp2 => 40.0,
            TransporterType::Oat1 => 200.0,
            TransporterType::Oat3 => 150.0,
            TransporterType::Oct1 => 100.0,
            TransporterType::Oct2 => 120.0,
            TransporterType::Pept1 => 500.0,
            TransporterType::Pept2 => 300.0,
            TransporterType::Gat1 => 50.0,
            TransporterType::Gat3 => 30.0,
            TransporterType::Sert => 10.0,
            TransporterType::Dat => 15.0,
            TransporterType::Net => 12.0,
            TransporterType::Eaat => 100.0,
            TransporterType::Glut1 => 1000.0,
            TransporterType::Lat1 => 200.0,
        }
    }

    /// ATP molecules consumed per substrate molecule transported
    pub fn atp_stoichiometry(&self) -> f64 {
        match self {
            TransporterType::Pgp => 2.0,
            TransporterType::Bcrp => 1.0,
            TransporterType::Mrp1 => 1.0,
            TransporterType::Mrp2 => 1.0,
            // Non-ATP dependent transporters
            _ => 0.0,
        }
    }
}

/// Kinetic parameters for a transporter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransporterKinetics {
    /// Transporter type
    pub transporter_type: TransporterType,
    /// Maximum transport rate (pmol/min/cm²)
    pub jmax: f64,
    /// Michaelis constant for drug (µM)
    pub km_drug: f64,
    /// Michaelis constant for ATP (mM) - for efflux pumps
    pub km_atp: f64,
    /// Hill coefficient for cooperativity
    pub hill_coefficient: f64,
    /// Expression level relative to reference
    pub expression_level: f64,
}

impl TransporterKinetics {
    pub fn new(transporter_type: TransporterType) -> Self {
        Self {
            transporter_type,
            jmax: transporter_type.typical_jmax(),
            km_drug: transporter_type.typical_km_um(),
            km_atp: 0.5,  // Typical Km for ATP
            hill_coefficient: 1.0,
            expression_level: 1.0,
        }
    }

    /// Calculate transport rate (pmol/min/cm²)
    pub fn transport_rate(&self, drug_um: f64, atp_mm: f64) -> f64 {
        let effective_jmax = self.jmax * self.expression_level;

        if self.transporter_type.is_efflux() {
            // ATP-dependent transport
            let drug_term = drug_um.powf(self.hill_coefficient)
                / (self.km_drug.powf(self.hill_coefficient)
                   + drug_um.powf(self.hill_coefficient));

            let atp_term = atp_mm / (self.km_atp + atp_mm);

            effective_jmax * drug_term * atp_term
        } else {
            // Facilitated or secondary active transport
            let saturation = drug_um.powf(self.hill_coefficient)
                / (self.km_drug.powf(self.hill_coefficient)
                   + drug_um.powf(self.hill_coefficient));

            effective_jmax * saturation
        }
    }

    /// ATP consumption rate (pmol/min/cm²)
    pub fn atp_consumption(&self, drug_um: f64, atp_mm: f64) -> f64 {
        let transport = self.transport_rate(drug_um, atp_mm);
        transport * self.transporter_type.atp_stoichiometry()
    }
}

/// Inhibitor of a transporter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransporterInhibitor {
    /// Inhibitor name
    pub name: String,
    /// Target transporter
    pub target: TransporterType,
    /// Inhibition constant Ki (µM)
    pub ki_um: f64,
    /// Is it a competitive inhibitor?
    pub competitive: bool,
    /// Current concentration (µM)
    pub concentration_um: f64,
}

impl TransporterInhibitor {
    pub fn new(name: &str, target: TransporterType, ki_um: f64) -> Self {
        Self {
            name: name.to_string(),
            target,
            ki_um,
            competitive: true,
            concentration_um: 0.0,
        }
    }

    /// Calculate inhibition factor (0-1, where 0 = complete inhibition)
    pub fn inhibition_factor(&self) -> f64 {
        1.0 / (1.0 + self.concentration_um / self.ki_um)
    }
}

/// Drug substrate properties for transporters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugTransporterProfile {
    /// Drug name
    pub name: String,
    /// Substrates: transporter -> relative affinity (1.0 = reference)
    pub substrate_of: HashMap<TransporterType, f64>,
    /// Inhibitors: transporter -> Ki (µM)
    pub inhibitor_of: HashMap<TransporterType, f64>,
}

impl DrugTransporterProfile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            substrate_of: HashMap::new(),
            inhibitor_of: HashMap::new(),
        }
    }

    /// Is this drug a substrate of the given transporter?
    pub fn is_substrate_of(&self, transporter: TransporterType) -> bool {
        self.substrate_of.contains_key(&transporter)
    }

    /// Is this drug an inhibitor of the given transporter?
    pub fn is_inhibitor_of(&self, transporter: TransporterType) -> bool {
        self.inhibitor_of.contains_key(&transporter)
    }

    /// Get relative affinity for a transporter
    pub fn affinity_for(&self, transporter: TransporterType) -> f64 {
        self.substrate_of.get(&transporter).copied().unwrap_or(0.0)
    }
}

/// Database of drug-transporter interactions
#[derive(Debug, Clone, Default)]
pub struct TransporterDatabase {
    pub profiles: HashMap<String, DrugTransporterProfile>,
}

impl TransporterDatabase {
    pub fn new() -> Self {
        let mut db = Self::default();

        // P-gp substrates (effluxed from brain)
        let mut loperamide = DrugTransporterProfile::new("loperamide");
        loperamide.substrate_of.insert(TransporterType::Pgp, 1.0);
        db.profiles.insert("loperamide".to_string(), loperamide);

        let mut digoxin = DrugTransporterProfile::new("digoxin");
        digoxin.substrate_of.insert(TransporterType::Pgp, 1.2);
        db.profiles.insert("digoxin".to_string(), digoxin);

        let mut fexofenadine = DrugTransporterProfile::new("fexofenadine");
        fexofenadine.substrate_of.insert(TransporterType::Pgp, 0.8);
        db.profiles.insert("fexofenadine".to_string(), fexofenadine);

        // P-gp inhibitors (increase brain penetration)
        let mut ketoconazole = DrugTransporterProfile::new("ketoconazole");
        ketoconazole.inhibitor_of.insert(TransporterType::Pgp, 0.5);
        db.profiles.insert("ketoconazole".to_string(), ketoconazole);

        let mut verapamil = DrugTransporterProfile::new("verapamil");
        verapamil.inhibitor_of.insert(TransporterType::Pgp, 1.5);
        verapamil.substrate_of.insert(TransporterType::Pgp, 0.5);
        db.profiles.insert("verapamil".to_string(), verapamil);

        let mut cyclosporine = DrugTransporterProfile::new("cyclosporine");
        cyclosporine.inhibitor_of.insert(TransporterType::Pgp, 0.2);
        db.profiles.insert("cyclosporine".to_string(), cyclosporine);

        let mut quinidine = DrugTransporterProfile::new("quinidine");
        quinidine.inhibitor_of.insert(TransporterType::Pgp, 0.8);
        db.profiles.insert("quinidine".to_string(), quinidine);

        // BCRP substrates
        let mut rosuvastatin = DrugTransporterProfile::new("rosuvastatin");
        rosuvastatin.substrate_of.insert(TransporterType::Bcrp, 1.0);
        rosuvastatin.substrate_of.insert(TransporterType::Oat3, 0.5);
        db.profiles.insert("rosuvastatin".to_string(), rosuvastatin);

        // Neurotransmitter transporter substrates
        let mut serotonin = DrugTransporterProfile::new("serotonin");
        serotonin.substrate_of.insert(TransporterType::Sert, 1.0);
        db.profiles.insert("serotonin".to_string(), serotonin);

        let mut dopamine = DrugTransporterProfile::new("dopamine");
        dopamine.substrate_of.insert(TransporterType::Dat, 1.0);
        db.profiles.insert("dopamine".to_string(), dopamine);

        let mut norepinephrine = DrugTransporterProfile::new("norepinephrine");
        norepinephrine.substrate_of.insert(TransporterType::Net, 1.0);
        db.profiles.insert("norepinephrine".to_string(), norepinephrine);

        let mut gaba = DrugTransporterProfile::new("gaba");
        gaba.substrate_of.insert(TransporterType::Gat1, 1.0);
        gaba.substrate_of.insert(TransporterType::Gat3, 0.7);
        db.profiles.insert("gaba".to_string(), gaba);

        // SERT inhibitors (SSRIs)
        let mut fluoxetine = DrugTransporterProfile::new("fluoxetine");
        fluoxetine.inhibitor_of.insert(TransporterType::Sert, 0.001);  // Very potent
        db.profiles.insert("fluoxetine".to_string(), fluoxetine);

        let mut sertraline = DrugTransporterProfile::new("sertraline");
        sertraline.inhibitor_of.insert(TransporterType::Sert, 0.0003);
        sertraline.inhibitor_of.insert(TransporterType::Dat, 0.025);
        db.profiles.insert("sertraline".to_string(), sertraline);

        let mut paroxetine = DrugTransporterProfile::new("paroxetine");
        paroxetine.inhibitor_of.insert(TransporterType::Sert, 0.0001);  // Most potent SSRI
        db.profiles.insert("paroxetine".to_string(), paroxetine);

        // DAT inhibitors
        let mut methylphenidate = DrugTransporterProfile::new("methylphenidate");
        methylphenidate.inhibitor_of.insert(TransporterType::Dat, 0.02);
        methylphenidate.inhibitor_of.insert(TransporterType::Net, 0.1);
        db.profiles.insert("methylphenidate".to_string(), methylphenidate);

        let mut cocaine = DrugTransporterProfile::new("cocaine");
        cocaine.inhibitor_of.insert(TransporterType::Dat, 0.2);
        cocaine.inhibitor_of.insert(TransporterType::Net, 0.3);
        cocaine.inhibitor_of.insert(TransporterType::Sert, 0.4);
        db.profiles.insert("cocaine".to_string(), cocaine);

        // GAT inhibitors
        let mut tiagabine = DrugTransporterProfile::new("tiagabine");
        tiagabine.inhibitor_of.insert(TransporterType::Gat1, 0.07);
        db.profiles.insert("tiagabine".to_string(), tiagabine);

        db
    }

    pub fn get(&self, drug: &str) -> Option<&DrugTransporterProfile> {
        self.profiles.get(drug)
    }
}

/// Blood-Brain Barrier transport model
#[derive(Debug, Clone)]
pub struct BbbTransport {
    /// Transporters expressed at BBB
    pub transporters: HashMap<TransporterType, TransporterKinetics>,
    /// Current inhibitors
    pub inhibitors: Vec<TransporterInhibitor>,
    /// Surface area (cm²) per gram brain
    pub surface_area_per_g: f64,
    /// Brain weight (g)
    pub brain_weight_g: f64,
    /// ATP concentration (mM)
    pub atp_mm: f64,
}

impl Default for BbbTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl BbbTransport {
    pub fn new() -> Self {
        let mut transporters = HashMap::new();

        // Major BBB transporters
        transporters.insert(TransporterType::Pgp, TransporterKinetics::new(TransporterType::Pgp));
        transporters.insert(TransporterType::Bcrp, TransporterKinetics::new(TransporterType::Bcrp));
        transporters.insert(TransporterType::Mrp1, TransporterKinetics::new(TransporterType::Mrp1));
        transporters.insert(TransporterType::Lat1, TransporterKinetics::new(TransporterType::Lat1));
        transporters.insert(TransporterType::Glut1, TransporterKinetics::new(TransporterType::Glut1));
        transporters.insert(TransporterType::Oat3, TransporterKinetics::new(TransporterType::Oat3));
        transporters.insert(TransporterType::Oct1, TransporterKinetics::new(TransporterType::Oct1));

        Self {
            transporters,
            inhibitors: Vec::new(),
            surface_area_per_g: 100.0,  // ~100 cm²/g brain (capillary surface)
            brain_weight_g: 1400.0,
            atp_mm: 3.0,  // Normal intracellular ATP
        }
    }

    /// Calculate net flux across BBB (positive = blood to brain)
    pub fn net_flux(&self, drug: &str, blood_um: f64, brain_um: f64, drug_profile: Option<&DrugTransporterProfile>) -> NetFlux {
        let total_surface = self.surface_area_per_g * self.brain_weight_g;

        // Passive diffusion (bidirectional)
        let passive_permeability = 1.0;  // µm/s - needs drug-specific value
        let passive_flux = passive_permeability * (blood_um - brain_um) * total_surface * 60.0 / 1e8;
        // Convert to pmol/min

        // Active efflux (brain to blood)
        let mut efflux = 0.0;
        if let Some(profile) = drug_profile {
            for (transporter_type, kinetics) in &self.transporters {
                if transporter_type.is_efflux() && profile.is_substrate_of(*transporter_type) {
                    let affinity = profile.affinity_for(*transporter_type);
                    let adjusted_km = kinetics.km_drug / affinity;

                    // Apply inhibition
                    let inhibition_factor = self.get_inhibition_factor(*transporter_type);

                    let rate = kinetics.jmax * kinetics.expression_level
                        * brain_um / (adjusted_km + brain_um)
                        * self.atp_mm / (kinetics.km_atp + self.atp_mm)
                        * inhibition_factor;

                    efflux += rate * total_surface;
                }
            }
        }

        // Active influx (blood to brain)
        let mut influx = 0.0;
        if let Some(profile) = drug_profile {
            for (transporter_type, kinetics) in &self.transporters {
                if transporter_type.is_influx() && profile.is_substrate_of(*transporter_type) {
                    let affinity = profile.affinity_for(*transporter_type);
                    let adjusted_km = kinetics.km_drug / affinity;

                    let rate = kinetics.jmax * kinetics.expression_level
                        * blood_um / (adjusted_km + blood_um);

                    influx += rate * total_surface;
                }
            }
        }

        NetFlux {
            passive_pmol_min: passive_flux,
            active_influx_pmol_min: influx,
            active_efflux_pmol_min: efflux,
            net_pmol_min: passive_flux + influx - efflux,
        }
    }

    /// Get combined inhibition factor for a transporter
    fn get_inhibition_factor(&self, transporter: TransporterType) -> f64 {
        let mut factor = 1.0;
        for inhibitor in &self.inhibitors {
            if inhibitor.target == transporter {
                factor *= inhibitor.inhibition_factor();
            }
        }
        factor
    }

    /// Add an inhibitor
    pub fn add_inhibitor(&mut self, inhibitor: TransporterInhibitor) {
        self.inhibitors.push(inhibitor);
    }

    /// Set inhibitor concentration
    pub fn set_inhibitor_concentration(&mut self, name: &str, concentration_um: f64) {
        for inhibitor in &mut self.inhibitors {
            if inhibitor.name == name {
                inhibitor.concentration_um = concentration_um;
            }
        }
    }

    /// Calculate brain:blood ratio at steady state
    pub fn predict_brain_blood_ratio(&self, drug_profile: Option<&DrugTransporterProfile>) -> f64 {
        // At steady state, net flux = 0
        // Passive flux = Active efflux - Active influx

        let test_blood = 1.0;  // 1 µM reference

        // Iterate to find brain concentration where net flux = 0
        let mut brain = test_blood;
        for _ in 0..100 {
            let flux = self.net_flux("test", test_blood, brain, drug_profile);
            let adjustment = flux.net_pmol_min / 1000.0;  // Damped update
            brain -= adjustment;
            brain = brain.max(0.001);

            if flux.net_pmol_min.abs() < 0.01 {
                break;
            }
        }

        brain / test_blood
    }
}

/// Net flux result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetFlux {
    /// Passive diffusion flux (pmol/min)
    pub passive_pmol_min: f64,
    /// Active influx (pmol/min)
    pub active_influx_pmol_min: f64,
    /// Active efflux (pmol/min)
    pub active_efflux_pmol_min: f64,
    /// Net flux (pmol/min, positive = into brain)
    pub net_pmol_min: f64,
}

/// Neurotransmitter reuptake system
#[derive(Debug, Clone)]
pub struct ReuptakeSystem {
    /// Transporters for each neurotransmitter
    pub transporters: HashMap<String, TransporterKinetics>,
    /// Current drug inhibition levels
    pub inhibition: HashMap<TransporterType, f64>,
}

impl ReuptakeSystem {
    pub fn new() -> Self {
        let mut transporters = HashMap::new();

        transporters.insert(
            "serotonin".to_string(),
            TransporterKinetics::new(TransporterType::Sert)
        );
        transporters.insert(
            "dopamine".to_string(),
            TransporterKinetics::new(TransporterType::Dat)
        );
        transporters.insert(
            "norepinephrine".to_string(),
            TransporterKinetics::new(TransporterType::Net)
        );
        transporters.insert(
            "gaba".to_string(),
            TransporterKinetics::new(TransporterType::Gat1)
        );
        transporters.insert(
            "glutamate".to_string(),
            TransporterKinetics::new(TransporterType::Eaat)
        );

        Self {
            transporters,
            inhibition: HashMap::new(),
        }
    }

    /// Apply drug inhibition
    pub fn apply_drug(&mut self, drug_profile: &DrugTransporterProfile, concentration_um: f64) {
        for (transporter_type, ki) in &drug_profile.inhibitor_of {
            let inhibition = concentration_um / (concentration_um + ki);
            self.inhibition.insert(*transporter_type, inhibition);
        }
    }

    /// Calculate reuptake rate for a neurotransmitter
    pub fn reuptake_rate(&self, neurotransmitter: &str, concentration_um: f64) -> f64 {
        if let Some(transporter) = self.transporters.get(neurotransmitter) {
            let base_rate = transporter.transport_rate(concentration_um, 3.0);

            // Apply inhibition
            let inhibition = self.inhibition
                .get(&transporter.transporter_type)
                .copied()
                .unwrap_or(0.0);

            base_rate * (1.0 - inhibition)
        } else {
            0.0
        }
    }

    /// Calculate synaptic cleft clearance time constant (ms)
    pub fn clearance_tau_ms(&self, neurotransmitter: &str) -> f64 {
        // Approximation: τ ≈ 1 / (k_reuptake + k_diffusion)
        // Base values
        let base_tau = match neurotransmitter {
            "serotonin" => 50.0,
            "dopamine" => 100.0,
            "norepinephrine" => 80.0,
            "gaba" => 20.0,
            "glutamate" => 5.0,
            _ => 50.0,
        };

        // Inhibition prolongs clearance
        if let Some(transporter) = self.transporters.get(neurotransmitter) {
            let inhibition = self.inhibition
                .get(&transporter.transporter_type)
                .copied()
                .unwrap_or(0.0);

            base_tau / (1.0 - inhibition + 0.01)  // Prevent division by zero
        } else {
            base_tau
        }
    }
}

impl Default for ReuptakeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgp_efflux() {
        let kinetics = TransporterKinetics::new(TransporterType::Pgp);

        // Should show saturable kinetics
        let rate_low = kinetics.transport_rate(0.1, 3.0);
        let rate_high = kinetics.transport_rate(10.0, 3.0);

        assert!(rate_high > rate_low);
        assert!(rate_high < rate_low * 20.0);  // Not linear
    }

    #[test]
    fn test_inhibition() {
        let mut bbb = BbbTransport::new();

        let quinidine = TransporterInhibitor::new("quinidine", TransporterType::Pgp, 0.8);
        bbb.add_inhibitor(quinidine);
        bbb.set_inhibitor_concentration("quinidine", 5.0);

        let factor = bbb.get_inhibition_factor(TransporterType::Pgp);
        assert!(factor < 0.2);  // Strong inhibition at 5x Ki
    }

    #[test]
    fn test_reuptake_inhibition() {
        let mut reuptake = ReuptakeSystem::new();

        let base_clearance = reuptake.clearance_tau_ms("serotonin");

        // Apply fluoxetine
        let db = TransporterDatabase::new();
        let fluoxetine = db.get("fluoxetine").unwrap();
        reuptake.apply_drug(fluoxetine, 0.1);

        let inhibited_clearance = reuptake.clearance_tau_ms("serotonin");
        assert!(inhibited_clearance > base_clearance * 2.0);
    }
}
