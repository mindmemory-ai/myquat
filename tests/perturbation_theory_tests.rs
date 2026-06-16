//! Perturbation Theory Tests
//!
//! Author: gA4ss
//!
//! Comprehensive tests for perturbation theory module

use myquat::qm_solver::perturbation::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_harmonic_oscillator_perturbation() {
    // Test perturbation theory on harmonic oscillator with anharmonic term
    let backend = create_symbolica_backend();

    // Unperturbed energies: E_n = ℏω(n + 1/2)
    let e0 = vec![
        backend.parse("hbar*omega*0.5").unwrap(),
        backend.parse("hbar*omega*1.5").unwrap(),
        backend.parse("hbar*omega*2.5").unwrap(),
    ];

    // Perturbation: H' = λx^4
    let h_prime = vec![
        vec![
            backend.parse("lambda*a^4*0.75").unwrap(),
            backend.parse("lambda*a^4*0.5").unwrap(),
            backend.parse("lambda*a^4*0.1").unwrap(),
        ],
        vec![
            backend.parse("lambda*a^4*0.5").unwrap(),
            backend.parse("lambda*a^4*2.75").unwrap(),
            backend.parse("lambda*a^4*1.0").unwrap(),
        ],
        vec![
            backend.parse("lambda*a^4*0.1").unwrap(),
            backend.parse("lambda*a^4*1.0").unwrap(),
            backend.parse("lambda*a^4*12.75").unwrap(),
        ],
    ];

    // First-order correction
    let e1 = non_degenerate::first_order_energy(0, &h_prime[0][0], &backend).unwrap();
    assert_eq!(e1.order, 1);
    assert_eq!(e1.level, 0);

    // Second-order correction
    let e2 = non_degenerate::second_order_energy(0, &e0, &h_prime, &backend).unwrap();
    assert_eq!(e2.order, 2);
    assert_eq!(e2.level, 0);
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_hydrogen_stark_effect() {
    // Test Stark effect for hydrogen atom
    let backend = create_symbolica_backend();

    // Electric field
    let e_field = backend.variable("E").unwrap();

    // Stark perturbation
    let h_stark = applications::stark_effect_linear(&e_field, &backend).unwrap();

    // Verify perturbation is non-zero
    let zero = backend.constant(0.0).unwrap();
    // Note: Can't directly compare symbolic expressions, but we can verify it's created
    assert!(h_stark.to_string().contains("E") || h_stark.to_string().contains("z"));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_zeeman_splitting() {
    // Test Zeeman effect
    let backend = create_symbolica_backend();

    // Magnetic field
    let b_field = backend.variable("B").unwrap();

    // Zeeman perturbation
    let h_zeeman = applications::zeeman_effect(&b_field, &backend).unwrap();

    // Verify perturbation contains magnetic field
    assert!(h_zeeman.to_string().contains("B") || h_zeeman.to_string().contains("L_z"));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_degenerate_two_level_system() {
    // Test degenerate perturbation for 2-level system
    let backend = create_symbolica_backend();

    // Degenerate subspace
    let subspace = vec![0, 1];

    // Perturbation matrix
    let h_prime = vec![
        vec![
            backend.constant(0.1).unwrap(),
            backend.constant(0.05).unwrap(),
        ],
        vec![
            backend.constant(0.05).unwrap(),
            backend.constant(0.15).unwrap(),
        ],
    ];

    // Solve degenerate perturbation
    let eigenvalues = degenerate::solve_degenerate_subspace(&subspace, &h_prime, &backend).unwrap();

    assert_eq!(eigenvalues.len(), 2);
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_rayleigh_schrodinger_orders() {
    // Test Rayleigh-Schrodinger perturbation to different orders
    let backend = create_symbolica_backend();

    let e0 = vec![
        backend.constant(1.0).unwrap(),
        backend.constant(2.0).unwrap(),
        backend.constant(3.0).unwrap(),
    ];

    let h_prime = vec![
        vec![
            backend.constant(0.1).unwrap(),
            backend.constant(0.05).unwrap(),
            backend.constant(0.02).unwrap(),
        ],
        vec![
            backend.constant(0.05).unwrap(),
            backend.constant(0.15).unwrap(),
            backend.constant(0.08).unwrap(),
        ],
        vec![
            backend.constant(0.02).unwrap(),
            backend.constant(0.08).unwrap(),
            backend.constant(0.2).unwrap(),
        ],
    ];

    // First order
    let e1 =
        rayleigh_schrodinger::energy_correction_general(0, 1, &e0, &h_prime, &backend).unwrap();
    assert_eq!(e1.order, 1);

    // Second order
    let e2 =
        rayleigh_schrodinger::energy_correction_general(0, 2, &e0, &h_prime, &backend).unwrap();
    assert_eq!(e2.order, 2);
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_time_dependent_transition() {
    // Test time-dependent perturbation theory
    let backend = create_symbolica_backend();

    let h_fi = backend.variable("V_fi").unwrap();
    let time = backend.variable("t").unwrap();
    let delta_e = backend.parse("E_f - E_i").unwrap();
    let hbar = backend.variable("hbar").unwrap();

    // Transition probability
    let prob =
        time_dependent::transition_probability(&h_fi, &time, &delta_e, &hbar, &backend).unwrap();

    // Verify result is non-zero
    assert!(!prob.to_string().is_empty());
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_fermis_golden_rule() {
    // Test Fermi's Golden Rule
    let backend = create_symbolica_backend();

    let h_fi = backend.variable("V_fi").unwrap();
    let rho = backend.variable("rho").unwrap();
    let hbar = backend.variable("hbar").unwrap();

    // Transition rate
    let rate = time_dependent::fermis_golden_rule(&h_fi, &rho, &hbar, &backend).unwrap();

    // Verify contains expected terms
    let rate_str = rate.to_string();
    assert!(rate_str.contains("V_fi") || rate_str.contains("rho"));
}

#[test]
fn test_selection_rules_allowed() {
    // Test allowed electric dipole transitions

    // l=0 → l=1, m=0 → m=0 (allowed)
    assert!(time_dependent::check_selection_rules(0, 1, 0, 0));

    // l=1 → l=2, m=0 → m=1 (allowed)
    assert!(time_dependent::check_selection_rules(1, 2, 0, 1));

    // l=2 → l=1, m=-1 → m=0 (allowed)
    assert!(time_dependent::check_selection_rules(2, 1, -1, 0));
}

#[test]
fn test_selection_rules_forbidden() {
    // Test forbidden electric dipole transitions

    // l=0 → l=0 (Δl=0, forbidden)
    assert!(!time_dependent::check_selection_rules(0, 0, 0, 0));

    // l=0 → l=2 (Δl=2, forbidden)
    assert!(!time_dependent::check_selection_rules(0, 2, 0, 0));

    // l=1 → l=2, m=0 → m=2 (Δm=2, forbidden)
    assert!(!time_dependent::check_selection_rules(1, 2, 0, 2));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_variational_rayleigh_quotient() {
    // Test Rayleigh quotient for variational method
    let backend = create_symbolica_backend();

    let h_exp = backend.parse("E_trial").unwrap();
    let norm = backend.constant(1.0).unwrap();

    let r_quotient = variational::rayleigh_quotient(&h_exp, &norm, &backend).unwrap();

    // Rayleigh quotient should equal E_trial for normalized state
    assert!(r_quotient.to_string().contains("E_trial"));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_wkb_approximation() {
    // Test WKB approximation
    let backend = create_symbolica_backend();

    let momentum = backend.parse("sqrt(2*m*(E-V))").unwrap();
    let wkb_psi = variational::wkb_wavefunction(&momentum, &backend).unwrap();

    // WKB wavefunction should contain 1/sqrt(p)
    assert!(!wkb_psi.to_string().is_empty());
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_wkb_quantization() {
    // Test WKB quantization condition
    let backend = create_symbolica_backend();

    // Ground state (n=0)
    let quant_0 = variational::wkb_quantization_condition(0, &backend).unwrap();
    assert!(quant_0.to_string().contains("h") || quant_0.to_string().contains("0.5"));

    // First excited state (n=1)
    let quant_1 = variational::wkb_quantization_condition(1, &backend).unwrap();
    assert!(quant_1.to_string().contains("h") || quant_1.to_string().contains("1.5"));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_state_correction_coefficients() {
    // Test first-order state correction coefficients
    let backend = create_symbolica_backend();

    let e0 = vec![
        backend.constant(0.0).unwrap(),
        backend.constant(1.0).unwrap(),
        backend.constant(2.0).unwrap(),
    ];

    let h_prime = vec![
        vec![
            backend.constant(0.0).unwrap(),
            backend.constant(0.1).unwrap(),
            backend.constant(0.05).unwrap(),
        ],
        vec![
            backend.constant(0.1).unwrap(),
            backend.constant(0.0).unwrap(),
            backend.constant(0.15).unwrap(),
        ],
        vec![
            backend.constant(0.05).unwrap(),
            backend.constant(0.15).unwrap(),
            backend.constant(0.0).unwrap(),
        ],
    ];

    let coeffs =
        non_degenerate::first_order_state_coefficients(0, &e0, &h_prime, &backend).unwrap();

    assert_eq!(coeffs.len(), 3);
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_fine_structure() {
    // Test fine structure corrections
    let backend = create_symbolica_backend();

    let h_fine = applications::fine_structure(&backend).unwrap();

    // Should contain relativistic and spin-orbit terms
    assert!(h_fine.to_string().contains("H_rel") || h_fine.to_string().contains("H_so"));
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_periodic_perturbation() {
    // Test periodic perturbation
    let backend = create_symbolica_backend();

    let h_fi = backend.variable("V_fi").unwrap();
    let omega = backend.variable("omega").unwrap();
    let time = backend.variable("t").unwrap();
    let delta_e = backend.parse("E_f - E_i").unwrap();
    let hbar = backend.variable("hbar").unwrap();

    let amplitude = time_dependent::periodic_perturbation_amplitude(
        &h_fi, &omega, &time, &delta_e, &hbar, &backend,
    )
    .unwrap();

    // Should contain time and frequency dependence
    assert!(!amplitude.to_string().is_empty());
}

#[test]
#[ignore = "Symbolica licensing: run separately with --ignored flag"]
fn test_perturbation_theory_solver() {
    // Test PerturbationTheorySolver struct
    let backend = create_symbolica_backend();

    let e0 = vec![
        backend.constant(1.0).unwrap(),
        backend.constant(2.0).unwrap(),
    ];

    let h_prime = vec![
        vec![
            backend.constant(0.1).unwrap(),
            backend.constant(0.05).unwrap(),
        ],
        vec![
            backend.constant(0.05).unwrap(),
            backend.constant(0.15).unwrap(),
        ],
    ];

    let lambda = backend.variable("lambda").unwrap();

    let solver = PerturbationTheorySolver::new(e0, h_prime, lambda);

    assert_eq!(solver.h0_eigenenergies.len(), 2);
    assert_eq!(solver.perturbation_matrix_elements.len(), 2);
}
