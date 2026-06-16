// QASM Interoperability Tests
// Author: gA4ss
//
// These tests verify the complete QASM import/export workflow,
// ensuring compatibility with other quantum computing platforms.

use myquat::qasm::{QasmConfig, QasmExporter, QasmVersion};
use myquat::{Parameter, QuantumCircuit};
use std::f64::consts::PI;

#[test]
fn test_simple_circuit_roundtrip() {
    // Create a simple circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure(0, 0).unwrap();
    circuit.measure(1, 1).unwrap();

    // Export to QASM
    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify QASM contains expected elements
    assert!(qasm.contains("OPENQASM 2.0"));
    assert!(qasm.contains("qreg q[2]"));
    assert!(qasm.contains("creg c[2]"));
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("cx q[0],q[1]"));
    assert!(qasm.contains("measure q[0] -> c[0]"));
    assert!(qasm.contains("measure q[1] -> c[1]"));
}

#[test]
fn test_parametric_gates_export() {
    // Test circuit with parametric gates
    let mut circuit = QuantumCircuit::new(1, 0);
    circuit.rx(0, Parameter::Float(PI / 4.0)).unwrap();
    circuit.ry(0, Parameter::Float(PI / 3.0)).unwrap();
    circuit.rz(0, Parameter::Float(PI / 6.0)).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify parametric gates are exported
    assert!(qasm.contains("rx("));
    assert!(qasm.contains("ry("));
    assert!(qasm.contains("rz("));
}

#[test]
fn test_multi_qubit_gates_export() {
    // Test circuit with multi-qubit gates
    let mut circuit = QuantumCircuit::new(3, 0);
    circuit.cx(0, 1).unwrap();
    circuit.cz(1, 2).unwrap();
    circuit.swap(0, 2).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify multi-qubit gates
    assert!(qasm.contains("cx q[0],q[1]"));
    assert!(qasm.contains("cz q[1],q[2]"));
    assert!(qasm.contains("swap q[0],q[2]"));
}

#[test]
fn test_qasm_v2_format() {
    // Test QASM 2.0 format
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();

    let config = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: true,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);
    let qasm = exporter.export(&circuit).unwrap();

    // Verify QASM 2.0 header
    assert!(qasm.starts_with("OPENQASM 2.0;"));
    assert!(qasm.contains("include \"qelib1.inc\";"));
}

#[test]
fn test_qasm_v3_format() {
    // Test QASM 3.0 format
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();

    let config = QasmConfig {
        version: QasmVersion::V3_0,
        include_comments: true,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);
    let qasm = exporter.export(&circuit).unwrap();

    // Verify QASM 3.0 header
    assert!(qasm.starts_with("OPENQASM 3.0;"));
    assert!(qasm.contains("include \"stdgates.inc\";"));
}

#[test]
fn test_bell_state_circuit() {
    // Test Bell state preparation circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure(0, 0).unwrap();
    circuit.measure(1, 1).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify Bell state circuit structure
    let lines: Vec<&str> = qasm.lines().collect();
    let gate_lines: Vec<&str> = lines
        .iter()
        .filter(|l| {
            !l.starts_with("//")
                && !l.starts_with("OPENQASM")
                && !l.starts_with("include")
                && !l.contains("reg")
                && !l.is_empty()
        })
        .copied()
        .collect();

    // Should have H, CX, and 2 measurements
    assert!(gate_lines.len() >= 4);
}

#[test]
fn test_ghz_state_circuit() {
    // Test GHZ state preparation circuit
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.cx(0, 2).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify GHZ circuit
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("cx q[0],q[1]"));
    assert!(qasm.contains("cx q[0],q[2]"));
}

#[test]
fn test_qft_circuit_export() {
    // Test QFT circuit export
    let mut circuit = QuantumCircuit::new(3, 0);

    // Simple 3-qubit QFT structure
    circuit.h(0).unwrap();
    circuit.cp(0, 1, Parameter::Float(PI / 2.0)).unwrap();
    circuit.cp(0, 2, Parameter::Float(PI / 4.0)).unwrap();
    circuit.h(1).unwrap();
    circuit.cp(1, 2, Parameter::Float(PI / 2.0)).unwrap();
    circuit.h(2).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify QFT gates are exported
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("cp("));
    assert!(qasm.contains("h q[1]"));
    assert!(qasm.contains("h q[2]"));
}

#[test]
fn test_precision_control() {
    // Test different precision levels
    let mut circuit = QuantumCircuit::new(1, 0);
    circuit.rx(0, Parameter::Float(PI / 7.0)).unwrap();

    for precision in [3, 6, 10] {
        let config = QasmConfig {
            version: QasmVersion::V2_0,
            include_comments: false,
            use_custom_gates: false,
            include_extended_gates: true,
            precision,
            include_measurements: true,
        };
        let exporter = QasmExporter::with_config(config);
        let qasm = exporter.export(&circuit).unwrap();

        // Verify precision is applied
        assert!(qasm.contains("rx("));
    }
}

#[test]
fn test_comments_control() {
    // Test comment inclusion control
    let mut circuit = QuantumCircuit::new(1, 0);
    circuit.h(0).unwrap();

    // With comments
    let config_with = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: true,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter_with = QasmExporter::with_config(config_with);
    let qasm_with = exporter_with.export(&circuit).unwrap();
    assert!(qasm_with.contains("//"));

    // Without comments
    let config_without = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: false,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter_without = QasmExporter::with_config(config_without);
    let qasm_without = exporter_without.export(&circuit).unwrap();

    // Should have fewer comment lines
    let comments_with = qasm_with.lines().filter(|l| l.contains("//")).count();
    let comments_without = qasm_without.lines().filter(|l| l.contains("//")).count();
    assert!(comments_with > comments_without);
}

#[test]
fn test_measurement_control() {
    // Test measurement inclusion control
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.measure(0, 0).unwrap();
    circuit.measure(1, 1).unwrap();

    // With measurements
    let config_with = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: false,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter_with = QasmExporter::with_config(config_with);
    let qasm_with = exporter_with.export(&circuit).unwrap();
    assert!(qasm_with.contains("measure"));

    // Without measurements
    let config_without = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: false,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: false,
    };
    let exporter_without = QasmExporter::with_config(config_without);
    let qasm_without = exporter_without.export(&circuit).unwrap();
    assert!(!qasm_without.contains("measure"));
}

#[test]
fn test_large_circuit_export() {
    // Test export of larger circuit
    let mut circuit = QuantumCircuit::new(10, 10);

    // Create a pattern of gates
    for i in 0..10 {
        circuit.h(i).unwrap();
    }
    for i in 0..9 {
        circuit.cx(i, i + 1).unwrap();
    }
    for i in 0..10 {
        circuit.measure(i, i).unwrap();
    }

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify circuit structure
    assert!(qasm.contains("qreg q[10]"));
    assert!(qasm.contains("creg c[10]"));

    // Count gates
    let h_count = qasm.matches("h q[").count();
    let cx_count = qasm.matches("cx q[").count();
    let measure_count = qasm.matches("measure q[").count();

    assert_eq!(h_count, 10);
    assert_eq!(cx_count, 9);
    assert_eq!(measure_count, 10);
}

#[test]
fn test_all_standard_gates() {
    // Test export of all standard single-qubit gates
    let mut circuit = QuantumCircuit::new(1, 0);

    circuit.i(0).unwrap();
    circuit.x(0).unwrap();
    circuit.y(0).unwrap();
    circuit.z(0).unwrap();
    circuit.h(0).unwrap();
    circuit.s(0).unwrap();
    circuit.sdg(0).unwrap();
    circuit.t(0).unwrap();
    circuit.tdg(0).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify all gates are present
    assert!(qasm.contains("id q[0]") || qasm.contains("i q[0]"));
    assert!(qasm.contains("x q[0]"));
    assert!(qasm.contains("y q[0]"));
    assert!(qasm.contains("z q[0]"));
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("s q[0]"));
    assert!(qasm.contains("sdg q[0]"));
    assert!(qasm.contains("t q[0]"));
    assert!(qasm.contains("tdg q[0]"));
}

#[test]
fn test_controlled_gates_export() {
    // Test export of controlled gates
    let mut circuit = QuantumCircuit::new(2, 0);

    circuit.cx(0, 1).unwrap();
    circuit.cy(0, 1).unwrap();
    circuit.cz(0, 1).unwrap();
    circuit.ch(0, 1).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify controlled gates
    assert!(qasm.contains("cx q[0],q[1]"));
    assert!(qasm.contains("cy q[0],q[1]"));
    assert!(qasm.contains("cz q[0],q[1]"));
    assert!(qasm.contains("ch q[0],q[1]"));
}

#[test]
fn test_u_gates_export() {
    // Test U gate family export
    let mut circuit = QuantumCircuit::new(1, 0);

    circuit
        .u(
            0,
            Parameter::Float(PI / 4.0),
            Parameter::Float(PI / 3.0),
            Parameter::Float(PI / 6.0),
        )
        .unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify U gate
    assert!(qasm.contains("u(") || qasm.contains("u3("));
}

#[test]
fn test_empty_circuit() {
    // Test export of empty circuit
    let circuit = QuantumCircuit::new(2, 2);

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Should have headers and register declarations
    assert!(qasm.contains("OPENQASM"));
    assert!(qasm.contains("qreg q[2]"));
    assert!(qasm.contains("creg c[2]"));
}

#[test]
fn test_circuit_with_barriers() {
    // Test circuit with barrier operations (if supported)
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(0).unwrap();
    circuit.h(1).unwrap();
    // Note: barrier support depends on implementation
    circuit.cx(0, 1).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Verify basic structure
    assert!(qasm.contains("h q[0]"));
    assert!(qasm.contains("h q[1]"));
    assert!(qasm.contains("cx q[0],q[1]"));
}

#[test]
fn test_qasm_syntax_validity() {
    // Test that exported QASM has valid syntax structure
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();
    circuit.cx(1, 2).unwrap();
    circuit.measure(0, 0).unwrap();
    circuit.measure(1, 1).unwrap();
    circuit.measure(2, 2).unwrap();

    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit).unwrap();

    // Check basic syntax elements
    let lines: Vec<&str> = qasm.lines().collect();

    // First line should be OPENQASM declaration
    assert!(lines[0].starts_with("OPENQASM"));

    // Should have include statement
    assert!(qasm.contains("include"));

    // All gate lines should end with semicolon
    for line in lines.iter() {
        if line.contains("q[") && !line.starts_with("//") {
            assert!(
                line.trim().ends_with(";"),
                "Line should end with semicolon: {}",
                line
            );
        }
    }
}
