//! # Enhanced Grover's Algorithm Implementation
//!
//! This example demonstrates various aspects of Grover's quantum search algorithm,
//! including amplitude amplification, multiple target search, and optimization.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🔍 Enhanced Grover's Algorithm Demonstrations");
    println!("============================================\n");

    demo_basic_grover()?;
    demo_multiple_targets()?;
    demo_amplitude_amplification()?;
    demo_grover_optimization()?;
    demo_grover_analysis()?;

    println!("🎯 Grover's Algorithm Summary:");
    println!("- Provides quadratic speedup for unstructured search");
    println!("- Optimal number of iterations: π/4 × √(N/M)");
    println!("- Can be generalized to amplitude amplification");
    println!("- Useful for optimization and constraint satisfaction");

    Ok(())
}

/// Demo 1: Basic Grover's Algorithm
fn demo_basic_grover() -> Result<()> {
    println!("🎯 Demo 1: Basic Grover's Algorithm (4-qubit search)");
    println!("---------------------------------------------------\n");

    let num_qubits = 4;
    let target_state = 0b1010; // Target: |1010⟩
    let num_items = 1 << num_qubits; // 2^4 = 16 items

    println!("Search space: {} items ({} qubits)", num_items, num_qubits);
    println!(
        "Target state: |{:04b}⟩ (decimal {})",
        target_state, target_state
    );

    // Calculate optimal number of iterations
    let optimal_iterations = calculate_grover_iterations(num_items, 1);
    println!("Optimal iterations: {}", optimal_iterations);

    // Create Grover circuit
    let circuit = create_grover_circuit(num_qubits, target_state, optimal_iterations)?;

    println!("\n🎨 Grover Circuit Visualization:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    println!("📊 Circuit Analysis:");
    let viz = CircuitVisualizer::new();
    println!("{}", viz.statistics_table(&circuit));

    // Analyze success probability
    println!("🎲 Expected Success Probability:");
    let success_prob = calculate_success_probability(num_items, 1, optimal_iterations);
    println!(
        "After {} iterations: {:.1}%",
        optimal_iterations,
        success_prob * 100.0
    );

    println!();
    Ok(())
}

/// Demo 2: Multiple Target Search
fn demo_multiple_targets() -> Result<()> {
    println!("🎯 Demo 2: Multiple Target Search");
    println!("---------------------------------\n");

    let num_qubits = 3;
    let targets = vec![0b001, 0b101, 0b110]; // Multiple targets
    let num_items = 1 << num_qubits;
    let num_targets = targets.len();

    println!("Search space: {} items", num_items);
    println!(
        "Target states: {:?}",
        targets
            .iter()
            .map(|t| format!("|{:03b}⟩", t))
            .collect::<Vec<_>>()
    );
    println!("Number of targets: {}", num_targets);

    let optimal_iterations = calculate_grover_iterations(num_items, num_targets);
    println!("Optimal iterations: {}", optimal_iterations);

    // Create multi-target Grover circuit
    let circuit = create_multi_target_grover(num_qubits, &targets, optimal_iterations)?;

    println!("\n🎨 Multi-target Grover Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    // Success probability analysis
    let success_prob = calculate_success_probability(num_items, num_targets, optimal_iterations);
    println!("\n🎲 Success Analysis:");
    println!(
        "Probability of finding any target: {:.1}%",
        success_prob * 100.0
    );
    println!(
        "Probability per target: {:.1}%",
        success_prob * 100.0 / num_targets as f64
    );

    println!();
    Ok(())
}

/// Demo 3: Amplitude Amplification
fn demo_amplitude_amplification() -> Result<()> {
    println!("🎯 Demo 3: Amplitude Amplification");
    println!("----------------------------------\n");

    println!("Amplitude amplification generalizes Grover's algorithm");
    println!("to amplify the amplitude of any desired quantum state.\n");

    let num_qubits = 3;

    // Create initial state preparation
    let initial_circuit = create_initial_state_preparation(num_qubits)?;
    println!("🔧 Initial State Preparation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&initial_circuit));

    // Create amplitude amplification circuit
    let aa_circuit = create_amplitude_amplification_circuit(num_qubits, 2)?;
    println!("🔄 Amplitude Amplification (2 iterations):");
    println!("{}", CircuitVisualizer::to_ascii_art(&aa_circuit));

    println!("💡 Key Concepts:");
    println!("- Selective phase inversion of target states");
    println!("- Inversion about average amplitude");
    println!("- Geometric rotation in amplitude space");
    println!("- Applicable to any quantum algorithm with success amplitude < 1");

    println!();
    Ok(())
}

/// Demo 4: Grover Optimization
fn demo_grover_optimization() -> Result<()> {
    println!("🎯 Demo 4: Grover for Optimization Problems");
    println!("-------------------------------------------\n");

    println!("Grover's algorithm can solve optimization problems by");
    println!("searching for states that satisfy certain constraints.\n");

    // Example: Find 3-bit strings with exactly 2 ones (constraint satisfaction)
    let num_qubits = 3;
    let valid_states = vec![0b011, 0b101, 0b110]; // States with exactly 2 ones

    println!("🎯 Constraint: Find 3-bit strings with exactly 2 ones");
    println!(
        "Valid states: {:?}",
        valid_states
            .iter()
            .map(|s| format!("{:03b}", s))
            .collect::<Vec<_>>()
    );

    let optimal_iterations = calculate_grover_iterations(8, valid_states.len());
    let circuit = create_constraint_satisfaction_grover(num_qubits, optimal_iterations)?;

    println!("\n🔧 Constraint Satisfaction Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    println!("🧮 Optimization Applications:");
    println!("- Boolean satisfiability (SAT)");
    println!("- Graph coloring problems");
    println!("- Traveling salesman problem (with constraints)");
    println!("- Portfolio optimization");
    println!("- Machine learning feature selection");

    println!();
    Ok(())
}

/// Demo 5: Grover Algorithm Analysis
fn demo_grover_analysis() -> Result<()> {
    println!("🎯 Demo 5: Grover Algorithm Analysis");
    println!("------------------------------------\n");

    println!("📈 Success Probability vs Iterations:");
    println!("(4-qubit search space, 1 target)\n");

    let num_items = 16;
    let num_targets = 1;

    // Analyze success probability for different iteration counts
    for iterations in 0..=6 {
        let prob = calculate_success_probability(num_items, num_targets, iterations);
        let bar = "█".repeat((prob * 20.0) as usize);
        println!("Iter {}: {:.1}% {}", iterations, prob * 100.0, bar);
    }

    println!("\n🔬 Geometric Interpretation:");
    println!("- Initial state: uniform superposition");
    println!("- Each iteration rotates by θ = 2×arcsin(√(M/N))");
    println!("- Optimal angle: π/2 (maximum amplitude)");
    println!("- Over-rotation decreases success probability");

    println!("\n⚡ Performance Comparison:");
    println!("┌─────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Search Size │ Classical   │ Grover      │ Speedup     │");
    println!("├─────────────┼─────────────┼─────────────┼─────────────┤");

    for &n in &[16, 64, 256, 1024, 4096] {
        let classical = n / 2; // Average case
        let grover = calculate_grover_iterations(n, 1);
        let speedup = classical as f64 / grover as f64;
        println!(
            "│ {:11} │ {:11} │ {:11} │ {:11.1} │",
            n, classical, grover, speedup
        );
    }
    println!("└─────────────┴─────────────┴─────────────┴─────────────┘");

    println!();
    Ok(())
}

/// Create basic Grover circuit
fn create_grover_circuit(
    num_qubits: usize,
    target: usize,
    iterations: usize,
) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // Initial superposition
    for qubit in 0..num_qubits {
        circuit.h(qubit)?;
    }

    // Grover iterations
    for _ in 0..iterations {
        // Oracle: flip phase of target state
        oracle_single_target(&mut circuit, target, num_qubits)?;

        // Diffusion operator (inversion about average)
        diffusion_operator(&mut circuit, num_qubits)?;
    }

    // Measurement
    circuit.measure_all()?;

    Ok(circuit)
}

/// Create multi-target Grover circuit
fn create_multi_target_grover(
    num_qubits: usize,
    targets: &[usize],
    iterations: usize,
) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // Initial superposition
    for qubit in 0..num_qubits {
        circuit.h(qubit)?;
    }

    // Grover iterations
    for _ in 0..iterations {
        // Oracle: flip phase of all target states
        oracle_multiple_targets(&mut circuit, targets, num_qubits)?;

        // Diffusion operator
        diffusion_operator(&mut circuit, num_qubits)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

/// Oracle for single target
fn oracle_single_target(
    circuit: &mut QuantumCircuit,
    target: usize,
    num_qubits: usize,
) -> Result<()> {
    // Flip qubits that should be 0 in the target state
    for i in 0..num_qubits {
        if (target >> i) & 1 == 0 {
            circuit.x(i)?;
        }
    }

    // Multi-controlled Z gate (flip phase if all qubits are 1)
    multi_controlled_z(circuit, num_qubits)?;

    // Flip back
    for i in 0..num_qubits {
        if (target >> i) & 1 == 0 {
            circuit.x(i)?;
        }
    }

    Ok(())
}

/// Oracle for multiple targets
fn oracle_multiple_targets(
    circuit: &mut QuantumCircuit,
    targets: &[usize],
    num_qubits: usize,
) -> Result<()> {
    for &target in targets {
        oracle_single_target(circuit, target, num_qubits)?;
    }
    Ok(())
}

/// Diffusion operator (inversion about average)
fn diffusion_operator(circuit: &mut QuantumCircuit, num_qubits: usize) -> Result<()> {
    // H gates
    for qubit in 0..num_qubits {
        circuit.h(qubit)?;
    }

    // X gates
    for qubit in 0..num_qubits {
        circuit.x(qubit)?;
    }

    // Multi-controlled Z
    multi_controlled_z(circuit, num_qubits)?;

    // X gates
    for qubit in 0..num_qubits {
        circuit.x(qubit)?;
    }

    // H gates
    for qubit in 0..num_qubits {
        circuit.h(qubit)?;
    }

    Ok(())
}

/// Multi-controlled Z gate
fn multi_controlled_z(circuit: &mut QuantumCircuit, num_qubits: usize) -> Result<()> {
    if num_qubits == 1 {
        circuit.z(0)?;
    } else if num_qubits == 2 {
        circuit.cz(0, 1)?;
    } else {
        // For more qubits, we'd need to decompose into basic gates
        // This is a simplified implementation
        for i in 0..num_qubits - 1 {
            circuit.cz(i, i + 1)?;
        }
    }
    Ok(())
}

/// Create initial state preparation for amplitude amplification
fn create_initial_state_preparation(num_qubits: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_qubits, 0);

    // Create a non-uniform superposition
    circuit.h(0)?;
    circuit.ry(1, Parameter::Float(PI / 3.0))?;
    circuit.cx(0, 1)?;

    if num_qubits > 2 {
        circuit.ry(2, Parameter::Float(PI / 6.0))?;
        circuit.cx(1, 2)?;
    }

    Ok(circuit)
}

/// Create amplitude amplification circuit
fn create_amplitude_amplification_circuit(
    num_qubits: usize,
    iterations: usize,
) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // Initial state preparation
    let prep_circuit = create_initial_state_preparation(num_qubits)?;
    // circuit.compose(&prep_circuit, &(0..num_qubits).collect::<Vec<_>>())?;

    // Amplitude amplification iterations
    for _ in 0..iterations {
        // Selective phase inversion (oracle)
        circuit.z(0)?; // Simplified oracle

        // Inversion about initial state
        // This would involve the inverse of state preparation + diffusion + state preparation
        diffusion_operator(&mut circuit, num_qubits)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

/// Create constraint satisfaction Grover circuit
fn create_constraint_satisfaction_grover(
    num_qubits: usize,
    iterations: usize,
) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_qubits, num_qubits);

    // Initial superposition
    for qubit in 0..num_qubits {
        circuit.h(qubit)?;
    }

    // Grover iterations
    for _ in 0..iterations {
        // Oracle for "exactly 2 ones" constraint
        constraint_oracle_exactly_two_ones(&mut circuit, num_qubits)?;

        // Diffusion operator
        diffusion_operator(&mut circuit, num_qubits)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

/// Oracle for "exactly 2 ones" constraint
fn constraint_oracle_exactly_two_ones(
    circuit: &mut QuantumCircuit,
    num_qubits: usize,
) -> Result<()> {
    // This is a simplified implementation
    // Real implementation would need more complex logic circuits

    // For 3 qubits, valid states are: 011, 101, 110
    // We'll implement this as multiple single-target oracles
    let valid_states = vec![0b011, 0b101, 0b110];

    for &state in &valid_states {
        oracle_single_target(circuit, state, num_qubits)?;
    }

    Ok(())
}

/// Calculate optimal number of Grover iterations
fn calculate_grover_iterations(num_items: usize, num_targets: usize) -> usize {
    let ratio = num_targets as f64 / num_items as f64;
    let theta = 2.0 * ratio.sqrt().asin();
    let optimal = (PI / (4.0 * theta)).round() as usize;
    optimal.max(1)
}

/// Calculate success probability after given iterations
fn calculate_success_probability(num_items: usize, num_targets: usize, iterations: usize) -> f64 {
    let ratio = num_targets as f64 / num_items as f64;
    let theta = 2.0 * ratio.sqrt().asin();
    let angle = (2 * iterations + 1) as f64 * theta;
    (angle / 2.0).sin().powi(2)
}
