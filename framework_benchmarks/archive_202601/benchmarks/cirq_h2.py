#!/usr/bin/env python3
"""
Cirq H₂ Molecule Hamiltonian Simulation Benchmark
Author: gA4ss

Benchmark for Trotter-Suzuki decomposition of H₂ molecule using Cirq.
"""

import time
import numpy as np
import cirq
from cirq.ops import X, Y, Z


def create_h2_hamiltonian_terms():
    """
    Create H₂ molecule Hamiltonian terms (4 qubits, 15 terms).
    Returns list of (pauli_string, coefficient) tuples.
    """
    return [
        ('IIII', -0.8105),
        ('IIIZ', 0.1721),
        ('IIZI', -0.2228),
        ('IZII', 0.1721),
        ('ZIII', -0.2228),
        ('IIZZ', 0.1686),
        ('IZIZ', 0.1205),
        ('IZZI', 0.1686),
        ('ZIIZ', 0.1686),
        ('ZIZI', 0.1205),
        ('ZZII', 0.1686),
        ('IIXX', 0.0454),
        ('IIYY', 0.0454),
        ('IXIX', 0.0454),
        ('IYIY', 0.0454),
    ]


def pauli_string_to_cirq(pauli_str, qubits, coeff):
    """Convert Pauli string to Cirq PauliString."""
    ops = []
    for i, p in enumerate(pauli_str):
        if p == 'X':
            ops.append(X(qubits[i]))
        elif p == 'Y':
            ops.append(Y(qubits[i]))
        elif p == 'Z':
            ops.append(Z(qubits[i]))
        # I is identity, skip
    
    if not ops:
        return None  # Identity term
    
    return cirq.PauliString(*ops, coefficient=coeff)


def create_trotter_step(hamiltonian_terms, qubits, dt, order=2):
    """
    Create a single Trotter step circuit.
    
    Args:
        hamiltonian_terms: List of (pauli_string, coefficient) tuples
        qubits: List of qubits
        dt: Time step
        order: Trotter order (1 or 2)
    
    Returns:
        List of cirq operations
    """
    ops = []
    
    if order == 1:
        # First-order: exp(-iH*dt) ≈ exp(-iH1*dt) * exp(-iH2*dt) * ...
        for pauli_str, coeff in hamiltonian_terms:
            if pauli_str == 'IIII':
                continue  # Skip identity
            # Create PauliString with coefficient=1, use exponent_neg for rotation angle
            ps = pauli_string_to_cirq(pauli_str, qubits, 1.0)
            if ps is not None:
                # Rotation angle = coefficient * dt
                ops.append(cirq.PauliStringPhasor(ps, exponent_neg=coeff * dt / np.pi))
    
    elif order == 2:
        # Second-order: symmetric splitting
        # Forward half step
        for pauli_str, coeff in hamiltonian_terms:
            if pauli_str == 'IIII':
                continue
            ps = pauli_string_to_cirq(pauli_str, qubits, 1.0)
            if ps is not None:
                ops.append(cirq.PauliStringPhasor(ps, exponent_neg=coeff * dt / 2 / np.pi))
        
        # Backward half step
        for pauli_str, coeff in reversed(hamiltonian_terms):
            if pauli_str == 'IIII':
                continue
            ps = pauli_string_to_cirq(pauli_str, qubits, 1.0)
            if ps is not None:
                ops.append(cirq.PauliStringPhasor(ps, exponent_neg=coeff * dt / 2 / np.pi))
    
    return ops


def benchmark_cirq_h2(num_trotter_steps=100, evolution_time=1.0, order=2):
    """
    Benchmark Cirq Hamiltonian simulation for H₂.
    
    Args:
        num_trotter_steps: Number of Trotter steps
        evolution_time: Total evolution time
        order: Trotter order (1 or 2)
    
    Returns:
        dict: Benchmark results
    """
    # Create qubits
    qubits = [cirq.LineQubit(i) for i in range(4)]
    
    # Get Hamiltonian terms
    hamiltonian_terms = create_h2_hamiltonian_terms()
    
    # Time the circuit compilation
    start_time = time.perf_counter()
    
    # Create circuit
    circuit = cirq.Circuit()
    
    # Time step
    dt = evolution_time / num_trotter_steps
    
    # Add Trotter steps
    for _ in range(num_trotter_steps):
        ops = create_trotter_step(hamiltonian_terms, qubits, dt, order)
        circuit.append(ops)
    
    # Decompose to native gates
    circuit = cirq.optimize_for_target_gateset(
        circuit,
        gateset=cirq.CZTargetGateset()
    )
    
    end_time = time.perf_counter()
    compilation_time = (end_time - start_time) * 1000  # Convert to ms
    
    # Count gates
    gate_count = len(list(circuit.all_operations()))
    
    # Calculate circuit depth
    circuit_depth = len(circuit)
    
    results = {
        'framework': 'Cirq',
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
    print("Cirq H₂ Hamiltonian Simulation Benchmark")
    print("=" * 60)
    print()
    
    # Run benchmark
    results = benchmark_cirq_h2(num_trotter_steps=100, evolution_time=1.0, order=2)
    
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
