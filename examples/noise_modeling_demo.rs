//! Noise modeling and realistic device simulation demo
//!
//! This example demonstrates the advanced noise modeling capabilities of MyQuat,
//! including decoherence, gate errors, and readout errors.

use myquat::*;

fn main() -> Result<()> {
    println!("🔬 Noise Modeling and Realistic Device Simulation Demo");
    println!("====================================================\n");

    demo_device_noise_models()?;
    demo_decoherence_channels()?;
    demo_noisy_circuit_simulation()?;
    demo_noise_characterization()?;
    demo_fidelity_comparison()?;

    println!("✅ Noise modeling demo completed successfully!");
    Ok(())
}

/// Demo 1: Device noise model creation and configuration
fn demo_device_noise_models() -> Result<()> {
    println!("🔧 Demo 1: Device Noise Model Configuration");
    println!("-------------------------------------------");

    // Create a realistic noise model for a 3-qubit device
    let mut noise_model = DeviceNoiseModel::realistic_device(3);

    println!("📊 Realistic Device Parameters:");
    for qubit in 0..3 {
        let t1 = noise_model.t1_times[&qubit];
        let t2 = noise_model.t2_times[&qubit];
        let (error_0_to_1, error_1_to_0) = noise_model.readout_errors[&qubit];

        println!(
            "  Qubit {}: T1 = {:.1} μs, T2 = {:.1} μs, Readout Error = {:.3}%",
            qubit,
            t1,
            t2,
            (error_0_to_1 + error_1_to_0) * 50.0
        );
    }

    println!("\n🚪 Gate Error Rates:");
    let important_gates = ["X", "H", "CNOT", "RZ"];
    for gate in &important_gates {
        if let Some(&error_rate) = noise_model.gate_errors.get(*gate) {
            println!("  {}: {:.4} ({:.2}%)", gate, error_rate, error_rate * 100.0);
        }
    }

    // Customize noise model
    println!("\n🔧 Customizing Noise Model:");
    noise_model.set_t1(0, 100.0); // Better T1 for qubit 0
    noise_model.set_gate_error("CNOT", 0.005); // Better CNOT fidelity

    println!("  Updated Qubit 0: T1 = {:.1} μs", noise_model.t1_times[&0]);
    println!(
        "  Updated CNOT error: {:.4}",
        noise_model.gate_errors["CNOT"]
    );

    println!();
    Ok(())
}

/// Demo 2: Individual noise channels
fn demo_decoherence_channels() -> Result<()> {
    println!("🔧 Demo 2: Decoherence and Noise Channels");
    println!("------------------------------------------");

    // Create a simple 1-qubit density matrix in |0⟩ state
    let rho = DensityMatrix::zero_state(1);

    println!("📊 Testing Different Noise Channels:");

    // Test decoherence channel
    println!("  🌊 Decoherence Channel (T1=50μs, T2=30μs, gate_time=20ns):");
    let decoherence = DecoherenceChannel::new(50.0, 30.0, 20.0);
    let mut test_rho = rho.clone();
    decoherence.apply(&mut test_rho, 0)?;
    println!("    Applied T1/T2 decoherence");

    // Test depolarizing channel
    println!("  🎲 Depolarizing Channel (p=0.01):");
    let depolarizing = DepolarizingChannel::new(0.01);
    let mut test_rho2 = rho.clone();
    depolarizing.apply(&mut test_rho2, 0)?;
    println!("    Applied depolarizing noise");

    // Test Pauli channel
    println!("  ⚡ Pauli Channel (px=py=pz=0.001):");
    let pauli = PauliChannel::new(0.001, 0.001, 0.001);
    let mut test_rho3 = rho.clone();
    pauli.apply(&mut test_rho3, 0)?;
    println!("    Applied random Pauli errors");

    // Test readout errors
    println!("  📖 Readout Error Model (2% symmetric):");
    let readout = ReadoutErrorModel::symmetric(0.02);
    println!("    Readout fidelity: {:.3}", readout.fidelity());

    // Simulate some measurements with errors
    let measurements = [true, false, true, true, false];
    print!("    Ideal measurements:  ");
    for &m in &measurements {
        print!("{} ", if m { "1" } else { "0" });
    }
    print!("\n    Noisy measurements: ");
    for &m in &measurements {
        let noisy = readout.apply_to_measurement(m);
        print!("{} ", if noisy { "1" } else { "0" });
    }
    println!();

    println!();
    Ok(())
}

/// Demo 3: Noisy circuit simulation
fn demo_noisy_circuit_simulation() -> Result<()> {
    println!("🔧 Demo 3: Noisy Quantum Circuit Simulation");
    println!("--------------------------------------------");

    // Create a Bell state circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure_all()?;

    println!("📋 Circuit: H(0) → CNOT(0,1) → Measure All");

    // Create noisy simulator
    let mut noisy_sim = NoisyQuantumSimulator::realistic_device(2);

    println!("\n🎯 Running 1000 shots with realistic noise:");
    let results = noisy_sim.run_shots(&circuit, 1000)?;

    // Display results
    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by_key(|(state, _)| *state);

    for (state, count) in sorted_results {
        let percentage = (*count as f64 / 1000.0) * 100.0;
        println!("  |{}⟩: {} shots ({:.1}%)", state, count, percentage);
    }

    println!(
        "\n⏱️  Total execution time: {:.1} ns",
        noisy_sim.execution_time()
    );

    // Compare with ideal results (should be 50% |00⟩ and 50% |11⟩)
    let ideal_00 = 50.0;
    let ideal_11 = 50.0;
    let actual_00 = results.get("00").copied().unwrap_or(0) as f64 / 10.0;
    let actual_11 = results.get("11").copied().unwrap_or(0) as f64 / 10.0;

    println!("\n📊 Comparison with Ideal:");
    println!(
        "  |00⟩: Ideal {:.1}%, Actual {:.1}%, Difference {:.1}%",
        ideal_00,
        actual_00,
        (actual_00 - ideal_00).abs()
    );
    println!(
        "  |11⟩: Ideal {:.1}%, Actual {:.1}%, Difference {:.1}%",
        ideal_11,
        actual_11,
        (actual_11 - ideal_11).abs()
    );

    println!();
    Ok(())
}

/// Demo 4: Noise characterization
fn demo_noise_characterization() -> Result<()> {
    println!("🔧 Demo 4: Device Noise Characterization");
    println!("-----------------------------------------");

    // Create a noisy simulator
    let mut simulator = NoisyQuantumSimulator::realistic_device(3);

    println!("🔬 Characterizing 3-qubit device...");

    // Perform noise characterization
    let characterization = NoiseCharacterization::characterize_device(&mut simulator, 3)?;

    // Generate and display report
    let report = characterization.generate_report();
    println!("\n{}", report);

    // Additional analysis
    println!("📈 Device Quality Metrics:");

    // Calculate average gate fidelity
    let avg_single_qubit_fidelity: f64 = ["X", "Y", "Z", "H"]
        .iter()
        .filter_map(|gate| characterization.gate_fidelities.get(*gate))
        .sum::<f64>()
        / 4.0;

    let avg_two_qubit_fidelity = characterization
        .gate_fidelities
        .get("CNOT")
        .copied()
        .unwrap_or(0.99);

    println!(
        "  Average single-qubit gate fidelity: {:.4}",
        avg_single_qubit_fidelity
    );
    println!("  Two-qubit gate fidelity: {:.4}", avg_two_qubit_fidelity);

    // Calculate average readout fidelity
    let avg_readout_fidelity: f64 = characterization.readout_fidelities.values().sum::<f64>()
        / characterization.readout_fidelities.len() as f64;
    println!("  Average readout fidelity: {:.4}", avg_readout_fidelity);

    // Calculate coherence quality factor
    let avg_t2_t1_ratio: f64 = (0..3)
        .map(|q| {
            let t1 = characterization
                .measured_t1
                .get(&q)
                .copied()
                .unwrap_or(50.0);
            let t2 = characterization
                .measured_t2
                .get(&q)
                .copied()
                .unwrap_or(25.0);
            t2 / t1
        })
        .sum::<f64>()
        / 3.0;

    println!("  Average T2/T1 ratio: {:.3}", avg_t2_t1_ratio);

    println!();
    Ok(())
}

/// Demo 5: Fidelity comparison between ideal and noisy simulation
fn demo_fidelity_comparison() -> Result<()> {
    println!("🔧 Demo 5: Fidelity Analysis");
    println!("----------------------------");

    // Create test circuits of increasing complexity
    let test_circuits = vec![
        ("Single H gate", create_single_h_circuit()),
        ("Bell state", create_bell_circuit()),
        ("3-qubit GHZ", create_ghz_circuit()),
        ("Random circuit", create_random_circuit()),
    ];

    println!("📊 Fidelity Analysis for Different Circuits:");
    println!("Circuit               | Depth | Gates | Fidelity | Error Rate");
    println!("---------------------|-------|-------|----------|----------");

    for (name, circuit) in test_circuits {
        let circuit = circuit?;

        // Calculate circuit metrics
        let depth = calculate_circuit_depth(&circuit);
        let gate_count = circuit.size();

        // Simulate ideal circuit (using density matrix without noise)
        let mut ideal_sim = DensityMatrixSimulator::new(circuit.num_qubits());
        ideal_sim.execute_circuit(&circuit)?;
        let ideal_state = ideal_sim.state().clone();

        // Simulate with noise
        let mut noisy_sim = NoisyQuantumSimulator::realistic_device(circuit.num_qubits());
        let fidelity = noisy_sim.calculate_fidelity(&circuit, &ideal_state)?;

        let error_rate = 1.0 - fidelity;

        println!(
            "{:<20} | {:>5} | {:>5} | {:>8.4} | {:>9.2}%",
            name,
            depth,
            gate_count,
            fidelity,
            error_rate * 100.0
        );
    }

    println!("\n💡 Observations:");
    println!("  • Fidelity decreases with circuit depth and gate count");
    println!("  • Two-qubit gates contribute more to fidelity loss");
    println!("  • Decoherence effects accumulate over time");

    println!();
    Ok(())
}

// Helper functions for creating test circuits

fn create_single_h_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(1, 1);
    circuit.h(0)?;
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

fn create_random_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(3, 3);

    // Add some random gates
    circuit.h(0)?;
    circuit.ry(1, Parameter::Float(std::f64::consts::PI / 4.0))?;
    circuit.cx(0, 1)?;
    circuit.rz(2, Parameter::Float(std::f64::consts::PI / 3.0))?;
    circuit.cx(1, 2)?;
    circuit.h(2)?;
    circuit.cx(0, 2)?;
    circuit.measure_all()?;

    Ok(circuit)
}

fn calculate_circuit_depth(circuit: &QuantumCircuit) -> usize {
    // Simplified depth calculation
    circuit.data().instructions().len()
}
