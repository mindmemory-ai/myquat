# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build
cargo build --release
cargo build --features cuda        # GPU support (requires CUDA 12.0.50)

# Test
cargo test                          # All tests
cargo test <substring>              # Single test by name (substring match)
cargo test --lib                    # Library unit tests only (~754 tests)
cargo test --test comprehensive_gates_test   # Specific integration test
cargo test --test qasm_interoperability      # QASM import/export tests
cargo test --test fermion_hamiltonian_tests  # Fermion/Hamiltonian tests

# Lint & Format
cargo clippy -- -D warnings
cargo fmt
cargo fmt -- --check               # CI-style check

# Docs
cargo doc --open                    # Build and open API docs

# Benchmarks (Criterion, harness=false)
cargo bench
cargo bench --bench gate_benchmarks
cargo bench --bench fermion_transform_benchmarks

# Run examples
cargo run --example bell_state
cargo run --example grover_enhanced
cargo run --example shor_algorithm

# Hamiltonian benchmark (used for cross-framework comparison)
cargo run --example benchmark_hamiltonian -- 4 10 1.0 1 IIII:-0.81 IIIZ:0.17 ...

# Experiments (reverse Hamiltonian extraction)
cargo run --bin exp01_trotter_detection
cargo run --bin exp02_hamiltonian_extraction
cargo run --bin exp03_deoptimization_pipeline

# CLI binary
cargo run --bin myquat-cli -- <command>       # Commands: demo, bell, grover, qft, optimize, visualize

# Release build failure workaround: libopenblas may not be available on WSL
# Use debug builds for the benchmark binary:
cargo build --example benchmark_hamiltonian
```

## External Dependency

The `mymat` integration was planned but never implemented — it is not a dependency.

## Feature Flags

- `cuda` — enables `cudarc` GPU acceleration (requires CUDA 12.0.50 toolkit)
- No other feature flags; `reqwest` (cloud backends) is always included

---

# Current Work Context (Phase 10: Final Polish & New Features)

## Branch & Git State

- **Current branch:** `quantum-mechanics-solver`
- **Base branch:** `master`
- **HEAD:** `6cf5a24` — "Phase 10c: phase polynomial pass — parity-based Rz merging in PassManager level_5"
- **Git user:** gA4ss
- **Working tree:** clean

### Phase 10 Completed

| Phase | Description | Status |
|-------|-------------|--------|
| 10a | PauliLevel alternate_reverse fix — forward/reverse passes use identical QWC block structure | ✅ Committed (`737df8e`) |
| 10b | GateFusionPass wiring documentation — deferred due to ZYZ global_phase bug | ✅ Committed (`83b8a13`) |
| 10c | Phase polynomial module (18 tests): Parity-based Rz merging, GrayCodeSynthesis, ChainSynthesis, PassManager::level_5() | ✅ Committed (`6cf5a24`) |
| 10d | Integration testing & baseline regeneration | ✅ This commit |

## Goal (Phases 9-10)

Close the gate-count gap between MyQuat and TKET in Hamiltonian compilation benchmarks.

### Phase 9 Completed (9a-9q)

Extended CommutationChecker, symbolic CNOT templates, GateFusion investigation, Shannon fixes, BlockConsolidation fixes, TemplateMatchingPass fix & wiring, TrotterAwarePass, shared CNOT tree + alternate reverse, iterative convergence loop, ZYZ theta=0 fix, cross-step analysis (1st/2nd/4th order), Suzuki coefficient fix, cross-step block reuse.

### Phase 10 Completed (10a-10d)

1. **10a: PauliLevel alternate_reverse fix** — `is_reverse` flag now wired in `compile_step_pauli_synthesis`. When true and no cache, uses `reverse_blocks()` for Trotter symmetry.
2. **10b: GateFusionPass wiring** — Documented as deferred. ZYZ global_phase bug makes it unsafe (`Rz(λ+α) ≠ e^{iα}·Rz(λ)`). Fix requires U3-based decomposition or per-circuit phase tracking.
3. **10c: Phase polynomial module** (`src/phase_polynomial.rs`, ~900 lines) — Parity-based Rz merging in `{CX, Rz}` circuit segments. `DiagonalSynthesis` trait with `ChainSynthesis` and `GrayCodeSynthesis` strategies. `PhasePolynomialPass` (CircuitPass) wired into `PassManager::level_5()`. 18 unit tests pass.
4. **10d: Integration testing** — Full test suite (881 pass, 6 pre-existing failures), H2_4q benchmark (449 gates confirmed), strategy comparison (no regressions), multi-Hamiltonian validation (Heisenberg improvements confirmed).

## Baseline Experiment History

**Experiment framework:** `experiments/framework_benchmarks/run_baseline.py`
Compares MyQuat, Qiskit, Qiskit_Opt, Cirq, TKET, PennyLane, Paulihedral across 12 Hamiltonians × 5 configs = 420 tests.

**Key:** Accuracy is Frobenius norm error normalized by √(dim). Value ~1.4 = essentially random unitary vs target. Value ~0.0 = perfect match.

| Baseline | Date | MyQuat Gates | MyQuat Acc. | TKET Gates | Note |
|----------|------|-------------|-------------|-----------|------|
| 170405 | May 18 17:04 | 12,011 | 1.38 | — | First baseline (240 tests, 4 FW) |
| 215818 | May 18 21:58 | 11,833 | 1.37 | 5,006 | First with all 7 frameworks |
| **012456** | **May 19 01:24** | **7,859** | **1.19** | **5,006** | **ANOMALY — old binary, pre-PauliLevel** |
| 144124 | May 19 14:41 | 11,500 | 1.38 | 5,006 | TemplateMatching regression |
| 152125 | May 19 15:21 | 11,500 | 1.38 | 5,006 | Still regressed |
| 175818 | May 19 17:58 | 12,020 | 1.37 | 5,006 | After reverting circuit_optimization.rs |

**The REAL baseline is ~12,000 gates, ~1.37 accuracy.** TKET consistently achieves 5,006 gates.

### Per-Problem Reference (H2_4q, 10 steps, order 1)

| Code Version | Gates | CX | Depth |
|-------------|-------|-----|-------|
| Phase 9o (GateLevel + opt) | 449 | 190 | 231 |
| Phase 9o (GateLevel RAW) | 466 | 190 | 240 |
| Phase 9o (PauliLevel + opt) | 500 | 200 | 263 |
| 1-step RAW (either strategy) | 50 | 20 | — |
| TKET | 329 | — | 200 |
| **Gap vs TKET** | **120 (27%)** | | **31** |

### Cross-Step Results (ignores trotter_steps — single step of specified order)

| Order | Gates | CX | Depth | Fidelity |
|-------|-------|-----|-------|----------|
| 1st | 50 | 20 | 24 | 0.964368 |
| 2nd | 100 | 40 | 51 | 0.997226 |
| 4th | 500 | 200 | 272 | 0.999984 |

## Key Architecture for Current Work

### Hamiltonian Compilation Flow

```
Hamiltonian
  → HamiltonianCompiler::compile()
    → Trotter decomposition (1st/2nd/4th/6th/Nth order)
      → apply_trotter_step_cached()
        ├── PauliLevel: pauli_synthesis::compile_step_pauli_synthesis()
        │     (shared CNOT trees, QWC block merging, inter-block adjacency)
        └── GateLevel: compile_pauli_group()
              (fixed CNOT ladder, grouped commuting terms)
    → Post-synthesis optimization:
      ├── PassManager::level_2()  (always)
      └── PassManager::level_4()  (if gates > 50)
```

### Optimization Pipeline (PassManager levels)

```
level_1: MergeRotations → CancelInversePairs → SingleQubitOptimizer
level_2: SingleQubitOptimizer → CancelInversePairs → CNOTOptimizer → CommutativeCancellation → SingleQubitOptimizer
level_3: Like level_2 but with CommutativeCancellation lookahead(100) + extra passes
level_4: SingleQubitOptimizer → CancelInversePairs → CNOTOptimizer → CommutativeCancellation → BlockConsolidationPass(KAK) → CNOTOptimizer → CommutativeCancellation → SingleQubitOptimizer
```

**TemplateMatchingPass EXISTS but is NOT wired into any level** (only used in tests, lines 1430, 1449, 1832, 1890 of circuit_optimization.rs).

### CNOT Templates (current committed state)

Only 4 hardcoded templates matching specific qubit indices (0,1,2):
1. Inverse pair (handled separately)
2. CX(0,1)·CX(1,0)·CX(0,1) → CX(1,0)
3. CX(0,1)·CX(1,2)·CX(0,1)·CX(1,2) → CX(0,2)
4. CX(0,1)·CX(2,1)·CX(0,1) → CX(2,1)
5. CX(1,0)·CX(1,2)·CX(1,0) → CX(1,2)

Phase 9 planned to add 8 symbolic templates with arbitrary qubit matching — these need to be re-implemented.

### Key Files

| File | Purpose |
|------|---------|
| `src/cnot_optimizer.rs` | CNOT template matching (hardcoded indices only), CNOTOptimizer, AdjacencyCache |
| `src/circuit_optimization.rs` | PassManager levels 1-5, TemplateMatchingPass, CommutationChecker, BlockConsolidationPass, CommutativeCancellationPass, CancelInversePairsPass |
| `src/hamiltonian/hamiltonian_compiler.rs` | HamiltonianCompiler, CompilerConfig, CompilationStrategy, Trotter decomposition |
| `src/hamiltonian/pauli_synthesis.rs` | compile_step_pauli_synthesis (shared CNOT trees, QWC merging), reverse_blocks() |
| `src/phase_polynomial.rs` | **NEW (Phase 10c):** Parity-based Rz merging, DiagonalSynthesis trait, ChainSynthesis, GrayCodeSynthesis, PhasePolynomialPass |
| `src/single_qubit_optimizer.rs` | SingleQubitOptimizer (merges consecutive rotations) |
| `examples/benchmark_hamiltonian.rs` | Standalone binary for cross-framework benchmarking |
| `examples/compare_strategies.rs` | Head-to-head comparison of all synthesis strategies |
| `examples/multi_hamiltonian_validate.rs` | Multi-Hamiltonian regression validation |
| `experiments/framework_benchmarks/run_baseline.py` | Python script orchestrating 7-framework comparison |

## Known Bugs & Gotchas

### API Stability Contract (P2, 2026-06-10)

**🟢 Stable modules** (semver-guaranteed for 1.x):
`circuit`, `gates`, `gates_extended`, `parameter`, `simulator`, `density_matrix`,
`noisy_simulator`, `measurement_stats`, `conditional`, `hamiltonian`, `deoptimization`,
`algorithms`, `error_mitigation`, `qm_solver`, `device_topology`, `hardware_constraints`,
`noise_models`, `transpiler`, `error`, `qasm`, `visualization`, `easy_api`,
`quantum_info`, `benchmarks`, `regression_detector`, `symbolic`, `adaptive_optimizer`

**🟡 Expert modules** (public, APIs may evolve in minor releases):
`circuit_optimization`, `circuit_optimizer`, `phase_polynomial`
— Prefer `PassManager::level_N()` or `HamiltonianCompiler::compile()` over direct imports.

**🔴 Internal modules** (no semver guarantees, pub only for examples/tests):
All others including `cnot_optimizer`, `single_qubit_optimizer`, `two_qubit_decompose`,
`clifford_tableau`, `parity_synth`, `gate_decomposition`, `circuit_analyzer`, etc.

**CompilerConfig fields** (18 fields, all `pub`):
- 8 stable: `trotter_order`, `trotter_steps`, `evolution_time`, `hbar`, `optimization_strategy`,
  `group_commuting_terms`, `apply_circuit_optimization`, `skip_identities`
- 5 experimental: `cross_step_synthesis`, `alternate_reverse_steps`, `clifford_enhanced_blocks`,
  `block_grouping_strategy`, `pauli_gadget_optimization`
- 5 expert: `adaptive`, `adaptive_tolerance`, `min_step_size`, `max_step_size`,
  `auto_optimize_grouping`, `layout_aware_grouping`

### Known Bugs (current as of 2026-06-10)

### GrayCodeSynthesis: frame-tracking bug — FIXED (Phase 11h, verified 2026-06-10)

The frame-tracking bug was fixed in Phase 11h. The implementation now maintains a
proper frame→qubit mapping throughout the Gray-code traversal. Tests confirm:
- `test_reversible_graycode_restores_frame` — frame returns to identity after reverse
- `test_reversible_graycode_scattered_frame_restoration` — scattered parities with backtrack
- `test_graycode_multibit_edge` — Hamming distance 4 edges decomposed into correct CNOT steps
- `test_rowcol_vs_graycode_prefix_parities` / `test_rowcol_vs_graycode_scattered_parities` — comparison

`ReversibleGrayCodeSynthesis` IS included in `AdaptiveSynthesis::with_default_strategies()`.

### TemplateMatchingPass: ZX→Y and XZ→Y global phase bug — PREVENTIVE NOTE
The ZX→Y and XZ→Y templates were NEVER added to the codebase (verified 2026-06-10).
`standard_templates()` contains only 3 safe templates: H-CX-H→CZ, CX-H-CX→H-CZ, S-H-S→H-Sdg.
This note is retained as a warning: if these templates are added in the future, they need
phase correction (Z·X = i·Y, X·Z = -i·Y).

### GateFusion: ZYZ decomposition global_phase — FIXED, DISABLED BY DESIGN
The original bug (dropping `global_phase` entirely) is fixed — the phase is now absorbed
into the first Rz gate. However, `Rz(λ+α) ≠ e^{iα}·Rz(λ)` makes the absorption mathematically
approximate for multi-qubit circuits. GateFusionPass is intentionally NOT wired into any
PassManager level. Fixing this "properly" requires U3-based decomposition or per-circuit
global_phase tracking. Low priority — GateFusion was never in the default optimization path.

### CNOT symbolic templates: distinctness enforcement — FIXED (verified 2026-06-10)
`SymCNOTTemplate::try_match()` enforces distinctness at lines 486-496: different variable
names must bind to DIFFERENT qubit indices. The `try_bind_qref` function correctly tracks
all bindings in a HashMap and rejects overlaps.

### Release build fails on WSL
`cargo build --release` fails with missing `libopenblas` (from `ndarray-linalg` with
`openblas-system` feature). Debug builds work fine. Workaround: install `libopenblas-dev`
on WSL, or use `cargo build` (debug) for examples. Not a code bug — environment constraint.

### CNOTOptimizer::commutes_with — VERIFIED CORRECT (Phase 9m)
Two CNOTs with the same target but different controls DO commute:
```
CX(a,b)·CX(c,b) = CX(c,b)·CX(a,b)  when a ≠ c
```
XOR is commutative (x⊕a⊕b = x⊕b⊕a), so the target qubit's final state
is the same regardless of CNOT order. `commute_reorder` is disabled for
other reasons (O(N²) cost, limited benefit for Trotter circuits).

### CNOTOptimizer→SingleQubitOptimizer interaction — RESOLVED (Phase 9g)
**Previous diagnosis:** CNOTOpt immediately followed by SQOpt corrupts circuits.
**Corrected finding:** CNOTOpt→SQOpt adjacency is SAFE (pi_dist = 0.0 in all tests).
The original corruption was from BC pass bugs (reversed gate emission, broken pattern fallback)
fixed in Phase 9d/9e. The Phase 9f workaround (extra CancelInversePairs between CNOTOpt and SQOpt)
was unnecessary and has been removed in Phase 9g.

## Next Steps for Phase 9

### Completed (Phase 9a-9n)
1. Extended CommutationChecker (CZ, CCX, controlled-rotation rules) ✅
2. CNOT symbolic templates (8 templates, distinctness constraint) ✅
3. GateFusion global phase fix (disabled by default, ZYZ introduces Ry gates) ✅
4. Shannon decomposition fixes (ZYZ phase, tensor product, 1-CX case) ✅
5. add_gate_to_circuit extended to full StandardGate enum ✅
6. level_4 pipeline corruption fix → simplified post-BC cleanup ✅
7. CNOTOpt→SQOpt root cause analysis (adjacency SAFE, workaround removed) ✅
8. TemplateMatchingPass fixed (gate copier) and wired into level_2/level_4 ✅
9. Phase 9i: Full baseline regeneration against TKET ✅
10. Phase 9j: TrotterAwarePass for inter-step CNOT cancellation ✅
11. Phase 9k: Shared CNOT tree for Z-terms + alternate reverse steps ✅
12. Phase 9l: Iterative convergence loop + default strategy → GateLevel ✅
13. Phase 9m: ZYZ theta=0 fix + GateFusion diagonal optimization ✅
14. Phase 9n: Cross-step higher-order analysis (2nd order = 100 gates, 0.997) ✅
15. Phase 9o: Fix cross-step 4th-order Suzuki coefficient sequence ✅
16. Phase 9p: Refactor cross-step synthesis — extract form_blocks_from_scaled/synthesize_blocks, reuse forward block structure for reverse pass to guarantee Trotter symmetry ✅

### Current Results (H2_4q, 10 steps, 1st order, Phase 10d)

| Strategy | Gates | CX | Depth | Fidelity |
|----------|-------|-----|-------|----------|
| GateLevel + opt | 449 | 190 | 231 | 0.999892 |
| PauliLevel + opt | 500 | 200 | 264 | 0.999892 |
| Cross-step 1st + opt | 50 | 20 | 27 | 0.964368 |
| Cross-step 2nd + opt | 100 | 40 | 54 | 0.997226 |
| Cross-step 4th + opt | 500 | 200 | 259 | 0.999984 |
| **TKET (target)** | **329** | — | **200** | — |
| **Gap vs TKET** | **120 (27%)** | | **31** | |

- `alternate_reverse_steps` saves 51 gates (10%): 500→449 gates, 200→190 CX.
  Without it, optimization has ZERO effect on gate count.
- Optimization converges in 1 iteration for H2_4q.
- Cross-step 2nd order is the sweet spot: 78% fewer gates than per-step (100 vs 449)
  at 0.997 fidelity.
- Cross-step 4th order fixed (Phase 9o): Suzuki formula S_4(t) = [S_2(p·t)]²·S_2((1-4p)·t)·[S_2(p·t)]²
  requires coefficient sequence [p, p, 1-4p, p, p], NOT [p, 1-4p, p, 1-4p, p].
  The buggy sequence gave 3p+2(1-4p) = -0.073 instead of 4p+(1-4p) = 1.0.
  4th order now achieves 0.999984 fidelity. Gate count is higher than 1st/2nd order
  because 4th-order Trotter requires 10× more exponentials (5 S_2 blocks × 2 passes).

## Remaining Work (Phase 10+)

1. **Phase polynomial / ZX-calculus synthesis** — The remaining 27% gap to TKET requires
   global CNOT-network optimization. For K4 (H2_4q), Z-term CNOT is already optimal at
   12 CX/step. The gap is in single-qubit gate management and XY-term optimization.
   Cross-step 2nd order (100 gates, 0.997 fidelity) is 78% better than per-step.
   `PhasePolynomialPass` provides the foundation (parity-based Rz merging) but doesn't
   yet re-synthesize CNOT networks via GrayCodeSynthesis.
2. **GateFusion for Trotter** — Adapted (Phase 9m) but not wired into pipeline.
   ZYZ global_phase bug (`Rz(λ+α) ≠ e^{iα}·Rz(λ)`) makes it unsafe.
   Won't help Trotter circuits significantly (Rz gates isolated between CX/H gates),
   but useful for general circuit optimization.
3. **GateLevel depth vs PauliLevel** — GateLevel wins on gate count but loses on
   depth for larger circuits. PauliLevel's shared trees give better parallelism.
4. **PauliLevel alternate_reverse** ✅ FIXED (Phase 10a) — `is_reverse` flag now functional.
5. **Enable PhasePolynomialPass by default** — Currently experimental (`level_5()`).
   Need more validation on diverse circuits before promoting to `level_2()`/`level_4()`.

## Completed (Phases 9a-9q + 10a-10d)

1. Extended CommutationChecker (CZ, CCX, controlled-rotation rules) ✅
2. CNOT symbolic templates (8 templates, distinctness constraint) ✅
3. GateFusion global phase investigation (deferred due to ZYZ bug) ✅
4. Shannon decomposition fixes (ZYZ phase, tensor product, 1-CX case) ✅
5. add_gate_to_circuit extended to full StandardGate enum ✅
6. level_4 pipeline corruption fix → simplified post-BC cleanup ✅
7. CNOTOpt→SQOpt root cause analysis (adjacency SAFE, workaround removed) ✅
8. TemplateMatchingPass fixed (gate copier) and wired into level_2/level_4 ✅
9. Full baseline regeneration against TKET ✅
10. TrotterAwarePass for inter-step CNOT cancellation ✅
11. Shared CNOT tree for Z-terms + alternate reverse steps ✅
12. Iterative convergence loop + default strategy → GateLevel ✅
13. ZYZ theta=0 fix + GateFusion diagonal optimization ✅
14. Cross-step higher-order analysis (2nd order = 100 gates, 0.997) ✅
15. Fix cross-step 4th-order Suzuki coefficient sequence ✅
16. Refactor cross-step synthesis for block reuse and Trotter symmetry ✅
17. PauliLevel alternate_reverse fix ✅
18. GateFusionPass wiring investigation & documentation ✅
19. Phase polynomial module (parity-based Rz merging + GrayCodeSynthesis) ✅
20. Integration testing & baseline regeneration ✅

### Multi-Hamiltonian Validation (Phase 10d, 10 steps, 1st order)

| Hamiltonian | GateLevel | GL CX | PauliLevel | PL CX | GL Depth | PL Depth |
|-------------|-----------|-------|------------|-------|----------|----------|
| H2 (4q, 15 terms) | 449 | 190 | 500 | 200 | 231 | 275 |
| Heisenberg-4 (12 terms) | 411 | 162 | 438 | 180 | 255 | 282 |
| Heisenberg-6 (18 terms) | 695 | 282 | 716 | 298 | 355 | 312 |
| TFIM-4 (7 terms) | 138 | 52 | 150 | 60 | 93 | 90 |
| TFIM-6 (11 terms) | 228 | 92 | 234 | 96 | 153 | 93 |

GateLevel is **universally better for gate count** (2.7-24.0% over PauliLevel).
PauliLevel is **better for circuit depth** on larger/structured Hamiltonians
due to shared CNOT tree parallelism.
Heisenberg-4 improved 24% (541→411) and Heisenberg-6 16% (825→695)
from Phase 9o-9q Suzuki fix + block reuse.

### Quick test command
```bash
# H2_4q benchmark (expected: 449 gates, 190 CX, 231 depth, 0.999892 fidelity)
cargo run --example benchmark_hamiltonian -- 4 10 1.0 1 IIII:-0.8105 IIIZ:0.1721 IIZI:-0.2228 IZII:0.1721 ZIII:-0.2228 IIZZ:0.1686 IZIZ:0.1205 IZZI:0.1686 ZIIZ:0.1686 ZIZI:0.1205 ZZII:0.1686 IIXX:0.0454 IIYY:0.0454 XXII:0.0454 YYII:0.0454 2>/dev/null

# Strategy comparison (all strategies + fidelity check)
cargo run --example compare_strategies 2>/dev/null

# Multi-Hamiltonian validation
cargo run --example multi_hamiltonian_validate 2>/dev/null

# Phase polynomial tests
cargo test --lib phase_polynomial
```
Target: TKET's 329 gates (27% gap remains). Cross-step 2nd order: 100 gates, 0.997 fidelity.

---

# Project Architecture (reference)

## Public API Pattern

Nearly all types are re-exported at the crate root (`src/lib.rs`). Consumers use `use myquat::*` or import specific types directly (e.g., `myquat::QuantumCircuit`, `myquat::StateVectorSimulator`). When adding a new public type, re-export it from `lib.rs` following the existing convention.

## Module Map

**Circuit & Gates** (`src/circuit.rs`, `src/gates.rs`, `src/gates_extended.rs`, `src/parameter.rs`)
- `QuantumCircuit` is the central data structure. Gates are applied via methods like `circuit.h(qubit)`, `circuit.cx(ctrl, tgt)`, `circuit.ry(qubit, param)`.
- `Parameter` enum supports both `Float(f64)` and `Symbol(String)` variants backed by `symbolica`.
- Extended gate infrastructure: `gate_decomposition.rs` (decompose complex gates into primitive sets), `gate_library.rs` (pre-built gate collections), `gate_inverse.rs` (compute gate inverses), `custom_gate_matrix.rs` (user-defined gates from arbitrary unitaries), `gate_expansion.rs` (expand gates to matrix representations).

**Simulation** (`src/simulator.rs`, `src/density_matrix.rs`, `src/noisy_simulator.rs`)
- `StateVectorSimulator` — pure-state simulation, supports up to ~30 qubits on CPU.
- `DensityMatrixSimulator` — mixed-state / open systems.
- `NoisyQuantumSimulator` — wraps a `DeviceNoiseModel` for NISQ-realistic execution.

**Optimization Pipeline** (`src/circuit_optimizer.rs`, `src/circuit_optimization.rs`, `src/optimization_passes.rs`, `src/adaptive_optimizer.rs`, `src/transpiler.rs`)
- `PassManager` in `circuit_optimization.rs` runs multi-pass optimization (levels 1–4).
- `AdaptiveOptimizer` selects passes based on circuit structure via `CircuitAnalyzer`.
- `Transpiler` maps logical circuits to hardware using `DeviceTopology` constraints.
- Sub-optimizers: `CNOTOptimizer` (cnot_optimizer.rs), `SingleQubitOptimizer` (single_qubit_optimizer.rs), `TwoQubitDecompose` (two_qubit_decompose.rs — KAK decomposition), `two_qubit_synthesis.rs`.

**Deoptimization / Reverse Extraction** (`src/deoptimization/`)
- Reverse Hamiltonian Extraction pipeline: given a circuit, recovers the underlying Hamiltonian.
- `template.rs` — Trotter template matching; `trotter_template.rs` — template detection.
- `kak.rs` / `kak_math.rs` — KAK decomposition analysis for reverse engineering.
- `qdrift_strategy.rs` — qDRIFT detection; `vqe_templates.rs` — VQE ansatz recognition.
- `temporal.rs` — temporal pattern analysis; `pauli_basis.rs` — Pauli basis decomposition.
- The `experiments/` directory contains 7 bin targets exercising this pipeline end-to-end.

**Hamiltonian Module** (`src/hamiltonian/`)
- `hamiltonian.rs` — Pauli Hamiltonians (`PauliHamiltonian`) and Pauli strings (`PauliString`).
- `hamiltonian_compiler.rs` — Trotter decomposition (1st through n-th order) into circuits.
- `circuit_analyzer.rs` — Detects Trotter structure in circuits (reverse extraction).
- `fermion.rs` — Fermionic operators and Jordan-Wigner/Bravyi-Kitaev mappings.
- `optimizer.rs` — Hamiltonian-level optimization and layout-aware grouping.
- `pauli_synthesis.rs` — Pauli-level synthesis with shared CNOT trees and QWC merging.
- `layout_aware_grouping.rs` — Layout-aware term ordering for topology-constrained hardware.
- `symbolic_hamiltonian.rs` / `symbolic_compiler.rs` — Symbolic Hamiltonian manipulation.

**QM Solver** (`src/qm_solver/`)
- Solves TISE (`tise_solver.rs`) and TDSE (`tdse_solver.rs`) problems numerically.
- Supports perturbation theory (`perturbation.rs`), angular momentum (`angular_momentum.rs`), multi-particle systems (`multi_particle.rs`).
- `hilbert_space.rs` and `operators.rs` define the mathematical foundation.
- `quantum_chemistry.rs` — HF/CI/CC methods; `dynamics.rs` — Schrodinger/Heisenberg/Interaction pictures.
- `numerical_methods.rs` — shared numerical routines.

**Algorithms** (`src/algorithms/`)
- Ready-to-use: `qft.rs` (QFT), `grover.rs` + `grover_oracle.rs` (Grover's search), `vqe.rs` + `vqe_core.rs` (VQE), `qaoa.rs` (QAOA), `phase_estimation.rs`.
- `combinatorial.rs` — MaxCut, TSP; `uccsd.rs` — unitary coupled cluster; `optimizer.rs` — parameter optimization with gradient descent and parameter shift rules.

**Compute Backends** (`src/compute/`)
- `backend_manager.rs` auto-selects CPU / SIMD / parallel / GPU / cloud based on circuit size.
- `simd_ops.rs` — SIMD-accelerated matrix operations; `parallel_ops.rs` — Rayon-based parallelism.
- `local/` — CPU backend implementations; `cloud/` — IBM Quantum and AWS Braket backends.
- `types.rs` — shared types (`ComputeBackendType`, `SelectionStrategy`, `ExecutionHints`).

**Symbolic** (`src/symbolic/`)
- Two backends: `symbolica` (primary, via `symbolica_adapter.rs`) and `MySym` (via `mysym_adapter.rs`).
- `factory.rs` — backend creation and auto-detection; `config.rs` — symbolic configuration.
- Used for parametric circuit differentiation and symbolic Hamiltonian manipulation.

**Simplified / Teaching API** (`src/easy_api.rs`, `src/conditional.rs`)
- `EasyCircuit`, `EasySimulator`, `EasyAlgorithms`, `EasyAnalysis` — simplified wrappers for beginners.
- `ClassicalCondition`, `ConditionalGate`, `ConditionalCircuit` — classically-controlled quantum operations.

**Visualization** (`src/visualization.rs`)
- `CircuitVisualizer` — ASCII art, SVG export, configurable wire/gate/color styles.

**Error Mitigation** (`src/error_mitigation.rs`)
- `ZeroNoiseExtrapolation` (ZNE) and `SymmetryVerification`.

**Serialization** (`src/qasm.rs`)
- OpenQASM 2.0 and 3.0 import/export for Qiskit interoperability.

**Performance/Memory Infrastructure**
- `circuit_optimized.rs` — `OptimizedCircuitData`, `CompactInstruction` for memory-efficient circuit storage.
- `memory_optimized.rs` — `MemoryEfficientState`, `ZeroCopyMatrixOps`.
- `memory_pool.rs` — `QuantumMemoryPool` for array reuse across simulations.
- `matrix_cache.rs` — global gate matrix cache with `MatrixCacheKey`.
- `performance_config.rs` — `PerformanceManager` for auto-tuning based on problem size.

**Supporting Modules**
- `error.rs` — `MyQuatError` enum (uses `thiserror`) and `Result<T>` alias. All fallible public APIs return this.
- `utils.rs`, `measurement_stats.rs`, `noise_models.rs`, `device_topology.rs`, `quantum_info.rs`.
- `regression_detector.rs` — performance regression detection for benchmarks.
- `hardware_constraints.rs` — validates circuits against hardware constraints.

## Key Type Hierarchy

```
QuantumCircuit
  └── Vec<Instruction>         (gate + qubit targets)
        └── Gate enum           (45+ variants)
              └── Parameter     (Float(f64) | Symbol(String))

StateVectorSimulator
  └── ndarray::Array1<Complex<f64>>   (state vector)

PauliHamiltonian
  └── Vec<PauliTerm>
        └── PauliString + coefficient

ComputeBackend (trait)          (unified backend interface)
  ├── CpuBackend
  ├── SimdBackend
  ├── ParallelBackend
  └── CloudBackend              (IBM Quantum, AWS Braket)
```

## Crate Structure

- **Library**: `src/lib.rs` → crate `myquat` (all modules public, re-exported at root)
- **CLI binary**: `src/bin/main.rs` → `myquat-cli`
- **Experiments**: 7 bin targets in `experiments/reverse_hamiltonian_extraction/`
- **Examples**: 10 teaching demos in `examples/teaching_demos/` (registered in Cargo.toml) + ~45 standalone examples in `examples/`
- **Integration tests**: 15 files in `tests/`
- **Benchmarks**: 2 Criterion benchmarks in `benches/` (both require `harness = false`)
- **Minimal supported Rust version**: 1.79

## Key Git Commits (recent, on quantum-mechanics-solver)

```
6cf5a24 Phase 10c: phase polynomial pass — parity-based Rz merging in PassManager level_5
43d09c5 Phase 10c: phase polynomial module — core types, DiagonalSynthesis trait, ChainSynthesis, GrayCodeSynthesis
83b8a13 Phase 10b: document GateFusionPass wiring deferred due to ZYZ global_phase bug
737df8e Phase 10a: fix PauliLevel alternate_reverse — forward/reverse passes use identical QWC block structure
1f39bd7 Phase 9q: extract reverse_blocks helper, fix doc comment, add unit tests
7157ef8 Phase 9p: refactor cross-step synthesis for block reuse and Trotter symmetry
0563643 fix: phase_insensitive_dist computes e^{i*angle} instead of 0+i*angle
ef919c5 Phase 9o: fix cross-step 4th-order Suzuki coefficient sequence
28f1b56 docs: multi-Hamiltonian validation confirms GateLevel universally better
9bbc890 docs: update CLAUDE.md with Phase 9n cross-step higher-order analysis
c3e3e93 feat: KAK decomposition block consolidation for post-synthesis optimization
ca2e80b feat: Pauli-level circuit synthesis with QWC block merging and shared CNOT tree (REGRESSION)
e74c121 feat: CNOTOptimizer fixes, research directory, and optimization design (LAST GOOD)
```
