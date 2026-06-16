//! VQE H2 Molecule Demo
//!
//! Demonstrates Variational Quantum Eigensolver for H2 ground state.
//! See `examples/teaching_demos/08_vqe_chemistry.rs` for full VQE workflow.

use myquat::hamiltonian::{Hamiltonian, PauliString};
use myquat::QuantumCircuit;

fn main() {
    println!("VQE H2 Demo");
    println!("===========\n");

    let h2 = build_h2_hamiltonian();
    println!("H2: {} qubits, {} terms", h2.num_qubits, h2.num_terms());

    let ansatz = create_ansatz();
    println!("Ansatz: {} gates, depth {}", ansatz.size(), ansatz.depth());
    println!("For full VQE workflow, see examples/teaching_demos/08_vqe_chemistry.rs");
}

fn build_h2_hamiltonian() -> Hamiltonian {
    let mut h = Hamiltonian::new(4);
    let terms: [(&str, f64); 15] = [
        ("IIII", -0.8105),
        ("IIIZ", 0.1721),
        ("IIZI", -0.2228),
        ("IZII", 0.1721),
        ("ZIII", -0.2228),
        ("IIZZ", 0.1686),
        ("IZIZ", 0.1205),
        ("IZZI", 0.1686),
        ("ZIIZ", 0.1686),
        ("ZIZI", 0.1205),
        ("ZZII", 0.1686),
        ("IIXX", 0.0454),
        ("IIYY", 0.0454),
        ("XXII", 0.0454),
        ("YYII", 0.0454),
    ];
    for (ps, coeff) in &terms {
        h.add_term(
            PauliString::from_str(ps).unwrap(),
            num_complex::Complex64::new(*coeff, 0.0),
        )
        .unwrap();
    }
    h
}

fn create_ansatz() -> QuantumCircuit {
    let mut c = QuantumCircuit::new(4, 0);
    c.h(0).unwrap();
    c.h(1).unwrap();
    c.cx(0, 1).unwrap();
    c.cx(1, 2).unwrap();
    c.cx(2, 3).unwrap();
    c
}
