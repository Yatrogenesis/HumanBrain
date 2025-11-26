// Hodgkin-Huxley neuron model - GPU compute shader
// Parallelizes voltage updates across thousands of neurons

struct NeuronState {
    voltage: f32,        // Membrane voltage (mV)
    na_m: f32,           // Na+ activation
    na_h: f32,           // Na+ inactivation
    k_n: f32,            // K+ activation
    ca_m: f32,           // Ca2+ activation
    ca_h: f32,           // Ca2+ inactivation
    calcium: f32,        // [Ca2+]i (µM)
    external_current: f32, // Injected current (pA)
}

struct Constants {
    dt: f32,             // Time step (ms)
    num_neurons: u32,    // Total neurons
    g_na_bar: f32,       // Max Na+ conductance (nS)
    g_k_bar: f32,        // Max K+ conductance (nS)
    g_ca_bar: f32,       // Max Ca2+ conductance (nS)
    g_leak: f32,         // Leak conductance (nS)
    e_na: f32,           // Na+ reversal (mV)
    e_k: f32,            // K+ reversal (mV)
    e_ca: f32,           // Ca2+ reversal (mV)
    e_leak: f32,         // Leak reversal (mV)
    capacitance: f32,    // Membrane capacitance (pF)
}

@group(0) @binding(0)
var<storage, read_write> neurons: array<NeuronState>;

@group(0) @binding(1)
var<uniform> constants: Constants;

// Sodium channel rate functions
fn alpha_m(v: f32) -> f32 {
    return 0.1 * (v + 40.0) / (1.0 - exp(-0.1 * (v + 40.0)));
}

fn beta_m(v: f32) -> f32 {
    return 4.0 * exp(-0.0556 * (v + 65.0));
}

fn alpha_h(v: f32) -> f32 {
    return 0.07 * exp(-0.05 * (v + 65.0));
}

fn beta_h(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-0.1 * (v + 35.0)));
}

// Potassium channel rate functions
fn alpha_n(v: f32) -> f32 {
    return 0.01 * (v + 55.0) / (1.0 - exp(-0.1 * (v + 55.0)));
}

fn beta_n(v: f32) -> f32 {
    return 0.125 * exp(-0.0125 * (v + 65.0));
}

// Calcium channel steady-state and time constants
fn ca_m_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(-0.15 * (v + 20.0)));
}

fn ca_h_inf(v: f32) -> f32 {
    return 1.0 / (1.0 + exp(0.2 * (v + 50.0)));
}

@compute @workgroup_size(256)
fn update_neurons(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;

    // Bounds check
    if (idx >= constants.num_neurons) {
        return;
    }

    // Load neuron state
    var neuron = neurons[idx];
    let v = neuron.voltage;

    // ============================================================
    // 1. UPDATE ION CHANNEL GATING VARIABLES
    // ============================================================

    // Sodium (Na+) channel
    let am = alpha_m(v);
    let bm = beta_m(v);
    let ah = alpha_h(v);
    let bh = beta_h(v);

    neuron.na_m += (am * (1.0 - neuron.na_m) - bm * neuron.na_m) * constants.dt;
    neuron.na_h += (ah * (1.0 - neuron.na_h) - bh * neuron.na_h) * constants.dt;

    // Potassium (K+) channel
    let an = alpha_n(v);
    let bn = beta_n(v);

    neuron.k_n += (an * (1.0 - neuron.k_n) - bn * neuron.k_n) * constants.dt;

    // Calcium (Ca2+) channel
    let ca_m_ss = ca_m_inf(v);
    let ca_h_ss = ca_h_inf(v);
    let tau_ca_m = 0.5;
    let tau_ca_h = 20.0;

    neuron.ca_m += ((ca_m_ss - neuron.ca_m) / tau_ca_m) * constants.dt;
    neuron.ca_h += ((ca_h_ss - neuron.ca_h) / tau_ca_h) * constants.dt;

    // ============================================================
    // 2. CALCULATE ION CHANNEL CURRENTS
    // ============================================================

    // Sodium current: I_Na = g_Na * m^3 * h * (V - E_Na)
    let g_na = constants.g_na_bar * pow(neuron.na_m, 3.0) * neuron.na_h;
    let i_na = g_na * (v - constants.e_na);

    // Potassium current: I_K = g_K * n^4 * (V - E_K)
    let g_k = constants.g_k_bar * pow(neuron.k_n, 4.0);
    let i_k = g_k * (v - constants.e_k);

    // Calcium current: I_Ca = g_Ca * m^2 * h * (V - E_Ca)
    let g_ca = constants.g_ca_bar * pow(neuron.ca_m, 2.0) * neuron.ca_h;
    let i_ca = g_ca * (v - constants.e_ca);

    // Leak current: I_leak = g_leak * (V - E_leak)
    let i_leak = constants.g_leak * (v - constants.e_leak);

    // Total ionic current
    let i_ion = i_na + i_k + i_ca + i_leak;

    // ============================================================
    // 3. UPDATE VOLTAGE
    // ============================================================

    // Cable equation: C * dV/dt = -I_ion + I_ext
    // dV = (-I_ion + I_ext) / C * dt
    let dv = (-i_ion + neuron.external_current) / constants.capacitance * constants.dt;
    neuron.voltage += dv;

    // ============================================================
    // 4. UPDATE CALCIUM CONCENTRATION
    // ============================================================

    // Ca2+ influx from voltage-gated channels
    let ca_influx = -i_ca * 0.001; // Convert current to concentration change
    neuron.calcium += ca_influx * constants.dt;

    // Ca2+ extrusion/buffering (exponential decay)
    neuron.calcium *= 0.9;

    // Clamp to physiological range
    neuron.calcium = clamp(neuron.calcium, 0.0001, 0.01);

    // ============================================================
    // 5. WRITE BACK UPDATED STATE
    // ============================================================

    neurons[idx] = neuron;
}
