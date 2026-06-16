// Deoptimization Strategies Demonstration
//
// Author: gA4ss
//
// This example demonstrates the complete deoptimization pipeline for
// restoring original circuit structure from optimized quantum circuits.
// It showcases all three strategies: KAK restoration, template matching,
// and temporal analysis.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::{
    DeoptStrategy, // Trait needed for confidence() method
    DeoptimizationPipeline,
    KakRestorationStrategy,
    TemplateMatchingStrategy,
    TemporalAnalysisStrategy,
};
use myquat::parameter::Parameter;
use std::f64::consts::PI;

fn print_separator() {
    println!("\n{}", "=".repeat(70));
}

fn print_circuit_info(name: &str, circuit: &QuantumCircuit) {
    println!("\n{} Circuit:", name);
    println!("  Qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Depth: {}", circuit.depth());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Deoptimization Strategies Demonstration");
    println!("========================================\n");
    println!("This demo shows how to restore original circuit structures");
    println!("from optimized quantum circuits using three strategies:\n");
    println!("1. KAK Restoration - Recognizes Pauli rotation patterns");
    println!("2. Template Matching - Identifies Trotter decomposition");
    println!("3. Temporal Analysis - Extracts time evolution parameters");

    // Example 1: Pauli Rotation Circuit (KAK Strategy)
    print_separator();
    println!("Example 1: Pauli Rotation Circuit (KAK Restoration)");
    print_separator();

    println!("\nCreating a circuit with Pauli rotations...");
    let mut pauli_circuit = QuantumCircuit::new(2, 0);

    // Add Pauli rotation sequence
    pauli_circuit.rx(0, Parameter::Float(PI / 4.0))?;
    pauli_circuit.ry(1, Parameter::Float(PI / 3.0))?;
    pauli_circuit.rz(0, Parameter::Float(PI / 6.0))?;
    pauli_circuit.cx(0, 1)?;

    print_circuit_info("Original", &pauli_circuit);

    // Apply KAK strategy
    println!("\nApplying KAK Restoration Strategy...");
    let kak_strategy = KakRestorationStrategy::new();
    let kak_confidence = kak_strategy.confidence(&pauli_circuit);
    println!("  Confidence: {:.2}%", kak_confidence * 100.0);

    if kak_confidence > 0.0 {
        let restored = kak_strategy.apply(&pauli_circuit)?;
        print_circuit_info("Restored", &restored);
        println!("\n  KAK Strategy recognized Pauli rotation patterns!");
    }

    // Example 2: Hamiltonian Evolution (Template Matching)
    print_separator();
    println!("Example 2: Hamiltonian Evolution (Template Matching)");
    print_separator();

    println!("\nCreating a Trotter-decomposed Hamiltonian circuit...");
    let mut trotter_circuit = QuantumCircuit::new(3, 0);

    // Trotter step with ZZ interactions
    for i in 0..2 {
        trotter_circuit.h(i)?;
        trotter_circuit.cx(i, i + 1)?;
        trotter_circuit.rz(i + 1, Parameter::Float(0.1))?;
        trotter_circuit.cx(i, i + 1)?;
        trotter_circuit.h(i)?;
    }

    // XX interactions
    for i in 0..2 {
        trotter_circuit.ry(i, Parameter::Float(PI / 2.0))?;
        trotter_circuit.ry(i + 1, Parameter::Float(PI / 2.0))?;
        trotter_circuit.cx(i, i + 1)?;
        trotter_circuit.rz(i + 1, Parameter::Float(0.2))?;
        trotter_circuit.cx(i, i + 1)?;
        trotter_circuit.ry(i, Parameter::Float(-PI / 2.0))?;
        trotter_circuit.ry(i + 1, Parameter::Float(-PI / 2.0))?;
    }

    print_circuit_info("Original", &trotter_circuit);

    // Apply Template Matching strategy
    println!("\nApplying Template Matching Strategy...");
    let template_strategy = TemplateMatchingStrategy::default();
    let template_confidence = template_strategy.confidence(&trotter_circuit);
    println!("  Confidence: {:.2}%", template_confidence * 100.0);

    if template_confidence > 0.0 {
        let restored = template_strategy.apply(&trotter_circuit)?;
        print_circuit_info("Restored", &restored);
        println!("\n  Template Matching identified Trotter structure!");
    }

    // Example 3: Time Evolution (Temporal Analysis)
    print_separator();
    println!("Example 3: Time Evolution (Temporal Analysis)");
    print_separator();

    println!("\nCreating a time evolution circuit with Suzuki decomposition...");
    let mut evolution_circuit = QuantumCircuit::new(2, 0);

    // Add rotation gates with specific angles (Suzuki pattern)
    let suzuki_angle = 0.41449077179437573; // Order-4 Suzuki coefficient
    evolution_circuit.rz(0, Parameter::Float(suzuki_angle))?;
    evolution_circuit.rz(1, Parameter::Float(suzuki_angle))?;
    evolution_circuit.cx(0, 1)?;
    evolution_circuit.rz(1, Parameter::Float(suzuki_angle * 2.0))?;
    evolution_circuit.cx(0, 1)?;

    print_circuit_info("Original", &evolution_circuit);

    // Apply Temporal Analysis strategy
    println!("\nApplying Temporal Analysis Strategy...");
    let temporal_strategy = TemporalAnalysisStrategy::new();
    let temporal_confidence = temporal_strategy.confidence(&evolution_circuit);
    println!("  Confidence: {:.2}%", temporal_confidence * 100.0);

    if temporal_confidence > 0.0 {
        let restored = temporal_strategy.apply(&evolution_circuit)?;
        print_circuit_info("Restored", &restored);
        println!("\n  Temporal Analysis extracted evolution parameters!");
    }

    // Example 4: Complete Pipeline
    print_separator();
    println!("Example 4: Complete Deoptimization Pipeline");
    print_separator();

    println!("\nCreating a mixed circuit (multiple patterns)...");
    let mut mixed_circuit = QuantumCircuit::new(3, 0);

    // Add various patterns
    mixed_circuit.h(0)?;
    mixed_circuit.cx(0, 1)?;
    mixed_circuit.rz(1, Parameter::Float(0.1))?;
    mixed_circuit.ry(2, Parameter::Float(PI / 4.0))?;
    mixed_circuit.cx(1, 2)?;

    print_circuit_info("Original", &mixed_circuit);

    // Use default pipeline (all three strategies)
    println!("\nApplying complete pipeline (KAK + Template + Temporal)...");
    let pipeline = DeoptimizationPipeline::default();

    // Analyze confidence scores
    println!("\nStrategy confidence scores:");
    let analysis = pipeline.analyze(&mixed_circuit);
    for (name, confidence) in &analysis {
        println!("  {}: {:.2}%", name, confidence * 100.0);
    }

    // Perform restoration
    println!("\nPerforming restoration...");
    let result = pipeline.restore_detailed(&mixed_circuit)?;

    println!("\nRestoration Results:");
    println!("  Overall confidence: {:.2}%", result.confidence * 100.0);
    println!("  Strategies applied: {}", result.strategies_applied);
    println!("  Early stopped: {}", result.early_stopped);

    print_circuit_info("Restored", &result.circuit);

    println!("\nPer-strategy confidences:");
    for (name, conf) in result.strategy_confidences {
        println!("  {}: {:.2}%", name, conf * 100.0);
    }

    // Example 5: Custom Pipeline Configuration
    print_separator();
    println!("Example 5: Custom Pipeline Configuration");
    print_separator();

    println!("\nCreating custom pipeline with high threshold...");
    let custom_pipeline = DeoptimizationPipeline::new()
        .add_strategy(Box::new(KakRestorationStrategy::new()))
        .add_strategy(Box::new(TemplateMatchingStrategy::default()))
        .with_threshold(0.90) // High confidence threshold
        .with_early_stop(true)
        .with_max_iterations(5);

    println!("Configuration:");
    println!("  Strategies: {}", custom_pipeline.num_strategies());
    println!("  Threshold: 0.90");
    println!("  Early stop: enabled");
    println!("  Max iterations: 5");

    let custom_result = custom_pipeline.restore_detailed(&mixed_circuit)?;

    println!("\nCustom Pipeline Results:");
    println!("  Confidence: {:.2}%", custom_result.confidence * 100.0);
    println!("  Strategies applied: {}", custom_result.strategies_applied);
    println!("  Early stopped: {}", custom_result.early_stopped);

    // Example 6: Comparison of Strategies
    print_separator();
    println!("Example 6: Strategy Comparison");
    print_separator();

    let test_circuit = QuantumCircuit::new(2, 0);

    println!("\nComparing all strategies on empty circuit:");
    println!("\n{:<25} {:>15}", "Strategy", "Confidence");
    println!("{}", "-".repeat(42));

    let strategies: Vec<(&str, Box<dyn Fn(&QuantumCircuit) -> f64>)> = vec![
        (
            "KAK Restoration",
            Box::new(|c| KakRestorationStrategy::new().confidence(c)),
        ),
        (
            "Template Matching",
            Box::new(|c| TemplateMatchingStrategy::default().confidence(c)),
        ),
        (
            "Temporal Analysis",
            Box::new(|c| TemporalAnalysisStrategy::new().confidence(c)),
        ),
    ];

    for (name, strategy) in strategies {
        let conf = strategy(&test_circuit);
        println!("{:<25} {:>14.2}%", name, conf * 100.0);
    }

    // Summary
    print_separator();
    println!("Summary");
    print_separator();

    println!("\nDeoptimization Pipeline Features:");
    println!("  1. Multiple strategy support (KAK, Template, Temporal)");
    println!("  2. Confidence-based decision making");
    println!("  3. Early stopping for efficiency");
    println!("  4. Detailed diagnostic information");
    println!("  5. Flexible configuration options");

    println!("\nUse Cases:");
    println!("  - Hamiltonian extraction from optimized circuits");
    println!("  - Reverse engineering quantum algorithms");
    println!("  - Understanding compiler transformations");
    println!("  - Debugging quantum circuits");
    println!("  - Circuit analysis and verification");

    println!("\nBest Practices:");
    println!("  1. Start with default pipeline for general cases");
    println!("  2. Use analyze() to inspect confidence scores");
    println!("  3. Adjust threshold based on accuracy requirements");
    println!("  4. Enable early stopping for performance");
    println!("  5. Use detailed results for diagnostics");

    print_separator();
    println!("\nDemo complete! See deoptimization module docs for more details.");

    Ok(())
}
