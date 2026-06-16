//! Bell State Example
//!
//! This example demonstrates how to create and analyze Bell states using MyQuat.

use myquat::{
    quantum_info::{QuantumInfo, QuantumState},
    CircuitVisualizer, QuantumCircuit,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bell State Example ===\n");

    // Create a Bell state circuit
    let mut circuit = QuantumCircuit::new_with_name(2, 2, "Bell State".to_string());
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure(0, 0)?;
    circuit.measure(1, 1)?;

    println!("Bell state circuit:");
    println!("{}", CircuitVisualizer::to_text(&circuit));
    println!("ASCII representation:");
    println!("{}\n", CircuitVisualizer::to_ascii_art(&circuit));

    // Create Bell states directly using quantum_info
    println!("Creating Bell states using quantum_info module:");

    for i in 0..4 {
        let bell_state = QuantumInfo::bell_state(i)?;
        let concurrence = QuantumInfo::concurrence(&bell_state)?;

        let name = match i {
            0 => "|Φ+⟩ = (|00⟩ + |11⟩)/√2",
            1 => "|Φ-⟩ = (|00⟩ - |11⟩)/√2",
            2 => "|Ψ+⟩ = (|01⟩ + |10⟩)/√2",
            3 => "|Ψ-⟩ = (|01⟩ - |10⟩)/√2",
            _ => unreachable!(),
        };

        println!("Bell state {}: {}", i, name);
        println!("  Concurrence (entanglement measure): {:.6}", concurrence);

        // Show amplitudes
        println!("  Amplitudes:");
        for j in 0..4 {
            let amp = bell_state.amplitude(j)?;
            if amp.norm() > 1e-10 {
                println!("    |{:02b}⟩: {:.6} + {:.6}i", j, amp.re, amp.im);
            }
        }
        println!();
    }

    // Demonstrate fidelity between Bell states
    println!("Fidelity between different Bell states:");
    let bell_0 = QuantumInfo::bell_state(0)?;
    let bell_1 = QuantumInfo::bell_state(1)?;
    let bell_2 = QuantumInfo::bell_state(2)?;

    println!("  F(|Φ+⟩, |Φ-⟩) = {:.6}", bell_0.fidelity(&bell_1)?);
    println!("  F(|Φ+⟩, |Ψ+⟩) = {:.6}", bell_0.fidelity(&bell_2)?);
    println!("  F(|Φ+⟩, |Φ+⟩) = {:.6}", bell_0.fidelity(&bell_0)?);

    // Demonstrate tensor product
    println!("\nTensor product example:");
    let zero_state = QuantumState::zero_state(1);
    let one_state = QuantumState::computational_basis_state(1, 1)?;

    let product = QuantumInfo::tensor_product(&zero_state, &one_state);
    println!("  |0⟩ ⊗ |1⟩ = |01⟩");
    println!("  Resulting state dimension: {}", product.dim());

    for i in 0..product.dim() {
        let amp = product.amplitude(i)?;
        if amp.norm() > 1e-10 {
            println!("    |{:02b}⟩: {:.6} + {:.6}i", i, amp.re, amp.im);
        }
    }

    Ok(())
}
