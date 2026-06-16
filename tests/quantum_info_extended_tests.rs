//! Extended quantum info tests
//!
//! Author: gA4ss
//!
//! Additional tests for quantum information utilities including
//! state operations, measurements, and entanglement.

use myquat::quantum_info::{QuantumInfo, QuantumState, SparseQuantumState, StateCompression};
use myquat::{QuantumCircuit, Result};
use ndarray::Array1;
use num_complex::Complex64;

#[test]
fn test_quantum_state_creation() -> Result<()> {
    // Test various state creation methods
    let zero = QuantumState::zero_state(2);
    assert_eq!(zero.num_qubits(), 2);
    assert_eq!(zero.dim(), 4);

    let plus = QuantumState::plus_state(2);
    assert_eq!(plus.num_qubits(), 2);

    // All amplitudes should have equal magnitude
    let amp0 = plus.amplitude(0).unwrap().norm();
    let amp1 = plus.amplitude(1).unwrap().norm();
    assert!((amp0 - amp1).abs() < 1e-10);

    Ok(())
}

#[test]
fn test_quantum_state_normalization() -> Result<()> {
    // Create unnormalized state
    let mut amplitudes = Array1::zeros(4);
    amplitudes[0] = Complex64::new(1.0, 0.0);
    amplitudes[1] = Complex64::new(1.0, 0.0);
    amplitudes[2] = Complex64::new(1.0, 0.0);
    amplitudes[3] = Complex64::new(1.0, 0.0);

    let state = QuantumState::new_normalized(amplitudes)?;

    // Check normalization
    let norm_sq: f64 = (0..state.dim())
        .map(|i| state.amplitude(i).unwrap().norm_sqr())
        .sum();

    assert!((norm_sq - 1.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn test_quantum_state_measurement() -> Result<()> {
    // Create |0> state
    let state = QuantumState::zero_state(2);

    // Probability of measuring |00> should be 1
    assert!((state.amplitude(0).unwrap().norm_sqr() - 1.0).abs() < 1e-10);

    // Probability of other states should be 0
    for i in 1..state.dim() {
        assert!(state.amplitude(i).unwrap().norm_sqr() < 1e-10);
    }

    Ok(())
}

#[test]
fn test_tensor_product() -> Result<()> {
    let state1 = QuantumState::zero_state(1);
    let state2 = QuantumState::computational_basis_state(1, 1)?;

    let product = QuantumInfo::tensor_product(&state1, &state2);

    assert_eq!(product.num_qubits(), 2);
    assert_eq!(product.dim(), 4);

    // Should be |01>
    assert!(product.amplitude(1).unwrap().norm_sqr() > 0.99);

    Ok(())
}

#[test]
fn test_sparse_state_operations() -> Result<()> {
    let mut sparse = SparseQuantumState::new(5);

    // Set some amplitudes
    sparse.set_amplitude(0, Complex64::new(0.7, 0.0));
    sparse.set_amplitude(5, Complex64::new(0.5, 0.0));
    sparse.set_amplitude(10, Complex64::new(0.3, 0.0));

    // Normalize
    sparse.normalize();

    // Check normalization
    let norm_sq: f64 = sparse
        .nonzero_indices()
        .iter()
        .map(|&i| sparse.amplitude(i).norm_sqr())
        .sum();

    assert!((norm_sq - 1.0).abs() < 1e-10);
    assert_eq!(sparse.num_nonzero(), 3);

    Ok(())
}

#[test]
fn test_sparse_dense_conversion() -> Result<()> {
    // Create dense state
    let dense = QuantumState::computational_basis_state(3, 5)?;

    // Convert to sparse
    let sparse = SparseQuantumState::from_dense(&dense, 1e-10);

    // Convert back to dense
    let dense2 = sparse.to_dense();

    // Should be identical
    for i in 0..dense.dim() {
        let diff = (dense.amplitude(i).unwrap() - dense2.amplitude(i).unwrap()).norm();
        assert!(diff < 1e-10);
    }

    Ok(())
}

#[test]
fn test_compression_statistics() -> Result<()> {
    let state = QuantumState::zero_state(10);
    let (sparse, stats) = StateCompression::lossless_compress(&state);

    // Verify statistics
    assert_eq!(stats.original_size, 1024);
    assert_eq!(stats.compressed_size, 1);
    assert!(stats.compression_ratio < 0.01);
    assert_eq!(stats.fidelity, 1.0);
    assert_eq!(stats.threshold, 0.0);

    Ok(())
}

#[test]
fn test_compression_with_threshold() -> Result<()> {
    // Create state with small amplitudes
    let mut amplitudes = Array1::zeros(16);
    amplitudes[0] = Complex64::new(0.99, 0.0);
    amplitudes[1] = Complex64::new(0.01, 0.0);
    amplitudes[2] = Complex64::new(0.01, 0.0);
    let state = QuantumState::new_normalized(amplitudes)?;

    // Compress with threshold
    let (sparse, stats) = StateCompression::truncate_amplitudes(&state, 0.001)?;

    // Small amplitudes should be removed
    assert!(sparse.num_nonzero() < state.dim());
    assert!(stats.fidelity > 0.9);

    Ok(())
}

#[test]
fn test_sparse_state_sparsity() -> Result<()> {
    // Test sparsity calculation
    let sparse1 = SparseQuantumState::zero_state(10);
    assert!(sparse1.sparsity() < 0.01);

    let sparse2 = SparseQuantumState::computational_basis_state(5, 10)?;
    assert_eq!(sparse2.sparsity(), 1.0 / 32.0);

    Ok(())
}

#[test]
fn test_state_with_circuit_evolution() -> Result<()> {
    // Create circuit
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(0)?;
    circuit.cx(0, 1)?;

    // This creates a Bell state
    // We can verify the circuit was built correctly
    assert_eq!(circuit.size(), 2);
    assert_eq!(circuit.num_qubits(), 2);

    Ok(())
}

#[test]
fn test_large_sparse_state() -> Result<()> {
    // Test with large qubit count
    let sparse = SparseQuantumState::zero_state(30);

    assert_eq!(sparse.num_qubits(), 30);
    assert_eq!(sparse.dim(), 1 << 30); // 1 billion+
    assert_eq!(sparse.num_nonzero(), 1);

    // Memory should still be minimal
    assert!(sparse.memory_usage() < 1000);

    Ok(())
}

#[test]
fn test_compression_fidelity_bounds() -> Result<()> {
    let state = QuantumState::plus_state(3);

    // Lossless compression should have fidelity 1.0
    let (_, stats1) = StateCompression::lossless_compress(&state);
    assert_eq!(stats1.fidelity, 1.0);

    // Lossy compression should have fidelity < 1.0 (for plus state)
    let result = StateCompression::truncate_amplitudes(&state, 0.1);
    if let Ok((_, stats2)) = result {
        assert!(stats2.fidelity <= 1.0);
        assert!(stats2.fidelity >= 0.0);
    }

    Ok(())
}

#[test]
fn test_adaptive_compression_target_fidelity() -> Result<()> {
    // Create state with varying amplitudes
    let mut amplitudes = Array1::zeros(32);
    for i in 0..8 {
        amplitudes[i] = Complex64::new(0.3, 0.0);
    }
    let state = QuantumState::new_normalized(amplitudes)?;

    // Test different target fidelities
    for target in [0.99, 0.95, 0.90] {
        let result = StateCompression::adaptive_compress(&state, target, 20)?;
        let (_, stats) = result;

        // Should meet or exceed target fidelity
        assert!(stats.fidelity >= target - 0.01); // Small tolerance
    }

    Ok(())
}
