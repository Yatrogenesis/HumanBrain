// Advanced Ion Channel Models - GPU Compute Shader
// Implements 15+ specialized ion channels with realistic kinetics
// Target: 100-400× speedup for biophysically accurate simulation

// ============================================================
// STRUCTURES
// ============================================================

struct AdvancedNeuronState {
    // Membrane voltage
    voltage: f32,

    // Nav1.1 (Soma/dendrite sodium)
    nav1_1_m: f32,
    nav1_1_h: f32,

    // Nav1.6 (AIS sodium - faster kinetics)
    nav1_6_m: f32,
    nav1_6_h: f32,

    // Kv1.1 (Low-threshold K+)
    kv1_1_n: f32,

    // Kv3.1 (Fast-spiking K+)
    kv3_1_n: f32,

    // Kv4.2 (A-type transient K+)
    kv4_2_m: f32,
    kv4_2_h: f32,

    // Kv7/M (M-current - slow adaptation)
    kv7_m: f32,

    // Cav1.2 (L-type Ca2+)
    cav1_2_m: f32,
    cav1_2_h: f32,

    // Cav2.1 (P/Q-type Ca2+)
    cav2_1_m: f32,
    cav2_1_h: f32,

    // Cav3.1 (T-type Ca2+ - low threshold)
    cav3_1_m: f32,
    cav3_1_h: f32,

    // SK channel (Ca-activated K+)
    sk_ca_i: f32,  // Internal calcium for SK

    // BK channel (Big K+ Ca-activated)
    bk_m: f32,
    bk_ca_i: f32,

    // HCN (Pacemaker current)
    hcn_m: f32,

    // NMDA receptor
    nmda_m: f32,

    // External current injection
    i_ext: f32,

    // Glutamate concentration (for NMDA)
    glu: f32,

    // Padding for alignment
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

struct ChannelConstants {
    dt: f32,
    num_neurons: u32,
    temperature: f32,      // 37.0 C
    ref_temperature: f32,  // 22.0 C

    // Nav1.1
    g_nav1_1: f32,
    e_na: f32,

    // Nav1.6
    g_nav1_6: f32,

    // Potassium channels
    g_kv1_1: f32,
    g_kv3_1: f32,
    g_kv4_2: f32,
    g_kv7_m: f32,
    e_k: f32,

    // Calcium channels
    g_cav1_2: f32,
    g_cav2_1: f32,
    g_cav3_1: f32,
    e_ca: f32,

    // Calcium-activated K+
    g_sk: f32,
    g_bk: f32,

    // HCN
    g_hcn: f32,
    e_h: f32,  // -30 mV (non-selective cation)

    // NMDA
    g_nmda: f32,
    e_nmda: f32,  // 0 mV
    mg_conc: f32,  // 1.0 mM

    // Leak
    g_leak: f32,
    e_leak: f32,

    // Membrane
    capacitance: f32,

    _padding: array<u32, 3>,
}

@group(0) @binding(0)
var<storage, read_write> neurons: array<AdvancedNeuronState>;

@group(0) @binding(1)
var<uniform> constants: ChannelConstants;

// ============================================================
// Q10 TEMPERATURE CORRECTION
// ============================================================

fn q10_correction(rate: f32, q10: f32) -> f32 {
    let temp_diff = (constants.temperature - constants.ref_temperature) / 10.0;
    return rate * pow(q10, temp_diff);
}

// ============================================================
// NAV1.1 - SOMA/DENDRITE SODIUM CHANNEL
// ============================================================

fn nav1_1_alpha_m(v: f32) -> f32 {
    let v_shift = v + 38.0;
    if (abs(v_shift) < 0.0001) {
        return 3.0;
    }
    return 0.182 * v_shift / (1.0 - exp(-v_shift / 6.0));
}

fn nav1_1_beta_m(v: f32) -> f32 {
    let v_shift = v + 38.0;
    return 0.124 * (-v_shift) / (1.0 - exp(v_shift / 6.0));
}

fn nav1_1_alpha_h(v: f32) -> f32 {
    return 0.024 * (v + 50.0) / (1.0 - exp(-(v + 50.0) / 5.0));
}

fn nav1_1_beta_h(v: f32) -> f32 {
    return 0.0091 * (-(v + 75.0)) / (1.0 - exp((v + 75.0) / 5.0));
}

fn nav1_1_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_nav1_1 * pow(m, 3.0) * h * (v - constants.e_na);
}

// ============================================================
// NAV1.6 - AXON INITIAL SEGMENT SODIUM (FASTER)
// ============================================================

fn nav1_6_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 30.0) / 6.0));
}

fn nav1_6_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 60.0) / 6.5));
}

fn nav1_6_tau_m(v: f32) -> f32 {
    return 0.1 + 0.4 / (1.0 + abs((v + 35.0) / 10.0));
}

fn nav1_6_tau_h(v: f32) -> f32 {
    return 1.5 + 1.0 / (1.0 + exp((v + 60.0) / 15.0));
}

fn nav1_6_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_nav1_6 * pow(m, 3.0) * h * (v - constants.e_na);
}

// ============================================================
// KV1.1 - LOW-THRESHOLD POTASSIUM
// ============================================================

fn kv1_1_n_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 15.0) / 8.0));
}

fn kv1_1_tau_n(v: f32) -> f32 {
    return 1.0 + 4.0 / (1.0 + exp((v + 30.0) / 20.0));
}

fn kv1_1_conductance(v: f32, n: f32) -> f32 {
    return constants.g_kv1_1 * pow(n, 4.0) * (v - constants.e_k);
}

// ============================================================
// KV3.1 - FAST-SPIKING INTERNEURON POTASSIUM
// ============================================================

fn kv3_1_n_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 10.0) / 16.0));
}

fn kv3_1_tau_n(v: f32) -> f32 {
    return 0.5 + 2.0 / (1.0 + exp((v + 40.0) / 15.0));
}

fn kv3_1_conductance(v: f32, n: f32) -> f32 {
    return constants.g_kv3_1 * pow(n, 4.0) * (v - constants.e_k);
}

// ============================================================
// KV4.2 - A-TYPE TRANSIENT POTASSIUM
// ============================================================

fn kv4_2_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 60.0) / 8.5));
}

fn kv4_2_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 78.0) / 6.0));
}

fn kv4_2_tau_m(v: f32) -> f32 {
    return 1.0 + 10.0 / (1.0 + exp((v + 60.0) / 20.0));
}

fn kv4_2_tau_h(v: f32) -> f32 {
    return 15.0 + 40.0 / (1.0 + exp((v + 70.0) / 15.0));
}

fn kv4_2_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_kv4_2 * pow(m, 4.0) * h * (v - constants.e_k);
}

// ============================================================
// KV7/M - M-CURRENT (SLOW ADAPTATION)
// ============================================================

fn kv7_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 35.0) / 10.0));
}

fn kv7_tau_m() -> f32 {
    return 100.0;  // Very slow
}

fn kv7_conductance(v: f32, m: f32) -> f32 {
    return constants.g_kv7_m * m * (v - constants.e_k);
}

// ============================================================
// CAV1.2 - L-TYPE CALCIUM
// ============================================================

fn cav1_2_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 10.0) / 6.0));
}

fn cav1_2_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 30.0) / 12.0));
}

fn cav1_2_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_cav1_2 * pow(m, 2.0) * h * (v - constants.e_ca);
}

// ============================================================
// CAV2.1 - P/Q-TYPE CALCIUM (PRESYNAPTIC)
// ============================================================

fn cav2_1_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 5.0) / 7.0));
}

fn cav2_1_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 25.0) / 8.0));
}

fn cav2_1_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_cav2_1 * pow(m, 2.0) * h * (v - constants.e_ca);
}

// ============================================================
// CAV3.1 - T-TYPE CALCIUM (LOW-THRESHOLD, BURST FIRING)
// ============================================================

fn cav3_1_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(v + 52.0) / 7.4));
}

fn cav3_1_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 80.0) / 5.0));
}

fn cav3_1_tau_m(v: f32) -> f32 {
    return 1.0 + 10.0 / (1.0 + exp((v + 60.0) / 15.0));
}

fn cav3_1_tau_h(v: f32) -> f32 {
    return 20.0 + 50.0 / (1.0 + exp((v + 70.0) / 10.0));
}

fn cav3_1_conductance(v: f32, m: f32, h: f32) -> f32 {
    return constants.g_cav3_1 * pow(m, 2.0) * h * (v - constants.e_ca);
}

// ============================================================
// SK - SMALL CONDUCTANCE CA-ACTIVATED K+
// ============================================================

fn sk_m_inf(ca_i: f32) -> f32 {
    // Hill equation: Kd = 0.3 μM, n = 4
    let k_d = 0.3;
    let n = 4.0;
    let ca4 = pow(ca_i, n);
    let kd4 = pow(k_d, n);
    return ca4 / (ca4 + kd4);
}

fn sk_conductance(v: f32, ca_i: f32) -> f32 {
    let m_inf = sk_m_inf(ca_i);
    return constants.g_sk * m_inf * (v - constants.e_k);
}

// ============================================================
// BK - BIG CONDUCTANCE CA-ACTIVATED K+
// ============================================================

fn bk_m_inf(v: f32, ca_i: f32) -> f32 {
    // Voltage AND calcium dependent
    let v_half = -20.0 - 80.0 / (1.0 + pow(ca_i / 1.0, 2.0));
    return 1.0 / (1.0 + exp(-(v - v_half) / 15.0));
}

fn bk_tau_m() -> f32 {
    return 1.0;  // Fast
}

fn bk_conductance(v: f32, m: f32) -> f32 {
    return constants.g_bk * pow(m, 2.0) * (v - constants.e_k);
}

// ============================================================
// HCN - HYPERPOLARIZATION-ACTIVATED CATION (PACEMAKER)
// ============================================================

fn hcn_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp((v + 75.0) / 5.5));
}

fn hcn_tau_m(v: f32) -> f32 {
    return 100.0 + 500.0 / (1.0 + exp((v + 70.0) / 10.0));
}

fn hcn_conductance(v: f32, m: f32) -> f32 {
    return constants.g_hcn * m * (v - constants.e_h);
}

// ============================================================
// NMDA - WITH MG2+ BLOCK
// ============================================================

fn nmda_mg_block(v: f32) -> f32 {
    // Year & Stevens (1990)
    return 1.0 / (1.0 + (constants.mg_conc / 3.57) * exp(-0.062 * v));
}

fn nmda_m_inf(glu: f32) -> f32 {
    let k_d = 10.0;  // μM
    return glu / (glu + k_d);
}

fn nmda_conductance(v: f32, m: f32, glu: f32) -> f32 {
    let mg_block = nmda_mg_block(v);
    let glu_binding = nmda_m_inf(glu);
    return constants.g_nmda * glu_binding * m * mg_block * (v - constants.e_nmda);
}

// ============================================================
// MAIN COMPUTE KERNEL
// ============================================================

@compute @workgroup_size(256)
fn update_neurons_advanced(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    if (idx >= constants.num_neurons) {
        return;
    }

    var neuron = neurons[idx];
    let v = neuron.voltage;
    let dt = constants.dt;

    // ================================================================
    // 1. UPDATE GATING VARIABLES
    // ================================================================

    // Nav1.1
    let nav1_1_am = q10_correction(nav1_1_alpha_m(v), 2.3);
    let nav1_1_bm = q10_correction(nav1_1_beta_m(v), 2.3);
    let nav1_1_ah = q10_correction(nav1_1_alpha_h(v), 2.3);
    let nav1_1_bh = q10_correction(nav1_1_beta_h(v), 2.3);
    neuron.nav1_1_m += (nav1_1_am * (1.0 - neuron.nav1_1_m) - nav1_1_bm * neuron.nav1_1_m) * dt;
    neuron.nav1_1_h += (nav1_1_ah * (1.0 - neuron.nav1_1_h) - nav1_1_bh * neuron.nav1_1_h) * dt;

    // Nav1.6 (faster kinetics)
    let nav1_6_minf = nav1_6_m_inf(v);
    let nav1_6_hinf = nav1_6_h_inf(v);
    let nav1_6_taum = q10_correction(nav1_6_tau_m(v), 2.3);
    let nav1_6_tauh = q10_correction(nav1_6_tau_h(v), 2.3);
    neuron.nav1_6_m += (nav1_6_minf - neuron.nav1_6_m) / nav1_6_taum * dt;
    neuron.nav1_6_h += (nav1_6_hinf - neuron.nav1_6_h) / nav1_6_tauh * dt;

    // Kv1.1
    let kv1_1_ninf = kv1_1_n_inf(v);
    let kv1_1_taun = q10_correction(kv1_1_tau_n(v), 3.0);
    neuron.kv1_1_n += (kv1_1_ninf - neuron.kv1_1_n) / kv1_1_taun * dt;

    // Kv3.1
    let kv3_1_ninf = kv3_1_n_inf(v);
    let kv3_1_taun = q10_correction(kv3_1_tau_n(v), 3.0);
    neuron.kv3_1_n += (kv3_1_ninf - neuron.kv3_1_n) / kv3_1_taun * dt;

    // Kv4.2
    let kv4_2_minf = kv4_2_m_inf(v);
    let kv4_2_hinf = kv4_2_h_inf(v);
    let kv4_2_taum = q10_correction(kv4_2_tau_m(v), 3.0);
    let kv4_2_tauh = q10_correction(kv4_2_tau_h(v), 3.0);
    neuron.kv4_2_m += (kv4_2_minf - neuron.kv4_2_m) / kv4_2_taum * dt;
    neuron.kv4_2_h += (kv4_2_hinf - neuron.kv4_2_h) / kv4_2_tauh * dt;

    // Kv7/M (slow)
    let kv7_minf = kv7_m_inf(v);
    let kv7_taum = q10_correction(kv7_tau_m(), 2.5);
    neuron.kv7_m += (kv7_minf - neuron.kv7_m) / kv7_taum * dt;

    // Cav1.2
    let cav1_2_minf = cav1_2_m_inf(v);
    let cav1_2_hinf = cav1_2_h_inf(v);
    neuron.cav1_2_m += (cav1_2_minf - neuron.cav1_2_m) / 5.0 * dt;
    neuron.cav1_2_h += (cav1_2_hinf - neuron.cav1_2_h) / 50.0 * dt;

    // Cav2.1
    let cav2_1_minf = cav2_1_m_inf(v);
    let cav2_1_hinf = cav2_1_h_inf(v);
    neuron.cav2_1_m += (cav2_1_minf - neuron.cav2_1_m) / 3.0 * dt;
    neuron.cav2_1_h += (cav2_1_hinf - neuron.cav2_1_h) / 25.0 * dt;

    // Cav3.1 (T-type)
    let cav3_1_minf = cav3_1_m_inf(v);
    let cav3_1_hinf = cav3_1_h_inf(v);
    let cav3_1_taum = q10_correction(cav3_1_tau_m(v), 3.0);
    let cav3_1_tauh = q10_correction(cav3_1_tau_h(v), 3.0);
    neuron.cav3_1_m += (cav3_1_minf - neuron.cav3_1_m) / cav3_1_taum * dt;
    neuron.cav3_1_h += (cav3_1_hinf - neuron.cav3_1_h) / cav3_1_tauh * dt;

    // BK
    let bk_minf = bk_m_inf(v, neuron.bk_ca_i);
    let bk_taum = q10_correction(bk_tau_m(), 3.0);
    neuron.bk_m += (bk_minf - neuron.bk_m) / bk_taum * dt;

    // HCN
    let hcn_minf = hcn_m_inf(v);
    let hcn_taum = q10_correction(hcn_tau_m(v), 2.5);
    neuron.hcn_m += (hcn_minf - neuron.hcn_m) / hcn_taum * dt;

    // NMDA
    let nmda_minf = nmda_m_inf(neuron.glu);
    neuron.nmda_m += (nmda_minf - neuron.nmda_m) / 50.0 * dt;

    // ================================================================
    // 2. CALCULATE CURRENTS
    // ================================================================

    let i_nav1_1 = nav1_1_conductance(v, neuron.nav1_1_m, neuron.nav1_1_h);
    let i_nav1_6 = nav1_6_conductance(v, neuron.nav1_6_m, neuron.nav1_6_h);
    let i_kv1_1 = kv1_1_conductance(v, neuron.kv1_1_n);
    let i_kv3_1 = kv3_1_conductance(v, neuron.kv3_1_n);
    let i_kv4_2 = kv4_2_conductance(v, neuron.kv4_2_m, neuron.kv4_2_h);
    let i_kv7 = kv7_conductance(v, neuron.kv7_m);
    let i_cav1_2 = cav1_2_conductance(v, neuron.cav1_2_m, neuron.cav1_2_h);
    let i_cav2_1 = cav2_1_conductance(v, neuron.cav2_1_m, neuron.cav2_1_h);
    let i_cav3_1 = cav3_1_conductance(v, neuron.cav3_1_m, neuron.cav3_1_h);
    let i_sk = sk_conductance(v, neuron.sk_ca_i);
    let i_bk = bk_conductance(v, neuron.bk_m);
    let i_hcn = hcn_conductance(v, neuron.hcn_m);
    let i_nmda = nmda_conductance(v, neuron.nmda_m, neuron.glu);
    let i_leak = constants.g_leak * (v - constants.e_leak);

    let i_ion = i_nav1_1 + i_nav1_6 + i_kv1_1 + i_kv3_1 + i_kv4_2 + i_kv7 +
                i_cav1_2 + i_cav2_1 + i_cav3_1 + i_sk + i_bk + i_hcn + i_nmda + i_leak;

    // ================================================================
    // 3. UPDATE VOLTAGE
    // ================================================================

    let dv = (-i_ion + neuron.i_ext) / constants.capacitance * dt;
    neuron.voltage += dv;

    // ================================================================
    // 4. UPDATE CALCIUM CONCENTRATIONS
    // ================================================================

    // SK calcium pool
    let ca_decay = 0.002;
    let ca_influx_sk = -i_cav1_2 * 0.001 - i_cav2_1 * 0.001 - i_cav3_1 * 0.001;
    neuron.sk_ca_i += (-ca_decay * neuron.sk_ca_i + ca_influx_sk) * dt;
    neuron.sk_ca_i = max(neuron.sk_ca_i, 0.05);
    neuron.sk_ca_i = min(neuron.sk_ca_i, 10.0);

    // BK calcium pool
    let ca_influx_bk = select(0.0, 0.01, v > -20.0);
    neuron.bk_ca_i += (-ca_decay * neuron.bk_ca_i + ca_influx_bk) * dt;
    neuron.bk_ca_i = max(neuron.bk_ca_i, 0.05);
    neuron.bk_ca_i = min(neuron.bk_ca_i, 10.0);

    // NMDA calcium influx
    if (neuron.nmda_m > 0.1) {
        neuron.sk_ca_i += 0.005 * dt;
    }

    // ================================================================
    // 5. WRITE BACK
    // ================================================================

    neurons[idx] = neuron;
}
