//! Clinical Literature Reference Values
//! =====================================
//!
//! PET imaging and pharmacokinetic data from peer-reviewed literature
//! for validation of pharmacological models.
//!
//! # Data Sources
//! - PET occupancy: [11C]flumazenil (GABA-A), [11C]raclopride (D2), [11C]DASB (SERT)
//! - Pharmacokinetics: FDA labels, clinical pharmacology reviews
//! - Brain concentrations: Microdialysis and CSF studies
//!
//! # References (PMID)
//! - Lingford-Hughes A et al. (2002) Neuropsychopharmacology 27:867-876
//! - Farde L et al. (1992) Arch Gen Psychiatry 49:538-544
//! - Meyer JH et al. (2004) Am J Psychiatry 161:826-835
//! - Melichar JK et al. (2005) Neuropsychopharmacology 30:516-524

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// PET occupancy data from clinical studies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetOccupancyData {
    /// Drug name
    pub drug: String,
    /// Receptor target (e.g., "GABA-A", "D2", "SERT", "OPRM1")
    pub receptor: String,
    /// Dose (mg)
    pub dose_mg: f64,
    /// Route of administration
    pub route: String,
    /// Time post-dose for measurement (hours)
    pub time_h: f64,
    /// Occupancy percentage (0-100)
    pub occupancy_percent: f64,
    /// Standard deviation if available
    pub sd: Option<f64>,
    /// PubMed ID
    pub pmid: Option<u32>,
    /// Brain region measured
    pub region: String,
}

/// Clinical pharmacokinetic reference values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalPkReference {
    /// Drug name
    pub drug: String,
    /// Dose (mg)
    pub dose_mg: f64,
    /// Route
    pub route: String,
    /// Peak plasma concentration (ng/mL)
    pub cmax_plasma_ng_ml: f64,
    /// Time to peak (hours)
    pub tmax_h: f64,
    /// Half-life (hours)
    pub t_half_h: f64,
    /// Brain/plasma ratio (if known)
    pub brain_plasma_ratio: Option<f64>,
    /// CSF concentration (ng/mL) if known
    pub csf_ng_ml: Option<f64>,
    /// Source (FDA label, clinical study)
    pub source: String,
}

/// Validation result comparing model to literature
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub drug: String,
    pub metric: String,
    pub model_value: f64,
    pub literature_value: f64,
    pub percent_error: f64,
    pub within_tolerance: bool,
}

impl ValidationResult {
    pub fn new(drug: &str, metric: &str, model: f64, literature: f64, tolerance: f64) -> Self {
        let error = if literature != 0.0 {
            ((model - literature) / literature).abs() * 100.0
        } else {
            if model == 0.0 { 0.0 } else { 100.0 }
        };

        Self {
            drug: drug.to_string(),
            metric: metric.to_string(),
            model_value: model,
            literature_value: literature,
            percent_error: error,
            within_tolerance: error <= tolerance * 100.0,
        }
    }
}

/// Database of clinical literature values
pub struct ClinicalLiteratureDb {
    pet_data: HashMap<String, Vec<PetOccupancyData>>,
    pk_data: HashMap<String, Vec<ClinicalPkReference>>,
}

impl Default for ClinicalLiteratureDb {
    fn default() -> Self {
        Self::new()
    }
}

impl ClinicalLiteratureDb {
    pub fn new() -> Self {
        let mut db = Self {
            pet_data: HashMap::new(),
            pk_data: HashMap::new(),
        };
        db.init_pet_data();
        db.init_pk_data();
        db
    }

    fn init_pet_data(&mut self) {
        // ============================================================
        // BENZODIAZEPINES - [11C]Flumazenil PET
        // ============================================================
        // Lingford-Hughes A et al. (2002) PMID: 12499952

        self.add_pet(PetOccupancyData {
            drug: "diazepam".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 10.0,
            route: "oral".to_string(),
            time_h: 1.5,
            occupancy_percent: 15.0, // Low occupancy at therapeutic doses
            sd: Some(5.0),
            pmid: Some(12499952),
            region: "cortex".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "diazepam".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 20.0,
            route: "oral".to_string(),
            time_h: 1.5,
            occupancy_percent: 28.0,
            sd: Some(8.0),
            pmid: Some(12499952),
            region: "cortex".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "lorazepam".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 2.0,
            route: "oral".to_string(),
            time_h: 2.0,
            occupancy_percent: 20.0,
            sd: Some(6.0),
            pmid: Some(12499952),
            region: "cortex".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "midazolam".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 7.5,
            route: "IV".to_string(),
            time_h: 0.25,
            occupancy_percent: 35.0,
            sd: Some(10.0),
            pmid: Some(12499952),
            region: "cortex".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "alprazolam".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 1.0,
            route: "oral".to_string(),
            time_h: 1.5,
            occupancy_percent: 22.0,
            sd: Some(7.0),
            pmid: Some(12499952),
            region: "cortex".to_string(),
        });

        // ============================================================
        // ANTIPSYCHOTICS - [11C]Raclopride PET (D2)
        // ============================================================
        // Farde L et al. (1992) PMID: 1616206
        // Kapur S et al. (2000) PMID: 10686270

        self.add_pet(PetOccupancyData {
            drug: "haloperidol".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 5.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 70.0, // Therapeutic range 65-80%
            sd: Some(8.0),
            pmid: Some(1616206),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "haloperidol".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 10.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 80.0,
            sd: Some(5.0),
            pmid: Some(1616206),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "risperidone".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 2.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 66.0,
            sd: Some(10.0),
            pmid: Some(10686270),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "risperidone".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 4.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 75.0,
            sd: Some(8.0),
            pmid: Some(10686270),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "olanzapine".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 10.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 60.0,
            sd: Some(12.0),
            pmid: Some(10686270),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "clozapine".to_string(),
            receptor: "D2".to_string(),
            dose_mg: 400.0,
            route: "oral".to_string(),
            time_h: 4.0,
            occupancy_percent: 52.0, // Lower D2 occupancy is characteristic
            sd: Some(15.0),
            pmid: Some(10686270),
            region: "striatum".to_string(),
        });

        // ============================================================
        // SSRIs - [11C]DASB PET (SERT)
        // ============================================================
        // Meyer JH et al. (2004) PMID: 15121618

        self.add_pet(PetOccupancyData {
            drug: "fluoxetine".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 20.0,
            route: "oral".to_string(),
            time_h: 336.0, // Steady state (2 weeks)
            occupancy_percent: 80.0,
            sd: Some(5.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "paroxetine".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 20.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 83.0,
            sd: Some(6.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "sertraline".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 50.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 77.0,
            sd: Some(8.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "citalopram".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 20.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 72.0,
            sd: Some(7.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "escitalopram".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 10.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 80.0,
            sd: Some(5.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "venlafaxine".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 75.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 45.0, // Lower at low doses
            sd: Some(12.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "venlafaxine".to_string(),
            receptor: "SERT".to_string(),
            dose_mg: 150.0,
            route: "oral".to_string(),
            time_h: 336.0,
            occupancy_percent: 70.0,
            sd: Some(10.0),
            pmid: Some(15121618),
            region: "striatum".to_string(),
        });

        // ============================================================
        // OPIOIDS - [11C]Carfentanil PET (mu-opioid)
        // ============================================================
        // Melichar JK et al. (2005) PMID: 15483561

        self.add_pet(PetOccupancyData {
            drug: "morphine".to_string(),
            receptor: "OPRM1".to_string(),
            dose_mg: 10.0,
            route: "IV".to_string(),
            time_h: 0.5,
            occupancy_percent: 42.0,
            sd: Some(12.0),
            pmid: Some(15483561),
            region: "thalamus".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "fentanyl".to_string(),
            receptor: "OPRM1".to_string(),
            dose_mg: 0.1, // 100 mcg
            route: "IV".to_string(),
            time_h: 0.25,
            occupancy_percent: 35.0,
            sd: Some(10.0),
            pmid: Some(15483561),
            region: "thalamus".to_string(),
        });

        self.add_pet(PetOccupancyData {
            drug: "buprenorphine".to_string(),
            receptor: "OPRM1".to_string(),
            dose_mg: 2.0,
            route: "sublingual".to_string(),
            time_h: 2.0,
            occupancy_percent: 75.0, // High occupancy even at low doses
            sd: Some(8.0),
            pmid: Some(15483561),
            region: "thalamus".to_string(),
        });

        // ============================================================
        // ANESTHETICS - Propofol effect-site concentration
        // ============================================================
        // Schuttler J & Ihmsen H (2000) Anesthesiology

        self.add_pet(PetOccupancyData {
            drug: "propofol".to_string(),
            receptor: "GABA-A".to_string(),
            dose_mg: 140.0, // 2 mg/kg for 70 kg
            route: "IV".to_string(),
            time_h: 0.05, // 3 min
            occupancy_percent: 50.0, // At LOC
            sd: Some(15.0),
            pmid: Some(10754634),
            region: "cortex".to_string(),
        });

        // ============================================================
        // NMDA ANTAGONISTS - Ketamine
        // ============================================================

        self.add_pet(PetOccupancyData {
            drug: "ketamine".to_string(),
            receptor: "NMDA".to_string(),
            dose_mg: 35.0, // 0.5 mg/kg for 70 kg
            route: "IV".to_string(),
            time_h: 0.25,
            occupancy_percent: 30.0, // Subanesthetic
            sd: Some(10.0),
            pmid: Some(11283682),
            region: "cortex".to_string(),
        });
    }

    fn init_pk_data(&mut self) {
        // ============================================================
        // BENZODIAZEPINES
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "diazepam".to_string(),
            dose_mg: 10.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 300.0,
            tmax_h: 1.0,
            t_half_h: 43.0,
            brain_plasma_ratio: Some(0.9),
            csf_ng_ml: Some(15.0),
            source: "FDA Label / Greenblatt DJ (1980)".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "midazolam".to_string(),
            dose_mg: 7.5,
            route: "IV".to_string(),
            cmax_plasma_ng_ml: 100.0, // Immediate
            tmax_h: 0.0,
            t_half_h: 2.5,
            brain_plasma_ratio: Some(0.8),
            csf_ng_ml: Some(5.0),
            source: "FDA Label".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "lorazepam".to_string(),
            dose_mg: 2.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 25.0,
            tmax_h: 2.0,
            t_half_h: 12.0,
            brain_plasma_ratio: Some(0.8),
            csf_ng_ml: Some(4.0),
            source: "FDA Label".to_string(),
        });

        // ============================================================
        // ANESTHETICS
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "propofol".to_string(),
            dose_mg: 140.0, // 2 mg/kg
            route: "IV".to_string(),
            cmax_plasma_ng_ml: 4000.0, // 4 µg/mL at induction
            tmax_h: 0.0,
            t_half_h: 0.5, // Distribution half-life
            brain_plasma_ratio: Some(1.2), // Rapid equilibration
            csf_ng_ml: Some(800.0), // Effect-site ~1 µg/mL
            source: "Schuttler J & Ihmsen H (2000)".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "ketamine".to_string(),
            dose_mg: 35.0, // 0.5 mg/kg
            route: "IV".to_string(),
            cmax_plasma_ng_ml: 500.0,
            tmax_h: 0.0,
            t_half_h: 2.5,
            brain_plasma_ratio: Some(4.0), // High brain penetration
            csf_ng_ml: Some(250.0),
            source: "Clements JA (1982)".to_string(),
        });

        // ============================================================
        // OPIOIDS
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "morphine".to_string(),
            dose_mg: 10.0,
            route: "IV".to_string(),
            cmax_plasma_ng_ml: 100.0,
            tmax_h: 0.0,
            t_half_h: 3.0,
            brain_plasma_ratio: Some(0.3), // Poor BBB penetration
            csf_ng_ml: Some(10.0),
            source: "FDA Label".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "fentanyl".to_string(),
            dose_mg: 0.1, // 100 mcg
            route: "IV".to_string(),
            cmax_plasma_ng_ml: 1.5, // 1.5 ng/mL
            tmax_h: 0.0,
            t_half_h: 4.0,
            brain_plasma_ratio: Some(2.5), // High lipophilicity
            csf_ng_ml: Some(0.5),
            source: "FDA Label".to_string(),
        });

        // ============================================================
        // ANTIPSYCHOTICS
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "haloperidol".to_string(),
            dose_mg: 5.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 5.0,
            tmax_h: 4.0,
            t_half_h: 18.0,
            brain_plasma_ratio: Some(20.0), // Very high brain accumulation
            csf_ng_ml: Some(1.5),
            source: "Farde L (1992)".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "risperidone".to_string(),
            dose_mg: 2.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 10.0,
            tmax_h: 1.0,
            t_half_h: 3.0, // Parent compound
            brain_plasma_ratio: Some(3.0),
            csf_ng_ml: Some(2.0),
            source: "FDA Label".to_string(),
        });

        // ============================================================
        // SSRIs
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "fluoxetine".to_string(),
            dose_mg: 20.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 20.0,
            tmax_h: 6.0,
            t_half_h: 72.0, // Long half-life
            brain_plasma_ratio: Some(3.0),
            csf_ng_ml: Some(10.0),
            source: "FDA Label".to_string(),
        });

        self.add_pk(ClinicalPkReference {
            drug: "sertraline".to_string(),
            dose_mg: 50.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 40.0,
            tmax_h: 6.0,
            t_half_h: 26.0,
            brain_plasma_ratio: Some(5.0),
            csf_ng_ml: Some(8.0),
            source: "FDA Label".to_string(),
        });

        // ============================================================
        // ANTI-PARKINSON
        // ============================================================

        self.add_pk(ClinicalPkReference {
            drug: "levodopa".to_string(),
            dose_mg: 100.0,
            route: "oral".to_string(),
            cmax_plasma_ng_ml: 1500.0, // With carbidopa
            tmax_h: 1.0,
            t_half_h: 1.5,
            brain_plasma_ratio: Some(0.1), // Requires active transport
            csf_ng_ml: Some(50.0),
            source: "Nutt JG (2008)".to_string(),
        });
    }

    fn add_pet(&mut self, data: PetOccupancyData) {
        let key = data.drug.to_lowercase();
        self.pet_data.entry(key).or_insert_with(Vec::new).push(data);
    }

    fn add_pk(&mut self, data: ClinicalPkReference) {
        let key = data.drug.to_lowercase();
        self.pk_data.entry(key).or_insert_with(Vec::new).push(data);
    }

    /// Get PET occupancy data for a drug
    pub fn get_pet_data(&self, drug: &str) -> Option<&Vec<PetOccupancyData>> {
        self.pet_data.get(&drug.to_lowercase())
    }

    /// Get PK reference data for a drug
    pub fn get_pk_data(&self, drug: &str) -> Option<&Vec<ClinicalPkReference>> {
        self.pk_data.get(&drug.to_lowercase())
    }

    /// Find matching PET study for validation
    pub fn find_matching_pet(&self, drug: &str, receptor: &str, dose_mg: f64, route: &str) -> Option<&PetOccupancyData> {
        self.pet_data.get(&drug.to_lowercase())?.iter()
            .find(|d| {
                d.receptor.to_lowercase() == receptor.to_lowercase() &&
                d.route.to_lowercase() == route.to_lowercase() &&
                (d.dose_mg - dose_mg).abs() / dose_mg < 0.3 // Within 30% of dose
            })
    }

    /// Validate model occupancy against literature
    pub fn validate_occupancy(
        &self,
        drug: &str,
        receptor: &str,
        dose_mg: f64,
        route: &str,
        model_occupancy_percent: f64,
        tolerance: f64,
    ) -> Option<ValidationResult> {
        let lit = self.find_matching_pet(drug, receptor, dose_mg, route)?;
        Some(ValidationResult::new(
            drug,
            &format!("{} occupancy", receptor),
            model_occupancy_percent,
            lit.occupancy_percent,
            tolerance,
        ))
    }

    /// Get all drugs with PET data
    pub fn drugs_with_pet_data(&self) -> Vec<String> {
        self.pet_data.keys().cloned().collect()
    }

    /// Get all drugs with PK data
    pub fn drugs_with_pk_data(&self) -> Vec<String> {
        self.pk_data.keys().cloned().collect()
    }

    /// Calculate expected occupancy using Hill equation
    /// Occupancy = 100 * C^n / (EC50^n + C^n)
    pub fn calculate_expected_occupancy(
        concentration_um: f64,
        ec50_um: f64,
        hill_coefficient: f64,
    ) -> f64 {
        let c_n = concentration_um.powf(hill_coefficient);
        let ec50_n = ec50_um.powf(hill_coefficient);
        100.0 * c_n / (ec50_n + c_n)
    }
}

/// Calculate receptor occupancy from free brain concentration
/// Uses standard pharmacological occupancy equation
///
/// # Arguments
/// * `free_brain_um` - Free drug concentration in brain (µM)
/// * `ki_nm` - Binding affinity Ki (nM)
///
/// # Returns
/// Occupancy percentage (0-100)
pub fn calculate_occupancy_from_ki(free_brain_um: f64, ki_nm: f64) -> f64 {
    // Convert Ki from nM to µM
    let ki_um = ki_nm / 1000.0;
    // Occupancy = 100 * C / (Ki + C)
    100.0 * free_brain_um / (ki_um + free_brain_um)
}

/// Run comprehensive validation for a drug
pub fn validate_drug(
    drug: &str,
    model_cmax_ng_ml: f64,
    model_brain_um: f64,
    model_tmax_h: f64,
    tolerance: f64,
) -> Vec<ValidationResult> {
    let db = ClinicalLiteratureDb::new();
    let mut results = Vec::new();

    if let Some(pk_list) = db.get_pk_data(drug) {
        for pk in pk_list {
            // Validate Cmax
            results.push(ValidationResult::new(
                drug,
                "Cmax plasma (ng/mL)",
                model_cmax_ng_ml,
                pk.cmax_plasma_ng_ml,
                tolerance,
            ));

            // Validate Tmax
            if pk.tmax_h > 0.0 && model_tmax_h > 0.0 {
                results.push(ValidationResult::new(
                    drug,
                    "Tmax (h)",
                    model_tmax_h,
                    pk.tmax_h,
                    tolerance,
                ));
            }

            // Validate brain concentration if CSF data available
            if let Some(csf) = pk.csf_ng_ml {
                // Convert brain µM to ng/mL for comparison
                // Approximate MW lookup would be needed
                let approx_mw = 300.0; // Placeholder
                let model_brain_ng_ml = model_brain_um * approx_mw / 1000.0;
                results.push(ValidationResult::new(
                    drug,
                    "Brain/CSF (ng/mL)",
                    model_brain_ng_ml,
                    csf,
                    tolerance,
                ));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literature_db_creation() {
        let db = ClinicalLiteratureDb::new();
        assert!(db.get_pet_data("diazepam").is_some());
        assert!(db.get_pk_data("propofol").is_some());
    }

    #[test]
    fn test_occupancy_calculation() {
        // At EC50, occupancy should be 50%
        let occ = ClinicalLiteratureDb::calculate_expected_occupancy(1.0, 1.0, 1.0);
        assert!((occ - 50.0).abs() < 0.1);

        // At 10x EC50, occupancy should be ~91%
        let occ_high = ClinicalLiteratureDb::calculate_expected_occupancy(10.0, 1.0, 1.0);
        assert!((occ_high - 90.9).abs() < 1.0);
    }

    #[test]
    fn test_occupancy_from_ki() {
        // Morphine: Ki ~0.3 nM for OPRM1
        // At 0.3 nM concentration, should be ~50% occupancy
        let occ = calculate_occupancy_from_ki(0.0003, 0.3);
        assert!((occ - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_validation_result() {
        // Model predicts 75%, literature says 70%, tolerance 5%
        let result = ValidationResult::new("haloperidol", "D2 occupancy", 75.0, 70.0, 0.05);
        assert!(!result.within_tolerance); // 7.1% error > 5%

        // Within tolerance
        let result2 = ValidationResult::new("haloperidol", "D2 occupancy", 72.0, 70.0, 0.05);
        assert!(result2.within_tolerance); // 2.9% error < 5%
    }

    #[test]
    fn test_find_matching_pet() {
        let db = ClinicalLiteratureDb::new();

        // Should find diazepam GABA-A data at 10mg oral
        let pet = db.find_matching_pet("diazepam", "GABA-A", 10.0, "oral");
        assert!(pet.is_some());
        let data = pet.unwrap();
        assert_eq!(data.occupancy_percent, 15.0);
    }

    #[test]
    fn test_antipsychotic_d2_occupancy() {
        let db = ClinicalLiteratureDb::new();

        // Haloperidol should have ~70% D2 occupancy at 5mg
        let pet = db.find_matching_pet("haloperidol", "D2", 5.0, "oral");
        assert!(pet.is_some());
        assert_eq!(pet.unwrap().occupancy_percent, 70.0);
    }
}
