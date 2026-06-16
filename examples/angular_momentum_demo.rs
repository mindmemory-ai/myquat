//! Angular Momentum Demonstration
//!
//! Author: gA4ss
//!
//! This example demonstrates angular momentum theory in quantum mechanics,
//! including orbital angular momentum, spin, angular momentum coupling,
//! and central force problems.

use myquat::qm_solver::angular_momentum::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend, SymbolicError, SymbolicResult};

fn main() -> SymbolicResult<()> {
    println!("=== Angular Momentum in Quantum Mechanics ===\n");

    // Create symbolic backend
    let backend = create_symbolica_backend();

    // 1. Orbital angular momentum
    println!("1. Orbital Angular Momentum");
    orbital_angular_momentum_demo(&backend)?;

    // 2. Spin angular momentum
    println!("\n2. Spin Angular Momentum");
    spin_angular_momentum_demo(&backend)?;

    // 3. Angular momentum coupling
    println!("\n3. Angular Momentum Coupling");
    angular_momentum_coupling_demo(&backend)?;

    // 4. Spherical harmonics
    println!("\n4. Spherical Harmonics");
    spherical_harmonics_demo(&backend)?;

    // 5. Central force problems
    println!("\n5. Central Force Problems");
    central_force_demo(&backend)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demonstrate orbital angular momentum
fn orbital_angular_momentum_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    let hbar = backend.variable("hbar")?;

    println!("   Orbital Angular Momentum Operators");
    println!("   L² and L_z eigenvalue equations:");

    // L² eigenvalue for l=1 (p orbital)
    let l = 1;
    let l_squared = orbital::l_squared_eigenvalue(l, &hbar, backend)?;
    println!("   L²|l=1,m⟩ = {} |l=1,m⟩", l_squared);
    println!("   (Expected: 2ℏ²)");

    // L_z eigenvalues for different m
    println!("\n   L_z eigenvalues for l=1:");
    for m in -1..=1 {
        let l_z = orbital::l_z_eigenvalue(m, &hbar, backend)?;
        println!("   L_z|l=1,m={}⟩ = {} |l=1,m={}⟩", m, l_z, m);
    }

    // Ladder operators
    println!("\n   Ladder Operators:");
    let l_plus = orbital::ladder_plus(1, 0, &hbar, backend)?;
    println!("   L₊|l=1,m=0⟩ = {} |l=1,m=1⟩", l_plus);
    println!("   (Expected: ℏ√2)");

    let l_minus = orbital::ladder_minus(1, 0, &hbar, backend)?;
    println!("   L₋|l=1,m=0⟩ = {} |l=1,m=-1⟩", l_minus);
    println!("   (Expected: ℏ√2)");

    // Boundary conditions
    let l_plus_max = orbital::ladder_plus(1, 1, &hbar, backend)?;
    println!("\n   Boundary: L₊|l=1,m=1⟩ = {} (annihilates)", l_plus_max);

    println!("   ✅ Orbital angular momentum operators demonstrated");
    Ok(())
}

/// Demonstrate spin angular momentum
fn spin_angular_momentum_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Spin-1/2 System");

    // Pauli matrices
    let pauli = spin::PauliMatrices::new(backend)?;
    println!("   Pauli Matrices:");
    println!("   σₓ = {}", pauli.sigma_x);
    println!("   σᵧ = {}", pauli.sigma_y);
    println!("   σᵤ = {}", pauli.sigma_z);

    // Spin eigenstates
    let spin_up = spin::Spinor::spin_up(backend)?;
    let spin_down = spin::Spinor::spin_down(backend)?;

    println!("\n   Spin Eigenstates:");
    println!("   |↑⟩ = [{}, {}]ᵀ", spin_up.alpha, spin_up.beta);
    println!("   |↓⟩ = [{}, {}]ᵀ", spin_down.alpha, spin_down.beta);

    // Spin-orbit coupling
    let lambda = backend.variable("lambda")?;
    let l_dot_s = backend.parse("L·S")?;
    let h_so = spin::spin_orbit_coupling(&lambda, &l_dot_s, backend)?;

    println!("\n   Spin-Orbit Coupling:");
    println!("   H_SO = {}", h_so);
    println!("   (Fine structure in atomic spectra)");

    println!("   ✅ Spin angular momentum demonstrated");
    Ok(())
}

/// Demonstrate angular momentum coupling
fn angular_momentum_coupling_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Coupling Two Angular Momenta");
    println!("   J = L + S (LS coupling)");

    // Example: l=1 (p orbital) + s=1/2 (electron spin)
    // Note: Using integer arithmetic, so s=1 for demonstration
    let l = 1;
    let s = 1;

    let j_values: Vec<usize> = (((l as isize - s as isize).unsigned_abs())..=(l + s)).collect();

    println!("\n   For l={}, s={}:", l, s);
    println!("   Possible j values: {:?}", j_values);
    println!("   (Triangle inequality: |l-s| ≤ j ≤ l+s)");

    // Clebsch-Gordan coefficients
    println!("\n   Clebsch-Gordan Coefficients:");
    println!("   ⟨j₁ m₁ j₂ m₂|j m⟩");

    let cg_example = clebsch_gordan::coefficient(1, 0, 1, 0, 0, 0);
    println!("   ⟨1 0 1 0|0 0⟩ = {:.4}", cg_example);

    let cg_example2 = clebsch_gordan::coefficient(1, 1, 1, -1, 1, 0);
    println!("   ⟨1 1 1 -1|1 0⟩ = {:.4}", cg_example2);

    // Selection rules
    println!("\n   Selection Rules:");
    println!("   - m = m₁ + m₂ (magnetic quantum number conservation)");
    println!("   - |j₁ - j₂| ≤ j ≤ j₁ + j₂ (triangle inequality)");

    println!("   ✅ Angular momentum coupling demonstrated");
    Ok(())
}

/// Demonstrate spherical harmonics
fn spherical_harmonics_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Spherical Harmonics Y_l^m(θ,φ)");
    println!("   Solutions to angular part of Laplacian");

    // Common spherical harmonics
    let orbitals = vec![
        (0, 0, "s orbital"),
        (1, 0, "p_z orbital"),
        (1, 1, "p_x + ip_y"),
        (2, 0, "d_z²"),
        (2, 2, "d_xy"),
    ];

    println!("\n   Atomic Orbitals:");
    for (l, m, name) in orbitals {
        let ylm = SphericalHarmonic::new(l, m, "theta", "phi", backend)?;
        println!("   Y_{}^{}(θ,φ) - {}", l, m, name);
        println!("   Expression: {}", ylm.expression);
    }

    // Angular momentum state
    let state = AngularMomentumState::new(2, 1).map_err(|e| SymbolicError::InvalidExpression(e))?;
    println!("\n   Angular Momentum State:");
    println!("   {}", state);
    println!("   (d orbital with m=1)");

    println!("   ✅ Spherical harmonics demonstrated");
    Ok(())
}

/// Demonstrate central force problems
fn central_force_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Central Force Problems");
    println!("   Radial equation: [-ℏ²/(2μ)∇² + V(r)]ψ = Eψ");

    // Hydrogen atom
    println!("\n   1. Hydrogen Atom");
    let a0 = backend.variable("a0")?;

    let r_21 = central_force::hydrogen_radial(2, 1, &a0, backend)?;
    println!("   R₂₁(r) = {}", r_21);
    println!("   (2p orbital radial function)");

    let r_10 = central_force::hydrogen_radial(1, 0, &a0, backend)?;
    println!("   R₁₀(r) = {}", r_10);
    println!("   (1s orbital - ground state)");

    // 3D Harmonic oscillator
    println!("\n   2. 3D Harmonic Oscillator");
    let omega = backend.variable("omega")?;

    let ho_00 = central_force::harmonic_oscillator_3d(0, 0, &omega, backend)?;
    println!("   Ground state (n=0, l=0):");
    println!("   E = {}", ho_00.energy);
    println!("   (Expected: 3ℏω/2)");

    let ho_01 = central_force::harmonic_oscillator_3d(0, 1, &omega, backend)?;
    println!("\n   First excited (n=0, l=1):");
    println!("   E = {}", ho_01.energy);
    println!("   (Expected: 5ℏω/2)");

    // Spherical well
    println!("\n   3. Spherical Well Potential");
    let v0 = backend.variable("V0")?;
    let radius = backend.variable("a")?;

    let well = central_force::spherical_well(&v0, &radius, backend)?;
    println!("   V(r) = {}", well);
    println!("   (Finite square well)");

    // Energy levels
    println!("\n   Energy Level Structure:");
    println!("   Hydrogen: E_n = -13.6 eV / n²");
    println!("   3D HO: E_n = ℏω(2n + l + 3/2)");
    println!("   Degeneracy depends on l quantum number");

    println!("   ✅ Central force problems demonstrated");
    Ok(())
}
