//! H2 Molecule VQE Example
//! Author: gA4ss
//!
//! Demonstrates the use of VQE to find the ground state energy of H2 molecule.

use myquat::algorithms::vqe_core::GradientDescentOptimizer;
use myquat::algorithms::{VQEAnsatz, VQEConfig, VQE};
use myquat::hamiltonian::{Hamiltonian, PauliOperator, PauliString};
use myquat::{QuantumCircuit, Result};
use num_complex::Complex64;
use std::time::Instant;

/// Create H2 molecule Hamiltonian at equilibrium bond distance
fn create_h2_hamiltonian() -> Result<Hamiltonian> {
    // H2 Hamiltonian in minimal basis (STO-3G)
    // Bond distance: 0.735 Angstrom
    // These are the standard coefficients for H2 in Jordan-Wigner encoding

    let mut hamiltonian = Hamiltonian::new(4);

    // Identity term (constant)
    hamiltonian.constant_term = Complex64::new(0.7137539936876207, 0.0);

    // Single-qubit terms
    let terms = vec![
        // Z0
        (
            vec![
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::I,
            ],
            Complex64::new(-0.4718416346703808, 0.0),
        ),
        // Z1
        (
            vec![
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
            ],
            Complex64::new(-0.4718416346703808, 0.0),
        ),
        // Z2
        (
            vec![
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
            ],
            Complex64::new(0.17771287746257453, 0.0),
        ),
        // Z3
        (
            vec![
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
            ],
            Complex64::new(0.17771287746257453, 0.0),
        ),
        // Two-qubit terms
        // Z0Z1
        (
            vec![
                PauliOperator::Z,
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
            ],
            Complex64::new(0.12293305049846498, 0.0),
        ),
        // Z0Z2
        (
            vec![
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
            ],
            Complex64::new(0.16768318993186888, 0.0),
        ),
        // Z0Z3
        (
            vec![
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
            ],
            Complex64::new(0.17627640789223785, 0.0),
        ),
        // Z1Z2
        (
            vec![
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::Z,
                PauliOperator::I,
            ],
            Complex64::new(0.17627640789223785, 0.0),
        ),
        // Z1Z3
        (
            vec![
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::Z,
            ],
            Complex64::new(0.16768318993186888, 0.0),
        ),
        // Z2Z3
        (
            vec![
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::Z,
            ],
            Complex64::new(0.12293305049846498, 0.0),
        ),
        // Exchange terms
        // X0X1Y2Y3
        (
            vec![
                PauliOperator::X,
                PauliOperator::X,
                PauliOperator::Y,
                PauliOperator::Y,
            ],
            Complex64::new(0.04532220205287402, 0.0),
        ),
        // X0Y1Y2X3
        (
            vec![
                PauliOperator::X,
                PauliOperator::Y,
                PauliOperator::Y,
                PauliOperator::X,
            ],
            Complex64::new(0.04532220205287402, 0.0),
        ),
        // Y0X1X2Y3
        (
            vec![
                PauliOperator::Y,
                PauliOperator::X,
                PauliOperator::X,
                PauliOperator::Y,
            ],
            Complex64::new(0.04532220205287402, 0.0),
        ),
        // Y0Y1X2X3
        (
            vec![
                PauliOperator::Y,
                PauliOperator::Y,
                PauliOperator::X,
                PauliOperator::X,
            ],
            Complex64::new(0.04532220205287402, 0.0),
        ),
    ];

    for (operators, coeff) in terms {
        let pauli_string = PauliString::new(operators, Complex64::new(1.0, 0.0));
        hamiltonian.add_term(pauli_string, coeff)?;
    }

    Ok(hamiltonian)
}

fn main() -> Result<()> {
    println!("H2 Molecule Ground State Energy - VQE Example");
    println!("==============================================\n");

    // Create H2 Hamiltonian
    println!("Creating H2 Hamiltonian...");
    let hamiltonian = create_h2_hamiltonian()?;
    println!("  Number of qubits: {}", hamiltonian.num_qubits);
    println!("  Number of terms: {}", hamiltonian.terms.len());
    println!("  Constant term: {:.10}\n", hamiltonian.constant_term.re);

    // Known exact ground state energy for comparison
    let exact_energy = -1.137270422018;
    println!("Exact ground state energy: {:.10} Ha\n", exact_energy);

    // Test 1: Hardware-efficient ansatz with Simplex optimizer
    println!("Test 1: Hardware-Efficient Ansatz");
    println!("{}", "-".repeat(50));

    let n_qubits = 4;
    let n_layers = 2;
    let ansatz = VQEAnsatz::new(n_qubits, n_layers);

    println!("  Ansatz layers: {}", n_layers);
    println!("  Number of parameters: {}", ansatz.num_parameters());

    // Create VQE instance
    let optimizer = Box::new(GradientDescentOptimizer::new(200, 0.01, 1e-6));
    let config = VQEConfig {
        max_iterations: 200,
        energy_tolerance: 1e-6,
        gradient_tolerance: 1e-5,
        use_gradients: false,
        store_history: true,
        parameter_shift: std::f64::consts::PI / 2.0,
    };

    let mut vqe = VQE::new(optimizer, config);

    // Initial parameters (small random values)
    let initial_params = vec![0.1; ansatz.num_parameters()];

    println!("  Running VQE optimization...");
    let start = Instant::now();

    let result = vqe.run(
        |params| ansatz.build_hardware_efficient_ansatz(params),
        &hamiltonian,
        &initial_params,
    )?;

    let elapsed = start.elapsed();

    println!("\n  Results:");
    println!(
        "    Ground state energy: {:.10} Ha",
        result.ground_state_energy
    );
    println!("    Exact energy:        {:.10} Ha", exact_energy);
    println!(
        "    Energy error:        {:.10} Ha ({:.6}%)",
        result.ground_state_energy - exact_energy,
        ((result.ground_state_energy - exact_energy) / exact_energy * 100.0).abs()
    );
    println!("    Iterations:          {}", result.num_iterations);
    println!("    Converged:           {}", result.converged);
    println!("    Time elapsed:        {:.2?}", elapsed);
    println!("    Evaluations:         {}", vqe.evaluation_count());

    // Show energy convergence
    if result.energy_history.len() > 1 {
        println!("\n  Energy convergence:");
        let show_points = 5.min(result.energy_history.len());
        for i in 0..show_points {
            println!("    Iter {:3}: {:.10} Ha", i, result.energy_history[i]);
        }
        if result.energy_history.len() > show_points {
            println!("    ...");
            let last_idx = result.energy_history.len() - 1;
            println!(
                "    Iter {:3}: {:.10} Ha",
                last_idx, result.energy_history[last_idx]
            );
        }
        println!(
            "    Energy improvement: {:.10} Ha",
            result.energy_improvement()
        );
    }

    println!("\n");

    // Test 2: UCCSD-inspired ansatz
    println!("Test 2: UCCSD-Inspired Ansatz");
    println!("{}", "-".repeat(50));

    let ansatz_uccsd = VQEAnsatz::new(n_qubits, 1);
    println!(
        "  Number of parameters: {}",
        ansatz_uccsd.num_uccsd_parameters()
    );

    let optimizer2 = Box::new(GradientDescentOptimizer::new(200, 0.01, 1e-6));
    let config2 = VQEConfig {
        max_iterations: 150,
        energy_tolerance: 1e-6,
        gradient_tolerance: 1e-5,
        use_gradients: false,
        store_history: true,
        parameter_shift: std::f64::consts::PI / 2.0,
    };

    let mut vqe2 = VQE::new(optimizer2, config2);

    let initial_params2 = vec![0.05; ansatz_uccsd.num_uccsd_parameters()];

    println!("  Running VQE optimization...");
    let start2 = Instant::now();

    let result2 = vqe2.run(
        |params| ansatz_uccsd.build_uccsd_ansatz(params),
        &hamiltonian,
        &initial_params2,
    )?;

    let elapsed2 = start2.elapsed();

    println!("\n  Results:");
    println!(
        "    Ground state energy: {:.10} Ha",
        result2.ground_state_energy
    );
    println!("    Exact energy:        {:.10} Ha", exact_energy);
    println!(
        "    Energy error:        {:.10} Ha ({:.6}%)",
        result2.ground_state_energy - exact_energy,
        ((result2.ground_state_energy - exact_energy) / exact_energy * 100.0).abs()
    );
    println!("    Iterations:          {}", result2.num_iterations);
    println!("    Converged:           {}", result2.converged);
    println!("    Time elapsed:        {:.2?}", elapsed2);
    println!("    Evaluations:         {}", vqe2.evaluation_count());

    println!("\n");

    // Summary
    println!("Summary");
    println!("{}", "=".repeat(50));
    println!("Ansatz             | Energy Error  | Iterations | Time");
    println!("{}", "-".repeat(50));
    println!(
        "Hardware-Efficient | {:.6} mHa | {:10} | {:.2?}",
        (result.ground_state_energy - exact_energy) * 1000.0,
        result.num_iterations,
        elapsed
    );
    println!(
        "UCCSD-Inspired     | {:.6} mHa | {:10} | {:.2?}",
        (result2.ground_state_energy - exact_energy) * 1000.0,
        result2.num_iterations,
        elapsed2
    );

    println!("\nVQE optimization completed successfully!");

    Ok(())
}
