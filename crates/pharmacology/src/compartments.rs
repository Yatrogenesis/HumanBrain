//! Microanatomical Compartments for Drug Distribution
//! ====================================================
//!
//! Hierarchical compartment model with physiologically realistic time constants:
//!
//! | Compartment    | Volume | τ (time constant) | Primary processes           |
//! |----------------|--------|-------------------|------------------------------|
//! | Synaptic cleft | ~1 fL  | 100 µs           | Fast neurotransmitter action |
//! | Perisynaptic   | ~10 fL | 10 ms            | Spillover, reuptake          |
//! | Extrasynaptic  | ~1 pL  | 1 s              | Volume transmission          |
//! | Interstitial   | ~1 nL  | 1 min            | Glymphatic flow              |
//! | CSF            | 150 mL | hours            | Bulk circulation             |
//! | Blood-Brain    | varies | seconds-minutes  | BBB transport                |
//!
//! # Diffusion Model
//!
//! Fick's second law in 3D:
//! ```text
//! ∂C/∂t = D·∇²C - v·∇C + R
//! ```
//! where D is diffusion coefficient, v is bulk flow velocity, R is reaction term.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compartment types in order of increasing size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompartmentType {
    /// Synaptic cleft: ~20 nm width, ~1 femtoliter
    /// τ ≈ 100 µs for small molecules
    SynapticCleft,
    /// Perisynaptic region: extends ~500 nm from synapse
    /// τ ≈ 10 ms, captures spillover effects
    Perisynaptic,
    /// Extrasynaptic space: general extracellular space
    /// τ ≈ 1 s, volume transmission occurs here
    Extrasynaptic,
    /// Interstitial fluid: larger scale extracellular
    /// τ ≈ 1 min, glymphatic clearance
    Interstitial,
    /// Cerebrospinal fluid: bulk fluid compartment
    /// τ ≈ hours, circulates through ventricles
    Csf,
    /// Blood compartment: arterial/venous blood
    BloodArterial,
    BloodVenous,
    BloodCapillary,
    /// Cellular compartments
    Cytoplasm,
    Mitochondria,
    EndoplasmicReticulum,
}

impl CompartmentType {
    /// Typical volume in liters
    pub fn volume_l(&self) -> f64 {
        match self {
            CompartmentType::SynapticCleft => 1e-15,        // 1 femtoliter
            CompartmentType::Perisynaptic => 1e-14,         // 10 femtoliters
            CompartmentType::Extrasynaptic => 1e-12,        // 1 picoliter (per neuron)
            CompartmentType::Interstitial => 1e-9,          // 1 nanoliter (local region)
            CompartmentType::Csf => 0.150,                  // 150 mL total
            CompartmentType::BloodArterial => 0.750,        // ~750 mL cerebral
            CompartmentType::BloodVenous => 1.0,
            CompartmentType::BloodCapillary => 0.050,       // ~50 mL capillary
            CompartmentType::Cytoplasm => 1e-12,            // ~1 pL per cell
            CompartmentType::Mitochondria => 1e-14,
            CompartmentType::EndoplasmicReticulum => 1e-13,
        }
    }

    /// Time constant for equilibration (seconds)
    pub fn tau_s(&self) -> f64 {
        match self {
            CompartmentType::SynapticCleft => 1e-4,         // 100 µs
            CompartmentType::Perisynaptic => 1e-2,          // 10 ms
            CompartmentType::Extrasynaptic => 1.0,          // 1 s
            CompartmentType::Interstitial => 60.0,          // 1 min
            CompartmentType::Csf => 3600.0,                 // 1 hour
            CompartmentType::BloodArterial => 10.0,
            CompartmentType::BloodVenous => 30.0,
            CompartmentType::BloodCapillary => 1.0,
            CompartmentType::Cytoplasm => 0.1,
            CompartmentType::Mitochondria => 0.01,
            CompartmentType::EndoplasmicReticulum => 0.05,
        }
    }

    /// Tortuosity factor (λ) - ratio of actual to straight-line path
    /// Affects effective diffusion coefficient: D_eff = D_free / λ²
    pub fn tortuosity(&self) -> f64 {
        match self {
            CompartmentType::SynapticCleft => 1.0,          // Relatively free
            CompartmentType::Perisynaptic => 1.2,
            CompartmentType::Extrasynaptic => 1.6,          // Typical brain value
            CompartmentType::Interstitial => 1.8,
            CompartmentType::Csf => 1.0,                    // Free fluid
            CompartmentType::BloodArterial => 1.0,
            CompartmentType::BloodVenous => 1.0,
            CompartmentType::BloodCapillary => 1.0,
            CompartmentType::Cytoplasm => 2.0,              // Crowded with organelles
            CompartmentType::Mitochondria => 1.5,
            CompartmentType::EndoplasmicReticulum => 2.5,
        }
    }

    /// Volume fraction (α) - fraction of total tissue volume
    pub fn volume_fraction(&self) -> f64 {
        match self {
            CompartmentType::SynapticCleft => 0.001,
            CompartmentType::Perisynaptic => 0.01,
            CompartmentType::Extrasynaptic => 0.15,         // ~15-20% is typical
            CompartmentType::Interstitial => 0.20,
            CompartmentType::Csf => 0.10,
            CompartmentType::BloodArterial => 0.02,
            CompartmentType::BloodVenous => 0.03,
            CompartmentType::BloodCapillary => 0.01,
            CompartmentType::Cytoplasm => 0.40,
            CompartmentType::Mitochondria => 0.05,
            CompartmentType::EndoplasmicReticulum => 0.03,
        }
    }
}

/// A single compartment with concentration dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compartment {
    /// Compartment type
    pub compartment_type: CompartmentType,
    /// Current drug concentrations: drug_name -> concentration (µM)
    pub concentrations: HashMap<String, f64>,
    /// Volume of this specific compartment instance (L)
    pub volume_l: f64,
    /// pH of the compartment (affects ionization)
    pub ph: f64,
    /// Temperature (K)
    pub temperature_k: f64,
    /// Active transport rates: drug -> rate coefficient (1/s)
    pub active_transport: HashMap<String, f64>,
    /// Binding sites occupied (reduces free drug)
    pub bound_fraction: HashMap<String, f64>,
}

impl Compartment {
    pub fn new(compartment_type: CompartmentType) -> Self {
        let ph = match compartment_type {
            CompartmentType::SynapticCleft => 7.3,
            CompartmentType::Perisynaptic => 7.3,
            CompartmentType::Extrasynaptic => 7.3,
            CompartmentType::Interstitial => 7.35,
            CompartmentType::Csf => 7.32,
            CompartmentType::BloodArterial => 7.40,
            CompartmentType::BloodVenous => 7.35,
            CompartmentType::BloodCapillary => 7.38,
            CompartmentType::Cytoplasm => 7.2,
            CompartmentType::Mitochondria => 8.0,           // More alkaline
            CompartmentType::EndoplasmicReticulum => 7.1,
        };

        Self {
            compartment_type,
            concentrations: HashMap::new(),
            volume_l: compartment_type.volume_l(),
            ph,
            temperature_k: 310.15,
            active_transport: HashMap::new(),
            bound_fraction: HashMap::new(),
        }
    }

    /// Get free (unbound) concentration of a drug
    pub fn free_concentration(&self, drug: &str) -> f64 {
        let total = self.concentrations.get(drug).copied().unwrap_or(0.0);
        let bound_frac = self.bound_fraction.get(drug).copied().unwrap_or(0.0);
        total * (1.0 - bound_frac)
    }

    /// Calculate ionized fraction using Henderson-Hasselbalch
    /// For acids: ionized = 1 / (1 + 10^(pKa - pH))
    /// For bases: ionized = 1 / (1 + 10^(pH - pKa))
    pub fn ionized_fraction(&self, pka: f64, is_acid: bool) -> f64 {
        if is_acid {
            1.0 / (1.0 + 10.0_f64.powf(pka - self.ph))
        } else {
            1.0 / (1.0 + 10.0_f64.powf(self.ph - pka))
        }
    }

    /// Amount of drug in this compartment (µmol)
    pub fn amount_umol(&self, drug: &str) -> f64 {
        self.concentrations.get(drug).copied().unwrap_or(0.0) * self.volume_l * 1000.0
    }

    /// Add drug to compartment
    pub fn add_drug(&mut self, drug: &str, amount_umol: f64) {
        let delta_concentration = amount_umol / (self.volume_l * 1000.0);
        *self.concentrations.entry(drug.to_string()).or_insert(0.0) += delta_concentration;
    }

    /// Set bound fraction for a drug
    pub fn set_binding(&mut self, drug: &str, bound_fraction: f64) {
        self.bound_fraction.insert(drug.to_string(), bound_fraction.clamp(0.0, 0.99));
    }
}

/// Inter-compartment transfer mechanism
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransferMechanism {
    /// Passive diffusion (Fick's law)
    PassiveDiffusion {
        /// Permeability-surface area product (mL/min)
        ps_product: f64,
    },
    /// Facilitated diffusion (saturable)
    FacilitatedDiffusion {
        /// Maximum transport rate (µmol/min)
        vmax: f64,
        /// Michaelis constant (µM)
        km: f64,
    },
    /// Active transport (ATP-dependent, against gradient)
    ActiveTransport {
        /// Maximum transport rate (µmol/min)
        vmax: f64,
        /// Michaelis constant (µM)
        km: f64,
        /// ATP consumption per molecule transported
        atp_per_molecule: f64,
    },
    /// P-glycoprotein efflux (drug pumped back)
    PgpEfflux {
        /// Efflux rate constant (1/min)
        k_efflux: f64,
        /// IC50 for P-gp inhibition (µM)
        ic50_inhibition: f64,
    },
    /// Bulk flow (convective transport)
    BulkFlow {
        /// Flow rate (mL/min)
        flow_rate: f64,
    },
}

/// Connection between two compartments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompartmentConnection {
    /// Source compartment index
    pub from: usize,
    /// Destination compartment index
    pub to: usize,
    /// Transfer mechanism
    pub mechanism: TransferMechanism,
    /// Is this a bidirectional connection?
    pub bidirectional: bool,
}

impl CompartmentConnection {
    /// Calculate transfer rate (µmol/s) from source to destination
    pub fn transfer_rate(&self, source_conc: f64, dest_conc: f64, inhibitor_conc: f64) -> f64 {
        match self.mechanism {
            TransferMechanism::PassiveDiffusion { ps_product } => {
                // Fick's law: flux = PS * (C_source - C_dest)
                let gradient = source_conc - dest_conc;
                ps_product / 60.0 * gradient  // Convert mL/min to mL/s
            }
            TransferMechanism::FacilitatedDiffusion { vmax, km } => {
                // Michaelis-Menten for net transport
                let net_gradient = source_conc - dest_conc;
                if net_gradient > 0.0 {
                    vmax / 60.0 * source_conc / (km + source_conc)
                } else {
                    -vmax / 60.0 * dest_conc / (km + dest_conc)
                }
            }
            TransferMechanism::ActiveTransport { vmax, km, .. } => {
                // Active transport is unidirectional and saturable
                vmax / 60.0 * source_conc / (km + source_conc)
            }
            TransferMechanism::PgpEfflux { k_efflux, ic50_inhibition } => {
                // P-gp efflux (pumps drug OUT of brain)
                // Inhibited by some drugs
                let inhibition_factor = 1.0 / (1.0 + inhibitor_conc / ic50_inhibition);
                k_efflux / 60.0 * source_conc * inhibition_factor
            }
            TransferMechanism::BulkFlow { flow_rate } => {
                // Convective transport: flux = Q * C
                flow_rate / 60.0 * source_conc
            }
        }
    }
}

/// Multi-compartment model for drug distribution
#[derive(Debug, Clone)]
pub struct MultiCompartmentModel {
    /// All compartments
    pub compartments: Vec<Compartment>,
    /// Connections between compartments
    pub connections: Vec<CompartmentConnection>,
    /// Drug properties for transport calculations
    pub drug_properties: HashMap<String, DrugTransportProperties>,
    /// Current simulation time (s)
    pub time_s: f64,
}

/// Drug-specific transport properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugTransportProperties {
    /// Drug name
    pub name: String,
    /// Molecular weight (Da)
    pub mw: f64,
    /// LogP (lipophilicity)
    pub logp: f64,
    /// pKa for ionization
    pub pka: f64,
    /// Is it an acid or base?
    pub is_acid: bool,
    /// Free diffusion coefficient in water (cm²/s)
    pub d_free: f64,
    /// Is it a P-gp substrate?
    pub pgp_substrate: bool,
    /// Protein binding in plasma (fraction)
    pub plasma_protein_binding: f64,
}

impl DrugTransportProperties {
    /// Estimate diffusion coefficient from molecular weight
    /// Stokes-Einstein: D = kT / (6πηr)
    /// Empirical: D ≈ 10^(-4.15) / MW^0.46 (cm²/s)
    pub fn estimate_diffusion_coefficient(mw: f64) -> f64 {
        10.0_f64.powf(-4.15) / mw.powf(0.46)
    }

    /// Effective diffusion coefficient in a compartment
    pub fn effective_diffusion(&self, compartment: &Compartment) -> f64 {
        let tortuosity = compartment.compartment_type.tortuosity();
        self.d_free / (tortuosity * tortuosity)
    }

    /// BBB permeability estimate based on lipophilicity
    /// Uses empirical correlation from Pardridge lab
    pub fn estimate_bbb_permeability(&self) -> f64 {
        // Log PS = -0.81 + 0.43 * LogP (for passive diffusion)
        // Returns PS product in mL/(min·g brain)
        let log_ps = -0.81 + 0.43 * self.logp;
        10.0_f64.powf(log_ps)
    }
}

impl Default for DrugTransportProperties {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            mw: 300.0,
            logp: 2.0,
            pka: 7.0,
            is_acid: false,
            d_free: 5e-6,
            pgp_substrate: false,
            plasma_protein_binding: 0.5,
        }
    }
}

impl MultiCompartmentModel {
    /// Create a standard brain compartment model
    pub fn standard_brain_model() -> Self {
        let mut compartments = Vec::new();

        // Create standard compartments
        compartments.push(Compartment::new(CompartmentType::BloodCapillary));
        compartments.push(Compartment::new(CompartmentType::Interstitial));
        compartments.push(Compartment::new(CompartmentType::Extrasynaptic));
        compartments.push(Compartment::new(CompartmentType::Perisynaptic));
        compartments.push(Compartment::new(CompartmentType::SynapticCleft));
        compartments.push(Compartment::new(CompartmentType::Cytoplasm));
        compartments.push(Compartment::new(CompartmentType::Csf));

        // Define connections
        let mut connections = Vec::new();

        // Blood -> Interstitial (BBB)
        connections.push(CompartmentConnection {
            from: 0,
            to: 1,
            mechanism: TransferMechanism::PassiveDiffusion {
                ps_product: 0.1,  // Will be adjusted per drug
            },
            bidirectional: true,
        });

        // Interstitial -> Extrasynaptic
        connections.push(CompartmentConnection {
            from: 1,
            to: 2,
            mechanism: TransferMechanism::PassiveDiffusion {
                ps_product: 10.0,
            },
            bidirectional: true,
        });

        // Extrasynaptic -> Perisynaptic
        connections.push(CompartmentConnection {
            from: 2,
            to: 3,
            mechanism: TransferMechanism::PassiveDiffusion {
                ps_product: 100.0,
            },
            bidirectional: true,
        });

        // Perisynaptic -> Synaptic cleft
        connections.push(CompartmentConnection {
            from: 3,
            to: 4,
            mechanism: TransferMechanism::PassiveDiffusion {
                ps_product: 1000.0,  // Very fast
            },
            bidirectional: true,
        });

        // Extrasynaptic -> Cytoplasm (cellular uptake)
        connections.push(CompartmentConnection {
            from: 2,
            to: 5,
            mechanism: TransferMechanism::PassiveDiffusion {
                ps_product: 1.0,
            },
            bidirectional: true,
        });

        // Interstitial <-> CSF (glymphatic)
        connections.push(CompartmentConnection {
            from: 1,
            to: 6,
            mechanism: TransferMechanism::BulkFlow {
                flow_rate: 0.3,  // ~0.3 mL/min glymphatic flow
            },
            bidirectional: false,
        });

        // P-gp efflux at BBB (brain -> blood)
        connections.push(CompartmentConnection {
            from: 1,
            to: 0,
            mechanism: TransferMechanism::PgpEfflux {
                k_efflux: 0.5,
                ic50_inhibition: 10.0,
            },
            bidirectional: false,
        });

        Self {
            compartments,
            connections,
            drug_properties: HashMap::new(),
            time_s: 0.0,
        }
    }

    /// Add a drug with its transport properties
    pub fn add_drug(&mut self, props: DrugTransportProperties) {
        self.drug_properties.insert(props.name.clone(), props);
    }

    /// Inject drug into a specific compartment
    pub fn inject(&mut self, compartment_idx: usize, drug: &str, amount_umol: f64) {
        if compartment_idx < self.compartments.len() {
            self.compartments[compartment_idx].add_drug(drug, amount_umol);
        }
    }

    /// Simulate one time step
    pub fn step(&mut self, dt_s: f64) {
        // Calculate all fluxes first (to avoid order dependency)
        let mut fluxes: Vec<(usize, usize, String, f64)> = Vec::new();

        for connection in &self.connections {
            let from_idx = connection.from;
            let to_idx = connection.to;

            // Get all drugs present in source compartment
            let drugs: Vec<String> = self.compartments[from_idx]
                .concentrations
                .keys()
                .cloned()
                .collect();

            for drug in drugs {
                let source_conc = self.compartments[from_idx].free_concentration(&drug);
                let dest_conc = self.compartments[to_idx].free_concentration(&drug);

                // Check if drug is P-gp substrate for efflux calculations
                let pgp_inhibitor_conc = 0.0;  // Could be computed from other drugs

                let flux = connection.transfer_rate(source_conc, dest_conc, pgp_inhibitor_conc);

                if flux.abs() > 1e-15 {
                    fluxes.push((from_idx, to_idx, drug.clone(), flux * dt_s));

                    // Handle bidirectional connections
                    if connection.bidirectional && flux < 0.0 {
                        // Negative flux means reverse direction
                        fluxes.push((to_idx, from_idx, drug, -flux * dt_s));
                    }
                }
            }
        }

        // Apply all fluxes
        for (from_idx, to_idx, drug, amount) in fluxes {
            let source_amount = self.compartments[from_idx].amount_umol(&drug);

            // Don't transfer more than available
            let actual_amount = amount.min(source_amount).max(0.0);

            if actual_amount > 0.0 {
                // Remove from source
                let source = &mut self.compartments[from_idx];
                let delta = actual_amount / (source.volume_l * 1000.0);
                if let Some(conc) = source.concentrations.get_mut(&drug) {
                    *conc = (*conc - delta).max(0.0);
                }

                // Add to destination
                self.compartments[to_idx].add_drug(&drug, actual_amount);
            }
        }

        self.time_s += dt_s;
    }

    /// Get concentration profile at synaptic cleft
    pub fn synaptic_concentration(&self, drug: &str) -> f64 {
        self.compartments
            .iter()
            .find(|c| c.compartment_type == CompartmentType::SynapticCleft)
            .map(|c| c.free_concentration(drug))
            .unwrap_or(0.0)
    }

    /// Get all compartment concentrations for a drug
    pub fn concentration_profile(&self, drug: &str) -> HashMap<CompartmentType, f64> {
        self.compartments
            .iter()
            .map(|c| (c.compartment_type, c.free_concentration(drug)))
            .collect()
    }

    /// Simulate to steady state
    pub fn simulate_to_steady_state(&mut self, max_time_s: f64, tolerance: f64) -> bool {
        let dt = 0.01;  // 10 ms time step
        let mut last_profile: HashMap<CompartmentType, f64> = HashMap::new();

        while self.time_s < max_time_s {
            // Get current profile for first drug
            let drug = self.drug_properties.keys().next().cloned();
            if let Some(drug_name) = drug {
                let current_profile = self.concentration_profile(&drug_name);

                // Check for convergence
                if !last_profile.is_empty() {
                    let max_change: f64 = current_profile
                        .iter()
                        .filter_map(|(ct, conc)| {
                            last_profile.get(ct).map(|last| {
                                if *last > 1e-10 {
                                    (conc - last).abs() / last
                                } else {
                                    0.0
                                }
                            })
                        })
                        .fold(0.0, f64::max);

                    if max_change < tolerance {
                        return true;
                    }
                }

                last_profile = current_profile;
            }

            self.step(dt);
        }

        false  // Did not reach steady state
    }
}

/// Synaptic cleft dynamics with neurotransmitter release
#[derive(Debug, Clone)]
pub struct SynapticCleftDynamics {
    /// Cleft dimensions
    pub width_nm: f64,
    pub radius_nm: f64,
    /// Current neurotransmitter concentration (mM)
    pub neurotransmitter_mm: f64,
    /// Reuptake rate constant (1/ms)
    pub reuptake_rate: f64,
    /// Diffusion out of cleft rate (1/ms)
    pub diffusion_rate: f64,
    /// Drug modulation of reuptake (fraction)
    pub reuptake_inhibition: f64,
}

impl SynapticCleftDynamics {
    pub fn new_glutamatergic() -> Self {
        Self {
            width_nm: 20.0,
            radius_nm: 150.0,
            neurotransmitter_mm: 0.0,
            reuptake_rate: 0.7,      // τ ≈ 1.4 ms
            diffusion_rate: 0.5,      // τ ≈ 2 ms
            reuptake_inhibition: 0.0,
        }
    }

    pub fn new_gabaergic() -> Self {
        Self {
            width_nm: 20.0,
            radius_nm: 200.0,
            neurotransmitter_mm: 0.0,
            reuptake_rate: 0.3,      // GABA reuptake is slower
            diffusion_rate: 0.4,
            reuptake_inhibition: 0.0,
        }
    }

    /// Simulate vesicle release (single quantum)
    /// Glutamate: ~4000-5000 molecules per vesicle
    /// GABA: ~2000-3000 molecules per vesicle
    pub fn release_vesicle(&mut self, n_molecules: u32) {
        // Cleft volume = π * r² * width
        let volume_nm3 = std::f64::consts::PI
            * self.radius_nm.powi(2)
            * self.width_nm;
        let volume_l = volume_nm3 * 1e-24;  // nm³ to L

        // Convert molecules to moles
        let n_mol = n_molecules as f64 / 6.022e23;

        // Concentration in M
        let delta_m = n_mol / volume_l;

        // Add to current concentration (in mM)
        self.neurotransmitter_mm += delta_m * 1000.0;
    }

    /// Update cleft dynamics for one time step
    pub fn step(&mut self, dt_ms: f64) {
        // Effective reuptake considering drug inhibition
        let effective_reuptake = self.reuptake_rate * (1.0 - self.reuptake_inhibition);

        // Combined clearance rate
        let k_total = effective_reuptake + self.diffusion_rate;

        // Exponential decay
        self.neurotransmitter_mm *= (-k_total * dt_ms).exp();
    }

    /// Apply SSRI-like reuptake inhibition
    pub fn apply_reuptake_inhibitor(&mut self, inhibition_fraction: f64) {
        self.reuptake_inhibition = inhibition_fraction.clamp(0.0, 0.99);
    }

    /// Get spillover concentration to perisynaptic region
    pub fn spillover_concentration(&self) -> f64 {
        // Spillover is proportional to diffusion rate and current concentration
        self.neurotransmitter_mm * self.diffusion_rate * 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compartment_hierarchy() {
        // Verify time constants are in correct order
        assert!(CompartmentType::SynapticCleft.tau_s() < CompartmentType::Perisynaptic.tau_s());
        assert!(CompartmentType::Perisynaptic.tau_s() < CompartmentType::Extrasynaptic.tau_s());
        assert!(CompartmentType::Extrasynaptic.tau_s() < CompartmentType::Interstitial.tau_s());
        assert!(CompartmentType::Interstitial.tau_s() < CompartmentType::Csf.tau_s());
    }

    #[test]
    fn test_ionization() {
        let mut compartment = Compartment::new(CompartmentType::BloodArterial);
        compartment.ph = 7.4;

        // Diazepam pKa ≈ 3.3 (weak acid)
        let ionized = compartment.ionized_fraction(3.3, true);
        assert!(ionized > 0.999);  // Almost fully ionized at pH 7.4

        // Morphine pKa ≈ 8.0 (weak base)
        let ionized = compartment.ionized_fraction(8.0, false);
        assert!(ionized > 0.7);
        assert!(ionized < 0.9);
    }

    #[test]
    fn test_multicompartment_model() {
        let mut model = MultiCompartmentModel::standard_brain_model();

        // Add a test drug
        model.add_drug(DrugTransportProperties {
            name: "test_drug".to_string(),
            mw: 300.0,
            logp: 2.5,
            pka: 7.0,
            is_acid: false,
            d_free: 5e-6,
            pgp_substrate: false,
            plasma_protein_binding: 0.5,
        });

        // Inject into blood compartment
        model.inject(0, "test_drug", 1.0);  // 1 µmol

        // Run simulation
        for _ in 0..1000 {
            model.step(0.01);
        }

        // Drug should have distributed to other compartments
        let synaptic = model.synaptic_concentration("test_drug");
        assert!(synaptic > 0.0);
    }

    #[test]
    fn test_synaptic_release() {
        let mut cleft = SynapticCleftDynamics::new_glutamatergic();

        // Release glutamate vesicle
        cleft.release_vesicle(4000);

        // Peak should be in mM range
        assert!(cleft.neurotransmitter_mm > 0.1);
        assert!(cleft.neurotransmitter_mm < 10.0);

        // Decay over time
        for _ in 0..10 {
            cleft.step(0.1);
        }

        assert!(cleft.neurotransmitter_mm < 0.01);
    }
}
