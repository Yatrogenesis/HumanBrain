//! Multi-compartmental neuron implementation.
//!
//! This module implements neurons as collections of interconnected compartments,
//! each solving the cable equation to capture spatial voltage dynamics.

use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use crate::{Result, NeuronError, constants::*};
use crate::channels::IonChannel;

/// Type of neural compartment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompartmentType {
    Soma,
    Dendrite,
    ApicalDendrite,
    BasalDendrite,
    Axon,
    AxonInitialSegment,
}

/// A single compartment in a multi-compartmental neuron.
///
/// Each compartment has:
/// - Membrane voltage
/// - Ion channels
/// - Geometric properties (length, diameter)
/// - Connections to other compartments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compartment {
    /// Compartment type
    pub compartment_type: CompartmentType,

    /// Membrane voltage (mV)
    pub voltage: f64,

    /// Length of compartment (um)
    pub length: f64,

    /// Diameter of compartment (um)
    pub diameter: f64,

    /// Surface area (um^2)
    pub surface_area: f64,

    /// Axial resistance to parent (MOhm)
    pub axial_resistance: f64,

    /// Membrane capacitance (pF)
    pub capacitance: f64,

    /// Leak conductance (nS)
    pub g_leak: f64,

    /// Leak reversal potential (mV)
    pub e_leak: f64,

    /// Calcium concentration (mM)
    pub ca_concentration: f64,

    /// Index of parent compartment (None for soma)
    pub parent_idx: Option<usize>,

    /// Indices of child compartments
    pub children_idx: Vec<usize>,

    /// Ion channel densities (channels per um^2)
    pub channel_densities: Vec<(String, f64)>,
}

impl Compartment {
    /// Create a new compartment
    pub fn new(
        compartment_type: CompartmentType,
        length: f64,
        diameter: f64,
    ) -> Self {
        let surface_area = std::f64::consts::PI * diameter * length;
        let capacitance = C_M * surface_area / 100.0; // Convert to pF

        // Axial resistance: R = (R_A * length) / (pi * (diameter/2)^2)
        let axial_resistance = (R_A * length) /
            (std::f64::consts::PI * (diameter / 2.0).powi(2));

        Self {
            compartment_type,
            voltage: -70.0, // Resting potential
            length,
            diameter,
            surface_area,
            axial_resistance,
            capacitance,
            g_leak: 0.025, // nS - typical leak conductance
            e_leak: -70.0,
            ca_concentration: 0.0001, // Resting Ca2+ concentration (mM)
            parent_idx: None,
            children_idx: Vec::new(),
            channel_densities: Vec::new(),
        }
    }

    /// Add an ion channel with specified density
    pub fn add_channel(&mut self, channel_name: String, density: f64) {
        self.channel_densities.push((channel_name, density));
    }

    /// Calculate total channel conductance for a specific channel type
    pub fn get_channel_conductance(&self, channel_name: &str) -> f64 {
        self.channel_densities
            .iter()
            .find(|(name, _)| name == channel_name)
            .map(|(_, density)| density * self.surface_area)
            .unwrap_or(0.0)
    }
}

/// Multi-compartmental neuron model.
///
/// Implements the cable equation across all compartments:
/// C_m * dV/dt = -I_ion - I_axial + I_ext
///
/// where I_axial couples adjacent compartments.
#[derive(Debug, Clone)]
pub struct MultiCompartmentalNeuron {
    /// Neuron ID
    pub id: usize,

    /// All compartments
    pub compartments: Vec<Compartment>,

    /// Current injection for each compartment (pA)
    pub external_current: Array1<f64>,

    /// Synaptic currents for each compartment (pA)
    pub synaptic_current: Array1<f64>,

    /// Time step (ms)
    pub dt: f64,

    /// Spike threshold (mV)
    pub spike_threshold: f64,

    /// Last spike time (ms)
    pub last_spike_time: f64,

    /// Is neuron currently spiking?
    pub is_spiking: bool,
}

impl MultiCompartmentalNeuron {
    /// Create a new multi-compartmental neuron
    pub fn new(id: usize, num_compartments: usize, dt: f64) -> Self {
        let mut compartments = Vec::with_capacity(num_compartments);

        // Create soma
        compartments.push(Compartment::new(CompartmentType::Soma, 20.0, 20.0));

        // Create dendritic compartments
        for _ in 1..num_compartments {
            compartments.push(Compartment::new(
                CompartmentType::Dendrite,
                50.0,  // 50 um length
                2.0,   // 2 um diameter
            ));
        }

        Self {
            id,
            compartments,
            external_current: Array1::zeros(num_compartments),
            synaptic_current: Array1::zeros(num_compartments),
            dt,
            spike_threshold: -40.0,
            last_spike_time: -1000.0,
            is_spiking: false,
        }
    }

    /// Create a pyramidal neuron with realistic morphology
    pub fn new_pyramidal(id: usize, dt: f64) -> Self {
        let mut neuron = Self::new(id, 1, dt);
        neuron.compartments.clear();

        // Soma
        let soma = Compartment::new(CompartmentType::Soma, 25.0, 25.0);
        neuron.compartments.push(soma);

        // Apical dendrite (100 compartments)
        for i in 0..100 {
            let mut comp = Compartment::new(
                CompartmentType::ApicalDendrite,
                10.0,
                2.0 - (i as f64 * 0.015), // Tapering
            );
            comp.parent_idx = Some(i);
            if i > 0 {
                neuron.compartments[i].children_idx.push(i + 1);
            } else {
                neuron.compartments[0].children_idx.push(1);
            }
            neuron.compartments.push(comp);
        }

        // Basal dendrites (50 compartments)
        for i in 0..50 {
            let mut comp = Compartment::new(
                CompartmentType::BasalDendrite,
                8.0,
                1.5,
            );
            comp.parent_idx = Some(0); // All connect to soma
            neuron.compartments[0].children_idx.push(101 + i);
            neuron.compartments.push(comp);
        }

        // Axon initial segment
        let mut ais = Compartment::new(CompartmentType::AxonInitialSegment, 30.0, 1.0);
        ais.parent_idx = Some(0);
        neuron.compartments[0].children_idx.push(151);
        neuron.compartments.push(ais);

        // Resize current arrays
        let n = neuron.compartments.len();
        neuron.external_current = Array1::zeros(n);
        neuron.synaptic_current = Array1::zeros(n);

        neuron
    }

    /// Step the neuron simulation forward by dt
    pub fn step(&mut self, channel_states: &mut Vec<ChannelStates>) {
        let n = self.compartments.len();
        let mut dv = Array1::zeros(n);

        // Calculate voltage changes for each compartment
        for i in 0..n {
            let comp = &self.compartments[i];

            // Leak current
            let i_leak = comp.g_leak * (comp.voltage - comp.e_leak);

            // Ion channel currents
            let i_ion = self.calculate_ion_currents(i, &channel_states[i]);

            // Axial current from parent
            let i_axial_parent = if let Some(parent_idx) = comp.parent_idx {
                let parent = &self.compartments[parent_idx];
                (parent.voltage - comp.voltage) / comp.axial_resistance
            } else {
                0.0
            };

            // Axial currents from children
            let i_axial_children: f64 = comp.children_idx.iter()
                .map(|&child_idx| {
                    let child = &self.compartments[child_idx];
                    (child.voltage - comp.voltage) / child.axial_resistance
                })
                .sum();

            // External and synaptic currents
            let i_ext = self.external_current[i] + self.synaptic_current[i];

            // Cable equation: C * dV/dt = -I_leak - I_ion + I_axial + I_ext
            dv[i] = (-i_leak - i_ion + i_axial_parent + i_axial_children + i_ext)
                / comp.capacitance;
        }

        // Update voltages
        for i in 0..n {
            self.compartments[i].voltage += dv[i] * self.dt;
        }

        // Update channel states
        for i in 0..n {
            self.update_channel_states(i, &mut channel_states[i]);
        }

        // Detect spikes at soma
        self.is_spiking = self.compartments[0].voltage > self.spike_threshold;
    }

    /// Calculate ion channel currents for a compartment
    fn calculate_ion_currents(&self, comp_idx: usize, states: &ChannelStates) -> f64 {
        let comp = &self.compartments[comp_idx];
        let v = comp.voltage;

        let mut total_current = 0.0;

        // Hodgkin-Huxley Na+ current
        let g_na_bar = comp.get_channel_conductance("Na");
        if g_na_bar > 0.0 {
            let e_na = 50.0; // mV
            let g_na = g_na_bar * states.na_m.powi(3) * states.na_h;
            total_current += g_na * (v - e_na);
        }

        // Hodgkin-Huxley K+ current
        let g_k_bar = comp.get_channel_conductance("K");
        if g_k_bar > 0.0 {
            let e_k = -90.0; // mV
            let g_k = g_k_bar * states.k_n.powi(4);
            total_current += g_k * (v - e_k);
        }

        // Calcium current
        let g_ca_bar = comp.get_channel_conductance("Ca");
        if g_ca_bar > 0.0 {
            let e_ca = 120.0; // mV
            let g_ca = g_ca_bar * states.ca_m.powi(2) * states.ca_h;
            total_current += g_ca * (v - e_ca);
        }

        total_current
    }

    /// Update ion channel gating variables
    fn update_channel_states(&self, comp_idx: usize, states: &mut ChannelStates) {
        let v = self.compartments[comp_idx].voltage;

        // Sodium channel (Hodgkin-Huxley)
        let alpha_m = 0.1 * (v + 40.0) / (1.0 - (-0.1 * (v + 40.0)).exp());
        let beta_m = 4.0 * ((-0.0556 * (v + 65.0)).exp());
        let alpha_h = 0.07 * ((-0.05 * (v + 65.0)).exp());
        let beta_h = 1.0 / (1.0 + ((-0.1 * (v + 35.0)).exp()));

        states.na_m += (alpha_m * (1.0 - states.na_m) - beta_m * states.na_m) * self.dt;
        states.na_h += (alpha_h * (1.0 - states.na_h) - beta_h * states.na_h) * self.dt;

        // Potassium channel
        let alpha_n = 0.01 * (v + 55.0) / (1.0 - (-0.1 * (v + 55.0)).exp());
        let beta_n = 0.125 * ((-0.0125 * (v + 65.0)).exp());

        states.k_n += (alpha_n * (1.0 - states.k_n) - beta_n * states.k_n) * self.dt;

        // Calcium channel (simplified)
        let ca_m_inf = 1.0 / (1.0 + ((-0.15 * (v + 20.0)).exp()));
        let ca_h_inf = 1.0 / (1.0 + ((0.2 * (v + 50.0)).exp()));
        let tau_ca_m = 0.5;
        let tau_ca_h = 20.0;

        states.ca_m += ((ca_m_inf - states.ca_m) / tau_ca_m) * self.dt;
        states.ca_h += ((ca_h_inf - states.ca_h) / tau_ca_h) * self.dt;
    }

    /// Get soma voltage
    pub fn get_soma_voltage(&self) -> f64 {
        self.compartments[0].voltage
    }

    /// Inject current into a compartment
    pub fn inject_current(&mut self, comp_idx: usize, current: f64) {
        if comp_idx < self.external_current.len() {
            self.external_current[comp_idx] = current;
        }
    }
}

/// Ion channel gating variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStates {
    pub na_m: f64,  // Sodium activation
    pub na_h: f64,  // Sodium inactivation
    pub k_n: f64,   // Potassium activation
    pub ca_m: f64,  // Calcium activation
    pub ca_h: f64,  // Calcium inactivation
}

impl Default for ChannelStates {
    fn default() -> Self {
        Self {
            na_m: 0.05,
            na_h: 0.6,
            k_n: 0.32,
            ca_m: 0.01,
            ca_h: 0.9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compartment_creation() {
        let comp = Compartment::new(CompartmentType::Soma, 20.0, 20.0);
        assert_eq!(comp.compartment_type, CompartmentType::Soma);
        assert_eq!(comp.length, 20.0);
        assert_eq!(comp.diameter, 20.0);
        assert!(comp.surface_area > 0.0);
    }

    #[test]
    fn test_neuron_initialization() {
        let neuron = MultiCompartmentalNeuron::new(0, 10, 0.01);
        assert_eq!(neuron.compartments.len(), 10);
        assert_eq!(neuron.compartments[0].compartment_type, CompartmentType::Soma);
    }

    #[test]
    fn test_pyramidal_neuron() {
        let neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
        assert!(neuron.compartments.len() > 100);

        // Check that soma has children
        assert!(!neuron.compartments[0].children_idx.is_empty());
    }
}
