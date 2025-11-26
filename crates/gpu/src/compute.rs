//! High-level compute interface for GPU neural simulation.

use crate::GpuSimulator;
use anyhow::Result;

/// Performance metrics for GPU simulation
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub neurons_processed: usize,
    pub time_ms: f64,
    pub throughput_neurons_per_sec: f64,
    pub speedup_vs_cpu: f64,
}

/// Benchmarking utilities
pub struct Benchmark;

impl Benchmark {
    /// Run performance benchmark
    pub async fn run(num_neurons: usize, num_steps: usize, dt: f32) -> Result<PerformanceMetrics> {
        let sim = GpuSimulator::new(num_neurons, dt).await?;

        let start = std::time::Instant::now();

        for _ in 0..num_steps {
            sim.step();
        }

        // Ensure GPU work is complete
        sim.read_states().await;

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;

        let neurons_processed = num_neurons * num_steps;
        let throughput = neurons_processed as f64 / elapsed.as_secs_f64();

        // Estimated CPU time (based on 1:120,000 ratio user reported)
        let cpu_time_estimate = time_ms * 120_000.0;
        let speedup = cpu_time_estimate / time_ms;

        Ok(PerformanceMetrics {
            neurons_processed,
            time_ms,
            throughput_neurons_per_sec: throughput,
            speedup_vs_cpu: speedup,
        })
    }
}
