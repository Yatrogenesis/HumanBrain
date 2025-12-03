//! Mechanistic Pharmacology Crate
//! ================================
//!
//! First-principles biophysical models for drug-receptor interactions.
//! Enables prediction of effects for novel compounds and drug combinations.
//!
//! # Key Modules
//!
//! ## Core Pharmacology
//! - : GABA_A receptor binding and modulation
//! - : ADME modeling (absorption, distribution, metabolism, elimination)
//! - : Nernst-Planck equations for ion channels
//!
//! ## Advanced Kinetics
//! - : Michaelis-Menten with saturation detection
//! - : Microanatomical drug distribution
//! - : P-gp efflux and transporter dynamics
//!
//! ## Receptor Dynamics
//! - : Desensitization, internalization, tolerance
//!
//! ## Individual Variation
//! - : CYP450 polymorphisms and phenotypes
//! - : Chaotic threshold dynamics, rare events
//!
//! ## Safety
//! - : GSH balance and hepatotoxicity
//! - : Bayesian adverse event predictor
//!
//! ## Validation
//! - : PET imaging and PK data from clinical studies
//!
//! # Author
//! Francisco Molina Burgos (Yatrogenesis)
//! ORCID: 0009-0008-6093-8267

// Core modules
pub mod receptor_mechanisms;
pub mod ion_dynamics;
pub mod pharmacokinetics;

// Advanced kinetics
pub mod enzyme_kinetics;
pub mod compartments;
pub mod active_transport;

// Receptor dynamics
pub mod receptor_trafficking;

// Individual variation
pub mod pharmacogenomics;
pub mod stochastic_resonance;

// Safety
pub mod reactive_metabolites;
pub mod adverse_events;

// Validation against clinical literature
pub mod clinical_literature;

// Re-exports for convenience
pub use receptor_mechanisms::*;
pub use enzyme_kinetics::{EnzymeKinetics, SaturationRegime, Cyp450Database};
pub use compartments::{MultiCompartmentModel, CompartmentType};
pub use pharmacogenomics::{PharmacogenomicProfile, MetabolizerPhenotype, CypIsoform};
pub use stochastic_resonance::OntologicalOscillator;
pub use adverse_events::AdverseEventPredictor;
pub use clinical_literature::{ClinicalLiteratureDb, ValidationResult, calculate_occupancy_from_ki};
