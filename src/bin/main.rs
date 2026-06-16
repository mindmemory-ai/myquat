//! MyQuat command-line interface
//!
//! This binary provides a command-line interface for the MyQuat quantum computing library.

use myquat::{transpiler::PassManager, CircuitVisualizer, Parameter, QuantumCircuit};
use std::collections::HashMap;
use std::env;
use std::f64::consts::PI;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "demo" => run_demo(),
        "bell" => create_bell_state(),
        "grover" => create_grover_circuit(),
        "qft" => create_qft_circuit(),
        "optimize" => {
            if args.len() < 3 {
                eprintln!("Usage: myquat optimize <circuit_file>");
                return;
            }
            optimize_circuit(&args[2]);
        }
        "visualize" => {
            if args.len() < 3 {
                eprintln!("Usage: myquat visualize <circuit_file>");
                return;
            }
            visualize_circuit(&args[2]);
        }
        "help" | "--help" | "-h" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!("MyQuat - Rust Quantum Computing Library");
    println!();
    println!("USAGE:");
    println!("    myquat <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    demo                 Run a demonstration of basic quantum circuits");
    println!("    bell                 Create and display a Bell state circuit");
    println!("    grover               Create a Grover's algorithm circuit");
    println!("    qft                  Create a Quantum Fourier Transform circuit");
    println!("    optimize <file>      Optimize a quantum circuit from file");
    println!("    visualize <file>     Visualize a quantum circuit from file");
    println!("    help                 Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    myquat demo");
    println!("    myquat bell");
    println!("    myquat optimize my_circuit.json");
}

fn run_demo() {
    println!("=== MyQuat Quantum Computing Demo ===\n");

    // Create a simple quantum circuit
    println!("1. Creating a simple quantum circuit...");
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0).expect("Failed to add Hadamard gate");
    circuit.cx(0, 1).expect("Failed to add CNOT gate");
    circuit.cx(1, 2).expect("Failed to add CNOT gate");
    circuit.measure(0, 0).expect("Failed to add measurement");
    circuit.measure(1, 1).expect("Failed to add measurement");
    circuit.measure(2, 2).expect("Failed to add measurement");

    println!("{}", CircuitVisualizer::to_text(&circuit));
    println!("ASCII representation:");
    println!("{}\n", CircuitVisualizer::to_ascii_art(&circuit));

    // Demonstrate parametric gates
    println!("2. Creating a parametric circuit...");
    let mut param_circuit = QuantumCircuit::new(2, 0);
    param_circuit
        .ry(0, Parameter::new_symbol("theta"))
        .expect("Failed to add RY gate");
    param_circuit.cx(0, 1).expect("Failed to add CNOT gate");
    param_circuit
        .rz(1, Parameter::new_float(PI / 4.0))
        .expect("Failed to add RZ gate");

    println!("{}", CircuitVisualizer::to_text(&param_circuit));

    // Bind parameters
    let mut symbols = HashMap::new();
    symbols.insert("theta".to_string(), PI / 3.0);

    let bound_circuit = param_circuit
        .bind_parameters(&symbols)
        .expect("Failed to bind parameters");
    println!("After binding theta = π/3:");
    println!("{}", CircuitVisualizer::to_text(&bound_circuit));

    // Demonstrate circuit optimization
    println!("3. Circuit optimization demo...");
    let mut unoptimized = QuantumCircuit::new(2, 0);
    unoptimized.i(0).expect("Failed to add I gate");
    unoptimized.s(0).expect("Failed to add S gate");
    unoptimized.sdg(0).expect("Failed to add Sdg gate");
    unoptimized.x(1).expect("Failed to add X gate");
    unoptimized.x(1).expect("Failed to add X gate");

    println!("Before optimization:");
    println!("{}", CircuitVisualizer::to_text(&unoptimized));

    let pass_manager = PassManager::default_optimization();
    pass_manager
        .run(&mut unoptimized)
        .expect("Failed to optimize circuit");

    println!("After optimization:");
    println!("{}", CircuitVisualizer::to_text(&unoptimized));

    println!("Demo completed!");
}

fn create_bell_state() {
    println!("=== Bell State Circuit ===\n");

    let mut circuit = QuantumCircuit::new_with_name(2, 2, "Bell State".to_string());
    circuit.h(0).expect("Failed to add Hadamard gate");
    circuit.cx(0, 1).expect("Failed to add CNOT gate");
    circuit.measure(0, 0).expect("Failed to add measurement");
    circuit.measure(1, 1).expect("Failed to add measurement");

    println!("Bell state circuit creates the entangled state (|00⟩ + |11⟩)/√2");
    println!("{}", CircuitVisualizer::to_text(&circuit));
    println!("ASCII representation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    // Show QASM export
    println!("\nQASM representation:");
    println!("{}", myquat::utils::CircuitIO::to_qasm(&circuit));
}

fn create_grover_circuit() {
    println!("=== Grover's Algorithm Circuit (2 qubits) ===\n");

    let mut circuit = QuantumCircuit::new_with_name(2, 2, "Grover 2-qubit".to_string());

    // Initialize superposition
    circuit.h(0).expect("Failed to add H gate");
    circuit.h(1).expect("Failed to add H gate");

    // Oracle (marking |11⟩)
    circuit.cz(0, 1).expect("Failed to add CZ gate");

    // Diffusion operator
    circuit.h(0).expect("Failed to add H gate");
    circuit.h(1).expect("Failed to add H gate");
    circuit.x(0).expect("Failed to add X gate");
    circuit.x(1).expect("Failed to add X gate");
    circuit.cz(0, 1).expect("Failed to add CZ gate");
    circuit.x(0).expect("Failed to add X gate");
    circuit.x(1).expect("Failed to add X gate");
    circuit.h(0).expect("Failed to add H gate");
    circuit.h(1).expect("Failed to add H gate");

    // Measurements
    circuit.measure(0, 0).expect("Failed to add measurement");
    circuit.measure(1, 1).expect("Failed to add measurement");

    println!("Grover's algorithm for 2 qubits, searching for |11⟩");
    println!("{}", CircuitVisualizer::to_text(&circuit));
    println!("ASCII representation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
}

fn create_qft_circuit() {
    println!("=== Quantum Fourier Transform Circuit (3 qubits) ===\n");

    let mut circuit = QuantumCircuit::new_with_name(3, 0, "QFT 3-qubit".to_string());

    // QFT implementation for 3 qubits
    // Qubit 0
    circuit.h(0).expect("Failed to add H gate");
    circuit
        .p(0, Parameter::new_float(PI / 2.0))
        .expect("Failed to add P gate");
    circuit.cx(1, 0).expect("Failed to add CNOT gate");
    circuit
        .p(0, Parameter::new_float(PI / 4.0))
        .expect("Failed to add P gate");
    circuit.cx(2, 0).expect("Failed to add CNOT gate");

    // Qubit 1
    circuit.h(1).expect("Failed to add H gate");
    circuit
        .p(1, Parameter::new_float(PI / 2.0))
        .expect("Failed to add P gate");
    circuit.cx(2, 1).expect("Failed to add CNOT gate");

    // Qubit 2
    circuit.h(2).expect("Failed to add H gate");

    // Swap qubits to get correct order
    circuit.swap(0, 2).expect("Failed to add SWAP gate");

    println!("3-qubit Quantum Fourier Transform");
    println!("{}", CircuitVisualizer::to_text(&circuit));
    println!("ASCII representation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
}

fn optimize_circuit(filename: &str) {
    println!("=== Circuit Optimization ===\n");

    match myquat::utils::CircuitIO::load_from_file(filename) {
        Ok(mut circuit) => {
            println!("Loaded circuit from: {}", filename);
            println!("Before optimization:");
            println!("{}", CircuitVisualizer::to_text(&circuit));

            let pass_manager = PassManager::default_optimization();
            match pass_manager.run(&mut circuit) {
                Ok(()) => {
                    println!("After optimization:");
                    println!("{}", CircuitVisualizer::to_text(&circuit));

                    // Save optimized circuit
                    let output_filename = format!(
                        "{}_optimized.json",
                        filename.strip_suffix(".json").unwrap_or(filename)
                    );

                    match myquat::utils::CircuitIO::save_to_file(&circuit, &output_filename) {
                        Ok(()) => println!("Optimized circuit saved to: {}", output_filename),
                        Err(e) => eprintln!("Failed to save optimized circuit: {}", e),
                    }
                }
                Err(e) => eprintln!("Optimization failed: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to load circuit: {}", e),
    }
}

fn visualize_circuit(filename: &str) {
    println!("=== Circuit Visualization ===\n");

    match myquat::utils::CircuitIO::load_from_file(filename) {
        Ok(circuit) => {
            println!("Circuit from: {}", filename);
            println!("{}", CircuitVisualizer::to_text(&circuit));
            println!("\nASCII representation:");
            println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
            println!("\nQASM representation:");
            println!("{}", myquat::utils::CircuitIO::to_qasm(&circuit));
        }
        Err(e) => eprintln!("Failed to load circuit: {}", e),
    }
}
