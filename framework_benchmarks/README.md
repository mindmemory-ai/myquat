# Quantum Framework Baseline Tests

Comprehensive baseline comparison of MyQuat (Rust) against five Python quantum computing frameworks.

## Frameworks Tested

| Framework | Language | Status |
|-----------|----------|--------|
| MyQuat    | Rust     | via compiled binary |
| Qiskit    | Python   | raw + optimized (level 3) |
| Cirq      | Python   | PauliStringPhasor |
| TKET      | Python   | pytket circuit builder |
| PennyLane | Python   | TrotterProduct |
| Paulihedral | Python | Pauli optimization |

## Quick Start

```bash
# 1. Activate conda environment (all Python frameworks pre-installed)
conda activate quantum

# 2. Build MyQuat benchmark binary
cd .. && cargo build --release --example benchmark_hamiltonian && cd framework_benchmarks

# 3. Run baseline
python run_baseline.py
```

## Test Dimensions

### A. Hamiltonian Compilation
Trotter decomposition of Pauli Hamiltonians into gate circuits.
- 12 problems: H2, LiH, Heisenberg, TFIM, Random (4–10 qubits, 15–40 terms)
- 5 configs: Trotter orders {1, 2} × steps {10, 50, 100}
- Metrics: compilation time (ms), gate count, circuit depth

## Output

```
results/
├── baseline_YYYYMMDD_HHMMSS.csv        # raw data
├── baseline_YYYYMMDD_HHMMSS.json       # full results  
└── baseline_YYYYMMDD_HHMMSS_report.md  # formatted summary
```

## Archive

Previous benchmark code and results (January 2026) are archived in `archive_202601/`.
