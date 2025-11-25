//! Example: Simulate a small cortical network

use cortex::CorticalColumn;

fn main() {
    println!("HumanBrain: Cortical Network Simulation");
    println!("========================================\n");

    // Create a cortical column with 1000 neurons
    let num_neurons = 1000;
    let dt = 0.1; // ms

    println!("Creating cortical column:");
    println!("  - Neurons: {}", num_neurons);
    println!("  - Time step: {} ms", dt);
    println!("  - Initializing...");

    let mut column = CorticalColumn::new(0, num_neurons, dt);

    println!("  - Synapses: {}", column.synaptic_network.synapses.len());
    println!("  - Astrocytes: {}", column.astrocytes.len());
    println!("  - Oligodendrocytes: {}", column.oligodendrocytes.len());
    println!("  - Microglia: {}\n", column.microglia.len());

    // Simulate for 1000 ms
    let simulation_time = 1000.0; // ms
    let num_steps = (simulation_time / dt) as usize;

    println!("Running simulation for {} ms...", simulation_time);

    // External input (baseline + some random fluctuations)
    let mut input = vec![0.0; num_neurons];

    for step in 0..num_steps {
        let t = step as f64 * dt;

        // Apply some external input to a subset of neurons
        for i in 0..num_neurons {
            // Random background input
            input[i] = if rand::random::<f64>() < 0.01 {
                50.0 // pA
            } else {
                0.0
            };
        }

        // Add stimulus at t=200-400 ms to first 100 neurons
        if t >= 200.0 && t < 400.0 {
            for i in 0..100 {
                input[i] += 30.0;
            }
        }

        // Step the simulation
        column.step(&input).ok();

        // Print progress every 100 ms
        if step % ((100.0 / dt) as usize) == 0 {
            let avg_voltage = column.get_average_voltage();
            let spike_count = column.get_spike_count();
            println!("  t = {:.0} ms: {} spikes, avg V = {:.2} mV",
                     t, spike_count, avg_voltage);
        }
    }

    println!("\nSimulation complete!");
    println!("Total spikes: {}", column.get_spike_count());
    println!("Average firing rate: {:.2} Hz",
             column.get_spike_count() as f64 / (num_neurons as f64 * simulation_time / 1000.0));

    println!("\nMetabolic state:");
    println!("  Average ATP: {:.2} mM", column.metabolism.average_atp());
}
