# HumanBrain: Technical Implementation Guide

Maximum-detail technical documentation for developers and researchers.

## Architecture Overview

**Modular Rust crates with GPU acceleration**

### GPU Module ()

#### Cable Simulator ()
- wgpu compute shaders (WGSL)
- 152 compartments per neuron
- Tree topology buffers
- Forward Euler integration (dt=0.025-0.05ms)
- Performance: 80 FPS for 10K neurons on RTX 3050

#### Feedback Loop ()
- Adaptive homeostatic control
- Voltage history buffer (VecDeque<f32>, 10K samples)
- Attractor analysis integration
- Regime-specific parameter mapping
- Smooth transitions (smoothing factor 0.9)

### Whole-Brain Integration ()

**8 Anatomical Pathways:**
1. Thalamocortical
2. Corticothalamic  
3. Corticostriatal
4. Pallidothalamic
5. Hippocampal-Cortical
6. Cortico-Cortical
7. Thalamo-Striatal
8. Subthalamo-Pallidal

**Layer Extraction**: Real filtering by LayerType enum, soma voltage (compartment 0)

### Analysis Module ()

- Correlation dimension D₂ (Grassberger-Procaccia)
- Lyapunov exponents λ₁ (Rosenstein)
- FFT dominant frequency
- Regime classification

### Visualization ()

- wgpu rendering pipeline
- Compartment-level vertices
- Physical voltage-to-color mapping
- Orbit camera controls
- Real-time updates

## Usage Examples

See repository for complete code examples.

## Performance Tuning

RTX 3050: 10K neurons optimal  
RTX 4070: 25K neurons feasible  
Multi-GPU: Future roadmap

## References

See SCIENTIFIC_PAPER.md for citations.

---

© 2025 Francisco Molina Burgos
