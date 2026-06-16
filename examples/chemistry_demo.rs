//! Quantum Chemistry Demo
//! Author: gA4ss
//!
//! Demonstrates quantum chemistry calculations using VQE with different
//! fermion-to-qubit mappings.

use myquat::hamiltonian::{MappingMethod, MoleculeDatabase};
use myquat::Result;
use std::time::Instant;

fn main() -> Result<()> {
    println!("Quantum Chemistry with VQE");
    println!("===========================\n");

    // Load molecule database
    let db = MoleculeDatabase::new();
    println!("Molecule Database loaded with {} molecules", db.size());
    println!("Available molecules: {:?}\n", db.list_molecules());

    // Test 1: H2 molecule with different mappings
    println!("Test 1: H2 Molecule Ground State Energy");
    println!("{}", "-".repeat(60));

    let h2 = db.get("H2").unwrap();
    println!("Molecule: {}", h2.name);
    println!("  Atoms: {:?}", h2.geometry.atoms);
    println!("  Coordinates (Angstrom):");
    for (i, (x, y, z)) in h2.geometry.coordinates.iter().enumerate() {
        println!(
            "    {}: ({:.3}, {:.3}, {:.3})",
            h2.geometry.atoms[i], x, y, z
        );
    }
    println!("  Number of electrons: {}", h2.num_electrons);
    println!("  Number of orbitals: {}", h2.num_orbitals);
    println!(
        "  Reference energy: {:.6} Ha\n",
        h2.reference_energy.unwrap()
    );

    // Test different mappings
    let mappings = vec![
        ("Jordan-Wigner", MappingMethod::JordanWigner),
        ("Bravyi-Kitaev", MappingMethod::BravyiKitaev),
        ("Parity", MappingMethod::Parity),
    ];

    for (name, method) in &mappings {
        println!("  Mapping: {}", name);

        let start = Instant::now();
        let hamiltonian = h2
            .hamiltonian
            .as_ref()
            .unwrap()
            .to_qubit_hamiltonian(*method)?;
        let elapsed = start.elapsed();

        println!("    Number of qubits: {}", hamiltonian.num_qubits);
        println!("    Number of terms: {}", hamiltonian.terms.len());
        println!("    Constant term: {:.6}", hamiltonian.constant_term.re);
        println!("    Mapping time: {:?}", elapsed);

        // In a real implementation, we would run VQE here
        println!("    [VQE simulation would run here]");
        println!();
    }

    println!();

    // Test 2: LiH molecule
    println!("Test 2: LiH Molecule");
    println!("{}", "-".repeat(60));

    let lih = db.get("LiH").unwrap();
    println!("Molecule: {}", lih.name);
    println!("  Number of electrons: {}", lih.num_electrons);
    println!("  Number of orbitals: {}", lih.num_orbitals);
    println!(
        "  Reference energy: {:.6} Ha",
        lih.reference_energy.unwrap()
    );

    let hamiltonian_lih = lih
        .hamiltonian
        .as_ref()
        .unwrap()
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)?;

    println!("  Qubit Hamiltonian (Jordan-Wigner):");
    println!("    Number of qubits: {}", hamiltonian_lih.num_qubits);
    println!("    Number of terms: {}", hamiltonian_lih.terms.len());
    println!();

    // Test 3: H2O molecule
    println!("Test 3: H2O Molecule");
    println!("{}", "-".repeat(60));

    let h2o = db.get("H2O").unwrap();
    println!("Molecule: {}", h2o.name);
    println!("  Atoms: {:?}", h2o.geometry.atoms);
    println!("  Number of electrons: {}", h2o.num_electrons);
    println!("  Number of orbitals: {}", h2o.num_orbitals);
    println!(
        "  Reference energy: {:.6} Ha",
        h2o.reference_energy.unwrap()
    );

    let hamiltonian_h2o = h2o
        .hamiltonian
        .as_ref()
        .unwrap()
        .to_qubit_hamiltonian(MappingMethod::JordanWigner)?;

    println!("  Qubit Hamiltonian (Jordan-Wigner):");
    println!("    Number of qubits: {}", hamiltonian_h2o.num_qubits);
    println!("    Number of terms: {}", hamiltonian_h2o.terms.len());
    println!();

    // Test 4: Comparison table
    println!("Test 4: Molecule Comparison");
    println!("{}", "-".repeat(60));
    println!(
        "{:<12} | {:>10} | {:>10} | {:>14}",
        "Molecule", "Electrons", "Orbitals", "Ref Energy (Ha)"
    );
    println!("{}", "-".repeat(60));

    for mol_name in db.list_molecules() {
        if let Some(mol) = db.get(&mol_name) {
            if let Some(energy) = mol.reference_energy {
                println!(
                    "{:<12} | {:>10} | {:>10} | {:>14.6}",
                    mol.name, mol.num_electrons, mol.num_orbitals, energy
                );
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Quantum Chemistry Demo Completed Successfully!");
    println!("{}", "=".repeat(60));

    Ok(())
}
