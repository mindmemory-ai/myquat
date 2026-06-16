# Changelog

All notable changes to the MyQuat project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-06-13

### Initial Release

First official version. 1073 tests passing, `cargo publish` ready.

### Core Quantum Computing

- **Circuit system** — 45+ quantum gates (X, H, CX, Rx, Ry, Rz, U1-U3, CCX, SWAP, etc.), parameterized gates (`Float` / `Symbol`), conditional operations, measurement
- **Simulators** — State vector (up to ~30 qubits), density matrix, noisy simulator (Kraus channels, T1/T2 decoherence, Lindblad master equation)
- **Algorithms** — Grover's search, QFT, VQE, QAOA, Phase Estimation, UCCSD
- **Hamiltonian compilation** — Pauli Hamiltonian construction, Trotter-Suzuki decomposition (1st/2nd/4th/6th/Nth order), GateLevel / PauliLevel / PauliGadget / CrossStep compilation strategies
- **Fermion mappings** — Jordan-Wigner, Bravyi-Kitaev, Parity transformations
- **Symbolic computation** — Symbolica backend, parameter differentiation, symbolic Hamiltonian manipulation
- **QASM** — OpenQASM 2.0 / 3.0 import and export
- **Visualization** — ASCII art circuit diagrams, SVG/PNG export
- **Transpiler** — Device topology mapping, swap routing, hardware constraint validation

### Optimization Pipeline

5-level `PassManager` with 17 circuit passes:

| Level | Pipeline | Use Case |
|-------|----------|----------|
| 1 | MergeRotations → CancelInversePairs → SingleQubitOptimizer | Quick cleanup |
| 2 | SingleQubitOptimizer → CancelInversePairs → CNOTOptimizer → CommutativeCancellation → TemplateMatchingPass → SingleQubitOptimizer | Default production |
| 3 | Level 2 variant with deep CommutativeCancellation (lookahead 100/50) + extra SQOpt rounds | Aggressive |
| 4 | Level 2 core → BlockConsolidationPass(KAK) → PhasePolynomialPass(ReversibleRowCol) → Post-cleanup | Two-qubit block heavy |
| 5 | ConvergenceLoop(level_2+level_4_core, 20 iter) → PauliGadgetPass → CliffordSimplificationPass → PhasePolynomialPass(AdaptiveSynthesis) → Post-cleanup | Full production |

**Diagonal synthesis strategies (7):** ChainSynthesis, GrayCodeSynthesis, RowColSynthesis, ReversibleRowColSynthesis, ReversibleGrayCodeSynthesis, ParitySynthSynthesis, AdaptiveSynthesis.

**Circuit passes (17):** MergeRotationsPass, SingleQubitOptimizer, CancelInversePairsPass, CNOTOptimizer, CommutativeCancellationPass, TemplateMatchingPass, BlockConsolidationPass, PhasePolynomialPass, GlobalPhasePolynomialPass, CliffordSimplificationPass, PauliGadgetPass, TrotterAwarePass, GateFusionPass (disabled, ZYZ phase bug), ConvergenceLoopPass, TQEPass (placeholder), SwapRoutingPass, GlobalOptimizationPass.

### Deoptimization (Reverse Hamiltonian Extraction)

Experimental pipeline for extracting Hamiltonians from compiled circuits:
- Trotter template detection (1st/2nd order), KAK decomposition analysis, VQE/qDRIFT recognition
- Coefficient scaling with evolution_time and hbar
- 7 experiment binaries

### Infrastructure

- **Compute backends** — CPU, SIMD, parallel (Rayon), cloud (IBM Quantum, AWS Braket)
- **GPU acceleration** — Optional CUDA 12.0.50 support via `cudarc` (feature flag `cuda`)
- **Memory** — Memory pool, matrix cache, zero-copy operations
- **Linear algebra** — `ndarray` + `ndarray-linalg` (openblas-system), `nalgebra` (SVD/Schur)
- **Serialization** — serde/JSON for circuits and results
- **CLI** — `myquat-cli` binary (demo, bell, grover, qft, optimize, visualize)

### API Documentation

LaTeX-enhanced API docs across 16 modules: gate matrices, state evolution, algorithm steps, Hamiltonian formalism, error mitigation. 25 stable public modules, 3 expert modules.

### Key Metrics

- **H2_4q benchmark:** 375 gates, 152 CX, 193 depth, fidelity 0.999892
- **Cross-step 2nd order:** 100 gates at 0.997226 fidelity (78% reduction vs per-step)
- **1073 tests** passing, 0 failures

### Known Limitations

- GateFusionPass disabled (ZYZ global_phase makes it unsafe for multi-qubit circuits)
- TQEPass placeholder (needs full Clifford absorption pipeline)
- ZX→Y / XZ→Y templates excluded (phase correction needed)
- Release build requires system `libopenblas`
- 227 clippy warnings, 118 ignored tests (6 known pre-existing failures)

---

## Pre-Release Development

The project was under active development from early 2026 through June 2026 before this first formal release. Earlier version numbers found in git history (`0.0.1`, `0.2.0`, `1.1.0`, `1.2.0`) were internal placeholders and do not represent published releases.

### Major Development Phases

#### Phase 9 — Circuit Optimization (May–Jun 2026)

Extended CommutationChecker (CZ, CCX, controlled-rotation rules), CNOT symbolic templates (8 templates with distinctness constraint), GateFusion investigation, Shannon/KAK decomposition fixes, TemplateMatchingPass fix & wiring, TrotterAwarePass for inter-step CNOT cancellation, shared CNOT tree for Z-terms, iterative convergence loop, ZYZ theta=0 fix, cross-step 4th-order Suzuki coefficient fix, block reuse for Trotter symmetry.

#### Phase 10 — Phase Polynomial (Jun 2026)

PauliLevel alternate_reverse fix, GateFusionPass documentation, PhasePolynomialPass implementation (~900 lines): parity-based Rz merging in {CX, Rz} segments, 7 DiagonalSynthesis strategies, PassManager level_5 wiring, integration testing & multi-Hamiltonian validation.

#### Phase 11 — Clifford Absorption (Jun 2026)

CliffordSimplificationPass, PauliGadgetPass (forward-semantics Clifford absorption), RowColSynthesis, ReversibleRowColSynthesis, GrayCodeSynthesis frame-tracking fix, GreedyPauliSimp compilation strategy, AdaptiveSynthesis, level_5 production pipeline, TQE synthesis foundations.

#### Phase 12 — Forward-Semantics & TQE (Jun 2026)

Forward-semantics PauliGadgetPass with H absorption, CliffordSimplificationPass with tableau-based Clifford tail synthesis, AG symplectic tableau synthesis (Aaronson-Gottesman algorithm), TQE pass wiring. Achieved 375 gates, 152 CX at fidelity 0.999892.

#### Pre-Release Code Review (Jun 2026)

Discovered and fixed 23 critical bugs: ZYZ decomposition unification, SIMD CNOT, gate_expansion no-ops, diagonalisation angle factors, Suzuki coefficient sequence, openblas build chain, Grover optimal_iterations offset, noise channel formulas, and more.

---

## Versioning Convention

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible feature additions
- **PATCH**: Backward-compatible bug fixes

Change categories follow [Keep a Changelog](https://keepachangelog.com/): **Added**, **Changed**, **Deprecated**, **Removed**, **Fixed**, **Security**, **Performance**.
