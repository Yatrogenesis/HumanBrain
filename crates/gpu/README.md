# GPU-Accelerated Neural Simulation

GPU compute backend for HumanBrain using WGPU (CUDA/Vulkan/Metal cross-platform support).

## Overview

This crate provides massively parallel computation for Hodgkin-Huxley neurons and cable equations targeting **100-400× speedup** over CPU.

### Performance Target

- **Current CPU**: ~1:120,000 ratio (1 second simulation = 33 hours)
- **GPU Target**: ~1:1 ratio (real-time simulation)
- **Expected Speedup**: 100-400×

## Architecture

### Compute Shader (`src/shaders/hodgkin_huxley.wgsl`)

WGSL compute shader implementing:
- Hodgkin-Huxley equations (Na+, K+, Ca2+ channels)
- Cable equation (multi-compartmental)
- Voltage-dependent ion channels
- Calcium dynamics
- Parallelization across 256 threads per workgroup

**Key Features:**
- Workgroup size: 256 neurons/threads
- Update frequency: Every timestep (dt)
- Memory layout: Structure of Arrays (SoA) for coalesced access

### Rust Backend (`src/lib.rs`)

GPU simulator with:
- `GpuSimulator`: Main interface for GPU-accelerated simulation
- `GpuNeuronState`: GPU-compatible neuron state (#[repr(C)])
- `HHConstants`: Simulation parameters (uniform buffer)
- Async operations for GPU read/write

**API Example:**
```rust
use gpu::GpuSimulator;

// Create simulator for 10,000 neurons
let sim = GpuSimulator::new(10_000, 0.01).await?;

// Inject currents
sim.set_currents(&currents);

// Step simulation (GPU kernel dispatch)
for _ in 0..1000 {
    sim.step();
}

// Read results
let voltages = sim.get_voltages().await;
```

## Implementation Status

### ✅ Completed (Fase 5)

1. **WGSL Compute Shader** (158 lines)
   - Hodgkin-Huxley equations
   - Ion channel gating dynamics
   - Cable equation coupling
   - Calcium concentration updates

2. **Rust GPU Backend** (357 lines)
   - wgpu initialization (CUDA/Vulkan/Metal)
   - Buffer management (storage + uniform)
   - Compute pipeline
   - Async GPU read/write
   - Test suite (4 tests)

3. **Structures**
   - `GpuNeuronState`: 8 floats (32 bytes)
   - `HHConstants`: 13 parameters + padding

### 🚧 Pending

1. **Sparse Matrix Operations**
   - CSR format for synaptic connectivity
   - GPU-optimized sparse-dense multiplication

2. **Event-Driven Propagation**
   - Spike queue on GPU
   - Conditional kernel execution

3. **Benchmarking**
   - CPU vs GPU comparison
   - Scaling tests (1K, 10K, 100K neurons)
   - Profiling tools

## Scientific Foundations

### Hodgkin-Huxley Model

```
C_m * dV/dt = -I_ion - I_axial + I_ext

where:
  I_Na  = g_Na * m^3 * h * (V - E_Na)
  I_K   = g_K * n^4 * (V - E_K)
  I_Ca  = g_Ca * m^2 * h * (V - E_Ca)
  I_leak = g_leak * (V - E_leak)
```

### Cable Equation

```
C_m * dV/dt = -I_ion + (V_parent - V) / R_axial + Σ(V_children - V) / R_child + I_ext
```

### Parameters (from literature)

- `g_Na_bar = 120 nS` (Hodgkin & Huxley, 1952)
- `g_K_bar = 36 nS`
- `E_Na = 50 mV`, `E_K = -90 mV`, `E_Ca = 120 mV`
- `C_m = 1 pF` (membrane capacitance)

## Dependencies

```toml
wgpu = "0.18"              # GPU compute
pollster = "0.3"           # Async runtime
bytemuck = "1.14"          # Pod types
futures-intrusive = "0.5"  # Async channels
ndarray = "0.15"           # Arrays
```

## Testing

```bash
cargo test --package gpu
```

Tests include:
- GPU initialization
- Single neuron resting potential
- Current injection → spike
- Parallel simulation (10K neurons)

## Future Optimizations

1. **Shared Memory**: Use workgroup-local memory for connectivity matrix
2. **Kernel Fusion**: Combine voltage update + channel gating in single kernel
3. **Half Precision**: Use `f16` where precision allows
4. **Persistent Kernels**: Keep GPU kernels resident
5. **Multi-GPU**: Distribute network across GPUs

## References

- Hodgkin & Huxley (1952): Action potential equations
- Hines & Carnevale (1997): NEURON simulator
- Brette & Goodman (2012): Brian simulator
- Knight & Nowotny (2018): GeNN GPU framework

## License

MIT OR Apache-2.0
