//! Quantum Spin Systems Example
//!
//! Author: gA4ss
//!
//! Demonstrates quantum spin and Pauli matrices.

use myquat::qm_solver::angular_momentum;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

fn main() {
    println!("=== Quantum Spin Systems ===\n");

    let backend = create_symbolica_backend();

    // Spin-1/2
    println!("1. Spin-1/2 Particles:");
    println!("   Fundamental quantum property");
    println!("   Eigenvalues: ±ℏ/2");
    println!("   Two-state system: |↑⟩, |↓⟩\n");

    // Pauli matrices
    println!("2. Pauli Matrices:");
    match angular_momentum::spin::PauliMatrices::new(&backend) {
        Ok(pauli) => {
            println!("   σx = {}", pauli.sigma_x);
            println!("   σy = {}", pauli.sigma_y);
            println!("   σz = {}", pauli.sigma_z);
        }
        Err(e) => println!("   Error: {:?}", e),
    }
    println!();

    // Commutation relations
    println!("3. Pauli Matrix Properties:");
    println!("   [σi, σj] = 2iεijk σk");
    println!("   {{σi, σj}} = 2δij I");
    println!("   σi² = I");
    println!("   Tr(σi) = 0\n");

    // Spin operators
    println!("4. Spin Operators:");
    println!("   S = (ℏ/2)σ");
    println!("   Sx = (ℏ/2)σx");
    println!("   Sy = (ℏ/2)σy");
    println!("   Sz = (ℏ/2)σz\n");

    // Eigenstates
    println!("5. Spin-z Eigenstates:");
    println!("   |↑⟩ = |sz=+ℏ/2⟩ = (1, 0)ᵀ");
    println!("   |↓⟩ = |sz=-ℏ/2⟩ = (0, 1)ᵀ\n");

    println!("6. Spin-x Eigenstates:");
    println!("   |→⟩ = (|↑⟩ + |↓⟩)/√2");
    println!("   |←⟩ = (|↑⟩ - |↓⟩)/√2\n");

    // Spinors
    println!("7. General Spinor:");
    let alpha = backend.variable("alpha").unwrap();
    let beta = backend.variable("beta").unwrap();
    let _spinor = angular_momentum::spin::Spinor::new(alpha, beta);
    println!("   |ψ⟩ = α|↑⟩ + β|↓⟩");
    println!("   Normalization: |α|² + |β|² = 1");
    println!();

    // Bloch sphere
    println!("8. Bloch Sphere Representation:");
    println!("   |ψ⟩ = cos(θ/2)|↑⟩ + e^(iφ)sin(θ/2)|↓⟩");
    println!("   θ: polar angle [0, π]");
    println!("   φ: azimuthal angle [0, 2π]");
    println!("   Pure states: surface of sphere\n");

    // Stern-Gerlach
    println!("9. Stern-Gerlach Experiment:");
    println!("   Spatial separation by spin");
    println!("   Sequential measurements → quantum nature");
    println!("   No classical analogue\n");

    // Spin-orbit coupling
    println!("10. Spin-Orbit Coupling:");
    let lambda = backend.variable("lambda").unwrap();
    let l_dot_s = backend.variable("L_dot_S").unwrap();
    match angular_momentum::spin::spin_orbit_coupling(&lambda, &l_dot_s, &backend) {
        Ok(hso) => {
            println!("   Hso = {}", hso);
            println!("   Couples orbital and spin angular momentum");
        }
        Err(e) => println!("   Error: {:?}", e),
    }
    println!();

    // Addition of spins
    println!("11. Addition of Angular Momenta:");
    println!("   Two spin-1/2: J = L + S");
    println!("   |j, m⟩ = Σ ⟨l,ml;s,ms|j,m⟩ |l,ml⟩|s,ms⟩");
    println!("   Clebsch-Gordan coefficients\n");

    // Applications
    println!("12. Applications:");
    println!("   - Electron spin resonance (ESR)");
    println!("   - Nuclear magnetic resonance (NMR)");
    println!("   - Spintronics");
    println!("   - Quantum computing (qubits)\n");

    println!("=== Complete ===");
}
