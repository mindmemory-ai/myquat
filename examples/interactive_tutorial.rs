//! # Interactive MyQuat Tutorial
//!
//! This example provides an interactive tutorial for learning MyQuat's API
//! through hands-on examples with detailed explanations and runnable code.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🎓 Welcome to the Interactive MyQuat Tutorial!");
    println!("==============================================\n");

    tutorial_1_basic_circuits()?;
    tutorial_2_quantum_gates()?;
    tutorial_3_measurements()?;
    tutorial_4_parametric_circuits()?;
    tutorial_5_visualization()?;
    tutorial_6_simulation()?;
    tutorial_7_noise_and_mitigation()?;
    tutorial_8_advanced_features()?;

    println!("🎉 Congratulations! You've completed the MyQuat tutorial!");
    println!("Next steps:");
    println!("- Explore more examples in the examples/ directory");
    println!("- Read the full documentation with: cargo doc --open");
    println!("- Try building your own quantum algorithms!");

    Ok(())
}

/// Tutorial 1: Basic Circuit Construction
fn tutorial_1_basic_circuits() -> Result<()> {
    println!("📚 Tutorial 1: Basic Circuit Construction");
    println!("=========================================\n");

    println!("Let's start by creating your first quantum circuit!");
    println!(
        "A quantum circuit needs qubits (quantum bits) and classical bits for measurements.\n"
    );

    // Create a simple 2-qubit circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    println!("✨ Created a circuit with 2 qubits and 2 classical bits:");
    println!("   let mut circuit = QuantumCircuit::new(2, 2);\n");

    // Add some gates
    circuit.h(0)?; // Hadamard gate on qubit 0
    circuit.cx(0, 1)?; // CNOT gate from qubit 0 to 1

    println!("🚪 Added quantum gates:");
    println!("   circuit.h(0)?;      // Hadamard gate creates superposition");
    println!("   circuit.cx(0, 1)?;  // CNOT gate creates entanglement\n");

    // Visualize the circuit
    println!("🎨 Circuit visualization:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    println!("📊 Circuit statistics:");
    println!("   Qubits: {}", circuit.num_qubits());
    println!("   Classical bits: {}", circuit.num_clbits());
    println!("   Gates: {}", circuit.size());
    println!("   Depth: {}", circuit.depth());
    println!("   Is empty: {}\n", circuit.is_empty());

    println!("🎯 Key concepts learned:");
    println!("   - QuantumCircuit::new(qubits, classical_bits) creates a circuit");
    println!("   - Gates are applied with simple method calls");
    println!("   - Circuits can be analyzed with built-in metrics\n");

    Ok(())
}

/// Tutorial 2: Quantum Gates
fn tutorial_2_quantum_gates() -> Result<()> {
    println!("📚 Tutorial 2: Quantum Gates");
    println!("=============================\n");

    println!("MyQuat supports all standard quantum gates. Let's explore them!\n");

    // Single-qubit gates
    println!("🔧 Single-Qubit Gates:");
    let mut single_qubit_demo = QuantumCircuit::new(1, 0);

    single_qubit_demo.i(0)?; // Identity
    single_qubit_demo.x(0)?; // Pauli-X (NOT gate)
    single_qubit_demo.y(0)?; // Pauli-Y
    single_qubit_demo.z(0)?; // Pauli-Z
    single_qubit_demo.h(0)?; // Hadamard
    single_qubit_demo.s(0)?; // S gate
    single_qubit_demo.t(0)?; // T gate

    println!("   I, X, Y, Z - Pauli gates (fundamental operations)");
    println!("   H - Hadamard (creates superposition)");
    println!("   S, T - Phase gates (add quantum phases)");
    println!("{}", CircuitVisualizer::to_ascii_art(&single_qubit_demo));

    // Parametric rotation gates
    println!("🔄 Parametric Rotation Gates:");
    let mut rotation_demo = QuantumCircuit::new(1, 0);

    rotation_demo.rx(0, Parameter::Float(PI / 4.0))?; // Rotation around X-axis
    rotation_demo.ry(0, Parameter::Float(PI / 3.0))?; // Rotation around Y-axis
    rotation_demo.rz(0, Parameter::Float(PI / 6.0))?; // Rotation around Z-axis

    println!("   RX, RY, RZ - Rotation gates with angle parameters");
    println!("   Angles can be symbolic or numeric");
    println!("{}", CircuitVisualizer::to_ascii_art(&rotation_demo));

    // Two-qubit gates
    println!("🔗 Two-Qubit Gates (Entangling):");
    let mut two_qubit_demo = QuantumCircuit::new(3, 0);

    two_qubit_demo.cx(0, 1)?; // CNOT (Controlled-X)
    two_qubit_demo.cz(1, 2)?; // Controlled-Z
    two_qubit_demo.swap(0, 2)?; // SWAP gate

    println!("   CX (CNOT) - Controlled-X, creates entanglement");
    println!("   CZ - Controlled-Z, phase-flip controlled gate");
    println!("   SWAP - Exchanges two qubit states");
    println!("{}", CircuitVisualizer::to_ascii_art(&two_qubit_demo));

    println!("🎯 Key concepts learned:");
    println!("   - Single-qubit gates: I, X, Y, Z, H, S, T");
    println!("   - Parametric gates: RX, RY, RZ with angle parameters");
    println!("   - Two-qubit gates: CX, CZ, SWAP for entanglement");
    println!("   - All gates return Result<()> for error handling\n");

    Ok(())
}

/// Tutorial 3: Measurements
fn tutorial_3_measurements() -> Result<()> {
    println!("📚 Tutorial 3: Measurements and Classical Bits");
    println!("===============================================\n");

    println!("Quantum measurements collapse quantum states to classical bits.\n");

    // Bell state with measurements
    let mut bell_circuit = QuantumCircuit::new(2, 2);
    bell_circuit.h(0)?;
    bell_circuit.cx(0, 1)?;

    println!("🔔 Bell State Preparation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&bell_circuit));

    // Add measurements
    bell_circuit.measure(0, 0)?; // Measure qubit 0 to classical bit 0
    bell_circuit.measure(1, 1)?; // Measure qubit 1 to classical bit 1

    println!("📏 Adding Measurements:");
    println!("   circuit.measure(0, 0)?;  // Qubit 0 → Classical bit 0");
    println!("   circuit.measure(1, 1)?;  // Qubit 1 → Classical bit 1");
    println!("{}", CircuitVisualizer::to_ascii_art(&bell_circuit));

    // Measure all shortcut
    let mut ghz_circuit = QuantumCircuit::new(3, 3);
    ghz_circuit.h(0)?;
    ghz_circuit.cx(0, 1)?;
    ghz_circuit.cx(1, 2)?;
    ghz_circuit.measure_all()?; // Convenient shortcut

    println!("🚀 Measure All Shortcut:");
    println!("   circuit.measure_all()?;  // Measures all qubits to classical bits");
    println!("{}", CircuitVisualizer::to_ascii_art(&ghz_circuit));

    println!("🎯 Key concepts learned:");
    println!("   - measure(qubit, classical_bit) for individual measurements");
    println!("   - measure_all() for measuring all qubits at once");
    println!("   - Measurements collapse quantum superposition to classical bits");
    println!("   - Classical bits store the measurement outcomes\n");

    Ok(())
}

/// Tutorial 4: Parametric Circuits
fn tutorial_4_parametric_circuits() -> Result<()> {
    println!("📚 Tutorial 4: Parametric Circuits");
    println!("===================================\n");

    println!("Parametric circuits use symbolic parameters that can be bound later.\n");

    // Create a parametric circuit
    let mut parametric_circuit = QuantumCircuit::new(2, 0);

    // Use symbolic parameters
    parametric_circuit.ry(0, Parameter::Symbol("theta".to_string()))?;
    parametric_circuit.cx(0, 1)?;
    parametric_circuit.rz(1, Parameter::Symbol("phi".to_string()))?;

    println!("🔧 Parametric Circuit with Symbols:");
    println!("   ry(0, Parameter::Symbol(\"theta\"))?;");
    println!("   rz(1, Parameter::Symbol(\"phi\"))?;");
    println!("{}", CircuitVisualizer::to_ascii_art(&parametric_circuit));

    // Bind parameters
    use std::collections::HashMap;
    let mut params = HashMap::new();
    params.insert("theta".to_string(), PI / 4.0);
    params.insert("phi".to_string(), PI / 6.0);

    let bound_circuit = parametric_circuit.bind_parameters(&params)?;

    println!("🔗 Parameter Binding:");
    println!("   let mut params = HashMap::new();");
    println!("   params.insert(\"theta\", π/4);");
    println!("   params.insert(\"phi\", π/6);");
    println!("   let bound = circuit.bind_parameters(&params)?;");
    println!("{}", CircuitVisualizer::to_ascii_art(&bound_circuit));

    // Numeric parameters
    let mut numeric_circuit = QuantumCircuit::new(2, 0);
    numeric_circuit.ry(0, Parameter::Float(PI / 3.0))?;
    numeric_circuit.cx(0, 1)?;

    println!("🔢 Direct Numeric Parameters:");
    println!("   ry(0, Parameter::Float(π/3))?;");
    println!("{}", CircuitVisualizer::to_ascii_art(&numeric_circuit));

    println!("🎯 Key concepts learned:");
    println!("   - Parameter::Symbol(\"name\") for symbolic parameters");
    println!("   - Parameter::Float(value) for numeric parameters");
    println!("   - bind_parameters() to substitute symbols with values");
    println!("   - Useful for variational algorithms and optimization\n");

    Ok(())
}

/// Tutorial 5: Visualization
fn tutorial_5_visualization() -> Result<()> {
    println!("📚 Tutorial 5: Circuit Visualization");
    println!("====================================\n");

    println!("MyQuat provides powerful visualization tools for circuits.\n");

    // Create a complex circuit for visualization
    let mut complex_circuit = QuantumCircuit::new(4, 4);
    complex_circuit.h(0)?;
    complex_circuit.cx(0, 1)?;
    complex_circuit.ry(2, Parameter::Float(PI / 4.0))?;
    complex_circuit.cz(1, 2)?;
    complex_circuit.swap(0, 3)?;
    complex_circuit.measure_all()?;

    println!("🎨 Basic ASCII Art:");
    println!("{}", CircuitVisualizer::to_ascii_art(&complex_circuit));

    // Advanced visualization
    let viz = CircuitVisualizer::new();

    println!("📊 Circuit Summary:");
    println!("{}", viz.circuit_summary(&complex_circuit));

    println!("📈 Statistics Table:");
    println!("{}", viz.statistics_table(&complex_circuit));

    println!("🔍 Gate-by-Gate Breakdown:");
    println!("{}", viz.gate_breakdown(&complex_circuit));

    // Different visualization styles
    println!("🎭 Different Visualization Styles:");

    let compact = CircuitVisualizer::compact();
    println!("Compact style:");
    println!("{}", compact.draw_circuit(&complex_circuit));

    let detailed = CircuitVisualizer::detailed();
    println!("Detailed style:");
    println!("{}", detailed.draw_circuit(&complex_circuit));

    println!("🎯 Key concepts learned:");
    println!("   - CircuitVisualizer::to_ascii_art() for simple visualization");
    println!("   - CircuitVisualizer::new() for advanced features");
    println!("   - circuit_summary(), statistics_table(), gate_breakdown()");
    println!("   - Different styles: compact(), detailed(), custom styling");
    println!("   - SVG export with export_svg() for publications\n");

    Ok(())
}

/// Tutorial 6: Quantum Simulation
fn tutorial_6_simulation() -> Result<()> {
    println!("📚 Tutorial 6: Quantum Simulation");
    println!("==================================\n");

    println!("MyQuat provides high-performance quantum simulators.\n");

    // Create a Bell state circuit
    let mut bell_circuit = QuantumCircuit::new(2, 2);
    bell_circuit.h(0)?;
    bell_circuit.cx(0, 1)?;
    bell_circuit.measure_all()?;

    println!("🔔 Bell State Circuit for Simulation:");
    println!("{}", CircuitVisualizer::to_ascii_art(&bell_circuit));

    // State vector simulation
    println!("🌊 State Vector Simulation:");
    let mut simulator = StateVectorSimulator::new(2, 2);

    println!("   let mut simulator = StateVectorSimulator::new(2, 2);");
    println!("   // Simulator executes quantum circuits step by step");
    println!("   // Results show measurement outcomes and probabilities");

    // Analyze measurement statistics
    println!("\n📊 Measurement Analysis:");
    println!("   Bell states should show 50% |00⟩ and 50% |11⟩");
    println!("   This demonstrates quantum entanglement!");
    println!("   - When qubit 0 is measured as |0⟩, qubit 1 is always |0⟩");
    println!("   - When qubit 0 is measured as |1⟩, qubit 1 is always |1⟩");

    // Show expected results
    println!("\n📈 Expected Bell State Results:");
    println!("   |00⟩: ~50% probability");
    println!("   |11⟩: ~50% probability");
    println!("   |01⟩: ~0% probability (forbidden by entanglement)");
    println!("   |10⟩: ~0% probability (forbidden by entanglement)");

    println!("\n🎯 Key concepts learned:");
    println!("   - StateVectorSimulator for exact quantum simulation");
    println!("   - run(circuit, shots) executes circuit multiple times");
    println!("   - MeasurementStatistics for analyzing results");
    println!("   - Histogram analysis of measurement outcomes\n");

    Ok(())
}

/// Tutorial 7: Noise and Error Mitigation
fn tutorial_7_noise_and_mitigation() -> Result<()> {
    println!("📚 Tutorial 7: Noise Models and Error Mitigation");
    println!("================================================\n");

    println!("Real quantum devices have noise. MyQuat simulates this!\n");

    // Create a test circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.ry(0, Parameter::Float(PI / 4.0))?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;

    println!("🧪 Test Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    // Noisy simulation
    println!("🔊 Noisy Quantum Simulation:");
    let noise_model = DeviceNoiseModel::realistic_device(2);
    let _noisy_sim = NoisyQuantumSimulator::new(2, noise_model);

    println!("   let noise_model = DeviceNoiseModel::realistic_device();");
    println!("   let mut noisy_sim = NoisyQuantumSimulator::new(2, noise_model);");

    // Error mitigation
    println!("\n🛡️ Zero-Noise Extrapolation (ZNE):");
    let _zne = ZeroNoiseExtrapolation::new();
    let _observable = Observable::PauliZ(0);

    println!("   let zne = ZeroNoiseExtrapolation::new();");
    println!("   let observable = Observable::PauliZ(0);");
    println!("   // ZNE mitigates errors by extrapolating to zero noise");

    // Symmetry verification
    println!("\n⚖️ Symmetry Verification:");
    let mut symmetry = SymmetryVerification::new();
    symmetry.add_symmetry(
        "parity".to_string(),
        vec![PauliOperator::Z, PauliOperator::Z],
        1.0,
    );

    println!("   let mut symmetry = SymmetryVerification::new();");
    println!("   symmetry.add_symmetry(\"parity\", [Z⊗Z], 1.0);");
    println!("   // Detects errors by checking quantum symmetries");

    println!("\n🎯 Key concepts learned:");
    println!("   - DeviceNoiseModel simulates realistic quantum noise");
    println!("   - NoisyQuantumSimulator for NISQ device simulation");
    println!("   - ZeroNoiseExtrapolation mitigates systematic errors");
    println!("   - SymmetryVerification detects symmetry violations");
    println!("   - Error mitigation improves algorithm accuracy\n");

    Ok(())
}

/// Tutorial 8: Advanced Features
fn tutorial_8_advanced_features() -> Result<()> {
    println!("📚 Tutorial 8: Advanced Features");
    println!("================================\n");

    println!("MyQuat includes many advanced features for research and development.\n");

    // Circuit optimization
    println!("⚡ Circuit Optimization:");
    let mut unoptimized = QuantumCircuit::new(2, 0);
    unoptimized.i(0)?; // Identity gates
    unoptimized.x(0)?;
    unoptimized.x(0)?; // Two X gates cancel out
    unoptimized.s(0)?;
    unoptimized.sdg(0)?; // S and S-dagger cancel out

    println!("Before optimization:");
    println!("{}", CircuitVisualizer::to_ascii_art(&unoptimized));

    let config = OptimizationConfig::default();
    let mut optimizer = CircuitOptimizer::new(config);
    let optimized = optimizer.optimize(&unoptimized)?;

    println!("After optimization:");
    println!("{}", CircuitVisualizer::to_ascii_art(&optimized));
    println!(
        "   Removed {} redundant gates!",
        unoptimized.size() - optimized.size()
    );

    // Device topology
    println!("\n🗺️ Device Topology and Routing:");
    let topology = DeviceTopology::grid(2, 2); // 2x2 grid
    let _router = CircuitRouter::new(topology);

    println!("   Device topology constrains which qubits can interact");
    println!("   CircuitRouter automatically inserts SWAP gates");
    println!("   Supports IBM, Google, and custom device topologies");

    // Performance features
    println!("\n🚀 Performance Features:");
    println!("   - SIMD acceleration for vector operations");
    println!("   - Parallel processing for multi-qubit operations");
    println!("   - Memory pooling to reduce allocations");
    println!("   - GPU acceleration (optional)");
    println!("   - Optimized data structures for large circuits");

    // Backend integration
    println!("\n🔌 Backend Integration:");
    println!("   - IBM Quantum platform support");
    println!("   - QASM import/export for interoperability");
    println!("   - Hardware validation and constraint checking");
    println!("   - Job submission and result retrieval");

    // Benchmarking
    println!("\n📊 Built-in Benchmarking:");
    let _benchmark_suite = BenchmarkSuite::new();
    println!("   let suite = BenchmarkSuite::new();");
    println!("   // Comprehensive performance testing");
    println!("   // Regression detection and CI integration");
    println!("   // Comprehensive performance testing and analysis");

    println!("\n🎯 Key concepts learned:");
    println!("   - CircuitOptimizer removes redundant gates");
    println!("   - DeviceTopology models hardware constraints");
    println!("   - Performance features for high-speed simulation");
    println!("   - Backend integration for real quantum devices");
    println!("   - Built-in benchmarking for performance analysis\n");

    Ok(())
}
