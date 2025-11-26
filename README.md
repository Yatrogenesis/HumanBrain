# HumanBrain: World-Class Anatomically Realistic Human Brain Simulator

A production-grade biologically realistic human brain simulator implemented in Rust, with GPU-accelerated physics and complete anatomical integration addressing fundamental limitations of existing brain simulation approaches.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.17720224.svg)](https://doi.org/10.5281/zenodo.17720224)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![GPU](https://img.shields.io/badge/GPU-wgpu%20%2F%20Vulkan-green.svg)](https://wgpu.rs)

## Overview

HumanBrain represents a paradigm shift in computational neuroscience: **no reduccionismos, no simplismos** - complete biological realism at maximum level. This simulator integrates:

- **GPU-Accelerated Multi-Compartmental Neurons** (wgpu compute shaders, 152 compartments/neuron)
- **Complete Anatomical Integration** (8 biologically validated inter-regional pathways)
- **Attractor-Based Dynamics Analysis** (correlation dimension D₂, Lyapunov exponents λ₁)
- **Adaptive Feedback Control** (homeostatic plasticity, activity-dependent regulation)
- **Metabolic Constraints** (ATP, glucose, oxygen with neurovascular coupling)
- **Glial Cell Dynamics** (astrocytes, oligodendrocytes, microglia)
- **Layer-Specific Cortical Connectivity** (anatomically grounded, no placeholders)
- **Real-Time 3D Visualization** (physical voltage mapping with camera controls)

**Philosophy**: *"No quiero suficiencia, quiero realidad"* - World-class quality without shortcuts.

---

## Hardware Requirements

### [OK] Tested Configuration: HP Victus 15
- **CPU**: Intel i7-12th gen H-series
- **RAM**: 16 GB
- **GPU**: NVIDIA GeForce RTX 3050 (4GB VRAM, Ampere, Vulkan 1.3)
- **Performance**: 10,000 neurons @ 50-80 FPS (scale=0.1)

### Minimum Specifications
- **CPU**: 4 cores (Intel i5-8th gen / AMD Ryzen 3000+)
- **RAM**: 8 GB
- **GPU**: Vulkan 1.1+ (Intel UHD 630, NVIDIA GTX 1650, AMD RX 5500)

### Recommended for Research
- **CPU**: 8+ cores
- **RAM**: 16+ GB
- **GPU**: 4+ GB VRAM (RTX 3050+, RX 6600+)

---

## Quick Start

```bash
git clone https://github.com/yatrogenesis/HumanBrain.git
cd HumanBrain
cargo build --release
cargo test --all
```

---

## Documentation

**For complete documentation, see**:
- [SCIENTIFIC_PAPER.md](docs/SCIENTIFIC_PAPER.md) - Full computational neuroscience paper
- [TECHNICAL_PAPER.md](docs/TECHNICAL_PAPER.md) - Maximum-detail implementation guide
- [BIOLOGICAL_ACCURACY.md](docs/BIOLOGICAL_ACCURACY.md) - Validation against experimental data

---

## Citation

```bibtex
@software{humanbrain2025,
  title = {HumanBrain: GPU-Accelerated Anatomically Realistic Brain Simulator},
  author = {Molina Burgos, Francisco},
  year = {2025},
  url = {https://github.com/yatrogenesis/HumanBrain},
  version = {1.0.0}
}
```

---

## Contact

- **Author**: Francisco Molina Burgos (pako.molina@gmail.com)
- **ORCID**: 0009-0008-6093-8267
- **GitHub**: github.com/Yatrogenesis/HumanBrain

---

**"No quiero suficiencia, quiero realidad."**

*HumanBrain - Where physics meets biology at GPU speed.*
