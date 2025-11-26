# HumanBrain Architecture

This document describes the architectural decisions and design philosophy behind HumanBrain.

## Design Philosophy

**"Maximize biological realism within computational constraints"**

We prioritize:
1. **Biological accuracy** over computational speed (but optimize when possible)
2. **Modularity** - each brain region is an independent crate
3. **Scalability** - support for statistical representations at scale
4. **Testability** - comprehensive unit tests for all components
5. **Documentation** - inline docs explaining biological basis

## Core Design Principles

### 1. Multi-Scale Modeling

HumanBrain operates at multiple scales simultaneously:

- **Subcellular**: Ion channels, calcium dynamics, metabolic processes
- **Cellular**: Multi-compartmental neurons, dendrites, axons
- **Network**: Synaptic connections, plasticity
- **Regional**: Cortical columns, hippocampal circuits
- **Whole-brain**: Long-range connections, cognitive functions

### 2. Modular Architecture

Each crate is self-contained and can be used independently:

```
neurons/        # Neuron models (used by all brain regions)
  ↓
synapses/       # Synaptic models (used by all brain regions)
  ↓
cortex/         # Neocortex (uses neurons + synapses)
hippocampus/    # Hippocampus (uses neurons + synapses)
cognition/      # High-level functions (uses cortex + hippocampus)
```

### 3. Separation of Concerns

**Neurons crate**: Pure neuron biophysics
- Compartmental models
- Ion channels
- Cable equation
- No knowledge of brain regions or networks

**Synapses crate**: Pure synaptic dynamics
- Release mechanisms
- Plasticity rules
- Neurotransmitter systems
- No knowledge of brain regions

**Brain region crates** (cortex, hippocampus, etc.):
- Network connectivity
- Region-specific cell types
- Layer organization
- Uses neurons and synapses crates

**Cognition crate**: High-level functions
- Working memory
- Attention
- Language
- Uses brain region crates

## Key Components

### Multi-Compartmental Neurons

**Why not point neurons?**
Point neuron models (Integrate-and-Fire, Izhikevich) ignore spatial dynamics:
- Cannot model dendritic computation
- No backpropagating action potentials
- No dendritic spikes (NMDA, Ca²⁺)
- Miss 80% of neuronal computation

**Our approach:**
- Cable equation: `C_m * dV/dt = (1/R_a) * d²V/dx² - g_leak*(V - E_leak) - I_ion`
- Each compartment coupled to neighbors
- Active conductances in dendrites

**Trade-off:**
- 152 compartments/neuron vs. 1 (152x more computation)
- But captures realistic dendritic integration

### Metabolic Constraints

**Why include metabolism?**
Neurons are energy-limited:
- Human brain = 2% of body weight, 20% of energy
- ATP limits firing rates
- Blood flow couples to activity

**Implementation:**
- ATP production (oxidative phosphorylation, glycolysis)
- ATP consumption (spikes, synapses, baseline)
- Neurovascular coupling (activity → blood flow)

**Effect:**
- Neurons cannot fire if ATP depleted
- Realistic energy budgets
- Captures metabolic diseases (hypoxia, ischemia)

### Glial Cells

**Why include glia?**
Glia outnumber neurons 1:1 and perform critical functions:
- Astrocytes: Glutamate clearance, K⁺ buffering
- Oligodendrocytes: Myelination (50-100x conduction speed)
- Microglia: Synaptic pruning, immune response

**Implementation:**
- Astrocytes clear glutamate (prevent excitotoxicity)
- K⁺ buffering prevents hyperexcitability
- Myelination increases conduction velocity
- Synaptic pruning removes weak connections

### Cortical Organization

**Six-layer structure:**

```
Layer 1:  Molecular layer (mostly dendrites/axons)
Layer 2/3: Pyramidal neurons → cortico-cortical
Layer 4:   Granule layer → receives thalamic input
Layer 5:   Large pyramidal → subcortical output
Layer 6:   Corticothalamic → feedback to thalamus
```

**Connectivity:**
- L4 → L2/3 (feedforward)
- L2/3 → L5 (intracolumnar)
- L5 → L6 (deep layers)
- L6 → L4 (feedback)

**Statistical scaling:**
- Full neocortex: 16 billion neurons
- 1 column template: 100,000 neurons (fully simulated)
- 160,000 columns total
- Representative sampling + statistical extrapolation

## Data Structures

### Neuron Representation

```rust
struct MultiCompartmentalNeuron {
    compartments: Vec<Compartment>,     // Spatial structure
    external_current: Array1<f64>,       // Injected current per compartment
    synaptic_current: Array1<f64>,       // Synaptic input per compartment
}

struct Compartment {
    voltage: f64,                        // Membrane potential
    length: f64,                         // Compartment length (um)
    diameter: f64,                       // Diameter (um)
    axial_resistance: f64,               // Coupling to neighbors
    parent_idx: Option<usize>,           // Tree structure
    children_idx: Vec<usize>,
}
```

**Memory layout:**
- Cache-friendly: Compartments stored contiguously
- Tree structure via indices (no pointers)

### Synaptic Network

```rust
struct SynapticNetwork {
    synapses: Vec<Synapse>,
    pre_to_synapses: Vec<Vec<usize>>,   // Pre-neuron → synapse indices
    post_to_synapses: Vec<Vec<usize>>,  // Post-neuron → synapse indices
}
```

**Why separate pre/post indices?**
- Fast lookup of incoming synapses (for post-synaptic current)
- Fast lookup of outgoing synapses (for spike propagation)
- Sparse matrix representation

### Connectivity Matrix

For large networks, we use sparse matrices:
- Cortical connectivity: ~10,000 synapses/neuron × 10⁹ neurons = 10¹³ synapses
- Sparse storage: Only store non-zero entries
- petgraph for graph algorithms

## Computational Strategy

### Parallelism

**Intra-column parallelism:**
```rust
// Update neurons in parallel
neurons.par_iter_mut()
    .for_each(|neuron| neuron.step());
```

**Inter-column parallelism:**
```rust
// Update columns in parallel
columns.par_iter_mut()
    .for_each(|column| column.step());
```

### SIMD Vectorization

Use ndarray for vectorized operations:
```rust
// Calculate all axial currents at once
let i_axial = (v_parent - v_compartments) / r_axial;
```

### GPU Acceleration (Future)

Target architecture:
- Neurons: 1 thread per neuron
- Synapses: 1 thread per synapse
- Reduce for post-synaptic currents

## Time Stepping

**Fixed time step**: dt = 0.01 - 0.1 ms
- Small enough for action potentials (1-2 ms duration)
- Large enough for computational efficiency

**Explicit Euler integration:**
```
V(t + dt) = V(t) + dV/dt * dt
```

**Future**: Adaptive time stepping for stiff systems

## Memory Management

**Memory requirements:**
- 1 neuron (152 compartments): ~10 KB
- 1 synapse: ~200 bytes
- 100,000 neuron column: ~1 GB
- 1,000 columns: ~1 TB (requires distributed memory)

**Optimization strategies:**
1. Sparse matrices for connectivity
2. HDF5 for disk storage
3. Streaming for large-scale simulations
4. Statistical representation for inactive regions

## Error Handling

**Design principle**: Use Rust's Result type for recoverable errors

```rust
pub type Result<T> = std::result::Result<T, NeuronError>;

fn step(&mut self) -> Result<()> {
    if self.atp < threshold {
        return Err(MetabolismError::EnergyConstraint("ATP depleted".into()));
    }
    Ok(())
}
```

**Error types:**
- `NeuronError`: Invalid compartment configuration
- `SynapseError`: Invalid plasticity parameters
- `MetabolismError`: Energy constraint violations
- `CortexError`: Network configuration errors

## Testing Strategy

**Unit tests** for each component:
```rust
#[test]
fn test_neuron_spike() {
    let mut neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
    neuron.inject_current(0, 100.0); // Strong stimulus

    for _ in 0..1000 {
        neuron.step(&mut states);
    }

    assert!(neuron.spike_count > 0);
}
```

**Integration tests** for networks:
```rust
#[test]
fn test_cortical_column() {
    let mut column = CorticalColumn::new(0, 1000, 0.1);
    column.step(&input)?;

    // Check physiological firing rates
    assert!(column.average_firing_rate() < 100.0); // Not hyperactive
}
```

## Future Directions

### GPU Acceleration
- CUDA/ROCm kernels for neuron updates
- Graph neural networks for connectivity
- Mixed precision (FP16 for some calculations)

### Distributed Computing
- MPI for multi-node simulations
- Partition brain regions across nodes
- Efficient inter-node communication

### Real-time Visualization
- WebGPU for 3D rendering
- Live spike rasters
- Voltage heatmaps

### Learning Algorithms
- Reinforcement learning in basal ganglia
- Supervised learning in cerebellum
- Unsupervised learning in cortex

## References

### Simulation Frameworks
- NEURON (Hines & Carnevale, 1997)
- NEST (Gewaltig & Diesmann, 2007)
- Brian2 (Stimberg et al., 2019)

### Cortical Modeling
- Markram et al. (2015). Blue Brain Project
- Potjans & Diesmann (2014). Cortical microcircuit model

### Computational Techniques
- Carnevale & Hines (2006). *The NEURON Book*
- Bower & Beeman (1998). *The Book of GENESIS*
