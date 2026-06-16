#!/usr/bin/env python3
"""Verify MyQuat H2 simulation with correct parameters"""

import subprocess
import json
import numpy as np
from scipy.linalg import expm

def get_h2_hamiltonian_matrix():
    """Get H2 Hamiltonian as 16x16 matrix"""
    I = np.array([[1, 0], [0, 1]], dtype=complex)
    X = np.array([[0, 1], [1, 0]], dtype=complex)
    Y = np.array([[0, -1j], [1j, 0]], dtype=complex)
    Z = np.array([[1, 0], [0, -1]], dtype=complex)
    
    pauli_map = {'I': I, 'X': X, 'Y': Y, 'Z': Z}
    
    hamiltonian_terms = [
        ("IIII", -0.8105), ("IIIZ", 0.1721), ("IIZI", -0.2228),
        ("IZII", 0.1721), ("ZIII", -0.2228), ("IIZZ", 0.1686),
        ("IZIZ", 0.1205), ("IZZI", 0.1686), ("ZIIZ", 0.1686),
        ("ZIZI", 0.1205), ("ZZII", 0.1686), ("IIXX", 0.0454),
        ("IIYY", 0.0454), ("IXIX", 0.0454), ("IYIY", 0.0454),
    ]
    
    H = np.zeros((16, 16), dtype=complex)
    for pauli_str, coeff in hamiltonian_terms:
        term = pauli_map[pauli_str[0]]
        for p in pauli_str[1:]:
            term = np.kron(term, pauli_map[p])
        H += coeff * term
    
    return H

def test_myquat_order1():
    """Test MyQuat with first-order Trotter (same as Qiskit in verify_accuracy.py)"""
    print("Testing MyQuat H2 simulation (First-order Trotter)...")
    print("="*60)
    
    num_steps = 100
    evolution_time = 1.0
    order = 1  # First-order to match Qiskit
    
    print(f"Parameters: steps={num_steps}, time={evolution_time}, order={order}")
    
    # Exact evolution
    H = get_h2_hamiltonian_matrix()
    psi0 = np.zeros(16, dtype=complex)
    psi0[0] = 1.0
    U_exact = expm(-1j * H * evolution_time)
    exact_state = U_exact @ psi0
    exact_energy = np.real(np.vdot(exact_state, H @ exact_state))
    
    print(f"Exact energy: {exact_energy:.6f}")
    
    # MyQuat simulation
    result = subprocess.run(
        ['./target/release/examples/h2_simulate', str(num_steps), str(evolution_time), str(order)],
        capture_output=True, text=True,
        cwd='/home/ga4ss/workspace/myquat',
        timeout=30
    )
    
    if result.returncode != 0:
        print(f"✗ MyQuat failed: {result.stderr}")
        return False
    
    data = json.loads(result.stdout)
    sv = np.array([complex(r, i) for r, i in data['state_vector']])
    
    # Normalize
    sv = sv / np.linalg.norm(sv)
    exact_state = exact_state / np.linalg.norm(exact_state)
    
    # Compute fidelity
    fidelity = np.abs(np.vdot(exact_state, sv))**2
    
    # Compute energy
    energy = np.real(np.vdot(sv, H @ sv))
    energy_error = abs(energy - exact_energy)
    
    print(f"MyQuat gates: {data['gate_count']}")
    print(f"MyQuat energy: {energy:.6f}")
    print(f"Fidelity: {fidelity:.10f}")
    print(f"Energy error: {energy_error:.2e}")
    
    if fidelity > 0.999:
        print("✓ PASS: High fidelity achieved!")
        return True
    else:
        print(f"✗ FAIL: Fidelity {fidelity:.6f} < 0.999")
        return False

if __name__ == "__main__":
    success = test_myquat_order1()
    print("="*60)
    if success:
        print("✓ MyQuat H2 simulation verified!")
    else:
        print("✗ MyQuat H2 simulation needs investigation")
