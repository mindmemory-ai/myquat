#!/usr/bin/env python3
"""
Qiskit H₂ Molecule Hamiltonian Simulation Benchmark
Author: gA4ss

Benchmark for Trotter-Suzuki decomposition of H₂ molecule using Qiskit.
"""

import time
import numpy as np
from qiskit import QuantumCircuit
from qiskit.circuit.library import PauliEvolutionGate
from qiskit.quantum_info import SparsePauliOp
from qiskit.synthesis import SuzukiTrotter


def create_h2_hamiltonian():
    """
    Create H₂ molecule Hamiltonian (4 qubits, 15 terms).
    This is a simplified representation of H₂ at 0.74 Angstrom.
    """
    # Pauli strings and coefficients for H₂
    # These coefficients are approximate for demonstration
    pauli_list = [
        ('IIII', -0.8105),  # Identity term (energy offset)
        ('IIIZ', 0.1721),   # Single Z terms
        ('IIZI', -0.2228),
        ('IZII', 0.1721),
        ('ZIII', -0.2228),
        ('IIZZ', 0.1686),   # ZZ interactions
        ('IZIZ', 0.1205),
        ('IZZI', 0.1686),
        ('ZIIZ', 0.1686),
        ('ZIZI', 0.1205),
        ('ZZII', 0.1686),
        ('IIXX', 0.0454),   # XX interactions
        ('IIYY', 0.0454),
        ('IXIX', 0.0454),
        ('IYIY', 0.0454),
    ]
    
    return SparsePauliOp.from_list(pauli_list)


def benchmark_qiskit_h2(num_trotter_steps=100, evolution_time=1.0, order=2):
    """
    Benchmark Qiskit Hamiltonian simulation for H₂.
    
    Args:
        num_trotter_steps: Number of Trotter steps
        evolution_time: Total evolution time
        order: Trotter order (1 or 2)
    
    Returns:
        dict: Benchmark results
    """
    # Create Hamiltonian
    hamiltonian = create_h2_hamiltonian()
    
    # Time the circuit compilation
    start_time = time.perf_counter()
    
    # Create quantum circuit
    qc = QuantumCircuit(4)
    
    # Create Suzuki-Trotter synthesizer
    if order == 1:
        synthesizer = SuzukiTrotter(order=1, reps=num_trotter_steps)
    else:
        synthesizer = SuzukiTrotter(order=2, reps=num_trotter_steps)
    
    # Create evolution gate
    evolution_gate = PauliEvolutionGate(
        hamiltonian,
        time=evolution_time,
        synthesis=synthesizer
    )
    
    # Add to circuit
    qc.append(evolution_gate, range(4))
    
    # Decompose to get actual gates
    qc = qc.decompose()
    qc = qc.decompose()  # Decompose twice to get to native gates
    
    end_time = time.perf_counter()
    compilation_time = (end_time - start_time) * 1000  # Convert to ms
    
    # Count gates
    gate_count = len(qc.data)
    
    # Calculate circuit depth
    circuit_depth = qc.depth()
    
    results = {
        'framework': 'Qiskit',
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
    print("Qiskit H₂ Hamiltonian Simulation Benchmark")
    print("=" * 60)
    print()
    
    # Run benchmark
    results = benchmark_qiskit_h2(num_trotter_steps=100, evolution_time=1.0, order=2)
    
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
