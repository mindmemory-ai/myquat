//! Cloud Backend Configuration Demo
//! Author: gA4ss
//!
//! Demonstrates the new unified cloud configuration system

use myquat::compute::CloudConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MyQuat Cloud Configuration Demo ===\n");

    // Method 1: Load from default locations
    println!("Method 1: Loading from default locations...");
    match CloudConfig::load_default() {
        Ok(config) => {
            print_config_info(&config);
        }
        Err(e) => {
            println!("No default config found: {}", e);
            println!("This is expected if cloud_config.toml doesn't exist yet.\n");
        }
    }

    // Method 2: Load from environment variables (CI/CD friendly)
    println!("Method 2: Loading from environment variables...");
    match CloudConfig::from_env() {
        Ok(config) => {
            println!("✓ Successfully loaded config from environment");
            print_config_info(&config);
        }
        Err(e) => {
            println!("✗ Failed to load from env: {}", e);
            println!("Set environment variables like IBM_QUANTUM_TOKEN to use this method.\n");
        }
    }

    // Method 3: Create example configuration programmatically
    println!("Method 3: Creating configuration programmatically...");
    let mut config = CloudConfig::default();

    // Configure IBM Quantum (example with placeholder)
    config.ibm_quantum = Some(myquat::compute::IbmQuantumConfig {
        enabled: true,
        api_token: "your_ibm_token_here".to_string(),
        api_url: "https://auth.quantum-computing.ibm.com/api".to_string(),
        hub: Some("ibm-q".to_string()),
        group: Some("open".to_string()),
        project: Some("main".to_string()),
        default_backend: "ibmq_qasm_simulator".to_string(),
        prefer_simulator: true,
        max_qubits: 127,
    });

    // Configure AWS Braket (example with placeholder)
    config.aws_braket = Some(myquat::compute::AwsBraketConfig {
        enabled: false, // Disabled by default
        access_key_id: "your_aws_key_id".to_string(),
        secret_access_key: "your_aws_secret".to_string(),
        session_token: None,
        region: "us-east-1".to_string(),
        s3_bucket: "amazon-braket-results".to_string(),
        s3_prefix: "myquat-results".to_string(),
        default_backend: "arn:aws:braket:::device/quantum-simulator/amazon/sv1".to_string(),
        max_qubits: 34,
    });

    println!("✓ Created example configuration");
    print_config_info(&config);

    // Method 4: Parse from TOML string
    println!("Method 4: Parsing from TOML string...");
    let toml_str = r#"
[global]
timeout_secs = 300
max_retries = 3

[ibm_quantum]
enabled = true
api_token = "example_token"
default_backend = "ibmq_qasm_simulator"

[preferences]
auto_select = true
prefer_simulators = true
backend_priority = ["ibm_quantum", "aws_braket"]

[job_management]
enable_result_cache = true
auto_poll = true
poll_interval_secs = 10

[logging]
level = "info"
log_api_calls = false
    "#;

    match CloudConfig::from_toml(toml_str) {
        Ok(config) => {
            println!("✓ Successfully parsed TOML configuration");
            print_config_info(&config);
        }
        Err(e) => {
            println!("✗ Failed to parse TOML: {}", e);
        }
    }

    println!("\n=== Demo Complete ===");
    println!("\nNext steps:");
    println!("1. Copy cloud_config.toml.example to cloud_config.toml");
    println!("2. Fill in your API credentials");
    println!("3. Run this demo again to see your actual configuration");
    println!("\nSecurity reminder:");
    println!("- NEVER commit cloud_config.toml to version control");
    println!("- Add it to .gitignore");
    println!("- Protect file permissions: chmod 600 cloud_config.toml");

    Ok(())
}

fn print_config_info(config: &CloudConfig) {
    println!("\n--- Configuration Summary ---");

    // Global settings
    println!("Global:");
    println!("  Timeout: {} seconds", config.global.timeout_secs);
    println!("  Max retries: {}", config.global.max_retries);
    println!("  Cache enabled: {}", config.global.enable_cache);

    // Enabled backends
    let enabled = config.enabled_backends();
    println!(
        "\nEnabled backends: {}",
        if enabled.is_empty() {
            "none".to_string()
        } else {
            enabled.join(", ")
        }
    );

    // IBM Quantum
    if let Some(ref ibm) = config.ibm_quantum {
        println!("\nIBM Quantum:");
        println!(
            "  Status: {}",
            if ibm.enabled { "enabled" } else { "disabled" }
        );
        if ibm.enabled {
            println!("  API URL: {}", ibm.api_url);
            println!("  Default backend: {}", ibm.default_backend);
            println!("  Max qubits: {}", ibm.max_qubits);
            println!("  Prefer simulator: {}", ibm.prefer_simulator);
            if let Some(ref hub) = ibm.hub {
                println!("  Hub: {}", hub);
            }
        }
    }

    // AWS Braket
    if let Some(ref aws) = config.aws_braket {
        println!("\nAWS Braket:");
        println!(
            "  Status: {}",
            if aws.enabled { "enabled" } else { "disabled" }
        );
        if aws.enabled {
            println!("  Region: {}", aws.region);
            println!("  S3 Bucket: {}", aws.s3_bucket);
            println!("  Default backend: {}", aws.default_backend);
            println!("  Max qubits: {}", aws.max_qubits);
        }
    }

    // Azure Quantum
    if let Some(ref azure) = config.azure_quantum {
        println!("\nAzure Quantum:");
        println!(
            "  Status: {}",
            if azure.enabled { "enabled" } else { "disabled" }
        );
        if azure.enabled {
            println!("  Location: {}", azure.location);
            println!("  Workspace: {}", azure.workspace_name);
            println!("  Provider: {}", azure.default_provider);
            println!("  Max qubits: {}", azure.max_qubits);
        }
    }

    // Google Quantum
    if let Some(ref google) = config.google_quantum {
        println!("\nGoogle Quantum:");
        println!(
            "  Status: {}",
            if google.enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        if google.enabled {
            println!("  Project: {}", google.project_id);
            println!("  Processor: {}", google.processor_id);
            println!("  Max qubits: {}", google.max_qubits);
        }
    }

    // Preferences
    println!("\nPreferences:");
    println!("  Auto-select: {}", config.preferences.auto_select);
    println!(
        "  Prefer simulators: {}",
        config.preferences.prefer_simulators
    );
    println!(
        "  Max cost per job: ${:.2}",
        config.preferences.max_cost_per_job
    );
    println!(
        "  Backend priority: {:?}",
        config.preferences.backend_priority
    );

    // Job Management
    println!("\nJob Management:");
    println!(
        "  Result cache: {}",
        config.job_management.enable_result_cache
    );
    println!("  Auto poll: {}", config.job_management.auto_poll);
    println!(
        "  Poll interval: {} seconds",
        config.job_management.poll_interval_secs
    );
    println!(
        "  Max wait time: {} seconds",
        config.job_management.max_wait_time_secs
    );

    // Validation
    match config.validate() {
        Ok(_) => println!("\n✓ Configuration is valid"),
        Err(e) => println!("\n✗ Configuration validation failed: {}", e),
    }

    println!("----------------------------\n");
}
