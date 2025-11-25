//! Simple example: Simulate a single pyramidal neuron

use neurons::{MultiCompartmentalNeuron, compartmental::ChannelStates};

fn main() {
    println!("HumanBrain: Single Neuron Simulation");
    println!("=====================================\n");

    // Create a pyramidal neuron with realistic morphology
    let mut neuron = MultiCompartmentalNeuron::new_pyramidal(0, 0.01);
    let num_compartments = neuron.compartments.len();

    println!("Created pyramidal neuron:");
    println!("  - Compartments: {}", num_compartments);
    println!("  - Soma diameter: {:.1} um", neuron.compartments[0].diameter);
    println!("  - Resting voltage: {:.1} mV\n", neuron.compartments[0].voltage);

    // Initialize channel states
    let mut channel_states = vec![ChannelStates::default(); num_compartments];

    // Inject current into soma (100 pA for 100 ms)
    println!("Injecting 100 pA current into soma...\n");

    let mut spike_times = Vec::new();
    let simulation_time = 500.0; // ms
    let dt = 0.01; // ms

    for step in 0..(simulation_time / dt) as usize {
        let t = step as f64 * dt;

        // Apply current injection for first 100 ms
        if t < 100.0 {
            neuron.inject_current(0, 100.0);
        } else {
            neuron.inject_current(0, 0.0);
        }

        // Step the simulation
        neuron.step(&mut channel_states);

        // Detect spikes
        if neuron.is_spiking && (spike_times.is_empty() || t - spike_times.last().unwrap() > 2.0) {
            spike_times.push(t);
            println!("Spike at t = {:.2} ms (V = {:.1} mV)",
                     t, neuron.get_soma_voltage());
        }
    }

    println!("\nSimulation complete!");
    println!("Total spikes: {}", spike_times.len());

    if spike_times.len() > 1 {
        let isi: Vec<f64> = spike_times.windows(2)
            .map(|w| w[1] - w[0])
            .collect();
        let mean_isi = isi.iter().sum::<f64>() / isi.len() as f64;
        let firing_rate = 1000.0 / mean_isi;
        println!("Mean ISI: {:.2} ms", mean_isi);
        println!("Firing rate: {:.2} Hz", firing_rate);
    }

    println!("\nFinal soma voltage: {:.2} mV", neuron.get_soma_voltage());
}
