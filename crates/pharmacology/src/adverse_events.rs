//! Bayesian Adverse Event Predictor
//! ==================================
//!
//! Integrates all pharmacology modules to predict adverse drug reactions:
//! - Drug-receptor binding effects
//! - Pharmacogenomic risk factors
//! - Drug-drug interactions
//! - Hepatotoxicity risk
//! - QT prolongation
//! - CNS effects
//!
//! # Bayesian Network Structure
//!
//! ```text
//!                    ┌─────────────┐
//!                    │ Patient     │
//!                    │ Factors     │
//!                    └──────┬──────┘
//!                           │
//!           ┌───────────────┼───────────────┐
//!           │               │               │
//!           ▼               ▼               ▼
//!    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
//!    │ Genetics    │ │ Comorbidity │ │ Concomitant │
//!    │ (CYP450)    │ │             │ │ Drugs       │
//!    └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
//!           │               │               │
//!           └───────────────┼───────────────┘
//!                           │
//!                           ▼
//!                    ┌─────────────┐
//!                    │ Exposure    │
//!                    │ Level       │
//!                    └──────┬──────┘
//!                           │
//!           ┌───────────────┼───────────────┐
//!           │               │               │
//!           ▼               ▼               ▼
//!    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
//!    │ Hepato-     │ │ CNS         │ │ Cardiac     │
//!    │ toxicity    │ │ Effects     │ │ Effects     │
//!    └─────────────┘ └─────────────┘ └─────────────┘
//! ```
//!
//! # MedDRA Classification
//!
//! Adverse events classified using Medical Dictionary for Regulatory
//! Activities (MedDRA) System Organ Classes (SOCs).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MedDRA System Organ Class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemOrganClass {
    /// Blood and lymphatic system disorders
    BloodLymphatic,
    /// Cardiac disorders
    Cardiac,
    /// Congenital, familial and genetic disorders
    Congenital,
    /// Ear and labyrinth disorders
    EarLabyrinth,
    /// Endocrine disorders
    Endocrine,
    /// Eye disorders
    Eye,
    /// Gastrointestinal disorders
    Gastrointestinal,
    /// General disorders
    GeneralDisorders,
    /// Hepatobiliary disorders
    Hepatobiliary,
    /// Immune system disorders
    Immune,
    /// Infections and infestations
    Infections,
    /// Investigations (lab abnormalities)
    Investigations,
    /// Metabolism and nutrition disorders
    Metabolism,
    /// Musculoskeletal and connective tissue disorders
    Musculoskeletal,
    /// Nervous system disorders
    NervousSystem,
    /// Psychiatric disorders
    Psychiatric,
    /// Renal and urinary disorders
    RenalUrinary,
    /// Reproductive system disorders
    Reproductive,
    /// Respiratory disorders
    Respiratory,
    /// Skin and subcutaneous tissue disorders
    Skin,
    /// Vascular disorders
    Vascular,
}

impl SystemOrganClass {
    pub fn description(&self) -> &'static str {
        match self {
            SystemOrganClass::BloodLymphatic => "Blood and lymphatic system disorders",
            SystemOrganClass::Cardiac => "Cardiac disorders",
            SystemOrganClass::Congenital => "Congenital, familial and genetic disorders",
            SystemOrganClass::EarLabyrinth => "Ear and labyrinth disorders",
            SystemOrganClass::Endocrine => "Endocrine disorders",
            SystemOrganClass::Eye => "Eye disorders",
            SystemOrganClass::Gastrointestinal => "Gastrointestinal disorders",
            SystemOrganClass::GeneralDisorders => "General disorders and administration site conditions",
            SystemOrganClass::Hepatobiliary => "Hepatobiliary disorders",
            SystemOrganClass::Immune => "Immune system disorders",
            SystemOrganClass::Infections => "Infections and infestations",
            SystemOrganClass::Investigations => "Investigations",
            SystemOrganClass::Metabolism => "Metabolism and nutrition disorders",
            SystemOrganClass::Musculoskeletal => "Musculoskeletal and connective tissue disorders",
            SystemOrganClass::NervousSystem => "Nervous system disorders",
            SystemOrganClass::Psychiatric => "Psychiatric disorders",
            SystemOrganClass::RenalUrinary => "Renal and urinary disorders",
            SystemOrganClass::Reproductive => "Reproductive system and breast disorders",
            SystemOrganClass::Respiratory => "Respiratory, thoracic and mediastinal disorders",
            SystemOrganClass::Skin => "Skin and subcutaneous tissue disorders",
            SystemOrganClass::Vascular => "Vascular disorders",
        }
    }
}

/// Severity of adverse event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Severity {
    /// Grade 1: Mild
    Mild,
    /// Grade 2: Moderate
    Moderate,
    /// Grade 3: Severe
    Severe,
    /// Grade 4: Life-threatening
    LifeThreatening,
    /// Grade 5: Death
    Fatal,
}

impl Severity {
    pub fn grade(&self) -> u8 {
        match self {
            Severity::Mild => 1,
            Severity::Moderate => 2,
            Severity::Severe => 3,
            Severity::LifeThreatening => 4,
            Severity::Fatal => 5,
        }
    }

    pub fn from_grade(grade: u8) -> Self {
        match grade {
            1 => Severity::Mild,
            2 => Severity::Moderate,
            3 => Severity::Severe,
            4 => Severity::LifeThreatening,
            _ => Severity::Fatal,
        }
    }
}

/// A specific adverse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventType {
    /// MedDRA preferred term
    pub preferred_term: String,
    /// System organ class
    pub soc: SystemOrganClass,
    /// Typical severity range
    pub typical_severity: Severity,
    /// Is this dose-dependent?
    pub dose_dependent: bool,
    /// Time to onset (hours)
    pub onset_hours: (f64, f64),  // (min, max)
}

/// Patient risk factors
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatientRiskFactors {
    /// Age in years
    pub age: f64,
    /// Creatinine clearance (mL/min)
    pub crcl: f64,
    /// Child-Pugh score (5-15)
    pub child_pugh: u8,
    /// Known allergies
    pub allergies: Vec<String>,
    /// Current medications
    pub medications: Vec<String>,
    /// Genetic risk factors
    pub genetic_risks: HashMap<String, f64>,
    /// Comorbidities
    pub comorbidities: Vec<String>,
}

impl PatientRiskFactors {
    pub fn new(age: f64, crcl: f64) -> Self {
        Self {
            age,
            crcl,
            child_pugh: 5,  // Normal
            ..Default::default()
        }
    }

    /// Is patient elderly (>65)?
    pub fn is_elderly(&self) -> bool {
        self.age >= 65.0
    }

    /// Has renal impairment?
    pub fn renal_impairment(&self) -> RenalFunction {
        if self.crcl >= 90.0 {
            RenalFunction::Normal
        } else if self.crcl >= 60.0 {
            RenalFunction::MildImpairment
        } else if self.crcl >= 30.0 {
            RenalFunction::ModerateImpairment
        } else if self.crcl >= 15.0 {
            RenalFunction::SevereImpairment
        } else {
            RenalFunction::EndStage
        }
    }

    /// Has hepatic impairment?
    pub fn hepatic_impairment(&self) -> HepaticFunction {
        match self.child_pugh {
            5..=6 => HepaticFunction::Normal,
            7..=9 => HepaticFunction::MildModerate,
            _ => HepaticFunction::Severe,
        }
    }
}

/// Renal function classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RenalFunction {
    Normal,
    MildImpairment,
    ModerateImpairment,
    SevereImpairment,
    EndStage,
}

/// Hepatic function classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HepaticFunction {
    Normal,
    MildModerate,
    Severe,
}

/// Conditional probability table for Bayesian network
#[derive(Debug, Clone, Default)]
pub struct ConditionalProbabilityTable {
    /// P(event | conditions)
    pub probabilities: HashMap<Vec<String>, f64>,
}

impl ConditionalProbabilityTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set probability for a condition
    pub fn set(&mut self, conditions: Vec<&str>, probability: f64) {
        let key: Vec<String> = conditions.iter().map(|s| s.to_string()).collect();
        self.probabilities.insert(key, probability);
    }

    /// Get probability for conditions
    pub fn get(&self, conditions: &[String]) -> f64 {
        self.probabilities.get(conditions).copied().unwrap_or(0.01)
    }
}

/// Bayesian network node
#[derive(Debug, Clone)]
pub struct BayesNode {
    /// Node name
    pub name: String,
    /// Parent nodes
    pub parents: Vec<String>,
    /// Conditional probability table
    pub cpt: ConditionalProbabilityTable,
    /// Current evidence/value
    pub evidence: Option<String>,
}

impl BayesNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parents: Vec::new(),
            cpt: ConditionalProbabilityTable::new(),
            evidence: None,
        }
    }

    pub fn with_parents(mut self, parents: Vec<&str>) -> Self {
        self.parents = parents.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Adverse event prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventPrediction {
    /// Event type
    pub event: String,
    /// Probability of occurrence (0-1)
    pub probability: f64,
    /// Expected severity if occurs
    pub expected_severity: Severity,
    /// Key risk factors contributing
    pub risk_factors: Vec<String>,
    /// Recommended actions
    pub recommendations: Vec<String>,
    /// Number Needed to Harm (NNH)
    pub nnh: Option<f64>,
}

/// Complete Bayesian adverse event predictor
#[derive(Debug, Clone)]
pub struct AdverseEventPredictor {
    /// Bayesian network nodes
    pub nodes: HashMap<String, BayesNode>,
    /// Drug-specific event profiles
    pub drug_events: HashMap<String, Vec<DrugEventProfile>>,
    /// Patient factors
    pub patient: PatientRiskFactors,
}

/// Drug-specific adverse event profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugEventProfile {
    pub drug: String,
    pub event: AdverseEventType,
    pub base_incidence: f64,
    pub dose_response: Option<DoseResponse>,
    pub risk_modifiers: Vec<RiskModifier>,
}

/// Dose-response relationship for adverse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseResponse {
    /// Reference dose (mg)
    pub reference_dose: f64,
    /// Exponent for dose scaling
    pub exponent: f64,
}

/// Factor that modifies risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskModifier {
    pub factor: String,
    pub relative_risk: f64,
}

impl Default for AdverseEventPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl AdverseEventPredictor {
    pub fn new() -> Self {
        let mut predictor = Self {
            nodes: HashMap::new(),
            drug_events: HashMap::new(),
            patient: PatientRiskFactors::default(),
        };

        predictor.build_network();
        predictor.add_drug_profiles();

        predictor
    }

    /// Build the Bayesian network structure
    fn build_network(&mut self) {
        // Root nodes (patient factors)
        self.nodes.insert("age".to_string(), BayesNode::new("age"));
        self.nodes.insert("genetics".to_string(), BayesNode::new("genetics"));
        self.nodes.insert("renal_function".to_string(), BayesNode::new("renal_function"));
        self.nodes.insert("hepatic_function".to_string(), BayesNode::new("hepatic_function"));
        self.nodes.insert("concomitant_drugs".to_string(), BayesNode::new("concomitant_drugs"));

        // Intermediate nodes
        let mut exposure = BayesNode::new("exposure")
            .with_parents(vec!["genetics", "renal_function", "hepatic_function", "concomitant_drugs"]);
        self.nodes.insert("exposure".to_string(), exposure);

        // Outcome nodes
        let hepatotoxicity = BayesNode::new("hepatotoxicity")
            .with_parents(vec!["exposure", "hepatic_function", "age"]);
        self.nodes.insert("hepatotoxicity".to_string(), hepatotoxicity);

        let qt_prolongation = BayesNode::new("qt_prolongation")
            .with_parents(vec!["exposure", "genetics", "concomitant_drugs"]);
        self.nodes.insert("qt_prolongation".to_string(), qt_prolongation);

        let cns_depression = BayesNode::new("cns_depression")
            .with_parents(vec!["exposure", "age", "concomitant_drugs"]);
        self.nodes.insert("cns_depression".to_string(), cns_depression);
    }

    /// Add drug-specific adverse event profiles
    fn add_drug_profiles(&mut self) {
        // Benzodiazepines
        let benzo_events = vec![
            DrugEventProfile {
                drug: "diazepam".to_string(),
                event: AdverseEventType {
                    preferred_term: "Somnolence".to_string(),
                    soc: SystemOrganClass::NervousSystem,
                    typical_severity: Severity::Mild,
                    dose_dependent: true,
                    onset_hours: (0.5, 2.0),
                },
                base_incidence: 0.15,
                dose_response: Some(DoseResponse {
                    reference_dose: 5.0,
                    exponent: 1.2,
                }),
                risk_modifiers: vec![
                    RiskModifier { factor: "elderly".to_string(), relative_risk: 2.0 },
                    RiskModifier { factor: "hepatic_impairment".to_string(), relative_risk: 1.5 },
                ],
            },
            DrugEventProfile {
                drug: "diazepam".to_string(),
                event: AdverseEventType {
                    preferred_term: "Anterograde amnesia".to_string(),
                    soc: SystemOrganClass::Psychiatric,
                    typical_severity: Severity::Moderate,
                    dose_dependent: true,
                    onset_hours: (0.5, 4.0),
                },
                base_incidence: 0.05,
                dose_response: Some(DoseResponse {
                    reference_dose: 10.0,
                    exponent: 1.5,
                }),
                risk_modifiers: vec![
                    RiskModifier { factor: "elderly".to_string(), relative_risk: 3.0 },
                ],
            },
            DrugEventProfile {
                drug: "diazepam".to_string(),
                event: AdverseEventType {
                    preferred_term: "Respiratory depression".to_string(),
                    soc: SystemOrganClass::Respiratory,
                    typical_severity: Severity::Severe,
                    dose_dependent: true,
                    onset_hours: (0.5, 2.0),
                },
                base_incidence: 0.001,
                dose_response: Some(DoseResponse {
                    reference_dose: 20.0,
                    exponent: 2.0,
                }),
                risk_modifiers: vec![
                    RiskModifier { factor: "opioid_concurrent".to_string(), relative_risk: 10.0 },
                    RiskModifier { factor: "copd".to_string(), relative_risk: 5.0 },
                    RiskModifier { factor: "elderly".to_string(), relative_risk: 2.5 },
                ],
            },
        ];
        self.drug_events.insert("diazepam".to_string(), benzo_events);

        // SSRIs
        let ssri_events = vec![
            DrugEventProfile {
                drug: "fluoxetine".to_string(),
                event: AdverseEventType {
                    preferred_term: "Nausea".to_string(),
                    soc: SystemOrganClass::Gastrointestinal,
                    typical_severity: Severity::Mild,
                    dose_dependent: true,
                    onset_hours: (1.0, 24.0),
                },
                base_incidence: 0.20,
                dose_response: Some(DoseResponse {
                    reference_dose: 20.0,
                    exponent: 0.8,
                }),
                risk_modifiers: vec![],
            },
            DrugEventProfile {
                drug: "fluoxetine".to_string(),
                event: AdverseEventType {
                    preferred_term: "Serotonin syndrome".to_string(),
                    soc: SystemOrganClass::NervousSystem,
                    typical_severity: Severity::LifeThreatening,
                    dose_dependent: false,
                    onset_hours: (2.0, 24.0),
                },
                base_incidence: 0.001,
                dose_response: None,
                risk_modifiers: vec![
                    RiskModifier { factor: "maoi_concurrent".to_string(), relative_risk: 100.0 },
                    RiskModifier { factor: "tramadol_concurrent".to_string(), relative_risk: 10.0 },
                    RiskModifier { factor: "triptans_concurrent".to_string(), relative_risk: 5.0 },
                ],
            },
            DrugEventProfile {
                drug: "fluoxetine".to_string(),
                event: AdverseEventType {
                    preferred_term: "QT prolongation".to_string(),
                    soc: SystemOrganClass::Cardiac,
                    typical_severity: Severity::Severe,
                    dose_dependent: true,
                    onset_hours: (24.0, 168.0),
                },
                base_incidence: 0.005,
                dose_response: Some(DoseResponse {
                    reference_dose: 40.0,
                    exponent: 1.5,
                }),
                risk_modifiers: vec![
                    RiskModifier { factor: "hypokalemia".to_string(), relative_risk: 5.0 },
                    RiskModifier { factor: "female".to_string(), relative_risk: 1.5 },
                    RiskModifier { factor: "long_qt_syndrome".to_string(), relative_risk: 20.0 },
                ],
            },
        ];
        self.drug_events.insert("fluoxetine".to_string(), ssri_events);

        // Acetaminophen
        let apap_events = vec![
            DrugEventProfile {
                drug: "acetaminophen".to_string(),
                event: AdverseEventType {
                    preferred_term: "Acute liver failure".to_string(),
                    soc: SystemOrganClass::Hepatobiliary,
                    typical_severity: Severity::LifeThreatening,
                    dose_dependent: true,
                    onset_hours: (24.0, 72.0),
                },
                base_incidence: 0.0001,
                dose_response: Some(DoseResponse {
                    reference_dose: 4000.0,
                    exponent: 3.0,  // Steep dose-response
                }),
                risk_modifiers: vec![
                    RiskModifier { factor: "chronic_alcohol".to_string(), relative_risk: 5.0 },
                    RiskModifier { factor: "fasting".to_string(), relative_risk: 2.0 },
                    RiskModifier { factor: "cyp2e1_induced".to_string(), relative_risk: 2.0 },
                ],
            },
        ];
        self.drug_events.insert("acetaminophen".to_string(), apap_events);
    }

    /// Set patient factors
    pub fn set_patient(&mut self, patient: PatientRiskFactors) {
        self.patient = patient;
    }

    /// Predict adverse events for a drug at given dose
    pub fn predict(&self, drug: &str, dose_mg: f64) -> Vec<AdverseEventPrediction> {
        let mut predictions = Vec::new();

        if let Some(events) = self.drug_events.get(drug) {
            for profile in events {
                let prediction = self.calculate_prediction(profile, dose_mg);
                if prediction.probability > 0.0001 {  // Only include if non-negligible
                    predictions.push(prediction);
                }
            }
        }

        // Sort by probability (highest first)
        predictions.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap());

        predictions
    }

    /// Calculate prediction for a single event profile
    fn calculate_prediction(&self, profile: &DrugEventProfile, dose_mg: f64) -> AdverseEventPrediction {
        let mut probability = profile.base_incidence;
        let mut risk_factors = Vec::new();

        // Apply dose-response
        if let Some(dr) = &profile.dose_response {
            let dose_factor = (dose_mg / dr.reference_dose).powf(dr.exponent);
            probability *= dose_factor;
            if dose_mg > dr.reference_dose {
                risk_factors.push(format!("High dose ({} mg)", dose_mg));
            }
        }

        // Apply risk modifiers based on patient factors
        for modifier in &profile.risk_modifiers {
            let applies = self.check_risk_factor(&modifier.factor);
            if applies {
                probability *= modifier.relative_risk;
                risk_factors.push(modifier.factor.clone());
            }
        }

        // Cap at 1.0
        probability = probability.min(1.0);

        // Calculate NNH
        let nnh = if probability > 0.0 {
            Some(1.0 / probability)
        } else {
            None
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(profile, probability);

        AdverseEventPrediction {
            event: profile.event.preferred_term.clone(),
            probability,
            expected_severity: profile.event.typical_severity,
            risk_factors,
            recommendations,
            nnh,
        }
    }

    /// Check if a risk factor applies to current patient
    fn check_risk_factor(&self, factor: &str) -> bool {
        match factor {
            "elderly" => self.patient.is_elderly(),
            "hepatic_impairment" => self.patient.hepatic_impairment() != HepaticFunction::Normal,
            "renal_impairment" => self.patient.renal_impairment() != RenalFunction::Normal,
            _ => {
                // Check comorbidities
                self.patient.comorbidities.iter().any(|c| c.contains(factor))
                    || self.patient.medications.iter().any(|m| m.contains(factor))
            }
        }
    }

    /// Generate recommendations based on risk
    fn generate_recommendations(&self, profile: &DrugEventProfile, probability: f64) -> Vec<String> {
        let mut recs = Vec::new();

        if probability > 0.1 {
            recs.push("Consider alternative medication".to_string());
        } else if probability > 0.01 {
            recs.push("Monitor closely for early signs".to_string());
        }

        // Event-specific recommendations
        match profile.event.soc {
            SystemOrganClass::Hepatobiliary => {
                recs.push("Monitor liver function tests".to_string());
            }
            SystemOrganClass::Cardiac => {
                recs.push("Obtain baseline ECG".to_string());
                recs.push("Monitor QTc interval".to_string());
            }
            SystemOrganClass::Respiratory => {
                recs.push("Have reversal agent available".to_string());
            }
            _ => {}
        }

        // Patient-specific
        if self.patient.is_elderly() {
            recs.push("Start with lower dose in elderly".to_string());
        }

        recs
    }

    /// Get overall risk summary for a regimen
    pub fn risk_summary(&self, drugs: &[(String, f64)]) -> RiskSummary {
        let mut all_predictions: Vec<AdverseEventPrediction> = Vec::new();

        for (drug, dose) in drugs {
            let predictions = self.predict(drug, *dose);
            all_predictions.extend(predictions);
        }

        // Find highest severity predicted event
        let highest_severity = all_predictions.iter()
            .map(|p| p.expected_severity)
            .max()
            .unwrap_or(Severity::Mild);

        // Calculate overall risk score (0-10)
        let risk_score: f64 = all_predictions.iter()
            .map(|p| p.probability * p.expected_severity.grade() as f64)
            .sum::<f64>()
            .min(10.0);

        // Get top risks
        let top_risks: Vec<String> = all_predictions.iter()
            .take(5)
            .map(|p| format!("{} ({:.1}%)", p.event, p.probability * 100.0))
            .collect();

        RiskSummary {
            overall_risk_score: risk_score,
            highest_severity_predicted: highest_severity,
            top_risks,
            total_events_predicted: all_predictions.len(),
            recommendations: self.aggregate_recommendations(&all_predictions),
        }
    }

    /// Aggregate recommendations from all predictions
    fn aggregate_recommendations(&self, predictions: &[AdverseEventPrediction]) -> Vec<String> {
        let mut recs: Vec<String> = predictions.iter()
            .flat_map(|p| p.recommendations.clone())
            .collect();

        recs.sort();
        recs.dedup();

        recs
    }
}

/// Summary of overall risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    /// Overall risk score (0-10)
    pub overall_risk_score: f64,
    /// Highest severity event predicted
    pub highest_severity_predicted: Severity,
    /// Top 5 risks by probability
    pub top_risks: Vec<String>,
    /// Total number of events with non-negligible probability
    pub total_events_predicted: usize,
    /// Aggregated recommendations
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_prediction() {
        let predictor = AdverseEventPredictor::new();

        let predictions = predictor.predict("diazepam", 10.0);

        assert!(!predictions.is_empty());
        assert!(predictions[0].probability > 0.0);
    }

    #[test]
    fn test_dose_response() {
        let predictor = AdverseEventPredictor::new();

        let low_dose = predictor.predict("diazepam", 2.0);
        let high_dose = predictor.predict("diazepam", 20.0);

        // Higher dose should have higher somnolence probability
        let low_somn = low_dose.iter()
            .find(|p| p.event == "Somnolence")
            .map(|p| p.probability)
            .unwrap_or(0.0);

        let high_somn = high_dose.iter()
            .find(|p| p.event == "Somnolence")
            .map(|p| p.probability)
            .unwrap_or(0.0);

        assert!(high_somn > low_somn);
    }

    #[test]
    fn test_elderly_risk() {
        let mut predictor = AdverseEventPredictor::new();

        // Young patient
        predictor.set_patient(PatientRiskFactors::new(30.0, 100.0));
        let young_pred = predictor.predict("diazepam", 10.0);

        // Elderly patient
        predictor.set_patient(PatientRiskFactors::new(75.0, 60.0));
        let elderly_pred = predictor.predict("diazepam", 10.0);

        let young_risk = young_pred.iter()
            .find(|p| p.event == "Somnolence")
            .map(|p| p.probability)
            .unwrap_or(0.0);

        let elderly_risk = elderly_pred.iter()
            .find(|p| p.event == "Somnolence")
            .map(|p| p.probability)
            .unwrap_or(0.0);

        assert!(elderly_risk > young_risk);
    }

    #[test]
    fn test_risk_summary() {
        let predictor = AdverseEventPredictor::new();

        let drugs = vec![
            ("diazepam".to_string(), 10.0),
            ("fluoxetine".to_string(), 20.0),
        ];

        let summary = predictor.risk_summary(&drugs);

        assert!(summary.overall_risk_score >= 0.0);
        assert!(!summary.recommendations.is_empty());
    }
}
