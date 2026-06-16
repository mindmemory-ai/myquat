#!/usr/bin/env python3
"""
Quantum Framework Benchmarks Runner
Author: gA4ss

Runs comprehensive benchmarks comparing MyQuat with Qiskit, Cirq, and PennyLane.
"""

import sys
import os
import time
from datetime import datetime
from pathlib import Path

# Add benchmarks directory to path
sys.path.insert(0, str(Path(__file__).parent / 'benchmarks'))

from qiskit_h2 import benchmark_qiskit_h2
from cirq_h2 import benchmark_cirq_h2
from myquat_h2 import benchmark_myquat_h2

# Try to import PennyLane, skip if incompatible
try:
    from pennylane_h2 import benchmark_pennylane_h2
    PENNYLANE_AVAILABLE = True
except Exception as e:
    print(f"Warning: PennyLane not available: {e}")
    PENNYLANE_AVAILABLE = False


def run_all_benchmarks():
    """Run all framework benchmarks and collect results."""
    print("=" * 80)
    print("Quantum Framework Benchmarks")
    print("=" * 80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()
    
    results = []
    
    # Benchmark parameters
    num_steps = 100
    evolution_time = 1.0
    order = 2
    
    # MyQuat
    print("[1/4] Running MyQuat benchmark...")
    try:
        myquat_results = benchmark_myquat_h2(num_steps, evolution_time, order)
        results.append(myquat_results)
        print(f"✓ MyQuat: {myquat_results['compilation_time_ms']:.2f} ms, "
              f"{myquat_results['gate_count']} gates")
    except Exception as e:
        print(f"✗ MyQuat benchmark failed: {e}")
    print()
    
    # Qiskit
    print("[2/4] Running Qiskit benchmark...")
    try:
        qiskit_results = benchmark_qiskit_h2(num_steps, evolution_time, order)
        results.append(qiskit_results)
        print(f"✓ Qiskit: {qiskit_results['compilation_time_ms']:.2f} ms, "
              f"{qiskit_results['gate_count']} gates")
    except Exception as e:
        print(f"✗ Qiskit benchmark failed: {e}")
    print()
    
    # Cirq
    print("[3/4] Running Cirq benchmark...")
    try:
        cirq_results = benchmark_cirq_h2(num_steps, evolution_time, order)
        results.append(cirq_results)
        print(f"✓ Cirq: {cirq_results['compilation_time_ms']:.2f} ms, "
              f"{cirq_results['gate_count']} gates")
    except Exception as e:
        print(f"✗ Cirq benchmark failed: {e}")
    print()
    
    # PennyLane
    if PENNYLANE_AVAILABLE:
        print("[4/4] Running PennyLane benchmark...")
        try:
            pennylane_results = benchmark_pennylane_h2(num_steps, evolution_time, order)
            results.append(pennylane_results)
            print(f"✓ PennyLane: {pennylane_results['compilation_time_ms']:.2f} ms, "
                  f"{pennylane_results['gate_count']} gates")
        except Exception as e:
            print(f"✗ PennyLane benchmark failed: {e}")
        print()
    else:
        print("[4/4] PennyLane benchmark skipped (not available)")
        print()
    
    return results


def print_comparison_table(results):
    """Print formatted comparison table."""
    print("=" * 80)
    print("Benchmark Results Comparison")
    print("=" * 80)
    print()
    
    # Header
    print(f"{'Framework':<12} {'Language':<8} {'Time (ms)':<12} {'Gates':<8} {'Depth':<8} {'Speedup':<8}")
    print("-" * 80)
    
    # Find baseline (MyQuat) time
    baseline_time = None
    for r in results:
        if r['framework'] == 'MyQuat':
            baseline_time = r['compilation_time_ms']
            break
    
    # Print results
    for r in results:
        speedup = baseline_time / r['compilation_time_ms'] if baseline_time and r['compilation_time_ms'] > 0 else 0
        speedup_str = f"{speedup:.2f}x" if speedup > 0 else "N/A"
        
        print(f"{r['framework']:<12} {r['language']:<8} "
              f"{r['compilation_time_ms']:<12.2f} "
              f"{r['gate_count']:<8} "
              f"{r['circuit_depth']:<8} "
              f"{speedup_str:<8}")
    
    print()


def save_results(results):
    """Save results to CSV file."""
    results_dir = Path(__file__).parent / 'results'
    results_dir.mkdir(exist_ok=True)
    
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    csv_file = results_dir / f'benchmark_results_{timestamp}.csv'
    
    # Write CSV
    with open(csv_file, 'w') as f:
        # Header
        f.write('Framework,Language,CompilationTime_ms,GateCount,CircuitDepth,')
        f.write('NumQubits,NumTerms,TrotterSteps,TrotterOrder\n')
        
        # Data
        for r in results:
            f.write(f"{r['framework']},{r['language']},")
            f.write(f"{r['compilation_time_ms']:.4f},{r['gate_count']},{r['circuit_depth']},")
            f.write(f"{r['num_qubits']},{r['num_terms']},{r['trotter_steps']},{r['trotter_order']}\n")
    
    print(f"Results saved to: {csv_file}")
    print()
    
    return csv_file


def main():
    """Main entry point."""
    # Run benchmarks
    results = run_all_benchmarks()
    
    if not results:
        print("Error: No benchmarks completed successfully.")
        return 1
    
    # Print comparison
    print_comparison_table(results)
    
    # Save results
    csv_file = save_results(results)
    
    print("=" * 80)
    print(f"Benchmarks completed: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("=" * 80)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
