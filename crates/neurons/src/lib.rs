//! Multi-compartmental neuron models for realistic brain simulation.
//!
//! This crate implements biologically realistic neuron models that go beyond
//! simple point neurons. Each neuron can have multiple compartments (soma,
//! dendrites, axon) with spatial dynamics governed by the cable equation.
//!
//! # Features
//!
//! - Multi-compartmental neurons with 10-1000 compartments
//! - Cable equation with spatial voltage dynamics
//! - Active dendritic conductances (NMDA spikes, Ca2+ spikes)
//! - Backpropagating action potentials
//! - Hodgkin-Huxley ion channels (Na+, K+, Ca2+, etc.)
//! - Realistic morphologies

pub mod compartmental;
pub mod channels;
pub mod morphology;

pub use compartmental::{Compartment, MultiCompartmentalNeuron, CompartmentType};
pub use channels::{IonChannel, HodgkinHuxleyNa, HodgkinHuxleyK, CalciumChannel, NMDAChannel};
pub use morphology::{NeuronMorphology, DendriticTree};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NeuronError {
    #[error("Invalid compartment configuration: {0}")]
    InvalidCompartment(String),
    
    #[error("Channel error: {0}")]
    ChannelError(String),
    
    #[error("Morphology error: {0}")]
    MorphologyError(String),
}

pub type Result<T> = std::result::Result<T, NeuronError>;

/// Physical constants
pub mod constants {
    /// Faraday constant (C/mol)
    pub const FARADAY: f64 = 96485.0;
    
    /// Gas constant (J/(mol*K))
    pub const R_GAS: f64 = 8.314;
    
    /// Temperature (K) - physiological temperature
    pub const TEMPERATURE: f64 = 310.0;
    
    /// Membrane capacitance (uF/cm^2)
    pub const C_M: f64 = 1.0;
    
    /// Axial resistivity (ohm*cm)
    pub const R_A: f64 = 100.0;
}

