//! Unified Cloud Backend Demo
//! Author: gA4ss
//!
//! Demonstrates the new unified cloud backend system with configuration management

use myquat::compute::{AwsBraketBackend, CloudConfig, IbmQuantumBackend, JobStatus, TaskStatus};
use myquat::{QuantumCircuit, StandardGate};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MyQuat Unified Cloud Backend Demo ===\n");

    // Demo 1: Load cloud configuration
    println!("1. Loading Cloud Configuration");
    println!("   ---------------------------");

    let config = match CloudConfig::load_default() {
        Ok(cfg) => {
            println!("   ✓ Configuration loaded successfully");
            println!("   Enabled backends: {:?}", cfg.enabled_backends());
            cfg
        }
        Err(e) => {
            println!("   ! No configuration found: {}", e);
            println!("   Using environment variables or default config...");
            CloudConfig::from_env().unwrap_or_else(|_| create_demo_config())
        }
    };

    println!();

    // Demo 2: IBM Quantum Backend
    if let Some(ref ibm_config) = config.ibm_quantum {
        if ibm_config.enabled {
            demo_ibm_quantum(ibm_config.clone())?;
        }
    } else {
        println!("2. IBM Quantum Backend");
        println!("   -------------------");
        println!("   ! IBM Quantum not configured");
        println!();
    }

    // Demo 3: AWS Braket Backend
    if let Some(ref aws_config) = config.aws_braket {
        if aws_config.enabled {
            demo_aws_braket(aws_config.clone())?;
        }
    } else {
        println!("3. AWS Braket Backend");
        println!("   ------------------");
        println!("   ! AWS Braket not configured");
        println!();
    }

    // Demo 4: Configuration-driven backend selection
    demo_backend_selection(&config)?;

    println!("=== Demo Complete ===");
    println!("\nNext steps:");
    println!("1. Configure cloud_config.toml with your credentials");
    println!("2. Run this demo again to see actual cloud execution");
    println!("3. Explore the unified backend manager API");

    Ok(())
}

fn demo_ibm_quantum(
    config: myquat::compute::IbmQuantumConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("2. IBM Quantum Backend");
    println!("   -------------------");

    // Create backend
    let mut backend = IbmQuantumBackend::new(config.clone())?;
    println!("   ✓ IBM Quantum backend initialized");
    println!("   Backend: {}", config.default_backend);
    println!("   Max qubits: {}", config.max_qubits);

    // Create a simple Bell state circuit
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0)?;
    circuit.cx(0, 1)?;

    println!("\n   Circuit:");
    println!("   --------");
    println!("   q0: ─H─●─");
    println!("   q1: ───X─");

    // Submit job (in real implementation)
    println!("\n   Job submission:");
    match backend.submit_job(&circuit, 1000, None) {
        Ok(job_id) => {
            println!("   ✓ Job submitted: {}", job_id);

            // Wait for completion (simulated)
            println!("   Waiting for job completion...");
            match backend.wait_for_job(&job_id, Duration::from_secs(300), Duration::from_secs(5)) {
                Ok(result) => {
                    println!("   ✓ Job completed!");
                    println!("   Results:");
                    for (state, count) in result.counts.iter() {
                        println!("     {}: {}", state, count);
                    }
                }
                Err(e) => println!("   ! Job failed: {}", e),
            }
        }
        Err(e) => println!("   ! Submission failed: {}", e),
    }

    // List available backends
    println!("\n   Available backends:");
    match backend.list_backends() {
        Ok(backends) => {
            for (i, name) in backends.iter().enumerate() {
                println!("     {}. {}", i + 1, name);
            }
        }
        Err(e) => println!("   ! Failed to list backends: {}", e),
    }

    println!();
    Ok(())
}

fn demo_aws_braket(
    config: myquat::compute::AwsBraketConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("3. AWS Braket Backend");
    println!("   ------------------");

    // Create backend
    let mut backend = AwsBraketBackend::new(config.clone())?;
    println!("   ✓ AWS Braket backend initialized");
    println!("   Region: {}", config.region);
    println!("   Device: {}", config.default_backend);
    println!("   Max qubits: {}", config.max_qubits);

    // Create a GHZ state circuit
    let mut circuit = QuantumCircuit::new(3, 3);
    circuit.h(0)?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;

    println!("\n   Circuit (GHZ State):");
    println!("   -------------------");
    println!("   q0: ─H─●───");
    println!("   q1: ───X─●─");
    println!("   q2: ─────X─");

    // Submit task
    println!("\n   Task submission:");
    match backend.submit_task(&circuit, 1000, None) {
        Ok(task_arn) => {
            println!("   ✓ Task submitted: {}", task_arn);

            // Wait for completion
            println!("   Waiting for task completion...");
            match backend.wait_for_task(&task_arn, Duration::from_secs(300), Duration::from_secs(5))
            {
                Ok(result) => {
                    println!("   ✓ Task completed!");
                    println!("   Results:");
                    for (state, count) in result.counts.iter() {
                        println!("     {} : {}", state, count);
                    }
                }
                Err(e) => println!("   ! Task failed: {}", e),
            }
        }
        Err(e) => println!("   ! Submission failed: {}", e),
    }

    // List available devices
    println!("\n   Available devices:");
    match backend.list_devices() {
        Ok(devices) => {
            for device in devices.iter() {
                println!(
                    "     - {} ({:?}, {} qubits)",
                    device.name, device.device_type, device.num_qubits
                );
            }
        }
        Err(e) => println!("   ! Failed to list devices: {}", e),
    }

    println!();
    Ok(())
}

fn demo_backend_selection(
    config: &myquat::compute::CloudConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Configuration-Driven Backend Selection");
    println!("   -------------------------------------");

    let enabled = config.enabled_backends();

    if enabled.is_empty() {
        println!("   ! No cloud backends are enabled");
        println!("   Configure cloud_config.toml to enable backends");
    } else {
        println!("   Enabled backends: {}", enabled.len());
        for backend in &enabled {
            println!("     ✓ {}", backend);
        }

        println!("\n   Backend priority:");
        for (i, name) in config.preferences.backend_priority.iter().enumerate() {
            let symbol = if enabled.contains(&name.as_str()) {
                "✓"
            } else {
                "○"
            };
            println!("     {}. {} {}", i + 1, symbol, name);
        }

        println!("\n   Preferences:");
        println!("     Auto-select: {}", config.preferences.auto_select);
        println!(
            "     Prefer simulators: {}",
            config.preferences.prefer_simulators
        );
        println!("     Max cost: ${:.2}", config.preferences.max_cost_per_job);

        println!("\n   Job management:");
        println!(
            "     Result cache: {}",
            config.job_management.enable_result_cache
        );
        println!("     Auto poll: {}", config.job_management.auto_poll);
        println!(
            "     Poll interval: {}s",
            config.job_management.poll_interval_secs
        );
    }

    println!();
    Ok(())
}

fn create_demo_config() -> myquat::compute::CloudConfig {
    use myquat::compute::{AwsBraketConfig, IbmQuantumConfig};

    let mut config = myquat::compute::CloudConfig::default();

    // Configure with example values (not real credentials)
    config.ibm_quantum = Some(IbmQuantumConfig {
        enabled: true,
        api_token: "demo_token_replace_with_real".to_string(),
        api_url: "https://auth.quantum-computing.ibm.com/api".to_string(),
        hub: None,
        group: None,
        project: None,
        default_backend: "ibmq_qasm_simulator".to_string(),
        prefer_simulator: true,
        max_qubits: 127,
    });

    config.aws_braket = Some(AwsBraketConfig {
        enabled: false,
        access_key_id: "demo_key".to_string(),
        secret_access_key: "demo_secret".to_string(),
        session_token: None,
        region: "us-east-1".to_string(),
        s3_bucket: "demo-bucket".to_string(),
        s3_prefix: "results".to_string(),
        default_backend: "arn:aws:braket:::device/quantum-simulator/amazon/sv1".to_string(),
        max_qubits: 34,
    });

    config
}
