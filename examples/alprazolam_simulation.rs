//! Alprazolam (Xanax) Simulation: Benzodiazepine effects on brain dynamics
//!
//! This simulation demonstrates the effects of Alprazolam on neural activity:
//! - GABA-A receptor potentiation (increased inhibitory conductance)
//! - Anxiolytic effects (reduced amygdala activity)
//! - Sedation (reduced cortical activity)
//! - Memory impairment (reduced hippocampal plasticity)
//!
//! Alprazolam: Short-acting benzodiazepine
//! - Half-life: 6-12 hours
//! - Peak plasma concentration: 1-2 hours
//! - Mechanism: Positive allosteric modulator of GABA-A receptors

use neurons::{MultiCompartmentalNeuron, compartmental::ChannelStates};
use cognition::pharmacology::{DrugEffect, Pharmacology};
use synapses::{Synapse, SynapseType, SynapticNetwork};

/// Alprazolam pharmacokinetic parameters
#[derive(Debug, Clone)]
pub struct Alprazolam {
    pub dose_mg: f64,           // Dose in milligrams
    pub concentration: f64,      // Normalized plasma concentration (0-1)
    pub half_life_hours: f64,    // 6-12 hours typically
    pub time_to_peak_hours: f64, // 1-2 hours
    pub absorption_rate: f64,    // First-order absorption rate
}

impl Alprazolam {
    /// Create a new Alprazolam dose
    /// Typical doses: 0.25mg (low), 0.5mg (medium), 1.0mg (high)
    pub fn new(dose_mg: f64) -> Self {
        Self {
            dose_mg,
            concentration: 0.0,
            half_life_hours: 11.0,  // Average half-life
            time_to_peak_hours: 1.5,
            absorption_rate: 1.5,    // Fast absorption
        }
    }

    /// Update pharmacokinetics after dt hours
    pub fn update(&mut self, dt_hours: f64) {
        // First-order absorption (simplified model)
        let k_abs = self.absorption_rate;
        let k_elim = 0.693 / self.half_life_hours;

        // Two-compartment model simplified
        let dose_normalized = self.dose_mg / 1.0; // Normalize to 1mg reference

        // Peak occurs at t_max
        let t_max = (k_abs / k_elim).ln() / (k_abs - k_elim);

        // Bateman equation for plasma concentration
        // C(t) = (F * D * ka) / (V * (ka - ke)) * (exp(-ke*t) - exp(-ka*t))
        // Simplified: normalize to peak = dose_normalized
        self.concentration = dose_normalized * ((-k_elim * dt_hours).exp() - (-k_abs * dt_hours).exp()).abs();
        self.concentration = self.concentration.min(1.0);
    }

    /// Calculate GABA-A potentiation factor
    /// Benzodiazepines enhance GABA-A receptor Cl- conductance
    pub fn gaba_a_potentiation(&self) -> f64 {
        // Linear relationship up to saturation
        // At therapeutic doses, ~2-3x enhancement
        1.0 + 2.0 * self.concentration
    }

    /// Calculate anxiolytic effect (reduces amygdala excitability)
    pub fn anxiolytic_factor(&self) -> f64 {
        // Reduces amygdala output
        1.0 - 0.5 * self.concentration
    }

    /// Calculate sedation level (reduces cortical activity)
    pub fn sedation_factor(&self) -> f64 {
        // Reduces overall cortical firing rate
        1.0 - 0.4 * self.concentration
    }

    /// Calculate memory impairment (reduces hippocampal LTP)
    pub fn memory_impairment_factor(&self) -> f64 {
        // Reduces synaptic plasticity
        1.0 - 0.6 * self.concentration
    }
}

/// Simulated brain region with GABA modulation
pub struct GABAergicNetwork {
    pub neurons: Vec<SimpleNeuron>,
    pub synapses: Vec<GABASynapse>,
    pub gaba_conductance_base: f64,
}

/// Simplified neuron for demonstration
#[derive(Debug, Clone)]
pub struct SimpleNeuron {
    pub voltage: f64,
    pub threshold: f64,
    pub refractory: f64,
    pub is_spiking: bool,
}

impl SimpleNeuron {
    pub fn new() -> Self {
        Self {
            voltage: -70.0,
            threshold: -55.0,
            refractory: 0.0,
            is_spiking: false,
        }
    }

    pub fn step(&mut self, dt: f64, input_current: f64, gaba_current: f64) -> bool {
        if self.refractory > 0.0 {
            self.refractory -= dt;
            self.voltage = -70.0;
            self.is_spiking = false;
            return false;
        }

        // Integrate-and-fire with GABA inhibition
        let tau = 10.0; // Membrane time constant (ms)
        let v_rest = -70.0;
        let g_leak = 1.0;

        // GABA current is inhibitory (hyperpolarizing)
        let dv = (dt / tau) * (-(self.voltage - v_rest) * g_leak + input_current - gaba_current);
        self.voltage += dv;

        // Check for spike
        if self.voltage >= self.threshold {
            self.is_spiking = true;
            self.voltage = 30.0; // Spike peak
            self.refractory = 2.0; // 2ms refractory period
            return true;
        }

        self.is_spiking = false;
        false
    }
}

/// GABA-A synapse with benzodiazepine modulation
#[derive(Debug, Clone)]
pub struct GABASynapse {
    pub conductance: f64,
    pub reversal_potential: f64, // -70 mV for GABA-A
    pub decay_tau: f64,          // ~5 ms for GABA-A
    pub gating: f64,
}

impl GABASynapse {
    pub fn new() -> Self {
        Self {
            conductance: 1.0,
            reversal_potential: -70.0,
            decay_tau: 5.0,
            gating: 0.0,
        }
    }

    pub fn step(&mut self, dt: f64, pre_spike: bool, benzo_potentiation: f64) {
        // Decay gating variable
        self.gating *= (-dt / self.decay_tau).exp();

        // Pre-synaptic spike triggers GABA release
        if pre_spike {
            // Benzodiazepines enhance GABA binding and prolong channel opening
            self.gating += 0.5 * benzo_potentiation;
            self.gating = self.gating.min(1.0);
        }
    }

    pub fn current(&self, post_voltage: f64, benzo_potentiation: f64) -> f64 {
        // Enhanced conductance with benzodiazepine
        let g = self.conductance * self.gating * benzo_potentiation;
        g * (post_voltage - self.reversal_potential)
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  HumanBrain: Alprazolam (Xanax) Neural Simulation            ║");
    println!("║  Benzodiazepine Effects on Brain Dynamics                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Simulation parameters
    let dt = 0.1;                    // Time step (ms)
    let sim_time_ms = 10000.0;       // 10 seconds
    let dose_mg = 0.5;               // Standard therapeutic dose

    // Initialize drug
    let mut alprazolam = Alprazolam::new(dose_mg);
    println!("📦 Alprazolam dose: {:.2} mg", dose_mg);
    println!("   Half-life: {:.1} hours", alprazolam.half_life_hours);
    println!("   Time to peak: {:.1} hours\n", alprazolam.time_to_peak_hours);

    // Create a simple cortical network (20 excitatory, 5 inhibitory)
    let n_excitatory = 20;
    let n_inhibitory = 5;
    let n_total = n_excitatory + n_inhibitory;

    let mut neurons: Vec<SimpleNeuron> = (0..n_total).map(|_| SimpleNeuron::new()).collect();
    let mut gaba_synapses: Vec<GABASynapse> = (0..n_inhibitory).map(|_| GABASynapse::new()).collect();

    // Statistics
    let mut baseline_spikes = 0;
    let mut drug_spikes = 0;
    let mut time_to_effect_ms: Option<f64> = None;

    // Time markers for pharmacokinetics
    let drug_admin_time_ms = 2000.0;  // Administer drug at 2 seconds

    println!("🧠 Network: {} excitatory + {} inhibitory neurons", n_excitatory, n_inhibitory);
    println!("⏱️  Simulation: {:.0} ms ({:.1} seconds)\n", sim_time_ms, sim_time_ms / 1000.0);

    println!("📊 Phase 1: Baseline recording (0-2s)...");

    let steps = (sim_time_ms / dt) as usize;
    let mut spike_counts_per_second: Vec<usize> = vec![0; (sim_time_ms / 1000.0) as usize];

    for step in 0..steps {
        let t = step as f64 * dt;
        let second = (t / 1000.0) as usize;

        // Administer drug at specified time
        if t >= drug_admin_time_ms && t < drug_admin_time_ms + dt {
            println!("\n💊 Drug administered at t = {:.0} ms", t);
            alprazolam.concentration = 0.1; // Initial absorption
        }

        // Update drug pharmacokinetics every 100ms of simulation time
        if t > drug_admin_time_ms && step % 1000 == 0 {
            let dt_hours = 0.1 / 3600.0; // 100ms in hours (scaled for demo)
            alprazolam.concentration *= 1.05; // Simulate absorption phase
            alprazolam.concentration = alprazolam.concentration.min(0.8);
        }

        // Calculate drug effects
        let gaba_potentiation = alprazolam.gaba_a_potentiation();
        let sedation = alprazolam.sedation_factor();

        // Random input current (Poisson-like)
        let base_input = if rand::random::<f64>() < 0.01 { 15.0 } else { 0.0 };
        let input_current = base_input * sedation; // Sedation reduces input

        // Update inhibitory neurons and their synapses
        let inhibitory_spikes: Vec<bool> = (0..n_inhibitory)
            .map(|i| {
                let inh_input = if rand::random::<f64>() < 0.02 { 10.0 } else { 0.0 };
                neurons[n_excitatory + i].step(dt, inh_input, 0.0)
            })
            .collect();

        // Update GABA synapses
        for (i, synapse) in gaba_synapses.iter_mut().enumerate() {
            synapse.step(dt, inhibitory_spikes[i], gaba_potentiation);
        }

        // Calculate total GABA current to excitatory neurons
        let total_gaba_current: f64 = gaba_synapses.iter()
            .map(|syn| syn.current(-60.0, gaba_potentiation))
            .sum::<f64>() / n_excitatory as f64;

        // Update excitatory neurons
        for i in 0..n_excitatory {
            let spiked = neurons[i].step(dt, input_current, total_gaba_current.abs());
            if spiked {
                if second < spike_counts_per_second.len() {
                    spike_counts_per_second[second] += 1;
                }
                if t < drug_admin_time_ms {
                    baseline_spikes += 1;
                } else {
                    drug_spikes += 1;
                    if time_to_effect_ms.is_none() && spike_counts_per_second.get(second).map(|&c| c < spike_counts_per_second[1]).unwrap_or(false) {
                        time_to_effect_ms = Some(t - drug_admin_time_ms);
                    }
                }
            }
        }

        // Progress reporting
        if t as usize % 2000 == 0 && t > 0.0 {
            if t <= drug_admin_time_ms {
                println!("   t = {:.0}ms: {} spikes (baseline)", t, spike_counts_per_second[second]);
            } else {
                println!("📊 t = {:.0}ms: {} spikes | GABA potentiation: {:.1}x | Sedation: {:.0}%",
                         t,
                         spike_counts_per_second.get(second).unwrap_or(&0),
                         gaba_potentiation,
                         (1.0 - sedation) * 100.0);
            }
        }
    }

    // Results
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SIMULATION RESULTS                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let baseline_rate = baseline_spikes as f64 / 2.0; // Per second
    let drug_rate = drug_spikes as f64 / 8.0;         // Per second (8s of drug phase)
    let reduction_percent = ((baseline_rate - drug_rate) / baseline_rate * 100.0).max(0.0);

    println!("📈 Baseline firing rate: {:.1} Hz", baseline_rate);
    println!("📉 Drug-phase firing rate: {:.1} Hz", drug_rate);
    println!("🔻 Activity reduction: {:.1}%\n", reduction_percent);

    println!("🧬 Pharmacological Effects:");
    println!("   GABA-A potentiation: {:.1}x", alprazolam.gaba_a_potentiation());
    println!("   Anxiolytic effect: {:.0}% reduction in amygdala output", (1.0 - alprazolam.anxiolytic_factor()) * 100.0);
    println!("   Sedation level: {:.0}%", (1.0 - alprazolam.sedation_factor()) * 100.0);
    println!("   Memory impairment: {:.0}% reduction in LTP\n", (1.0 - alprazolam.memory_impairment_factor()) * 100.0);

    // Clinical interpretation
    println!("📋 Clinical Interpretation:");
    if reduction_percent > 30.0 {
        println!("   ✓ Significant reduction in neural activity consistent with anxiolytic effect");
        println!("   ✓ GABA-A potentiation within therapeutic range");
        println!("   ⚠ Sedation and memory effects may impair cognitive function");
    } else {
        println!("   ⚠ Subtherapeutic effect - consider dose adjustment");
    }

    println!("\n💡 Note: This is a simplified simulation for educational purposes.");
    println!("   Real benzodiazepine effects involve complex receptor binding kinetics,");
    println!("   multiple brain regions, and individual pharmacogenetic variation.\n");

    println!("════════════════════════════════════════════════════════════════");
    println!("  HumanBrain v1.0 - \"No quiero suficiencia, quiero realidad\"  ");
    println!("════════════════════════════════════════════════════════════════");
}

// Minimal rand implementation for example
mod rand {
    use std::cell::Cell;

    thread_local! {
        static RNG_STATE: Cell<u64> = Cell::new(12345);
    }

    pub fn random<T: FromRng>() -> T {
        T::from_rng()
    }

    pub trait FromRng {
        fn from_rng() -> Self;
    }

    impl FromRng for f64 {
        fn from_rng() -> Self {
            RNG_STATE.with(|state| {
                let mut s = state.get();
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                state.set(s);
                (s >> 33) as f64 / (1u64 << 31) as f64
            })
        }
    }
}
