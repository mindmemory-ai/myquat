//! Quantum Entanglement Example
//!
//! Author: gA4ss
//!
//! Demonstrates quantum entanglement and Bell states.

use myquat::qm_solver::multi_particle;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

fn main() {
    println!("=== Quantum Entanglement ===\n");

    let backend = create_symbolica_backend();

    // Bell states
    println!("1. Bell States (Maximally Entangled):");
    println!("   |Φ+⟩ = (|00⟩ + |11⟩)/√2");
    println!("   |Φ-⟩ = (|00⟩ - |11⟩)/√2");
    println!("   |Ψ+⟩ = (|01⟩ + |10⟩)/√2");
    println!("   |Ψ-⟩ = (|01⟩ - |10⟩)/√2\n");

    // Create Bell state
    match multi_particle::entanglement::bell::create_bell_state(
        multi_particle::entanglement::bell::BellState::PhiPlus,
        &backend,
    ) {
        Ok(bell_state) => {
            println!("2. Generated Bell State |Φ+⟩:");
            println!("   Expression: {}\n", bell_state);
        }
        Err(e) => println!("   Error: {:?}\n", e),
    }

    // Entanglement properties
    println!("3. Entanglement Properties:");
    println!("   - Non-separable: ψ(1,2) ≠ ψ(1) ⊗ ψ(2)");
    println!("   - Measurement correlation: 100%");
    println!("   - No local hidden variables (Bell's theorem)");
    println!("   - Violates Bell inequality\n");

    // Schmidt decomposition
    println!("4. Schmidt Decomposition:");
    println!("   |ψ⟩ = Σi √λi |ui⟩A ⊗ |vi⟩B");
    println!("   Schmidt number: rank of decomposition");
    println!("   Entangled if Schmidt number > 1\n");

    let coeff1 = backend.constant(1.0 / 2.0_f64.sqrt()).unwrap();
    let coeff2 = backend.constant(1.0 / 2.0_f64.sqrt()).unwrap();
    let schmidt = multi_particle::entanglement::SchmidtDecomposition::new(vec![coeff1, coeff2]);

    println!("5. Bell State |Φ+⟩ Schmidt Decomposition:");
    println!("   λ1 ≈ 0.707, λ2 ≈ 0.707");
    println!("   Schmidt rank: {}", schmidt.rank);
    println!("   Entangled: {}\n", schmidt.is_entangled());

    // Entanglement entropy
    println!("6. Entanglement Entropy:");
    println!("   S = -Tr(ρA log ρA)");
    println!("   For Bell state: S = log 2 ≈ 0.693");
    println!("   Separable state: S = 0");
    println!("   Maximal for Bell states\n");

    // Reduced density matrix
    println!("7. Reduced Density Matrix:");
    println!("   ρA = TrB(|ψ⟩⟨ψ|)");
    println!("   For |Φ+⟩: ρA = I/2 (maximally mixed)");
    println!("   Purity: Tr(ρA²) = 1/2 (mixed)\n");

    // Concurrence
    println!("8. Entanglement Measures:");
    println!("   Concurrence C ∈ [0, 1]");
    println!("   C = 0: separable");
    println!("   C = 1: maximally entangled");
    println!("   For Bell states: C = 1\n");

    // EPR paradox
    println!("9. EPR Paradox:");
    println!("   Einstein-Podolsky-Rosen (1935)");
    println!("   'Spooky action at a distance'");
    println!("   Quantum mechanics: complete description");
    println!("   Local realism: violated by experiments\n");

    // Bell's inequality
    println!("10. Bell's Inequality:");
    println!("   CHSH: |⟨AB⟩ + ⟨AB'⟩ + ⟨A'B⟩ - ⟨A'B'⟩| ≤ 2 (classical)");
    println!("   Quantum: can reach 2√2 ≈ 2.828");
    println!("   Violation proves quantum nonlocality\n");

    // Applications
    println!("11. Applications:");
    println!("   - Quantum teleportation");
    println!("   - Quantum cryptography (QKD)");
    println!("   - Superdense coding");
    println!("   - Quantum computing gates\n");

    println!("=== Complete ===");
}
