//! Hamiltonian Optimizer Demo
//!
//! Author: gA4ss
//!
//! This example demonstrates advanced Hamiltonian optimization techniques including:
//! - Qubit Tapering based on symmetry detection
//! - Commuting term grouping for measurement optimization
//! - Jordan-Wigner transformation for fermionic systems
//! - Term merging and simplification

use myquat::hamiltonian::{
    constructors, Hamiltonian, HamiltonianOptimizer, JordanWignerTransform, PauliString,
};
use num_complex::Complex64;

fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(70));
    println!("{}", title);
    println!("{}", "=".repeat(70));
}

/// Demonstration 1: Symmetry Detection and Qubit Tapering
fn demo_qubit_tapering() {
    print_separator("1. Qubit Tapering via Symmetry Detection");

    // Create a Hamiltonian with known symmetries
    // H = Z0*Z1 + Z1*Z2 + X0*X1 + X1*X2
    let mut h = Hamiltonian::new(3);

    h.add_term(
        PauliString::from_str("ZZI").unwrap(),
        Complex64::new(1.0, 0.0),
    )
    .unwrap();
    h.add_term(
        PauliString::from_str("IZZ").unwrap(),
        Complex64::new(1.0, 0.0),
    )
    .unwrap();
    h.add_term(
        PauliString::from_str("XXI").unwrap(),
        Complex64::new(0.5, 0.0),
    )
    .unwrap();
    h.add_term(
        PauliString::from_str("IXX").unwrap(),
        Complex64::new(0.5, 0.0),
    )
    .unwrap();

    println!("\nOriginal Hamiltonian:");
    println!("  Number of qubits: {}", h.num_qubits);
    println!("  Number of terms: {}", h.terms.len());
    println!("{}", h.to_string());

    let mut optimizer = HamiltonianOptimizer::new();

    // Detect symmetries
    println!("\nDetecting symmetries...");
    let symmetries = optimizer.detect_symmetries(&h);
    println!("  Found {} potential symmetry operators", symmetries.len());

    // Apply qubit tapering
    if !symmetries.is_empty() {
        println!("\nApplying qubit tapering...");
        if let Ok(tapered) = optimizer.apply_qubit_tapering(&h, &symmetries[0]) {
            println!("  Tapered Hamiltonian:");
            println!("  Number of qubits: {}", tapered.num_qubits);
            println!("  Number of terms: {}", tapered.terms.len());
            println!(
                "  Qubit reduction: {} -> {}",
                h.num_qubits, tapered.num_qubits
            );
        }
    }
}

/// Demonstration 2: Commuting Term Grouping
fn demo_commuting_groups() {
    print_separator("2. Commuting Term Grouping for Measurement Optimization");

    // Create Heisenberg model Hamiltonian
    println!("\nCreating 3-qubit Heisenberg model...");
    let h = constructors::heisenberg_model(3, 1.0, 1.0, 1.0).unwrap();

    println!("  Total terms: {}", h.terms.len());
    println!("  Terms:");
    for (i, term) in h.terms.iter().enumerate() {
        println!(
            "    {}: {} * {}",
            i,
            term.coefficient,
            term.pauli_string.to_string()
        );
    }

    let mut optimizer = HamiltonianOptimizer::new();

    // Group commuting terms
    println!("\nGrouping commuting terms...");
    let groups = optimizer.group_commuting_terms(&h);

    println!("  Found {} commuting groups:", groups.len());
    for (i, group) in groups.iter().enumerate() {
        println!("    Group {}: {} terms", i, group.len());
        print!("      Indices: [");
        for (j, &idx) in group.iter().enumerate() {
            if j > 0 {
                print!(", ");
            }
            print!("{}", idx);
        }
        println!("]");
    }

    println!("\nMeasurement optimization:");
    println!(
        "  Without grouping: {} measurement circuits needed",
        h.terms.len()
    );
    println!(
        "  With grouping: {} measurement circuits needed",
        groups.len()
    );
    println!(
        "  Reduction: {:.1}%",
        (1.0 - groups.len() as f64 / h.terms.len() as f64) * 100.0
    );
}

/// Demonstration 3: Term Merging and Simplification
fn demo_term_merging() {
    print_separator("3. Term Merging and Simplification");

    // Create a Hamiltonian with duplicate terms
    let mut h = Hamiltonian::new(2);

    println!("\nAdding terms with duplicates:");
    let xx = PauliString::from_str("XX").unwrap();
    let zz = PauliString::from_str("ZZ").unwrap();
    let yz = PauliString::from_str("YZ").unwrap();

    h.add_term(xx.clone(), Complex64::new(1.0, 0.0)).unwrap();
    println!("  Added: 1.0 * XX");
    h.add_term(zz.clone(), Complex64::new(0.5, 0.0)).unwrap();
    println!("  Added: 0.5 * ZZ");
    h.add_term(xx.clone(), Complex64::new(2.0, 0.0)).unwrap();
    println!("  Added: 2.0 * XX");
    h.add_term(zz.clone(), Complex64::new(0.5, 0.0)).unwrap();
    println!("  Added: 0.5 * ZZ");
    h.add_term(yz.clone(), Complex64::new(1.0, 0.0)).unwrap();
    println!("  Added: 1.0 * YZ");

    println!("\nBefore merging:");
    println!("  Number of terms: {}", h.terms.len());

    let optimizer = HamiltonianOptimizer::new();
    let merged = optimizer.merge_identical_terms(&h);

    println!("\nAfter merging:");
    println!("  Number of terms: {}", merged.terms.len());
    println!("  Terms:");
    for term in &merged.terms {
        println!(
            "    {} * {}",
            term.coefficient,
            term.pauli_string.to_string()
        );
    }

    println!(
        "\nTerm reduction: {} -> {} ({:.1}% reduction)",
        h.terms.len(),
        merged.terms.len(),
        (1.0 - merged.terms.len() as f64 / h.terms.len() as f64) * 100.0
    );
}

/// Demonstration 4: Jordan-Wigner Transformation
fn demo_jordan_wigner() {
    print_separator("4. Jordan-Wigner Transformation for Fermionic Systems");

    let n_modes = 4;
    let jw = JordanWignerTransform::new(n_modes);

    println!("\nFermionic system with {} modes", n_modes);
    println!("Jordan-Wigner mapping to {} qubits\n", n_modes);

    // Creation operator
    println!("Creation operator a†_1:");
    let a_dag_1 = jw.creation_operator(1).unwrap();
    println!("  a†_1 = (X_1 - iY_1)/2 with Z-string Z_0");
    println!("  {} Pauli terms:", a_dag_1.len());
    for term in &a_dag_1 {
        println!(
            "    {} * {}",
            term.coefficient,
            term.pauli_string.to_string()
        );
    }

    // Annihilation operator
    println!("\nAnnihilation operator a_1:");
    let a_1 = jw.annihilation_operator(1).unwrap();
    println!("  a_1 = (X_1 + iY_1)/2 with Z-string Z_0");
    println!("  {} Pauli terms:", a_1.len());
    for term in &a_1 {
        println!(
            "    {} * {}",
            term.coefficient,
            term.pauli_string.to_string()
        );
    }

    // Number operator
    println!("\nNumber operator n_1 = a†_1 * a_1:");
    let n_1 = jw.number_operator(1).unwrap();
    println!("  n_1 = (I - Z_1)/2");
    println!("  {} Pauli terms:", n_1.len());
    for term in &n_1 {
        println!(
            "    {} * {}",
            term.coefficient,
            term.pauli_string.to_string()
        );
    }

    // Hopping term
    println!("\nHopping term: a†_0*a_1 + a†_1*a_0");
    let hopping = jw.hopping_term(0, 1).unwrap();
    println!("  This represents particle hopping between sites 0 and 1");
    println!(
        "  {} Pauli terms after simplification:",
        hopping.terms.len()
    );
    for (i, term) in hopping.terms.iter().take(5).enumerate() {
        println!(
            "    {} * {}",
            term.coefficient,
            term.pauli_string.to_string()
        );
        if i == 4 && hopping.terms.len() > 5 {
            println!("    ... ({} more terms)", hopping.terms.len() - 5);
        }
    }
}

/// Demonstration 5: Complete Optimization Pipeline
fn demo_optimization_pipeline() {
    print_separator("5. Complete Optimization Pipeline");

    println!("\nCreating Ising model Hamiltonian...");
    let h = constructors::ising_model(4, 1.0, 0.5).unwrap();

    println!("  Number of qubits: {}", h.num_qubits);
    println!("  Number of terms: {}", h.terms.len());

    let mut optimizer = HamiltonianOptimizer::new();

    println!("\nStep 1: Merging identical terms...");
    let h1 = optimizer.merge_identical_terms(&h);
    println!("  Terms: {} -> {}", h.terms.len(), h1.terms.len());

    println!("\nStep 2: Grouping commuting terms...");
    let groups = optimizer.group_commuting_terms(&h1);
    println!("  Commuting groups: {}", groups.len());

    println!("\nStep 3: Detecting symmetries...");
    let symmetries = optimizer.detect_symmetries(&h1);
    println!("  Potential symmetries found: {}", symmetries.len());

    println!("\nStep 4: Generating optimization report...");
    let report = optimizer.generate_report(&h, &h1);

    println!("\nOptimization Report:");
    println!("{}", "─".repeat(70));
    println!("Original Hamiltonian:");
    println!("  Qubits: {}", report.original_qubits);
    println!("  Terms: {}", report.original_terms);
    println!("  Estimated gates: {}", report.estimated_original_gates);

    println!("\nOptimized Hamiltonian:");
    println!("  Qubits: {}", report.optimized_qubits);
    println!("  Terms: {}", report.optimized_terms);
    println!("  Estimated gates: {}", report.estimated_optimized_gates);
    println!("  Commuting groups: {}", report.commuting_groups);

    println!("\nReductions:");
    println!("  Term reduction: {:.1}%", report.term_reduction_percent);
    println!("  Gate reduction: {:.1}%", report.gate_reduction_percent);
    println!(
        "  Measurement reduction: {:.1}%",
        report.measurement_reduction_percent
    );
    println!("{}", "─".repeat(70));
}

fn main() {
    println!("\n{}{}{}", "╔", "═".repeat(68), "╗");
    println!(
        "║{}{}║",
        " Hamiltonian Optimizer Demonstration ",
        " ".repeat(31)
    );
    println!("{}{}{}", "╚", "═".repeat(68), "╝");

    demo_qubit_tapering();
    demo_commuting_groups();
    demo_term_merging();
    demo_jordan_wigner();
    demo_optimization_pipeline();

    print_separator("Summary");
    println!("\nThis demo showcased:");
    println!("  1. Qubit Tapering - Reducing qubit count via symmetry detection");
    println!("  2. Commuting Groups - Optimizing measurements by grouping");
    println!("  3. Term Merging - Simplifying Hamiltonians by combining terms");
    println!("  4. Jordan-Wigner - Mapping fermionic operators to qubits");
    println!("  5. Complete Pipeline - Applying all optimizations together");

    println!("\nThese techniques are essential for:");
    println!("  - NISQ algorithms (VQE, QAOA)");
    println!("  - Quantum chemistry simulations");
    println!("  - Quantum many-body physics");
    println!("  - Efficient quantum circuit compilation");

    println!("\n{}", "═".repeat(70));
    println!("Demo completed successfully!");
    println!("{}", "═".repeat(70));
}
