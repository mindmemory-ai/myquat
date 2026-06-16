#!/usr/bin/env python3
"""
PennyLane H₂ Molecule Hamiltonian Simulation Benchmark
Author: gA4ss

Benchmark for Trotter-Suzuki decomposition of H₂ molecule using PennyLane.
"""

import time
import numpy as np
import pennylane as qml
from pennylane import qchem


def create_h2_hamiltonian():
    """
    Create H₂ molecule Hamiltonian (4 qubits, 15 terms).
    Returns PennyLane Hamiltonian object.
    """
    # Pauli strings and coefficients for H₂
    coeffs = [
        -0.8105,  # IIII
        0.1721,   # IIIZ
        -0.2228,  # IIZI
        0.1721,   # IZII
        -0.2228,  # ZIII
        0.1686,   # IIZZ
        0.1205,   # IZIZ
        0.1686,   # IZZI
        0.1686,   # ZIIZ
        0.1205,   # ZIZI
        0.1686,   # ZZII
        0.0454,   # IIXX
        0.0454,   # IIYY
        0.0454,   # IXIX
        0.0454,   # IYIY
    ]
    
    observables = [
        qml.Identity(0),                      # IIII
        qml.PauliZ(3),                        # IIIZ
        qml.PauliZ(2),                        # IIZI
        qml.PauliZ(1),                        # IZII
        qml.PauliZ(0),                        # ZIII
        qml.PauliZ(2) @ qml.PauliZ(3),       # IIZZ
        qml.PauliZ(1) @ qml.PauliZ(3),       # IZIZ
        qml.PauliZ(1) @ qml.PauliZ(2),       # IZZI
        qml.PauliZ(0) @ qml.PauliZ(3),       # ZIIZ
        qml.PauliZ(0) @ qml.PauliZ(2),       # ZIZI
        qml.PauliZ(0) @ qml.PauliZ(1),       # ZZII
        qml.PauliX(2) @ qml.PauliX(3),       # IIXX
        qml.PauliY(2) @ qml.PauliY(3),       # IIYY
        qml.PauliX(1) @ qml.PauliX(3),       # IXIX
        qml.PauliY(1) @ qml.PauliY(3),       # IYIY
    ]
    
    return qml.Hamiltonian(coeffs, observables)


def benchmark_pennylane_h2(num_trotter_steps=100, evolution_time=1.0, order=2):
    """
    Benchmark PennyLane Hamiltonian simulation for H₂.
    
    Args:
        num_trotter_steps: Number of Trotter steps
        evolution_time: Total evolution time
        order: Trotter order (1 or 2)
    
    Returns:
        dict: Benchmark results
    """
    # Create Hamiltonian
    hamiltonian = create_h2_hamiltonian()
    
    # Create device
    dev = qml.device('default.qubit', wires=4)
    
    # Time the circuit compilation
    start_time = time.perf_counter()
    
    @qml.qnode(dev)
    def circuit():
        # Apply Trotter approximation
        qml.TrotterProduct(
            hamiltonian,
            time=evolution_time,
            n=num_trotter_steps,
            order=order
        )
        return qml.state()
    
    # Get the circuit tape to count operations
    with qml.tape.QuantumTape() as tape:
        qml.TrotterProduct(
            hamiltonian,
            time=evolution_time,
            n=num_trotter_steps,
            order=order
        )
    
    # Expand to get actual gates
    expanded = tape.expand(depth=3)
    
    end_time = time.perf_counter()
    compilation_time = (end_time - start_time) * 1000  # Convert to ms
    
    # Count gates (operations)
    gate_count = len(expanded.operations)
    
    # PennyLane doesn't have direct depth calculation, estimate from tape
    circuit_depth = gate_count  # Simplified estimate
    
    results = {
        'framework': 'PennyLane',
        'language': 'Python',
        'compilation_time_ms': compilation_time,
        'gate_count': gate_count,
        'circuit_depth': circuit_depth,
        'num_qubits': 4,
        'num_terms': 15,
        'trotter_steps': num_trotter_steps,
        'trotter_order': order,
    }
    
    return results


def main():
    """Run benchmark and print results."""
    print("=" * 60)
    print("PennyLane H₂ Hamiltonian Simulation Benchmark")
    print("=" * 60)
    print()
    
    # Run benchmark
    results = benchmark_pennylane_h2(num_trotter_steps=100, evolution_time=1.0, order=2)
    
    # Print results
    print(f"Framework:         {results['framework']}")
    print(f"Language:          {results['language']}")
    print(f"Compilation Time:  {results['compilation_time_ms']:.2f} ms")
    print(f"Gate Count:        {results['gate_count']}")
    print(f"Circuit Depth:     {results['circuit_depth']}")
    print(f"Qubits:            {results['num_qubits']}")
    print(f"Hamiltonian Terms: {results['num_terms']}")
    print(f"Trotter Steps:     {results['trotter_steps']}")
    print(f"Trotter Order:     {results['trotter_order']}")
    print()
    
    return results


if __name__ == '__main__':
    main()
