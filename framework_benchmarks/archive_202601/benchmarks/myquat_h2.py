#!/usr/bin/env python3
"""
MyQuat H₂ Molecule Hamiltonian Simulation Benchmark
Author: gA4ss

Benchmark for Trotter-Suzuki decomposition of H₂ molecule using MyQuat (Rust).
This script calls the Rust benchmark executable.
"""

import time
import subprocess
import json
import sys
import os


def benchmark_myquat_h2(num_trotter_steps=100, evolution_time=1.0, order=2):
    """
    Benchmark MyQuat Hamiltonian simulation for H₂.
    
    Args:
        num_trotter_steps: Number of Trotter steps
        evolution_time: Total evolution time
        order: Trotter order (1 or 2)
    
    Returns:
        dict: Benchmark results
    """
    # Path to MyQuat root directory
    myquat_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '../..'))
    
    # Build the Rust benchmark if needed
    print("Building MyQuat benchmark (Rust)...")
    build_result = subprocess.run(
        ['cargo', 'build', '--release', '--example', 'h2_benchmark'],
        cwd=myquat_root,
        capture_output=True,
        text=True
    )
    
    if build_result.returncode != 0:
        print(f"Error building MyQuat benchmark: {build_result.stderr}")
        sys.exit(1)
    
    # Run the benchmark
    print("Running MyQuat benchmark...")
    benchmark_exe = os.path.join(myquat_root, 'target/release/examples/h2_benchmark')
    
    start_time = time.perf_counter()
    
    result = subprocess.run(
        [benchmark_exe, str(num_trotter_steps), str(evolution_time), str(order)],
        capture_output=True,
        text=True
    )
    
    end_time = time.perf_counter()
    
    if result.returncode != 0:
        print(f"Error running MyQuat benchmark: {result.stderr}")
        sys.exit(1)
    
    # Parse output (expecting JSON in stdout)
    try:
        # Extract JSON by finding the first complete JSON object
        output = result.stdout.strip()
        json_start = output.find('{')
        json_end = output.find('}', json_start) + 1
        
        if json_start != -1 and json_end > json_start:
            json_str = output[json_start:json_end]
            benchmark_results = json.loads(json_str)
        else:
            raise ValueError("No valid JSON found in output")
    
    except (json.JSONDecodeError, ValueError) as e:
        print(f"Error parsing benchmark output: {e}")
        print(f"Output: {result.stdout}")
        sys.exit(1)
    
    results = {
        'framework': 'MyQuat',
        'language': 'Rust',
        'compilation_time_ms': benchmark_results.get('compilation_time_ms', 0.0),
        'gate_count': benchmark_results.get('gate_count', 0),
        'circuit_depth': benchmark_results.get('circuit_depth', 0),
        'num_qubits': 4,
        'num_terms': 15,
        'trotter_steps': num_trotter_steps,
        'trotter_order': order,
    }
    
    return results


def main():
    """Run benchmark and print results."""
    print("=" * 60)
    print("MyQuat H₂ Hamiltonian Simulation Benchmark")
    print("=" * 60)
    print()
    
    # Run benchmark
    results = benchmark_myquat_h2(num_trotter_steps=100, evolution_time=1.0, order=2)
    
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
