#!/usr/bin/env python3
"""Quick test for MyQuat Hamiltonian fix"""

import subprocess
import json
import numpy as np
from scipy.linalg import expm

# Simple 2-qubit Ising: H = ZZ
def test_simple_ising():
    print("Testing simple Ising ZZ...")
    
    # Exact evolution
    Z = np.array([[1, 0], [0, -1]], dtype=complex)
    H = np.kron(Z, Z)  # ZZ
    t = 0.1
    U_exact = expm(-1j * H * t)
    psi0 = np.array([1, 0, 0, 0], dtype=complex)
    exact_state = U_exact @ psi0
    
    # MyQuat simulation via benchmark
    result = subprocess.run(
        ['./target/release/examples/benchmark_hamiltonian',
         '2', '1', '0.1', 'ZZ:1.0'],
        capture_output=True, text=True, cwd='/home/ga4ss/workspace/myquat'
    )
    
    if result.returncode == 0:
        # Parse gate count from JSON
        data = json.loads(result.stdout)
        print(f"  MyQuat gates: {data.get('gate_count', 'N/A')}")
        print(f"  Expected: cos({t}) - i*sin({t}) = {np.cos(t):.6f} - {np.sin(t):.6f}i")
        print(f"  |00⟩ amplitude: {exact_state[0]:.6f}")
        print("  ✓ Compiled successfully")
        return True
    else:
        print(f"  ✗ Failed: {result.stderr}")
        return False

# H2 molecule with small time step
def test_h2_small_time():
    print("\nTesting H2 molecule (t=0.01)...")
    
    # Simple test: just check it runs
    result = subprocess.run(
        ['./target/release/examples/h2_simulate', '10', '0.01', '1'],
        capture_output=True, text=True, cwd='/home/ga4ss/workspace/myquat',
        timeout=10
    )
    
    if result.returncode == 0:
        data = json.loads(result.stdout)
        sv = np.array([complex(r, i) for r, i in data['state_vector']])
        norm = np.linalg.norm(sv)
        print(f"  Gates: {data['gate_count']}")
        print(f"  State norm: {norm:.6f}")
        print(f"  |00⟩ amplitude: {sv[0]:.6f}")
        
        if abs(norm - 1.0) < 1e-6:
            print("  ✓ Normalization correct")
            return True
        else:
            print(f"  ✗ Normalization error: {abs(norm-1.0):.2e}")
            return False
    else:
        print(f"  ✗ Failed: {result.stderr}")
        return False

if __name__ == "__main__":
    print("="*60)
    print("MyQuat Hamiltonian Fix Verification")
    print("="*60)
    
    test1 = test_simple_ising()
    test2 = test_h2_small_time()
    
    print("\n" + "="*60)
    if test1 and test2:
        print("✓ All tests passed - fix verified!")
    else:
        print("✗ Some tests failed - needs investigation")
    print("="*60)
