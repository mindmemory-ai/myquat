//! Quantum Time Evolution Example
//!
//! Author: gA4ss
//!
//! Demonstrates time evolution of quantum states.

use myquat::qm_solver::{TDSESolver, TimeEvolutionMethod};
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

fn main() {
    println!("=== Quantum Time Evolution ===\n");

    let backend = create_symbolica_backend();

    // Time-dependent Schrodinger equation
    println!("1. Time-Dependent Schrodinger Equation:");
    println!("   iℏ ∂|ψ⟩/∂t = H|ψ⟩");
    println!("   Unitary evolution preserves norm\n");

    // Time evolution operator
    println!("2. Time Evolution Operator:");
    println!("   U(t) = exp(-iHt/ℏ)");
    println!("   |ψ(t)⟩ = U(t)|ψ(0)⟩");
    println!("   Properties:");
    println!("   - Unitary: U†U = I");
    println!("   - Composition: U(t2)U(t1) = U(t1+t2)");
    println!("   - Inverse: U†(t) = U(-t)\n");

    // Stationary states
    println!("3. Stationary States:");
    println!("   Energy eigenstates: H|n⟩ = En|n⟩");
    println!("   Time evolution: |n(t)⟩ = exp(-iEnt/ℏ)|n⟩");
    println!("   Phase factor only, no change in |ψ|²\n");

    // Superposition evolution
    println!("4. Superposition Evolution:");
    println!("   |ψ(0)⟩ = Σn cn|n⟩");
    println!("   |ψ(t)⟩ = Σn cn exp(-iEnt/ℏ)|n⟩");
    println!("   Relative phases → oscillations\n");

    // Create TDSE solver
    let h_expr = backend.variable("H").unwrap();
    let h_op = myquat::qm_solver::QuantumOperator::new(
        h_expr,
        myquat::qm_solver::OperatorType::Hamiltonian,
        "H",
        true,
    );

    match TDSESolver::new(h_op, &backend) {
        Ok(_solver) => {
            println!("5. TDSE Solver Created:");
            println!("   Time-independent Hamiltonian solver initialized\n");
        }
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // Ehrenfest theorem
    println!("6. Ehrenfest Theorem:");
    println!("   d⟨x⟩/dt = ⟨p⟩/m (quantum → classical)");
    println!("   d⟨p⟩/dt = -⟨dV/dx⟩");
    println!("   Quantum expectation values follow classical equations\n");

    // Heisenberg picture
    println!("7. Heisenberg Picture:");
    println!("   States fixed, operators evolve");
    println!("   A(t) = U†(t)A(0)U(t)");
    println!("   dA/dt = (i/ℏ)[H,A] + ∂A/∂t\n");

    // Interaction picture
    println!("8. Interaction Picture:");
    println!("   H = H0 + V(t)");
    println!("   Hybrid: states and operators both evolve");
    println!("   Useful for perturbation theory\n");

    // Adiabatic evolution
    println!("9. Adiabatic Evolution:");
    println!("   Slow H(t) changes");
    println!("   State follows instantaneous eigenstate");
    println!("   Adiabatic theorem: no transitions if slow enough\n");

    // Quantum Zeno effect
    println!("10. Quantum Zeno Effect:");
    println!("   Frequent measurements freeze evolution");
    println!("   'Watched pot never boils'");
    println!("   Measurement back-action\n");

    println!("=== Complete ===");
}
