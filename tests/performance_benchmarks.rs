//! Performance benchmark tests
//!
//! Author: gA4ss
//!
//! Performance regression tests to ensure optimization improvements
//! don't degrade performance.

use myquat::circuit_optimization::{CircuitPass, MergeRotationsPass, TemplateMatchingPass};
use myquat::quantum_info::{QuantumState, SparseQuantumState, StateCompression};
use myquat::{Parameter, QuantumCircuit, Result};
use std::time::Instant;

/// Performance threshold in milliseconds
const PERF_THRESHOLD_MS: u128 = 100;

#[test]
fn bench_circuit_creation() -> Result<()> {
    let start = Instant::now();

    // Create 100 circuits
    for _ in 0..100 {
        let mut circuit = QuantumCircuit::new(10, 0);
        circuit.h(0)?;
        circuit.cx(0, 1)?;
        circuit.rz(2, Parameter::Float(std::f64::consts::PI / 4.0))?;
    }

    let duration = start.elapsed().as_millis();
    println!("Circuit creation (100x): {}ms", duration);

    // Should be fast
    assert!(duration < PERF_THRESHOLD_MS);

    Ok(())
}

#[test]
fn bench_template_matching() -> Result<()> {
    // Create a circuit with patterns
    let mut circuit = QuantumCircuit::new(5, 0);
    for _ in 0..10 {
        circuit.h(0)?;
        circuit.cx(0, 1)?;
        circuit.h(0)?;
    }

    let start = Instant::now();

    let pass = TemplateMatchingPass::new();
    pass.run(&mut circuit)?;

    let duration = start.elapsed().as_millis();
    println!("Template matching: {}ms", duration);

    // Should be fast
    assert!(duration < PERF_THRESHOLD_MS);

    Ok(())
}

#[test]
fn bench_rotation_merging() -> Result<()> {
    // Create circuit with many rotations
    let mut circuit = QuantumCircuit::new(5, 0);
    for i in 0..50 {
        circuit.rz(i % 5, Parameter::Float(std::f64::consts::PI / 8.0))?;
    }

    let start = Instant::now();

    let pass = MergeRotationsPass::new();
    pass.run(&mut circuit)?;

    let duration = start.elapsed().as_millis();
    println!("Rotation merging: {}ms", duration);

    // Should be fast
    assert!(duration < PERF_THRESHOLD_MS);

    Ok(())
}

#[test]
fn bench_sparse_state_creation() -> Result<()> {
    let start = Instant::now();

    // Create 1000 sparse states
    for _ in 0..1000 {
        let _sparse = SparseQuantumState::zero_state(20);
    }

    let duration = start.elapsed().as_millis();
    println!("Sparse state creation (1000x): {}ms", duration);

    // Should be very fast
    assert!(duration < PERF_THRESHOLD_MS);

    Ok(())
}

#[test]
fn bench_state_compression() -> Result<()> {
    let state = QuantumState::zero_state(12);

    let start = Instant::now();

    // Compress 100 times
    for _ in 0..100 {
        let _ = StateCompression::lossless_compress(&state);
    }

    let duration = start.elapsed().as_millis();
    println!("State compression (100x): {}ms", duration);

    // Should be reasonably fast
    assert!(duration < PERF_THRESHOLD_MS * 2);

    Ok(())
}

#[test]
fn bench_sparse_dense_conversion() -> Result<()> {
    let dense = QuantumState::zero_state(10);
    let sparse = SparseQuantumState::from_dense(&dense, 1e-10);

    let start = Instant::now();

    // Convert 100 times
    for _ in 0..100 {
        let _ = sparse.to_dense();
    }

    let duration = start.elapsed().as_millis();
    println!("Sparse to dense conversion (100x): {}ms", duration);

    // Should be fast
    assert!(duration < PERF_THRESHOLD_MS);

    Ok(())
}

#[test]
fn bench_circuit_size_scaling() -> Result<()> {
    // Test performance scales reasonably with circuit size
    let sizes = [10, 20, 30, 40, 50];
    let mut times = Vec::new();

    for &size in &sizes {
        let mut circuit = QuantumCircuit::new(5, 0);
        for _ in 0..size {
            circuit.h(0)?;
            circuit.cx(0, 1)?;
        }

        let start = Instant::now();
        let pass = TemplateMatchingPass::new();
        pass.run(&mut circuit)?;
        let duration = start.elapsed().as_micros();

        times.push(duration);
        println!("Size {}: {}μs", size, duration);
    }

    // Performance should scale reasonably (not exponentially)
    // Check that doubling size doesn't increase time by more than 4x
    if times.len() >= 2 {
        let ratio = times[times.len() - 1] as f64 / times[0] as f64;
        let size_ratio = sizes[sizes.len() - 1] as f64 / sizes[0] as f64;
        println!(
            "Time ratio: {:.2}x for size ratio: {:.2}x",
            ratio, size_ratio
        );

        // Should be roughly linear or better
        assert!(ratio < size_ratio * 2.0);
    }

    Ok(())
}

#[test]
fn bench_memory_efficiency() -> Result<()> {
    // Compare memory usage of sparse vs dense states
    let sparse_20 = SparseQuantumState::zero_state(20);
    let sparse_25 = SparseQuantumState::zero_state(25);
    let sparse_30 = SparseQuantumState::zero_state(30);

    let mem_20 = sparse_20.memory_usage();
    let mem_25 = sparse_25.memory_usage();
    let mem_30 = sparse_30.memory_usage();

    println!("Memory usage:");
    println!("  20 qubits: {} bytes", mem_20);
    println!("  25 qubits: {} bytes", mem_25);
    println!("  30 qubits: {} bytes", mem_30);

    // Memory should be roughly constant for same sparsity
    assert!(mem_20 < 2000);
    assert!(mem_25 < 2000);
    assert!(mem_30 < 2000);

    // Dense state would need 2^30 * 16 bytes = 16GB for 30 qubits
    let dense_30_size = (1_u64 << 30) * 16;
    println!(
        "  Dense 30 qubits would need: {} bytes ({} GB)",
        dense_30_size,
        dense_30_size / (1024 * 1024 * 1024)
    );

    Ok(())
}

#[test]
fn bench_adaptive_compression_performance() -> Result<()> {
    use ndarray::Array1;
    use num_complex::Complex64;

    // Create state with varying amplitudes
    let mut amplitudes = Array1::zeros(64);
    for i in 0..16 {
        amplitudes[i] = Complex64::new(0.2, 0.0);
    }
    let state = QuantumState::new_normalized(amplitudes)?;

    let start = Instant::now();

    // Run adaptive compression
    let _ = StateCompression::adaptive_compress(&state, 0.95, 20)?;

    let duration = start.elapsed().as_millis();
    println!("Adaptive compression: {}ms", duration);

    // Should complete in reasonable time
    assert!(duration < PERF_THRESHOLD_MS * 5);

    Ok(())
}

#[test]
fn bench_large_circuit_optimization() -> Result<()> {
    // Create a large realistic circuit
    let mut circuit = QuantumCircuit::new(10, 0);

    // Add 100 gates
    for i in 0..100 {
        let q = i % 10;
        match i % 4 {
            0 => circuit.h(q)?,
            1 => circuit.rz(q, Parameter::Float(std::f64::consts::PI / 4.0))?,
            2 => {
                if q < 9 {
                    circuit.cx(q, q + 1)?;
                }
            }
            _ => circuit.rz(q, Parameter::Float(std::f64::consts::PI / 8.0))?,
        }
    }

    let start = Instant::now();

    // Apply multiple optimization passes
    let template_pass = TemplateMatchingPass::new();
    template_pass.run(&mut circuit)?;

    let merge_pass = MergeRotationsPass::new();
    merge_pass.run(&mut circuit)?;

    let duration = start.elapsed().as_millis();
    println!("Large circuit optimization: {}ms", duration);

    // Should complete in reasonable time
    assert!(duration < PERF_THRESHOLD_MS * 3);

    Ok(())
}

#[test]
fn bench_state_operations() -> Result<()> {
    let state = QuantumState::zero_state(8);

    let start = Instant::now();

    // Perform 1000 amplitude queries
    for _ in 0..1000 {
        for i in 0..state.dim() {
            let _ = state.amplitude(i);
        }
    }

    let duration = start.elapsed().as_millis();
    println!("State amplitude queries (256k): {}ms", duration);

    // Should be fast
    assert!(duration < PERF_THRESHOLD_MS * 2);

    Ok(())
}
