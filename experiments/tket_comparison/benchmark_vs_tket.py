#!/usr/bin/env python3
"""MyQuat vs TKET benchmarking for Hamiltonian compilation.

Runs the same Hamiltonians through TKET's PauliSimp + FullPeepholeOptimise
pipeline and compares gate counts, depth, and fidelity with MyQuat.

Outputs JSON for consumption by `examples/gap_analysis.rs`.

Usage:
    python3 benchmark_vs_tket.py [--steps 10] [--output results.json]
"""

import json
import math
import sys
import time
from typing import List, Dict, Tuple

import numpy as np
from pytket import Circuit, OpType
from pytket.passes import (
    PauliSimp,
    GreedyPauliSimp,
    FullPeepholeOptimise,
    CliffordSimp,
    SynthesiseTket,
)
from pytket.partition import PauliPartitionStrat
from pytket.transform import Transform
from pytket.utils import get_operator_expectation_value


# ── Hamiltonian definitions (matching MyQuat's benchmark_hamiltonian.rs) ──────

HAMILTONIANS = {
    "H2_4q": {
        "num_qubits": 4,
        "terms": [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ],
    },
    "Heisenberg-4": {
        "num_qubits": 4,
        "terms": [
            ("XXII", 1.0), ("YYII", 1.0), ("ZZII", 1.0),
            ("IXXI", 1.0), ("IYYI", 1.0), ("IZZI", 1.0),
            ("IIXX", 1.0), ("IIYY", 1.0), ("IIZZ", 1.0),
            ("XIXI", 1.0), ("YIYI", 1.0), ("ZIZI", 1.0),
        ],
    },
    "Heisenberg-6": {
        "num_qubits": 6,
        "terms": [
            ("XXIIII", 1.0), ("YYIIII", 1.0), ("ZZIIII", 1.0),
            ("IXXIII", 1.0), ("IYYIII", 1.0), ("IZZIII", 1.0),
            ("IIXXII", 1.0), ("IIYYII", 1.0), ("IIZZII", 1.0),
            ("IIIXXI", 1.0), ("IIIYYI", 1.0), ("IIIZZI", 1.0),
            ("IIIIXX", 1.0), ("IIIIYY", 1.0), ("IIIIZZ", 1.0),
            ("XIIXII", 1.0), ("YIIYII", 1.0), ("ZIIZII", 1.0),
        ],
    },
    "TFIM-4": {
        "num_qubits": 4,
        "terms": [
            ("ZZII", 1.0), ("IZZI", 1.0), ("IIZZ", 1.0),
            ("XIII", 1.0), ("IXII", 1.0), ("IIXI", 1.0), ("IIIX", 1.0),
        ],
    },
    "TFIM-6": {
        "num_qubits": 6,
        "terms": [
            ("ZZIIII", 1.0), ("IZZIII", 1.0), ("IIZZII", 1.0),
            ("IIIZZI", 1.0), ("IIIIZZ", 1.0),
            ("XIIIII", 1.0), ("IXIIII", 1.0), ("IIXIII", 1.0),
            ("IIIXII", 1.0), ("IIIIXI", 1.0), ("IIIIIX", 1.0),
        ],
    },
}


def pauli_string_to_qubits(ps: str) -> List[Tuple[int, str]]:
    """Convert Pauli string like 'XXII' to list of (qubit, op) pairs."""
    result = []
    for i, c in enumerate(ps):
        if c != 'I':
            result.append((i, c))
    return result


def build_trotter_circuit(
    hamiltonian: Dict, dt: float, steps: int, order: int = 1
) -> Circuit:
    """Build a Trotter circuit for the given Hamiltonian."""
    n = hamiltonian["num_qubits"]
    circ = Circuit(n)

    for _ in range(steps):
        for ps_str, coeff in hamiltonian["terms"]:
            angle = 2.0 * coeff * dt  # hbar=1
            paulis = pauli_string_to_qubits(ps_str)
            if not paulis:
                continue
            # Build Pauli exponential via basis change + CNOT ladder + Rz
            qubits = [q for q, _ in paulis]
            ops = [op for _, op in paulis]

            # Basis change: X→H, Y→Rx(pi/2)
            for q, op in paulis:
                if op == 'X':
                    circ.H(q)
                elif op == 'Y':
                    circ.Rx(0.5, q)  # Rx(pi/2) in half-turns

            # CNOT ladder
            for i in range(len(qubits) - 1):
                circ.CX(qubits[i], qubits[i + 1])

            # Rz rotation
            circ.Rz(angle / math.pi, qubits[-1])  # half-turns

            # Uncompute: inverse CNOT ladder
            for i in range(len(qubits) - 2, -1, -1):
                circ.CX(qubits[i], qubits[i + 1])

            # Inverse basis change
            for q, op in paulis:
                if op == 'X':
                    circ.H(q)
                elif op == 'Y':
                    circ.Rx(-0.5, q)

    return circ


def count_gate_types(circ: Circuit) -> Dict[str, int]:
    """Count gate types in a TKET circuit."""
    counts = {}
    for cmd in circ:
        optype = cmd.op.type
        name = str(optype).split('.')[-1]
        counts[name] = counts.get(name, 0) + 1
    return counts


def run_tket_compile(
    circ: Circuit,
    strategy: str = "PauliSimp",
    pauli_strat: PauliPartitionStrat = PauliPartitionStrat.CommutingSets,
) -> Tuple[Circuit, float]:
    """Apply TKET optimization pipeline and return (optimized_circuit, time_ms)."""
    start = time.time()

    if strategy == "PauliSimp":
        PauliSimp().apply(circ)
    elif strategy == "GreedyPauliSimp":
        GreedyPauliSimp().apply(circ)
    elif strategy == "PauliSimp+Greedy":
        PauliSimp().apply(circ)
        GreedyPauliSimp().apply(circ)

    # Standard TKET optimization
    CliffordSimp().apply(circ)
    FullPeepholeOptimise().apply(circ)

    elapsed = (time.time() - start) * 1000.0
    return circ, elapsed


def benchmark_all(
    steps: int = 10, order: int = 1, dt: float = 1.0, output_path: str = None
) -> Dict:
    """Run full benchmark across all Hamiltonians and strategies."""
    results = {
        "config": {"steps": steps, "order": order, "dt": dt},
        "hamiltonians": {},
    }

    strategies = ["None", "PauliSimp", "GreedyPauliSimp", "PauliSimp+Greedy"]

    for name, ham in HAMILTONIANS.items():
        print(f"\n{'='*60}")
        print(f"  {name} ({ham['num_qubits']}q, {len(ham['terms'])} terms)")
        print(f"{'='*60}")

        h_results = {}
        for strat in strategies:
            print(f"  {strat:25s} ... ", end="", flush=True)

            # Build raw Trotter circuit
            circ = build_trotter_circuit(ham, dt, steps, order)

            raw_counts = count_gate_types(circ)
            raw_total = sum(raw_counts.values())

            if strat == "None":
                opt_circ = circ
                elapsed = 0.0
            else:
                opt_circ, elapsed = run_tket_compile(
                    circ.copy(), strategy=strat
                )

            opt_counts = count_gate_types(opt_circ)
            opt_total = sum(opt_counts.values())

            print(f"{raw_total}→{opt_total} gates ({elapsed:.0f}ms)")

            h_results[strat] = {
                "raw_gates": raw_total,
                "raw_counts": raw_counts,
                "opt_gates": opt_total,
                "opt_counts": opt_counts,
                "depth": opt_circ.depth(),
                "two_qubit_gates": opt_circ.n_2qb_gates(),
                "time_ms": elapsed,
            }

        results["hamiltonians"][name] = h_results

    if output_path:
        with open(output_path, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults written to {output_path}")

    return results


def main():
    import argparse
    parser = argparse.ArgumentParser(description="MyQuat vs TKET benchmark")
    parser.add_argument("--steps", type=int, default=10, help="Trotter steps")
    parser.add_argument("--order", type=int, default=1, help="Trotter order")
    parser.add_argument("--dt", type=float, default=1.0, help="Evolution time")
    parser.add_argument("--output", type=str, default="experiments/tket_comparison/tket_results.json",
                        help="Output JSON path")
    args = parser.parse_args()

    benchmark_all(
        steps=args.steps,
        order=args.order,
        dt=args.dt,
        output_path=args.output,
    )


if __name__ == "__main__":
    main()
