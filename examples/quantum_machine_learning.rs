//! # Quantum Machine Learning Examples
//!
//! This example demonstrates various quantum machine learning algorithms
//! including quantum neural networks, quantum feature maps, and QAOA.

use myquat::*;
use std::collections::HashMap;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🤖 Quantum Machine Learning Demonstrations");
    println!("==========================================\n");

    demo_quantum_feature_maps()?;
    demo_variational_classifier()?;
    demo_qaoa_maxcut()?;
    demo_quantum_neural_network()?;
    demo_quantum_kernel_methods()?;

    println!("🎯 Quantum ML Summary:");
    println!("- Quantum feature maps can provide exponential feature space");
    println!("- Variational circuits enable trainable quantum models");
    println!("- QAOA solves combinatorial optimization problems");
    println!("- Quantum kernels offer new similarity measures");
    println!("- Near-term algorithms suitable for NISQ devices");

    Ok(())
}

/// Demo 1: Quantum Feature Maps
fn demo_quantum_feature_maps() -> Result<()> {
    println!("🎯 Demo 1: Quantum Feature Maps");
    println!("-------------------------------\n");

    println!("Quantum feature maps encode classical data into quantum states,");
    println!("potentially providing exponential feature space expansion.\n");

    // Create different types of feature maps
    let data_point = vec![0.5, -0.3, 0.8]; // 3D classical data point
    println!("Input data: {:?}", data_point);

    // 1. Angle encoding feature map
    println!("\n🔄 Angle Encoding Feature Map:");
    let angle_circuit = create_angle_encoding_feature_map(&data_point)?;
    println!("{}", CircuitVisualizer::to_ascii_art(&angle_circuit));
    println!("Encodes data as rotation angles: RY(π×x_i)");

    // 2. Amplitude encoding feature map
    println!("\n📊 Amplitude Encoding Feature Map:");
    let amplitude_circuit = create_amplitude_encoding_feature_map(&data_point)?;
    println!("{}", CircuitVisualizer::to_ascii_art(&amplitude_circuit));
    println!("Encodes data as quantum state amplitudes");

    // 3. Higher-order feature map
    println!("\n🔗 Higher-order Feature Map (with entanglement):");
    let higher_order_circuit = create_higher_order_feature_map(&data_point)?;
    println!("{}", CircuitVisualizer::to_ascii_art(&higher_order_circuit));
    println!("Creates non-linear feature interactions through entanglement");

    println!("\n💡 Feature Map Properties:");
    println!("- Angle encoding: Linear in data, easy to implement");
    println!("- Amplitude encoding: Exponential capacity, normalization required");
    println!("- Higher-order: Non-linear features, better expressivity");

    println!();
    Ok(())
}

/// Demo 2: Variational Quantum Classifier
fn demo_variational_classifier() -> Result<()> {
    println!("🎯 Demo 2: Variational Quantum Classifier");
    println!("-----------------------------------------\n");

    println!("Variational quantum classifiers use parameterized quantum circuits");
    println!("trained with classical optimization to classify data.\n");

    // Create a simple 2-class classification problem
    let training_data = vec![
        (vec![0.2, 0.8], 0),   // Class 0
        (vec![0.7, 0.3], 0),   // Class 0
        (vec![-0.3, -0.6], 1), // Class 1
        (vec![-0.8, -0.2], 1), // Class 1
    ];

    println!("📚 Training Data:");
    for (i, (features, label)) in training_data.iter().enumerate() {
        println!("  Sample {}: {:?} → Class {}", i + 1, features, label);
    }

    // Create variational classifier circuit
    let num_features = 2;
    let num_layers = 2;
    let classifier_circuit = create_variational_classifier(num_features, num_layers)?;

    println!("\n🧠 Variational Classifier Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&classifier_circuit));

    // Show parameter structure
    println!("🎛️ Trainable Parameters:");
    println!("- Feature encoding: {} rotation parameters", num_features);
    println!(
        "- Variational layers: {} parameters per layer",
        num_features * 2
    );
    println!(
        "- Total parameters: {}",
        num_features + num_layers * num_features * 2
    );

    println!("\n🔄 Training Process:");
    println!("1. Encode classical data using feature map");
    println!("2. Apply parameterized variational circuit");
    println!("3. Measure expectation value as prediction");
    println!("4. Update parameters using gradient descent");
    println!("5. Repeat until convergence");

    // Simulate training progress
    println!("\n📈 Simulated Training Progress:");
    let training_losses = vec![0.85, 0.72, 0.58, 0.41, 0.29, 0.18, 0.12, 0.08];
    for (epoch, loss) in training_losses.iter().enumerate() {
        let bar = "█".repeat((10.0 * (1.0 - loss)) as usize);
        println!("Epoch {}: Loss = {:.3} {}", epoch + 1, loss, bar);
    }

    println!();
    Ok(())
}

/// Demo 3: QAOA for Max-Cut Problem
fn demo_qaoa_maxcut() -> Result<()> {
    println!("🎯 Demo 3: QAOA for Max-Cut Problem");
    println!("-----------------------------------\n");

    println!("Quantum Approximate Optimization Algorithm (QAOA) solves");
    println!("combinatorial optimization problems using variational quantum circuits.\n");

    // Define a simple graph for Max-Cut
    let edges = vec![(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)]; // 4-node graph
    let num_nodes = 4;

    println!("🔗 Graph for Max-Cut Problem:");
    println!("Nodes: {}", num_nodes);
    println!("Edges: {:?}", edges);
    println!("Goal: Find cut that maximizes edges between partitions");

    // Create QAOA circuit
    let qaoa_layers = 2;
    let qaoa_circuit = create_qaoa_maxcut_circuit(num_nodes, &edges, qaoa_layers)?;

    println!("\n🔄 QAOA Circuit (p={} layers):", qaoa_layers);
    println!("{}", CircuitVisualizer::to_ascii_art(&qaoa_circuit));

    println!("🧮 QAOA Components:");
    println!("1. Initial state: |+⟩^⊗n (equal superposition)");
    println!("2. Problem Hamiltonian: H_C = Σ(1 - Z_i Z_j)/2 for each edge");
    println!("3. Mixer Hamiltonian: H_B = Σ X_i");
    println!("4. Alternating layers: e^(-iγH_C) e^(-iβH_B)");

    // Show parameter optimization
    println!("\n🎛️ QAOA Parameters:");
    println!("- γ parameters (problem): {} values", qaoa_layers);
    println!("- β parameters (mixer): {} values", qaoa_layers);
    println!("- Total parameters: {}", 2 * qaoa_layers);

    println!("\n📊 Expected Max-Cut Solutions:");
    println!("For this 4-node graph, optimal cuts:");
    println!("- Partition {{0,2}} vs {{1,3}}: 4 edges cut");
    println!("- Partition {{0,3}} vs {{1,2}}: 3 edges cut");

    println!();
    Ok(())
}

/// Demo 4: Quantum Neural Network
fn demo_quantum_neural_network() -> Result<()> {
    println!("🎯 Demo 4: Quantum Neural Network");
    println!("---------------------------------\n");

    println!("Quantum neural networks use quantum circuits as trainable");
    println!("computational units, analogous to classical neural networks.\n");

    let input_size = 3;
    let hidden_size = 4;
    let output_size = 2;

    println!("🧠 Network Architecture:");
    println!("Input layer: {} qubits", input_size);
    println!("Hidden layer: {} qubits", hidden_size);
    println!("Output layer: {} qubits", output_size);

    // Create quantum neural network
    let qnn_circuit = create_quantum_neural_network(input_size, hidden_size, output_size)?;

    println!("\n🔗 Quantum Neural Network Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&qnn_circuit));

    println!("🔄 QNN Components:");
    println!("1. Data encoding layer: Maps classical input to quantum state");
    println!("2. Variational layers: Parameterized quantum gates (weights)");
    println!("3. Entangling layers: Create quantum correlations");
    println!("4. Measurement layer: Extract classical output");

    println!("\n⚡ Quantum Advantages:");
    println!("- Exponential parameter space");
    println!("- Natural handling of quantum data");
    println!("- Potential quantum speedup for certain problems");
    println!("- Built-in regularization through quantum interference");

    println!("\n🎯 Applications:");
    println!("- Quantum data classification");
    println!("- Quantum state preparation");
    println!("- Quantum control optimization");
    println!("- Hybrid quantum-classical models");

    println!();
    Ok(())
}

/// Demo 5: Quantum Kernel Methods
fn demo_quantum_kernel_methods() -> Result<()> {
    println!("🎯 Demo 5: Quantum Kernel Methods");
    println!("---------------------------------\n");

    println!("Quantum kernels compute similarity between data points");
    println!("in quantum feature space, enabling quantum-enhanced ML.\n");

    // Sample data points
    let data_points = vec![vec![0.5, 0.3], vec![-0.2, 0.8], vec![0.7, -0.4]];

    println!("📊 Data Points:");
    for (i, point) in data_points.iter().enumerate() {
        println!("  x_{}: {:?}", i + 1, point);
    }

    // Create quantum kernel circuit
    let kernel_circuit = create_quantum_kernel_circuit(&data_points[0], &data_points[1])?;

    println!("\n🔗 Quantum Kernel Circuit (for x_1, x_2):");
    println!("{}", CircuitVisualizer::to_ascii_art(&kernel_circuit));

    println!("🧮 Kernel Computation:");
    println!("1. Encode x_i using feature map: |φ(x_i)⟩");
    println!("2. Encode x_j using feature map: |φ(x_j)⟩");
    println!("3. Compute overlap: K(x_i, x_j) = |⟨φ(x_i)|φ(x_j)⟩|²");
    println!("4. Use kernel matrix in classical ML algorithms");

    // Simulate kernel matrix
    println!("\n📊 Simulated Quantum Kernel Matrix:");
    println!("┌─────────┬─────────┬─────────┬─────────┐");
    println!("│         │   x_1   │   x_2   │   x_3   │");
    println!("├─────────┼─────────┼─────────┼─────────┤");
    let kernel_values = [
        [1.000, 0.742, 0.231],
        [0.742, 1.000, 0.156],
        [0.231, 0.156, 1.000],
    ];

    for i in 0..3 {
        print!("│   x_{}   │", i + 1);
        for j in 0..3 {
            print!(" {:7.3} │", kernel_values[i][j]);
        }
        println!();
    }
    println!("└─────────┴─────────┴─────────┴─────────┘");

    println!("\n🎯 Quantum Kernel Advantages:");
    println!("- Access to exponentially large feature spaces");
    println!("- Quantum interference effects in similarity");
    println!("- Natural quantum data processing");
    println!("- Provable quantum advantage for certain datasets");

    println!();
    Ok(())
}

/// Create angle encoding feature map
fn create_angle_encoding_feature_map(data: &[f64]) -> Result<QuantumCircuit> {
    let num_qubits = data.len();
    let mut circuit = QuantumCircuit::new(num_qubits, 0);

    for (i, &value) in data.iter().enumerate() {
        circuit.ry(i, Parameter::Float(PI * value))?;
    }

    Ok(circuit)
}

/// Create amplitude encoding feature map
fn create_amplitude_encoding_feature_map(data: &[f64]) -> Result<QuantumCircuit> {
    let num_qubits = (data.len() as f64).log2().ceil() as usize;
    let mut circuit = QuantumCircuit::new(num_qubits, 0);

    // Simplified amplitude encoding (real implementation would be more complex)
    for (i, &value) in data.iter().enumerate() {
        if i < num_qubits {
            circuit.ry(i, Parameter::Float(2.0 * value.abs().asin()))?;
        }
    }

    Ok(circuit)
}

/// Create higher-order feature map
fn create_higher_order_feature_map(data: &[f64]) -> Result<QuantumCircuit> {
    let num_qubits = data.len();
    let mut circuit = QuantumCircuit::new(num_qubits, 0);

    // First-order encoding
    for (i, &value) in data.iter().enumerate() {
        circuit.ry(i, Parameter::Float(PI * value))?;
    }

    // Second-order interactions through entanglement
    for i in 0..num_qubits {
        for j in i + 1..num_qubits {
            circuit.cx(i, j)?;
            circuit.rz(j, Parameter::Float(PI * data[i] * data[j]))?;
            circuit.cx(i, j)?;
        }
    }

    Ok(circuit)
}

/// Create variational classifier circuit
fn create_variational_classifier(num_features: usize, num_layers: usize) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_features, 1);

    // Feature encoding (would be filled with actual data)
    for i in 0..num_features {
        circuit.ry(i, Parameter::Symbol(format!("x_{}", i)))?;
    }

    // Variational layers
    for layer in 0..num_layers {
        // Parameterized rotations
        for i in 0..num_features {
            circuit.ry(i, Parameter::Symbol(format!("theta_{}_{}", layer, i)))?;
            circuit.rz(i, Parameter::Symbol(format!("phi_{}_{}", layer, i)))?;
        }

        // Entangling gates
        for i in 0..num_features - 1 {
            circuit.cx(i, i + 1)?;
        }
        if num_features > 2 {
            circuit.cx(num_features - 1, 0)?; // Circular entanglement
        }
    }

    // Measurement (expectation value of first qubit)
    circuit.measure(0, 0)?;

    Ok(circuit)
}

/// Create QAOA circuit for Max-Cut
fn create_qaoa_maxcut_circuit(
    num_nodes: usize,
    edges: &[(usize, usize)],
    layers: usize,
) -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(num_nodes, num_nodes);

    // Initial state: |+⟩^⊗n
    for i in 0..num_nodes {
        circuit.h(i)?;
    }

    // QAOA layers
    for layer in 0..layers {
        // Problem Hamiltonian: e^(-iγH_C)
        for &(i, j) in edges {
            circuit.cx(i, j)?;
            circuit.rz(j, Parameter::Symbol(format!("gamma_{}", layer)))?;
            circuit.cx(i, j)?;
        }

        // Mixer Hamiltonian: e^(-iβH_B)
        for i in 0..num_nodes {
            circuit.rx(i, Parameter::Symbol(format!("beta_{}", layer)))?;
        }
    }

    // Measurements
    circuit.measure_all()?;

    Ok(circuit)
}

/// Create quantum neural network
fn create_quantum_neural_network(
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
) -> Result<QuantumCircuit> {
    let total_qubits = input_size.max(hidden_size).max(output_size);
    let mut circuit = QuantumCircuit::new(total_qubits, output_size);

    // Input encoding
    for i in 0..input_size {
        circuit.ry(i, Parameter::Symbol(format!("input_{}", i)))?;
    }

    // Hidden layer processing
    for i in 0..hidden_size {
        circuit.ry(i, Parameter::Symbol(format!("w1_{}", i)))?;
        circuit.rz(i, Parameter::Symbol(format!("w2_{}", i)))?;
    }

    // Entangling layer
    for i in 0..hidden_size - 1 {
        circuit.cx(i, i + 1)?;
    }

    // Output layer
    for i in 0..output_size {
        circuit.ry(i, Parameter::Symbol(format!("w_out_{}", i)))?;
    }

    // Measurements
    for i in 0..output_size {
        circuit.measure(i, i)?;
    }

    Ok(circuit)
}

/// Create quantum kernel circuit
fn create_quantum_kernel_circuit(data1: &[f64], data2: &[f64]) -> Result<QuantumCircuit> {
    let num_qubits = data1.len() * 2; // Double for both data points
    let mut circuit = QuantumCircuit::new(num_qubits, 1);

    // Encode first data point
    for (i, &value) in data1.iter().enumerate() {
        circuit.ry(i, Parameter::Float(PI * value))?;
    }

    // Encode second data point
    for (i, &value) in data2.iter().enumerate() {
        circuit.ry(i + data1.len(), Parameter::Float(PI * value))?;
    }

    // Create entanglement between corresponding qubits
    for i in 0..data1.len() {
        circuit.cx(i, i + data1.len())?;
    }

    // Measure overlap (simplified)
    circuit.measure(0, 0)?;

    Ok(circuit)
}
