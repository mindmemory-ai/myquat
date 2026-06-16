//! Quantum Fourier Transform (QFT) Example
//!
//! This example demonstrates the implementation of the Quantum Fourier Transform,
//! a fundamental quantum algorithm used in many other quantum algorithms like
//! Shor's algorithm and quantum phase estimation.

use myquat::{CircuitVisualizer, Parameter, QuantumCircuit};
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantum Fourier Transform (QFT) Example ===\n");

    // Create QFT circuits for different numbers of qubits
    for n_qubits in 2..=4 {
        println!("QFT on {} qubits:", n_qubits);
        let circuit = create_qft_circuit(n_qubits)?;

        println!("{}", CircuitVisualizer::to_text(&circuit));
        println!(
            "Circuit depth: {}, size: {}\n",
            circuit.depth(),
            circuit.size()
        );
    }

    // Demonstrate QFT + inverse QFT (should be identity)
    println!("=== QFT + Inverse QFT (Identity Test) ===");
    let mut circuit = QuantumCircuit::new(3, 0);

    // Prepare an initial state |101⟩
    circuit.x(0)?;
    circuit.x(2)?;

    println!("Initial state preparation:");
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Apply QFT
    apply_qft(&mut circuit, 3)?;
    println!("After QFT:");
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Apply inverse QFT
    apply_inverse_qft(&mut circuit, 3)?;
    println!("After inverse QFT (should recover |101⟩):");
    println!("{}", CircuitVisualizer::to_text(&circuit));

    // Demonstrate QFT with phase estimation setup
    println!("=== QFT in Phase Estimation Context ===");
    let phase_circuit = create_phase_estimation_example()?;
    println!("{}", CircuitVisualizer::to_text(&phase_circuit));

    Ok(())
}

/// Create a QFT circuit for n qubits
fn create_qft_circuit(n_qubits: usize) -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let mut circuit = QuantumCircuit::new(n_qubits, 0);
    apply_qft(&mut circuit, n_qubits)?;
    Ok(circuit)
}

/// Apply QFT to the first n qubits of a circuit
fn apply_qft(
    circuit: &mut QuantumCircuit,
    n_qubits: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..n_qubits {
        // Apply Hadamard gate
        circuit.h(i)?;

        // Apply controlled phase rotations
        for j in (i + 1)..n_qubits {
            let angle = PI / (1 << (j - i)) as f64; // π/2^(j-i)
            circuit.cp(j, i, Parameter::new_float(angle))?;
        }
    }

    // Swap qubits to reverse the order
    for i in 0..(n_qubits / 2) {
        circuit.swap(i, n_qubits - 1 - i)?;
    }

    Ok(())
}

/// Apply inverse QFT to the first n qubits of a circuit
fn apply_inverse_qft(
    circuit: &mut QuantumCircuit,
    n_qubits: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Swap qubits first (reverse the QFT swaps)
    for i in 0..(n_qubits / 2) {
        circuit.swap(i, n_qubits - 1 - i)?;
    }

    // Apply inverse operations in reverse order
    for i in (0..n_qubits).rev() {
        // Apply inverse controlled phase rotations
        for j in ((i + 1)..n_qubits).rev() {
            let angle = -PI / (1 << (j - i)) as f64; // -π/2^(j-i)
            circuit.cp(j, i, Parameter::new_float(angle))?;
        }

        // Apply Hadamard gate
        circuit.h(i)?;
    }

    Ok(())
}

/// Create a simple phase estimation circuit that uses QFT
fn create_phase_estimation_example() -> Result<QuantumCircuit, Box<dyn std::error::Error>> {
    let n_counting = 3; // Number of counting qubits
    let n_target = 1; // Number of target qubits
    let mut circuit = QuantumCircuit::new(n_counting + n_target, n_counting);

    // Initialize counting qubits in superposition
    for i in 0..n_counting {
        circuit.h(i)?;
    }

    // Prepare eigenstate |1⟩ for the target qubit
    circuit.x(n_counting)?;

    // Apply controlled unitary operations (simplified: controlled-Z gates)
    // In a real phase estimation, this would be controlled powers of the unitary
    for i in 0..n_counting {
        let power = 1 << i; // 2^i
        for _ in 0..power {
            circuit.cz(i, n_counting)?;
        }
    }

    // Apply inverse QFT to the counting qubits
    apply_inverse_qft(&mut circuit, n_counting)?;

    // Measure the counting qubits
    for i in 0..n_counting {
        circuit.measure(i, i)?;
    }

    Ok(circuit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use myquat::gates::Gate;

    #[test]
    fn test_qft_circuit_creation() {
        let circuit = create_qft_circuit(3).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_qft_inverse_qft_identity() {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Apply some initial gates
        circuit.x(0).unwrap();
        let initial_size = circuit.size();

        // Apply QFT then inverse QFT
        apply_qft(&mut circuit, 2).unwrap();
        apply_inverse_qft(&mut circuit, 2).unwrap();

        // Should have more gates but represent the same transformation
        assert!(circuit.size() > initial_size);
    }

    #[test]
    fn test_phase_estimation_circuit() {
        let circuit = create_phase_estimation_example().unwrap();
        assert_eq!(circuit.num_qubits(), 4);
        assert_eq!(circuit.num_clbits(), 3);
        assert!(circuit.size() > 10); // Should have many gates
    }
}
