//! Large-scale quantum state tests
//!
//! Author: gA4ss
//!
//! Tests for sparse states and compression on large qubit systems.

use myquat::quantum_info::{QuantumState, SparseQuantumState, StateCompression};

#[test]
fn test_sparse_state_20_qubits() {
    // 20 qubits = 2^20 = 1,048,576 dimensional Hilbert space
    let sparse = SparseQuantumState::zero_state(20);

    assert_eq!(sparse.num_qubits(), 20);
    assert_eq!(sparse.dim(), 1 << 20);
    assert_eq!(sparse.num_nonzero(), 1);

    // Sparsity should be very low
    assert!(sparse.sparsity() < 1e-5);

    // Memory usage should be minimal
    let memory = sparse.memory_usage();
    assert!(memory < 1000); // Less than 1KB
}

#[test]
fn test_sparse_state_25_qubits() {
    // 25 qubits = 2^25 = 33,554,432 dimensional Hilbert space
    // This would require ~1GB for dense representation
    let sparse = SparseQuantumState::computational_basis_state(25, 12345).unwrap();

    assert_eq!(sparse.num_qubits(), 25);
    assert_eq!(sparse.num_nonzero(), 1);
    assert_eq!(sparse.amplitude(12345).re, 1.0);

    // Memory should still be minimal
    let memory = sparse.memory_usage();
    assert!(memory < 1000);
}

#[test]
fn test_compression_on_large_state() {
    // Create a 10-qubit state (1024 dimensions)
    let state = QuantumState::zero_state(10);

    // Lossless compression
    let (sparse, stats) = StateCompression::lossless_compress(&state);

    assert_eq!(stats.fidelity, 1.0);
    assert_eq!(sparse.num_nonzero(), 1);
    assert!(stats.compression_ratio < 0.01);

    println!("10-qubit compression:");
    stats.print();
}

#[test]
fn test_sparse_state_operations() {
    // Test multiple operations on sparse states
    let mut sparse = SparseQuantumState::zero_state(15);

    // Set some amplitudes
    use num_complex::Complex64;
    sparse.set_amplitude(0, Complex64::new(0.7, 0.0));
    sparse.set_amplitude(100, Complex64::new(0.5, 0.0));
    sparse.set_amplitude(1000, Complex64::new(0.3, 0.0));
    sparse.set_amplitude(10000, Complex64::new(0.2, 0.0));

    // Normalize
    sparse.normalize();

    // Check normalization
    let norm_sq: f64 = sparse
        .nonzero_indices()
        .iter()
        .map(|&i| sparse.amplitude(i).norm_sqr())
        .sum();

    assert!((norm_sq - 1.0).abs() < 1e-10);
    assert_eq!(sparse.num_nonzero(), 4);
}

#[test]
fn test_memory_scaling() {
    // Test memory usage scales with sparsity, not dimension
    let sparse_10 = SparseQuantumState::zero_state(10);
    let sparse_20 = SparseQuantumState::zero_state(20);
    let sparse_30 = SparseQuantumState::zero_state(30);

    let mem_10 = sparse_10.memory_usage();
    let mem_20 = sparse_20.memory_usage();
    let mem_30 = sparse_30.memory_usage();

    // Memory should be roughly constant for same sparsity
    assert!((mem_10 as f64 / mem_20 as f64 - 1.0).abs() < 0.5);
    assert!((mem_20 as f64 / mem_30 as f64 - 1.0).abs() < 0.5);

    println!("Memory scaling:");
    println!("  10 qubits: {} bytes", mem_10);
    println!("  20 qubits: {} bytes", mem_20);
    println!("  30 qubits: {} bytes", mem_30);
}

#[test]
fn test_sparse_to_dense_conversion() {
    // Test conversion for moderately sized state
    let sparse = SparseQuantumState::computational_basis_state(8, 42).unwrap();
    let dense = sparse.to_dense();

    assert_eq!(dense.num_qubits(), 8);
    assert_eq!(dense.amplitude(42).unwrap().re, 1.0);

    // All other amplitudes should be zero
    for i in 0..dense.dim() {
        if i != 42 {
            assert!(dense.amplitude(i).unwrap().norm() < 1e-10);
        }
    }
}

#[test]
fn test_compression_benchmark() {
    // Benchmark compression on various state sizes
    for num_qubits in [8, 10, 12] {
        let state = QuantumState::zero_state(num_qubits);
        let start = std::time::Instant::now();
        let (sparse, stats) = StateCompression::lossless_compress(&state);
        let duration = start.elapsed();

        println!("\n{} qubits:", num_qubits);
        println!("  Dimension: {}", state.dim());
        println!("  Non-zero: {}", sparse.num_nonzero());
        println!("  Compression time: {:?}", duration);
        println!(
            "  Memory saved: {:.2}%",
            (1.0 - stats.compression_ratio) * 100.0
        );

        assert!(stats.compression_time.as_millis() < 100);
    }
}
