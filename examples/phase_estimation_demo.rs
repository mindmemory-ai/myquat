// Phase Estimation Algorithm Demonstration
// Author: gA4ss
//
// This example demonstrates various use cases of the Quantum Phase Estimation (QPE) algorithm.

use myquat::algorithms::phase_estimation::{ControlledUnitary, EigenstatePreparation};
use myquat::algorithms::PhaseEstimation;
use myquat::error::Result;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("=== Quantum Phase Estimation (QPE) Demonstrations ===\n");

    // Example 1: Basic phase estimation with |1⟩ eigenstate
    example_1_basic_phase_estimation()?;

    // Example 2: Phase estimation with computational basis state
    example_2_computational_basis_eigenstate()?;

    // Example 3: Phase estimation with |+⟩ state
    example_3_plus_state_eigenstate()?;

    // Example 4: Z rotation phase estimation
    example_4_z_rotation_phase()?;

    // Example 5: High precision phase estimation
    example_5_high_precision()?;

    // Example 6: Multiple eigenstate qubits
    example_6_multiple_eigenstates()?;

    println!("\n=== All QPE demonstrations completed successfully! ===");
    Ok(())
}

/// Example 1: Basic phase estimation with |1⟩ eigenstate
///
/// Estimates the phase of U = e^(iφ)|1⟩⟨1| where φ = π/4
fn example_1_basic_phase_estimation() -> Result<()> {
    println!("Example 1: Basic Phase Estimation");
    println!("{}", "=".repeat(50));

    let num_counting = 3;
    let num_eigenstate = 1;
    let phase = PI / 4.0;

    // Create QPE with |1⟩ eigenstate and phase unitary
    let pe = PhaseEstimation::with_unitary(
        num_counting,
        num_eigenstate,
        EigenstatePreparation::OneState,
        ControlledUnitary::Phase(phase),
    );

    println!("Configuration:");
    println!("  Counting qubits: {}", num_counting);
    println!("  Eigenstate qubits: {}", num_eigenstate);
    println!("  True phase: {:.4}π ({:.4} rad)", phase / PI, phase);
    println!(
        "  Phase precision: 1/{} = {:.4}",
        1 << num_counting,
        pe.phase_precision()
    );

    // Build and analyze circuit
    let circuit = pe.build_circuit(None)?;
    println!("\nCircuit statistics:");
    println!("  Total qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Classical bits: {}", circuit.num_clbits());

    // Expected measurement outcome
    let expected_value = (phase / (2.0 * PI) * (1 << num_counting) as f64).round() as usize;
    println!("\nExpected measurement:");
    println!(
        "  Binary: {:0width$b}",
        expected_value,
        width = num_counting
    );
    println!("  Decimal: {}", expected_value);
    println!(
        "  Estimated phase: {:.4}π",
        pe.estimate_phase_from_bitstring(&format!(
            "{:0width$b}",
            expected_value,
            width = num_counting
        ))
    );

    println!("✓ Example 1 completed\n");
    Ok(())
}

/// Example 2: Phase estimation with computational basis state
///
/// Estimates phase using |101⟩ as eigenstate
fn example_2_computational_basis_eigenstate() -> Result<()> {
    println!("Example 2: Computational Basis Eigenstate");
    println!("{}", "=".repeat(50));

    let num_counting = 4;
    let num_eigenstate = 3;
    let eigenstate_value = 0b101; // |101⟩

    let pe = PhaseEstimation::with_eigenstate_prep(
        num_counting,
        num_eigenstate,
        EigenstatePreparation::ComputationalBasis(eigenstate_value),
    );

    println!("Configuration:");
    println!("  Counting qubits: {}", num_counting);
    println!("  Eigenstate qubits: {}", num_eigenstate);
    println!(
        "  Eigenstate: |{:0width$b}⟩",
        eigenstate_value,
        width = num_eigenstate
    );
    println!("  Phase precision: {:.5}", pe.phase_precision());

    let circuit = pe.build_circuit(None)?;
    println!("\nCircuit statistics:");
    println!("  Total qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());

    println!("✓ Example 2 completed\n");
    Ok(())
}

/// Example 3: Phase estimation with |+⟩ state
///
/// Uses equal superposition state as eigenstate
fn example_3_plus_state_eigenstate() -> Result<()> {
    println!("Example 3: |+⟩ State Eigenstate");
    println!("{}", "=".repeat(50));

    let num_counting = 3;
    let num_eigenstate = 2;

    let pe = PhaseEstimation::with_eigenstate_prep(
        num_counting,
        num_eigenstate,
        EigenstatePreparation::PlusState,
    );

    println!("Configuration:");
    println!("  Counting qubits: {}", num_counting);
    println!("  Eigenstate qubits: {}", num_eigenstate);
    println!("  Eigenstate: |+⟩^⊗{}", num_eigenstate);
    println!("  Phase precision: {:.4}", pe.phase_precision());

    let circuit = pe.build_circuit(None)?;
    println!("\nCircuit statistics:");
    println!("  Total qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  (|+⟩ state requires H gates for preparation)");

    println!("✓ Example 3 completed\n");
    Ok(())
}

/// Example 4: Z rotation phase estimation
///
/// Estimates the phase of Rz(θ) rotation
fn example_4_z_rotation_phase() -> Result<()> {
    println!("Example 4: Z Rotation Phase Estimation");
    println!("{}", "=".repeat(50));

    let num_counting = 4;
    let num_eigenstate = 1;
    let rotation_angle = PI / 3.0; // 60 degrees

    let pe = PhaseEstimation::with_unitary(
        num_counting,
        num_eigenstate,
        EigenstatePreparation::OneState,
        ControlledUnitary::ZRotation(rotation_angle),
    );

    println!("Configuration:");
    println!("  Counting qubits: {}", num_counting);
    println!(
        "  Rotation angle: {:.4}π ({:.4} rad)",
        rotation_angle / PI,
        rotation_angle
    );
    println!("  Phase precision: {:.5}", pe.phase_precision());

    let circuit = pe.build_circuit(None)?;
    println!("\nCircuit statistics:");
    println!("  Total qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());

    // For Rz(θ), the phase is θ/(2π)
    let expected_phase = rotation_angle / (2.0 * PI);
    println!("\nExpected phase: {:.4}", expected_phase);

    println!("✓ Example 4 completed\n");
    Ok(())
}

/// Example 5: High precision phase estimation
///
/// Uses more counting qubits for higher precision
fn example_5_high_precision() -> Result<()> {
    println!("Example 5: High Precision Phase Estimation");
    println!("{}", "=".repeat(50));

    let phase = PI / 7.0; // Irrational phase

    println!("True phase: {:.10}π ({:.10} rad)", phase / PI, phase);
    println!("\nPrecision comparison:");

    for num_counting in [3, 5, 8, 10] {
        let pe = PhaseEstimation::with_unitary(
            num_counting,
            1,
            EigenstatePreparation::OneState,
            ControlledUnitary::Phase(phase),
        );

        let precision = pe.phase_precision();
        let circuit = pe.build_circuit(None)?;

        println!("  {} counting qubits:", num_counting);
        println!("    Precision: {:.10} ({:.2e})", precision, precision);
        println!("    Gates: {}", circuit.size());
        println!("    Bits of precision: {:.2}", -(precision.log2()));
    }

    println!("✓ Example 5 completed\n");
    Ok(())
}

/// Example 6: Multiple eigenstate qubits
///
/// Phase estimation with multi-qubit eigenstate
fn example_6_multiple_eigenstates() -> Result<()> {
    println!("Example 6: Multiple Eigenstate Qubits");
    println!("{}", "=".repeat(50));

    let num_counting = 4;
    let num_eigenstate = 4;
    let eigenstate_value = 0b1010; // |1010⟩

    let pe = PhaseEstimation::with_unitary(
        num_counting,
        num_eigenstate,
        EigenstatePreparation::ComputationalBasis(eigenstate_value),
        ControlledUnitary::Phase(PI / 8.0),
    );

    println!("Configuration:");
    println!("  Counting qubits: {}", num_counting);
    println!("  Eigenstate qubits: {}", num_eigenstate);
    println!(
        "  Eigenstate: |{:0width$b}⟩",
        eigenstate_value,
        width = num_eigenstate
    );
    println!("  Total qubits: {}", pe.total_qubits());

    let circuit = pe.build_circuit(None)?;
    println!("\nCircuit statistics:");
    println!("  Total qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Circuit depth (estimated): {}", pe.circuit_depth());

    println!("\nResource requirements:");
    println!("  Quantum memory: {} qubits", circuit.num_qubits());
    println!("  Classical memory: {} bits", circuit.num_clbits());
    println!("  Gate operations: {}", circuit.size());

    println!("✓ Example 6 completed\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_examples() {
        assert!(example_1_basic_phase_estimation().is_ok());
        assert!(example_2_computational_basis_eigenstate().is_ok());
        assert!(example_3_plus_state_eigenstate().is_ok());
        assert!(example_4_z_rotation_phase().is_ok());
        assert!(example_5_high_precision().is_ok());
        assert!(example_6_multiple_eigenstates().is_ok());
    }

    #[test]
    fn test_phase_accuracy() {
        // Test that phase estimation precision improves with more qubits
        let true_phase = PI / 4.0;

        for n in 2..6 {
            let pe = PhaseEstimation::with_unitary(
                n,
                1,
                EigenstatePreparation::OneState,
                ControlledUnitary::Phase(true_phase),
            );

            let precision = pe.phase_precision();
            assert!(precision <= 1.0 / (1 << n) as f64);
        }
    }

    #[test]
    fn test_circuit_scaling() {
        // Test that circuit size scales appropriately
        let base_pe = PhaseEstimation::new(3, 1);
        let base_circuit = base_pe.build_circuit(None).unwrap();
        let base_size = base_circuit.size();

        // More counting qubits should increase circuit size
        let larger_pe = PhaseEstimation::new(5, 1);
        let larger_circuit = larger_pe.build_circuit(None).unwrap();

        assert!(larger_circuit.size() > base_size);
    }
}
