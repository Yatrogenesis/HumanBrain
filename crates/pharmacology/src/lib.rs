//! Mechanistic Pharmacology Crate
//! ================================
//!
//! First-principles biophysical models for drug-receptor interactions.
//! Enables prediction of effects for novel compounds and drug combinations.
//!
//! # Key Modules
//!
//! ## Core Pharmacology
//! - `receptor_mechanisms`: GABA_A receptor binding and modulation
//! - `pharmacokinetics`: ADME modeling (absorption, distribution, metabolism, elimination)
//! - `ion_dynamics`: Nernst-Planck equations for ion channels
//!
//! ## Advanced Kinetics
//! - `enzyme_kinetics`: Michaelis-Menten with saturation detection
//! - `compartments`: Microanatomical drug distribution
//! - `active_transport`: P-gp efflux and transporter dynamics
//!
//! ## Receptor Dynamics
//! - `receptor_trafficking`: Desensitization, internalization, tolerance
//!
//! ## Individual Variation
//! - `pharmacogenomics`: CYP450 polymorphisms and phenotypes
//! - `stochastic_resonance`: Chaotic threshold dynamics, rare events
//!
//! ## Safety
//! - `reactive_metabolites`: GSH balance and hepatotoxicity
//! - `adverse_events`: Bayesian adverse event predictor
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

// Re-exports for convenience
pub use receptor_mechanisms::*;
pub use enzyme_kinetics::{EnzymeKinetics, SaturationRegime, Cyp450Database};
pub use compartments::{MultiCompartmentModel, CompartmentType};
pub use pharmacogenomics::{PharmacogenomicProfile, MetabolizerPhenotype, CypIsoform};
pub use stochastic_resonance::OntologicalOscillator;
pub use adverse_events::AdverseEventPredictor;
