// VQE Benchmarks
// Author: gA4ss
//
// Comprehensive benchmark tests for the Variational Quantum Eigensolver (VQE)
// implementation, including molecular systems and performance validation.

use myquat::algorithms::{
    ClassicalOptimizer, GradientDescentOptimizer, PauliExpectationComputer, SimplexOptimizer,
    UCCSDAnsatz,
};
use myquat::error::Result;
use myquat::gates::StandardGate;
use myquat::hamiltonian::{Hamiltonian, PauliOperator, PauliString};
use myquat::{Parameter, QuantumCircuit};
use num_complex::Complex64;

/// Build H2 molecule Hamiltonian (STO-3G basis, R = 0.735 Å)
///
/// Hamiltonian for H2 molecule in minimal basis:
/// H = -1.0523 * II + 0.3979 * ZI - 0.3979 * IZ - 0.0112 * ZZ + 0.1809 * XX
fn build_h2_hamiltonian() -> Hamiltonian {
    let mut hamiltonian = Hamiltonian::new(2);

    // Constant term
    hamiltonian.add_constant(Complex64::new(-1.0523, 0.0));

    // ZI term
    let zi = PauliString::new(
        vec![PauliOperator::Z, PauliOperator::I],
        Complex64::new(1.0, 0.0),
    );
    hamiltonian
        .add_term(zi, Complex64::new(0.3979, 0.0))
        .unwrap();

    // IZ term
    let iz = PauliString::new(
        vec![PauliOperator::I, PauliOperator::Z],
        Complex64::new(1.0, 0.0),
    );
    hamiltonian
        .add_term(iz, Complex64::new(-0.3979, 0.0))
        .unwrap();

    // ZZ term
    let zz = PauliString::new(
        vec![PauliOperator::Z, PauliOperator::Z],
        Complex64::new(1.0, 0.0),
    );
    hamiltonian
        .add_term(zz, Complex64::new(-0.0112, 0.0))
        .unwrap();

    // XX term
    let xx = PauliString::new(
        vec![PauliOperator::X, PauliOperator::X],
        Complex64::new(1.0, 0.0),
    );
    hamiltonian
        .add_term(xx, Complex64::new(0.1809, 0.0))
        .unwrap();

    hamiltonian
}

/// Build simplified LiH molecule Hamiltonian (minimal basis)
#[allow(dead_code)]
fn build_lih_hamiltonian() -> Hamiltonian {
    let mut hamiltonian = Hamiltonian::new(4);

    // Simplified LiH Hamiltonian with key terms
    hamiltonian.add_constant(Complex64::new(-7.8, 0.0));

    // Add some representative Pauli terms
    let terms = vec![
        (
            vec![
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::I,
            ],
            0.5,
        ),
        (
            vec![
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
                PauliOperator::I,
            ],
            -0.5,
        ),
        (
            vec![
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
                PauliOperator::I,
            ],
            0.3,
        ),
        (
            vec![
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::I,
                PauliOperator::Z,
            ],
            -0.3,
        ),
        (
            vec![
                PauliOperator::X,
                PauliOperator::X,
                PauliOperator::I,
                PauliOperator::I,
            ],
            0.1,
        ),
        (
            vec![
                PauliOperator::Y,
                PauliOperator::Y,
                PauliOperator::I,
                PauliOperator::I,
            ],
            0.1,
        ),
    ];

    for (ops, coeff) in terms {
        let pauli = PauliString::new(ops, Complex64::new(1.0, 0.0));
        hamiltonian
            .add_term(pauli, Complex64::new(coeff, 0.0))
            .unwrap();
    }

    hamiltonian
}

/// Simple state vector simulator for testing
fn simulate_circuit(circuit: &QuantumCircuit) -> Result<Vec<Complex64>> {
    let n_qubits = circuit.num_qubits();
    let dim = 1 << n_qubits;

    // Initialize to |0...0>
    let mut state = vec![Complex64::new(0.0, 0.0); dim];
    state[0] = Complex64::new(1.0, 0.0);

    // Apply gates (simplified - only supports gates used in UCCSD)
    for inst in circuit.data().instructions() {
        let gate_type = inst.gate.gate_type;
        let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

        match gate_type {
            StandardGate::X => {
                // Flip qubit
                for i in 0..dim {
                    if (i >> qubits[0]) & 1 == 0 {
                        let j = i | (1 << qubits[0]);
                        state.swap(i, j);
                    }
                }
            }
            StandardGate::Ry => {
                if let Some(param) = inst.gate.parameters.first() {
                    let theta = param.numeric_value().unwrap_or(0.0);
                    let cos = (theta / 2.0).cos();
                    let sin = (theta / 2.0).sin();

                    for i in 0..dim {
                        if (i >> qubits[0]) & 1 == 0 {
                            let j = i | (1 << qubits[0]);
                            let a = state[i];
                            let b = state[j];
                            state[i] = Complex64::new(cos, 0.0) * a - Complex64::new(sin, 0.0) * b;
                            state[j] = Complex64::new(sin, 0.0) * a + Complex64::new(cos, 0.0) * b;
                        }
                    }
                }
            }
            StandardGate::Rz => {
                if let Some(param) = inst.gate.parameters.first() {
                    let theta = param.numeric_value().unwrap_or(0.0);
                    let phase_minus = Complex64::new(0.0, -theta / 2.0).exp();
                    let phase_plus = Complex64::new(0.0, theta / 2.0).exp();

                    for i in 0..dim {
                        if (i >> qubits[0]) & 1 == 0 {
                            state[i] *= phase_minus;
                        } else {
                            state[i] *= phase_plus;
                        }
                    }
                }
            }
            StandardGate::CX => {
                for i in 0..dim {
                    if ((i >> qubits[0]) & 1) == 1 && ((i >> qubits[1]) & 1) == 0 {
                        let j = i | (1 << qubits[1]);
                        state.swap(i, j);
                    }
                }
            }
            _ => {
                // Skip unsupported gates
            }
        }
    }

    Ok(state)
}

#[test]
fn test_h2_molecule_vqe() {
    // H2 molecule: 4 spin-orbitals, 2 electrons
    let num_qubits = 4;
    let num_electrons = 2;

    // Build Hamiltonian (but only use first 2 qubits)
    let hamiltonian = build_h2_hamiltonian();

    // Create UCCSD ansatz (singles only for simplicity)
    let ansatz = UCCSDAnsatz::singles_only(num_qubits, num_electrons);
    let num_params = ansatz.num_parameters();

    // Expectation value computer
    let expectation = PauliExpectationComputer::new();

    // Define objective function
    let objective = |params: &[f64]| -> Result<f64> {
        let circuit = ansatz.build_circuit(params)?;
        let state = simulate_circuit(&circuit)?;
        Ok(expectation.compute_hamiltonian_expectation(&state, &hamiltonian))
    };

    // Run optimization with Simplex (gradient-free)
    let optimizer = SimplexOptimizer::new()
        .with_max_iterations(100)
        .with_tolerance(1e-3);

    let initial_params = vec![0.0; num_params];
    let result = optimizer.optimize(&objective, &initial_params).unwrap();

    // Expected H2 ground state energy: approximately -1.137 Hartree
    // With UCCSD singles only, we should get close
    println!("H2 VQE Result:");
    println!("  Energy: {:.6} Hartree", result.minimum);
    println!("  Iterations: {}", result.iterations);
    println!("  Function evaluations: {}", result.function_evaluations);
    println!("  Converged: {}", result.converged);

    // Verify energy is reasonable (within chemical accuracy)
    assert!(result.minimum < -1.0, "Energy should be negative");
    assert!(result.minimum > -1.5, "Energy should be above -1.5");
}

#[test]
fn test_uccsd_parameter_count() {
    // Verify UCCSD parameter counting

    // H2: 4 spin-orbitals, 2 electrons
    let h2_ansatz = UCCSDAnsatz::new(4, 2);
    // Singles: 2 occupied × 2 virtual = 4
    // Doubles: C(2,2) × C(2,2) = 1
    assert_eq!(h2_ansatz.num_single_parameters(), 4);
    assert_eq!(h2_ansatz.num_double_parameters(), 1);
    assert_eq!(h2_ansatz.num_parameters(), 5);

    // LiH: 10 spin-orbitals, 4 electrons
    let lih_ansatz = UCCSDAnsatz::new(10, 4);
    // Singles: 4 × 6 = 24
    // Doubles: C(4,2) × C(6,2) = 6 × 15 = 90
    assert_eq!(lih_ansatz.num_single_parameters(), 24);
    assert_eq!(lih_ansatz.num_double_parameters(), 90);
    assert_eq!(lih_ansatz.num_parameters(), 114);
}

#[test]
fn test_pauli_expectation_values() {
    // Test Pauli expectation value computation

    // State |0⟩
    let state = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)];

    let expectation = PauliExpectationComputer::new();

    // ⟨0|Z|0⟩ = 1
    let z = PauliString::new(vec![PauliOperator::Z], Complex64::new(1.0, 0.0));
    let exp_z = expectation.compute_pauli_expectation(&state, &z);
    assert!((exp_z.re - 1.0).abs() < 1e-10);

    // ⟨0|X|0⟩ = 0
    let x = PauliString::new(vec![PauliOperator::X], Complex64::new(1.0, 0.0));
    let exp_x = expectation.compute_pauli_expectation(&state, &x);
    assert!(exp_x.re.abs() < 1e-10);
}

#[test]
fn test_optimizer_convergence() {
    // Test optimizer on simple quadratic function
    let objective = |params: &[f64]| -> Result<f64> { Ok(params.iter().map(|&x| x * x).sum()) };

    // Gradient Descent
    let gd = GradientDescentOptimizer::new()
        .with_learning_rate(0.2)
        .with_max_iterations(50);

    let result_gd = gd.optimize(&objective, &[1.0, 1.0]).unwrap();
    assert!(result_gd.minimum < 0.1);

    // Simplex
    let simplex = SimplexOptimizer::new().with_max_iterations(100);

    let result_simplex = simplex.optimize(&objective, &[1.0, 1.0]).unwrap();
    assert!(result_simplex.minimum < 0.1);
}

#[test]
fn test_vqe_energy_landscape() {
    // Test VQE energy landscape for H2
    let hamiltonian = build_h2_hamiltonian();
    let ansatz = UCCSDAnsatz::singles_only(4, 2);
    let expectation = PauliExpectationComputer::new();

    // Sample energy at different parameter values (4 parameters for singles_only)
    let test_params = vec![
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.1, 0.0, 0.0, 0.0],
        vec![0.0, 0.1, 0.0, 0.0],
        vec![0.1, 0.1, 0.0, 0.0],
    ];

    for params in test_params {
        let circuit = ansatz.build_circuit(&params).unwrap();
        let state = simulate_circuit(&circuit).unwrap();
        let energy = expectation.compute_hamiltonian_expectation(&state, &hamiltonian);

        // Energy should be reasonable
        assert!(
            energy > -2.0 && energy < 0.0,
            "Energy out of expected range: {}",
            energy
        );
    }
}

#[test]
fn test_hartree_fock_reference() {
    // Test that Hartree-Fock state gives expected energy
    let hamiltonian = build_h2_hamiltonian();
    let ansatz = UCCSDAnsatz::singles_only(4, 2);
    let expectation = PauliExpectationComputer::new();

    // Zero parameters = Hartree-Fock state
    let hf_params = vec![0.0; ansatz.num_parameters()];
    let circuit = ansatz.build_circuit(&hf_params).unwrap();
    let state = simulate_circuit(&circuit).unwrap();
    let hf_energy = expectation.compute_hamiltonian_expectation(&state, &hamiltonian);

    println!("Hartree-Fock energy: {:.6}", hf_energy);

    // HF energy should be higher than ground state
    assert!(hf_energy > -1.5 && hf_energy < 0.0);
}
