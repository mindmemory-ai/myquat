//! Debug: what KAK coefficients does CX·Rz·CX produce?
use myquat::*;
use ndarray::Array2;
use num_complex::Complex64;

fn main() {
    // Build CX·Rz(0.5)·CX
    println!("=== KAK coefficients for CX·Rz(0.5)·CX ===");
    let mut c = QuantumCircuit::new(2, 0);
    c.cx(0, 1).unwrap();
    c.rz(1, Parameter::Float(0.5)).unwrap();
    c.cx(0, 1).unwrap();
    let u = c.unitary(&std::collections::HashMap::new()).unwrap();

    // Also print the unitary
    println!("Unitary:");
    for i in 0..4 {
        for j in 0..4 {
            print!("{:8.4} ", u[[i, j]]);
        }
        println!();
    }

    use crate::two_qubit_decompose::{TwoQubitBasis, TwoQubitDecomposer, TwoQubitMatrixBuilder};
    let decomposer = TwoQubitDecomposer::new(TwoQubitBasis::CX);

    // Direct KAK
    let kak = decomposer.decompose(u.view()).unwrap();
    println!("\nKAK coefficients: {:?}", kak.coefficients);
    println!("KAK num_basis_gates: {}", kak.num_basis_gates);

    // Shannon
    let shannon = decomposer.shannon_decomposition(u.view()).unwrap();
    println!("\nShannon num_cnots: {}", shannon.num_cnots);

    // Manual: build Rzz(0.5) directly
    println!("\n=== Direct Rzz(0.5) matrix ===");
    let theta = 0.5;
    let c_val = Complex64::new((theta / 2.0_f64).cos(), 0.0);
    let s_val = Complex64::new(0.0, -(theta / 2.0_f64).sin());
    // exp(-i θ/2 Z⊗Z) = c·I⊗I - i·s·Z⊗Z
    // Z⊗Z = diag(1, -1, -1, 1)
    let mut rzz = Array2::zeros((4, 4));
    rzz[[0, 0]] = Complex64::new((theta / 2.0_f64).cos(), -(theta / 2.0_f64).sin()); // c - i*s for +1
    rzz[[1, 1]] = Complex64::new((theta / 2.0_f64).cos(), (theta / 2.0_f64).sin()); // c + i*s for -1
    rzz[[2, 2]] = Complex64::new((theta / 2.0_f64).cos(), (theta / 2.0_f64).sin()); // c + i*s for -1
    rzz[[3, 3]] = Complex64::new((theta / 2.0_f64).cos(), -(theta / 2.0_f64).sin()); // c - i*s for +1

    for i in 0..4 {
        for j in 0..4 {
            print!("{:8.4} ", rzz[[i, j]]);
        }
        println!();
    }

    // Compare: are they the same?
    let diff = &u - &rzz;
    let frob = (diff.iter().map(|c| c.norm_sqr()).sum::<f64>() / 4.0).sqrt();
    println!("\nFrob diff between circuit unitary and Rzz: {:.6}", frob);

    // KAK of Rzz
    let kak_rzz = decomposer.decompose(rzz.view()).unwrap();
    println!("KAK(Rzz) coefficients: {:?}", kak_rzz.coefficients);
    println!("KAK(Rzz) num_basis_gates: {}", kak_rzz.num_basis_gates);
}
