# Getting Started with HumanBrain

Welcome to HumanBrain, a comprehensive human brain simulator in Rust!

## Quick Start

### 1. Build the Project

```bash
cd HumanBrain
cargo build --release
```

### 2. Run Tests

```bash
# Test all crates
cargo test --all

# Test specific crates
cargo test --package neurons
cargo test --package synapses
cargo test --package cortex
```

### 3. Basic Usage

#### Simulate a Single Neuron

```rust
use neurons::{MultiCompartmentalNeuron, compartmental::ChannelStates};

fn main() {
    // Create a pyramidal neuron with 152 compartments
    let mut neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
    let mut states = vec![ChannelStates::default(); neuron.compartments.len()];

    // Inject current into soma
    neuron.inject_current(0, 100.0); // 100 pA

    // Simulate for 1000 steps
    for _ in 0..1000 {
        neuron.step(&mut states);

        if neuron.is_spiking {
            println!("Spike! Voltage: {:.1} mV", neuron.get_soma_voltage());
        }
    }
}
```

#### Simulate a Cortical Column

```rust
use cortex::CorticalColumn;

fn main() {
    // Create a column with 1000 neurons
    let mut column = CorticalColumn::new(0, 1000, 0.1);

    // External input
    let input = vec![10.0; 1000]; // 10 pA to each neuron

    // Simulate
    for _ in 0..10000 {
        column.step(&input).unwrap();
    }

    println!("Total spikes: {}", column.get_spike_count());
}
```

#### Simulate with Metabolism

```rust
use metabolism::NeuronMetabolism;

fn main() {
    let mut metabolism = NeuronMetabolism::new();

    // Simulate 100 spikes
    for i in 0..100 {
        let spiking = true;
        let synaptic_events = 10;

        match metabolism.step(0.1, spiking, synaptic_events) {
            Ok(_) => println!("Step {}: ATP = {:.2} mM", i, metabolism.atp),
            Err(e) => println!("Energy depleted: {}", e),
        }

        // Supply nutrients
        metabolism.supply_nutrients(0.1, 0.01);
    }
}
```

## Project Structure

```
HumanBrain/
├── crates/
│   ├── neurons/         - Multi-compartmental neuron models
│   ├── synapses/        - Synaptic dynamics and plasticity
│   ├── glia/            - Astrocytes, oligodendrocytes, microglia
│   ├── metabolism/      - ATP, glucose, oxygen dynamics
│   ├── cortex/          - Neocortex (6 layers)
│   ├── hippocampus/     - Memory circuits (to be implemented)
│   ├── thalamus/        - Thalamocortical loops (to be implemented)
│   └── ...
├── examples/            - Example programs
├── docs/                - Documentation
└── README.md            - Main documentation
```

## Key Concepts

### Multi-Compartmental Neurons

Unlike simple point neurons, HumanBrain neurons have spatial structure:

- **Soma**: Cell body where action potentials are generated
- **Dendrites**: Input regions (can have active conductances)
- **Axon**: Output region (high Na+ channel density)

Each compartment solves the cable equation:
```
C_m * dV/dt = (1/R_a) * d²V/dx² - I_leak - I_ion + I_ext
```

### Synaptic Dynamics

Synapses include:
- **Short-term plasticity**: Facilitation and depression
- **Long-term plasticity**: STDP (spike-timing-dependent)
- **Stochastic release**: Probabilistic neurotransmitter release
- **Multiple receptors**: AMPA, NMDA, GABA_A, GABA_B

### Metabolic Constraints

Neurons require ATP to function:
- **Spike cost**: ~10^9 ATP molecules per action potential
- **Baseline cost**: Maintaining ion gradients
- **Production**: Oxidative phosphorylation (requires O2) and glycolysis

### Glial Support

- **Astrocytes**: Clear glutamate, buffer K+, regulate blood flow
- **Oligodendrocytes**: Myelinate axons (50-100x faster conduction)
- **Microglia**: Prune weak synapses, immune response

## Common Patterns

### Pattern 1: Single Neuron Analysis

```rust
use neurons::MultiCompartmentalNeuron;
use neurons::compartmental::ChannelStates;

let mut neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
let mut states = vec![ChannelStates::default(); neuron.compartments.len()];

// Record voltage at different compartments
let soma_voltages = Vec::new();
let apical_voltages = Vec::new();

for _ in 0..10000 {
    neuron.inject_current(0, 100.0);
    neuron.step(&mut states);

    soma_voltages.push(neuron.compartments[0].voltage);
    apical_voltages.push(neuron.compartments[50].voltage);
}

// Analyze: backpropagation, dendritic spikes, etc.
```

### Pattern 2: Network Simulation

```rust
use cortex::CorticalColumn;

let mut column = CorticalColumn::new(0, 1000, 0.1);

// Apply different inputs over time
for t in 0..10000 {
    let mut input = vec![0.0; 1000];

    // Stimulus to layer 4 neurons (indices 400-600)
    if t > 1000 && t < 3000 {
        for i in 400..600 {
            input[i] = 50.0; // Strong input
        }
    }

    column.step(&input).unwrap();

    // Record activity in different layers
    // Analyze propagation, oscillations, etc.
}
```

### Pattern 3: Plasticity Studies

```rust
use synapses::Synapse;
use synapses::SynapseType;

let mut synapse = Synapse::new(0, 0, 1, SynapseType::AMPA, 0.5);

// Pairing protocol (100 pairings)
for _ in 0..100 {
    // Pre spike at t=0
    synapse.last_pre_spike = 0.0;

    // Post spike at t=10 (10 ms later)
    synapse.last_post_spike = 10.0;

    // Apply STDP
    synapse.apply_stdp(10.0);
}

println!("Final weight: {:.2}", synapse.weight);
// Should show LTP (weight increase)
```

## Debugging Tips

### 1. Check Voltage Stability

```rust
// Neurons should rest at -70 mV without input
let v = neuron.get_soma_voltage();
assert!(v > -75.0 && v < -65.0, "Unstable resting potential");
```

### 2. Check Firing Rates

```rust
// Typical cortical neurons fire 0.1-10 Hz
let rate = column.get_spike_count() as f64 / (num_neurons as f64 * time_seconds);
assert!(rate < 100.0, "Network hyperactive");
```

### 3. Check Metabolic Health

```rust
// ATP should stay above 0.5 mM
assert!(metabolism.atp > 0.5, "ATP depleted");
```

### 4. Enable Detailed Logging

```rust
// Add to Cargo.toml:
// tracing = "0.1"
// tracing-subscriber = "0.3"

use tracing::info;

tracing_subscriber::fmt::init();

info!("Neuron voltage: {:.2}", neuron.get_soma_voltage());
```

## Performance Tips

### 1. Use Release Mode

```bash
cargo build --release
cargo run --release
```

Release mode is 10-100x faster than debug mode.

### 2. Reduce Compartments for Speed

```rust
// Faster: 10 compartments
let neuron = MultiCompartmentalNeuron::new(0, 10, 0.01);

// Slower but more accurate: 152 compartments
let neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
```

### 3. Use Larger Time Steps (Carefully)

```rust
// Faster: dt = 0.1 ms
let column = CorticalColumn::new(0, 1000, 0.1);

// More accurate: dt = 0.01 ms
let column = CorticalColumn::new(0, 1000, 0.01);
```

Note: Time step must be small enough to capture action potentials (< 0.1 ms recommended).

### 4. Parallelize Multiple Simulations

```rust
use rayon::prelude::*;

let results: Vec<_> = (0..100).into_par_iter()
    .map(|trial| {
        let mut column = CorticalColumn::new(trial, 1000, 0.1);
        // Run simulation
        // Return results
    })
    .collect();
```

## Next Steps

1. **Read the [README](README.md)** for architectural overview
2. **Explore [ARCHITECTURE.md](docs/ARCHITECTURE.md)** for design details
3. **Check [BIOLOGICAL_ACCURACY.md](docs/BIOLOGICAL_ACCURACY.md)** for validation
4. **Review [BENCHMARKS.md](docs/BENCHMARKS.md)** for performance data
5. **Look at [examples/](examples/)** for complete programs
6. **Run tests** to see the simulator in action

## Getting Help

- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions, share results
- **Documentation**: Inline docs (`cargo doc --open`)

## Contributing

Contributions welcome! Areas needing help:
- Hippocampus implementation
- Thalamus implementation
- GPU acceleration
- Visualization tools
- Validation against experimental data

See [README.md](README.md) for contribution guidelines.

---

Happy simulating!
