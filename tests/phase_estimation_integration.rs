// Integration tests for Quantum Phase Estimation
// Author: gA4ss
//
// These tests verify the complete QPE workflow including circuit construction,
// eigenstate preparation, controlled unitaries, and inverse QFT.

use myquat::algorithms::phase_estimation::{ControlledUnitary, EigenstatePreparation};
use myquat::algorithms::PhaseEstimation;
use std::f64::consts::PI;

#[test]
fn test_qpe_basic_workflow() {
    // Test basic QPE workflow with default configuration
    let pe = PhaseEstimation::new(3, 1);
    let circuit = pe.build_circuit(None).unwrap();

    assert_eq!(circuit.num_qubits(), 4);
    assert_eq!(circuit.num_clbits(), 3);
    assert!(circuit.size() > 0);
}

#[test]
fn test_qpe_with_one_state() {
    // Test QPE with |1⟩ eigenstate preparation
    let pe = PhaseEstimation::with_eigenstate_prep(3, 1, EigenstatePreparation::OneState);

    let circuit = pe.build_circuit(None).unwrap();
    assert_eq!(circuit.num_qubits(), 4);

    // Should have X gate for |1⟩ preparation
    assert!(circuit.size() > 3);
}

#[test]
fn test_qpe_with_plus_state() {
    // Test QPE with |+⟩ eigenstate preparation
    let pe = PhaseEstimation::with_eigenstate_prep(2, 2, EigenstatePreparation::PlusState);

    let circuit = pe.build_circuit(None).unwrap();
    assert_eq!(circuit.num_qubits(), 4);

    // Should have H gates for |+⟩ preparation
    assert!(circuit.size() > 2);
}

#[test]
fn test_qpe_with_computational_basis() {
    // Test QPE with computational basis state |101⟩
    let pe = PhaseEstimation::with_eigenstate_prep(
        3,
        3,
        EigenstatePreparation::ComputationalBasis(0b101),
    );

    let circuit = pe.build_circuit(None).unwrap();
    assert_eq!(circuit.num_qubits(), 6);

    // Should have X gates for |101⟩ preparation (2 X gates)
    assert!(circuit.size() > 3);
}

#[test]
fn test_qpe_phase_unitary() {
    // Test QPE with phase unitary
    let phase = PI / 4.0;
    let pe = PhaseEstimation::with_unitary(
        3,
        1,
        EigenstatePreparation::OneState,
        ControlledUnitary::Phase(phase),
    );

    let circuit = pe.build_circuit(None).unwrap();
    assert_eq!(circuit.num_qubits(), 4);

    // Should have controlled phase gates
    assert!(circuit.size() > 5);
}

#[test]
fn test_qpe_z_rotation_unitary() {
    // Test QPE with Z rotation unitary
    let angle = PI / 3.0;
    let pe = PhaseEstimation::with_unitary(
        4,
        1,
        EigenstatePreparation::OneState,
        ControlledUnitary::ZRotation(angle),
    );

    let circuit = pe.build_circuit(None).unwrap();
    assert_eq!(circuit.num_qubits(), 5);
    assert!(circuit.size() > 0);
}

#[test]
fn test_qpe_precision_scaling() {
    // Test that precision improves with more counting qubits
    let precisions: Vec<f64> = (2..8)
        .map(|n| {
            let pe = PhaseEstimation::new(n, 1);
            pe.phase_precision()
        })
        .collect();

    // Precision should decrease (improve) with more qubits
    for i in 0..precisions.len() - 1 {
        assert!(precisions[i] > precisions[i + 1]);
    }

    // Precision should be 1/2^n
    for (i, &precision) in precisions.iter().enumerate() {
        let n = i + 2;
        let expected = 1.0 / (1 << n) as f64;
        assert!((precision - expected).abs() < 1e-10);
    }
}

#[test]
fn test_qpe_phase_estimation_accuracy() {
    // Test phase estimation from bitstring
    let pe = PhaseEstimation::new(3, 1);

    // Test various bitstrings
    let test_cases = vec![
        ("000", 0.0),
        ("001", 0.25),
        ("010", 0.5),
        ("011", 0.75),
        ("100", 1.0),
        ("101", 1.25),
        ("110", 1.5),
        ("111", 1.75),
    ];

    for (bitstring, expected_phase) in test_cases {
        let estimated = pe.estimate_phase_from_bitstring(bitstring);
        assert!(
            (estimated - expected_phase).abs() < 1e-10,
            "Bitstring {} should give phase {}, got {}",
            bitstring,
            expected_phase,
            estimated
        );
    }
}

#[test]
fn test_qpe_circuit_depth() {
    // Test circuit depth calculation
    let pe = PhaseEstimation::new(4, 2);
    let depth = pe.circuit_depth();

    // Depth should be: superposition + controlled unitaries + inverse QFT
    // = 1 + num_counting + num_counting = 1 + 4 + 4 = 9
    assert_eq!(depth, 9);
}

#[test]
fn test_qpe_multiple_eigenstate_qubits() {
    // Test QPE with multiple eigenstate qubits
    for num_eigenstate in 1..5 {
        let pe = PhaseEstimation::new(3, num_eigenstate);
        let circuit = pe.build_circuit(None).unwrap();

        assert_eq!(circuit.num_qubits(), 3 + num_eigenstate);
        assert_eq!(circuit.num_clbits(), 3);
    }
}

#[test]
fn test_qpe_large_scale() {
    // Test QPE with larger number of qubits
    let pe = PhaseEstimation::new(8, 4);
    let circuit = pe.build_circuit(None).unwrap();

    assert_eq!(circuit.num_qubits(), 12);
    assert_eq!(circuit.num_clbits(), 8);
    assert!(circuit.size() > 20);
}

#[test]
fn test_qpe_inverse_qft_structure() {
    // Test that inverse QFT is properly applied
    let pe = PhaseEstimation::new(3, 1);
    let circuit = pe.build_circuit(None).unwrap();

    // Circuit should have:
    // - Eigenstate preparation (0-1 gates)
    // - Superposition (3 H gates)
    // - Controlled unitaries (3 CP gates)
    // - Inverse QFT (H gates + CP gates + SWAP gates)
    // - Measurements (3)

    // Minimum gates: 3 (superposition) + 3 (controlled) + 3 (inverse QFT H) + 1 (SWAP) + 3 (measure)
    assert!(circuit.size() >= 10);
}

#[test]
fn test_qpe_all_unitary_types() {
    // Test all supported unitary types
    let unitaries = vec![
        ControlledUnitary::Phase(PI / 4.0),
        ControlledUnitary::ZRotation(PI / 3.0),
        ControlledUnitary::XRotation(PI / 6.0),
        ControlledUnitary::YRotation(PI / 8.0),
        ControlledUnitary::Custom,
    ];

    for unitary in unitaries {
        let pe = PhaseEstimation::with_unitary(3, 1, EigenstatePreparation::OneState, unitary);

        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.size() > 0);
    }
}

#[test]
fn test_qpe_all_eigenstate_types() {
    // Test all supported eigenstate preparation types
    let preparations = vec![
        EigenstatePreparation::None,
        EigenstatePreparation::OneState,
        EigenstatePreparation::PlusState,
        EigenstatePreparation::ComputationalBasis(0b10),
    ];

    for prep in preparations {
        let pe = PhaseEstimation::with_eigenstate_prep(3, 2, prep);
        let circuit = pe.build_circuit(None).unwrap();

        assert_eq!(circuit.num_qubits(), 5);
        assert!(circuit.size() > 0);
    }
}

#[test]
fn test_qpe_phase_range() {
    // Test QPE with various phase values
    let phases = vec![0.0, PI / 8.0, PI / 4.0, PI / 2.0, PI, 2.0 * PI];

    for phase in phases {
        let pe = PhaseEstimation::with_unitary(
            4,
            1,
            EigenstatePreparation::OneState,
            ControlledUnitary::Phase(phase),
        );

        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 5);
    }
}

#[test]
fn test_qpe_resource_estimation() {
    // Test resource requirements for different configurations
    let configs = vec![
        (2, 1), // Small
        (4, 2), // Medium
        (8, 4), // Large
    ];

    for (num_counting, num_eigenstate) in configs {
        let pe = PhaseEstimation::new(num_counting, num_eigenstate);
        let circuit = pe.build_circuit(None).unwrap();

        // Verify resource scaling
        assert_eq!(circuit.num_qubits(), num_counting + num_eigenstate);
        assert_eq!(circuit.num_clbits(), num_counting);

        // Gate count should scale with number of counting qubits
        assert!(circuit.size() > num_counting);
    }
}

#[test]
fn test_qpe_edge_cases() {
    // Test edge cases

    // Minimum configuration: 1 counting qubit, 1 eigenstate qubit
    let pe_min = PhaseEstimation::new(1, 1);
    let circuit_min = pe_min.build_circuit(None).unwrap();
    assert_eq!(circuit_min.num_qubits(), 2);

    // No eigenstate qubits (edge case)
    let pe_no_eigen = PhaseEstimation::new(3, 0);
    let circuit_no_eigen = pe_no_eigen.build_circuit(None).unwrap();
    assert_eq!(circuit_no_eigen.num_qubits(), 3);
}

#[test]
fn test_qpe_consistency() {
    // Test that multiple builds produce consistent circuits
    let pe = PhaseEstimation::with_unitary(
        3,
        1,
        EigenstatePreparation::OneState,
        ControlledUnitary::Phase(PI / 4.0),
    );

    let circuit1 = pe.build_circuit(None).unwrap();
    let circuit2 = pe.build_circuit(None).unwrap();

    assert_eq!(circuit1.num_qubits(), circuit2.num_qubits());
    assert_eq!(circuit1.num_clbits(), circuit2.num_clbits());
    assert_eq!(circuit1.size(), circuit2.size());
}
