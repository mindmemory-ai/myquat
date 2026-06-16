//! Hydrogen Atom Quantum Mechanics Example
//!
//! Author: gA4ss
//!
//! Demonstrates solving the hydrogen atom problem using the quantum mechanics solver.

use myquat::qm_solver::angular_momentum;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend};

fn main() {
    println!("=== Hydrogen Atom Quantum Mechanics ===\n");

    let backend = create_symbolica_backend();

    // Hydrogen atom parameters
    println!("1. Hydrogen Atom Parameters:");
    println!("   - Bohr radius: a0 = 0.529 Angstrom");
    println!("   - Ground state energy: E1 = -13.6 eV");
    println!("   - Rydberg constant: R_inf = 13.6 eV\n");

    // Energy levels
    println!("2. Energy Levels (En = -13.6/n² eV):");
    for n in 1..=5 {
        let energy = -13.6 / (n * n) as f64;
        println!("   n={}: E = {:.3} eV", n, energy);
    }
    println!();

    // Radial wavefunctions
    println!("3. Radial Wavefunctions:");
    let a0 = backend.variable("a0").unwrap();

    match angular_momentum::central_force::hydrogen_radial(1, 0, &a0, &backend) {
        Ok(r_10) => {
            println!("   R_10(r) = {}", r_10);
        }
        Err(e) => println!("   Error computing R_10: {:?}", e),
    }

    match angular_momentum::central_force::hydrogen_radial(2, 0, &a0, &backend) {
        Ok(r_20) => {
            println!("   R_20(r) = {}", r_20);
        }
        Err(e) => println!("   Error computing R_20: {:?}", e),
    }
    println!();

    // Spherical harmonics
    println!("4. Angular Wavefunctions (Spherical Harmonics):");
    match angular_momentum::SphericalHarmonic::new(0, 0, "theta", "phi", &backend) {
        Ok(_y_00) => {
            println!("   Y_0^0(θ,φ) = 1/√(4π) (s orbital)");
        }
        Err(e) => println!("   Error: {:?}", e),
    }

    match angular_momentum::SphericalHarmonic::new(1, 0, "theta", "phi", &backend) {
        Ok(_y_10) => {
            println!("   Y_1^0(θ,φ) = √(3/4π) cos(θ) (p_z orbital)");
        }
        Err(e) => println!("   Error: {:?}", e),
    }
    println!();

    // Quantum numbers
    println!("5. Quantum Numbers:");
    println!("   Principal: n = 1, 2, 3, ...");
    println!("   Angular: l = 0, 1, ..., n-1");
    println!("   Magnetic: m = -l, -l+1, ..., l-1, l");
    println!("   Spin: s = ±1/2\n");

    // Orbital names
    println!("6. Orbital Names:");
    println!("   l=0: s orbital (spherical)");
    println!("   l=1: p orbital (dumbbell)");
    println!("   l=2: d orbital (cloverleaf)");
    println!("   l=3: f orbital (complex)\n");

    // Selection rules
    println!("7. Electric Dipole Selection Rules:");
    println!("   Δl = ±1");
    println!("   Δm = 0, ±1");
    println!("   Δs = 0\n");

    println!("=== Complete ===");
}
