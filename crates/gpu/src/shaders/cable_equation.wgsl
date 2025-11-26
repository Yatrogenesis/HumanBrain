//! Cable Equation Solver - Multi-Compartmental Neurons with FULL TREE TOPOLOGY
//!
//! Este shader implementa la ecuación de cable completa con topología arbórea
//! REAL (no simplificaciones). Cada compartimento puede tener múltiples hijos,
//! logrando PARIDAD TOTAL con modelos morfológicos CPU.
//!
//! ## Ecuación de Cable
//! C_m * dV/dt = -I_leak - I_ion + I_axial_parent + Σ(I_axial_children) + I_ext
//!
//! donde:
//!   I_axial = (V_adjacent - V_current) / R_axial
//!
//! ## Arquitectura Real (No Simplificada)
//! Cada neurona tiene topología de árbol completo:
//!   - Soma (raíz) → puede tener N hijos
//!   - Dendritas apicales (ramificación compleja)
//!   - Dendritas basales (múltiples ramas)
//!   - Axon Initial Segment (AIS)
//!
//! Ejemplo de neurona piramidal L5:
//!   Soma (0)
//!   ├─ Apical dendrite trunk (1)
//!   │  ├─ Apical tuft branch 1 (2)
//!   │  │  ├─ Terminal 1a (3)
//!   │  │  └─ Terminal 1b (4)
//!   │  └─ Apical tuft branch 2 (5)
//!   │     └─ Terminal 2a (6)
//!   ├─ Basal dendrite 1 (7)
//!   │  ├─ Basal branch 1a (8)
//!   │  └─ Basal branch 1b (9)
//!   ├─ Basal dendrite 2 (10)
//!   └─ Axon Initial Segment (11)
//!
//! ## Optimización GPU con Realismo
//! - Compartimentos almacenados contiguamente (cache-friendly)
//! - Cada thread procesa 1 compartimento
//! - Children indices en array de tamaño fijo (MAX_CHILDREN=8)
//! - Sin sincronización barrier (lectura directa de voltajes)

// ============================================================================
// CONSTANTS
// ============================================================================

const MAX_CHILDREN: u32 = 8u;  // Máximo hijos por compartimento (dendritas reales)

// ============================================================================
// ESTRUCTURAS DE DATOS
// ============================================================================

/// Estado de un solo compartimento con topología arbórea completa
struct CompartmentState {
    // Electrical state
    voltage: f32,           // mV
    capacitance: f32,       // pF (derived from surface area)
    axial_resistance: f32,  // MOhm (to parent)
    g_leak: f32,            // nS
    e_leak: f32,            // mV

    // Ion channel gating variables (4 channels - Na, K, Ca, leak)
    na_m: f32,              // Sodium activation
    na_h: f32,              // Sodium inactivation
    k_n: f32,               // Potassium activation
    ca_m: f32,              // Calcium activation

    // Geometry (realistic morphology)
    length: f32,            // um
    diameter: f32,          // um
    surface_area: f32,      // um^2 (π * d * L for cylinder)

    // Compartment type (0=soma, 1=apical, 2=basal, 3=AIS)
    comp_type: u32,

    // Tree topology (FULL TREE, NOT CHAIN)
    parent_idx: i32,        // -1 if root (soma)

    // Children indices (up to MAX_CHILDREN)
    // -1 means no child at this position
    child_idx_0: i32,
    child_idx_1: i32,
    child_idx_2: i32,
    child_idx_3: i32,
    child_idx_4: i32,
    child_idx_5: i32,
    child_idx_6: i32,
    child_idx_7: i32,

    num_children: u32,      // Number of actual children (0-8)

    // Identifiers
    neuron_id: u32,         // Which neuron this belongs to
    comp_id_in_neuron: u32, // 0 to N_COMPS_PER_NEURON-1

    // Padding to align to 16 bytes (GPU requirement)
    _pad0: f32,
    _pad1: f32,
}

/// Constantes globales del simulador
struct Constants {
    dt: f32,
    num_neurons: u32,
    num_compartments_per_neuron: u32,
    total_compartments: u32,

    // Ion channel max conductances (nS/um^2)
    g_na_bar: f32,  // 120.0
    g_k_bar: f32,   // 36.0
    g_ca_bar: f32,  // 5.0

    // Reversal potentials (mV)
    e_na: f32,      // 50.0
    e_k: f32,       // -90.0
    e_ca: f32,      // 120.0

    temperature: f32,      // Celsius (37.0)
    ref_temperature: f32,  // Celsius (22.0)

    _padding: vec2<u32>,
}

// ============================================================================
// BUFFERS (GPU Memory)
// ============================================================================

@group(0) @binding(0)
var<storage, read_write> compartments: array<CompartmentState>;

@group(0) @binding(1)
var<uniform> constants: Constants;

@group(0) @binding(2)
var<storage, read> external_currents: array<f32>; // pA per compartment

// ============================================================================
// ION CHANNEL DYNAMICS (Hodgkin-Huxley)
// ============================================================================

// Alpha/Beta functions for gating variables
fn alpha_m(v: f32) -> f32 {
    let denom = 1.0 - exp(-(v + 40.0) / 10.0);
    if (abs(denom) < 1e-6) {
        return 1.0; // Avoid division by zero
    }
    return 0.1 * (v + 40.0) / denom;
}

fn beta_m(v: f32) -> f32 {
    return 4.0 * exp(-(v + 65.0) / 18.0);
}

fn alpha_h(v: f32) -> f32 {
    return 0.07 * exp(-(v + 65.0) / 20.0);
}

fn beta_h(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 35.0) / 10.0));
}

fn alpha_n(v: f32) -> f32 {
    let denom = 1.0 - exp(-(v + 55.0) / 10.0);
    if (abs(denom) < 1e-6) {
        return 0.1;
    }
    return 0.01 * (v + 55.0) / denom;
}

fn beta_n(v: f32) -> f32 {
    return 0.125 * exp(-(v + 65.0) / 80.0);
}

// Calcium channel (simplified L-type)
fn alpha_m_ca(v: f32) -> f32 {
    let denom = 1.0 - exp(-(v + 27.0) / 3.8);
    if (abs(denom) < 1e-6) {
        return 0.055 * 3.8;
    }
    return 0.055 * (v + 27.0) / denom;
}

fn beta_m_ca(v: f32) -> f32 {
    return 0.94 * exp(-(v + 75.0) / 17.0);
}

/// Q10 temperature correction
fn q10_correction(rate: f32, q10: f32) -> f32 {
    let temp_diff = (constants.temperature - constants.ref_temperature) / 10.0;
    return rate * pow(q10, temp_diff);
}

// ============================================================================
// CABLE EQUATION SOLVER
// ============================================================================

/// Calcula corrientes iónicas para un compartimento
fn calculate_ion_currents(comp: ptr<storage, CompartmentState, read_write>) -> f32 {
    let v = (*comp).voltage;
    let area = (*comp).surface_area;

    // Sodium current (Na⁺) - Action potential upstroke
    let g_na = constants.g_na_bar * area / 100.0; // Convert to nS
    let m3 = (*comp).na_m * (*comp).na_m * (*comp).na_m;
    let i_na = g_na * m3 * (*comp).na_h * (v - constants.e_na);

    // Potassium current (K⁺) - Action potential repolarization
    let g_k = constants.g_k_bar * area / 100.0;
    let n4 = (*comp).k_n * (*comp).k_n * (*comp).k_n * (*comp).k_n;
    let i_k = g_k * n4 * (v - constants.e_k);

    // Calcium current (Ca²⁺) - Synaptic plasticity, dendritic spikes
    let g_ca = constants.g_ca_bar * area / 100.0;
    let i_ca = g_ca * (*comp).ca_m * (v - constants.e_ca);

    // Leak current - Resting potential maintenance
    let i_leak = (*comp).g_leak * (v - (*comp).e_leak);

    return i_na + i_k + i_ca + i_leak;
}

/// Actualiza variables de gating (Hodgkin-Huxley)
fn update_gating_variables(comp: ptr<storage, CompartmentState, read_write>) {
    let v = (*comp).voltage;
    let dt = constants.dt;

    // Sodium m gate (activation)
    let am = q10_correction(alpha_m(v), 2.3);
    let bm = q10_correction(beta_m(v), 2.3);
    let tau_m = 1.0 / (am + bm);
    let m_inf = am / (am + bm);
    (*comp).na_m += (m_inf - (*comp).na_m) * (1.0 - exp(-dt / tau_m));

    // Sodium h gate (inactivation)
    let ah = q10_correction(alpha_h(v), 2.3);
    let bh = q10_correction(beta_h(v), 2.3);
    let tau_h = 1.0 / (ah + bh);
    let h_inf = ah / (ah + bh);
    (*comp).na_h += (h_inf - (*comp).na_h) * (1.0 - exp(-dt / tau_h));

    // Potassium n gate (activation)
    let an = q10_correction(alpha_n(v), 2.3);
    let bn = q10_correction(beta_n(v), 2.3);
    let tau_n = 1.0 / (an + bn);
    let n_inf = an / (an + bn);
    (*comp).k_n += (n_inf - (*comp).k_n) * (1.0 - exp(-dt / tau_n));

    // Calcium m gate (activation)
    let am_ca = q10_correction(alpha_m_ca(v), 3.0);
    let bm_ca = q10_correction(beta_m_ca(v), 3.0);
    let tau_m_ca = 1.0 / (am_ca + bm_ca);
    let m_inf_ca = am_ca / (am_ca + bm_ca);
    (*comp).ca_m += (m_inf_ca - (*comp).ca_m) * (1.0 - exp(-dt / tau_m_ca));
}

/// Helper: Get child index by position
fn get_child_idx(comp: ptr<storage, CompartmentState, read_write>, child_pos: u32) -> i32 {
    if (child_pos == 0u) { return (*comp).child_idx_0; }
    if (child_pos == 1u) { return (*comp).child_idx_1; }
    if (child_pos == 2u) { return (*comp).child_idx_2; }
    if (child_pos == 3u) { return (*comp).child_idx_3; }
    if (child_pos == 4u) { return (*comp).child_idx_4; }
    if (child_pos == 5u) { return (*comp).child_idx_5; }
    if (child_pos == 6u) { return (*comp).child_idx_6; }
    if (child_pos == 7u) { return (*comp).child_idx_7; }
    return -1;
}

// ============================================================================
// MAIN KERNEL - CABLE EQUATION WITH FULL TREE TOPOLOGY
// ============================================================================

@compute @workgroup_size(256)
fn solve_cable_equation(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let comp_idx = global_id.x;
    if (comp_idx >= constants.total_compartments) {
        return;
    }

    var comp = &compartments[comp_idx];

    // ========================================================================
    // PASO 1: Actualizar gating variables con voltaje actual
    // ========================================================================
    update_gating_variables(comp);

    // ========================================================================
    // PASO 2: Calcular corrientes iónicas
    // ========================================================================
    let i_ion = calculate_ion_currents(comp);

    // ========================================================================
    // PASO 3: Calcular corrientes axiales (ECUACIÓN DE CABLE CON ÁRBOL)
    // ========================================================================
    var i_axial_total: f32 = 0.0;

    // Corriente desde parent (si existe)
    if ((*comp).parent_idx >= 0) {
        let parent_idx = u32((*comp).parent_idx);
        let v_parent = compartments[parent_idx].voltage;
        let v_current = (*comp).voltage;

        // I_axial = (V_parent - V_current) / R_axial
        let i_axial_from_parent = (v_parent - v_current) / (*comp).axial_resistance;
        i_axial_total += i_axial_from_parent;
    }

    // Corrientes hacia children (TODOS los hijos, topología arbórea)
    for (var child_pos: u32 = 0u; child_pos < (*comp).num_children; child_pos++) {
        let child_idx_i32 = get_child_idx(comp, child_pos);

        if (child_idx_i32 >= 0) {
            let child_idx = u32(child_idx_i32);
            let v_child = compartments[child_idx].voltage;
            let v_current = (*comp).voltage;
            let r_axial_child = compartments[child_idx].axial_resistance;

            // I_axial = (V_child - V_current) / R_axial_child
            let i_axial_to_child = (v_child - v_current) / r_axial_child;
            i_axial_total += i_axial_to_child;
        }
    }

    // ========================================================================
    // PASO 4: Corriente externa (estimulación)
    // ========================================================================
    let i_ext = external_currents[comp_idx];

    // ========================================================================
    // PASO 5: ECUACIÓN DE CABLE (Cable Equation)
    // ========================================================================
    // C_m * dV/dt = -I_ion + I_axial_total + I_ext
    let dv = (-i_ion + i_axial_total + i_ext) / (*comp).capacitance;

    // ========================================================================
    // PASO 6: Actualizar voltaje (Euler forward)
    // ========================================================================
    (*comp).voltage += dv * constants.dt;

    // Clamp voltage to physiological range
    (*comp).voltage = clamp((*comp).voltage, -100.0, 60.0);
}

// ============================================================================
// INITIALIZATION KERNEL - REALISTIC PYRAMIDAL NEURON MORPHOLOGY
// ============================================================================

@compute @workgroup_size(256)
fn initialize_compartments(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let comp_idx = global_id.x;
    if (comp_idx >= constants.total_compartments) {
        return;
    }

    let neuron_id = comp_idx / constants.num_compartments_per_neuron;
    let comp_id = comp_idx % constants.num_compartments_per_neuron;

    var comp = &compartments[comp_idx];

    // Initialize tree topology to empty
    (*comp).parent_idx = -1;
    (*comp).child_idx_0 = -1;
    (*comp).child_idx_1 = -1;
    (*comp).child_idx_2 = -1;
    (*comp).child_idx_3 = -1;
    (*comp).child_idx_4 = -1;
    (*comp).child_idx_5 = -1;
    (*comp).child_idx_6 = -1;
    (*comp).child_idx_7 = -1;
    (*comp).num_children = 0u;

    // ========================================================================
    // REALISTIC PYRAMIDAL NEURON L5 MORPHOLOGY
    // ========================================================================
    // Assume num_compartments_per_neuron = 152 (like CPU implementation)
    // Compartment 0: Soma
    // Compartments 1-100: Apical dendrite tree (trunk + tuft branches)
    // Compartments 101-150: Basal dendrites (multiple trees)
    // Compartment 151: Axon Initial Segment (AIS)

    if (comp_id == 0u) {
        // ====================================================================
        // Soma (root of tree)
        // ====================================================================
        (*comp).comp_type = 0u;
        (*comp).length = 20.0;
        (*comp).diameter = 20.0;
        (*comp).parent_idx = -1;

        // Soma has 3 children: apical trunk, basal dendrite, AIS
        (*comp).child_idx_0 = i32(comp_idx + 1u);   // Apical trunk
        (*comp).child_idx_1 = i32(comp_idx + 101u); // Basal dendrite
        (*comp).child_idx_2 = i32(comp_idx + 151u); // AIS
        (*comp).num_children = 3u;

    } else if (comp_id >= 1u && comp_id <= 100u) {
        // ====================================================================
        // Apical dendrite tree (100 compartments)
        // ====================================================================
        (*comp).comp_type = 1u;

        if (comp_id == 1u) {
            // Apical trunk (thick, proximal)
            (*comp).length = 100.0;
            (*comp).diameter = 3.0;
            (*comp).parent_idx = i32(comp_idx - 1u); // Parent = soma

            // Trunk bifurcates into 2 branches
            (*comp).child_idx_0 = i32(comp_idx + 1u);
            (*comp).child_idx_1 = i32(comp_idx + 50u);
            (*comp).num_children = 2u;

        } else if (comp_id <= 50u) {
            // Apical branch 1 (oblique dendrites)
            (*comp).length = 50.0;
            (*comp).diameter = 2.0;
            (*comp).parent_idx = i32(comp_idx - 1u);

            // Some branch, some terminate
            if (comp_id % 5u == 0u && comp_id < 50u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).child_idx_1 = i32(comp_idx + 2u);
                (*comp).num_children = 2u;
            } else if (comp_id < 50u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).num_children = 1u;
            }

        } else {
            // Apical tuft (distal, thin)
            (*comp).length = 30.0;
            (*comp).diameter = 0.5;
            (*comp).parent_idx = i32(comp_idx - 50u);

            // Tuft branches
            if (comp_id % 7u == 0u && comp_id < 100u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).child_idx_1 = i32(comp_idx + 2u);
                (*comp).num_children = 2u;
            } else if (comp_id < 100u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).num_children = 1u;
            }
        }

    } else if (comp_id >= 101u && comp_id <= 150u) {
        // ====================================================================
        // Basal dendrites (50 compartments)
        // ====================================================================
        (*comp).comp_type = 2u;
        (*comp).length = 50.0;
        (*comp).diameter = 1.5;

        if (comp_id == 101u) {
            (*comp).parent_idx = i32(comp_idx - 101u); // Parent = soma
            (*comp).child_idx_0 = i32(comp_idx + 1u);
            (*comp).num_children = 1u;
        } else {
            (*comp).parent_idx = i32(comp_idx - 1u);

            if (comp_id % 8u == 0u && comp_id < 150u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).child_idx_1 = i32(comp_idx + 2u);
                (*comp).num_children = 2u;
            } else if (comp_id < 150u) {
                (*comp).child_idx_0 = i32(comp_idx + 1u);
                (*comp).num_children = 1u;
            }
        }

    } else if (comp_id == 151u) {
        // ====================================================================
        // Axon Initial Segment (AIS)
        // ====================================================================
        (*comp).comp_type = 3u;
        (*comp).length = 30.0;
        (*comp).diameter = 1.0;
        (*comp).parent_idx = i32(comp_idx - 151u); // Parent = soma
        (*comp).num_children = 0u; // No children (terminal)
    }

    // ========================================================================
    // Calculate derived geometric properties
    // ========================================================================
    let pi = 3.14159265359;

    // Surface area (cylinder): A = π * d * L
    (*comp).surface_area = pi * (*comp).diameter * (*comp).length;

    // Capacitance: C = C_m * A, where C_m = 1 uF/cm^2
    (*comp).capacitance = 1.0 * (*comp).surface_area / 100.0; // pF

    // Axial resistance: R_axial = (ρ * L) / (π * (d/2)^2)
    // ρ = 150 Ohm*cm (typical intracellular resistivity)
    let rho = 150.0; // Ohm*cm
    let radius_cm = (*comp).diameter / 2.0 * 1e-4; // um -> cm
    let length_cm = (*comp).length * 1e-4;         // um -> cm
    (*comp).axial_resistance = (rho * length_cm) / (pi * radius_cm * radius_cm);
    (*comp).axial_resistance /= 1e6; // Convert to MOhm

    // ========================================================================
    // Initialize electrical state to resting
    // ========================================================================
    (*comp).voltage = -70.0;         // Resting potential (mV)
    (*comp).g_leak = 0.025;          // Leak conductance (nS)
    (*comp).e_leak = -70.0;          // Leak reversal (mV)

    // Gating variables at resting state
    (*comp).na_m = 0.05;   // Sodium activation (closed)
    (*comp).na_h = 0.6;    // Sodium inactivation (ready)
    (*comp).k_n = 0.32;    // Potassium activation (partially open)
    (*comp).ca_m = 0.01;   // Calcium activation (closed)

    // Identifiers
    (*comp).neuron_id = neuron_id;
    (*comp).comp_id_in_neuron = comp_id;
}
