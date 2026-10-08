# HumanBrain: Scientific Paper

Complete documentation paper omitted for brevity - see main implementation in HumanBrain repository.

## Summary

HumanBrain explores GPU-accelerated multi-compartmental neurons, literature-referenced anatomical connectivity, and adaptive feedback control.

Key architectural features:
- wgpu compute shaders for cable equation dynamics (152 comp/neuron)
- 8 literature-referenced inter-regional pathways
- Hybrid CPU-GPU attractor analysis feedback loop
- Metabolic and glial constraint models

Performance target: 10K neurons (~50-80 FPS exploratory run on RTX 3050; benchmark suite pending)

See repository for full technical details.
