# MyQuat — Quantum Computing Library in Rust

[![Crates.io](https://img.shields.io/crates/v/myquat.svg)](https://crates.io/crates/myquat)
[![Rust](https://img.shields.io/badge/rust-1.79%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/myquat/badge.svg)](https://docs.rs/myquat)

**Author:** gA4ss &nbsp;|&nbsp; **Language:** Rust &nbsp;|&nbsp; **Version:** 0.2.0

A high-performance quantum computing simulation and development library. MyQuat
provides a complete stack from basic gates to advanced algorithms, with
Hamiltonian compilation, cloud backends, GPU acceleration, and Qiskit
interoperability.

---

## Quick Start

```toml
[dependencies]
myquat = "0.2.0"

# Optional: GPU acceleration
myquat = { version = "0.2.0", features = ["cuda"] }
```

```rust
use myquat::{QuantumCircuit, StateVectorSimulator};

fn main() -> myquat::Result<()> {
    // Create Bell state: (|00⟩ + |11⟩)/√2
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;

    let simulator = StateVectorSimulator::new();
    let result = simulator.run(&circuit, 1000)?;
    println!("{:?}", result.counts());
    // {"00": ~500, "11": ~500}
    Ok(())
}
```

```bash
# Run examples
cargo run --example bell_state
cargo run --example grover_enhanced
cargo run --example benchmark_hamiltonian -- 4 10 1.0 1 IIII:-0.8105 IIIZ:0.1721 ...
```

---

## Core Features

### Quantum Circuit System
- 45+ quantum gates — standard, parameterized, and controlled
- Flexible circuit construction with conditional operations and measurement
- **5-level optimization pipeline** — `PassManager::level_1()` through `level_5()`
  - 17 circuit passes: merge rotations, CNOT optimization, commutative cancellation,
    template matching, KAK block consolidation, phase polynomial, Clifford
    simplification, Pauli gadget, and more
  - 7 diagonal synthesis strategies (Chain, GrayCode, RowCol, Adaptive)
- Visualization: ASCII art diagrams, SVG/PNG export

### Hamiltonian Compilation
- Pauli Hamiltonian construction with full operator algebra
- Trotter-Suzuki decomposition (1st/2nd/4th/6th/Nth order)
- Multiple compilation strategies: GateLevel, PauliLevel, PauliGadget, CrossStep
- Shared CNOT tree synthesis with QWC block merging
- Reverse Hamiltonian extraction (circuit → Hamiltonian)

### Quantum Simulators
- **State vector** — pure-state simulation up to ~30 qubits on CPU
- **Density matrix** — mixed states and open quantum systems
- **Noisy simulator** — T1/T2 decoherence, depolarizing, Lindblad equation
- SIMD acceleration (2-4×), parallel computing (Rayon), memory pools

### Quantum Algorithms
- **VQE** — Variational Quantum Eigensolver for chemistry
- **QAOA** — Quantum Approximate Optimization (MaxCut, TSP)
- **Grover's search**, **QFT**, **Phase Estimation**, **UCCSD**
- Parameter optimization: gradient descent, parameter shift rules
- Fermion-to-qubit mappings: Jordan-Wigner, Bravyi-Kitaev, Parity

### NISQ & Hardware
- Device topology modeling and SWAP routing
- Error mitigation: Zero-Noise Extrapolation (ZNE), symmetry verification
- Noise models: T1/T2, gate errors, readout errors

### QM Solver
- TISE/TDSE numerical solvers
- Perturbation theory (non-degenerate, degenerate, time-dependent)
- Angular momentum (orbital, spin, coupling, spherical harmonics)
- Quantum dynamics: Schrödinger, Heisenberg, Interaction pictures

### Cloud & GPU
- **IBM Quantum** and **AWS Braket** cloud backends
- Optional **CUDA** GPU acceleration (10-50× for 20+ qubits)
- Intelligent CPU/GPU backend selection

### Interoperability
- OpenQASM 2.0 / 3.0 import and export
- Qiskit ↔ MyQuat circuit conversion
- Symbolic computation via Symbolica backend

### Compute Backend Architecture
- Local: CPU, SIMD, Parallel (Rayon), GPU (CUDA)
- Cloud: IBM Quantum, AWS Braket
- Auto-selection based on circuit size

---

## Optimization Pipeline

| Level | Passes | Use Case |
|-------|--------|----------|
| 1 | MergeRotations → CancelInverse → SQOpt | Quick cleanup |
| 2 | SQOpt → CancelInverse → CNOTOpt → CommCancel → TemplateMatch → SQOpt | Default production |
| 3 | Level 2 + deep CommCancel (lookahead 100/50) | Aggressive |
| 4 | Level 2 core → BlockConsolidation(KAK) → PhasePoly(ReversibleRowCol) → Post-cleanup | Two-qubit block heavy |
| 5 | ConvergenceLoop → PauliGadget → CliffordSimple → PhasePoly(Adaptive) → Post-cleanup | Full production / benchmarks |

---

## Performance

### H₂ (4 qubits) — Hamiltonian Compilation Benchmark

| Metric | MyQuat (level_5) | TKET |
|--------|-------------------|------|
| Total gates | 375 | 329 |
| CX gates | 152 | ~200 |
| Circuit depth | 193 | 200 |
| Fidelity | 0.999892 | — |

### Cross-Step Synthesis (H₂, 10 Trotter steps)

| Order | Gates | CX | Depth | Fidelity |
|-------|-------|-----|-------|----------|
| 1st | 50 | 20 | 27 | 0.964368 |
| 2nd | 100 | 40 | 54 | 0.997226 |
| 4th | 500 | 200 | 259 | 0.999984 |

---

## Documentation

- **[API Documentation](https://docs.rs/myquat)** — Full crate docs on docs.rs
- **[examples/](examples/)** — 45+ working examples
- **[docs/tutorials/](docs/tutorials/)** — Tutorial guides
- **[docs/theory/](docs/theory/)** — Theoretical background
- **[docs/superpowers/](docs/superpowers/)** — Design specs and implementation plans

---

## Development

```bash
# Build
cargo build

# Test (1,141+ passing)
cargo test --lib

# Lint & format
cargo clippy -- -D warnings
cargo fmt -- --check

# Docs
cargo doc --open

# Benchmarks
cargo bench
```

---

## Project Stats

- **Code:** ~40,000 lines of Rust across 127 source files
- **Tests:** 1,141+ passing, 0 failing
- **Modules:** 45+ public modules, 25 stable (semver-guaranteed)
- **Examples:** 45+ working examples
- **Minimal Rust version:** 1.79

---

## Contributing

Contributions are welcome! Please read:

- **[CONTRIBUTING.md](CONTRIBUTING.md)** — Development workflow, commit conventions, testing guide
- **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** — Community standards
- **[SECURITY.md](SECURITY.md)** — Vulnerability reporting

---

## License

Apache 2.0 — see [LICENSE](LICENSE) for details.

---

## Citation

```bibtex
@software{myquat2026,
  author = {gA4ss},
  title = {MyQuat: Quantum Computing Library in Rust},
  year = {2026},
  version = {0.2.0},
  url = {https://github.com/mindmemory-ai/myquat}
}
```

---

## Contact

- **Author:** gA4ss
- **Email:** logic.yan@me.com
