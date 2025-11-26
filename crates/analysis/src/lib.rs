//! Neural Dynamics Analysis - Chaotic Attractor Signatures
//!
//! Este módulo extrae "firmas dinámicas" de la actividad neural simulada,
//! permitiendo caracterizar regiones cerebrales como sistemas dinámicos
//! y usar esa información para parametrizar simulaciones macro-escala.
//!
//! ## Pipeline:
//! 1. **Input**: Time series de voltajes/spikes de simulación GPU micro-escala
//! 2. **Analysis**: Cálculo de D₂ (dimensión de correlación) y λ₁ (Lyapunov)
//! 3. **Output**: AttractorSignature → parámetros para macro_brain
//!
//! ## Referencias Científicas:
//! - Grassberger & Procaccia (1983): Correlation dimension algorithm
//! - Rosenstein et al. (1993): Practical method for Lyapunov exponents
//! - Stam (2005): Nonlinear dynamical analysis of EEG/MEG signals

pub mod attractor_analysis;

pub use attractor_analysis::{
    AttractorSignature,
    analyze_spike_train,
    analyze_voltage_trace,
    correlation_dimension,
    max_lyapunov_exponent,
};

/// Re-export for convenience
pub use attractor_analysis::DynamicalRegime;
