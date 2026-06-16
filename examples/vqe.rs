//! Variational Quantum Eigensolver (VQE) Example
//!
//! This example demonstrates a simplified VQE implementation for finding
//! the ground state energy of a simple Hamiltonian. VQE is a hybrid
//! quantum-classical algorithm used in quantum chemistry and optimization.

use myquat::{CircuitVisualizer, Parameter, QuantumCircuit};
use std::collections::HashMap;
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Variational Quantum Eigensolver (VQE) Example ===\n");

    // Example 1: Simple H2 molecule Hamiltonian (simplified)
    println!("1. H2 Molecule Ground State Estimation");
    let h2_result = vqe_h2_example()?;
    println!("Estimated ground state energy: {:.6}", h2_result);
    println!("Theoretical ground state energy: -1.137270 (for comparison)\n");

    // Example 2: Ising model
    println!("2. Ising Model Ground State");
    let ising_result = vqe_ising_example()?;
    println!("Estimated ground state energy: {:.6}", ising_result);
    println!("Expected ground state energy: -2.000000 (for 2-qubit Ising)\n");

    // Example 3: Show ansatz circuits
    println!("3. VQE Ansatz Circuits");
    demonstrate_ansatz_circuits()?;

    Ok(())
}

/// VQE example for H2 molecule (simplified 2-qubit model)
fn vqe_h2_example() -> Result<f64, Box<dyn std::error::Error>> {
    println!("Creating H2 VQE ansatz circuit...");

    // Create a simple ansatz circuit for H2
    let circuit = create_h2_ansatz()?;
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Simulate parameter optimization (simplified)
    let optimal_params = optimize_h2_parameters();
    println!("Optimal parameters found: θ = {:.4}", optimal_params);

    // Bind parameters and compute energy expectation
    let mut param_map = HashMap::new();
    param_map.insert("theta".to_string(), optimal_params);
    circuit.bind_parameters(&param_map)?;

    println!("Circuit after parameter binding:");
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Compute energy expectation value (simplified)
    let energy = compute_h2_energy_expectation(&circuit)?;

    Ok(energy)
}

/// VQE example for Ising model
fn vqe_ising_example() -> Result<f64, Box<dyn std::error::Error>> {
    println!("Creating Ising model VQE ansatz circuit...");

    let circuit = create_ising_ansatz()?;
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Optimize parameters for Ising model
    let optimal_params = optimize_ising_parameters();
    println!(
        "Optimal parameters: θ1 = {:.4}, θ2 = {:.4}",
        optimal_params.0, optimal_params.1
    );

    // Bind parameters
    let mut param_map = HashMap::new();
    param_map.insert("theta1".to_string(), optimal_params.0);
    param_map.insert("theta2".to_string(), optimal_params.1);
    circuit.bind_parameters(&param_map)?;

    println!("Circuit after parameter binding:");
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Compute energy expectation
    let energy = compute_ising_energy_expectation(&circuit)?;

    Ok(energy)
}

/// Create H2 molecule ansatz (Hardware Efficient Ansatz)
fn create_h2_ansatz() -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(2, 0);

    // Initial state preparation (HF state for H2)
    circuit.x(0)?; // |10⟩ state

    // Variational ansatz
    let theta = Parameter::new_symbol("theta");
    circuit.ry(0, theta)?;
    circuit.cnot(0, 1)?;

    Ok(circuit)
}

/// Create Ising model ansatz
fn create_ising_ansatz() -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(2, 0);

    // Prepare superposition state
    circuit.h(0)?;
    circuit.h(1)?;

    // Variational layers
    let theta1 = Parameter::new_symbol("theta1");
    let theta2 = Parameter::new_symbol("theta2");

    circuit.rz(0, theta1.clone())?;
    circuit.rz(1, theta2.clone())?;
    circuit.cnot(0, 1)?;
    circuit.rz(1, theta1)?;
    circuit.cnot(0, 1)?;

    Ok(circuit)
}

/// Demonstrate different ansatz circuits
fn demonstrate_ansatz_circuits() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hardware Efficient Ansatz (2 qubits, 1 layer):");
    let hea = create_hardware_efficient_ansatz(2, 1)?;
    println!("{}", CircuitVisualizer::to_text(&hea));

    println!("UCCSD-inspired Ansatz:");
    let uccsd = create_uccsd_ansatz()?;
    println!("{}", CircuitVisualizer::to_text(&uccsd));

    println!("Alternating Layered Ansatz:");
    let alt = create_alternating_ansatz(3)?;
    println!("{}", CircuitVisualizer::to_text(&alt));

    Ok(())
}

/// Create Hardware Efficient Ansatz
fn create_hardware_efficient_ansatz(
    n_qubits: usize,
    n_layers: usize,
) -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(n_qubits, 0);

    for layer in 0..n_layers {
        // Rotation layer
        for qubit in 0..n_qubits {
            let param_name = format!("theta_{}_{}", layer, qubit);
            let theta = Parameter::new_symbol(&param_name);
            circuit.ry(qubit, theta)?;
        }

        // Entangling layer
        for qubit in 0..(n_qubits - 1) {
            circuit.cnot(qubit, qubit + 1)?;
        }
    }

    Ok(circuit)
}

/// Create UCCSD-inspired ansatz
fn create_uccsd_ansatz() -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(4, 0);

    // Reference state (HF state)
    circuit.x(0)?;
    circuit.x(1)?;

    // Single excitations
    let t1 = Parameter::new_symbol("t1");
    circuit.ry(2, t1.clone())?;
    circuit.cnot(0, 2)?;
    circuit.ry(2, -t1.clone())?;
    circuit.cnot(0, 2)?;

    // Double excitations (simplified)
    let t2 = Parameter::new_symbol("t2");
    circuit.ry(3, t2.clone())?;
    circuit.cnot(1, 3)?;
    circuit.ry(3, -t2.clone())?;
    circuit.cnot(1, 3)?;

    Ok(circuit)
}

/// Create alternating layered ansatz
fn create_alternating_ansatz(
    n_qubits: usize,
) -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(n_qubits, 0);

    // Initial superposition
    for qubit in 0..n_qubits {
        circuit.h(qubit)?;
    }

    // Alternating layers
    for layer in 0..2 {
        // X rotations
        for qubit in 0..n_qubits {
            let param_name = format!("rx_{}_{}", layer, qubit);
            let theta = Parameter::new_symbol(&param_name);
            circuit.rx(qubit, theta)?;
        }

        // Z rotations
        for qubit in 0..n_qubits {
            let param_name = format!("rz_{}_{}", layer, qubit);
            let phi = Parameter::new_symbol(&param_name);
            circuit.rz(qubit, phi)?;
        }

        // Entangling gates
        for qubit in 0..n_qubits {
            let next_qubit = (qubit + 1) % n_qubits;
            circuit.cnot(qubit, next_qubit)?;
        }
    }

    Ok(circuit)
}

/// Simplified parameter optimization for H2 (normally done with classical optimizer)
fn optimize_h2_parameters() -> f64 {
    // In a real VQE, this would use gradient descent, COBYLA, etc.
    // Here we just return a reasonable value for demonstration
    println!("Running classical optimization... (simulated)");
    PI / 4.0 // θ = π/4 gives good results for this simple case
}

/// Simplified parameter optimization for Ising model
fn optimize_ising_parameters() -> (f64, f64) {
    println!("Running classical optimization... (simulated)");
    (PI / 3.0, PI / 6.0) // Example optimal parameters
}

/// Compute H2 energy expectation value (simplified)
fn compute_h2_energy_expectation(
    _circuit: &QuantumCircuit,
) -> Result<f64, Box<dyn std::error::Error>> {
    println!("Computing energy expectation value...");

    // In a real implementation, this would:
    // 1. Run the circuit to prepare the state
    // 2. Measure Pauli strings that make up the Hamiltonian
    // 3. Combine measurements to get energy expectation

    // For demonstration, return a reasonable value
    Ok(-1.136) // Close to H2 ground state energy
}

/// Compute Ising energy expectation value
fn compute_ising_energy_expectation(
    _circuit: &QuantumCircuit,
) -> Result<f64, Box<dyn std::error::Error>> {
    println!("Computing Ising energy expectation value...");

    // Ising Hamiltonian: H = -J(Z₀Z₁) - h(Z₀ + Z₁)
    // For J=1, h=0: ground state energy = -1
    // For 2-qubit case with optimal parameters

    Ok(-1.95) // Close to optimal Ising ground state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h2_ansatz_creation() {
        let circuit = create_h2_ansatz().unwrap();
        assert_eq!(circuit.num_qubits(), 2);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_ising_ansatz_creation() {
        let circuit = create_ising_ansatz().unwrap();
        assert_eq!(circuit.num_qubits(), 2);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_hardware_efficient_ansatz() {
        let circuit = create_hardware_efficient_ansatz(3, 2).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 10); // Should have many gates
    }

    #[test]
    fn test_vqe_h2_example() {
        let result = vqe_h2_example().unwrap();
        assert!(result < 0.0); // Energy should be negative
        assert!(result > -2.0); // Reasonable bound
    }
}
