//! Quantum Dynamics Demonstration
//!
//! Author: gA4ss
//!
//! This example demonstrates quantum dynamics including different pictures
//! (Schrodinger, Heisenberg, Interaction), time evolution, and path integrals.

use myquat::qm_solver::dynamics::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend, SymbolicResult};

fn main() -> SymbolicResult<()> {
    println!("=== Quantum Dynamics Demonstration ===\n");

    // Create symbolic backend
    let backend = create_symbolica_backend();

    // 1. Pictures of quantum mechanics
    println!("1. Pictures of Quantum Mechanics");
    quantum_pictures_demo(&backend)?;

    // 2. Heisenberg picture
    println!("\n2. Heisenberg Picture");
    heisenberg_picture_demo(&backend)?;

    // 3. Interaction picture
    println!("\n3. Interaction Picture");
    interaction_picture_demo(&backend)?;

    // 4. Path integral formulation
    println!("\n4. Path Integral Formulation");
    path_integral_demo(&backend)?;

    // 5. Time evolution of expectation values
    println!("\n5. Time Evolution of Expectation Values");
    expectation_values_demo(&backend)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demonstrate different pictures of quantum mechanics
fn quantum_pictures_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Three Equivalent Formulations:");

    let pictures = vec![
        (
            QuantumPicture::Schrodinger,
            "States evolve, operators fixed",
        ),
        (QuantumPicture::Heisenberg, "Operators evolve, states fixed"),
        (
            QuantumPicture::Interaction,
            "Hybrid approach for perturbations",
        ),
    ];

    for (picture, description) in pictures {
        println!("   {} Picture: {}", picture, description);
    }

    println!("\n   Schrodinger Picture:");
    println!("   iℏ ∂|ψ⟩/∂t = H|ψ⟩");
    println!("   |ψ(t)⟩ = U(t)|ψ(0)⟩ where U(t) = e^(-iHt/ℏ)");

    println!("\n   Heisenberg Picture:");
    println!("   dA/dt = (i/ℏ)[H,A] + ∂A/∂t");
    println!("   A(t) = U†(t)A(0)U(t)");

    println!("\n   Interaction Picture:");
    println!("   H = H₀ + V(t)");
    println!("   A_I(t) = e^(iH₀t/ℏ) A_S e^(-iH₀t/ℏ)");

    println!("   ✅ Quantum pictures explained");
    Ok(())
}

/// Demonstrate Heisenberg picture dynamics
fn heisenberg_picture_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Heisenberg Equation of Motion");

    // Position operator
    let x = backend.variable("x")?;
    let h = backend.variable("H")?;
    let hbar = backend.variable("hbar")?;

    // dx/dt = (i/ℏ)[H,x]
    let dxdt = heisenberg::equation_of_motion(&x, &h, &hbar, backend)?;
    println!("   dx/dt = (i/ℏ)[H,x] = {}", dxdt);

    // Momentum operator
    let p = backend.variable("p")?;
    let dpdt = heisenberg::equation_of_motion(&p, &h, &hbar, backend)?;
    println!("   dp/dt = (i/ℏ)[H,p] = {}", dpdt);

    // Time evolution operator
    println!("\n   Time Evolution of Operators:");
    let u_t = backend.parse("U(t)")?;
    let a_0 = backend.variable("A0")?;

    let a_t = heisenberg::operator_time_evolution(&a_0, &u_t, backend)?;
    println!("   A(t) = U†(t)A(0)U(t)");
    println!("   Result: {}", a_t);

    // Ehrenfest theorem
    println!("\n   Ehrenfest Theorem:");
    println!("   Classical-quantum correspondence");

    let x_exp = backend.parse("<x>")?;
    let p_exp = backend.parse("<p>")?;
    let mass = backend.variable("m")?;

    let dx_exp_dt = heisenberg::ehrenfest_theorem(&x_exp, &p_exp, &mass, backend)?;
    println!("   d⟨x⟩/dt = ⟨p⟩/m = {}", dx_exp_dt);
    println!("   d⟨p⟩/dt = -⟨dV/dx⟩");

    println!("   ✅ Heisenberg picture demonstrated");
    Ok(())
}

/// Demonstrate interaction picture
fn interaction_picture_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Interaction Picture Formalism");
    println!("   Used for time-dependent perturbation theory");

    // Transform to interaction picture
    let a_s = backend.variable("A_S")?;
    let h0 = backend.variable("H0")?;
    let t = backend.variable("t")?;
    let hbar = backend.variable("hbar")?;

    let a_i = interaction::to_interaction_picture(&a_s, &h0, &t, &hbar, backend)?;
    println!("\n   Transformation:");
    println!("   A_I(t) = e^(iH₀t/ℏ) A_S e^(-iH₀t/ℏ)");
    println!("   Result: {}", a_i);

    // Time evolution operator
    let v_i = backend.variable("V_I")?;
    let u_i = interaction::time_evolution_operator(&v_i, &t, &hbar, backend)?;

    println!("\n   Time Evolution Operator:");
    println!("   U_I(t) = T exp[-i/ℏ ∫₀ᵗ V_I(t')dt']");
    println!("   (Time-ordered exponential)");

    // Dyson series
    println!("\n   Dyson Series Expansion:");
    let dyson_terms = interaction::dyson_series(3, &v_i, backend)?;

    for (n, term) in dyson_terms.iter().enumerate() {
        println!("   Order {}: {}", n, term);
    }

    println!("\n   Applications:");
    println!("   - Time-dependent perturbation theory");
    println!("   - Scattering theory");
    println!("   - Quantum field theory");

    println!("   ✅ Interaction picture demonstrated");
    Ok(())
}

/// Demonstrate path integral formulation
fn path_integral_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Feynman Path Integral Formulation");
    println!("   Sum over all possible paths");

    // Propagator
    let xi = backend.variable("x_i")?;
    let xf = backend.variable("x_f")?;
    let action = backend.variable("S")?;
    let hbar = backend.variable("hbar")?;

    let propagator = path_integral::propagator(&xi, &xf, &action, &hbar, backend)?;

    println!("\n   Feynman Propagator:");
    println!("   K(x_f,t_f; x_i,t_i) = ∫ D[x(t)] exp(iS[x]/ℏ)");
    println!("   Result: {}", propagator);

    // Classical action
    let lagrangian = backend.variable("L")?;
    let time = backend.variable("t")?;

    let s = path_integral::classical_action(&lagrangian, &time, backend)?;
    println!("\n   Classical Action:");
    println!("   S[x] = ∫₀ᵗ L(x,ẋ,t')dt'");
    println!("   Result: {}", s);

    // WKB from path integrals
    let s0 = backend.variable("S0")?;
    let wkb = path_integral::wkb_from_path_integral(&s0, &hbar, backend)?;

    println!("\n   WKB Approximation:");
    println!("   ψ(x) ∝ exp(iS₀(x)/ℏ)");
    println!("   (Semi-classical limit ℏ → 0)");

    // Stationary phase
    let phase = backend.variable("phi")?;
    let stationary = path_integral::stationary_phase(&phase, backend)?;

    println!("\n   Stationary Phase Approximation:");
    println!("   Main contribution from δS/δx = 0");
    println!("   (Classical path dominates)");

    println!("\n   Physical Interpretation:");
    println!("   - Quantum: sum over all paths");
    println!("   - Classical limit: stationary action path");
    println!("   - Connects quantum and classical mechanics");

    println!("   ✅ Path integral formulation demonstrated");
    Ok(())
}

/// Demonstrate time evolution of expectation values
fn expectation_values_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Time Evolution of Expectation Values");

    // General formula
    println!("\n   General Formula:");
    println!("   d⟨A⟩/dt = (i/ℏ)⟨[H,A]⟩ + ⟨∂A/∂t⟩");

    // Example: position and momentum
    let x = backend.variable("x")?;
    let h = backend.variable("H")?;
    let psi = backend.variable("psi")?;
    let hbar = backend.variable("hbar")?;

    let dx_exp_dt = expectation::time_derivative(&x, &h, &psi, &hbar, backend)?;

    println!("\n   Position Expectation:");
    println!("   d⟨x⟩/dt = (i/ℏ)⟨[H,x]⟩");
    println!("   For H = p²/2m + V(x): d⟨x⟩/dt = ⟨p⟩/m");

    println!("\n   Momentum Expectation:");
    println!("   d⟨p⟩/dt = (i/ℏ)⟨[H,p]⟩");
    println!("   For H = p²/2m + V(x): d⟨p⟩/dt = -⟨dV/dx⟩");

    // Conservation laws
    println!("\n   Conservation Laws:");
    println!("   If [H,A] = 0, then d⟨A⟩/dt = 0");
    println!("   Examples:");
    println!("   - Energy: [H,H] = 0 → ⟨H⟩ conserved");
    println!("   - Angular momentum (central force): [H,L] = 0");
    println!("   - Parity (symmetric potential): [H,P] = 0");

    // Uncertainty relations
    println!("\n   Heisenberg Uncertainty Relations:");
    println!("   Δx Δp ≥ ℏ/2");
    println!("   ΔE Δt ≥ ℏ/2");
    println!("   (Fundamental quantum limits)");

    // Time scales
    println!("\n   Characteristic Time Scales:");
    println!("   - Oscillation period: T ~ ℏ/ΔE");
    println!("   - Tunneling time: τ ~ ℏ/Γ");
    println!("   - Decoherence time: T₂ (environment dependent)");

    println!("   ✅ Expectation value evolution demonstrated");
    Ok(())
}
