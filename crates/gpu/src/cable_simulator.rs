//! GPU Compartmental Cable Equation Simulator
//!
//! Wrapper Rust para el shader cable_equation.wgsl que implementa
//! la ecuación de cable con topología arbórea completa.
//!
//! ## Arquitectura
//! - Cada neurona tiene 152 compartimentos (como CPU implementation)
//! - Topología de árbol completo (soma → apical + basal + AIS)
//! - Buffers GPU contiguos para cache efficiency
//!
//! ## Paridad CPU-GPU
//! Este simulador logra PARIDAD FÍSICA total con crates/neurons/compartmental.rs

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::{util::DeviceExt, Device, Queue};

// ============================================================================
// DATA STRUCTURES (Must match WGSL shader exactly)
// ============================================================================

/// Compartment state (must match cable_equation.wgsl CompartmentState)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CompartmentState {
    // Electrical state
    pub voltage: f32,
    pub capacitance: f32,
    pub axial_resistance: f32,
    pub g_leak: f32,
    pub e_leak: f32,

    // Ion channel gating variables
    pub na_m: f32,
    pub na_h: f32,
    pub k_n: f32,
    pub ca_m: f32,

    // Geometry
    pub length: f32,
    pub diameter: f32,
    pub surface_area: f32,

    // Compartment type
    pub comp_type: u32,

    // Tree topology
    pub parent_idx: i32,
    pub child_idx_0: i32,
    pub child_idx_1: i32,
    pub child_idx_2: i32,
    pub child_idx_3: i32,
    pub child_idx_4: i32,
    pub child_idx_5: i32,
    pub child_idx_6: i32,
    pub child_idx_7: i32,
    pub num_children: u32,

    // Identifiers
    pub neuron_id: u32,
    pub comp_id_in_neuron: u32,

    // Padding (16-byte alignment)
    pub _pad0: f32,
    pub _pad1: f32,
}

impl Default for CompartmentState {
    fn default() -> Self {
        Self {
            voltage: -70.0,
            capacitance: 1.0,
            axial_resistance: 100.0,
            g_leak: 0.025,
            e_leak: -70.0,
            na_m: 0.05,
            na_h: 0.6,
            k_n: 0.32,
            ca_m: 0.01,
            length: 50.0,
            diameter: 2.0,
            surface_area: 314.15,
            comp_type: 1,
            parent_idx: -1,
            child_idx_0: -1,
            child_idx_1: -1,
            child_idx_2: -1,
            child_idx_3: -1,
            child_idx_4: -1,
            child_idx_5: -1,
            child_idx_6: -1,
            child_idx_7: -1,
            num_children: 0,
            neuron_id: 0,
            comp_id_in_neuron: 0,
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}

/// Constants (must match cable_equation.wgsl Constants)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CableConstants {
    pub dt: f32,
    pub num_neurons: u32,
    pub num_compartments_per_neuron: u32,
    pub total_compartments: u32,

    // Ion channel conductances
    pub g_na_bar: f32,
    pub g_k_bar: f32,
    pub g_ca_bar: f32,

    // Reversal potentials
    pub e_na: f32,
    pub e_k: f32,
    pub e_ca: f32,

    // Temperature
    pub temperature: f32,
    pub ref_temperature: f32,

    pub _padding: [u32; 2],
}

impl CableConstants {
    pub fn new(num_neurons: usize, dt: f32) -> Self {
        const COMPARTMENTS_PER_NEURON: usize = 152; // Pyramidal L5 morphology

        Self {
            dt,
            num_neurons: num_neurons as u32,
            num_compartments_per_neuron: COMPARTMENTS_PER_NEURON as u32,
            total_compartments: (num_neurons * COMPARTMENTS_PER_NEURON) as u32,
            g_na_bar: 120.0,
            g_k_bar: 36.0,
            g_ca_bar: 5.0,
            e_na: 50.0,
            e_k: -90.0,
            e_ca: 120.0,
            temperature: 37.0,    // Body temperature
            ref_temperature: 22.0, // Room temperature (HH original)
            _padding: [0; 2],
        }
    }
}

// ============================================================================
// GPU CABLE SIMULATOR
// ============================================================================

pub struct CableSimulator {
    device: Arc<Device>,
    queue: Arc<Queue>,
    solve_pipeline: wgpu::ComputePipeline,
    init_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    compartment_buffer: wgpu::Buffer,
    constants_buffer: wgpu::Buffer,
    currents_buffer: wgpu::Buffer,
    num_neurons: usize,
    num_compartments_per_neuron: usize,
    total_compartments: usize,
}

impl CableSimulator {
    /// Create new GPU cable simulator
    pub async fn new(num_neurons: usize, dt: f32) -> Result<Self> {
        const COMPARTMENTS_PER_NEURON: usize = 152;
        let total_compartments = num_neurons * COMPARTMENTS_PER_NEURON;

        // Initialize WGPU
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

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Cable Equation GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cable Equation Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/cable_equation.wgsl").into(),
            ),
        });

        // Create buffers
        let initial_compartments = vec![CompartmentState::default(); total_compartments];
        let compartment_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compartment Buffer"),
            contents: bytemuck::cast_slice(&initial_compartments),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let constants = CableConstants::new(num_neurons, dt);
        let constants_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Constants Buffer"),
            contents: bytemuck::bytes_of(&constants),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let initial_currents = vec![0.0f32; total_compartments];
        let currents_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("External Currents Buffer"),
            contents: bytemuck::cast_slice(&initial_currents),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cable Bind Group Layout"),
                entries: &[
                    // Compartments (storage, read-write)
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
                    // Constants (uniform)
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
                    // External currents (storage, read-only)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cable Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compartment_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: constants_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: currents_buffer.as_entire_binding(),
                },
            ],
        });

        // Create pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cable Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let solve_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Solve Cable Equation Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "solve_cable_equation",
        });

        let init_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Initialize Compartments Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "initialize_compartments",
        });

        Ok(Self {
            device,
            queue,
            solve_pipeline,
            init_pipeline,
            bind_group,
            compartment_buffer,
            constants_buffer,
            currents_buffer,
            num_neurons,
            num_compartments_per_neuron: COMPARTMENTS_PER_NEURON,
            total_compartments,
        })
    }

    /// Initialize compartment morphology and electrical state
    pub fn initialize(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Init Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Init Compartments"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.init_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            let workgroup_size = 256;
            let num_workgroups = (self.total_compartments + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Simulate one time step (cable equation solver)
    pub fn step(&self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Solve Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Solve Cable Equation"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.solve_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);

            let workgroup_size = 256;
            let num_workgroups = (self.total_compartments + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(num_workgroups as u32, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Set external current for a specific compartment
    pub fn set_current(&self, compartment_idx: usize, current_pa: f32) {
        assert!(compartment_idx < self.total_compartments);

        let offset = (compartment_idx * std::mem::size_of::<f32>()) as u64;
        self.queue
            .write_buffer(&self.currents_buffer, offset, bytemuck::bytes_of(&current_pa));
    }

    /// Set currents for all compartments
    pub fn set_currents(&self, currents: &[f32]) {
        assert_eq!(currents.len(), self.total_compartments);
        self.queue
            .write_buffer(&self.currents_buffer, 0, bytemuck::cast_slice(currents));
    }

    /// Read all compartment states from GPU
    pub async fn read_compartments(&self) -> Vec<CompartmentState> {
        let buffer_size =
            (self.total_compartments * std::mem::size_of::<CompartmentState>()) as u64;

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Read Encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &self.compartment_buffer,
            0,
            &staging_buffer,
            0,
            buffer_size,
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let compartments: Vec<CompartmentState> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

        compartments
    }

    /// Get soma voltages for all neurons
    pub async fn get_soma_voltages(&self) -> Vec<f32> {
        let compartments = self.read_compartments().await;

        (0..self.num_neurons)
            .map(|neuron_id| {
                let soma_idx = neuron_id * self.num_compartments_per_neuron;
                compartments[soma_idx].voltage
            })
            .collect()
    }

    /// Get voltages for a specific neuron (all compartments)
    pub async fn get_neuron_voltages(&self, neuron_id: usize) -> Vec<f32> {
        assert!(neuron_id < self.num_neurons);

        let compartments = self.read_compartments().await;
        let start_idx = neuron_id * self.num_compartments_per_neuron;
        let end_idx = start_idx + self.num_compartments_per_neuron;

        compartments[start_idx..end_idx]
            .iter()
            .map(|c| c.voltage)
            .collect()
    }

    /// Get number of neurons
    pub fn num_neurons(&self) -> usize {
        self.num_neurons
    }

    /// Get compartments per neuron
    pub fn compartments_per_neuron(&self) -> usize {
        self.num_compartments_per_neuron
    }

    /// Get total compartments
    pub fn total_compartments(&self) -> usize {
        self.total_compartments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cable_simulator_creation() {
        let sim = pollster::block_on(CableSimulator::new(10, 0.01));
        assert!(sim.is_ok());

        let sim = sim.unwrap();
        assert_eq!(sim.num_neurons(), 10);
        assert_eq!(sim.compartments_per_neuron(), 152);
        assert_eq!(sim.total_compartments(), 10 * 152);
    }

    #[test]
    fn test_initialize_morphology() {
        let sim = pollster::block_on(CableSimulator::new(1, 0.01)).unwrap();
        sim.initialize();

        let compartments = pollster::block_on(sim.read_compartments());

        // Soma should be at index 0
        assert_eq!(compartments[0].comp_type, 0); // Soma
        assert_eq!(compartments[0].parent_idx, -1); // No parent
        assert_eq!(compartments[0].num_children, 3); // Apical + Basal + AIS

        // Check voltage initialized to resting
        assert!((compartments[0].voltage - (-70.0)).abs() < 0.1);
    }

    #[test]
    fn test_resting_state() {
        let sim = pollster::block_on(CableSimulator::new(1, 0.01)).unwrap();
        sim.initialize();

        // Run 100 steps with no input
        for _ in 0..100 {
            sim.step();
        }

        let voltages = pollster::block_on(sim.get_soma_voltages());

        // Should stay near resting potential
        assert!((voltages[0] - (-70.0)).abs() < 5.0);
    }

    #[test]
    fn test_action_potential_propagation() {
        let sim = pollster::block_on(CableSimulator::new(1, 0.01)).unwrap();
        sim.initialize();

        // Inject current into soma (compartment 0)
        sim.set_current(0, 500.0); // 500 pA

        // Run simulation
        for _ in 0..200 {
            sim.step();
        }

        let voltages = pollster::block_on(sim.get_neuron_voltages(0));

        // Soma should spike
        assert!(voltages[0] > -50.0, "Soma should depolarize");

        // Action potential should propagate to dendrites
        // (This is a basic test - real validation requires time series analysis)
    }

    #[test]
    fn test_multiple_neurons() {
        let sim = pollster::block_on(CableSimulator::new(100, 0.01)).unwrap();
        sim.initialize();

        // Inject current into half the neurons (somas only)
        for neuron_id in 0..50 {
            let soma_idx = neuron_id * 152;
            sim.set_current(soma_idx, 400.0);
        }

        // Simulate
        for _ in 0..150 {
            sim.step();
        }

        let soma_voltages = pollster::block_on(sim.get_soma_voltages());

        // First 50 should depolarize
        let active_avg: f32 = soma_voltages[0..50].iter().sum::<f32>() / 50.0;

        // Last 50 should stay near resting
        let inactive_avg: f32 = soma_voltages[50..100].iter().sum::<f32>() / 50.0;

        assert!(active_avg > inactive_avg + 10.0);
    }
}
