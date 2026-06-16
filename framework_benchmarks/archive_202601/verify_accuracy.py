#!/usr/bin/env python3
"""
Accuracy Verification for Hamiltonian Simulation
Author: gA4ss

Validates that all quantum frameworks produce mathematically equivalent
results for H2 molecule Hamiltonian simulation.
"""

import sys
import numpy as np
from pathlib import Path

# Add benchmarks directory to path
sys.path.insert(0, str(Path(__file__).parent / 'benchmarks'))

# Framework imports
try:
    from qiskit import QuantumCircuit as QiskitCircuit
    from qiskit.quantum_info import Statevector, state_fidelity
    from qiskit_aer import AerSimulator
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False
    print("Warning: Qiskit not fully available")

try:
    import cirq
    CIRQ_AVAILABLE = True
except ImportError:
    CIRQ_AVAILABLE = False
    print("Warning: Cirq not available")

try:
    import pennylane as qml
    PENNYLANE_AVAILABLE = True
except Exception as e:
    PENNYLANE_AVAILABLE = False
    print(f"Warning: PennyLane not available: {e}")


def get_h2_hamiltonian_matrix():
    """Get the H2 Hamiltonian as a 16x16 matrix."""
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


def exact_evolution(t=1.0):
    """Compute exact time evolution using matrix exponentiation."""
    from scipy.linalg import expm
    H = get_h2_hamiltonian_matrix()
    psi0 = np.zeros(16, dtype=complex)
    psi0[0] = 1.0
    U = expm(-1j * H * t)
    return U @ psi0


def simulate_qiskit_circuit(num_steps=100, evolution_time=1.0, order=2):
    """Simulate Qiskit Hamiltonian evolution circuit."""
    if not QISKIT_AVAILABLE:
        return None
    
    print("\n[Qiskit] Building and simulating circuit...")
    
    try:
        from qiskit.quantum_info import SparsePauliOp
        
        # Create H2 Hamiltonian as SparsePauliOp
        pauli_list = [
            ("IIII", -0.8105), ("IIIZ", 0.1721), ("IIZI", -0.2228),
            ("IZII", 0.1721), ("ZIII", -0.2228), ("IIZZ", 0.1686),
            ("IZIZ", 0.1205), ("IZZI", 0.1686), ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205), ("ZZII", 0.1686), ("IIXX", 0.0454),
            ("IIYY", 0.0454), ("IXIX", 0.0454), ("IYIY", 0.0454),
        ]
        
        hamiltonian = SparsePauliOp.from_list(pauli_list)
        
        # Build circuit manually
        qc = QiskitCircuit(4)
        dt = evolution_time / num_steps
        
        # Helper to apply Pauli rotation
        def apply_pauli_rotation(pauli_str, coeff):
            # Find non-identity qubits
            targets = []
            paulis = []
            for i, p in enumerate(pauli_str):
                if p != 'I':
                    targets.append(i)
                    paulis.append(p)
            
            if not targets:
                return
            
            theta = 2.0 * coeff * dt
            
            # Basis change
            for i, p in enumerate(paulis):
                if p == 'X':
                    qc.h(targets[i])
                elif p == 'Y':
                    qc.rx(np.pi/2, targets[i])
            
            # CNOTs
            for i in range(len(targets) - 1):
                qc.cx(targets[i], targets[i+1])
            
            # RZ rotation
            qc.rz(theta, targets[-1])
            
            # Inverse CNOTs
            for i in range(len(targets) - 2, -1, -1):
                qc.cx(targets[i], targets[i+1])
            
            # Inverse basis change
            for i, p in enumerate(paulis):
                if p == 'X':
                    qc.h(targets[i])
                elif p == 'Y':
                    qc.rx(-np.pi/2, targets[i])
        
        # Build Trotter circuit
        for _ in range(num_steps):
            for pauli_str, coeff in pauli_list:
                apply_pauli_rotation(pauli_str, coeff)
        
        # Simulate
        backend = AerSimulator(method='statevector')
        qc.save_statevector()
        result = backend.run(qc).result()
        statevector = result.get_statevector()
        
        return np.array(statevector.data)
    
    except Exception as e:
        print(f"Qiskit simulation failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def simulate_cirq_circuit(num_steps=100, evolution_time=1.0, order=2):
    """Simulate Cirq Hamiltonian evolution circuit."""
    if not CIRQ_AVAILABLE:
        return None
    
    print("\n[Cirq] Building and simulating circuit...")
    
    try:
        qubits = [cirq.LineQubit(i) for i in range(4)]
        circuit = cirq.Circuit()
        
        # H2 Hamiltonian terms
        hamiltonian_terms = [
            ('IIII', -0.8105), ('IIIZ', 0.1721), ('IIZI', -0.2228),
            ('IZII', 0.1721), ('ZIII', -0.2228), ('IIZZ', 0.1686),
            ('IZIZ', 0.1205), ('IZZI', 0.1686), ('ZIIZ', 0.1686),
            ('ZIZI', 0.1205), ('ZZII', 0.1686), ('IIXX', 0.0454),
            ('IIYY', 0.0454), ('IXIX', 0.0454), ('IYIY', 0.0454),
        ]
        
        dt = evolution_time / num_steps
        
        # Helper to apply Pauli rotation exp(-i*theta*P)
        def apply_pauli_rotation(pauli_str, coeff):
            # Find non-identity qubits
            targets = []
            paulis = []
            for i, p in enumerate(pauli_str):
                if p != 'I':
                    targets.append(qubits[i])
                    paulis.append(p)
            
            if not targets:
                return  # All identity
            
            theta = 2.0 * coeff * dt
            
            # Basis change
            for i, p in enumerate(paulis):
                if p == 'X':
                    circuit.append(cirq.H(targets[i]))
                elif p == 'Y':
                    circuit.append(cirq.rx(np.pi/2)(targets[i]))
            
            # CNOTs for multi-qubit
            for i in range(len(targets) - 1):
                circuit.append(cirq.CNOT(targets[i], targets[i+1]))
            
            # RZ rotation
            circuit.append(cirq.rz(theta)(targets[-1]))
            
            # Inverse CNOTs
            for i in range(len(targets) - 2, -1, -1):
                circuit.append(cirq.CNOT(targets[i], targets[i+1]))
            
            # Inverse basis change
            for i, p in enumerate(paulis):
                if p == 'X':
                    circuit.append(cirq.H(targets[i]))
                elif p == 'Y':
                    circuit.append(cirq.rx(-np.pi/2)(targets[i]))
        
        # Build Trotter circuit
        for _ in range(num_steps):
            for pauli_str, coeff in hamiltonian_terms:
                apply_pauli_rotation(pauli_str, coeff)
        
        # Simulate
        simulator = cirq.Simulator()
        result = simulator.simulate(circuit)
        
        return result.final_state_vector
    
    except Exception as e:
        print(f"Cirq simulation failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def simulate_pennylane_circuit(num_steps=100, evolution_time=1.0, order=2):
    """Simulate PennyLane Hamiltonian evolution circuit."""
    if not PENNYLANE_AVAILABLE:
        return None
    
    print("\n[PennyLane] Building and simulating circuit...")
    
    try:
        # Create device
        dev = qml.device('default.qubit', wires=4)
        
        # Define H2 Hamiltonian
        coeffs = [-0.8105, 0.1721, -0.2228, 0.1721, -0.2228, 0.1686,
                  0.1205, 0.1686, 0.1686, 0.1205, 0.1686, 0.0454,
                  0.0454, 0.0454, 0.0454]
        
        obs = [qml.Identity(0) @ qml.Identity(1) @ qml.Identity(2) @ qml.Identity(3),
               qml.Identity(0) @ qml.Identity(1) @ qml.Identity(2) @ qml.PauliZ(3),
               qml.Identity(0) @ qml.Identity(1) @ qml.PauliZ(2) @ qml.Identity(3),
               qml.Identity(0) @ qml.PauliZ(1) @ qml.Identity(2) @ qml.Identity(3),
               qml.PauliZ(0) @ qml.Identity(1) @ qml.Identity(2) @ qml.Identity(3),
               qml.Identity(0) @ qml.Identity(1) @ qml.PauliZ(2) @ qml.PauliZ(3),
               qml.Identity(0) @ qml.PauliZ(1) @ qml.Identity(2) @ qml.PauliZ(3),
               qml.Identity(0) @ qml.PauliZ(1) @ qml.PauliZ(2) @ qml.Identity(3),
               qml.PauliZ(0) @ qml.Identity(1) @ qml.Identity(2) @ qml.PauliZ(3),
               qml.PauliZ(0) @ qml.Identity(1) @ qml.PauliZ(2) @ qml.Identity(3),
               qml.PauliZ(0) @ qml.PauliZ(1) @ qml.Identity(2) @ qml.Identity(3),
               qml.Identity(0) @ qml.Identity(1) @ qml.PauliX(2) @ qml.PauliX(3),
               qml.Identity(0) @ qml.Identity(1) @ qml.PauliY(2) @ qml.PauliY(3),
               qml.Identity(0) @ qml.PauliX(1) @ qml.Identity(2) @ qml.PauliX(3),
               qml.Identity(0) @ qml.PauliY(1) @ qml.Identity(2) @ qml.PauliY(3)]
        
        H = qml.Hamiltonian(coeffs, obs)
        
        @qml.qnode(dev)
        def circuit():
            # Apply Trotter evolution
            dt = evolution_time / num_steps
            for _ in range(num_steps):
                qml.ApproxTimeEvolution(H, dt, 1)
            return qml.state()
        
        statevector = circuit()
        return statevector
    
    except Exception as e:
        print(f"PennyLane simulation failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def compute_fidelity(state1, state2):
    """Compute state fidelity between two state vectors."""
    if state1 is None or state2 is None:
        return None
    
    # Normalize states
    state1 = state1 / np.linalg.norm(state1)
    state2 = state2 / np.linalg.norm(state2)
    
    # Fidelity: |⟨ψ1|ψ2⟩|^2
    overlap = np.abs(np.vdot(state1, state2))**2
    return overlap


def compute_energy_expectation(state, H):
    """Compute energy expectation value ⟨ψ|H|ψ⟩."""
    if state is None:
        return None
    
    state = state / np.linalg.norm(state)
    energy = np.real(np.vdot(state, H @ state))
    return energy


def simulate_myquat_circuit(num_steps=100, evolution_time=1.0, order=2):
    """Simulate MyQuat Hamiltonian evolution circuit."""
    print("\n[MyQuat] Building and simulating circuit...")
    
    try:
        import subprocess
        import json
        
        # Path to MyQuat simulator executable
        myquat_exe = Path(__file__).parent.parent / 'target/release/examples/h2_simulate'
        
        if not myquat_exe.exists():
            print(f"MyQuat executable not found at {myquat_exe}")
            return None
        
        # Run MyQuat simulation
        result = subprocess.run(
            [str(myquat_exe), str(num_steps), str(evolution_time), str(order)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            print(f"MyQuat simulation failed with code {result.returncode}")
            print(f"stderr: {result.stderr}")
            return None
        
        # Parse JSON output
        output_data = json.loads(result.stdout)
        state_vector_list = output_data['state_vector']
        
        # Convert to complex numpy array
        state_vector = np.array([complex(real, imag) for real, imag in state_vector_list])
        
        return state_vector
    
    except Exception as e:
        print(f"MyQuat simulation failed: {e}")
        import traceback
        traceback.print_exc()
        return None


def main():
    """Run accuracy verification."""
    print("=" * 80)
    print("Hamiltonian Simulation Accuracy Verification")
    print("=" * 80)
    print()
    
    # Parameters
    num_steps = 100
    evolution_time = 1.0
    order = 2
    
    print(f"Parameters:")
    print(f"  Trotter steps: {num_steps}")
    print(f"  Evolution time: {evolution_time}")
    print(f"  Trotter order: {order}")
    
    # Compute exact evolution
    print("\n[Exact] Computing exact time evolution...")
    exact_state = exact_evolution(evolution_time)
    H = get_h2_hamiltonian_matrix()
    exact_energy = compute_energy_expectation(exact_state, H)
    print(f"Exact ground state energy: {exact_energy:.6f}")
    
    # Simulate frameworks
    results = {}
    
    # MyQuat (run first to show Rust performance)
    myquat_state = simulate_myquat_circuit(num_steps, evolution_time, order)
    if myquat_state is not None:
        results['MyQuat'] = {
            'state': myquat_state,
            'fidelity': compute_fidelity(exact_state, myquat_state),
            'energy': compute_energy_expectation(myquat_state, H)
        }
    
    # Qiskit
    qiskit_state = simulate_qiskit_circuit(num_steps, evolution_time, order)
    if qiskit_state is not None:
        results['Qiskit'] = {
            'state': qiskit_state,
            'fidelity': compute_fidelity(exact_state, qiskit_state),
            'energy': compute_energy_expectation(qiskit_state, H)
        }
    
    # Cirq
    cirq_state = simulate_cirq_circuit(num_steps, evolution_time, order)
    if cirq_state is not None:
        results['Cirq'] = {
            'state': cirq_state,
            'fidelity': compute_fidelity(exact_state, cirq_state),
            'energy': compute_energy_expectation(cirq_state, H)
        }
    
    # PennyLane
    pennylane_state = simulate_pennylane_circuit(num_steps, evolution_time, order)
    if pennylane_state is not None:
        results['PennyLane'] = {
            'state': pennylane_state,
            'fidelity': compute_fidelity(exact_state, pennylane_state),
            'energy': compute_energy_expectation(pennylane_state, H)
        }
    
    # Print results
    print("\n" + "=" * 80)
    print("Accuracy Verification Results")
    print("=" * 80)
    print()
    print(f"{'Framework':<15} {'Fidelity':<15} {'Energy':<15} {'Energy Error':<15}")
    print("-" * 80)
    
    for framework, data in results.items():
        fid = data['fidelity']
        energy = data['energy']
        energy_error = abs(energy - exact_energy) if energy is not None else None
        
        fid_str = f"{fid:.10f}" if fid is not None else "N/A"
        energy_str = f"{energy:.6f}" if energy is not None else "N/A"
        error_str = f"{energy_error:.2e}" if energy_error is not None else "N/A"
        
        print(f"{framework:<15} {fid_str:<15} {energy_str:<15} {error_str:<15}")
    
    print()
    print("Note: Fidelity close to 1.0 indicates accurate simulation")
    print("      Energy error shows deviation from exact result")
    print()


if __name__ == "__main__":
    main()
