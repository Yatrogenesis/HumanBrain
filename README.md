# HumanBrain: GPU-Accelerated Multi-Scale Human Brain Simulator

A computational neuroscience simulation framework implemented in Rust, with GPU-accelerated compartmental dynamics and multi-regional anatomical connectivity models.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.17720224.svg)](https://doi.org/10.5281/zenodo.17720224)
[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![GPU](https://img.shields.io/badge/GPU-wgpu%20%2F%20Vulkan-green.svg)](https://wgpu.rs)

## Overview

HumanBrain is a research framework for multi-scale computational neuroscience modeling:

- **GPU-Accelerated Multi-Compartmental Neurons** (wgpu compute shaders, 152 compartments/neuron)
- **Multi-Regional Anatomical Pathways** (8 literature-derived inter-regional pathways based on anatomical connectivity references)
- **Attractor-Based Dynamics Analysis** (correlation dimension D₂, Lyapunov exponents λ₁)
- **Adaptive Feedback Control** (homeostatic plasticity, activity-dependent regulation)
- **Metabolic Constraints** (ATP, glucose, oxygen with neurovascular coupling models)
- **Glial Cell Dynamics** (astrocytes, oligodendrocytes, microglia models)
- **Layer-Specific Cortical Connectivity** (anatomically grounded architectural templates)
- **Real-Time 3D Visualization** (physical voltage mapping with camera controls)

**Philosophy**: *"No quiero suficiencia, quiero realidad"* - Rigorous biophysical modeling without unfounded claims.

---

## Hardware Requirements

### Exploratory Configuration (Single-run observation): HP Victus 15
- **CPU**: Intel i7-12th gen H-series
- **RAM**: 16 GB
- **GPU**: NVIDIA GeForce RTX 3050 (4GB VRAM, Ampere, Vulkan 1.3)
- **Performance Target**: Exploratory configuration for 10,000 neurons (scale=0.1; systematic benchmarking suite pending)

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
- [Scientific_Paper.md](docs/Scientific_Paper.md) - Computational neuroscience modeling description
- [Technical_Paper.md](docs/Technical_Paper.md) - Implementation guide
- [Biological_Accuracy.md](docs/Biological_Accuracy.md) - Model fidelity analysis and literature parameter comparisons (unverified experimental validation)

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

## License

HumanBrain is licensed under the **GNU Affero General Public License v3.0 or later** (`AGPL-3.0-or-later`, see `LICENSE`).
Earlier tagged releases (for example v1.0.0) were published under MIT; those published copies keep the license they were
released with, while development from this point on is under the AGPL. Commercial licensing: fmolina@avermex.com.

## Contact

- **Author**: Francisco Molina Burgos (pako.molina@gmail.com)
- **ORCID**: 0009-0008-6093-8267
- **GitHub**: github.com/Yatrogenesis/HumanBrain

---

**"No quiero suficiencia, quiero realidad."**

*HumanBrain - Where physics meets biology at GPU speed.*
