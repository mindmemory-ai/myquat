//! # Beginner's Tutorial for Quantum Computing
//!
//! This tutorial provides a gentle introduction to quantum computing concepts
//! using MyQuat library, designed for complete beginners.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🎓 Welcome to Quantum Computing with MyQuat!");
    println!("============================================\n");

    lesson_1_quantum_bits()?;
    lesson_2_quantum_gates()?;
    lesson_3_superposition()?;
    lesson_4_entanglement()?;
    lesson_5_measurement()?;
    lesson_6_first_algorithm()?;
    lesson_7_practical_applications()?;

    println!("🎉 Congratulations! You've completed the beginner's tutorial!");
    println!("Next steps:");
    println!("- Try the interactive tutorial: cargo run --example interactive_tutorial");
    println!("- Explore quantum algorithms: cargo run --example shor_algorithm");
    println!("- Learn about quantum ML: cargo run --example quantum_machine_learning");

    Ok(())
}

/// Lesson 1: Understanding Quantum Bits (Qubits)
fn lesson_1_quantum_bits() -> Result<()> {
    println!("📚 Lesson 1: Understanding Quantum Bits (Qubits)");
    println!("=================================================\n");

    println!("🔬 What is a Qubit?");
    println!("A qubit is the basic unit of quantum information, like a classical bit");
    println!("but with quantum properties that make it much more powerful.\n");

    println!("📊 Classical vs Quantum Bits:");
    println!("┌─────────────┬─────────────┬─────────────┐");
    println!("│    Type     │   States    │ Information │");
    println!("├─────────────┼─────────────┼─────────────┤");
    println!("│ Classical   │ 0 or 1      │ 1 bit       │");
    println!("│ Quantum     │ |0⟩, |1⟩, or│ Infinite    │");
    println!("│ (Qubit)     │ both at once│ possibilities│");
    println!("└─────────────┴─────────────┴─────────────┘\n");

    // Create a simple single-qubit circuit
    let mut circuit = QuantumCircuit::new(1, 1);
    println!("🔧 Creating your first quantum circuit:");
    println!("   let mut circuit = QuantumCircuit::new(1, 1);");
    println!("   // 1 qubit, 1 classical bit for measurement\n");

    // Show the initial state
    println!("🎯 Initial State:");
    println!("Our qubit starts in state |0⟩ (like classical bit 0)");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));

    println!("💡 Key Concepts:");
    println!("- Qubits are quantum mechanical systems");
    println!("- They can exist in 'superposition' of 0 and 1");
    println!("- This gives quantum computers their power");
    println!("- We use Dirac notation: |0⟩ and |1⟩ for quantum states\n");

    Ok(())
}

/// Lesson 2: Quantum Gates - The Building Blocks
fn lesson_2_quantum_gates() -> Result<()> {
    println!("📚 Lesson 2: Quantum Gates - The Building Blocks");
    println!("================================================\n");

    println!("🚪 What are Quantum Gates?");
    println!("Quantum gates are operations that change the state of qubits,");
    println!("similar to logic gates in classical computers.\n");

    // Demonstrate basic gates
    println!("🔧 Basic Quantum Gates:\n");

    // X Gate (NOT gate)
    println!("1. X Gate (Quantum NOT):");
    let mut x_circuit = QuantumCircuit::new(1, 1);
    x_circuit.x(0)?;
    x_circuit.measure(0, 0)?;
    println!("   Flips |0⟩ → |1⟩ and |1⟩ → |0⟩");
    println!("   {}", CircuitVisualizer::to_ascii_art(&x_circuit));

    // H Gate (Hadamard)
    println!("2. H Gate (Hadamard - Creates Superposition):");
    let mut h_circuit = QuantumCircuit::new(1, 1);
    h_circuit.h(0)?;
    h_circuit.measure(0, 0)?;
    println!("   Creates equal superposition: |0⟩ + |1⟩");
    println!("   {}", CircuitVisualizer::to_ascii_art(&h_circuit));

    // Z Gate
    println!("3. Z Gate (Phase Flip):");
    let mut z_circuit = QuantumCircuit::new(1, 1);
    z_circuit.z(0)?;
    z_circuit.measure(0, 0)?;
    println!("   Adds a phase: |1⟩ → -|1⟩ (invisible until interference)");
    println!("   {}", CircuitVisualizer::to_ascii_art(&z_circuit));

    println!("💡 Important Properties:");
    println!("- Quantum gates are reversible (unlike classical gates)");
    println!("- They preserve the 'norm' of quantum states");
    println!("- Gates can create and manipulate superposition");
    println!("- Multiple gates can be combined to create complex operations\n");

    Ok(())
}

/// Lesson 3: Superposition - Being in Multiple States
fn lesson_3_superposition() -> Result<()> {
    println!("📚 Lesson 3: Superposition - Being in Multiple States");
    println!("=====================================================\n");

    println!("🌊 What is Superposition?");
    println!("Superposition allows a qubit to be in a combination of |0⟩ and |1⟩");
    println!("simultaneously, until we measure it.\n");

    // Create superposition with Hadamard gate
    let mut superposition_circuit = QuantumCircuit::new(1, 1);
    superposition_circuit.h(0)?;

    println!("🔧 Creating Superposition:");
    println!("Starting with |0⟩, apply Hadamard gate:");
    println!(
        "{}",
        CircuitVisualizer::to_ascii_art(&superposition_circuit)
    );

    println!("📊 Mathematical Representation:");
    println!("H|0⟩ = (|0⟩ + |1⟩)/√2");
    println!("This means:");
    println!("- 50% probability of measuring |0⟩");
    println!("- 50% probability of measuring |1⟩");
    println!("- But the qubit is in BOTH states before measurement!\n");

    // Demonstrate measurement
    superposition_circuit.measure(0, 0)?;
    println!("🎲 After Measurement:");
    println!(
        "{}",
        CircuitVisualizer::to_ascii_art(&superposition_circuit)
    );
    println!("The superposition collapses to either |0⟩ or |1⟩\n");

    // Multiple qubits in superposition
    println!("🚀 Multiple Qubits in Superposition:");
    let mut multi_super = QuantumCircuit::new(2, 2);
    multi_super.h(0)?;
    multi_super.h(1)?;
    multi_super.measure_all()?;

    println!("Two qubits in superposition:");
    println!("{}", CircuitVisualizer::to_ascii_art(&multi_super));
    println!("Creates 4 possible states: |00⟩, |01⟩, |10⟩, |11⟩");
    println!("Each with 25% probability!\n");

    println!("💡 Why Superposition Matters:");
    println!("- Classical 2 bits: can be in 1 of 4 states at a time");
    println!("- Quantum 2 qubits: can be in ALL 4 states simultaneously");
    println!("- This exponential scaling gives quantum computers their power");
    println!("- n qubits can represent 2^n states at once!\n");

    Ok(())
}

/// Lesson 4: Entanglement - Spooky Action at a Distance
fn lesson_4_entanglement() -> Result<()> {
    println!("📚 Lesson 4: Entanglement - Spooky Action at a Distance");
    println!("=======================================================\n");

    println!("🔗 What is Entanglement?");
    println!("Entanglement creates a special connection between qubits where");
    println!("measuring one instantly affects the other, no matter the distance!\n");

    // Create Bell state (maximally entangled state)
    println!("🔧 Creating Entanglement - The Bell State:");
    let mut bell_circuit = QuantumCircuit::new(2, 2);

    println!("Step 1: Start with |00⟩");
    println!("q[0] ─────────");
    println!("q[1] ─────────\n");

    bell_circuit.h(0)?;
    println!("Step 2: Apply Hadamard to first qubit");
    println!("q[0] ─H───────  (creates |0⟩+|1⟩)/√2");
    println!("q[1] ─────────  (still |0⟩)\n");

    bell_circuit.cx(0, 1)?;
    println!("Step 3: Apply CNOT gate (Controlled-X)");
    println!("q[0] ─H──●────  (control qubit)");
    println!("q[1] ─────⊕───  (target qubit)\n");

    println!("🎯 Result: Bell State |Φ+⟩ = (|00⟩ + |11⟩)/√2");
    println!("This means:");
    println!("- 50% chance of measuring |00⟩ (both qubits are 0)");
    println!("- 50% chance of measuring |11⟩ (both qubits are 1)");
    println!("- 0% chance of measuring |01⟩ or |10⟩");
    println!("- The qubits are perfectly correlated!\n");

    bell_circuit.measure_all()?;
    println!("🔬 Complete Bell State Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&bell_circuit));

    println!("🌟 Amazing Properties of Entanglement:");
    println!("1. **Instant Correlation**: If you measure the first qubit as |0⟩,");
    println!("   the second qubit will ALWAYS be |0⟩ too!");
    println!("2. **Distance Independent**: This works even if qubits are light-years apart");
    println!("3. **No Communication**: No information travels between qubits");
    println!("4. **Quantum Resource**: Essential for many quantum algorithms\n");

    println!("🧪 Try This Thought Experiment:");
    println!("- Alice has qubit 0, Bob has qubit 1 (on different planets)");
    println!("- Alice measures her qubit and gets |0⟩");
    println!("- Instantly, Bob's qubit becomes |0⟩ too!");
    println!("- This is what Einstein called 'spooky action at a distance'\n");

    Ok(())
}

/// Lesson 5: Measurement - Collapsing the Quantum World
fn lesson_5_measurement() -> Result<()> {
    println!("📚 Lesson 5: Measurement - Collapsing the Quantum World");
    println!("=======================================================\n");

    println!("📏 What Happens When We Measure?");
    println!("Measurement is the bridge between the quantum and classical worlds.");
    println!("It forces qubits to 'choose' a definite state.\n");

    // Demonstrate measurement effects
    println!("🔬 The Measurement Process:\n");

    println!("Before Measurement:");
    println!("Qubit in superposition: (|0⟩ + |1⟩)/√2");
    println!("- Probability of |0⟩: 50%");
    println!("- Probability of |1⟩: 50%");
    println!("- Actual state: BOTH |0⟩ AND |1⟩\n");

    println!("During Measurement:");
    println!("The quantum state 'collapses' randomly to either |0⟩ or |1⟩");
    println!("- This is truly random (not just unknown)");
    println!("- The superposition is destroyed");
    println!("- We get a classical bit: 0 or 1\n");

    // Show measurement in different bases
    println!("🎯 Measurement Bases:");

    // Z-basis measurement (computational basis)
    let mut z_measure = QuantumCircuit::new(1, 1);
    z_measure.h(0)?;
    z_measure.measure(0, 0)?;

    println!("1. Z-basis (Computational Basis) - Default:");
    println!("   Measures |0⟩ vs |1⟩");
    println!("   {}", CircuitVisualizer::to_ascii_art(&z_measure));

    // X-basis measurement
    let mut x_measure = QuantumCircuit::new(1, 1);
    x_measure.h(0)?;
    x_measure.h(0)?; // H gate before measurement rotates to X-basis
    x_measure.measure(0, 0)?;

    println!("2. X-basis Measurement:");
    println!("   Measures |+⟩ vs |-⟩ (H before measurement)");
    println!("   {}", CircuitVisualizer::to_ascii_art(&x_measure));

    println!("🎲 Probabilistic Nature:");
    println!("Quantum mechanics is fundamentally probabilistic:");
    println!("- We can predict probabilities perfectly");
    println!("- Individual outcomes are truly random");
    println!("- Many measurements reveal the probability distribution\n");

    // Demonstrate statistics
    println!("📊 Statistical Example:");
    println!("If we run the Bell state circuit 1000 times:");
    println!("┌─────────┬─────────┬─────────┐");
    println!("│ Outcome │  Count  │ Percent │");
    println!("├─────────┼─────────┼─────────┤");
    println!("│   |00⟩  │  ~500   │  ~50%   │");
    println!("│   |01⟩  │   ~0    │   ~0%   │");
    println!("│   |10⟩  │   ~0    │   ~0%   │");
    println!("│   |11⟩  │  ~500   │  ~50%   │");
    println!("└─────────┴─────────┴─────────┘\n");

    println!("💡 Key Insights:");
    println!("- Measurement destroys superposition");
    println!("- Results are probabilistic, not deterministic");
    println!("- Choice of measurement basis affects results");
    println!("- Quantum algorithms use interference to bias probabilities\n");

    Ok(())
}

/// Lesson 6: Your First Quantum Algorithm
fn lesson_6_first_algorithm() -> Result<()> {
    println!("📚 Lesson 6: Your First Quantum Algorithm");
    println!("==========================================\n");

    println!("🚀 Let's Build a Quantum Random Number Generator!");
    println!("This simple algorithm demonstrates core quantum principles.\n");

    println!("🎯 Algorithm Goal:");
    println!("Generate truly random numbers using quantum superposition");
    println!("(Much better than classical pseudo-random generators!)\n");

    // Single-bit random number generator
    println!("🔧 Step-by-Step Construction:\n");

    println!("Version 1: Single Random Bit");
    let mut single_rng = QuantumCircuit::new(1, 1);

    println!("1. Start with |0⟩:");
    println!("   q[0] ─────");

    single_rng.h(0)?;
    println!("2. Apply Hadamard (create superposition):");
    println!("   q[0] ─H───  → (|0⟩ + |1⟩)/√2");

    single_rng.measure(0, 0)?;
    println!("3. Measure the qubit:");
    println!("   q[0] ─H─M─  → Random 0 or 1");

    println!("\n🎨 Complete Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&single_rng));

    // Multi-bit random number generator
    println!("Version 2: Multi-Bit Random Number (0-15)");
    let mut multi_rng = QuantumCircuit::new(4, 4);

    for i in 0..4 {
        multi_rng.h(i)?;
    }
    multi_rng.measure_all()?;

    println!("Using 4 qubits for 4-bit random numbers:");
    println!("{}", CircuitVisualizer::to_ascii_art(&multi_rng));

    println!("📊 This generates random numbers from 0 to 15:");
    println!("- |0000⟩ = 0,  |0001⟩ = 1,  |0010⟩ = 2,  |0011⟩ = 3");
    println!("- |0100⟩ = 4,  |0101⟩ = 5,  |0110⟩ = 6,  |0111⟩ = 7");
    println!("- |1000⟩ = 8,  |1001⟩ = 9,  |1010⟩ = 10, |1011⟩ = 11");
    println!("- |1100⟩ = 12, |1101⟩ = 13, |1110⟩ = 14, |1111⟩ = 15\n");

    println!("🌟 Why This is Special:");
    println!("✅ Truly random (not pseudo-random)");
    println!("✅ Based on quantum mechanics");
    println!("✅ Unpredictable even in principle");
    println!("✅ Perfect for cryptography and simulations\n");

    println!("🔍 Algorithm Analysis:");
    let viz = CircuitVisualizer::new();
    println!("{}", viz.statistics_table(&multi_rng));

    println!("💡 What You've Learned:");
    println!("- How to create quantum superposition");
    println!("- How measurement collapses superposition");
    println!("- How to scale quantum algorithms");
    println!("- The power of quantum parallelism\n");

    Ok(())
}

/// Lesson 7: Practical Applications
fn lesson_7_practical_applications() -> Result<()> {
    println!("📚 Lesson 7: Practical Applications of Quantum Computing");
    println!("========================================================\n");

    println!("🌍 Where Quantum Computing Makes a Difference:");
    println!("Quantum computers aren't just theoretical - they're solving real problems!\n");

    println!("🔐 1. Cryptography and Security:");
    println!("   • Breaking RSA encryption (Shor's algorithm)");
    println!("   • Creating unbreakable quantum keys");
    println!("   • Secure communication protocols");
    println!("   • Random number generation for security\n");

    println!("💊 2. Drug Discovery and Chemistry:");
    println!("   • Simulating molecular interactions");
    println!("   • Finding new medicines faster");
    println!("   • Understanding chemical reactions");
    println!("   • Designing new materials\n");

    println!("📈 3. Financial Modeling:");
    println!("   • Risk analysis and portfolio optimization");
    println!("   • High-frequency trading strategies");
    println!("   • Fraud detection algorithms");
    println!("   • Market prediction models\n");

    println!("🤖 4. Machine Learning and AI:");
    println!("   • Quantum neural networks");
    println!("   • Pattern recognition in big data");
    println!("   • Optimization problems");
    println!("   • Feature selection and classification\n");

    println!("🚗 5. Optimization Problems:");
    println!("   • Traffic flow optimization");
    println!("   • Supply chain management");
    println!("   • Scheduling and logistics");
    println!("   • Energy distribution\n");

    println!("🔬 Current State of Quantum Computing:");
    println!("┌─────────────────┬─────────────────┬─────────────────┐");
    println!("│      Era        │   Technology    │   Applications  │");
    println!("├─────────────────┼─────────────────┼─────────────────┤");
    println!("│ NISQ (Now)      │ 50-1000 qubits │ Research, demos │");
    println!("│ Fault-Tolerant  │ 1M+ qubits     │ Breaking RSA    │");
    println!("│ (Future)        │                 │ Drug discovery  │");
    println!("└─────────────────┴─────────────────┴─────────────────┘\n");

    println!("🎯 Famous Quantum Algorithms You Can Explore:");

    // Show a simple Deutsch algorithm
    println!("Example: Deutsch Algorithm (Quantum vs Classical)");
    let mut deutsch = QuantumCircuit::new(2, 1);
    deutsch.x(1)?; // Ancilla qubit
    deutsch.h(0)?; // Input qubit
    deutsch.h(1)?; // Ancilla qubit

    // Oracle (simplified - would depend on function)
    deutsch.cx(0, 1)?;

    deutsch.h(0)?; // Final Hadamard
    deutsch.measure(0, 0)?;

    println!("{}", CircuitVisualizer::to_ascii_art(&deutsch));
    println!("This algorithm determines if a function is constant or balanced");
    println!("in just ONE query (classical needs 2 queries)!\n");

    println!("🚀 Next Steps in Your Quantum Journey:");
    println!("1. 📖 Study quantum algorithms (Grover, Shor, VQE)");
    println!("2. 🧪 Experiment with quantum simulators");
    println!("3. 💻 Try quantum programming languages (Qiskit, Cirq)");
    println!("4. 🏫 Take quantum computing courses");
    println!("5. 🔬 Join quantum computing communities");
    println!("6. 🏢 Explore quantum computing careers\n");

    println!("🌟 Remember:");
    println!("- Quantum computing is still evolving rapidly");
    println!("- Every expert was once a beginner");
    println!("- The field needs diverse perspectives and skills");
    println!("- You're entering at an exciting time in history!\n");

    Ok(())
}
