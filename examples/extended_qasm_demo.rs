//! Extended Gate QASM Export Demo
//!
//! This example demonstrates the extended QASM export capabilities,
//! showing how to export circuits with extended gates to OpenQASM format.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🎯 Extended Gate QASM Export Demo");
    println!("==================================\n");

    // Create a circuit with extended gates
    let mut circuit = QuantumCircuit::new(3, 0);
    circuit.set_name("Extended Gate Demo".to_string());

    // Add some standard gates
    circuit.h(0)?;
    circuit.cx(0, 1)?;

    // Add rotation gates with parameters
    circuit.rx(1, Parameter::new_float(PI / 4.0))?;
    circuit.ry(2, Parameter::new_float(PI / 3.0))?;

    println!("📋 Created circuit with {} gates", circuit.size());
    println!("Circuit depth: {}", circuit.depth());
    println!();

    // Demo 1: Export to OpenQASM 2.0 with extended gates
    println!("🔧 Demo 1: OpenQASM 2.0 Export with Extended Gates");
    println!("--------------------------------------------------");

    let config_v2 = QasmConfig {
        version: QasmVersion::V2_0,
        include_extended_gates: true,
        include_comments: true,
        precision: 6,
        ..QasmConfig::default()
    };

    let exporter_v2 = QasmExporter::with_config(config_v2);
    let qasm_v2 = exporter_v2.export(&circuit)?;

    println!("Generated OpenQASM 2.0:");
    println!("{}", qasm_v2);

    // Demo 2: Export to OpenQASM 3.0 with extended gates
    println!("🔧 Demo 2: OpenQASM 3.0 Export with Extended Gates");
    println!("--------------------------------------------------");

    let config_v3 = QasmConfig {
        version: QasmVersion::V3_0,
        include_extended_gates: true,
        include_comments: true,
        precision: 4,
        ..QasmConfig::default()
    };

    let exporter_v3 = QasmExporter::with_config(config_v3);
    let qasm_v3 = exporter_v3.export(&circuit)?;

    println!("Generated OpenQASM 3.0:");
    println!("{}", qasm_v3);

    // Demo 3: Extended gate conversion examples
    println!("🔧 Demo 3: Extended Gate Conversion Examples");
    println!("--------------------------------------------");

    use myquat::circuit::Qubit;
    use myquat::gates_extended::ExtendedGate;

    let exporter = QasmExporter::new();

    // Square root gates
    let sqrt_gates = vec![
        ("SqrtX", ExtendedGate::SqrtX),
        ("SqrtY", ExtendedGate::SqrtY),
        ("SqrtZ", ExtendedGate::SqrtZ),
        ("SqrtH", ExtendedGate::SqrtH),
    ];

    println!("Square Root Gates:");
    for (name, gate) in sqrt_gates {
        let qubits = vec![Qubit::new(0)];
        let qasm = exporter.extended_gate_to_qasm(&gate, &qubits)?;
        println!("  {}: {}", name, qasm);
    }
    println!();

    // Two-qubit rotation gates
    let two_qubit_gates = vec![
        ("RXX", ExtendedGate::RXX(PI / 2.0)),
        ("RYY", ExtendedGate::RYY(PI / 3.0)),
        ("RZZ", ExtendedGate::RZZ(PI / 4.0)),
        ("RZX", ExtendedGate::RZX(PI / 6.0)),
    ];

    println!("Two-Qubit Rotation Gates:");
    for (name, gate) in two_qubit_gates {
        let qubits = vec![Qubit::new(0), Qubit::new(1)];
        let qasm = exporter.extended_gate_to_qasm(&gate, &qubits)?;
        println!("  {}: {}", name, qasm);
    }
    println!();

    // Phase gates
    let phase_gates = vec![
        ("GlobalPhase", ExtendedGate::GlobalPhase(PI / 8.0)),
        ("PhaseShift", ExtendedGate::PhaseShift(PI / 5.0)),
    ];

    println!("Phase Gates:");
    for (name, gate) in phase_gates {
        let qubits = if name == "GlobalPhase" {
            vec![]
        } else {
            vec![Qubit::new(0)]
        };
        let qasm = exporter.extended_gate_to_qasm(&gate, &qubits)?;
        println!("  {}: {}", name, qasm);
    }
    println!();

    // Demo 4: Custom gate example
    println!("🔧 Demo 4: Custom Gate Export");
    println!("-----------------------------");

    let custom_gate = ExtendedGate::Custom {
        name: "my_special_gate".to_string(),
        matrix_real: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        matrix_imag: vec![vec![0.0, 0.0], vec![0.0, 0.0]],
        num_qubits: 1,
        parameters: vec![],
    };

    let qubits = vec![Qubit::new(0)];
    let custom_qasm = exporter.extended_gate_to_qasm(&custom_gate, &qubits)?;
    println!("Custom Gate: {}", custom_qasm);
    println!();

    // Demo 5: Configuration comparison
    println!("🔧 Demo 5: Configuration Comparison");
    println!("-----------------------------------");

    let test_circuit = {
        let mut c = QuantumCircuit::new(2, 2);
        c.h(0)?;
        c.rx(1, Parameter::new_float(PI / 2.0))?;
        c.cx(0, 1)?;
        c.measure(0, 0)?;
        c.measure(1, 1)?;
        c
    };

    // Without extended gates
    let config_no_ext = QasmConfig {
        include_extended_gates: false,
        include_comments: false,
        ..QasmConfig::default()
    };
    let exporter_no_ext = QasmExporter::with_config(config_no_ext);
    let qasm_no_ext = exporter_no_ext.export(&test_circuit)?;

    println!("Without Extended Gates:");
    println!("{}", qasm_no_ext);

    // With extended gates
    let config_with_ext = QasmConfig {
        include_extended_gates: true,
        include_comments: false,
        ..QasmConfig::default()
    };
    let exporter_with_ext = QasmExporter::with_config(config_with_ext);
    let qasm_with_ext = exporter_with_ext.export(&test_circuit)?;

    println!("With Extended Gates:");
    println!("{}", qasm_with_ext);

    // Demo 6: Precision control
    println!("🔧 Demo 6: Precision Control");
    println!("----------------------------");

    let param_circuit = {
        let mut c = QuantumCircuit::new(1, 0);
        c.rx(0, Parameter::new_float(PI / 7.0))?; // Irrational angle
        c
    };

    for precision in [2, 4, 8] {
        let config = QasmConfig {
            precision,
            include_comments: false,
            ..QasmConfig::default()
        };
        let exporter = QasmExporter::with_config(config);
        let qasm = exporter.export(&param_circuit)?;

        println!(
            "Precision {}: {}",
            precision,
            qasm.lines().find(|line| line.contains("rx")).unwrap_or("")
        );
    }

    println!("\n✅ Extended Gate QASM Export Demo completed successfully!");
    println!("📊 Features demonstrated:");
    println!("   • OpenQASM 2.0 and 3.0 export with extended gates");
    println!("   • Square root gates (SqrtX, SqrtY, SqrtZ, SqrtH)");
    println!("   • Two-qubit rotation gates (RXX, RYY, RZZ, RZX)");
    println!("   • Phase gates (GlobalPhase, PhaseShift)");
    println!("   • Custom gate definitions");
    println!("   • Configuration options and precision control");

    Ok(())
}
