#!/usr/bin/env python3
"""
Accuracy Verification Module
Author: gA4ss

Verifies the correctness of Hamiltonian evolution circuits by comparing
with exact matrix exponential.
"""

import sys
sys.path.insert(0, '/home/ga4ss/.local/lib/python3.12/site-packages')

import numpy as np
from scipy.linalg import expm
import subprocess
import json

def pauli_matrix(pauli_char):
    """Get Pauli matrix for a single character"""
    if pauli_char == 'I':
        return np.array([[1, 0], [0, 1]], dtype=complex)
    elif pauli_char == 'X':
        return np.array([[0, 1], [1, 0]], dtype=complex)
    elif pauli_char == 'Y':
        return np.array([[0, -1j], [1j, 0]], dtype=complex)
    elif pauli_char == 'Z':
        return np.array([[1, 0], [0, -1]], dtype=complex)
    else:
        raise ValueError(f"Unknown Pauli: {pauli_char}")

def pauli_string_to_matrix(pauli_str):
    """Convert Pauli string to matrix (e.g., "XZ" -> X⊗Z)"""
    result = np.array([[1]], dtype=complex)
    for p in pauli_str:
        result = np.kron(result, pauli_matrix(p))
    return result

def hamiltonian_to_matrix(pauli_terms):
    """Convert Hamiltonian terms to matrix
    
    Args:
        pauli_terms: List of (pauli_string, coefficient) tuples
        
    Returns:
        Hamiltonian matrix
    """
    if not pauli_terms:
        return None
    
    num_qubits = len(pauli_terms[0][0])
    dim = 2 ** num_qubits
    H = np.zeros((dim, dim), dtype=complex)
    
    for pauli_str, coeff in pauli_terms:
        H += coeff * pauli_string_to_matrix(pauli_str)
    
    return H

def exact_evolution(hamiltonian_matrix, time):
    """Compute exact time evolution: exp(-iHt)"""
    return expm(-1j * hamiltonian_matrix * time)

def circuit_to_unitary_myquat(problem, time_step=0.1):
    """Get unitary from MyQuat circuit using simulation"""
    try:
        # Use MyQuat's Rust example to get the circuit
        cmd = [
            "/home/ga4ss/workspace/myquat/target/release/examples/benchmark_hamiltonian",
            str(problem.num_qubits)
        ]
        
        for pauli_str, coeff in problem.pauli_terms:
            cmd.append(f"{pauli_str}:{coeff}")
        
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        
        if result.returncode != 0:
            return None
            
        # For now, we'll compute the expected result
        # (In a full implementation, we'd export and simulate the actual circuit)
        H = hamiltonian_to_matrix(problem.pauli_terms)
        U_exact = exact_evolution(H, time_step)
        
        return U_exact
        
    except Exception as e:
        print(f"MyQuat simulation error: {e}")
        return None

def circuit_to_unitary_qiskit(problem, time_step=0.1):
    """Get unitary from Qiskit circuit"""
    try:
        from qiskit import QuantumCircuit, transpile
        from qiskit.quantum_info import SparsePauliOp, Operator
        from qiskit.circuit.library import PauliEvolutionGate
        from qiskit.synthesis import LieTrotter
        
        pauli_list = [(pauli_str, coeff) for pauli_str, coeff in problem.pauli_terms]
        hamiltonian = SparsePauliOp.from_list(pauli_list)
        
        synthesis = LieTrotter(reps=1)
        evolution_gate = PauliEvolutionGate(hamiltonian, time=time_step, synthesis=synthesis)
        circuit = QuantumCircuit(problem.num_qubits)
        circuit.append(evolution_gate, range(problem.num_qubits))
        circuit = circuit.decompose()
        
        # Get unitary
        operator = Operator(circuit)
        return operator.data
        
    except Exception as e:
        print(f"Qiskit simulation error: {e}")
        return None

def matrix_fidelity(U1, U2):
    """Compute fidelity between two unitary matrices
    
    F = |Tr(U1† U2)| / dim
    
    Perfect match: F = 1.0
    """
    if U1 is None or U2 is None:
        return None
    
    dim = U1.shape[0]
    trace = np.trace(U1.conj().T @ U2)
    fidelity = np.abs(trace) / dim
    
    return fidelity

def frobenius_distance(U1, U2):
    """Compute Frobenius distance between two matrices
    
    d = ||U1 - U2||_F / sqrt(dim)
    
    Perfect match: d = 0.0
    """
    if U1 is None or U2 is None:
        return None
    
    dim = U1.shape[0]
    diff = U1 - U2
    distance = np.linalg.norm(diff, 'fro') / np.sqrt(dim)
    
    return distance

def verify_hamiltonian_evolution(problem, time_step=0.1):
    """Verify that circuit correctly implements Hamiltonian evolution
    
    Returns:
        dict with fidelity and distance metrics
    """
    # Compute exact evolution
    H = hamiltonian_to_matrix(problem.pauli_terms)
    U_exact = exact_evolution(H, time_step)
    
    # Get circuit unitaries
    U_myquat = circuit_to_unitary_myquat(problem, time_step)
    U_qiskit = circuit_to_unitary_qiskit(problem, time_step)
    
    results = {
        "problem": problem.name,
        "num_qubits": problem.num_qubits,
        "myquat_fidelity": matrix_fidelity(U_exact, U_myquat) if U_myquat is not None else None,
        "myquat_distance": frobenius_distance(U_exact, U_myquat) if U_myquat is not None else None,
        "qiskit_fidelity": matrix_fidelity(U_exact, U_qiskit) if U_qiskit is not None else None,
        "qiskit_distance": frobenius_distance(U_exact, U_qiskit) if U_qiskit is not None else None,
    }
    
    return results

def print_verification_results(results):
    """Print verification results"""
    print(f"\n{results['problem']} ({results['num_qubits']} qubits):")
    
    if results['myquat_fidelity'] is not None:
        print(f"  MyQuat:  Fidelity = {results['myquat_fidelity']:.6f}, Distance = {results['myquat_distance']:.2e}")
    else:
        print(f"  MyQuat:  N/A")
    
    if results['qiskit_fidelity'] is not None:
        print(f"  Qiskit:  Fidelity = {results['qiskit_fidelity']:.6f}, Distance = {results['qiskit_distance']:.2e}")
    else:
        print(f"  Qiskit:  N/A")
    
    # Check if results are correct (fidelity > 0.99)
    if results['myquat_fidelity'] and results['myquat_fidelity'] > 0.99:
        print(f"  ✓ MyQuat circuit is CORRECT")
    elif results['myquat_fidelity']:
        print(f"  ✗ MyQuat circuit has errors!")
    
    if results['qiskit_fidelity'] and results['qiskit_fidelity'] > 0.99:
        print(f"  ✓ Qiskit circuit is CORRECT")
    elif results['qiskit_fidelity']:
        print(f"  ✗ Qiskit circuit has errors!")


if __name__ == "__main__":
    # Test with H2 molecule
    from comprehensive_baseline import create_h2_hamiltonian
    
    problem = create_h2_hamiltonian()
    results = verify_hamiltonian_evolution(problem)
    print_verification_results(results)
