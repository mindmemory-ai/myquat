//! Device topology and routing demo
//!
//! This example demonstrates device topology modeling, circuit validation,
//! and automatic routing for quantum hardware constraints.

use myquat::*;

fn main() -> Result<()> {
    println!("🔗 Device Topology and Circuit Routing Demo");
    println!("==========================================\n");

    demo_topology_types()?;
    demo_connectivity_analysis()?;
    demo_circuit_validation()?;
    demo_circuit_routing()?;
    demo_real_device_topologies()?;

    println!("✅ Device topology demo completed successfully!");
    Ok(())
}

/// Demo 1: Different topology types
fn demo_topology_types() -> Result<()> {
    println!("🔧 Demo 1: Device Topology Types");
    println!("---------------------------------");

    // Linear topology (chain)
    let linear = DeviceTopology::linear(5);
    println!("📏 Linear Topology (5 qubits):");
    println!("  Connections: 0-1-2-3-4");
    println!("  Total edges: {}", linear.edges().len());
    println!("  Distance 0→4: {} hops", linear.distance(0, 4).unwrap());

    // Grid topology (2D lattice)
    let grid = DeviceTopology::grid(2, 3);
    println!("\n🔲 Grid Topology (2×3):");
    println!("  Layout:");
    println!("    0 - 1 - 2");
    println!("    |   |   |");
    println!("    3 - 4 - 5");
    println!("  Total edges: {}", grid.edges().len());
    println!("  Distance 0→5: {} hops", grid.distance(0, 5).unwrap());

    // Heavy-hex topology (IBM-style)
    let heavy_hex = DeviceTopology::heavy_hex(2);
    println!("\n⬡ Heavy-Hex Topology:");
    println!("  Qubits: {}", heavy_hex.num_qubits);
    println!("  Total edges: {}", heavy_hex.edges().len());
    println!("  Connectivity: High (IBM-style)");

    // Fully connected
    let fully_connected = DeviceTopology::fully_connected(4);
    println!("\n🌐 Fully Connected Topology (4 qubits):");
    println!("  All-to-all connectivity");
    println!("  Total edges: {}", fully_connected.edges().len());
    println!("  Max distance: 1 hop");

    println!();
    Ok(())
}

/// Demo 2: Connectivity analysis
fn demo_connectivity_analysis() -> Result<()> {
    println!("🔧 Demo 2: Connectivity Analysis");
    println!("---------------------------------");

    let topology = DeviceTopology::grid(3, 3); // 3×3 grid

    println!("📊 3×3 Grid Topology Analysis:");
    println!("  Layout:");
    println!("    0 - 1 - 2");
    println!("    |   |   |");
    println!("    3 - 4 - 5");
    println!("    |   |   |");
    println!("    6 - 7 - 8");

    // Analyze connectivity patterns
    println!("\n🔍 Connectivity Analysis:");

    // Corner qubits (degree 2)
    let corners = [0, 2, 6, 8];
    for &corner in &corners {
        let neighbors = topology.neighbors(corner);
        println!(
            "  Corner qubit {}: {} neighbors {:?}",
            corner,
            neighbors.len(),
            neighbors
        );
    }

    // Edge qubits (degree 3)
    let edges = [1, 3, 5, 7];
    for &edge in &edges {
        let neighbors = topology.neighbors(edge);
        println!(
            "  Edge qubit {}: {} neighbors {:?}",
            edge,
            neighbors.len(),
            neighbors
        );
    }

    // Center qubit (degree 4)
    let center = 4;
    let neighbors = topology.neighbors(center);
    println!(
        "  Center qubit {}: {} neighbors {:?}",
        center,
        neighbors.len(),
        neighbors
    );

    // Distance analysis
    println!("\n📏 Distance Matrix (selected pairs):");
    let test_pairs = [(0, 8), (0, 4), (1, 7), (2, 6)];
    for (q1, q2) in test_pairs {
        let distance = topology.distance(q1, q2).unwrap();
        let path = topology.shortest_path(q1, q2).unwrap();
        println!("  {} → {}: {} hops, path: {:?}", q1, q2, distance, path);
    }

    println!();
    Ok(())
}

/// Demo 3: Circuit validation against topology
fn demo_circuit_validation() -> Result<()> {
    println!("🔧 Demo 3: Circuit Validation");
    println!("------------------------------");

    let topology = DeviceTopology::linear(4); // Linear: 0-1-2-3

    println!("📋 Testing circuits on linear topology (0-1-2-3):");

    // Valid circuit
    println!("\n✅ Valid Circuit:");
    let mut valid_circuit = QuantumCircuit::new(4, 0);
    valid_circuit.h(0)?;
    valid_circuit.cx(0, 1)?; // Adjacent qubits
    valid_circuit.cx(1, 2)?; // Adjacent qubits
    valid_circuit.cx(2, 3)?; // Adjacent qubits

    println!("  Gates: H(0), CNOT(0,1), CNOT(1,2), CNOT(2,3)");
    let result = topology.validate_circuit(&valid_circuit);
    println!(
        "  Validation: {}",
        if result.is_valid { "PASSED" } else { "FAILED" }
    );

    // Invalid circuit
    println!("\n❌ Invalid Circuit:");
    let mut invalid_circuit = QuantumCircuit::new(4, 0);
    invalid_circuit.h(0)?;
    invalid_circuit.cx(0, 2)?; // Non-adjacent qubits!
    invalid_circuit.cx(1, 3)?; // Non-adjacent qubits!

    println!("  Gates: H(0), CNOT(0,2), CNOT(1,3)");
    let result = topology.validate_circuit(&invalid_circuit);
    println!(
        "  Validation: {}",
        if result.is_valid { "PASSED" } else { "FAILED" }
    );

    if !result.violations.is_empty() {
        println!("  Violations found:");
        for violation in &result.violations {
            println!("    • {}: {}", violation.gate, violation.message);
        }
    }

    // Mixed circuit
    println!("\n⚠️  Mixed Circuit:");
    let mut mixed_circuit = QuantumCircuit::new(4, 0);
    mixed_circuit.h(0)?;
    mixed_circuit.cx(0, 1)?; // Valid
    mixed_circuit.cx(0, 3)?; // Invalid - not adjacent
    mixed_circuit.cx(2, 3)?; // Valid

    println!("  Gates: H(0), CNOT(0,1), CNOT(0,3), CNOT(2,3)");
    let result = topology.validate_circuit(&mixed_circuit);
    println!(
        "  Validation: {}",
        if result.is_valid { "PASSED" } else { "FAILED" }
    );
    println!("  Valid gates: 3/4, Invalid gates: 1/4");

    println!();
    Ok(())
}

/// Demo 4: Automatic circuit routing
fn demo_circuit_routing() -> Result<()> {
    println!("🔧 Demo 4: Automatic Circuit Routing");
    println!("-------------------------------------");

    let topology = DeviceTopology::linear(5); // Linear: 0-1-2-3-4
    let router = CircuitRouter::new(topology);

    println!("🛣️  Routing circuits for linear topology (0-1-2-3-4):");

    // Simple routing case
    println!("\n📋 Original Circuit:");
    let mut original = QuantumCircuit::new(3, 0);
    original.h(0)?;
    original.cx(0, 2)?; // This needs routing (0 and 2 not adjacent)
    original.h(2)?;

    println!("  Gates: H(0), CNOT(0,2), H(2)");
    println!("  Problem: CNOT(0,2) - qubits 0 and 2 are not adjacent");

    println!("\n🔄 Routed Circuit:");
    let routed = router.route_circuit(&original)?;
    println!("  Original gates: {}", original.size());
    println!("  Routed gates: {}", routed.size());
    println!("  Additional SWAP gates inserted for routing");

    // Complex routing case
    println!("\n📋 Complex Circuit:");
    let mut complex = QuantumCircuit::new(4, 0);
    complex.h(0)?;
    complex.h(3)?;
    complex.cx(0, 3)?; // Long distance
    complex.cx(1, 2)?; // Adjacent
    complex.cx(0, 2)?; // Medium distance

    println!("  Gates: H(0), H(3), CNOT(0,3), CNOT(1,2), CNOT(0,2)");
    println!("  Multiple routing challenges");

    let routed_complex = router.route_circuit(&complex)?;
    println!("\n🔄 Routed Complex Circuit:");
    println!("  Original gates: {}", complex.size());
    println!("  Routed gates: {}", routed_complex.size());
    println!(
        "  Routing overhead: {}%",
        ((routed_complex.size() as f64 / complex.size() as f64) - 1.0) * 100.0
    );

    println!();
    Ok(())
}

/// Demo 5: Real device topologies
fn demo_real_device_topologies() -> Result<()> {
    println!("🔧 Demo 5: Real Device Topologies");
    println!("----------------------------------");

    println!("🏭 Simulating Real Quantum Devices:");

    // IBM-style devices
    println!("\n🔵 IBM Quantum Devices:");

    let ibm_5q = DeviceTopology::linear(5);
    println!("  • ibmq_lima (5-qubit linear):");
    println!("    Topology: Linear chain");
    println!("    Connectivity: {}", ibm_5q.edges().len());
    println!("    Max distance: {} hops", ibm_5q.distance(0, 4).unwrap());

    let ibm_16q = DeviceTopology::heavy_hex(2);
    println!("  • ibmq_guadalupe (16-qubit heavy-hex):");
    println!("    Topology: Heavy-hex lattice");
    println!("    Connectivity: {}", ibm_16q.edges().len());
    println!("    Highly connected for efficient routing");

    let ibm_27q = DeviceTopology::heavy_hex(3);
    println!("  • ibmq_montreal (27-qubit heavy-hex):");
    println!("    Topology: Extended heavy-hex");
    println!("    Connectivity: {}", ibm_27q.edges().len());
    println!("    Enterprise-grade connectivity");

    // Google-style devices
    println!("\n🟡 Google Quantum Devices:");

    let google_54q = DeviceTopology::grid(6, 9);
    println!("  • Sycamore (54-qubit 2D grid):");
    println!("    Topology: 2D rectangular lattice");
    println!("    Connectivity: {}", google_54q.edges().len());
    println!("    Optimized for 2D algorithms");

    // Comparison analysis
    println!("\n📊 Topology Comparison:");
    let topologies = [
        ("Linear-5", ibm_5q),
        ("Heavy-Hex-16", ibm_16q),
        ("Grid-6x9", google_54q),
    ];

    println!("  Device        | Qubits | Edges | Avg Degree | Max Distance");
    println!("  --------------|--------|-------|------------|-------------");

    for (name, topology) in &topologies {
        let qubits = topology.num_qubits;
        let edges = topology.edges().len();
        let avg_degree = (2.0 * edges as f64) / qubits as f64;

        // Calculate max distance (diameter)
        let mut max_distance = 0;
        for i in 0..qubits {
            for j in i + 1..qubits {
                if let Some(dist) = topology.distance(i, j) {
                    max_distance = max_distance.max(dist);
                }
            }
        }

        println!(
            "  {:12} | {:6} | {:5} | {:10.1} | {:11}",
            name, qubits, edges, avg_degree, max_distance
        );
    }

    println!("\n💡 Insights:");
    println!("  • Linear topologies: Simple but limited connectivity");
    println!("  • Heavy-hex: Balanced connectivity and routing efficiency");
    println!("  • Grid topologies: Natural for 2D quantum algorithms");
    println!("  • Higher connectivity → Better routing → Lower overhead");

    println!();
    Ok(())
}
