//! Quantum Harmonic Oscillator Example
//!
//! Author: gA4ss
//!
//! Demonstrates quantum harmonic oscillator solutions.

use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

fn main() {
    println!("=== Quantum Harmonic Oscillator ===\n");

    let backend = create_symbolica_backend();

    // Parameters
    println!("1. System Parameters:");
    println!("   Potential: V(x) = (1/2)mω²x²");
    println!("   Classical frequency: ω");
    println!("   Mass: m\n");

    // Energy eigenvalues
    println!("2. Energy Eigenvalues:");
    println!("   En = ℏω(n + 1/2), n = 0, 1, 2, ...");
    for n in 0..5 {
        println!("   n={}: E = {}ℏω", n, n as f64 + 0.5);
    }
    println!();

    // Wavefunctions (symbolic)
    println!("3. Wavefunctions (Hermite polynomials):");
    println!("   ψn(x) = (mω/πℏ)^(1/4) / √(2^n n!) × Hn(√(mω/ℏ)x) × exp(-mωx²/2ℏ)");
    println!();
    println!("   Ground state (n=0):");
    println!("   ψ0(x) = (mω/πℏ)^(1/4) exp(-mωx²/2ℏ)");
    println!();
    println!("   First excited (n=1):");
    println!("   ψ1(x) = (mω/πℏ)^(1/4) √2 × (√(mω/ℏ)x) × exp(-mωx²/2ℏ)");
    println!();

    // Zero-point energy
    println!("4. Zero-Point Energy:");
    println!("   E0 = ℏω/2 (quantum fluctuations)");
    println!("   Classical minimum: E = 0");
    println!("   Quantum uncertainty prevents E = 0\n");

    // Ladder operators
    println!("5. Ladder Operators:");
    let a_plus = backend.parse("a_plus").unwrap();
    let a_minus = backend.parse("a_minus").unwrap();

    println!("   Raising: a† |n⟩ = √(n+1) |n+1⟩");
    println!("   Lowering: a |n⟩ = √n |n-1⟩");
    println!("   [a, a†] = 1");
    println!("   Hamiltonian: H = ℏω(a†a + 1/2)\n");

    // Position and momentum
    println!("6. Position and Momentum:");
    println!("   ⟨x⟩ = 0 (symmetric)");
    println!("   ⟨p⟩ = 0 (no net momentum)");
    println!("   Δx·Δp = ℏ(n + 1/2) (uncertainty)\n");

    // Classical correspondence
    println!("7. Classical Correspondence:");
    println!("   High n: quantum → classical");
    println!("   Amplitude: A = √(2En/mω²)");
    println!("   Period: T = 2π/ω\n");

    println!("=== Complete ===");
}
