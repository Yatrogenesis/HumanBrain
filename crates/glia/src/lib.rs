//! Glial cell models for realistic brain simulation.
//!
//! Glial cells perform critical support functions:
//! - Astrocytes: Glutamate uptake, K+ buffering, neurovascular coupling
//! - Oligodendrocytes: Myelination, axon support
//! - Microglia: Immune response, synaptic pruning

pub mod astrocytes;
pub mod oligodendrocytes;
pub mod microglia;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GliaError {
    #[error("Invalid glial configuration: {0}")]
    InvalidConfiguration(String),
}

pub type Result<T> = std::result::Result<T, GliaError>;

pub use astrocytes::Astrocyte;
pub use oligodendrocytes::Oligodendrocyte;
pub use microglia::Microglia;
