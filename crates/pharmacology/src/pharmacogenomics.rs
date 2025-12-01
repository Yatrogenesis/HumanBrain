//! Pharmacogenomics: CYP450 Polymorphisms and Individual Variation
//! =================================================================
//!
//! Models genetic variation in drug metabolism including:
//! - CYP450 allele frequencies and activity scores
//! - Phenotype prediction (PM, IM, NM, UM)
//! - Drug-specific dosing recommendations
//! - Population-level variation modeling
//!
//! # CYP2D6 Allele Activity Scores
//!
//! | Allele | Activity | Notes                    |
//! |--------|----------|--------------------------|
//! | *1     | 1.0      | Normal function          |
//! | *2     | 1.0      | Normal function          |
//! | *4     | 0.0      | No function (most common)|
//! | *5     | 0.0      | Gene deletion            |
//! | *10    | 0.25     | Reduced (Asian common)   |
//! | *17    | 0.5      | Reduced (African common) |
//! | *41    | 0.5      | Reduced                  |
//! | *1xN   | N        | Gene duplication         |
//!
//! # Phenotype Classification (CPIC)
//!
//! | Activity Score | Phenotype              |
//! |----------------|------------------------|
//! | 0              | Poor Metabolizer (PM)  |
//! | 0.25-1.0       | Intermediate (IM)      |
//! | 1.25-2.25      | Normal (NM)            |
//! | >2.25          | Ultrarapid (UM)        |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CYP450 enzyme isoform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CypIsoform {
    /// CYP2D6 - highly polymorphic, ~25% of drugs
    Cyp2d6,
    /// CYP2C19 - PPIs, clopidogrel, some benzos
    Cyp2c19,
    /// CYP2C9 - warfarin, phenytoin, NSAIDs
    Cyp2c9,
    /// CYP3A4 - most abundant, ~50% of drugs
    Cyp3a4,
    /// CYP3A5 - similar to 3A4, polymorphic
    Cyp3a5,
    /// CYP1A2 - caffeine, theophylline
    Cyp1a2,
    /// CYP2B6 - bupropion, efavirenz
    Cyp2b6,
    /// CYP2E1 - ethanol, acetaminophen
    Cyp2e1,
}

impl CypIsoform {
    /// Typical hepatic content (pmol/mg microsomal protein)
    pub fn hepatic_content(&self) -> f64 {
        match self {
            CypIsoform::Cyp3a4 => 108.0,    // Most abundant
            CypIsoform::Cyp2c9 => 73.0,
            CypIsoform::Cyp2c19 => 14.0,
            CypIsoform::Cyp2d6 => 10.0,     // Low but very important
            CypIsoform::Cyp3a5 => 2.0,      // Variable
            CypIsoform::Cyp1a2 => 45.0,
            CypIsoform::Cyp2b6 => 11.0,
            CypIsoform::Cyp2e1 => 49.0,
        }
    }

    /// Is this isoform highly polymorphic?
    pub fn is_highly_polymorphic(&self) -> bool {
        matches!(self, CypIsoform::Cyp2d6 | CypIsoform::Cyp2c19 |
                       CypIsoform::Cyp2c9 | CypIsoform::Cyp3a5)
    }
}

/// Metabolizer phenotype based on activity score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetabolizerPhenotype {
    /// No enzyme activity - drug accumulation risk
    PoorMetabolizer,
    /// Reduced activity - may need dose reduction
    IntermediateMetabolizer,
    /// Normal activity - standard dosing
    NormalMetabolizer,
    /// Increased activity - may need dose increase or prodrug efficacy issue
    UltrarapidMetabolizer,
    /// Cannot determine from genotype
    Indeterminate,
}

impl MetabolizerPhenotype {
    /// Activity score multiplier relative to normal
    pub fn activity_multiplier(&self) -> f64 {
        match self {
            MetabolizerPhenotype::PoorMetabolizer => 0.0,
            MetabolizerPhenotype::IntermediateMetabolizer => 0.5,
            MetabolizerPhenotype::NormalMetabolizer => 1.0,
            MetabolizerPhenotype::UltrarapidMetabolizer => 2.0,
            MetabolizerPhenotype::Indeterminate => 1.0,
        }
    }

    /// Dose adjustment factor (inverse of activity for active drugs)
    pub fn dose_adjustment_active_drug(&self) -> f64 {
        match self {
            MetabolizerPhenotype::PoorMetabolizer => 0.25,      // Reduce dose
            MetabolizerPhenotype::IntermediateMetabolizer => 0.5,
            MetabolizerPhenotype::NormalMetabolizer => 1.0,
            MetabolizerPhenotype::UltrarapidMetabolizer => 2.0,  // May need higher dose
            MetabolizerPhenotype::Indeterminate => 1.0,
        }
    }

    /// Dose adjustment for prodrugs (need activation)
    pub fn dose_adjustment_prodrug(&self) -> f64 {
        match self {
            MetabolizerPhenotype::PoorMetabolizer => 0.0,       // Avoid - won't activate
            MetabolizerPhenotype::IntermediateMetabolizer => 0.5,
            MetabolizerPhenotype::NormalMetabolizer => 1.0,
            MetabolizerPhenotype::UltrarapidMetabolizer => 0.5,  // Reduce - too much active
            MetabolizerPhenotype::Indeterminate => 1.0,
        }
    }

    /// Clinical recommendation
    pub fn recommendation(&self, is_prodrug: bool) -> &'static str {
        match (self, is_prodrug) {
            (MetabolizerPhenotype::PoorMetabolizer, false) =>
                "Reduce dose by 50-75% or use alternative drug",
            (MetabolizerPhenotype::PoorMetabolizer, true) =>
                "AVOID - prodrug will not be activated",
            (MetabolizerPhenotype::IntermediateMetabolizer, false) =>
                "Consider 25-50% dose reduction",
            (MetabolizerPhenotype::IntermediateMetabolizer, true) =>
                "Reduced efficacy expected, consider alternative",
            (MetabolizerPhenotype::NormalMetabolizer, _) =>
                "Standard dosing appropriate",
            (MetabolizerPhenotype::UltrarapidMetabolizer, false) =>
                "Standard or increased dose may be needed",
            (MetabolizerPhenotype::UltrarapidMetabolizer, true) =>
                "CAUTION - excessive active metabolite, reduce dose",
            (MetabolizerPhenotype::Indeterminate, _) =>
                "Monitor closely, genotype if possible",
        }
    }
}

/// Known CYP allele with activity value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CypAllele {
    /// Allele name (e.g., "*4", "*17")
    pub name: String,
    /// Activity score (0.0 = no function, 1.0 = normal)
    pub activity_score: f64,
    /// Is this allele functional?
    pub functional: bool,
    /// Common in which populations
    pub population_notes: String,
}

impl CypAllele {
    pub fn new(name: &str, activity: f64, notes: &str) -> Self {
        Self {
            name: name.to_string(),
            activity_score: activity,
            functional: activity > 0.0,
            population_notes: notes.to_string(),
        }
    }
}

/// Genotype for a CYP enzyme (diplotype)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CypGenotype {
    /// Enzyme isoform
    pub isoform: CypIsoform,
    /// Maternal allele
    pub allele1: CypAllele,
    /// Paternal allele
    pub allele2: CypAllele,
    /// Gene copy number variation (if applicable)
    pub copy_number: u8,
}

impl CypGenotype {
    pub fn new(isoform: CypIsoform, allele1: CypAllele, allele2: CypAllele) -> Self {
        Self {
            isoform,
            allele1,
            allele2,
            copy_number: 2,
        }
    }

    /// Calculate total activity score
    pub fn activity_score(&self) -> f64 {
        let base = self.allele1.activity_score + self.allele2.activity_score;
        // Adjust for copy number
        if self.copy_number > 2 {
            base + (self.copy_number - 2) as f64 * 1.0  // Each extra copy adds ~1.0
        } else if self.copy_number == 1 {
            base * 0.5  // Gene deletion
        } else {
            base
        }
    }

    /// Determine phenotype from activity score
    pub fn phenotype(&self) -> MetabolizerPhenotype {
        let score = self.activity_score();

        match self.isoform {
            CypIsoform::Cyp2d6 => {
                if score == 0.0 {
                    MetabolizerPhenotype::PoorMetabolizer
                } else if score <= 1.0 {
                    MetabolizerPhenotype::IntermediateMetabolizer
                } else if score <= 2.25 {
                    MetabolizerPhenotype::NormalMetabolizer
                } else {
                    MetabolizerPhenotype::UltrarapidMetabolizer
                }
            }
            CypIsoform::Cyp2c19 => {
                if score == 0.0 {
                    MetabolizerPhenotype::PoorMetabolizer
                } else if score <= 1.0 {
                    MetabolizerPhenotype::IntermediateMetabolizer
                } else if score <= 2.0 {
                    MetabolizerPhenotype::NormalMetabolizer
                } else {
                    MetabolizerPhenotype::UltrarapidMetabolizer
                }
            }
            CypIsoform::Cyp2c9 => {
                if score == 0.0 {
                    MetabolizerPhenotype::PoorMetabolizer
                } else if score < 1.5 {
                    MetabolizerPhenotype::IntermediateMetabolizer
                } else {
                    MetabolizerPhenotype::NormalMetabolizer
                }
            }
            _ => {
                // Less well-characterized
                if score < 0.5 {
                    MetabolizerPhenotype::PoorMetabolizer
                } else if score < 1.5 {
                    MetabolizerPhenotype::IntermediateMetabolizer
                } else {
                    MetabolizerPhenotype::NormalMetabolizer
                }
            }
        }
    }

    /// Display diplotype (e.g., "*1/*4")
    pub fn diplotype(&self) -> String {
        format!("{}/{}", self.allele1.name, self.allele2.name)
    }
}

/// Database of CYP alleles
#[derive(Debug, Clone, Default)]
pub struct AlleleDatabase {
    pub alleles: HashMap<(CypIsoform, String), CypAllele>,
}

impl AlleleDatabase {
    pub fn new() -> Self {
        let mut db = Self::default();

        // CYP2D6 alleles
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*1", 1.0, "Normal function, common"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*2", 1.0, "Normal function"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*3", 0.0, "No function, frameshift"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*4", 0.0, "No function, splicing defect, European ~20%"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*5", 0.0, "Gene deletion"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*6", 0.0, "No function, frameshift"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*9", 0.5, "Reduced function"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*10", 0.25, "Reduced, common in East Asian ~50%"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*17", 0.5, "Reduced, common in African ~20%"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*29", 0.5, "Reduced function"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*41", 0.5, "Reduced, common in Middle Eastern"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*1xN", 2.0, "Gene duplication, ultrarapid"));
        db.add(CypIsoform::Cyp2d6, CypAllele::new("*2xN", 2.0, "Gene duplication, ultrarapid"));

        // CYP2C19 alleles
        db.add(CypIsoform::Cyp2c19, CypAllele::new("*1", 1.0, "Normal function"));
        db.add(CypIsoform::Cyp2c19, CypAllele::new("*2", 0.0, "No function, common ~15-30%"));
        db.add(CypIsoform::Cyp2c19, CypAllele::new("*3", 0.0, "No function, Asian ~5%"));
        db.add(CypIsoform::Cyp2c19, CypAllele::new("*17", 1.5, "Increased function, European ~20%"));

        // CYP2C9 alleles
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*1", 1.0, "Normal function"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*2", 0.5, "Reduced, European ~10%"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*3", 0.1, "Markedly reduced, European ~6%"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*5", 0.5, "Reduced, African"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*6", 0.0, "No function"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*8", 0.5, "Reduced, African ~5%"));
        db.add(CypIsoform::Cyp2c9, CypAllele::new("*11", 0.5, "Reduced"));

        // CYP3A5 alleles
        db.add(CypIsoform::Cyp3a5, CypAllele::new("*1", 1.0, "Expressor, African ~70%"));
        db.add(CypIsoform::Cyp3a5, CypAllele::new("*3", 0.0, "Non-expressor, European ~90%"));
        db.add(CypIsoform::Cyp3a5, CypAllele::new("*6", 0.0, "Non-expressor"));
        db.add(CypIsoform::Cyp3a5, CypAllele::new("*7", 0.0, "Non-expressor"));

        // CYP1A2 alleles
        db.add(CypIsoform::Cyp1a2, CypAllele::new("*1A", 1.0, "Normal"));
        db.add(CypIsoform::Cyp1a2, CypAllele::new("*1C", 0.7, "Reduced"));
        db.add(CypIsoform::Cyp1a2, CypAllele::new("*1F", 1.5, "Increased inducibility"));

        db
    }

    fn add(&mut self, isoform: CypIsoform, allele: CypAllele) {
        self.alleles.insert((isoform, allele.name.clone()), allele);
    }

    pub fn get(&self, isoform: CypIsoform, name: &str) -> Option<&CypAllele> {
        self.alleles.get(&(isoform, name.to_string()))
    }

    /// Get default normal allele for an isoform
    pub fn normal_allele(&self, isoform: CypIsoform) -> CypAllele {
        let name = match isoform {
            CypIsoform::Cyp2d6 => "*1",
            CypIsoform::Cyp2c19 => "*1",
            CypIsoform::Cyp2c9 => "*1",
            CypIsoform::Cyp3a4 => "*1",
            CypIsoform::Cyp3a5 => "*1",
            CypIsoform::Cyp1a2 => "*1A",
            CypIsoform::Cyp2b6 => "*1",
            CypIsoform::Cyp2e1 => "*1",
        };

        self.get(isoform, name)
            .cloned()
            .unwrap_or_else(|| CypAllele::new("*1", 1.0, "Normal"))
    }
}

/// Individual's complete pharmacogenomic profile
#[derive(Debug, Clone)]
pub struct PharmacogenomicProfile {
    /// Genotypes for each CYP enzyme
    pub genotypes: HashMap<CypIsoform, CypGenotype>,
    /// Other pharmacogenomic markers
    pub other_markers: HashMap<String, String>,
    /// Population ancestry (affects allele prior probabilities)
    pub ancestry: Option<Ancestry>,
}

impl Default for PharmacogenomicProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl PharmacogenomicProfile {
    /// Create profile with all normal metabolizers
    pub fn new() -> Self {
        let db = AlleleDatabase::new();
        let mut genotypes = HashMap::new();

        for isoform in &[CypIsoform::Cyp2d6, CypIsoform::Cyp2c19, CypIsoform::Cyp2c9,
                        CypIsoform::Cyp3a4, CypIsoform::Cyp3a5, CypIsoform::Cyp1a2,
                        CypIsoform::Cyp2b6, CypIsoform::Cyp2e1] {
            let allele = db.normal_allele(*isoform);
            genotypes.insert(*isoform, CypGenotype::new(*isoform, allele.clone(), allele));
        }

        Self {
            genotypes,
            other_markers: HashMap::new(),
            ancestry: None,
        }
    }

    /// Create poor metabolizer for a specific enzyme
    pub fn with_pm(mut self, isoform: CypIsoform) -> Self {
        let null_allele = CypAllele::new("*null", 0.0, "No function");
        self.genotypes.insert(isoform, CypGenotype::new(isoform, null_allele.clone(), null_allele));
        self
    }

    /// Create ultrarapid metabolizer (gene duplication)
    pub fn with_um(mut self, isoform: CypIsoform) -> Self {
        let allele = CypAllele::new("*1xN", 2.0, "Gene duplication");
        self.genotypes.insert(isoform, CypGenotype {
            isoform,
            allele1: CypAllele::new("*1", 1.0, "Normal"),
            allele2: CypAllele::new("*1", 1.0, "Normal"),
            copy_number: 3,
        });
        self
    }

    /// Set specific genotype
    pub fn set_genotype(&mut self, genotype: CypGenotype) {
        self.genotypes.insert(genotype.isoform, genotype);
    }

    /// Get phenotype for an enzyme
    pub fn phenotype(&self, isoform: CypIsoform) -> MetabolizerPhenotype {
        self.genotypes
            .get(&isoform)
            .map(|g| g.phenotype())
            .unwrap_or(MetabolizerPhenotype::NormalMetabolizer)
    }

    /// Get activity score for an enzyme
    pub fn activity_score(&self, isoform: CypIsoform) -> f64 {
        self.genotypes
            .get(&isoform)
            .map(|g| g.activity_score())
            .unwrap_or(2.0)  // Normal diplotype = 2.0
    }

    /// Calculate metabolism rate relative to normal
    pub fn metabolism_rate(&self, isoform: CypIsoform) -> f64 {
        self.activity_score(isoform) / 2.0  // Normalize to 2.0 = 1.0 rate
    }

    /// Generate clinical report
    pub fn clinical_report(&self) -> ClinicalReport {
        let mut genes = Vec::new();

        for (isoform, genotype) in &self.genotypes {
            let phenotype = genotype.phenotype();
            genes.push(GeneReport {
                gene: format!("{:?}", isoform),
                diplotype: genotype.diplotype(),
                activity_score: genotype.activity_score(),
                phenotype,
                clinical_significance: match phenotype {
                    MetabolizerPhenotype::PoorMetabolizer |
                    MetabolizerPhenotype::UltrarapidMetabolizer => "Actionable",
                    MetabolizerPhenotype::IntermediateMetabolizer => "Informative",
                    _ => "Normal",
                }.to_string(),
            });
        }

        ClinicalReport {
            genes,
            drug_recommendations: Vec::new(),  // Populated by drug lookup
        }
    }
}

/// Population ancestry for allele frequency estimation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Ancestry {
    European,
    African,
    EastAsian,
    SouthAsian,
    MiddleEastern,
    Latino,
    Mixed,
}

impl Ancestry {
    /// CYP2D6 PM frequency by ancestry
    pub fn cyp2d6_pm_frequency(&self) -> f64 {
        match self {
            Ancestry::European => 0.07,
            Ancestry::African => 0.02,
            Ancestry::EastAsian => 0.01,
            Ancestry::SouthAsian => 0.02,
            Ancestry::MiddleEastern => 0.02,
            Ancestry::Latino => 0.04,
            Ancestry::Mixed => 0.04,
        }
    }

    /// CYP2D6 UM frequency by ancestry
    pub fn cyp2d6_um_frequency(&self) -> f64 {
        match self {
            Ancestry::European => 0.03,
            Ancestry::African => 0.30,
            Ancestry::EastAsian => 0.01,
            Ancestry::SouthAsian => 0.01,
            Ancestry::MiddleEastern => 0.10,
            Ancestry::Latino => 0.05,
            Ancestry::Mixed => 0.05,
        }
    }

    /// CYP2C19 PM frequency
    pub fn cyp2c19_pm_frequency(&self) -> f64 {
        match self {
            Ancestry::European => 0.02,
            Ancestry::African => 0.04,
            Ancestry::EastAsian => 0.15,
            Ancestry::SouthAsian => 0.05,
            Ancestry::MiddleEastern => 0.04,
            Ancestry::Latino => 0.03,
            Ancestry::Mixed => 0.05,
        }
    }
}

/// Report for a single gene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneReport {
    pub gene: String,
    pub diplotype: String,
    pub activity_score: f64,
    pub phenotype: MetabolizerPhenotype,
    pub clinical_significance: String,
}

/// Complete clinical pharmacogenomics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalReport {
    pub genes: Vec<GeneReport>,
    pub drug_recommendations: Vec<DrugRecommendation>,
}

/// Drug-specific recommendation based on genotype
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugRecommendation {
    pub drug: String,
    pub affected_genes: Vec<String>,
    pub recommendation: String,
    pub dose_adjustment: f64,
    pub strength: String,  // "Strong", "Moderate", "Optional"
    pub alternatives: Vec<String>,
}

/// Database of drug-gene interactions for CPIC guidelines
#[derive(Debug, Clone, Default)]
pub struct DrugGeneInteractions {
    pub interactions: HashMap<String, Vec<DrugGeneInteraction>>,
}

/// Single drug-gene interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugGeneInteraction {
    pub drug: String,
    pub gene: CypIsoform,
    pub is_prodrug: bool,
    pub pm_recommendation: String,
    pub um_recommendation: String,
}

impl DrugGeneInteractions {
    pub fn new() -> Self {
        let mut interactions: HashMap<String, Vec<DrugGeneInteraction>> = HashMap::new();

        // Codeine - CYP2D6 (classic example)
        interactions.insert("codeine".to_string(), vec![
            DrugGeneInteraction {
                drug: "codeine".to_string(),
                gene: CypIsoform::Cyp2d6,
                is_prodrug: true,
                pm_recommendation: "AVOID - codeine ineffective, no morphine formed".to_string(),
                um_recommendation: "AVOID - risk of morphine toxicity, respiratory depression".to_string(),
            }
        ]);

        // Tramadol - CYP2D6
        interactions.insert("tramadol".to_string(), vec![
            DrugGeneInteraction {
                drug: "tramadol".to_string(),
                gene: CypIsoform::Cyp2d6,
                is_prodrug: true,
                pm_recommendation: "Reduced efficacy, consider alternative".to_string(),
                um_recommendation: "Risk of toxicity, consider dose reduction or alternative".to_string(),
            }
        ]);

        // Fluoxetine - CYP2D6 (inhibitor and substrate)
        interactions.insert("fluoxetine".to_string(), vec![
            DrugGeneInteraction {
                drug: "fluoxetine".to_string(),
                gene: CypIsoform::Cyp2d6,
                is_prodrug: false,
                pm_recommendation: "Consider 50% dose reduction".to_string(),
                um_recommendation: "Standard dosing, may need higher dose for efficacy".to_string(),
            }
        ]);

        // Clopidogrel - CYP2C19
        interactions.insert("clopidogrel".to_string(), vec![
            DrugGeneInteraction {
                drug: "clopidogrel".to_string(),
                gene: CypIsoform::Cyp2c19,
                is_prodrug: true,
                pm_recommendation: "AVOID - use alternative (prasugrel, ticagrelor)".to_string(),
                um_recommendation: "Standard therapy appropriate".to_string(),
            }
        ]);

        // Omeprazole - CYP2C19
        interactions.insert("omeprazole".to_string(), vec![
            DrugGeneInteraction {
                drug: "omeprazole".to_string(),
                gene: CypIsoform::Cyp2c19,
                is_prodrug: false,
                pm_recommendation: "Increased efficacy, consider dose reduction for chronic use".to_string(),
                um_recommendation: "May need increased dose for H. pylori eradication".to_string(),
            }
        ]);

        // Warfarin - CYP2C9 and VKORC1
        interactions.insert("warfarin".to_string(), vec![
            DrugGeneInteraction {
                drug: "warfarin".to_string(),
                gene: CypIsoform::Cyp2c9,
                is_prodrug: false,
                pm_recommendation: "Reduce starting dose by 50-75%, high bleeding risk".to_string(),
                um_recommendation: "Standard dosing".to_string(),
            }
        ]);

        // Phenytoin - CYP2C9
        interactions.insert("phenytoin".to_string(), vec![
            DrugGeneInteraction {
                drug: "phenytoin".to_string(),
                gene: CypIsoform::Cyp2c9,
                is_prodrug: false,
                pm_recommendation: "Reduce starting dose by 50%, monitor levels".to_string(),
                um_recommendation: "Standard dosing".to_string(),
            }
        ]);

        Self { interactions }
    }

    /// Get recommendations for a drug based on phenotype
    pub fn get_recommendation(
        &self,
        drug: &str,
        profile: &PharmacogenomicProfile
    ) -> Option<DrugRecommendation> {
        let interactions = self.interactions.get(drug)?;

        let mut recommendation = String::new();
        let mut dose_adj = 1.0;
        let mut affected_genes = Vec::new();

        for interaction in interactions {
            let phenotype = profile.phenotype(interaction.gene);

            let rec = match phenotype {
                MetabolizerPhenotype::PoorMetabolizer => &interaction.pm_recommendation,
                MetabolizerPhenotype::UltrarapidMetabolizer => &interaction.um_recommendation,
                MetabolizerPhenotype::IntermediateMetabolizer =>
                    if interaction.is_prodrug {
                        "May have reduced efficacy"
                    } else {
                        "Consider modest dose reduction"
                    },
                _ => "Standard dosing appropriate",
            };

            recommendation.push_str(&format!("{:?}: {}\n", interaction.gene, rec));
            affected_genes.push(format!("{:?}", interaction.gene));

            // Calculate dose adjustment
            if interaction.is_prodrug {
                dose_adj *= phenotype.dose_adjustment_prodrug();
            } else {
                dose_adj *= phenotype.dose_adjustment_active_drug();
            }
        }

        Some(DrugRecommendation {
            drug: drug.to_string(),
            affected_genes,
            recommendation,
            dose_adjustment: dose_adj,
            strength: if dose_adj == 0.0 { "Strong (Avoid)" } else { "Moderate" }.to_string(),
            alternatives: Vec::new(),
        })
    }
}

/// Population simulator for pharmacogenomics
#[derive(Debug, Clone)]
pub struct PopulationSimulator {
    /// Allele database
    pub allele_db: AlleleDatabase,
    /// Drug-gene interactions
    pub drug_genes: DrugGeneInteractions,
}

impl PopulationSimulator {
    pub fn new() -> Self {
        Self {
            allele_db: AlleleDatabase::new(),
            drug_genes: DrugGeneInteractions::new(),
        }
    }

    /// Generate a random profile based on ancestry
    pub fn generate_profile(&self, ancestry: Ancestry) -> PharmacogenomicProfile {
        let mut profile = PharmacogenomicProfile::new();
        profile.ancestry = Some(ancestry);

        // Simulate CYP2D6 based on ancestry frequencies
        let pm_freq = ancestry.cyp2d6_pm_frequency();
        let um_freq = ancestry.cyp2d6_um_frequency();

        // Simple random assignment (would use proper Hardy-Weinberg in full implementation)
        let rand_val = rand::random::<f64>();
        if rand_val < pm_freq {
            profile = profile.with_pm(CypIsoform::Cyp2d6);
        } else if rand_val < pm_freq + um_freq {
            profile = profile.with_um(CypIsoform::Cyp2d6);
        }

        // Simulate CYP2C19
        let pm_freq = ancestry.cyp2c19_pm_frequency();
        if rand::random::<f64>() < pm_freq {
            profile = profile.with_pm(CypIsoform::Cyp2c19);
        }

        profile
    }

    /// Estimate phenotype distribution in a population
    pub fn phenotype_distribution(
        &self,
        ancestry: Ancestry,
        gene: CypIsoform,
        n_samples: usize
    ) -> HashMap<MetabolizerPhenotype, f64> {
        let mut counts: HashMap<MetabolizerPhenotype, usize> = HashMap::new();

        for _ in 0..n_samples {
            let profile = self.generate_profile(ancestry);
            let phenotype = profile.phenotype(gene);
            *counts.entry(phenotype).or_insert(0) += 1;
        }

        counts.into_iter()
            .map(|(k, v)| (k, v as f64 / n_samples as f64))
            .collect()
    }
}

impl Default for PopulationSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_score() {
        let db = AlleleDatabase::new();

        let allele1 = db.get(CypIsoform::Cyp2d6, "*1").unwrap().clone();
        let allele4 = db.get(CypIsoform::Cyp2d6, "*4").unwrap().clone();

        let genotype = CypGenotype::new(CypIsoform::Cyp2d6, allele1, allele4);

        // *1/*4 should have activity score of 1.0
        assert!((genotype.activity_score() - 1.0).abs() < 0.01);
        assert_eq!(genotype.phenotype(), MetabolizerPhenotype::IntermediateMetabolizer);
    }

    #[test]
    fn test_poor_metabolizer() {
        let db = AlleleDatabase::new();

        let allele4 = db.get(CypIsoform::Cyp2d6, "*4").unwrap().clone();

        let genotype = CypGenotype::new(CypIsoform::Cyp2d6, allele4.clone(), allele4);

        assert_eq!(genotype.activity_score(), 0.0);
        assert_eq!(genotype.phenotype(), MetabolizerPhenotype::PoorMetabolizer);
    }

    #[test]
    fn test_codeine_recommendation() {
        let drug_genes = DrugGeneInteractions::new();

        // PM for CYP2D6
        let pm_profile = PharmacogenomicProfile::new().with_pm(CypIsoform::Cyp2d6);

        let rec = drug_genes.get_recommendation("codeine", &pm_profile).unwrap();
        assert!(rec.recommendation.contains("AVOID"));
        assert_eq!(rec.dose_adjustment, 0.0);

        // UM for CYP2D6
        let um_profile = PharmacogenomicProfile::new().with_um(CypIsoform::Cyp2d6);

        let rec = drug_genes.get_recommendation("codeine", &um_profile).unwrap();
        assert!(rec.recommendation.contains("AVOID"));
    }

    #[test]
    fn test_population_frequencies() {
        let simulator = PopulationSimulator::new();

        // East Asian should have higher CYP2C19 PM rate
        assert!(Ancestry::EastAsian.cyp2c19_pm_frequency() > Ancestry::European.cyp2c19_pm_frequency());

        // African should have higher CYP2D6 UM rate
        assert!(Ancestry::African.cyp2d6_um_frequency() > Ancestry::European.cyp2d6_um_frequency());
    }
}
