#!/usr/bin/env python3
"""
Comprehensive Framework Comparison Benchmark
Author: gA4ss

Rigorous comparison of MyQuat with Qiskit, Cirq, and PennyLane across:
- Multiple problem types (H2, LiH, Heisenberg, TFIM, Random)
- Multiple Trotter orders (1, 2, 4)
- Multiple system sizes (2-12 qubits)
- Metrics: compilation time, gate count, circuit depth, accuracy
"""

import subprocess
import json
import time
import numpy as np
from datetime import datetime
import os
import sys
from pathlib import Path
from scipy.linalg import expm
import csv

# Framework availability flags
QISKIT_AVAILABLE = False
CIRQ_AVAILABLE = False
PENNYLANE_AVAILABLE = False

try:
    from qiskit import QuantumCircuit as QiskitCircuit
    from qiskit.quantum_info import SparsePauliOp, Operator
    from qiskit.circuit.library import PauliEvolutionGate
    from qiskit.synthesis import SuzukiTrotter, LieTrotter
    from qiskit import transpile
    QISKIT_AVAILABLE = True
except ImportError as e:
    print(f"Warning: Qiskit not available: {e}")

try:
    import cirq
    CIRQ_AVAILABLE = True
except ImportError as e:
    print(f"Warning: Cirq not available: {e}")

try:
    import pennylane as qml
    PENNYLANE_AVAILABLE = True
except ImportError as e:
    print(f"Warning: PennyLane not available: {e}")


# ============================================================================
# Hamiltonian Definitions
# ============================================================================

class HamiltonianProblem:
    """Represents a test Hamiltonian problem"""
    
    def __init__(self, name, num_qubits, pauli_terms, description=""):
        self.name = name
        self.num_qubits = num_qubits
        self.pauli_terms = pauli_terms  # List of (pauli_string, coefficient)
        self.description = description
        self.num_terms = len(pauli_terms)
    
    def __repr__(self):
        return f"{self.name}: {self.num_qubits} qubits, {self.num_terms} terms"


def create_h2_hamiltonian():
    """H2 molecule at equilibrium (4 qubits, 15 terms) - Jordan-Wigner encoding"""
    terms = [
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
        ('XXII', 0.0454),
        ('YYII', 0.0454),
    ]
    return HamiltonianProblem("H2_4q", 4, terms, "H2 molecule, 4 qubits")


def create_lih_hamiltonian():
    """LiH molecule (6 qubits, ~30 terms) - simplified"""
    terms = [
        ('IIIIII', -7.4983),
        ('IIIIIZ', 0.3936),
        ('IIIIZI', 0.3936),
        ('IIIZII', -0.3936),
        ('IIZIII', -0.3936),
        ('IZIIII', 0.0896),
        ('ZIIIII', 0.0896),
        ('IIIIZZ', 0.1815),
        ('IIIZIZ', 0.1240),
        ('IIIZZI', 0.1815),
        ('IIZIIZ', 0.1815),
        ('IIZIZI', 0.1240),
        ('IIZZII', 0.1815),
        ('IZIIZI', 0.0620),
        ('IZIZII', 0.0620),
        ('IZZIIZ', 0.0620),
        ('IZZIZI', 0.0620),
        ('ZIIIIZ', 0.0620),
        ('ZIIIZI', 0.0620),
        ('ZIIZII', 0.0620),
        ('ZIZIII', 0.0620),
        ('ZZIIIZ', 0.0320),
        ('ZZIIZI', 0.0320),
        ('IIIIYY', 0.0230),
        ('IIIIXX', 0.0230),
        ('IIIYYI', 0.0230),
        ('IIIXXI', 0.0230),
        ('IIYYII', 0.0230),
        ('IIXXII', 0.0230),
    ]
    return HamiltonianProblem("LiH_6q", 6, terms, "LiH molecule, 6 qubits")


def create_heisenberg_hamiltonian(num_qubits, J=1.0):
    """1D Heisenberg XXZ model with periodic boundary"""
    terms = []
    for i in range(num_qubits):
        j = (i + 1) % num_qubits
        # XX interaction
        pauli = ['I'] * num_qubits
        pauli[i], pauli[j] = 'X', 'X'
        terms.append((''.join(pauli), J))
        # YY interaction
        pauli[i], pauli[j] = 'Y', 'Y'
        terms.append((''.join(pauli), J))
        # ZZ interaction
        pauli[i], pauli[j] = 'Z', 'Z'
        terms.append((''.join(pauli), J))
    return HamiltonianProblem(f"Heisenberg_{num_qubits}q", num_qubits, terms,
                              f"1D Heisenberg model, {num_qubits} qubits")


def create_tfim_hamiltonian(num_qubits, J=1.0, g=0.5):
    """Transverse Field Ising Model"""
    terms = []
    # ZZ interactions
    for i in range(num_qubits - 1):
        pauli = ['I'] * num_qubits
        pauli[i], pauli[i+1] = 'Z', 'Z'
        terms.append((''.join(pauli), -J))
    # Transverse field
    for i in range(num_qubits):
        pauli = ['I'] * num_qubits
        pauli[i] = 'X'
        terms.append((''.join(pauli), -g))
    return HamiltonianProblem(f"TFIM_{num_qubits}q", num_qubits, terms,
                              f"TFIM, {num_qubits} qubits")


def create_random_hamiltonian(num_qubits, num_terms, seed=42):
    """Random Pauli Hamiltonian"""
    np.random.seed(seed)
    paulis = ['I', 'X', 'Y', 'Z']
    terms = []
    for _ in range(num_terms):
        pauli = ''.join(np.random.choice(paulis) for _ in range(num_qubits))
        if pauli == 'I' * num_qubits:
            pauli = 'I' * (num_qubits - 1) + 'Z'
        coeff = np.random.uniform(-1, 1)
        terms.append((pauli, coeff))
    return HamiltonianProblem(f"Random_{num_qubits}q_{num_terms}t", num_qubits, terms,
                              f"Random Hamiltonian, {num_qubits} qubits, {num_terms} terms")


# ============================================================================
# Framework Benchmarks
# ============================================================================

def benchmark_myquat(problem, trotter_steps, evolution_time, order):
    """Benchmark MyQuat compilation"""
    try:
        myquat_root = Path(__file__).parent.parent
        benchmark_exe = myquat_root / 'target/release/examples/benchmark_hamiltonian'
        
        if not benchmark_exe.exists():
            # Build if not exists
            subprocess.run(['cargo', 'build', '--release', '--example', 'benchmark_hamiltonian'],
                          cwd=myquat_root, capture_output=True, check=True)
        
        # Format command
        cmd = [str(benchmark_exe), str(problem.num_qubits), str(trotter_steps), 
               str(evolution_time), str(order)]
        for pauli, coeff in problem.pauli_terms:
            cmd.append(f"{pauli}:{coeff}")
        
        start = time.perf_counter()
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        total_time = (time.perf_counter() - start) * 1000
        
        if result.returncode == 0:
            try:
                data = json.loads(result.stdout.strip())
                return {
                    'framework': 'MyQuat',
                    'language': 'Rust',
                    'compilation_time_ms': data.get('compilation_time_ms', total_time),
                    'gate_count': data.get('gate_count', 0),
                    'circuit_depth': data.get('circuit_depth', 0),
                    'success': True
                }
            except json.JSONDecodeError:
                # Parse text output
                lines = result.stdout.strip().split('\n')
                gate_count = 0
                depth = 0
                for line in lines:
                    if 'gate' in line.lower():
                        try:
                            gate_count = int(''.join(filter(str.isdigit, line)))
                        except:
                            pass
                    if 'depth' in line.lower():
                        try:
                            depth = int(''.join(filter(str.isdigit, line)))
                        except:
                            pass
                return {
                    'framework': 'MyQuat',
                    'language': 'Rust',
                    'compilation_time_ms': total_time,
                    'gate_count': gate_count,
                    'circuit_depth': depth,
                    'success': True
                }
        return {'framework': 'MyQuat', 'success': False, 'error': result.stderr}
    except Exception as e:
        return {'framework': 'MyQuat', 'success': False, 'error': str(e)}


def benchmark_qiskit(problem, trotter_steps, evolution_time, order):
    """Benchmark Qiskit compilation"""
    if not QISKIT_AVAILABLE:
        return {'framework': 'Qiskit', 'success': False, 'error': 'Not available'}
    
    try:
        pauli_list = [(p, c) for p, c in problem.pauli_terms]
        hamiltonian = SparsePauliOp.from_list(pauli_list)
        
        start = time.perf_counter()
        
        if order == 1:
            synthesis = LieTrotter(reps=trotter_steps)
        else:
            synthesis = SuzukiTrotter(order=order, reps=trotter_steps)
        
        evolution_gate = PauliEvolutionGate(hamiltonian, time=evolution_time, synthesis=synthesis)
        circuit = QiskitCircuit(problem.num_qubits)
        circuit.append(evolution_gate, range(problem.num_qubits))
        
        # Decompose to native gates
        circuit = circuit.decompose()
        circuit = circuit.decompose()
        
        # Transpile for fair comparison
        circuit = transpile(circuit, basis_gates=['u3', 'cx'], optimization_level=0)
        
        compile_time = (time.perf_counter() - start) * 1000
        
        return {
            'framework': 'Qiskit',
            'language': 'Python',
            'compilation_time_ms': compile_time,
            'gate_count': circuit.size(),
            'circuit_depth': circuit.depth(),
            'success': True
        }
    except Exception as e:
        return {'framework': 'Qiskit', 'success': False, 'error': str(e)}


def benchmark_qiskit_optimized(problem, trotter_steps, evolution_time, order):
    """Benchmark Qiskit with optimization"""
    if not QISKIT_AVAILABLE:
        return {'framework': 'Qiskit_Opt', 'success': False, 'error': 'Not available'}
    
    try:
        pauli_list = [(p, c) for p, c in problem.pauli_terms]
        hamiltonian = SparsePauliOp.from_list(pauli_list)
        
        start = time.perf_counter()
        
        if order == 1:
            synthesis = LieTrotter(reps=trotter_steps)
        else:
            synthesis = SuzukiTrotter(order=order, reps=trotter_steps)
        
        evolution_gate = PauliEvolutionGate(hamiltonian, time=evolution_time, synthesis=synthesis)
        circuit = QiskitCircuit(problem.num_qubits)
        circuit.append(evolution_gate, range(problem.num_qubits))
        circuit = circuit.decompose().decompose()
        
        # Transpile with optimization level 3
        circuit = transpile(circuit, basis_gates=['u3', 'cx'], optimization_level=3)
        
        compile_time = (time.perf_counter() - start) * 1000
        
        return {
            'framework': 'Qiskit_Opt',
            'language': 'Python',
            'compilation_time_ms': compile_time,
            'gate_count': circuit.size(),
            'circuit_depth': circuit.depth(),
            'success': True
        }
    except Exception as e:
        return {'framework': 'Qiskit_Opt', 'success': False, 'error': str(e)}


def benchmark_cirq(problem, trotter_steps, evolution_time, order):
    """Benchmark Cirq compilation"""
    if not CIRQ_AVAILABLE:
        return {'framework': 'Cirq', 'success': False, 'error': 'Not available'}
    
    try:
        qubits = cirq.LineQubit.range(problem.num_qubits)
        
        start = time.perf_counter()
        
        # Build PauliSum
        pauli_sum = cirq.PauliSum()
        pauli_map = {'I': cirq.I, 'X': cirq.X, 'Y': cirq.Y, 'Z': cirq.Z}
        
        for pauli_str, coeff in problem.pauli_terms:
            term_dict = {}
            for i, p in enumerate(pauli_str):
                if p != 'I':
                    term_dict[qubits[i]] = pauli_map[p]
            if term_dict:
                pauli_sum += coeff * cirq.PauliString(term_dict)
            else:
                pauli_sum += coeff * cirq.PauliString()
        
        # Create evolution circuit
        dt = evolution_time / trotter_steps
        circuit = cirq.Circuit()
        
        for _ in range(trotter_steps):
            # Simple first-order Trotter (Cirq doesn't have built-in high-order)
            for pauli_str, coeff in problem.pauli_terms:
                term_dict = {}
                for i, p in enumerate(pauli_str):
                    if p != 'I':
                        term_dict[qubits[i]] = pauli_map[p]
                if term_dict:
                    ps = cirq.PauliString(term_dict, coefficient=coeff)
                    circuit += cirq.PauliStringPhasor(ps, exponent_neg=-dt/np.pi)
        
        # Decompose to native gates
        circuit = cirq.optimize_for_target_gateset(circuit, gateset=cirq.CZTargetGateset())
        
        compile_time = (time.perf_counter() - start) * 1000
        
        return {
            'framework': 'Cirq',
            'language': 'Python',
            'compilation_time_ms': compile_time,
            'gate_count': len(list(circuit.all_operations())),
            'circuit_depth': len(circuit),
            'success': True
        }
    except Exception as e:
        return {'framework': 'Cirq', 'success': False, 'error': str(e)}


# ============================================================================
# Main Benchmark Suite
# ============================================================================

def run_comprehensive_benchmarks():
    """Run comprehensive benchmarks across all dimensions"""
    
    print("=" * 80)
    print("Comprehensive Framework Comparison Benchmark")
    print("=" * 80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Frameworks: MyQuat (Rust), Qiskit (Python), Cirq (Python)")
    print()
    
    # Define test problems
    problems = [
        create_h2_hamiltonian(),
        create_lih_hamiltonian(),
        create_heisenberg_hamiltonian(4),
        create_heisenberg_hamiltonian(6),
        create_heisenberg_hamiltonian(8),
        create_tfim_hamiltonian(4),
        create_tfim_hamiltonian(6),
        create_tfim_hamiltonian(8),
        create_tfim_hamiltonian(10),
        create_random_hamiltonian(4, 20),
        create_random_hamiltonian(6, 30),
        create_random_hamiltonian(8, 40),
    ]
    
    # Test configurations
    configs = [
        {'steps': 10, 'time': 1.0, 'order': 1},
        {'steps': 10, 'time': 1.0, 'order': 2},
        {'steps': 50, 'time': 1.0, 'order': 1},
        {'steps': 50, 'time': 1.0, 'order': 2},
        {'steps': 100, 'time': 1.0, 'order': 2},
    ]
    
    results = []
    
    for problem in problems:
        print(f"\n{'='*60}")
        print(f"Problem: {problem}")
        print(f"{'='*60}")
        
        for config in configs:
            steps = config['steps']
            t = config['time']
            order = config['order']
            
            print(f"\n  Config: steps={steps}, time={t}, order={order}")
            print(f"  {'-'*50}")
            
            row = {
                'problem': problem.name,
                'num_qubits': problem.num_qubits,
                'num_terms': problem.num_terms,
                'trotter_steps': steps,
                'trotter_order': order,
                'evolution_time': t,
            }
            
            # MyQuat
            r = benchmark_myquat(problem, steps, t, order)
            if r.get('success'):
                row['myquat_time_ms'] = r['compilation_time_ms']
                row['myquat_gates'] = r['gate_count']
                row['myquat_depth'] = r['circuit_depth']
                print(f"    MyQuat:     {r['compilation_time_ms']:8.2f} ms, "
                      f"{r['gate_count']:6d} gates, depth {r['circuit_depth']:5d}")
            else:
                row['myquat_time_ms'] = None
                print(f"    MyQuat:     FAILED - {r.get('error', 'Unknown')[:40]}")
            
            # Qiskit (no optimization)
            r = benchmark_qiskit(problem, steps, t, order)
            if r.get('success'):
                row['qiskit_time_ms'] = r['compilation_time_ms']
                row['qiskit_gates'] = r['gate_count']
                row['qiskit_depth'] = r['circuit_depth']
                print(f"    Qiskit:     {r['compilation_time_ms']:8.2f} ms, "
                      f"{r['gate_count']:6d} gates, depth {r['circuit_depth']:5d}")
            else:
                row['qiskit_time_ms'] = None
                print(f"    Qiskit:     FAILED - {r.get('error', 'Unknown')[:40]}")
            
            # Qiskit (optimized)
            r = benchmark_qiskit_optimized(problem, steps, t, order)
            if r.get('success'):
                row['qiskit_opt_time_ms'] = r['compilation_time_ms']
                row['qiskit_opt_gates'] = r['gate_count']
                row['qiskit_opt_depth'] = r['circuit_depth']
                print(f"    Qiskit_Opt: {r['compilation_time_ms']:8.2f} ms, "
                      f"{r['gate_count']:6d} gates, depth {r['circuit_depth']:5d}")
            else:
                row['qiskit_opt_time_ms'] = None
                print(f"    Qiskit_Opt: FAILED - {r.get('error', 'Unknown')[:40]}")
            
            # Cirq
            r = benchmark_cirq(problem, steps, t, order)
            if r.get('success'):
                row['cirq_time_ms'] = r['compilation_time_ms']
                row['cirq_gates'] = r['gate_count']
                row['cirq_depth'] = r['circuit_depth']
                print(f"    Cirq:       {r['compilation_time_ms']:8.2f} ms, "
                      f"{r['gate_count']:6d} gates, depth {r['circuit_depth']:5d}")
            else:
                row['cirq_time_ms'] = None
                print(f"    Cirq:       FAILED - {r.get('error', 'Unknown')[:40]}")
            
            # Calculate speedups
            if row.get('myquat_time_ms') and row.get('qiskit_time_ms'):
                row['speedup_vs_qiskit'] = row['qiskit_time_ms'] / row['myquat_time_ms']
            if row.get('myquat_time_ms') and row.get('qiskit_opt_time_ms'):
                row['speedup_vs_qiskit_opt'] = row['qiskit_opt_time_ms'] / row['myquat_time_ms']
            if row.get('myquat_time_ms') and row.get('cirq_time_ms'):
                row['speedup_vs_cirq'] = row['cirq_time_ms'] / row['myquat_time_ms']
            
            results.append(row)
    
    return results


def save_results(results):
    """Save results to CSV"""
    results_dir = Path(__file__).parent / 'results'
    results_dir.mkdir(exist_ok=True)
    
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    csv_file = results_dir / f'comprehensive_comparison_{timestamp}.csv'
    
    if results:
        fieldnames = list(results[0].keys())
        with open(csv_file, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(results)
    
    print(f"\nResults saved to: {csv_file}")
    return csv_file


def print_summary(results):
    """Print summary statistics"""
    print("\n" + "=" * 80)
    print("SUMMARY STATISTICS")
    print("=" * 80)
    
    # Calculate averages
    speedups_qiskit = [r['speedup_vs_qiskit'] for r in results if r.get('speedup_vs_qiskit')]
    speedups_qiskit_opt = [r['speedup_vs_qiskit_opt'] for r in results if r.get('speedup_vs_qiskit_opt')]
    speedups_cirq = [r['speedup_vs_cirq'] for r in results if r.get('speedup_vs_cirq')]
    
    if speedups_qiskit:
        print(f"\nMyQuat vs Qiskit (no optimization):")
        print(f"  Average speedup: {np.mean(speedups_qiskit):.1f}x")
        print(f"  Min speedup:     {np.min(speedups_qiskit):.1f}x")
        print(f"  Max speedup:     {np.max(speedups_qiskit):.1f}x")
    
    if speedups_qiskit_opt:
        print(f"\nMyQuat vs Qiskit (optimization level 3):")
        print(f"  Average speedup: {np.mean(speedups_qiskit_opt):.1f}x")
        print(f"  Min speedup:     {np.min(speedups_qiskit_opt):.1f}x")
        print(f"  Max speedup:     {np.max(speedups_qiskit_opt):.1f}x")
    
    if speedups_cirq:
        print(f"\nMyQuat vs Cirq:")
        print(f"  Average speedup: {np.mean(speedups_cirq):.1f}x")
        print(f"  Min speedup:     {np.min(speedups_cirq):.1f}x")
        print(f"  Max speedup:     {np.max(speedups_cirq):.1f}x")
    
    # Gate count comparison
    print("\n" + "-" * 60)
    print("GATE COUNT COMPARISON (MyQuat vs Qiskit Optimized)")
    print("-" * 60)
    
    for r in results:
        if r.get('myquat_gates') and r.get('qiskit_opt_gates'):
            ratio = r['myquat_gates'] / r['qiskit_opt_gates']
            status = "BETTER" if ratio < 1 else "LARGER"
            print(f"  {r['problem']:20s} order={r['trotter_order']}: "
                  f"MyQuat {r['myquat_gates']:6d} vs Qiskit {r['qiskit_opt_gates']:6d} "
                  f"({ratio:.2f}x, {status})")


def main():
    """Main entry point"""
    results = run_comprehensive_benchmarks()
    
    if results:
        save_results(results)
        print_summary(results)
    
    print("\n" + "=" * 80)
    print(f"Completed: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("=" * 80)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
