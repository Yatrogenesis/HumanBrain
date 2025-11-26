//! GPU-accelerated neural simulation using wgpu compute shaders.
//!
//! This module provides massively parallel computation for Hodgkin-Huxley
//! neurons and cable equations, targeting 100-400× speedup over CPU.

pub mod compute;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::{util::DeviceExt, Device, Queue};

/// Neuron state for GPU computation (must be #[repr(C)] for GPU alignment)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuNeuronState {
    pub voltage: f32,
    pub na_m: f32,
    pub na_h: f32,
    pub k_n: f32,
    pub ca_m: f32,
    pub ca_h: f32,
    pub calcium: f32,
    pub external_current: f32,
}

/// Advanced neuron state matching channels_advanced.wgsl
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AdvancedNeuronState {
    pub voltage: f32,
    // Nav1.1
    pub nav1_1_m: f32,
    pub nav1_1_h: f32,
    // Nav1.6
    pub nav1_6_m: f32,
    pub nav1_6_h: f32,
    // Kv1.1
    pub kv1_1_n: f32,
    // Kv3.1
    pub kv3_1_n: f32,
    // Kv4.2
    pub kv4_2_m: f32,
    pub kv4_2_h: f32,
    // Kv7/M
    pub kv7_m: f32,
    // Cav1.2
    pub cav1_2_m: f32,
    pub cav1_2_h: f32,
    // Cav2.1
    pub cav2_1_m: f32,
    pub cav2_1_h: f32,
    // Cav3.1
    pub cav3_1_m: f32,
    pub cav3_1_h: f32,
    // SK
    pub sk_ca_i: f32,
    // BK
    pub bk_m: f32,
    pub bk_ca_i: f32,
    // HCN
    pub hcn_m: f32,
    // NMDA
    pub nmda_m: f32,
    // External current
    pub i_ext: f32,
    // Glutamate
    pub glu: f32,
    // Padding for GPU alignment
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

impl Default for AdvancedNeuronState {
    fn default() -> Self {
        Self {
            voltage: -70.0,
            nav1_1_m: 0.05,
            nav1_1_h: 0.6,
            nav1_6_m: 0.05,
            nav1_6_h: 0.6,
            kv1_1_n: 0.32,
            kv3_1_n: 0.32,
            kv4_2_m: 0.1,
            kv4_2_h: 0.9,
            kv7_m: 0.0,
            cav1_2_m: 0.01,
            cav1_2_h: 0.9,
            cav2_1_m: 0.01,
            cav2_1_h: 0.9,
            cav3_1_m: 0.01,
            cav3_1_h: 0.9,
            sk_ca_i: 0.05,
            bk_m: 0.0,
            bk_ca_i: 0.05,
            hcn_m: 0.0,
            nmda_m: 0.0,
            i_ext: 0.0,
            glu: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

impl Default for GpuNeuronState {
    fn default() -> Self {
        Self {
            voltage: -70.0,    // Resting potential
            na_m: 0.05,        // Sodium activation
            na_h: 0.6,         // Sodium inactivation
            k_n: 0.32,         // Potassium activation
            ca_m: 0.01,        // Calcium activation
            ca_h: 0.9,         // Calcium inactivation
            calcium: 0.0001,   // Resting Ca2+ (µM)
            external_current: 0.0,
        }
    }
}

/// Constants for HH simulation
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct HHConstants {
    pub dt: f32,
    pub num_neurons: u32,
    pub g_na_bar: f32,    // 120.0 nS
    pub g_k_bar: f32,     // 36.0 nS
    pub g_ca_bar: f32,    // 5.0 nS
    pub g_leak: f32,      // 0.3 nS
    pub e_na: f32,        // 50.0 mV
    pub e_k: f32,         // -90.0 mV
    pub e_ca: f32,        // 120.0 mV
    pub e_leak: f32,      // -70.0 mV
    pub capacitance: f32, // 1.0 pF
    pub _padding: [u32; 3], // Align to 16 bytes (GPU requirement)
}

impl HHConstants {
    pub fn new(num_neurons: usize, dt: f32) -> Self {
        Self {
            dt,
            num_neurons: num_neurons as u32,
            g_na_bar: 120.0,
            g_k_bar: 36.0,
            g_ca_bar: 5.0,
            g_leak: 0.3,
            e_na: 50.0,
            e_k: -90.0,
            e_ca: 120.0,
            e_leak: -70.0,
            capacitance: 1.0,
            _padding: [0; 3],
        }
    }
}

/// GPU neural simulator
pub struct GpuSimulator {
    device: Arc<Device>,
    queue: Arc<Queue>,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    neuron_buffer: wgpu::Buffer,
    constants_buffer: wgpu::Buffer,
    num_neurons: usize,
}

impl GpuSimulator {
    /// Initialize GPU simulator
    pub async fn new(num_neurons: usize, dt: f32) -> Result<Self> {
        // Request GPU adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find GPU adapter"))?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Neural Simulation GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Load compute shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hodgkin-Huxley Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/hodgkin_huxley.wgsl").into(),
            ),
        });

        // Create neuron state buffer (read-write storage)
        let initial_states = vec![GpuNeuronState::default(); num_neurons];
        let neuron_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Neuron State Buffer"),
            contents: bytemuck::cast_slice(&initial_states),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        // Create constants buffer (uniform)
        let constants = HHConstants::new(num_neurons, dt);
        let constants_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Constants Buffer"),
            contents: bytemuck::bytes_of(&constants),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[
                    // Neuron states (storage buffer)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Constants (uniform buffer)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: neuron_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: constants_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("HH Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "update_neurons",
        });

        Ok(Self {
            device,
            queue,
            compute_pipeline,
            bind_group,
            neuron_buffer,
            constants_buffer,
            num_neurons,
        })
    }

    /// Update neuron states (single time step)
    pub fn step(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("HH Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            // Dispatch workgroups (256 threads per workgroup)
            let workgroup_size = 256;
            let num_workgroups = (self.num_neurons + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Set external currents for all neurons
    pub fn set_currents(&self, currents: &[f32]) {
        assert_eq!(currents.len(), self.num_neurons);

        // Read current states
        let states = pollster::block_on(self.read_states());

        // Update external currents
        let updated: Vec<GpuNeuronState> = states
            .iter()
            .zip(currents.iter())
            .map(|(state, &current)| GpuNeuronState {
                external_current: current,
                ..*state
            })
            .collect();

        // Write back
        self.queue
            .write_buffer(&self.neuron_buffer, 0, bytemuck::cast_slice(&updated));
    }

    /// Read neuron states from GPU
    pub async fn read_states(&self) -> Vec<GpuNeuronState> {
        let buffer_size = (self.num_neurons * std::mem::size_of::<GpuNeuronState>()) as u64;

        // Create staging buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy GPU buffer to staging
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Read Encoder"),
            });

        encoder.copy_buffer_to_buffer(&self.neuron_buffer, 0, &staging_buffer, 0, buffer_size);

        self.queue.submit(Some(encoder.finish()));

        // Map buffer and read
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let states: Vec<GpuNeuronState> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

        states
    }

    /// Get voltages of all neurons
    pub async fn get_voltages(&self) -> Vec<f32> {
        self.read_states()
            .await
            .iter()
            .map(|s| s.voltage)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_simulator_creation() {
        let sim = pollster::block_on(GpuSimulator::new(1000, 0.01));
        assert!(sim.is_ok());
    }

    #[test]
    fn test_single_neuron_resting() {
        let sim = pollster::block_on(GpuSimulator::new(1, 0.01)).unwrap();

        // No input current - should stay near resting potential
        for _ in 0..100 {
            sim.step();
        }

        let voltages = pollster::block_on(sim.get_voltages());
        assert!(voltages[0] > -75.0 && voltages[0] < -65.0);
    }

    #[test]
    fn test_current_injection_spike() {
        let sim = pollster::block_on(GpuSimulator::new(1, 0.01)).unwrap();

        // Inject 100 pA - should spike
        sim.set_currents(&[100.0]);

        for _ in 0..100 {
            sim.step();
        }

        let states = pollster::block_on(sim.read_states());
        // Voltage should have increased significantly
        assert!(states[0].voltage > -60.0);
    }

    #[test]
    fn test_parallel_neurons() {
        let num_neurons = 10000;
        let sim = pollster::block_on(GpuSimulator::new(num_neurons, 0.01)).unwrap();

        // Different currents for each neuron
        let currents: Vec<f32> = (0..num_neurons)
            .map(|i| if i % 2 == 0 { 100.0 } else { 0.0 })
            .collect();

        sim.set_currents(&currents);

        for _ in 0..50 {
            sim.step();
        }

        let voltages = pollster::block_on(sim.get_voltages());

        // Even neurons (with current) should have higher voltage
        assert!(voltages[0] > voltages[1]);
    }
}
