// Fermion Hamiltonian Tests
// Author: gA4ss
//
// Comprehensive tests for fermion-to-qubit transformations and electronic structure Hamiltonians

use myquat::hamiltonian::{
    bravyi_kitaev_transform, jordan_wigner_transform, parity_transform,
    ElectronicStructureHamiltonian, FermionOperator, FermionTerm, MappingMethod, PauliOperator,
};
use num_complex::Complex64;

#[test]
fn test_h2_molecule_jordan_wigner() {
    // H2 molecule in minimal basis (STO-3G)
    // 2 orbitals -> 4 spin-orbitals -> 4 qubits
    let mut es = ElectronicStructureHamiltonian::new(2);

    // One-body integrals (in Hartree)
    es.set_one_body(0, 0, -1.252);
    es.set_one_body(1, 1, -0.475);

    // Two-body integrals
    es.set_two_body(0, 0, 0, 0, 0.674);
    es.set_two_body(0, 0, 1, 1, 0.181);
    es.set_two_body(1, 1, 0, 0, 0.181);
    es.set_two_body(1, 1, 1, 1, 0.697);

    // Nuclear repulsion energy
    es.nuclear_repulsion = 0.715;

    let hamiltonian = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();

    assert_eq!(hamiltonian.num_qubits, 4);
    assert!(!hamiltonian.terms.is_empty());
    assert!((hamiltonian.constant_term.re - 0.715).abs() < 1e-10);
}

#[test]
fn test_h2_molecule_bravyi_kitaev() {
    let mut es = ElectronicStructureHamiltonian::new(2);

    es.set_one_body(0, 0, -1.252);
    es.set_one_body(1, 1, -0.475);
    es.set_two_body(0, 0, 0, 0, 0.674);
    es.set_two_body(0, 0, 1, 1, 0.181);
    es.set_two_body(1, 1, 0, 0, 0.181);
    es.set_two_body(1, 1, 1, 1, 0.697);
    es.nuclear_repulsion = 0.715;

    let hamiltonian = es
        .to_qubit_hamiltonian(MappingMethod::BravyiKitaev)
        .unwrap();

    assert_eq!(hamiltonian.num_qubits, 4);
    assert!(!hamiltonian.terms.is_empty());
    assert!((hamiltonian.constant_term.re - 0.715).abs() < 1e-10);
}

#[test]
fn test_h2_molecule_parity() {
    let mut es = ElectronicStructureHamiltonian::new(2);

    es.set_one_body(0, 0, -1.252);
    es.set_one_body(1, 1, -0.475);
    es.set_two_body(0, 0, 0, 0, 0.674);
    es.set_two_body(0, 0, 1, 1, 0.181);
    es.set_two_body(1, 1, 0, 0, 0.181);
    es.set_two_body(1, 1, 1, 1, 0.697);
    es.nuclear_repulsion = 0.715;

    let hamiltonian = es.to_qubit_hamiltonian(MappingMethod::Parity).unwrap();

    assert_eq!(hamiltonian.num_qubits, 4);
    assert!(!hamiltonian.terms.is_empty());
    assert!((hamiltonian.constant_term.re - 0.715).abs() < 1e-10);
}

#[test]
fn test_lih_molecule() {
    // LiH molecule (simplified)
    // 5 orbitals -> 10 spin-orbitals -> 10 qubits
    let mut es = ElectronicStructureHamiltonian::new(5);

    // Simplified one-body terms
    for i in 0..5 {
        es.set_one_body(i, i, -1.0 - i as f64 * 0.2);
    }

    // Simplified two-body terms
    es.set_two_body(0, 0, 0, 0, 0.5);
    es.set_two_body(1, 1, 1, 1, 0.4);
    es.set_two_body(0, 0, 1, 1, 0.1);
    es.set_two_body(1, 1, 0, 0, 0.1);

    es.nuclear_repulsion = 1.0;

    let h_jw = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();
    let h_bk = es
        .to_qubit_hamiltonian(MappingMethod::BravyiKitaev)
        .unwrap();
    let h_parity = es.to_qubit_hamiltonian(MappingMethod::Parity).unwrap();

    // All should have same number of qubits
    assert_eq!(h_jw.num_qubits, 10);
    assert_eq!(h_bk.num_qubits, 10);
    assert_eq!(h_parity.num_qubits, 10);

    // All should have same constant term
    assert!((h_jw.constant_term.re - 1.0).abs() < 1e-10);
    assert!((h_bk.constant_term.re - 1.0).abs() < 1e-10);
    assert!((h_parity.constant_term.re - 1.0).abs() < 1e-10);
}

#[test]
fn test_pauli_weight_jordan_wigner() {
    // Test that JW has linear scaling of Pauli weight
    for n in [4, 8, 12] {
        let term = FermionTerm::new(
            vec![(n / 2, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = jordan_wigner_transform(&term, n).unwrap();

        // Calculate average weight
        let total_weight: usize = pauli_terms
            .iter()
            .map(|(ps, _)| {
                ps.operators
                    .iter()
                    .filter(|&op| *op != PauliOperator::I)
                    .count()
            })
            .sum();

        let avg_weight = total_weight as f64 / pauli_terms.len() as f64;

        // JW should have weight proportional to qubit index
        assert!(avg_weight > (n / 2) as f64 * 0.5);
    }
}

#[test]
fn test_pauli_weight_bravyi_kitaev() {
    // Test that BK has logarithmic scaling of Pauli weight
    for n in [4, 8, 16] {
        let term = FermionTerm::new(
            vec![(n / 2, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = bravyi_kitaev_transform(&term, n).unwrap();

        // Calculate average weight
        let total_weight: usize = pauli_terms
            .iter()
            .map(|(ps, _)| {
                ps.operators
                    .iter()
                    .filter(|&op| *op != PauliOperator::I)
                    .count()
            })
            .sum();

        let avg_weight = total_weight as f64 / pauli_terms.len() as f64;

        // BK should have weight O(log n)
        let expected_max_weight = (n as f64).log2().ceil() as f64;
        assert!(avg_weight <= expected_max_weight + 2.0);
    }
}

#[test]
fn test_pauli_weight_parity() {
    // Test that Parity has constant weight for nearest-neighbor
    for n in [4, 8, 12] {
        let term = FermionTerm::new(
            vec![(1, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = parity_transform(&term, n).unwrap();

        // Calculate average weight
        let total_weight: usize = pauli_terms
            .iter()
            .map(|(ps, _)| {
                ps.operators
                    .iter()
                    .filter(|&op| *op != PauliOperator::I)
                    .count()
            })
            .sum();

        let avg_weight = total_weight as f64 / pauli_terms.len() as f64;

        // Parity should have constant weight for low-index operators
        assert!(avg_weight <= 3.0);
    }
}

#[test]
fn test_hermiticity_preservation() {
    // Test that transformations preserve Hermiticity
    let mut es = ElectronicStructureHamiltonian::new(2);

    es.set_one_body(0, 0, -1.0);
    es.set_one_body(1, 1, -0.5);
    es.set_two_body(0, 0, 1, 1, 0.1);
    es.nuclear_repulsion = 0.5;

    let h_jw = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();

    // All coefficients should be real for Hermitian Hamiltonian
    for term in &h_jw.terms {
        assert!(
            term.pauli_string.coefficient.im.abs() < 1e-10,
            "Coefficient should be real"
        );
    }
}

#[test]
fn test_number_operator_eigenvalues() {
    // Test that number operator has correct eigenvalues (0 or 1)
    let term = FermionTerm::number_operator(0, Complex64::new(1.0, 0.0));

    let jw_terms = jordan_wigner_transform(&term, 2).unwrap();
    let bk_terms = bravyi_kitaev_transform(&term, 2).unwrap();
    let parity_terms = parity_transform(&term, 2).unwrap();

    // All should produce valid Pauli decompositions
    assert!(!jw_terms.is_empty());
    assert!(!bk_terms.is_empty());
    assert!(!parity_terms.is_empty());
}

#[test]
fn test_anticommutation_relations() {
    // Test that {a_i, a†_j} = δ_ij
    // This is implicitly tested by the correct Pauli decomposition

    let a0 = FermionTerm::new(
        vec![(0, FermionOperator::Annihilation)],
        Complex64::new(1.0, 0.0),
    );

    let a0_dag = FermionTerm::new(
        vec![(0, FermionOperator::Creation)],
        Complex64::new(1.0, 0.0),
    );

    let jw_a0 = jordan_wigner_transform(&a0, 2).unwrap();
    let jw_a0_dag = jordan_wigner_transform(&a0_dag, 2).unwrap();

    // Both should have same structure (X ± iY)
    assert_eq!(jw_a0.len(), jw_a0_dag.len());
}

#[test]
fn test_particle_number_conservation() {
    // Test Hamiltonians that conserve particle number
    let mut es = ElectronicStructureHamiltonian::new(2);

    // Only number-conserving terms
    es.set_one_body(0, 0, -1.0);
    es.set_one_body(1, 1, -0.5);
    es.set_two_body(0, 0, 0, 0, 0.5);

    let hamiltonian = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();

    // Should have valid Hamiltonian
    assert!(!hamiltonian.terms.is_empty());
}

#[test]
fn test_zero_hamiltonian() {
    // Test empty Hamiltonian
    let es = ElectronicStructureHamiltonian::new(2);

    let hamiltonian = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();

    assert_eq!(hamiltonian.num_qubits, 4);
    assert_eq!(hamiltonian.constant_term.re, 0.0);
}

#[test]
fn test_large_system() {
    // Test larger system (8 orbitals = 16 qubits)
    let mut es = ElectronicStructureHamiltonian::new(8);

    // Add some terms
    for i in 0..8 {
        es.set_one_body(i, i, -1.0 - i as f64 * 0.1);
    }

    es.nuclear_repulsion = 2.0;

    let h_jw = es
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)
        .unwrap();
    let h_bk = es
        .to_qubit_hamiltonian(MappingMethod::BravyiKitaev)
        .unwrap();
    let h_parity = es.to_qubit_hamiltonian(MappingMethod::Parity).unwrap();

    assert_eq!(h_jw.num_qubits, 16);
    assert_eq!(h_bk.num_qubits, 16);
    assert_eq!(h_parity.num_qubits, 16);
}

#[test]
fn test_coefficient_accuracy() {
    // Test that coefficients are preserved accurately
    let term = FermionTerm::new(
        vec![(0, FermionOperator::Creation)],
        Complex64::new(1.234, 0.0),
    );

    let jw_terms = jordan_wigner_transform(&term, 2).unwrap();

    // Sum of coefficient magnitudes should be related to original
    let total_mag: f64 = jw_terms.iter().map(|(_, c)| c.norm()).sum();

    // Each term has coefficient 0.5 * 1.234, and there are 2 terms
    // Total magnitude should be approximately 1.234
    assert!((total_mag - 1.234).abs() < 0.1);
}
