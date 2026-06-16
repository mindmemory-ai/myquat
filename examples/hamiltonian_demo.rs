//! Hamiltonian Duality Demo
//!
//! Author: gA4ss
//!
//! This example demonstrates the bidirectional conversion between
//! quantum circuits and Hamiltonians in MyQuat.
//!
//! Features:
//! 1. Circuit → Hamiltonian extraction
//! 2. Hamiltonian → Circuit compilation
//! 3. LaTeX and Markdown export
//! 4. Practical examples (Ising, Heisenberg models)

use myquat::hamiltonian::*;
use myquat::*;
use num_complex::Complex64;

fn main() -> Result<()> {
    println!("=================================================");
    println!("  MyQuat Hamiltonian Duality Demonstration");
    println!("=================================================\n");

    // Demo 1: Circuit to Hamiltonian
    demo_circuit_to_hamiltonian()?;

    // Demo 2: Hamiltonian to Circuit
    demo_hamiltonian_to_circuit()?;

    // Demo 3: Round-trip conversion
    demo_round_trip()?;

    // Demo 4: Physical models
    demo_physical_models()?;

    // Demo 5: LaTeX and Markdown export
    demo_export()?;

    println!("\n=================================================");
    println!("  All demonstrations completed successfully!");
    println!("=================================================");

    Ok(())
}

/// Demo 1: Extract Hamiltonian from quantum circuit
fn demo_circuit_to_hamiltonian() -> Result<()> {
    println!("--- Demo 1: Circuit to Hamiltonian Extraction ---\n");

    // Create a simple quantum circuit with rotation gates
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.rx(0, Parameter::Float(0.5))?;
    circuit.ry(1, Parameter::Float(0.3))?;
    circuit.rz(0, Parameter::Float(0.2))?;

    println!("Original Circuit:");
    println!("{}\n", CircuitVisualizer::to_ascii_art(&circuit));

    // Analyze the circuit to extract Hamiltonian
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);

    println!("Number of terms: {}", analysis.hamiltonian.num_terms());
    println!("Is Hermitian: {}", analysis.hamiltonian.is_hermitian());

    if let Some(steps) = analysis.trotter_steps {
        println!("Detected Trotter steps: {}", steps);
    }

    println!();
    Ok(())
}

/// Demo 2: Compile Hamiltonian into quantum circuit
fn demo_hamiltonian_to_circuit() -> Result<()> {
    println!("--- Demo 2: Hamiltonian to Circuit Compilation ---\n");

    // Create a simple Hamiltonian: H = X + 0.5*Y
    let mut h = Hamiltonian::new(1);

    let x_string = PauliString::from_str("X")?;
    h.add_term(x_string, Complex64::new(1.0, 0.0))?;

    let y_string = PauliString::from_str("Y")?;
    h.add_term(y_string, Complex64::new(0.5, 0.0))?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // Compile to circuit with different configurations
    let config1 = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
        evolution_time: 1.0,
        ..Default::default()
    };

    let compiler1 = HamiltonianCompiler::new(config1);
    let circuit1 = compiler1.compile(&h)?;

    println!("Compiled Circuit (1st order, 1 step):");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit1));
    println!("Gates: {}\n", circuit1.size());

    let config2 = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 5,
        evolution_time: 2.0,
        ..Default::default()
    };

    let compiler2 = HamiltonianCompiler::new(config2);
    let circuit2 = compiler2.compile(&h)?;

    println!("Compiled Circuit (2nd order, 5 steps):");
    println!("Gates: {} (more accurate)", circuit2.size());

    println!();
    Ok(())
}

/// Demo 3: Round-trip conversion (Circuit → H → Circuit)
fn demo_round_trip() -> Result<()> {
    println!("--- Demo 3: Round-trip Conversion ---\n");

    // Start with a circuit
    let mut original = QuantumCircuit::new(2, 0);
    original.rx(0, Parameter::Float(0.8))?;
    original.ry(1, Parameter::Float(0.6))?;

    println!("Original Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&original));
    println!("Original gates: {}\n", original.size());

    // Extract Hamiltonian
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&original)?;
    let extracted_h = &analysis.hamiltonian;

    println!("Extracted Hamiltonian:");
    println!("{}\n", extracted_h);

    // Compile back to circuit
    let config = CompilerConfig {
        trotter_steps: 1,
        evolution_time: 1.0,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config);
    let reconstructed = compiler.compile(extracted_h)?;

    println!("Reconstructed Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&reconstructed));
    println!("Reconstructed gates: {}", reconstructed.size());

    println!("\nNote: The reconstructed circuit may differ in gate decomposition");
    println!("but implements the same Hamiltonian time evolution.");

    println!();
    Ok(())
}

/// Demo 4: Physical model Hamiltonians
fn demo_physical_models() -> Result<()> {
    println!("--- Demo 4: Physical Model Hamiltonians ---\n");

    // 1. Ising Model
    println!("1. Ising Model Hamiltonian");
    println!("   H = -J*sum(Z_i*Z_j) - h*sum(X_i)\n");

    let ising = constructors::ising_model(3, 1.0, 0.5)?;
    println!(
        "   {} qubits, {} terms",
        ising.num_qubits,
        ising.num_terms()
    );
    println!("   {}\n", ising);

    // Compile Ising model
    let config = CompilerConfig {
        trotter_steps: 10,
        evolution_time: 1.0,
        ..Default::default()
    };
    let compiler = HamiltonianCompiler::new(config);
    let ising_circuit = compiler.compile(&ising)?;
    println!("   Compiled to {} gates\n", ising_circuit.size());

    // 2. Heisenberg Model
    println!("2. Heisenberg Model Hamiltonian");
    println!("   H = sum(J_x*X_i*X_j + J_y*Y_i*Y_j + J_z*Z_i*Z_j)\n");

    let heisenberg = constructors::heisenberg_model(3, 1.0, 1.0, 1.0)?;
    println!(
        "   {} qubits, {} terms",
        heisenberg.num_qubits,
        heisenberg.num_terms()
    );

    let heisenberg_circuit = compiler.compile(&heisenberg)?;
    println!("   Compiled to {} gates\n", heisenberg_circuit.size());

    // 3. Custom Hamiltonian
    println!("3. Custom Hamiltonian");
    let mut custom = Hamiltonian::new(2);

    // Add XY interaction
    let xy = PauliString::from_str("XY")?;
    custom.add_term(xy, Complex64::new(0.5, 0.0))?;

    // Add ZZ interaction
    let zz = PauliString::from_str("ZZ")?;
    custom.add_term(zz, Complex64::new(-0.3, 0.0))?;

    println!("   {}", custom);

    let custom_circuit = compiler.compile(&custom)?;
    println!("   Compiled to {} gates\n", custom_circuit.size());

    println!();
    Ok(())
}

/// Demo 5: LaTeX and Markdown export
fn demo_export() -> Result<()> {
    println!("--- Demo 5: LaTeX and Markdown Export ---\n");

    // Create Heisenberg model
    let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0)?;

    println!("Hamiltonian: {}\n", h);

    // Export to LaTeX
    println!("LaTeX representation:");
    println!("{}\n", h.to_latex());

    // Export to Markdown
    println!("Markdown representation:");
    println!("{}\n", h.to_markdown());

    // Export to JSON
    println!("JSON representation:");
    match h.to_json() {
        Ok(json) => println!("{}\n", json),
        Err(e) => println!("JSON export error: {}\n", e),
    }

    // Individual Pauli term export
    if let Some(term) = h.terms.first() {
        println!("First term LaTeX: {}", term.to_latex());
        println!("Pauli string: {}", term.pauli_string.to_string_repr());
    }

    println!();
    Ok(())
}
