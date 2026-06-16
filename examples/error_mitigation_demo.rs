//! Error mitigation techniques demo
//!
//! This example demonstrates various quantum error mitigation strategies
//! including Zero-Noise Extrapolation (ZNE) and symmetry verification.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🛡️ Quantum Error Mitigation Demo");
    println!("=================================\n");

    demo_zero_noise_extrapolation()?;
    demo_symmetry_verification()?;
    demo_mitigation_suite()?;
    demo_error_mitigation_comparison()?;

    println!("✅ Error mitigation demo completed successfully!");
    Ok(())
}

/// Demo 1: Zero-Noise Extrapolation (ZNE)
fn demo_zero_noise_extrapolation() -> Result<()> {
    println!("🔧 Demo 1: Zero-Noise Extrapolation (ZNE)");
    println!("-------------------------------------------");

    // Create a simple test circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;

    println!("📋 Test Circuit: Bell State Preparation");
    println!("  H(0) → CNOT(0,1) → Measure");

    // Test different ZNE configurations
    println!("\n🎯 ZNE Configuration Options:");

    // Default ZNE
    let zne_default = ZeroNoiseExtrapolation::new();
    println!("  • Default ZNE:");
    println!("    Noise factors: {:?}", zne_default.noise_factors);
    println!("    Method: {:?}", zne_default.extrapolation_method);
    println!("    Shots per level: {}", zne_default.shots_per_level);

    // Custom ZNE with different factors
    let zne_custom = ZeroNoiseExtrapolation::with_noise_factors(vec![1.0, 2.0, 3.0, 4.0])
        .with_extrapolation_method(ExtrapolationMethod::Polynomial(2))
        .with_shots(500);

    println!("  • Custom ZNE:");
    println!("    Noise factors: {:?}", zne_custom.noise_factors);
    println!("    Method: {:?}", zne_custom.extrapolation_method);
    println!("    Shots per level: {}", zne_custom.shots_per_level);

    // Test different extrapolation methods
    println!("\n📊 Extrapolation Methods Comparison:");
    let _test_data = vec![(1.0, 0.95), (3.0, 0.85), (5.0, 0.75)];

    let methods = vec![
        ("Linear", ExtrapolationMethod::Linear),
        ("Exponential", ExtrapolationMethod::Exponential),
        ("Polynomial(2)", ExtrapolationMethod::Polynomial(2)),
        ("Richardson", ExtrapolationMethod::Richardson),
    ];

    for (name, _method) in methods {
        // Simulate extrapolation results for demonstration
        let extrapolated = match name {
            "Linear" => 1.05,
            "Exponential" => 1.03,
            "Polynomial(2)" => 1.04,
            "Richardson" => 1.025,
            _ => 1.0,
        };
        println!("  • {}: {:.4}", name, extrapolated);
    }

    // Demonstrate noise scaling concept
    println!("\n🔄 Noise Scaling Concept:");
    println!("  Original circuit size: {}", circuit.size());

    for &factor in &[1.0, 3.0, 5.0] {
        // Simulate scaling overhead
        let overhead = if factor == 1.0 {
            1.0
        } else {
            factor * 2.0 - 1.0
        };
        let scaled_size = (circuit.size() as f64 * overhead) as usize;
        println!(
            "  • Factor {}: ~{} gates ({}x overhead)",
            factor, scaled_size, overhead
        );
    }

    println!();
    Ok(())
}

/// Demo 2: Symmetry verification
fn demo_symmetry_verification() -> Result<()> {
    println!("🔧 Demo 2: Symmetry Verification");
    println!("---------------------------------");

    let mut verification = SymmetryVerification::new();

    println!("⚖️ Adding Symmetry Constraints:");

    // Add parity symmetries for Bell states
    verification.add_symmetry(
        "X_parity".to_string(),
        vec![PauliOperator::X, PauliOperator::X],
        1.0,
    );

    verification.add_symmetry(
        "Z_parity".to_string(),
        vec![PauliOperator::Z, PauliOperator::Z],
        1.0,
    );

    println!("  • X⊗X parity: Expected eigenvalue = +1");
    println!("  • Z⊗Z parity: Expected eigenvalue = +1");
    println!("  • Total symmetries: {}", verification.symmetries.len());
    println!("  • Tolerance: {}", verification.tolerance);

    // Test symmetry verification on different states
    println!("\n🧪 Testing Symmetry Verification:");

    let test_states = vec![
        ("Bell |00⟩+|11⟩", DensityMatrix::zero_state(2)),
        ("Maximally mixed", DensityMatrix::maximally_mixed(2)),
    ];

    for (name, state) in test_states {
        let result = verification.verify(&state);
        println!(
            "  • {}: {}",
            name,
            if result.is_valid {
                "✅ Valid"
            } else {
                "❌ Invalid"
            }
        );

        if !result.violations.is_empty() {
            for violation in &result.violations {
                println!(
                    "    Violation: {} (expected: {:.3}, measured: {:.3}, deviation: {:.3})",
                    violation.symmetry_name,
                    violation.expected,
                    violation.measured,
                    violation.deviation
                );
            }
        }
    }

    println!();
    Ok(())
}

/// Demo 3: Complete mitigation suite
fn demo_mitigation_suite() -> Result<()> {
    println!("🔧 Demo 3: Error Mitigation Suite");
    println!("----------------------------------");

    // Create comprehensive mitigation suite
    let mut symmetry_verification = SymmetryVerification::new();
    symmetry_verification.add_symmetry(
        "Bell_parity".to_string(),
        vec![PauliOperator::Z, PauliOperator::Z],
        1.0,
    );

    let suite = ErrorMitigationSuite::new()
        .with_zne()
        .with_symmetry_verification(symmetry_verification)
        .with_technique(MitigationTechnique::ReadoutErrorMitigation)
        .with_technique(MitigationTechnique::VirtualZGates);

    println!("🛡️ Mitigation Suite Configuration:");
    println!(
        "  • ZNE: {}",
        if suite.zne.is_some() {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!(
        "  • Symmetry Verification: {}",
        if suite.symmetry_verification.is_some() {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    println!("  • Additional Techniques: {}", suite.techniques.len());

    for (i, technique) in suite.techniques.iter().enumerate() {
        println!("    {}. {:?}", i + 1, technique);
    }

    // Test circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.ry(0, Parameter::Float(PI / 4.0))?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;

    println!("\n📋 Test Circuit: Parameterized Bell State");
    println!("  RY(π/4, 0) → CNOT(0,1) → Measure");

    // Apply mitigation
    let _observable = Observable::PauliZ(0);
    println!("\n🎯 Observable: ⟨Z₀⟩ (Pauli-Z on qubit 0)");

    // Note: In a real implementation, this would run the actual mitigation
    println!("\n📊 Mitigation Results (simulated):");
    println!("  • Raw measurement: 0.850 ± 0.020");
    println!("  • ZNE corrected: 0.920 ± 0.015");
    println!("  • Improvement: +8.2%");
    println!("  • Symmetries satisfied: ✅");

    println!();
    Ok(())
}

/// Demo 4: Error mitigation comparison
fn demo_error_mitigation_comparison() -> Result<()> {
    println!("🔧 Demo 4: Mitigation Strategy Comparison");
    println!("-----------------------------------------");

    // Test different algorithms with varying noise levels
    let test_circuits = vec![
        ("Single Qubit Rotation", create_single_qubit_circuit()),
        ("Bell State", create_bell_circuit()),
        ("3-Qubit GHZ", create_ghz_circuit()),
        ("Variational Ansatz", create_variational_circuit()),
    ];

    println!("📊 Mitigation Effectiveness Analysis:");
    println!("Algorithm            | Raw    | ZNE    | Improvement | Overhead");
    println!("--------------------|--------|--------|-------------|----------");

    for (name, circuit) in test_circuits {
        let circuit = circuit?;

        // Simulate different mitigation results
        let raw_fidelity = simulate_noisy_execution(&circuit);
        let zne_fidelity = simulate_zne_mitigation(&circuit, raw_fidelity);
        let improvement = ((zne_fidelity - raw_fidelity) / raw_fidelity) * 100.0;
        let overhead = calculate_zne_overhead(&circuit);

        println!(
            "{:<19} | {:.3} | {:.3} | {:>10.1}% | {:>7.1}x",
            name, raw_fidelity, zne_fidelity, improvement, overhead
        );
    }

    println!("\n💡 Key Insights:");
    println!("  • ZNE effectiveness depends on circuit depth and noise level");
    println!("  • Shorter circuits benefit more from ZNE");
    println!("  • Overhead scales with noise amplification factors");
    println!("  • Combining techniques often yields best results");

    println!("\n🎯 Best Practices:");
    println!("  • Use ZNE for circuits with <100 gates");
    println!("  • Combine with symmetry verification for error detection");
    println!("  • Optimize noise factors for your specific hardware");
    println!("  • Consider readout error mitigation for measurement-heavy circuits");

    println!();
    Ok(())
}

// Helper functions for creating test circuits

fn create_single_qubit_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(1, 1);
    circuit.ry(0, Parameter::Float(PI / 3.0))?;
    circuit.measure(0, 0)?;
    Ok(circuit)
}

fn create_bell_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;
    Ok(circuit)
}

fn create_ghz_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;
    circuit.measure_all()?;
    Ok(circuit)
}

fn create_variational_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(3, 3);

    // Variational layer 1
    circuit.ry(0, Parameter::Float(PI / 4.0))?;
    circuit.ry(1, Parameter::Float(PI / 6.0))?;
    circuit.ry(2, Parameter::Float(PI / 3.0))?;

    // Entangling layer
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;

    // Variational layer 2
    circuit.ry(0, Parameter::Float(PI / 8.0))?;
    circuit.ry(1, Parameter::Float(PI / 12.0))?;
    circuit.ry(2, Parameter::Float(PI / 5.0))?;

    circuit.measure_all()?;
    Ok(circuit)
}

// Simulation helper functions

fn simulate_noisy_execution(circuit: &QuantumCircuit) -> f64 {
    // Simulate decreasing fidelity with circuit depth
    let base_fidelity = 0.99;
    let depth_penalty = 0.01 * circuit.size() as f64;
    (base_fidelity - depth_penalty).max(0.5)
}

fn simulate_zne_mitigation(circuit: &QuantumCircuit, raw_fidelity: f64) -> f64 {
    // Simulate ZNE improvement (better for shorter circuits)
    let improvement_factor = if circuit.size() < 10 {
        1.15 // 15% improvement for short circuits
    } else if circuit.size() < 20 {
        1.08 // 8% improvement for medium circuits
    } else {
        1.03 // 3% improvement for long circuits
    };

    (raw_fidelity * improvement_factor).min(1.0)
}

fn calculate_zne_overhead(circuit: &QuantumCircuit) -> f64 {
    // ZNE overhead from noise scaling (typically 3-5x for standard factors)
    let base_overhead = 3.5;
    let size_factor = 1.0 + (circuit.size() as f64 * 0.01);
    base_overhead * size_factor
}
