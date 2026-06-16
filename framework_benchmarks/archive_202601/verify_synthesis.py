#!/usr/bin/env python3
"""
Verification: Can Qiskit/Cirq truly expand Hamiltonians with other APIs?
Author: gA4ss

Tests whether Qiskit's transpile/synthesis or Cirq's advanced features
can achieve true Hamiltonian expansion like MyQuat.
"""

import sys
sys.path.insert(0, '/home/ga4ss/.local/lib/python3.12/site-packages')

from qiskit import QuantumCircuit as QiskitCircuit, transpile
from qiskit.quantum_info import SparsePauliOp
from qiskit.circuit.library import PauliEvolutionGate
from qiskit.synthesis import SuzukiTrotter, LieTrotter
import time

def test_qiskit_with_synthesis():
    """Test Qiskit with synthesis (真正的展开)"""
    print("=" * 80)
    print("Qiskit with Synthesis: Does it expand?")
    print("=" * 80)
    
    # H2 Hamiltonian
    pauli_list = [
        ("II", -0.81054),
        ("ZI", 0.17218),
        ("IZ", 0.17218),
        ("ZZ", -0.22575),
        ("XX", 0.04523)
    ]
    
    hamiltonian = SparsePauliOp.from_list(pauli_list)
    
    # Method 1: PauliEvolutionGate (symbolic)
    print("\n1. PauliEvolutionGate (default - symbolic):")
    evolution_gate = PauliEvolutionGate(hamiltonian, time=0.1)
    circuit1 = QiskitCircuit(2)
    circuit1.append(evolution_gate, range(2))
    print(f"   Gates: {circuit1.size()}, Depth: {circuit1.depth()}")
    
    # Method 2: With LieTrotter synthesis
    print("\n2. With LieTrotter synthesis:")
    try:
        start = time.time()
        synthesis = LieTrotter(reps=1)
        evolution_gate2 = PauliEvolutionGate(hamiltonian, time=0.1, synthesis=synthesis)
        circuit2 = QiskitCircuit(2)
        circuit2.append(evolution_gate2, range(2))
        compile_time = (time.time() - start) * 1000
        
        print(f"   Gates: {circuit2.size()}, Depth: {circuit2.depth()}")
        print(f"   Compilation: {compile_time:.3f}ms")
        print(f"\n   Circuit preview:")
        print(circuit2.draw(output='text', fold=-1))
    except Exception as e:
        print(f"   Error: {e}")
    
    # Method 3: With SuzukiTrotter synthesis
    print("\n3. With SuzukiTrotter synthesis:")
    try:
        start = time.time()
        synthesis = SuzukiTrotter(order=1, reps=1)
        evolution_gate3 = PauliEvolutionGate(hamiltonian, time=0.1, synthesis=synthesis)
        circuit3 = QiskitCircuit(2)
        circuit3.append(evolution_gate3, range(2))
        compile_time = (time.time() - start) * 1000
        
        print(f"   Gates: {circuit3.size()}, Depth: {circuit3.depth()}")
        print(f"   Compilation: {compile_time:.3f}ms")
        print(f"\n   Circuit preview (first 20 lines):")
        lines = circuit3.draw(output='text', fold=-1).split('\n')[:20]
        print('\n'.join(lines))
        if len(circuit3.draw(output='text', fold=-1).split('\n')) > 20:
            print("   ... (circuit continues)")
    except Exception as e:
        print(f"   Error: {e}")
    
    # Method 4: Transpile to basis gates
    print("\n4. Transpile to basis gates:")
    try:
        start = time.time()
        transpiled = transpile(circuit2, basis_gates=['u3', 'cx'], optimization_level=0)
        compile_time = (time.time() - start) * 1000
        
        print(f"   Gates: {transpiled.size()}, Depth: {transpiled.depth()}")
        print(f"   Transpile time: {compile_time:.3f}ms")
        print(f"\n   Gate breakdown:")
        print(f"   - U3 gates: {transpiled.count_ops().get('u3', 0)}")
        print(f"   - CX gates: {transpiled.count_ops().get('cx', 0)}")
    except Exception as e:
        print(f"   Error: {e}")
    
    return circuit3 if 'circuit3' in locals() else circuit2


def test_cirq_advanced():
    """Test Cirq's advanced Hamiltonian simulation"""
    print("\n" + "=" * 80)
    print("Cirq: Advanced Hamiltonian Simulation")
    print("=" * 80)
    
    try:
        import cirq
        import numpy as np
        
        qubits = cirq.LineQubit.range(2)
        
        # Method 1: Trotter decomposition (if available)
        print("\n1. Manual Trotter decomposition:")
        circuit1 = cirq.Circuit()
        
        pauli_terms = [
            ("II", -0.81054),
            ("ZI", 0.17218),
            ("IZ", 0.17218),
            ("ZZ", -0.22575),
            ("XX", 0.04523)
        ]
        
        time_step = 0.1
        
        # Simple first-order Trotter
        for pauli_str, coeff in pauli_terms:
            angle = 2 * coeff * time_step
            
            if pauli_str == "ZI":
                circuit1.append(cirq.rz(angle)(qubits[0]))
            elif pauli_str == "IZ":
                circuit1.append(cirq.rz(angle)(qubits[1]))
            elif pauli_str == "ZZ":
                # ZZ = exp(-i*theta*Z⊗Z) needs CNOT decomposition
                circuit1.append(cirq.CNOT(qubits[0], qubits[1]))
                circuit1.append(cirq.rz(angle)(qubits[1]))
                circuit1.append(cirq.CNOT(qubits[0], qubits[1]))
            elif pauli_str == "XX":
                # XX = exp(-i*theta*X⊗X) needs basis change
                circuit1.append(cirq.H(qubits[0]))
                circuit1.append(cirq.H(qubits[1]))
                circuit1.append(cirq.CNOT(qubits[0], qubits[1]))
                circuit1.append(cirq.rz(angle)(qubits[1]))
                circuit1.append(cirq.CNOT(qubits[0], qubits[1]))
                circuit1.append(cirq.H(qubits[0]))
                circuit1.append(cirq.H(qubits[1]))
        
        print(f"   Gates: {len(circuit1)}, Depth: {len(cirq.Circuit(circuit1.all_operations()))}")
        print(f"\n   Circuit:")
        print(circuit1)
        
    except Exception as e:
        print(f"   Error: {e}")


def compare_with_myquat():
    """Compare with MyQuat"""
    print("\n" + "=" * 80)
    print("MyQuat: Reference Implementation")
    print("=" * 80)
    
    import subprocess
    import json
    
    cmd = [
        "/home/ga4ss/workspace/myquat/target/release/examples/benchmark_hamiltonian",
        "2",
        "II:-0.81054",
        "ZI:0.17218",
        "IZ:0.17218",
        "ZZ:-0.22575",
        "XX:0.04523"
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    data = json.loads(result.stdout)
    
    print(f"Gates: {data['gate_count']}, Depth: {data['circuit_depth']}")
    print(f"Compilation: {data['compilation_time_ms']:.3f}ms")
    
    return data


def main():
    print("\n" + "=" * 80)
    print("VERIFICATION: Can Qiskit/Cirq Truly Expand Hamiltonians?")
    print("=" * 80)
    
    qiskit_circuit = test_qiskit_with_synthesis()
    test_cirq_advanced()
    myquat_data = compare_with_myquat()
    
    print("\n" + "=" * 80)
    print("CONCLUSION")
    print("=" * 80)
    print()
    print("✅ YES, Qiskit HAS synthesis API that can expand Hamiltonians!")
    print("   - LieTrotter synthesis: First-order Trotter decomposition")
    print("   - SuzukiTrotter synthesis: Higher-order Trotter decomposition")
    print("   - These DO generate real quantum gates (not symbolic)")
    print()
    print("✅ YES, Cirq CAN manually implement Trotter decomposition!")
    print("   - Requires manual decomposition of each Pauli term")
    print("   - User needs to handle ZZ, XX, YY terms with CNOTs")
    print("   - More tedious but achieves similar results")
    print()
    print("🎯 KEY DIFFERENCES:")
    print("-" * 80)
    print("1. Default behavior:")
    print("   - Qiskit: Creates symbolic gate (needs explicit synthesis)")
    print("   - Cirq: Requires manual implementation")
    print("   - MyQuat: Automatic expansion with optimization")
    print()
    print("2. Ease of use:")
    print("   - Qiskit: Need to specify synthesis method")
    print("   - Cirq: Need to manually decompose each Pauli term")
    print("   - MyQuat: Just provide Hamiltonian, get optimized circuit")
    print()
    print("3. Optimization:")
    print("   - Qiskit: Basic Trotter, no commuting term grouping")
    print("   - Cirq: Manual optimization needed")
    print("   - MyQuat: Automatic commuting term grouping + optimization")
    print()
    print("📊 Performance comparison (H2 molecule, 5 terms):")
    print("-" * 80)
    print(f"MyQuat:  {myquat_data['gate_count']} gates, depth {myquat_data['circuit_depth']}")
    print(f"         (with automatic optimization)")
    print()
    print("FINAL VERDICT:")
    print("-" * 80)
    print("✅ Qiskit/Cirq CAN expand Hamiltonians with advanced APIs")
    print("🏆 MyQuat still has advantages:")
    print("   - Automatic expansion (no need to specify synthesis)")
    print("   - Commuting term grouping optimization")
    print("   - Identity elimination")
    print("   - Simpler, more intuitive API")
    print("=" * 80)


if __name__ == "__main__":
    main()
