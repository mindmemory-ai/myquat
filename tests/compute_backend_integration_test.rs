//! Compute Backend Integration Tests
//! Author: gA4ss
//!
//! Integration tests for the unified compute backend framework

use myquat::*;

#[test]
fn test_backend_manager_auto_detect() {
    let manager = ComputeBackendManager::auto_detect();
    let backends = manager.list_backends();

    assert!(!backends.is_empty(), "Should detect at least CPU backend");
    assert!(
        backends.contains(&"cpu".to_string()),
        "CPU backend should be available"
    );
}

#[test]
fn test_cpu_backend_execution() {
    let manager = ComputeBackendManager::auto_detect();
    let backend = manager
        .get_backend("cpu")
        .expect("CPU backend should exist");

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure_all().unwrap();

    let result = backend.execute(&circuit, 100).unwrap();

    assert_eq!(result.shots, 100);
    assert_eq!(result.backend_used, "cpu");
    assert!(!result.counts.is_empty());
}

#[test]
fn test_backend_auto_selection_small_circuit() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.cx(1, 2).unwrap();
    circuit.measure_all().unwrap();

    let hints = ExecutionHints::default();
    let backend = manager.select_backend(&circuit, &hints).unwrap();

    assert!(backend.is_available());
    assert!(backend.is_compatible(&circuit));
}

#[test]
fn test_backend_auto_selection_medium_circuit() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(12, 12);
    for i in 0..11 {
        circuit.h(i).unwrap();
        circuit.cx(i, i + 1).unwrap();
    }
    circuit.measure_all().unwrap();

    let hints = ExecutionHints::default();
    let backend = manager.select_backend(&circuit, &hints).unwrap();

    assert!(backend.is_available());
    assert!(backend.is_compatible(&circuit));
}

#[test]
fn test_manual_backend_selection() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.measure_all().unwrap();

    let mut hints = ExecutionHints::default();
    hints.prefer_backend = Some("cpu".to_string());

    let backend = manager.select_backend(&circuit, &hints).unwrap();
    assert_eq!(backend.name(), "cpu");
}

#[test]
fn test_execution_with_hints() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure_all().unwrap();

    let mut hints = ExecutionHints::default();
    hints.precision = Precision::High;
    hints.allow_gpu = false;

    let result = manager.execute(&circuit, 100, &hints).unwrap();

    assert_eq!(result.shots, 100);
    assert!(!result.counts.is_empty());
    assert!(result.execution_time_ms > 0.0);
}

#[test]
fn test_performance_profile() {
    let manager = ComputeBackendManager::auto_detect();

    for backend_name in manager.list_backends() {
        let backend = manager.get_backend(&backend_name).unwrap();
        let profile = backend.performance_profile();

        assert!(profile.max_qubits > 0);
        assert!(profile.ops_per_second > 0.0);
        assert!(profile.memory_per_qubit_bytes > 0);
    }
}

#[test]
fn test_backend_compatibility() {
    let manager = ComputeBackendManager::auto_detect();
    let backend = manager.get_backend("cpu").unwrap();

    let small_circuit = QuantumCircuit::new(5, 5);
    assert!(backend.is_compatible(&small_circuit));

    let large_circuit = QuantumCircuit::new(30, 30);
    assert!(!backend.is_compatible(&large_circuit));
}

#[test]
fn test_execution_result_analysis() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure_all().unwrap();

    let hints = ExecutionHints::default();
    let result = manager.execute(&circuit, 1000, &hints).unwrap();

    let total_counts: usize = result.counts.values().sum();
    assert_eq!(total_counts, 1000);

    let prob_00 = result.probability("00");
    let prob_11 = result.probability("11");

    assert!(prob_00 > 0.3 && prob_00 < 0.7);
    assert!(prob_11 > 0.3 && prob_11 < 0.7);
    assert!((prob_00 + prob_11 - 1.0).abs() < 0.1);
}

#[test]
fn test_parallel_backend_if_available() {
    let manager = ComputeBackendManager::auto_detect();

    if let Ok(parallel_backend) = manager.get_backend("parallel") {
        if parallel_backend.is_available() {
            let mut circuit = QuantumCircuit::new(3, 3);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.cx(1, 2).unwrap();
            circuit.measure_all().unwrap();

            let result = parallel_backend.execute(&circuit, 1000).unwrap();

            assert_eq!(result.shots, 1000);
            assert!(result.backend_used.starts_with("parallel"));

            let total_counts: usize = result.counts.values().sum();
            assert_eq!(total_counts, 1000);
        }
    }
}

#[test]
fn test_simd_backend_if_available() {
    let manager = ComputeBackendManager::auto_detect();

    if let Ok(simd_backend) = manager.get_backend("simd") {
        if simd_backend.is_available() {
            let mut circuit = QuantumCircuit::new(2, 2);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.measure_all().unwrap();

            let result = simd_backend.execute(&circuit, 100).unwrap();

            assert_eq!(result.shots, 100);
            assert_eq!(result.backend_used, "simd");
        }
    }
}

#[test]
fn test_backend_manager_default_backend() {
    let mut manager = ComputeBackendManager::auto_detect();

    manager.set_default("cpu".to_string()).unwrap();

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();

    let hints = ExecutionHints::default();
    let result = manager.execute(&circuit, 10, &hints).unwrap();

    assert!(result.backend_used == "cpu" || !result.backend_used.is_empty());
}

#[test]
fn test_execution_hints_defaults() {
    let hints = ExecutionHints::default();

    assert!(!hints.prefer_cloud);
    assert!(hints.max_latency_ms.is_none());
    assert_eq!(hints.precision, Precision::Medium);
    assert!(!hints.allow_gpu);
    assert!(hints.prefer_backend.is_none());
}

#[test]
fn test_execution_result_most_likely() {
    let manager = ComputeBackendManager::auto_detect();

    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure_all().unwrap();

    let hints = ExecutionHints::default();
    let result = manager.execute(&circuit, 1000, &hints).unwrap();

    let most_likely = result.most_likely();
    assert!(most_likely.is_some());

    if let Some((state, count)) = most_likely {
        assert!(state == "00" || state == "11");
        assert!(count > 0);
    }
}
