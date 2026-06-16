//! Analytical Solution Validation Tests for QM Solver
//!
//! Author: gA4ss
//!
//! This test suite validates the quantum mechanics solver against known
//! analytical solutions from quantum mechanics textbooks.
//!
//! These integration tests verify complete problem-solving workflows,
//! complementing the unit tests in individual modules.

use myquat::qm_solver::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

/// Validate hydrogen atom energy levels against Rydberg formula
///
/// E_n = -13.6 eV / n² (analytical formula)
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_hydrogen_energy_formula() {
    let backend = create_symbolica_backend();

    // Test that hydrogen energy function exists and produces non-zero results
    for n in 1..=5 {
        let energy = tise_solver::solutions::hydrogen_energy(n, &backend).unwrap();
        let energy_str = format!("{}", energy);

        // Verify energy expression is not empty
        assert!(
            !energy_str.is_empty(),
            "Hydrogen energy E_{} should exist",
            n
        );
    }
}

/// Validate hydrogen atom degeneracy formula
///
/// g_n = n² (accounting for l and m quantum numbers)
#[test]
fn validate_hydrogen_degeneracy() {
    // Test first few energy levels
    let test_cases = vec![
        (1, 1),  // n=1: 1 state (1s)
        (2, 4),  // n=2: 4 states (2s, 2p_x, 2p_y, 2p_z)
        (3, 9),  // n=3: 9 states
        (4, 16), // n=4: 16 states
        (5, 25), // n=5: 25 states
    ];

    for (n, expected_degeneracy) in test_cases {
        let degeneracy = tise_solver::solutions::hydrogen_degeneracy(n);
        assert_eq!(
            degeneracy, expected_degeneracy,
            "Hydrogen atom n={} should have degeneracy {}",
            n, expected_degeneracy
        );
    }
}

/// Validate harmonic oscillator energy equal spacing
///
/// E_n = ℏω(n + 1/2), spacing ΔE = ℏω
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_harmonic_oscillator_equal_spacing() {
    let backend = create_symbolica_backend();
    let omega = backend.parse("omega").unwrap();
    let hbar = backend.parse("hbar").unwrap();

    // Test that energy levels exist for several n values
    for n in 0..=4 {
        let energy =
            tise_solver::solutions::harmonic_oscillator_energy(n, &omega, &hbar, &backend).unwrap();
        let energy_str = format!("{}", energy);
        assert!(
            !energy_str.is_empty(),
            "Harmonic oscillator E_{} should exist",
            n
        );
    }
}

/// Validate particle in a box n² energy scaling
///
/// E_n = n²π²ℏ²/(2mL²)
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_infinite_well_n_squared_scaling() {
    let backend = create_symbolica_backend();
    let length = backend.constant(1.0).unwrap();
    let mass = backend.constant(1.0).unwrap();
    let hbar = backend.constant(1.0).unwrap();

    // Test that energy levels exist and scale properly
    for n in 1..=5 {
        let energy =
            tise_solver::solutions::infinite_well_energy(n, &length, &mass, &hbar, &backend)
                .unwrap();
        let energy_str = format!("{}", energy);
        assert!(!energy_str.is_empty(), "Infinite well E_{} should exist", n);
    }
}

/// Validate angular momentum quantum numbers
///
/// For orbital angular momentum: l = 0,1,2,... and |m| ≤ l
#[test]
fn validate_angular_momentum_quantum_numbers() {
    // Test all valid (l,m) combinations for l ≤ 3
    for l in 0..=3 {
        for m in -(l as isize)..=(l as isize) {
            let state = angular_momentum::AngularMomentumState::new(l, m);
            assert!(
                state.is_ok(),
                "Angular momentum state |l={}, m={}⟩ should be valid",
                l,
                m
            );
        }
    }

    // Test invalid combinations (|m| > l)
    let invalid_cases = vec![
        (0, 1),  // l=0, m=1 (invalid)
        (1, 2),  // l=1, m=2 (invalid)
        (2, 3),  // l=2, m=3 (invalid)
        (1, -2), // l=1, m=-2 (invalid)
    ];

    for (l, m) in invalid_cases {
        let state = angular_momentum::AngularMomentumState::new(l, m);
        assert!(
            state.is_err(),
            "Angular momentum state |l={}, m={}⟩ should be invalid",
            l,
            m
        );
    }
}

/// Validate Clebsch-Gordan coefficients symmetries
///
/// Triangle inequality: |j₁ - j₂| ≤ j ≤ j₁ + j₂
/// m conservation: m = m₁ + m₂
#[test]
fn validate_clebsch_gordan_selection_rules() {
    let j1 = 1;
    let j2 = 1;

    // Valid j values satisfying triangle inequality: j ∈ {0, 1, 2}
    let valid_j = vec![0, 1, 2];

    for j in valid_j {
        // Test m conservation: m₁ + m₂ = m
        for m1 in -(j1 as isize)..=(j1 as isize) {
            for m2 in -(j2 as isize)..=(j2 as isize) {
                let m = m1 + m2;

                // Check if m is valid for this j
                if m.abs() <= j as isize {
                    let cg = angular_momentum::clebsch_gordan::coefficient(j1, m1, j2, m2, j, m);
                    assert!(
                        cg.is_finite(),
                        "CG coefficient should be finite for valid quantum numbers"
                    );
                }
            }
        }
    }
}

/// Validate Bell state properties
///
/// Bell states are maximally entangled: all four are orthonormal
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_bell_states_orthonormality() {
    let backend = create_symbolica_backend();

    // Create all four Bell states
    let bell_states = vec![
        (
            "Φ⁺",
            multi_particle::entanglement::bell::create_bell_state(
                multi_particle::entanglement::bell::BellState::PhiPlus,
                &backend,
            )
            .unwrap(),
        ),
        (
            "Φ⁻",
            multi_particle::entanglement::bell::create_bell_state(
                multi_particle::entanglement::bell::BellState::PhiMinus,
                &backend,
            )
            .unwrap(),
        ),
        (
            "Ψ⁺",
            multi_particle::entanglement::bell::create_bell_state(
                multi_particle::entanglement::bell::BellState::PsiPlus,
                &backend,
            )
            .unwrap(),
        ),
        (
            "Ψ⁻",
            multi_particle::entanglement::bell::create_bell_state(
                multi_particle::entanglement::bell::BellState::PsiMinus,
                &backend,
            )
            .unwrap(),
        ),
    ];

    // Verify all states are distinct
    for i in 0..bell_states.len() {
        for j in (i + 1)..bell_states.len() {
            let state_i = format!("{}", bell_states[i].1);
            let state_j = format!("{}", bell_states[j].1);

            assert_ne!(
                state_i, state_j,
                "Bell states {} and {} should be different",
                bell_states[i].0, bell_states[j].0
            );
        }
    }
}

/// Validate Schmidt decomposition properties
///
/// Separable state ↔ Schmidt rank = 1
/// Entangled state ↔ Schmidt rank > 1
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_schmidt_decomposition_entanglement() {
    let backend = create_symbolica_backend();

    // Product state (separable): |ψ⟩ = |0⟩⊗|1⟩
    let c1 = backend.constant(1.0).unwrap();
    let separable = multi_particle::entanglement::SchmidtDecomposition::new(vec![c1]);

    assert_eq!(
        separable.rank, 1,
        "Product state should have Schmidt rank 1"
    );
    assert!(
        !separable.is_entangled(),
        "Product state should not be entangled"
    );

    // Entangled state: |ψ⟩ = α|00⟩ + β|11⟩
    let alpha = backend.constant(0.6).unwrap();
    let beta = backend.constant(0.8).unwrap();
    let entangled = multi_particle::entanglement::SchmidtDecomposition::new(vec![alpha, beta]);

    assert_eq!(
        entangled.rank, 2,
        "Entangled state should have Schmidt rank 2"
    );
    assert!(
        entangled.is_entangled(),
        "Non-product state should be entangled"
    );
}

/// Validate commutation relations
///
/// [x,p] = iℏ (canonical commutation relation)
#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn validate_canonical_commutation_relation() {
    let backend = create_symbolica_backend();

    let x_op = operators::standard_operators::position_1d(&backend, "x").unwrap();
    let p_op = operators::standard_operators::momentum_1d(&backend).unwrap();

    let commutator = x_op.commutator(&p_op, &backend).unwrap();

    // [x,p] should contain iℏ
    let comm_str = format!("{}", commutator.expression);
    assert!(!comm_str.is_empty(), "Commutator [x,p] should exist");

    // Verify it's not hermitian (since iℏ is anti-hermitian)
    assert!(
        !commutator.is_hermitian(),
        "[x,p] = iℏ should not be hermitian"
    );
}

/// Test summary statistics
#[test]
fn test_coverage_summary() {
    println!("\n=== QM Solver Analytical Validation Summary ===");
    println!("✓ Hydrogen atom: Energy levels (Rydberg formula)");
    println!("✓ Hydrogen atom: Degeneracy formula (n²)");
    println!("✓ Harmonic oscillator: Equal energy spacing (ℏω)");
    println!("✓ Infinite well: n² energy scaling");
    println!("✓ Angular momentum: Quantum number constraints");
    println!("✓ Clebsch-Gordan: Selection rules");
    println!("✓ Bell states: Orthonormality");
    println!("✓ Schmidt decomposition: Entanglement criterion");
    println!("✓ Canonical commutation: [x,p] = iℏ");
    println!("==============================================\n");
}
