# MyQuat Example Programs Index

**Version**: v1.0  
**Last Updated**: 2026-05-16

This document provides a categorized index of all MyQuat library example programs to help you quickly find the examples you need.

---

## Quick Navigation

- [Tutorial Examples](#tutorial-examples) - For beginners
- [Core Algorithms](#core-algorithms) - Classical quantum algorithms
- [Feature Demonstrations](#feature-demonstrations) - Library feature showcases
- [Hamiltonians](#hamiltonians) - Quantum chemistry and simulation
- [Quantum Chemistry](#quantum-chemistry) - Molecular simulation
- [Backend Integration](#backend-integration) - Cloud services and hardware
- [Advanced Features](#advanced-features) - Performance optimization
- [Quantum Mechanics](#quantum-mechanics) - Physics teaching
- [Basic Examples](#basic-examples) - Simple introductions
- [Teaching Demos](#teaching-demos) - Systematic tutorials

---

## Tutorial Examples

Introductory tutorials suitable for quantum computing beginners.

| File | Description | Difficulty |
|------|-------------|------------|
| `beginner_tutorial.rs` | Complete beginner tutorial, from qubits to simple algorithms | ⭐ |
| `interactive_tutorial.rs` | Interactive tutorial covering all major features | ⭐⭐ |
| `easy_api_demo.rs` | Easy API demonstration for quick start | ⭐ |

**Recommended Learning Order**: beginner_tutorial → interactive_tutorial → easy_api_demo

---

## Core Algorithms

Complete implementations of classical quantum algorithms.

### Shor's Algorithm
| File | Description | Key Concepts |
|------|-------------|--------------|
| `shor_algorithm.rs` | Quantum algorithm for integer factorization | Quantum Fourier Transform, period finding, RSA threat |

### Grover's Algorithm
| File | Description | Key Concepts |
|------|-------------|--------------|
| `grover_enhanced.rs` | Enhanced Grover search algorithm | Amplitude amplification, Oracle design, multi-target search |

### Variational Quantum Algorithms
| File | Description | Key Concepts |
|------|-------------|--------------|
| `vqe.rs` | Variational Quantum Eigensolver | UCCSD ansatz, energy optimization |
| `vqe_h2.rs` | VQE for H2 molecule ground state energy | Quantum chemistry, molecular Hamiltonian |
| `qaoa_maxcut.rs` | QAOA for MaxCut problem | Combinatorial optimization, hybrid algorithms |

### Quantum Fourier Transform
| File | Description | Key Concepts |
|------|-------------|--------------|
| `qft.rs` | Quantum Fourier Transform and applications | Phase estimation, period finding |
| `phase_estimation_demo.rs` | Quantum phase estimation algorithm | Eigenvalue estimation, precision control |

### Quantum Machine Learning
| File | Description | Key Concepts |
|------|-------------|--------------|
| `quantum_machine_learning.rs` | Quantum machine learning algorithm suite | Feature mapping, variational classifier, quantum kernels |
| `quantum_algorithms_demo.rs` | Comprehensive quantum algorithm demonstrations | Algorithm comparison, performance analysis |

---

## Feature Demonstrations

Demonstration programs for MyQuat library core features.

### Visualization
| File | Description | Features |
|------|-------------|----------|
| `visualization_demo.rs` | Complete circuit visualization demo | ASCII art, SVG export, statistical analysis |

### Error Handling
| File | Description | Features |
|------|-------------|----------|
| `error_handling_demo.rs` | Error handling system demonstration | Error classification, recovery strategies, statistical reporting |
| `error_mitigation_demo.rs` | Quantum error mitigation techniques | ZNE, symmetry verification, Richardson extrapolation |
| `noise_modeling_demo.rs` | NISQ device noise modeling | Depolarization, decoherence, measurement errors |

### Device and Topology
| File | Description | Features |
|------|-------------|----------|
| `device_topology_demo.rs` | Quantum device topology constraints | Connectivity graph, SWAP insertion, routing optimization |

### Memory and Performance
| File | Description | Features |
|------|-------------|----------|
| `memory_pool_demo.rs` | Memory pool management system | Object pooling, memory reuse |
| `memory_optimization_demo.rs` | Memory optimization techniques | Sparse representation, compressed storage |
| `performance_optimization.rs` | Comprehensive performance optimization | SIMD, parallelization, cache optimization |

### Circuit Optimization
| File | Description | Features |
|------|-------------|----------|
| `optimization_layers_demo.rs` | Multi-layer optimization system | Gate elimination, gate fusion, depth optimization |
| `adaptive_optimization_demo.rs` | Adaptive optimization strategies | Dynamic optimization, performance monitoring |

### Extended Features
| File | Description | Features |
|------|-------------|----------|
| `extended_gates_demo.rs` | Extended quantum gates demonstration | Custom gates, composite gates |
| `extended_qasm_demo.rs` | Extended QASM support | QASM 3.0, custom instructions |

---

## Hamiltonians

Quantum Hamiltonian construction, optimization, and simulation.

### Basic Features
| File | Description | Applications |
|------|-------------|--------------|
| `hamiltonian_demo.rs` | Basic Hamiltonian operations | Pauli strings, coefficient management |
| `hamiltonian_forward_demo.rs` | Forward Hamiltonian compilation | Trotter decomposition, circuit generation |
| `hamiltonian_backward_demo.rs` | Backward Hamiltonian extraction from circuits | Circuit analysis, pattern recognition |

### Optimization Techniques
| File | Description | Techniques |
|------|-------------|------------|
| `hamiltonian_optimization_demo.rs` | Hamiltonian optimization techniques | Pauli grouping, QWC optimization |
| `hamiltonian_optimizer_demo.rs` | Hamiltonian optimizer | Automatic optimization, performance analysis |

### Trotter Decomposition
| File | Description | Methods |
|------|-------------|---------|
| `adaptive_trotter_demo.rs` | Adaptive Trotter decomposition | Error control, step size optimization |
| `higher_order_trotter_demo.rs` | Higher-order Trotter methods | Suzuki formulas, precision improvement |
| `trotter_templates.rs` | Trotter template library | Predefined templates, rapid construction |

---

## Quantum Chemistry

Molecular simulation and quantum chemistry applications.

| File | Description | Molecular Systems |
|------|-------------|-------------------|
| `chemistry_demo.rs` | Basic quantum chemistry demonstration | General chemical systems |
| `quantum_chemistry_demo.rs` | Complete quantum chemistry workflow | Fermion transformation, VQE solving |
| `h2_full_comparison.rs` | Complete H2 molecule comparison | Method comparison, precision analysis |

---

## Backend Integration

Cloud services and hardware backend integration.

| File | Description | Backends |
|------|-------------|----------|
| `backend_integration_demo.rs` | Backend integration framework | Generic backend interface |
| `cloud_backend_demo.rs` | Cloud quantum computing backends | IBM Quantum, AWS Braket |
| `cloud_config_demo.rs` | Cloud service configuration management | Authentication, configuration, connection |
| `unified_cloud_demo.rs` | Unified cloud service interface | Multi-cloud support, automatic switching |

---

## Advanced Features

Advanced functionality and performance optimization.

### Circuit Deoptimization
| File | Description | Techniques |
|------|-------------|------------|
| `deoptimization_demo.rs` | Circuit deoptimization techniques | Pattern recognition, circuit restoration |

### Hardware Acceleration
| File | Description | Techniques |
|------|-------------|------------|
| `gpu_acceleration_demo.rs` | GPU acceleration demonstration | CUDA, OpenCL |
| `cuda_demo.rs` | CUDA-specific demonstration | NVIDIA GPU optimization |
| `simd_performance_demo.rs` | SIMD vectorization | AVX, NEON instruction sets |

### Symbolic Computation
| File | Description | Features |
|------|-------------|----------|
| `symbolic_hamiltonian_demo.rs` | Symbolic Hamiltonians | Symbolic expressions, parameterization |
| `symbolic_quantum_mechanics.rs` | Symbolic quantum mechanics | Analytical solutions, symbolic derivation |

---

## Quantum Mechanics

Quantum mechanics fundamentals and physics teaching.

### Basic Concepts
| File | Description | Physical Concepts |
|------|-------------|-------------------|
| `qm_entanglement.rs` | Quantum entanglement | Bell states, EPR pairs |
| `qm_spin_systems.rs` | Spin systems | Pauli matrices, spin operators |

### Quantum Systems
| File | Description | Systems |
|------|-------------|---------|
| `qm_harmonic_oscillator.rs` | Quantum harmonic oscillator | Energy levels, ladder operators |
| `qm_hydrogen_atom.rs` | Hydrogen atom | Atomic orbitals, energy spectrum |
| `qm_time_evolution.rs` | Time evolution | Schrödinger equation, dynamics |

### Advanced Topics
| File | Description | Topics |
|------|-------------|--------|
| `qm_stationary_states.rs` | Stationary states | Eigenstates, time-independent solutions |
| `qm_angular_momentum.rs` | Angular momentum | Rotation operators, spherical harmonics |
| `qm_perturbation_theory.rs` | Perturbation theory | First-order, second-order corrections |
| `qm_quantum_dynamics.rs` | Quantum dynamics | Heisenberg picture, interaction picture |

---

## Basic Examples

Simple introductory examples.

| File | Description | Concepts |
|------|-------------|----------|
| `bell_state.rs` | Bell state preparation | Entanglement basics |
| `quantum_collapse_demo.rs` | Quantum state collapse | Measurement, wavefunction collapse |
| `comprehensive_gates_demo.rs` | Comprehensive gate demonstration | All standard gates |
| `quantum_algorithms_demo.rs` | Quantum algorithms overview | Multiple algorithms |

---

## Teaching Demos

Systematic tutorial programs in the `teaching_demos/` subdirectory.

| File | Description | Duration | Difficulty |
|------|-------------|----------|------------|
| `01_bell_state.rs` | Bell state preparation | 5 min | ⭐ |
| `02_beginner_tutorial.rs` | Complete beginner tutorial | 15 min | ⭐⭐ |
| `03_comprehensive_gates.rs` | All quantum gates | 10 min | ⭐⭐ |
| `04_visualization.rs` | Circuit visualization | 10 min | ⭐⭐ |
| `05_hamiltonian_forward.rs` | Forward Hamiltonian compilation | 15 min | ⭐⭐⭐ |
| `06_hamiltonian_backward.rs` | Backward Hamiltonian extraction | 15 min | ⭐⭐⭐ |
| `07_grover_algorithm.rs` | Grover search algorithm | 15 min | ⭐⭐⭐ |
| `08_vqe_chemistry.rs` | VQE chemistry simulation | 20 min | ⭐⭐⭐⭐ |
| `09_interactive_tutorial.rs` | Interactive complete tutorial | 30 min | ⭐⭐⭐ |
| `10_adaptive_optimization.rs` | Adaptive circuit optimization | 15 min | ⭐⭐⭐ |

**See**: [`teaching_demos/README.md`](teaching_demos/README.md) for detailed information.

---

## Running Examples

### Run a Single Example

```bash
cargo run --example bell_state
cargo run --example grover_enhanced
cargo run --example vqe_h2
```

### Run with Release Mode (Faster)

```bash
cargo run --release --example quantum_machine_learning
```

### Run Teaching Demos

```bash
cargo run --example teaching_demos/01_bell_state
cargo run --example teaching_demos/08_vqe_chemistry
```

---

## Learning Paths

### Path 1: Quick Start (30 minutes)
**For**: New users wanting a quick overview of MyQuat

```
beginner_tutorial → visualization_demo → bell_state
```

### Path 2: Quantum Algorithms (1 hour)
**For**: Learners interested in quantum algorithms

```
beginner_tutorial → grover_enhanced → vqe_h2 → qaoa_maxcut
```

### Path 3: Hamiltonian Simulation (1.5 hours)
**For**: Chemistry and physics researchers

```
hamiltonian_demo → hamiltonian_forward_demo → 
hamiltonian_backward_demo → quantum_chemistry_demo
```

### Path 4: Complete Learning (3 hours)
**For**: Systematic quantum computing learning

```
Run all teaching demos in order: 01 → 02 → ... → 10
```

### Path 5: Advanced Features (2 hours)
**For**: Performance optimization and advanced usage

```
performance_optimization → gpu_acceleration_demo → 
deoptimization_demo → adaptive_optimization_demo
```

---

## Example Categories Summary

| Category | Count | Difficulty Range | Topics |
|----------|-------|------------------|--------|
| Tutorial Examples | 3 | ⭐-⭐⭐ | Basics, API, Quick start |
| Core Algorithms | 8 | ⭐⭐-⭐⭐⭐⭐ | Shor, Grover, VQE, QAOA, QML |
| Feature Demos | 12 | ⭐⭐-⭐⭐⭐ | Visualization, errors, optimization |
| Hamiltonians | 7 | ⭐⭐-⭐⭐⭐ | Trotter, forward/backward |
| Quantum Chemistry | 3 | ⭐⭐⭐-⭐⭐⭐⭐ | Molecules, VQE, chemistry |
| Backend Integration | 4 | ⭐⭐-⭐⭐⭐ | Cloud, IBM, AWS |
| Advanced Features | 6 | ⭐⭐⭐-⭐⭐⭐⭐ | GPU, CUDA, symbolic |
| Quantum Mechanics | 9 | ⭐⭐-⭐⭐⭐ | QM fundamentals, physics |
| Basic Examples | 4 | ⭐-⭐⭐ | Simple introductions |
| Teaching Demos | 10 | ⭐-⭐⭐⭐⭐ | Systematic tutorials |

**Total**: 69 example programs

---

## Tips

### For Beginners
1. Start with `beginner_tutorial.rs`
2. Try modifying parameters and observe changes
3. Read code comments carefully
4. Use visualization to understand circuits

### For Researchers
1. Focus on Hamiltonian and chemistry examples
2. Study optimization techniques
3. Explore symbolic computation features
4. Test with your own molecular systems

### For Developers
1. Examine advanced features and optimizations
2. Study backend integration patterns
3. Learn from error handling implementations
4. Explore GPU acceleration techniques

---

## Additional Resources

- **API Documentation**: Run `cargo doc --open`
- **Best Practices**: See [`../docs/BEST_PRACTICES.md`](../docs/BEST_PRACTICES.md)
- **Theory**: See [`../docs/hamiltonian_circuit_theory/`](../docs/hamiltonian_circuit_theory/)
- **GitHub**: https://github.com/mindmemory-ai/myquat

---

## Contributing Examples

To add a new example:

1. Create file in `examples/your_example.rs`
2. Add file header with author and description
3. Include complete runnable code
4. Add comments explaining key steps
5. Update this index file
6. Test with `cargo run --example your_example`

---

**Happy Learning!** 🎉
