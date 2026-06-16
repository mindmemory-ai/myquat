//! Backend Integration Demo
//!
//! Demonstrates cloud backend configuration and QASM export.

use myquat::compute::cloud_config::{AwsBraketConfig, IbmQuantumConfig};
use myquat::qasm::QasmExporter;
use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("Backend Integration Demo");
    println!("=======================\n");

    let circuit = create_sample_circuit()?;
    println!(
        "Created circuit: {} gates, depth {}",
        circuit.size(),
        circuit.depth()
    );

    println!("\n--- QASM Export ---");
    demo_qasm_export(&circuit)?;

    println!("\n--- Cloud Configuration ---");
    demo_cloud_config();

    println!("\nBackend Integration Demo Completed!");
    Ok(())
}

fn create_sample_circuit() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;
    circuit.rz(1, Parameter::Float(PI / 4.0))?;
    circuit.measure_all()?;
    Ok(circuit)
}

fn demo_qasm_export(circuit: &QuantumCircuit) -> Result<()> {
    let exporter = QasmExporter::new();
    let qasm = exporter.export(circuit)?;
    println!("Exported QASM ({} bytes):", qasm.len());
    for line in qasm.lines().take(8) {
        println!("  {}", line);
    }
    Ok(())
}

fn demo_cloud_config() {
    let ibm = IbmQuantumConfig {
        enabled: true,
        api_token: "IBMQ_TOKEN".into(),
        api_url: "https://api.quantum-computing.ibm.com".into(),
        hub: None,
        group: None,
        project: None,
        default_backend: "ibmq_qasm_simulator".into(),
        prefer_simulator: true,
        max_qubits: 32,
    };
    let aws = AwsBraketConfig {
        enabled: true,
        access_key_id: "AWS_KEY".into(),
        secret_access_key: "AWS_SECRET".into(),
        session_token: None,
        region: "us-east-1".into(),
        s3_bucket: "amazon-braket-results".into(),
        s3_prefix: "myquat/".into(),
        default_backend: "sv1".into(),
        max_qubits: 34,
    };
    println!("IBM Quantum: backend={}", ibm.default_backend);
    println!(
        "AWS Braket: backend={}, region={}",
        aws.default_backend, aws.region
    );
}
