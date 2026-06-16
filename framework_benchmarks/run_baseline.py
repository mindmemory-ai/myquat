#!/usr/bin/env python3
"""
Quantum Framework Baseline Test Suite
========================================
Compares MyQuat (Rust) against Qiskit, Cirq, TKET, PennyLane, and Paulihedral
across Hamiltonian compilation, gate decomposition, and circuit transpilation.

Usage:
    conda activate quantum
    cargo build --release --example benchmark_hamiltonian
    python run_baseline.py

Output:
    results/baseline_YYYYMMDD_HHMMSS.csv   — raw data
    results/baseline_YYYYMMDD_HHMMSS.json  — full results
    results/baseline_YYYYMMDD_HHMMSS.md    — formatted report
"""

import json
import csv
import time
import os
import sys
import subprocess
import warnings
from datetime import datetime
from pathlib import Path

import numpy as np
import argparse

# ── Framework imports (all optional) ──────────────────────────────────────

FRAMEWORKS = {}

# Qiskit
try:
    import qiskit
    from qiskit import QuantumCircuit as QkCircuit, transpile
    from qiskit.quantum_info import SparsePauliOp, Operator
    from qiskit.circuit.library import PauliEvolutionGate
    from qiskit.synthesis import SuzukiTrotter, LieTrotter

    FRAMEWORKS["qiskit"] = True
except ImportError:
    FRAMEWORKS["qiskit"] = False

# Cirq
try:
    import cirq

    FRAMEWORKS["cirq"] = True
except ImportError:
    FRAMEWORKS["cirq"] = False

# TKET
try:
    from pytket import Circuit as TkCircuit
    from pytket.pauli import PauliStabiliser
    from pytket.passes import CliffordSimp, FullPeepholeOptimise, DecomposeBoxes
    from pytket.transform import Transform

    FRAMEWORKS["tket"] = True
except ImportError:
    FRAMEWORKS["tket"] = False

# PennyLane
try:
    import pennylane as qml

    FRAMEWORKS["pennylane"] = True
except ImportError:
    FRAMEWORKS["pennylane"] = False

# Paulihedral
try:
    import paulihedral

    FRAMEWORKS["paulihedral"] = True
except ImportError:
    FRAMEWORKS["paulihedral"] = False

# MyQuat — always attempted via subprocess
MYQUAT_BIN = Path(__file__).parent.parent / "target/release/examples/benchmark_hamiltonian"
FRAMEWORKS["myquat"] = MYQUAT_BIN.exists()


# ═══════════════════════════════════════════════════════════════════════════
# Hamiltonian Problem Definitions
# ═══════════════════════════════════════════════════════════════════════════

def _p(n, i, p):
    """Create a Pauli string of length n with 'p' at position i, 'I' elsewhere."""
    arr = ["I"] * n
    arr[i] = p
    return "".join(arr)


def _pp(n, i, j, p, q):
    """Create length-n Pauli string with p at i, q at j."""
    arr = ["I"] * n
    arr[i] = p
    arr[j] = q
    return "".join(arr)


def make_h2_hamiltonian():
    """H2 molecule, 4 qubits, 15 Pauli terms (Jordan-Wigner encoded)."""
    n = 4
    return [
        ("I" * n, -0.8105),
        (_p(n, 3, "Z"), 0.1721),
        (_p(n, 2, "Z"), -0.2228),
        (_p(n, 1, "Z"), 0.1721),
        (_p(n, 0, "Z"), -0.2228),
        (_pp(n, 2, 3, "Z", "Z"), 0.1686),
        (_pp(n, 1, 3, "Z", "Z"), 0.1205),
        (_pp(n, 1, 2, "Z", "Z"), 0.1686),
        (_pp(n, 0, 3, "Z", "Z"), 0.1686),
        (_pp(n, 0, 2, "Z", "Z"), 0.1205),
        (_pp(n, 0, 1, "Z", "Z"), 0.1686),
        (_pp(n, 2, 3, "X", "X"), 0.0454),
        (_pp(n, 2, 3, "Y", "Y"), 0.0454),
        (_pp(n, 0, 1, "X", "X"), 0.0454),
        (_pp(n, 0, 1, "Y", "Y"), 0.0454),
    ]


def make_lih_hamiltonian():
    """LiH molecule, 6 qubits, 29 Pauli terms."""
    n = 6
    return [
        ("I" * n, -7.4983),
        (_p(n, 5, "Z"), 0.3936), (_p(n, 4, "Z"), 0.3936),
        (_p(n, 3, "Z"), -0.3936), (_p(n, 2, "Z"), -0.3936),
        (_p(n, 1, "Z"), 0.0896), (_p(n, 0, "Z"), 0.0896),
        (_pp(n, 4, 5, "Z", "Z"), 0.1815), (_pp(n, 3, 5, "Z", "Z"), 0.1240),
        (_pp(n, 3, 4, "Z", "Z"), 0.1815), (_pp(n, 2, 5, "Z", "Z"), 0.1815),
        (_pp(n, 2, 4, "Z", "Z"), 0.1240), (_pp(n, 2, 3, "Z", "Z"), 0.1815),
        (_pp(n, 1, 4, "Z", "Z"), 0.0620), (_pp(n, 1, 3, "Z", "Z"), 0.0620),
        (_pp(n, 1, 5, "Z", "Z"), 0.0620), (_pp(n, 0, 4, "Z", "Z"), 0.0620),
        (_pp(n, 0, 3, "Z", "Z"), 0.0620), (_pp(n, 0, 2, "Z", "Z"), 0.0620),
        (_pp(n, 0, 1, "Z", "Z"), 0.0620),
        (_pp(n, 0, 5, "Z", "Z"), 0.0320), (_pp(n, 0, 4, "Z", "Z"), 0.0320),
        (_pp(n, 4, 5, "Y", "Y"), 0.0230), (_pp(n, 4, 5, "X", "X"), 0.0230),
        (_pp(n, 3, 5, "Y", "Y"), 0.0230), (_pp(n, 3, 5, "X", "X"), 0.0230),
        (_pp(n, 2, 3, "Y", "Y"), 0.0230), (_pp(n, 2, 3, "X", "X"), 0.0230),
    ]


def make_heisenberg_hamiltonian(num_qubits, J=1.0):
    """1D Heisenberg XXZ model with periodic boundary."""
    terms = []
    for i in range(num_qubits):
        j = (i + 1) % num_qubits
        for p, q in [("X", "X"), ("Y", "Y"), ("Z", "Z")]:
            terms.append((_pp(num_qubits, i, j, p, q), J))
    return terms


def make_tfim_hamiltonian(num_qubits, J=1.0, g=0.5):
    """Transverse-field Ising model (open boundary)."""
    terms = []
    for i in range(num_qubits - 1):
        terms.append((_pp(num_qubits, i, i + 1, "Z", "Z"), -J))
    for i in range(num_qubits):
        terms.append((_p(num_qubits, i, "X"), -g))
    return terms


def make_random_hamiltonian(num_qubits, num_terms, seed=42):
    """Random Pauli Hamiltonian with fixed seed."""
    rng = np.random.RandomState(seed)
    paulis = ["I", "X", "Y", "Z"]
    terms = []
    for _ in range(num_terms):
        ps = "".join(rng.choice(paulis, size=num_qubits))
        if ps == "I" * num_qubits:
            ps = "I" * (num_qubits - 1) + "Z"
        coeff = rng.uniform(-1, 1)
        terms.append((ps, float(coeff)))
    return terms


def hamiltonian_matrix(terms, num_qubits):
    """Build dense Hamiltonian matrix from Pauli terms (for accuracy checks)."""
    I = np.eye(2, dtype=complex)
    X = np.array([[0, 1], [1, 0]], dtype=complex)
    Y = np.array([[0, -1j], [1j, 0]], dtype=complex)
    Z = np.array([[1, 0], [0, -1]], dtype=complex)
    pauli_map = {"I": I, "X": X, "Y": Y, "Z": Z}

    H = np.zeros((2 ** num_qubits, 2 ** num_qubits), dtype=complex)
    for pauli_str, coeff in terms:
        op = np.eye(1, dtype=complex)
        for p in pauli_str:
            op = np.kron(op, pauli_map[p])
        H += coeff * op
    return H


# ═══════════════════════════════════════════════════════════════════════════
# Problem registry
# ═══════════════════════════════════════════════════════════════════════════

PROBLEMS = [
    ("H2_4q", make_h2_hamiltonian),
    ("LiH_6q", make_lih_hamiltonian),
    ("Heisenberg_4q", lambda: make_heisenberg_hamiltonian(4)),
    ("Heisenberg_6q", lambda: make_heisenberg_hamiltonian(6)),
    ("Heisenberg_8q", lambda: make_heisenberg_hamiltonian(8)),
    ("TFIM_4q", lambda: make_tfim_hamiltonian(4)),
    ("TFIM_6q", lambda: make_tfim_hamiltonian(6)),
    ("TFIM_8q", lambda: make_tfim_hamiltonian(8)),
    ("TFIM_10q", lambda: make_tfim_hamiltonian(10)),
    ("Random_4q_20t", lambda: make_random_hamiltonian(4, 20)),
    ("Random_6q_30t", lambda: make_random_hamiltonian(6, 30)),
    ("Random_8q_40t", lambda: make_random_hamiltonian(8, 40)),
]

CONFIGS = [
    {"steps": 10, "time": 1.0, "order": 1},
    {"steps": 10, "time": 1.0, "order": 2},
    {"steps": 50, "time": 1.0, "order": 1},
    {"steps": 50, "time": 1.0, "order": 2},
    {"steps": 100, "time": 1.0, "order": 2},
]


# ═══════════════════════════════════════════════════════════════════════════
# Framework Benchmark Functions — Dimension A (Hamiltonian Compilation)
# ═══════════════════════════════════════════════════════════════════════════

# ── MyQuat ────────────────────────────────────────────────────────────────

def bench_myquat(terms, num_qubits, steps, evolution_time, order):
    """Call MyQuat via compiled Rust binary."""
    if not MYQUAT_BIN.exists():
        return _fail("Binary not found — build with: cargo build --release --example benchmark_hamiltonian")

    cmd = [str(MYQUAT_BIN), str(num_qubits), str(steps), str(evolution_time), str(order)]
    for ps, coeff in terms:
        cmd.append(f"{ps}:{coeff}")

    try:
        start = time.perf_counter()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        wall_time = (time.perf_counter() - start) * 1000
        if result.returncode == 0:
            data = json.loads(result.stdout.strip())
            result_dict = _ok("MyQuat", "Rust", data.get("compilation_time_ms", wall_time),
                              data.get("gate_count", 0), data.get("circuit_depth", 0))

            # Compute accuracy (Frobenius norm error vs exact exp(-iHt))
            frob_err = _compute_accuracy(data, terms, num_qubits, evolution_time)
            if frob_err is not None:
                result_dict["accuracy_frobenius_error"] = frob_err
            return result_dict
        return _fail(f"MyQuat stderr: {result.stderr[:120]}")
    except subprocess.TimeoutExpired:
        return _fail("timeout (>120s)")
    except Exception as e:
        return _fail(str(e))


def compute_pennyLane_depth(operations, num_wires):
    """Compute circuit depth from a list of PennyLane operations.

    Uses per-wire time-stamps: for each gate, find the max time among
    its touched wires, increment by 1, and update all touched wires.
    """
    wire_time = [0] * num_wires
    for op in operations:
        wires = op.wires
        if not wires:
            continue
        max_t = max(wire_time[w] for w in wires)
        new_t = max_t + 1
        for w in wires:
            wire_time[w] = new_t
    return max(wire_time) if wire_time else 0


def _compute_accuracy(data, terms, num_qubits, evolution_time):
    """Compute Frobenius norm error between circuit unitary and exact exp(-iHt)."""
    unitary_data = data.get("unitary")
    if unitary_data is None:
        return None

    try:
        from scipy.linalg import expm

        # Parse circuit unitary from flat JSON array [[re,im], ...]
        dim = 1 << num_qubits
        flat = np.array(unitary_data, dtype=float)
        if flat.ndim != 2 or flat.shape[0] != dim * dim or flat.shape[1] != 2:
            return None
        U_circuit = (flat[:, 0] + 1j * flat[:, 1]).reshape(dim, dim)

        # Build exact exp(-i*H*t)
        H_mat = hamiltonian_matrix(terms, num_qubits)
        U_exact = expm(-1j * H_mat * evolution_time)

        # Align global phase: U_circuit may differ from U_exact by e^{i*theta}
        # due to identity terms in H being skipped (skip_identities=true).
        # Minimize ||U*e^{i*theta} - V||_F^2 => theta_opt = arg(tr(U^H V))
        overlap = np.trace(U_circuit.conj().T @ U_exact)
        theta_opt = np.angle(overlap)
        U_circuit_aligned = U_circuit * np.exp(1j * theta_opt)

        # Frobenius norm error, normalized by sqrt(dim)
        diff = U_circuit_aligned - U_exact
        frob_error = np.sqrt(np.sum(np.abs(diff) ** 2))
        normalized_error = frob_error / np.sqrt(dim)
        return float(normalized_error)
    except Exception:
        return None


# ── Qiskit ────────────────────────────────────────────────────────────────

def bench_qiskit(terms, num_qubits, steps, evolution_time, order):
    """Qiskit with no transpilation optimization."""
    if not FRAMEWORKS["qiskit"]:
        return _fail("not available")
    try:
        hamiltonian = SparsePauliOp.from_list([(p, c) for p, c in terms])
        start = time.perf_counter()
        if order == 1:
            synthesis = LieTrotter(reps=steps)
        else:
            synthesis = SuzukiTrotter(order=order, reps=steps)
        gate = PauliEvolutionGate(hamiltonian, time=evolution_time, synthesis=synthesis)
        circuit = QkCircuit(num_qubits)
        circuit.append(gate, range(num_qubits))
        circuit = circuit.decompose().decompose()
        circuit = transpile(circuit, basis_gates=["u3", "cx"], optimization_level=0)
        t = (time.perf_counter() - start) * 1000
        return _ok("Qiskit", "Python", t, circuit.size(), circuit.depth())
    except Exception as e:
        return _fail(str(e))


def bench_qiskit_opt(terms, num_qubits, steps, evolution_time, order):
    """Qiskit with optimization level 3."""
    if not FRAMEWORKS["qiskit"]:
        return _fail("not available")
    try:
        hamiltonian = SparsePauliOp.from_list([(p, c) for p, c in terms])
        start = time.perf_counter()
        if order == 1:
            synthesis = LieTrotter(reps=steps)
        else:
            synthesis = SuzukiTrotter(order=order, reps=steps)
        gate = PauliEvolutionGate(hamiltonian, time=evolution_time, synthesis=synthesis)
        circuit = QkCircuit(num_qubits)
        circuit.append(gate, range(num_qubits))
        circuit = circuit.decompose().decompose()
        circuit = transpile(circuit, basis_gates=["u3", "cx"], optimization_level=3)
        t = (time.perf_counter() - start) * 1000
        return _ok("Qiskit_Opt", "Python", t, circuit.size(), circuit.depth())
    except Exception as e:
        return _fail(str(e))


# ── Cirq ──────────────────────────────────────────────────────────────────

def bench_cirq(terms, num_qubits, steps, evolution_time, order):
    """Cirq — PauliStringPhasor with decomposition."""
    if not FRAMEWORKS["cirq"]:
        return _fail("not available")
    try:
        qubits = cirq.LineQubit.range(num_qubits)
        pauli_map = {"I": cirq.I, "X": cirq.X, "Y": cirq.Y, "Z": cirq.Z}
        dt = evolution_time / steps

        start = time.perf_counter()
        circuit = cirq.Circuit()
        for _ in range(steps):
            for pauli_str, coeff in terms:
                term_dict = {}
                for i, p in enumerate(pauli_str):
                    if p != "I":
                        term_dict[qubits[i]] = pauli_map[p]
                if term_dict:
                    # PauliString coefficient must be ±1 for PauliStringPhasor.
                    # Fold the Hamiltonian coefficient into the exponent.
                    ps = cirq.PauliString(term_dict)  # coefficient = +1
                    circuit += cirq.PauliStringPhasor(ps, exponent_neg=-coeff * dt / np.pi)
        try:
            circuit = cirq.optimize_for_target_gateset(
                circuit, gateset=cirq.CZTargetGateset()
            )
        except Exception:
            pass  # optimization is best-effort
        t = (time.perf_counter() - start) * 1000
        return _ok("Cirq", "Python", t, len(list(circuit.all_operations())), len(circuit))
    except Exception as e:
        return _fail(str(e))


# ── TKET ──────────────────────────────────────────────────────────────────

def bench_tket(terms, num_qubits, steps, evolution_time, order):
    """TKET — build Pauli exponential circuits and compile."""
    if not FRAMEWORKS["tket"]:
        return _fail("not available")
    try:
        start = time.perf_counter()

        # Build circuit: each Trotter step = product of exp(-i*coeff*dt*P)
        dt = evolution_time / steps
        circ = TkCircuit(num_qubits)

        for _ in range(steps):
            for pauli_str, coeff in terms:
                angle = -coeff * dt
                # Identify which qubits have non-identity Paulis
                qubit_indices = []
                pauli_types = []
                for i, p in enumerate(pauli_str):
                    if p != "I":
                        qubit_indices.append(i)
                        pauli_types.append(p)

                if not qubit_indices:
                    continue  # identity term — global phase, skip

                # Map Pauli string to single-qubit rotations + CX ladder
                # Strategy: H (for X) or Sdg+H (for Y) on each qubit,
                # then CX ladder, then Rz, then reverse
                _add_pauli_exponential_tket(circ, qubit_indices, pauli_types, angle)

        # Apply TKET optimizations
        DecomposeBoxes().apply(circ)
        CliffordSimp().apply(circ)

        t = (time.perf_counter() - start) * 1000
        gate_count = sum(1 for _ in circ.get_commands())
        depth = circ.depth()
        return _ok("TKET", "Python", t, gate_count, depth)
    except Exception as e:
        return _fail(str(e))


def _add_pauli_exponential_tket(circuit, qubit_indices, pauli_types, angle):
    """Add exp(-i * angle * P) to a TKET circuit using the standard ladder method."""
    from pytket import OpType
    from pytket.circuit import fresh_symbol

    n = len(qubit_indices)

    # Apply basis-change gates
    for idx, p in zip(qubit_indices, pauli_types):
        if p == "X":
            circuit.H(idx)
        elif p == "Y":
            circuit.Sdg(idx)
            circuit.H(idx)
        # Z: no basis change needed

    # CX ladder (entangling chain)
    for i in range(n - 1):
        circuit.CX(qubit_indices[i], qubit_indices[i + 1])

    # Rz rotation on last qubit
    circuit.Rz(2.0 * angle, qubit_indices[-1])

    # Uncompute CX ladder
    for i in reversed(range(n - 1)):
        circuit.CX(qubit_indices[i], qubit_indices[i + 1])

    # Undo basis-change gates
    for idx, p in zip(qubit_indices, pauli_types):
        if p == "X":
            circuit.H(idx)
        elif p == "Y":
            circuit.H(idx)
            circuit.S(idx)


# ── PennyLane ─────────────────────────────────────────────────────────────

def bench_pennylane(terms, num_qubits, steps, evolution_time, order):
    """PennyLane — build Trotterized time evolution circuit."""
    if not FRAMEWORKS["pennylane"]:
        return _fail("not available")
    try:
        start = time.perf_counter()

        # Build Hamiltonian observable using qml.dot (PennyLane ≥0.38)
        coeffs = [c for _, c in terms]
        obs_list = []
        for pauli_str, _ in terms:
            op = None
            for i, p in enumerate(pauli_str):
                if p == "I":
                    continue
                pauli_op = getattr(qml, f"Pauli{p}")(i)
                op = op @ pauli_op if op is not None else pauli_op
            if op is None:
                op = qml.Identity(0)
            obs_list.append(op)

        hamiltonian = qml.dot(coeffs, obs_list)

        # Build Trotterized circuit using qml.TrotterProduct
        dev = qml.device("default.qubit", wires=num_qubits)

        @qml.qnode(dev)
        def trotter_circuit():
            qml.TrotterProduct(hamiltonian, time=evolution_time, n=steps, order=order)
            return qml.state()

        _ = trotter_circuit()
        tape = trotter_circuit._tape  # PennyLane ≥0.38 uses _tape
        gate_count = 0
        depth = 0
        if tape:
            try:
                expanded = tape.expand()
                gate_count = len(expanded.operations)
                depth = compute_pennyLane_depth(expanded.operations, num_qubits)
            except Exception:
                gate_count = len(tape.operations)
                depth = compute_pennyLane_depth(tape.operations, num_qubits)

        t = (time.perf_counter() - start) * 1000
        return _ok("PennyLane", "Python", t, gate_count, depth)
    except Exception as e:
        return _fail(str(e))


# ── Paulihedral ───────────────────────────────────────────────────────────

def bench_paulihedral(terms, num_qubits, steps, evolution_time, order):
    """Paulihedral — optimize Pauli rotation sequences.

    Supports Trotter orders 1 (Lie-Trotter), 2 (Strang), 4 (Suzuki), 6 (Suzuki).
    Higher orders compose multiple block_opt_FT calls with appropriate time steps
    and term ordering.
    """
    if not FRAMEWORKS["paulihedral"]:
        return _fail("not available")
    try:
        from paulihedral.synthesis_FT import block_opt_FT, pauliString
        from qiskit import QuantumCircuit

        start = time.perf_counter()
        dt = evolution_time / steps

        def build_layers(scale, reverse_order=False):
            """Build layers for one S_1 step with given scale factor.

            Each term forms its own block: one block = one term = [[pauliString]].
            block_opt_FT expects List[List[List[pauliString]]] = layers→groups→terms.
            """
            layers = []
            term_list = reversed(terms) if reverse_order else terms
            for ps, coeff in term_list:
                if all(p == "I" for p in ps):
                    continue
                layers.append([[pauliString(ps, coeff=coeff * scale)]])
            return layers

        def s1_forward(scale):
            """First-order forward step: exp(-i*H*scale)."""
            return block_opt_FT(build_layers(scale, reverse_order=False),
                                time_parameter=1.0)

        def s1_reverse(scale):
            """First-order reverse step: exp(+i*H*scale) via reversed term order."""
            return block_opt_FT(build_layers(scale, reverse_order=True),
                                time_parameter=1.0)

        def s2_step(scale):
            """Second-order Strang: S_1(scale/2) * S_1_rev(scale/2)."""
            half = scale / 2.0
            qc_fwd = s1_forward(half)
            qc_rev = s1_reverse(half)
            return qc_fwd.compose(qc_rev)

        total_qc = QuantumCircuit(num_qubits)

        if order == 1:
            # Lie-Trotter: N steps of S_1(dt)
            for _ in range(steps):
                total_qc = total_qc.compose(s1_forward(dt))

        elif order == 2:
            # Strang: N steps of S_2(dt)
            for _ in range(steps):
                total_qc = total_qc.compose(s2_step(dt))

        elif order == 4:
            # Suzuki 4th: S_4(dt) = S_2(p1*dt) S_2(p2*dt) S_2(p1*dt) S_2(p2*dt) S_2(p1*dt)
            p1 = 1.0 / (4.0 - 4.0 ** (1.0 / 3.0))
            p2 = 1.0 - 4.0 * p1
            for _ in range(steps):
                for coeff in (p1, p2, p1, p2, p1):
                    total_qc = total_qc.compose(s2_step(dt * coeff))

        elif order == 6:
            # Suzuki 6th: S_6(dt) = S_4(p5*dt)^2 S_4((1-4p5)*dt) S_4(p5*dt)^2
            p5 = 1.0 / (4.0 - 4.0 ** (1.0 / 5.0))
            p6 = 1.0 - 4.0 * p5

            def s4_step(scale):
                p1 = 1.0 / (4.0 - 4.0 ** (1.0 / 3.0))
                p2 = 1.0 - 4.0 * p1
                qc = QuantumCircuit(num_qubits)
                for coeff in (p1, p2, p1, p2, p1):
                    qc = qc.compose(s2_step(scale * coeff))
                return qc

            for _ in range(steps):
                for coeff in (p5, p5, p6, p5, p5):
                    total_qc = total_qc.compose(s4_step(dt * coeff))

        else:
            # Fallback: treat as S_1 repeated order times
            for _ in range(steps):
                for _ in range(order):
                    total_qc = total_qc.compose(s1_forward(dt / order))

        gate_count = total_qc.size()
        depth = total_qc.depth()

        t = (time.perf_counter() - start) * 1000
        return _ok("Paulihedral", "Python", t, gate_count, depth)
    except Exception as e:
        return _fail(f"Paulihedral error: {str(e)[:120]}")


# ═══════════════════════════════════════════════════════════════════════════
# Helpers
# ═══════════════════════════════════════════════════════════════════════════

def _ok(framework, language, time_ms, gate_count, depth):
    return {
        "framework": framework,
        "language": language,
        "compilation_time_ms": time_ms,
        "gate_count": gate_count,
        "circuit_depth": depth,
        "success": True,
        "error": None,
    }


def _fail(error):
    return {
        "framework": "?",
        "language": "?",
        "compilation_time_ms": None,
        "gate_count": None,
        "circuit_depth": None,
        "success": False,
        "error": str(error),
    }


# Bench functions keyed by framework name
DIM_A_BENCHMARKS = {
    "MyQuat": bench_myquat,
    "Qiskit": bench_qiskit,
    "Qiskit_Opt": bench_qiskit_opt,
    "Cirq": bench_cirq,
    "TKET": bench_tket,
    "PennyLane": bench_pennylane,
    "Paulihedral": bench_paulihedral,
}


# ═══════════════════════════════════════════════════════════════════════════
# Main Benchmark Runner
# ═══════════════════════════════════════════════════════════════════════════

def run_all_benchmarks(filter_frameworks=None, filter_problems=None, filter_configs=None):
    """Run all benchmarks across all problems and configs.

    Args:
        filter_frameworks: Optional list of framework names to test.
        filter_problems: Optional list of problem names to test.
        filter_configs: Optional list of config indices (0-4) to test.
    """
    print("=" * 80)
    print("Quantum Framework Baseline Test Suite")
    print("=" * 80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    if filter_frameworks:
        print(f"Filter: frameworks={filter_frameworks}")
    if filter_problems:
        print(f"Filter: problems={filter_problems}")
    if filter_configs is not None:
        print(f"Filter: config_indices={filter_configs}")
    print(f"Frameworks: {', '.join(k for k, v in FRAMEWORKS.items() if v)}")
    print()

    results = []
    # Stats per framework for final report
    success_counts = {k: 0 for k in DIM_A_BENCHMARKS}
    total_attempts = {k: 0 for k in DIM_A_BENCHMARKS}

    for problem_name, problem_fn in PROBLEMS:
        if filter_problems and problem_name not in filter_problems:
            continue

        terms = problem_fn()
        num_qubits = len(terms[0][0]) if terms else 0
        num_terms = len(terms)

        print(f"\n{'=' * 60}")
        print(f"Problem: {problem_name}: {num_qubits} qubits, {num_terms} terms")
        print(f"{'=' * 60}")

        for ci, config in enumerate(CONFIGS):
            if filter_configs is not None and ci not in filter_configs:
                continue

            steps, evo_t, order = config["steps"], config["time"], config["order"]
            print(f"\n  Config: steps={steps}, time={evo_t}, order={order}")
            print(f"  {'-' * 50}")

            for fw_name, bench_fn in DIM_A_BENCHMARKS.items():
                if filter_frameworks and fw_name not in filter_frameworks:
                    continue
                total_attempts[fw_name] += 1
                row = {
                    "problem": problem_name,
                    "num_qubits": num_qubits,
                    "num_terms": num_terms,
                    "trotter_steps": steps,
                    "trotter_order": order,
                    "evolution_time": evo_t,
                    "framework": fw_name,
                }

                r = bench_fn(terms, num_qubits, steps, evo_t, order)
                if r.get("success"):
                    row.update(r)
                    success_counts[fw_name] += 1
                    print(f"    {fw_name:>15s}: {r['compilation_time_ms']:8.2f} ms, "
                          f"{r['gate_count']:6d} gates, depth {r['circuit_depth']:5d}")
                else:
                    row.update(r)
                    print(f"    {fw_name:>15s}: FAILED — {r.get('error', '')[:80]}")

                results.append(row)

    # ── Print summary ─────────────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("SUMMARY: Success Rates")
    print("=" * 80)
    for fw in DIM_A_BENCHMARKS:
        if total_attempts[fw] > 0:
            pct = 100 * success_counts[fw] / total_attempts[fw]
            print(f"  {fw:>15s}: {success_counts[fw]}/{total_attempts[fw]} ({pct:.0f}%)")

    # ── Speedup stats ─────────────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("SUMMARY: Speedup vs Qiskit (compilation time)")
    print("=" * 80)

    # Group by framework
    fw_times = {}
    for r in results:
        if not r.get("success"):
            continue
        fw = r["framework"]
        if fw not in fw_times:
            fw_times[fw] = []
        fw_times[fw].append(r["compilation_time_ms"])

    qiskit_times = fw_times.get("Qiskit", [])
    if qiskit_times:
        qiskit_avg = np.mean(qiskit_times)
        print(f"  Qiskit average: {qiskit_avg:.2f} ms (baseline)")
        for fw, times in fw_times.items():
            if fw != "Qiskit" and times:
                avg = np.mean(times)
                speedup = qiskit_avg / avg if avg > 0 else float("inf")
                print(f"  {fw:>15s}: {avg:.2f} ms ({speedup:.1f}x vs Qiskit)")

    # ── Gate count comparison ─────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("SUMMARY: Average Gate Count (across all successful runs)")
    print("=" * 80)
    for fw, times in sorted(fw_times.items(), key=lambda x: x[1] if x[1] else [0]):
        fw_gates = [r["gate_count"] for r in results
                    if r.get("success") and r["framework"] == fw and r.get("gate_count")]
        if fw_gates:
            fw_avg_time = np.mean([r["compilation_time_ms"] for r in results
                                   if r.get("success") and r["framework"] == fw])
            print(f"  {fw:>15s}: avg {np.mean(fw_gates):.0f} gates | "
                  f"avg {fw_avg_time:.2f} ms")

    return results


def save_results(results):
    """Save results to timestamped CSV, JSON, and Markdown report."""
    results_dir = Path(__file__).parent / "results"
    results_dir.mkdir(exist_ok=True)

    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    base = results_dir / f"baseline_{ts}"

    # CSV
    if results:
        fieldnames = list(results[0].keys())
        with open(f"{base}.csv", "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames, extrasaction="ignore")
            writer.writeheader()
            writer.writerows(results)

    # JSON
    with open(f"{base}.json", "w") as f:
        json.dump(results, f, indent=2, default=str)

    # Markdown report
    _write_markdown_report(results, base)

    print(f"\nResults saved to: {base}.csv, {base}.json, {base}_report.md")
    return base


def _write_markdown_report(results, base):
    """Generate a simple Markdown summary report."""
    lines = []
    lines.append("# Framework Baseline Report")
    lines.append(f"**Date:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    lines.append(f"**Total tests:** {len(results)}")
    lines.append("")

    # Success counts
    lines.append("## Success Rates")
    lines.append("")
    lines.append("| Framework | Success | Total | Rate |")
    lines.append("|-----------|---------|-------|------|")
    for fw in DIM_A_BENCHMARKS:
        total = sum(1 for r in results if r["framework"] == fw)
        ok = sum(1 for r in results if r["framework"] == fw and r.get("success"))
        if total > 0:
            lines.append(f"| {fw} | {ok} | {total} | {100*ok/total:.0f}% |")
    lines.append("")

    # Speedup stats (by problem and config)
    lines.append("## Speedup Summary (compilation time vs Qiskit)")
    lines.append("")

    fw_speedups = {}
    for r in results:
        if not r.get("success"):
            continue
        fw = r["framework"]
        if fw not in fw_speedups:
            fw_speedups[fw] = []
        fw_speedups[fw].append(r["compilation_time_ms"])

    qiskit_times = [r["compilation_time_ms"] for r in results
                    if r.get("success") and r["framework"] == "Qiskit"]
    if qiskit_times:
        qiskit_avg = np.mean(qiskit_times)
        lines.append(f"**Qiskit baseline:** {qiskit_avg:.2f} ms average")
        lines.append("")
        lines.append("| Framework | Avg Time (ms) | Speedup vs Qiskit | Avg Gates | Avg Depth | Avg Accuracy |")
        lines.append("|-----------|---------------|--------------------|-----------|-----------|--------------|")
        for fw in ["MyQuat", "Qiskit", "Qiskit_Opt", "Cirq", "TKET", "PennyLane", "Paulihedral"]:
            if fw not in fw_speedups:
                continue
            avg_t = np.mean(fw_speedups[fw])
            speedup = qiskit_avg / avg_t if avg_t > 0 else 0
            avg_gates = np.mean([r["gate_count"] for r in results
                                 if r.get("success") and r["framework"] == fw
                                 and r.get("gate_count")])
            avg_depth = np.mean([r["circuit_depth"] for r in results
                                 if r.get("success") and r["framework"] == fw
                                 and r.get("circuit_depth")])
            acc_vals = [r["accuracy_frobenius_error"] for r in results
                       if r.get("success") and r["framework"] == fw
                       and r.get("accuracy_frobenius_error") is not None]
            acc_str = f"{np.mean(acc_vals):.2e}" if acc_vals else "—"
            lines.append(f"| {fw} | {avg_t:.2f} | {speedup:.1f}x | {avg_gates:.0f} | {avg_depth:.0f} | {acc_str} |")

    # ── Accuracy section ─────────────────────────────────────────────────
    myquat_acc = [(r, r.get("accuracy_frobenius_error")) for r in results
                  if r.get("success") and r["framework"] == "MyQuat"
                  and r.get("accuracy_frobenius_error") is not None]
    if myquat_acc:
        lines.append("")
        lines.append("## MyQuat Accuracy (Frobenius norm error vs exact exp(-iHt))")
        lines.append("")
        lines.append("| Problem | Steps | Order | Error |")
        lines.append("|---------|-------|-------|-------|")
        for r, acc in myquat_acc:
            lines.append(f"| {r['problem']} | {r['trotter_steps']} | {r['trotter_order']} | {acc:.4e} |")

    lines.append("")
    lines.append("## Per-Problem Results")
    lines.append("")
    lines.append("| Problem | FW | Steps | Order | Time(ms) | Gates | Depth | Accuracy |")
    lines.append("|---------|-----|-------|-------|----------|-------|-------|----------|")
    for r in results:
        acc = r.get("accuracy_frobenius_error")
        acc_s = f"{acc:.2e}" if acc is not None else "—"
        if r.get("success"):
            lines.append(f"| {r['problem']} | {r['framework']} | {r['trotter_steps']} | "
                         f"{r['trotter_order']} | {r['compilation_time_ms']:.2f} | "
                         f"{r['gate_count']} | {r['circuit_depth']} | {acc_s} |")
        else:
            lines.append(f"| {r['problem']} | {r['framework']} | — | — | FAILED | — | — |")

    with open(f"{base}_report.md", "w") as f:
        f.write("\n".join(lines))


# ═══════════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(description="Quantum Framework Baseline Tests")
    parser.add_argument("--framework", "-f", nargs="*", default=None,
                        help="Specific framework(s) to test (e.g. MyQuat Qiskit)")
    parser.add_argument("--problem", "-p", nargs="*", default=None,
                        help="Specific problem(s) to test (e.g. H2_4q Heisenberg_6q)")
    parser.add_argument("--config", "-c", nargs="*", type=int, default=None,
                        help="Config indices 0-4 (0=10s1, 1=10s2, 2=50s1, 3=50s2, 4=100s2)")
    args = parser.parse_args()

    # Ensure MyQuat binary is built
    if FRAMEWORKS["myquat"]:
        print(f"MyQuat binary: {MYQUAT_BIN}")
    else:
        print(f"WARNING: MyQuat binary not found at {MYQUAT_BIN}")
        print("Build with: cargo build --release --example benchmark_hamiltonian")
        if not any(FRAMEWORKS.values()):
            print("ERROR: No frameworks available. Aborting.")
            return 1
        print()

    results = run_all_benchmarks(
        filter_frameworks=args.framework,
        filter_problems=args.problem,
        filter_configs=args.config,
    )
    save_results(results)

    print(f"\nCompleted: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
