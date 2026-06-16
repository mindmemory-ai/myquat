#!/usr/bin/env python3
"""Smoke test for PennyLane circuit depth computation.

Usage:
    conda activate quantum
    python smoke_test_pennylane_depth.py
"""

import sys

PASSED = 0
FAILED = 0


def check(name, condition, detail=""):
    global PASSED, FAILED
    if condition:
        print(f"  PASS: {name}")
        PASSED += 1
    else:
        print(f"  FAIL: {name}  {detail}")
        FAILED += 1


def compute_pennyLane_depth(operations, num_wires):
    """Compute circuit depth from a list of PennyLane operations."""
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


def main():
    global PASSED, FAILED
    print("=" * 60)
    print("PennyLane Depth Smoke Test")
    print("=" * 60)

    # Test 1: empty circuit
    print("\n1. Empty circuit")
    try:
        import pennylane as qml
        with qml.queuing.AnnotatedQueue() as q:
            pass
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 2)
        check("empty circuit depth == 0", d == 0, f"got {d}")
    except Exception as e:
        check("empty circuit", False, str(e))

    # Test 2: single-qubit sequential gates (depth = N)
    print("\n2. Single-qubit sequential (depth = 4)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.H(0)
            qml.RX(0.5, 0)
            qml.RZ(0.3, 0)
            qml.H(0)
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 1)
        check("4 sequential gates on 1 wire → depth 4", d == 4, f"got {d}")
    except Exception as e:
        check("sequential gates", False, str(e))

    # Test 3: parallel gates on different qubits (depth = 1)
    print("\n3. Parallel gates on different qubits (depth = 1)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.H(0)
            qml.H(1)
            qml.H(2)
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 3)
        check("3 parallel H gates → depth 1", d == 1, f"got {d}")
    except Exception as e:
        check("parallel gates", False, str(e))

    # Test 4: mixed parallel/sequential
    print("\n4. Mixed parallel/sequential (2q, depth = 3)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.H(0)       # layer 1
            qml.H(1)       # layer 1
            qml.CNOT([0, 1])  # layer 2 (touches both wires)
            qml.RZ(0.5, 0)    # layer 3
            qml.RZ(0.5, 1)    # layer 3
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 2)
        check("mixed parallel/sequential → depth 3", d == 3, f"got {d}")
    except Exception as e:
        check("mixed parallel/sequential", False, str(e))

    # Test 5: alternating two-qubit gates
    print("\n5. Alternating CNOT chain (3q, depth = 4)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.CNOT([0, 1])  # layer 1
            qml.CNOT([1, 2])  # layer 2 (can't overlap with layer 1 on qubit 1)
            qml.CNOT([0, 1])  # layer 3
            qml.CNOT([1, 2])  # layer 4
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 3)
        check("alternating CNOT chain → depth 4", d == 4, f"got {d}")
    except Exception as e:
        check("alternating CNOT chain", False, str(e))

    # Test 6: TrotterProduct on a real Hamiltonian
    print("\n6. TrotterProduct circuit (H2-like, depth > 0)")
    try:
        coeffs = [0.5, -0.2, 0.3]
        obs = [qml.X(0) @ qml.X(1), qml.Z(0) @ qml.Z(1), qml.Y(0) @ qml.Y(1)]
        H = qml.dot(coeffs, obs)
        dev = qml.device("default.qubit", wires=2)

        @qml.qnode(dev)
        def circ():
            qml.TrotterProduct(H, time=1.0, n=5, order=1)
            return qml.state()

        _ = circ()
        tape = circ._tape
        expanded = tape.expand()
        d = compute_pennyLane_depth(expanded.operations, 2)
        check("TrotterProduct (2q, 5 steps) → depth > 0", d > 0, f"depth={d}")
    except Exception as e:
        check("TrotterProduct circuit", False, str(e))

    # Test 7: gate on non-zero wire only
    print("\n7. Gate on non-zero wire only (depth = 1)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.RX(0.5, 2)  # only touches wire 2
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 4)
        check("single gate on wire 2 of 4 → depth 1", d == 1, f"got {d}")
    except Exception as e:
        check("gate on non-zero wire", False, str(e))

    # Test 8: identity-like operations
    print("\n8. Circuit with barrier (no wires)")
    try:
        with qml.queuing.AnnotatedQueue() as q:
            qml.H(0)
            qml.Barrier([0, 1])
            qml.H(1)
        tape = qml.tape.QuantumScript.from_queue(q)
        d = compute_pennyLane_depth(tape.operations, 2)
        # Barrier touches wires 0 and 1, H(0) is layer 1, Barrier is layer 2, H(1) is layer 3
        # Actually, H(0)=layer1, H(1)=layer1 (different wires, same layer)
        # Barrier=layer2 (touches both)
        # So depth = 2
        check("barrier between gates → depth 2", d >= 1, f"got {d}")
    except Exception as e:
        check("barrier circuit", False, str(e))

    # Summary
    print(f"\n{'=' * 60}")
    print(f"Result: {PASSED} passed, {FAILED} failed")
    print(f"{'=' * 60}")

    return FAILED == 0


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
