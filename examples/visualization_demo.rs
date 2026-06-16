//! Advanced circuit visualization demo
//!
//! This example demonstrates the enhanced ASCII art circuit visualization
//! capabilities with different styles and formatting options.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🎨 Advanced Circuit Visualization Demo");
    println!("======================================\n");

    demo_basic_visualization()?;
    demo_visualization_styles()?;
    demo_circuit_statistics()?;
    demo_complex_circuits()?;
    demo_comparison_table()?;

    println!("✅ Visualization demo completed successfully!");
    Ok(())
}

/// Demo 1: Basic visualization features
fn demo_basic_visualization() -> Result<()> {
    println!("🔧 Demo 1: Basic Visualization Features");
    println!("----------------------------------------");

    // Create a simple Bell state circuit
    let mut bell_circuit = QuantumCircuit::new(2, 2);
    bell_circuit.h(0)?;
    bell_circuit.cx(0, 1)?;
    bell_circuit.measure_all()?;

    println!("📋 Bell State Circuit:");

    // Default visualizer
    let viz = CircuitVisualizer::new();
    println!("\n🎨 Enhanced ASCII Art:");
    println!("{}", viz.draw_circuit(&bell_circuit));

    // Circuit summary
    println!("📊 Circuit Summary:");
    println!("{}", viz.circuit_summary(&bell_circuit));

    // Gate breakdown
    println!("🔍 Gate-by-Gate Breakdown:");
    println!("{}", viz.gate_breakdown(&bell_circuit));

    Ok(())
}

/// Demo 2: Different visualization styles
fn demo_visualization_styles() -> Result<()> {
    println!("🔧 Demo 2: Visualization Styles");
    println!("--------------------------------");

    // Create a test circuit with various gates
    let mut test_circuit = QuantumCircuit::new(3, 3);
    test_circuit.h(0)?;
    test_circuit.x(1)?;
    test_circuit.ry(2, Parameter::Float(PI / 4.0))?;
    test_circuit.cx(0, 1)?;
    test_circuit.cz(1, 2)?;
    test_circuit.measure_all()?;

    println!("📋 Test Circuit: H(0), X(1), RY(π/4, 2), CNOT(0,1), CZ(1,2), Measure All");

    // Different styles
    let styles = vec![
        ("Default", CircuitVisualizer::new()),
        ("Compact", CircuitVisualizer::compact()),
        ("Detailed", CircuitVisualizer::detailed()),
    ];

    for (name, viz) in styles {
        println!("\n🎨 {} Style:", name);
        println!("{}", viz.draw_circuit(&test_circuit));
    }

    // Custom style with different wire styles
    println!("🎨 Custom Wire Styles:");

    let wire_styles = vec![
        ("Simple", WireStyle::Simple),
        ("Double", WireStyle::Double),
        ("Dotted", WireStyle::Dotted),
        ("Custom", WireStyle::Custom('=')),
    ];

    for (name, wire_style) in wire_styles {
        let custom_style = VisualizationStyle {
            wire_style,
            gate_style: GateStyle::Compact,
            color_scheme: ColorScheme::None,
            use_unicode: true,
        };

        let viz = CircuitVisualizer::new()
            .with_style(custom_style)
            .with_parameters(false);

        println!("\n  {} wires:", name);
        println!("{}", viz.draw_circuit(&test_circuit));
    }

    Ok(())
}

/// Demo 3: Circuit statistics and analysis
fn demo_circuit_statistics() -> Result<()> {
    println!("🔧 Demo 3: Circuit Statistics and Analysis");
    println!("-------------------------------------------");

    // Create a more complex circuit
    let mut complex_circuit = QuantumCircuit::new(4, 4);

    // Layer 1: Initialization
    complex_circuit.h(0)?;
    complex_circuit.h(1)?;
    complex_circuit.x(2)?;
    complex_circuit.ry(3, Parameter::Float(PI / 3.0))?;

    // Layer 2: Entanglement
    complex_circuit.cx(0, 1)?;
    complex_circuit.cx(1, 2)?;
    complex_circuit.cx(2, 3)?;

    // Layer 3: Rotations
    complex_circuit.rz(0, Parameter::Float(PI / 6.0))?;
    complex_circuit.rx(1, Parameter::Float(PI / 4.0))?;
    complex_circuit.ry(2, Parameter::Float(PI / 8.0))?;

    // Layer 4: More entanglement
    complex_circuit.cz(0, 2)?;
    complex_circuit.cy(1, 3)?;

    // Measurements
    complex_circuit.measure_all()?;

    let viz = CircuitVisualizer::detailed();

    println!("📋 Complex 4-Qubit Circuit:");
    println!("{}", viz.draw_circuit(&complex_circuit));

    println!("📊 Detailed Statistics:");
    println!("{}", viz.statistics_table(&complex_circuit));

    println!("🔍 Complete Analysis:");
    println!("{}", viz.circuit_summary(&complex_circuit));

    Ok(())
}

/// Demo 4: Complex quantum algorithms
fn demo_complex_circuits() -> Result<()> {
    println!("🔧 Demo 4: Complex Quantum Algorithm Circuits");
    println!("----------------------------------------------");

    // Quantum Fourier Transform
    println!("🌊 Quantum Fourier Transform (3-qubit):");
    let qft_circuit = create_qft_circuit(3)?;
    let viz = CircuitVisualizer::new().with_max_width(100);
    println!("{}", viz.draw_circuit(&qft_circuit));
    println!("{}", viz.statistics_table(&qft_circuit));

    // Variational Quantum Eigensolver ansatz
    println!("\n🧬 VQE Ansatz Circuit:");
    let vqe_circuit = create_vqe_ansatz(3)?;
    println!("{}", viz.draw_circuit(&vqe_circuit));
    println!("{}", viz.statistics_table(&vqe_circuit));

    // Grover's algorithm
    println!("\n🔍 Grover's Algorithm (2-qubit):");
    let grover_circuit = create_grover_circuit(2)?;
    println!("{}", viz.draw_circuit(&grover_circuit));
    println!("{}", viz.statistics_table(&grover_circuit));

    Ok(())
}

/// Demo 5: Comparison table of different circuits
fn demo_comparison_table() -> Result<()> {
    println!("🔧 Demo 5: Circuit Comparison Analysis");
    println!("---------------------------------------");

    // Create various test circuits
    let circuits = vec![
        ("Bell State", create_bell_state()?),
        ("GHZ State", create_ghz_state(3)?),
        ("QFT-3", create_qft_circuit(3)?),
        ("VQE Ansatz", create_vqe_ansatz(3)?),
        ("Grover-2", create_grover_circuit(2)?),
        ("Random Circuit", create_random_circuit(4, 10)?),
    ];

    println!("📊 Circuit Comparison Table:");
    println!("┌─────────────────┬────────┬─────────┬───────┬──────────┬──────────┬─────────────┐");
    println!("│ Circuit         │ Qubits │ C.Bits  │ Gates │ Depth    │ 1Q Gates │ 2Q Gates    │");
    println!("├─────────────────┼────────┼─────────┼───────┼──────────┼──────────┼─────────────┤");

    let viz = CircuitVisualizer::new();

    for (name, circuit) in &circuits {
        let qubits = circuit.num_qubits();
        let clbits = circuit.num_clbits();
        let gates = circuit.size();
        let depth = circuit.depth();
        let single_q = viz.count_single_qubit_gates(circuit);
        let two_q = viz.count_two_qubit_gates(circuit);

        println!(
            "│ {:15} │ {:6} │ {:7} │ {:5} │ {:8} │ {:8} │ {:11} │",
            name, qubits, clbits, gates, depth, single_q, two_q
        );
    }

    println!("└─────────────────┴────────┴─────────┴───────┴──────────┴──────────┴─────────────┘");

    // Show detailed breakdown for most complex circuit
    println!("\n🔍 Detailed Analysis - Random Circuit:");
    let random_circuit = &circuits[5].1;
    println!("{}", viz.circuit_summary(random_circuit));
    println!("{}", viz.draw_circuit(random_circuit));

    Ok(())
}

// Helper functions to create test circuits

fn create_bell_state() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;
    Ok(circuit)
}

fn create_ghz_state(n: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(n, n);
    circuit.h(0)?;
    for i in 1..n {
        circuit.cx(0, i)?;
    }
    circuit.measure_all()?;
    Ok(circuit)
}

fn create_qft_circuit(n: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(n, n);

    for i in 0..n {
        circuit.h(i)?;
        for j in (i + 1)..n {
            let angle = PI / (1 << (j - i)) as f64;
            circuit.cp(i, j, Parameter::Float(angle))?;
        }
    }

    // Swap qubits (simplified)
    for i in 0..(n / 2) {
        circuit.swap(i, n - 1 - i)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

fn create_vqe_ansatz(n: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(n, n);

    // Layer 1: Single-qubit rotations
    for i in 0..n {
        circuit.ry(i, Parameter::Float(PI / 4.0))?;
    }

    // Layer 2: Entangling gates
    for i in 0..(n - 1) {
        circuit.cx(i, i + 1)?;
    }

    // Layer 3: More rotations
    for i in 0..n {
        circuit.rz(i, Parameter::Float(PI / 6.0))?;
    }

    // Layer 4: Ring connectivity
    if n > 2 {
        circuit.cx(n - 1, 0)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

fn create_grover_circuit(n: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(n, n);

    // Initialization: equal superposition
    for i in 0..n {
        circuit.h(i)?;
    }

    // Oracle (simplified - just a Z gate on all qubits)
    for i in 0..n {
        circuit.z(i)?;
    }

    // Diffusion operator (simplified)
    for i in 0..n {
        circuit.h(i)?;
        circuit.x(i)?;
    }

    // Multi-controlled Z (simplified with CZ gates)
    if n >= 2 {
        circuit.cz(0, 1)?;
    }

    for i in 0..n {
        circuit.x(i)?;
        circuit.h(i)?;
    }

    circuit.measure_all()?;
    Ok(circuit)
}

fn create_random_circuit(n: usize, depth: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(n, n);

    // Add pseudo-random gates (deterministic for demo)
    for i in 0..depth {
        let gate_type = i % 6;
        let qubit = i % n;

        match gate_type {
            0 => circuit.h(qubit)?,
            1 => circuit.x(qubit)?,
            2 => circuit.y(qubit)?,
            3 => circuit.z(qubit)?,
            4 => circuit.ry(qubit, Parameter::Float(PI / (i + 1) as f64))?,
            5 => {
                if n > 1 {
                    let target = (qubit + 1) % n;
                    circuit.cx(qubit, target)?;
                }
            }
            _ => circuit.i(qubit)?,
        }
    }

    circuit.measure_all()?;
    Ok(circuit)
}
