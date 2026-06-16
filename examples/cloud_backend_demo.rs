//! Cloud Backend Integration Demo
//! Author: gA4ss
//!
//! Demonstrates cloud backend configuration for IBM Quantum and AWS Braket.
//! Actual execution requires valid API credentials.

use myquat::compute::cloud_config::{AwsBraketConfig, IbmQuantumConfig};
use myquat::qasm::QasmExporter;
use myquat::Result;
use myquat::{Parameter, QuantumCircuit};

fn main() -> Result<()> {
    println!("Cloud Backend Configuration Demo");
    println!("================================\n");

    // IBM Quantum configuration
    println!("--- IBM Quantum ---");
    let ibm_config = IbmQuantumConfig {
        enabled: true,
        api_token: "YOUR_IBMQ_API_TOKEN".to_string(),
        api_url: "https://api.quantum-computing.ibm.com".to_string(),
        hub: None,
        group: None,
        project: None,
        default_backend: "ibmq_qasm_simulator".to_string(),
        prefer_simulator: true,
        max_qubits: 32,
    };
    println!(
        "Config: {} backend, {} qubits max",
        ibm_config.default_backend, ibm_config.max_qubits
    );

    // AWS Braket configuration
    println!("\n--- AWS Braket ---");
    let aws_config = AwsBraketConfig {
        enabled: true,
        access_key_id: "YOUR_AWS_ACCESS_KEY".to_string(),
        secret_access_key: "YOUR_AWS_SECRET_KEY".to_string(),
        session_token: None,
        region: "us-east-1".to_string(),
        s3_bucket: "amazon-braket-results".to_string(),
        s3_prefix: "myquat/".to_string(),
        default_backend: "sv1".to_string(),
        max_qubits: 34,
    };
    println!(
        "Config: {} backend in {}, bucket {}",
        aws_config.default_backend, aws_config.region, aws_config.s3_bucket
    );

    // Create a Bell state circuit for submission
    println!("\n--- Bell State Circuit ---");
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.measure(0, 0)?;
    circuit.measure(1, 1)?;
    println!("Gates: {}, Depth: {}", circuit.size(), circuit.depth());

    // QASM export (used by both IBM and AWS backends)
    println!("\n--- QASM Export ---");
    let exporter = QasmExporter::new();
    let qasm = exporter.export(&circuit)?;
    println!("Exported {} bytes of OpenQASM", qasm.len());

    println!("\nCloud Backend Demo Completed!");
    Ok(())
}
