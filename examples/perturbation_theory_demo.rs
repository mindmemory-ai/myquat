//! Perturbation Theory Demonstration
//!
//! Author: gA4ss
//!
//! This example demonstrates the use of perturbation theory in quantum mechanics,
//! including non-degenerate and degenerate perturbation theory, as well as
//! physical applications like the Stark and Zeeman effects.

use myquat::qm_solver::perturbation::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend, SymbolicResult};

fn main() -> SymbolicResult<()> {
    println!("=== Perturbation Theory Demonstration ===\n");

    // Create symbolic backend
    let backend = create_symbolica_backend();

    // 1. Non-degenerate perturbation theory
    println!("1. Non-Degenerate Perturbation Theory");
    println!("   System: Harmonic oscillator with anharmonic perturbation");
    non_degenerate_example(&backend)?;

    // 2. Degenerate perturbation theory
    println!("\n2. Degenerate Perturbation Theory");
    println!("   System: Degenerate energy levels with perturbation");
    degenerate_example(&backend)?;

    // 3. Stark effect
    println!("\n3. Stark Effect (Hydrogen in Electric Field)");
    stark_effect_example(&backend)?;

    // 4. Zeeman effect
    println!("\n4. Zeeman Effect (Atom in Magnetic Field)");
    zeeman_effect_example(&backend)?;

    // 5. Time-dependent perturbation
    println!("\n5. Time-Dependent Perturbation Theory");
    time_dependent_example(&backend)?;

    // 6. Variational methods
    println!("\n6. Variational Methods");
    variational_example(&backend)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demonstrate non-degenerate perturbation theory
fn non_degenerate_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Unperturbed energies: E_n = ℏω(n + 1/2)
    let e0 = vec![
        backend.parse("hbar*omega*0.5")?, // n=0
        backend.parse("hbar*omega*1.5")?, // n=1
        backend.parse("hbar*omega*2.5")?, // n=2
    ];

    // Perturbation matrix elements: H' = λx^4
    let lambda = backend.variable("lambda")?;
    let h_prime = vec![
        vec![
            backend.parse("lambda*a^4*3/4")?,       // ⟨0|x^4|0⟩
            backend.parse("lambda*a^4*sqrt(2)/2")?, // ⟨0|x^4|1⟩
            backend.parse("lambda*a^4*0.1")?,       // ⟨0|x^4|2⟩
        ],
        vec![
            backend.parse("lambda*a^4*sqrt(2)/2")?,
            backend.parse("lambda*a^4*11/4")?,
            backend.parse("lambda*a^4*sqrt(6)/2")?,
        ],
        vec![
            backend.parse("lambda*a^4*0.1")?,
            backend.parse("lambda*a^4*sqrt(6)/2")?,
            backend.parse("lambda*a^4*51/4")?,
        ],
    ];

    // First-order energy correction for ground state
    let e1 = non_degenerate::first_order_energy(0, &h_prime[0][0], backend)?;
    println!("   First-order energy correction:");
    println!("   {} = {}", e1, e1.correction);

    // Second-order energy correction
    let e2 = non_degenerate::second_order_energy(0, &e0, &h_prime, backend)?;
    println!("   Second-order energy correction:");
    println!("   {} (symbolic sum)", e2);

    // First-order state correction
    let state_coeffs = non_degenerate::first_order_state_coefficients(0, &e0, &h_prime, backend)?;
    println!("   First-order state correction coefficients:");
    for (i, coeff) in state_coeffs.iter().enumerate() {
        println!("   c_{} = {}", i, coeff);
    }

    println!("   ✅ Non-degenerate perturbation theory calculated");
    Ok(())
}

/// Demonstrate degenerate perturbation theory
fn degenerate_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Degenerate subspace: two states with same energy
    let subspace_indices = vec![0, 1];

    // Perturbation matrix within degenerate subspace
    let h_prime = vec![
        vec![backend.parse("V*a")?, backend.parse("V*b")?],
        vec![backend.parse("V*b")?, backend.parse("V*c")?],
    ];

    // Solve degenerate perturbation theory
    let eigenvalues = degenerate::solve_degenerate_subspace(&subspace_indices, &h_prime, backend)?;

    println!("   Degenerate subspace dimension: {}", eigenvalues.len());
    println!("   Energy shifts (diagonal elements):");
    for (i, eval) in eigenvalues.iter().enumerate() {
        println!("   ΔE_{} = {}", i, eval);
    }

    println!("   ✅ Degenerate perturbation theory solved");
    Ok(())
}

/// Demonstrate Stark effect
fn stark_effect_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Electric field strength
    let e_field = backend.variable("E")?;

    // Stark effect perturbation: H' = -eEz
    let h_stark = applications::stark_effect_linear(&e_field, backend)?;

    println!("   Perturbation Hamiltonian:");
    println!("   H' = {}", h_stark);
    println!("   (Linear Stark effect for hydrogen atom)");

    // For hydrogen n=2 level (l=0,1), Stark effect splits the degeneracy
    println!("   Effect: Splits degenerate n=2 levels");
    println!("   Energy shift: ΔE ∝ E (linear in field strength)");

    println!("   ✅ Stark effect perturbation constructed");
    Ok(())
}

/// Demonstrate Zeeman effect
fn zeeman_effect_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Magnetic field strength
    let b_field = backend.variable("B")?;

    // Zeeman effect perturbation: H' = μ_B B L_z
    let h_zeeman = applications::zeeman_effect(&b_field, backend)?;

    println!("   Perturbation Hamiltonian:");
    println!("   H' = {}", h_zeeman);
    println!("   (Zeeman effect for orbital angular momentum)");

    // Zeeman splitting
    println!("   Effect: Splits magnetic sublevels m_l");
    println!("   Energy shift: ΔE_m = μ_B B m (proportional to m_l)");

    println!("   ✅ Zeeman effect perturbation constructed");
    Ok(())
}

/// Demonstrate time-dependent perturbation theory
fn time_dependent_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Matrix element ⟨f|H'|i⟩
    let h_fi = backend.variable("V_fi")?;

    // Time
    let time = backend.variable("t")?;

    // Energy difference
    let delta_e = backend.parse("E_f - E_i")?;

    // ℏ
    let hbar = backend.variable("hbar")?;

    // Transition probability
    let prob = time_dependent::transition_probability(&h_fi, &time, &delta_e, &hbar, backend)?;

    println!("   Transition probability:");
    println!("   P_if(t) = (symbolic expression)");
    println!("   Contains sin²((E_f-E_i)t/(2ℏ)) factor");

    // Fermi's Golden Rule
    let rho = backend.variable("rho")?; // density of states
    let rate = time_dependent::fermis_golden_rule(&h_fi, &rho, &hbar, backend)?;

    println!("   Fermi's Golden Rule:");
    println!("   Γ = {}", rate);
    println!("   (Transition rate to continuum of states)");

    // Selection rules
    let allowed = time_dependent::check_selection_rules(0, 1, 0, 0);
    println!(
        "   Selection rules (l=0→1, m=0→0): {}",
        if allowed { "Allowed" } else { "Forbidden" }
    );

    println!("   ✅ Time-dependent perturbation theory demonstrated");
    Ok(())
}

/// Demonstrate variational methods
fn variational_example(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    // Trial wavefunction expectation value
    let h_exp = backend.parse("E_trial")?;
    let norm = backend.constant(1.0)?;

    // Rayleigh quotient
    let r_quotient = variational::rayleigh_quotient(&h_exp, &norm, backend)?;

    println!("   Rayleigh quotient:");
    println!("   R[ψ] = {}", r_quotient);
    println!("   (Upper bound on ground state energy)");

    // WKB approximation
    let momentum = backend.parse("sqrt(2*m*(E-V))")?;
    let wkb_psi = variational::wkb_wavefunction(&momentum, backend)?;

    println!("   WKB wavefunction:");
    println!("   ψ(x) ∝ {}", wkb_psi);
    println!("   (Semi-classical approximation)");

    // WKB quantization
    let wkb_quant = variational::wkb_quantization_condition(0, backend)?;
    println!("   WKB quantization (n=0):");
    println!("   ∮ p dx = {}", wkb_quant);

    println!("   ✅ Variational methods demonstrated");
    Ok(())
}
