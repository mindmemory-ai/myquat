//! Symbolic Hamiltonian and Automatic Differentiation Demo
//!
//! Author: gA4ss
//!
//! This example demonstrates the symbolic computation capabilities for Hamiltonians,
//! including parameter optimization, automatic differentiation, and gradient-based methods.

use myquat::error::Result;
use myquat::hamiltonian::{PauliString, SymbolicCompiler, SymbolicHamiltonian};
use myquat::symbolic::default_backend;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=================================================================");
    println!("Phase 5: Symbolic Hamiltonian and Automatic Differentiation Demo");
    println!("=================================================================\n");

    demo_1_symbolic_hamiltonian()?;
    demo_2_parameter_substitution()?;
    demo_3_gradient_computation()?;
    demo_4_vqe_example()?;
    demo_5_symbolic_compilation()?;

    Ok(())
}

fn demo_1_symbolic_hamiltonian() -> Result<()> {
    println!("Demo 1: Creating Symbolic Hamiltonians");
    println!("---------------------------------------");
    println!("We create a parameterized Ising model Hamiltonian:");
    println!("H(J, h) = J * Z₀Z₁ + h * X₀\n");

    let backend = default_backend();
    let mut h = SymbolicHamiltonian::new(2, backend)?;

    // Add ZZ interaction term with parameter J
    let zz = PauliString::from_str("ZZ")?;
    h.add_variable_term(zz, "J")?;

    // Add X field term with parameter h
    let xi = PauliString::from_str("XI")?;
    h.add_variable_term(xi, "h")?;

    println!("Created Hamiltonian:");
    println!("{}", h);
    println!("Parameters: {:?}\n", h.parameters);

    Ok(())
}

fn demo_2_parameter_substitution() -> Result<()> {
    println!("Demo 2: Parameter Substitution");
    println!("-------------------------------");
    println!("Substituting specific values for symbolic parameters\n");

    let backend = default_backend();
    let mut h = SymbolicHamiltonian::new(3, backend)?;

    // Create Heisenberg model: H = J_x XX + J_y YY + J_z ZZ
    h.add_variable_term(PauliString::from_str("XXI")?, "J_x")?;
    h.add_variable_term(PauliString::from_str("YYI")?, "J_y")?;
    h.add_variable_term(PauliString::from_str("ZZI")?, "J_z")?;

    println!("Original symbolic Hamiltonian:");
    println!("  Parameters: {:?}", h.parameters);

    // Substitute values
    let mut values = HashMap::new();
    values.insert("J_x".to_string(), 1.0);
    values.insert("J_y".to_string(), 1.0);
    values.insert("J_z".to_string(), 0.5);

    let h_sub = h.substitute(&values)?;
    println!("\nAfter substitution (J_x=1.0, J_y=1.0, J_z=0.5):");
    println!("  Number of terms: {}", h_sub.num_terms());

    // Evaluate to numerical Hamiltonian
    let h_numerical = h.evaluate(&values)?;
    println!("\nNumerical Hamiltonian:");
    println!("  Terms: {}", h_numerical.num_terms());
    println!("  Hermitian: {}\n", h_numerical.is_hermitian());

    Ok(())
}

fn demo_3_gradient_computation() -> Result<()> {
    println!("Demo 3: Automatic Differentiation");
    println!("----------------------------------");
    println!("Computing gradients for variational algorithms\n");

    let backend = default_backend();
    let mut h = SymbolicHamiltonian::new(2, backend)?;

    // H(J, h) = J * ZZ + h * (X₀ + X₁)
    h.add_variable_term(PauliString::from_str("ZZ")?, "J")?;
    h.add_variable_term(PauliString::from_str("XI")?, "h")?;
    h.add_variable_term(PauliString::from_str("IX")?, "h")?;

    println!("Hamiltonian: H(J, h) = J·ZZ + h·(X₀ + X₁)");
    println!("Parameters: {:?}\n", h.parameters);

    // Compute gradient with respect to J
    println!("Computing ∂H/∂J:");
    let grad_j = h.gradient("J")?;
    println!("  ∂H/∂J has {} terms", grad_j.num_terms());
    println!("  This gives us the sensitivity to the ZZ coupling\n");

    // Compute gradient with respect to h
    println!("Computing ∂H/∂h:");
    let grad_h = h.gradient("h")?;
    println!("  ∂H/∂h has {} terms", grad_h.num_terms());
    println!("  This gives us the sensitivity to the transverse field\n");

    // Compute all gradients
    println!("Computing all gradients:");
    let all_grads = h.gradients()?;
    for (param, grad) in &all_grads {
        println!("  ∂H/∂{}: {} terms", param, grad.num_terms());
    }
    println!();

    Ok(())
}

fn demo_4_vqe_example() -> Result<()> {
    println!("Demo 4: VQE-like Optimization Scenario");
    println!("---------------------------------------");
    println!("Simulating parameter optimization for VQE\n");

    let backend = default_backend();
    let mut h = SymbolicHamiltonian::new(2, backend)?;

    // Problem Hamiltonian
    h.add_variable_term(PauliString::from_str("ZZ")?, "J")?;
    h.add_variable_term(PauliString::from_str("XI")?, "h_x")?;
    h.add_variable_term(PauliString::from_str("IZ")?, "h_z")?;

    println!("Problem Hamiltonian: H = J·ZZ + h_x·X₀ + h_z·Z₁");
    println!("Optimization parameters: {:?}\n", h.parameters);

    // Simulate gradient descent
    println!("Gradient-based optimization workflow:");
    println!("1. Initialize parameters: J=1.0, h_x=0.5, h_z=0.3");

    let mut params = HashMap::new();
    params.insert("J".to_string(), 1.0);
    params.insert("h_x".to_string(), 0.5);
    params.insert("h_z".to_string(), 0.3);

    println!("2. Compute gradients ∂H/∂J, ∂H/∂h_x, ∂H/∂h_z");
    let grads = h.gradients()?;
    println!("   Number of gradient Hamiltonians: {}", grads.len());

    println!("3. Evaluate gradients at current parameters");
    println!("   (In actual VQE: ∂⟨ψ|H|ψ⟩/∂θ using parameter shift rule)");

    println!("4. Update parameters: θ_new = θ_old - learning_rate × gradient");
    println!("5. Repeat until convergence\n");

    println!("Advantages of symbolic approach:");
    println!("  - Automatic gradient computation (no manual derivation)");
    println!("  - Exact symbolic gradients (not numerical approximation)");
    println!("  - Can optimize gradient expressions before evaluation");
    println!("  - Supports complex parameter dependencies\n");

    Ok(())
}

fn demo_5_symbolic_compilation() -> Result<()> {
    println!("Demo 5: Symbolic Circuit Compilation");
    println!("-------------------------------------");
    println!("Compiling symbolic Hamiltonian to parameterized circuit\n");

    let backend = default_backend();
    let mut h = SymbolicHamiltonian::new(2, backend.clone())?;

    // Simple parameterized Hamiltonian
    h.add_variable_term(PauliString::from_str("ZZ")?, "theta")?;

    println!("Hamiltonian: H(θ) = θ · ZZ");
    println!("This will compile to: U(θ,t) = exp(-i·θ·ZZ·t/ℏ)\n");

    let compiler = SymbolicCompiler::new(backend);
    let symbolic_circuit = compiler.compile(&h)?;

    println!("Compiled to parameterized quantum circuit:");
    println!(
        "  Number of qubits: {}",
        symbolic_circuit.circuit.num_qubits()
    );
    println!("  Parameters: {:?}", symbolic_circuit.parameters);
    println!("  Parameter count: {}\n", symbolic_circuit.num_parameters());

    println!("Circuit successfully compiled with symbolic parameters!");

    println!("\nApplications:");
    println!("  - Variational quantum algorithms (VQE, QAOA)");
    println!("  - Quantum machine learning");
    println!("  - Optimal control pulse design");
    println!("  - Parameter sensitivity analysis\n");

    Ok(())
}
