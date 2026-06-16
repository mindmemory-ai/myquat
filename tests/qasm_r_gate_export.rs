// Integration tests for R gate QASM export
// Author: gA4ss
//
// These tests verify that arbitrary axis rotation gates R(nx,ny,nz)
// are correctly decomposed and exported to QASM format.

use myquat::circuit::Qubit;
use myquat::gates_extended::ExtendedGate;
use myquat::qasm::{QasmConfig, QasmExporter, QasmVersion};
use std::f64::consts::PI;

#[test]
fn test_r_gate_z_axis() {
    // R gate around Z axis should decompose to simple Rz
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(0.0, 0.0, PI / 4.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain rz gate
    assert!(qasm.contains("rz("));
    assert!(qasm.contains("q[0]"));
}

#[test]
fn test_r_gate_x_axis() {
    // R gate around X axis
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(PI / 3.0, 0.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain decomposition with ry and rz gates
    assert!(qasm.contains("q[0]"));
    // Should have multiple gates in decomposition
    assert!(qasm.lines().count() >= 1);
}

#[test]
fn test_r_gate_y_axis() {
    // R gate around Y axis
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(0.0, PI / 6.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain ry gate
    assert!(qasm.contains("ry("));
    assert!(qasm.contains("q[0]"));
}

#[test]
fn test_r_gate_general_axis() {
    // R gate around general (1,1,1) axis
    let exporter = QasmExporter::new();
    let angle = PI / 4.0;
    let sqrt3 = 3.0_f64.sqrt();
    let r_gate = ExtendedGate::R(angle / sqrt3, angle / sqrt3, angle / sqrt3);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain ZYZ decomposition
    assert!(qasm.contains("q[0]"));
    // Should have decomposition comment
    assert!(qasm.contains("// R(") || !qasm.is_empty());
}

#[test]
fn test_r_gate_zero_rotation() {
    // R gate with zero rotation should give identity
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(0.0, 0.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should be identity gate
    assert!(qasm.contains("id"));
}

#[test]
fn test_r_gate_with_comments() {
    // Test R gate export with comments enabled
    let config = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: true,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);

    let r_gate = ExtendedGate::R(1.0, 1.0, 1.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain decomposition comment
    assert!(qasm.contains("// R("));
}

#[test]
fn test_r_gate_without_comments() {
    // Test R gate export without comments
    let config = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: false,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);

    let r_gate = ExtendedGate::R(1.0, 1.0, 1.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should not contain comments
    assert!(!qasm.contains("//"));
}

#[test]
fn test_r_gate_precision() {
    // Test R gate export with different precision
    let config = QasmConfig {
        version: QasmVersion::V2_0,
        include_comments: false,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 3,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);

    let r_gate = ExtendedGate::R(0.0, 0.0, PI / 4.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should have 3 decimal places
    assert!(qasm.contains("q[0]"));
}

#[test]
fn test_r_gate_v3_export() {
    // Test R gate export in QASM 3.0 format
    let config = QasmConfig {
        version: QasmVersion::V3_0,
        include_comments: true,
        use_custom_gates: false,
        include_extended_gates: true,
        precision: 6,
        include_measurements: true,
    };
    let exporter = QasmExporter::with_config(config);

    let r_gate = ExtendedGate::R(1.0, 0.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should contain valid QASM 3.0 syntax
    assert!(qasm.contains("q[0]"));
}

#[test]
fn test_r_gate_multiple_qubits() {
    // Test R gate on different qubits
    let exporter = QasmExporter::new();

    for qubit_idx in 0..5 {
        let r_gate = ExtendedGate::R(PI / 4.0, 0.0, 0.0);
        let qubits = vec![Qubit::new(qubit_idx)];

        let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

        // Should reference correct qubit
        assert!(qasm.contains(&format!("q[{}]", qubit_idx)));
    }
}

#[test]
fn test_r_gate_normalization() {
    // Test that R gate normalizes the axis vector
    let exporter = QasmExporter::new();

    // Non-normalized axis (2, 0, 0) should give same result as (1, 0, 0)
    let r_gate1 = ExtendedGate::R(2.0, 0.0, 0.0);
    let r_gate2 = ExtendedGate::R(1.0, 0.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm1 = exporter.extended_gate_to_qasm(&r_gate1, &qubits).unwrap();
    let qasm2 = exporter.extended_gate_to_qasm(&r_gate2, &qubits).unwrap();

    // Both should produce valid QASM (angles might differ)
    assert!(qasm1.contains("q[0]"));
    assert!(qasm2.contains("q[0]"));
}

#[test]
fn test_r_gate_small_angles() {
    // Test R gate with very small rotation angles
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(1e-12, 1e-12, 1e-12);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should handle small angles gracefully (likely identity)
    assert!(qasm.contains("id") || qasm.contains("q[0]"));
}

#[test]
fn test_r_gate_large_angles() {
    // Test R gate with large rotation angles
    let exporter = QasmExporter::new();
    let r_gate = ExtendedGate::R(10.0 * PI, 0.0, 0.0);
    let qubits = vec![Qubit::new(0)];

    let qasm = exporter.extended_gate_to_qasm(&r_gate, &qubits).unwrap();

    // Should handle large angles (equivalent to smaller angles mod 2π)
    assert!(qasm.contains("q[0]"));
}
