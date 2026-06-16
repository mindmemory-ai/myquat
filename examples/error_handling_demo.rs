//! Error Handling Demo
//!
//! This example demonstrates the enhanced error handling capabilities
//! including detailed error classification, context information, and recovery mechanisms.

use myquat::error_handling::*;
use myquat::*;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("🚨 Enhanced Error Handling Demo");
    println!("===============================\n");

    // Demo 1: Error Classification
    demo_error_classification()?;

    // Demo 2: Error Context and Recovery
    demo_error_context_and_recovery()?;

    // Demo 3: Error Handler with Statistics
    demo_error_handler()?;

    // Demo 4: Recovery Strategies
    demo_recovery_strategies()?;

    println!("✅ Error handling demo completed successfully!");
    Ok(())
}

fn demo_error_classification() -> Result<()> {
    println!("🔧 Demo 1: Error Classification");
    println!("-------------------------------");

    // Circuit errors
    let circuit_errors = vec![
        ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
            qubit: 5,
            max_qubits: 4,
        }),
        ErrorCategory::Circuit(CircuitErrorKind::SizeMismatch {
            expected: 10,
            actual: 8,
        }),
        ErrorCategory::Circuit(CircuitErrorKind::ValidationFailure {
            rule: "qubit_connectivity".to_string(),
            details: "Qubits 0 and 3 are not connected".to_string(),
        }),
    ];

    println!("📋 Circuit Error Examples:");
    for (i, error) in circuit_errors.iter().enumerate() {
        println!("  {}. {:?}", i + 1, error);
    }
    println!();

    // Gate errors
    let gate_errors = vec![
        ErrorCategory::Gate(GateErrorKind::UnsupportedGate {
            gate: "CustomGate".to_string(),
            context: "hardware_backend".to_string(),
        }),
        ErrorCategory::Gate(GateErrorKind::InvalidParameters {
            gate: "RX".to_string(),
            expected: 1,
            actual: 3,
        }),
        ErrorCategory::Gate(GateErrorKind::ParameterOutOfRange {
            parameter: "angle".to_string(),
            value: "10.5".to_string(),
            range: "(0.0, 6.28)".to_string(),
        }),
    ];

    println!("🚪 Gate Error Examples:");
    for (i, error) in gate_errors.iter().enumerate() {
        println!("  {}. {:?}", i + 1, error);
    }
    println!();

    // Hardware errors
    let hardware_errors = vec![
        ErrorCategory::Hardware(HardwareErrorKind::DeviceUnavailable {
            device: "ibm_nairobi".to_string(),
            status: "maintenance".to_string(),
        }),
        ErrorCategory::Hardware(HardwareErrorKind::ConnectivityViolation {
            qubits: vec![0, 3],
            topology: "linear".to_string(),
        }),
        ErrorCategory::Hardware(HardwareErrorKind::CapabilityExceeded {
            capability: "max_qubits".to_string(),
            limit: 127,
            requested: 200,
        }),
    ];

    println!("🖥️  Hardware Error Examples:");
    for (i, error) in hardware_errors.iter().enumerate() {
        println!("  {}. {:?}", i + 1, error);
    }
    println!();

    Ok(())
}

fn demo_error_context_and_recovery() -> Result<()> {
    println!("🔧 Demo 2: Error Context and Recovery");
    println!("-------------------------------------");

    // Create error context with detailed information
    let mut context_data = HashMap::new();
    context_data.insert("circuit_depth".to_string(), "150".to_string());
    context_data.insert("num_qubits".to_string(), "10".to_string());
    context_data.insert("backend".to_string(), "ibm_quantum".to_string());

    let error_context = ErrorContext {
        category: ErrorCategory::Simulation(SimulationErrorKind::MemoryAllocation {
            requested: 1024 * 1024 * 1024, // 1GB
            available: 512 * 1024 * 1024,  // 512MB
        }),
        message: "Insufficient memory for quantum state simulation".to_string(),
        context: context_data,
        location: Some(ErrorLocation {
            file: "simulator.rs".to_string(),
            line: 245,
            function: "allocate_state_vector".to_string(),
        }),
        recovery_suggestions: vec![
            "Reduce circuit depth or number of qubits".to_string(),
            "Use memory-efficient simulation mode".to_string(),
            "Switch to distributed simulation".to_string(),
        ],
        severity: ErrorSeverity::Error,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    println!("📊 Error Context Information:");
    println!("  Category: {:?}", error_context.category);
    println!("  Message: {}", error_context.message);
    println!("  Severity: {}", error_context.severity);

    if let Some(location) = &error_context.location {
        println!(
            "  Location: {}:{} in {}",
            location.file, location.line, location.function
        );
    }

    println!("  Context Data:");
    for (key, value) in &error_context.context {
        println!("    {}: {}", key, value);
    }

    println!("  Recovery Suggestions:");
    for (i, suggestion) in error_context.recovery_suggestions.iter().enumerate() {
        println!("    {}. {}", i + 1, suggestion);
    }
    println!();

    Ok(())
}

fn demo_error_handler() -> Result<()> {
    println!("🔧 Demo 3: Error Handler with Statistics");
    println!("----------------------------------------");

    // Create error handler with custom configuration
    let config = ErrorHandlerConfig {
        auto_recovery: true,
        max_recovery_attempts: 5,
        log_errors: false, // Disable logging for demo
        collect_statistics: true,
        reporting_endpoint: None,
    };

    let mut error_handler = ErrorHandler::new(config);

    // Simulate various errors
    let test_errors = vec![
        (
            MyQuatError::circuit_error("Invalid qubit index"),
            ErrorContext {
                category: ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
                    qubit: 5,
                    max_qubits: 4,
                }),
                message: "Qubit index out of range".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Use valid qubit index (0-3)".to_string()],
                severity: ErrorSeverity::Error,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
        (
            MyQuatError::circuit_error("Gate not supported"),
            ErrorContext {
                category: ErrorCategory::Gate(GateErrorKind::UnsupportedGate {
                    gate: "CustomGate".to_string(),
                    context: "hardware".to_string(),
                }),
                message: "Gate not supported on hardware".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Use gate decomposition".to_string()],
                severity: ErrorSeverity::Warning,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
        (
            MyQuatError::circuit_error("Simulation failed"),
            ErrorContext {
                category: ErrorCategory::Simulation(SimulationErrorKind::NumericalInstability {
                    operation: "matrix_multiplication".to_string(),
                    details: "Condition number too large".to_string(),
                }),
                message: "Numerical instability detected".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Use higher precision arithmetic".to_string()],
                severity: ErrorSeverity::Critical,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
    ];

    println!("📈 Processing {} test errors...", test_errors.len());

    for (error, context) in test_errors {
        let enhanced_error = error_handler.handle_error(error, context);
        println!(
            "  Handled: {} (Severity: {})",
            enhanced_error.context.message, enhanced_error.context.severity
        );
    }

    // Display statistics
    let stats = error_handler.statistics();
    println!("\n📊 Error Statistics:");
    println!("  Total Errors: {}", stats.total_errors);
    println!(
        "  Recovery Success Rate: {:.1}%",
        stats.recovery_success_rate * 100.0
    );

    println!("  Errors by Category:");
    for (category, count) in &stats.errors_by_category {
        println!("    {}: {}", category, count);
    }

    println!("  Errors by Severity:");
    for (severity, count) in &stats.errors_by_severity {
        println!("    {}: {}", severity, count);
    }

    println!();
    Ok(())
}

fn demo_recovery_strategies() -> Result<()> {
    println!("🔧 Demo 4: Recovery Strategies");
    println!("------------------------------");

    let mut error_handler = ErrorHandler::default();

    // Test different recovery scenarios
    let recovery_scenarios = vec![
        (
            "Circuit Error Recovery",
            ErrorContext {
                category: ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
                    qubit: 10,
                    max_qubits: 5,
                }),
                message: "Qubit index too large".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Reduce circuit size".to_string()],
                severity: ErrorSeverity::Error,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
        (
            "Simulation Error Recovery",
            ErrorContext {
                category: ErrorCategory::Simulation(SimulationErrorKind::ConvergenceFailure {
                    iterations: 1000,
                    tolerance: "1e-6".to_string(),
                }),
                message: "Simulation did not converge".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Increase iteration limit".to_string()],
                severity: ErrorSeverity::Warning,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
        (
            "Hardware Error Recovery",
            ErrorContext {
                category: ErrorCategory::Hardware(HardwareErrorKind::DeviceUnavailable {
                    device: "ibm_quantum".to_string(),
                    status: "offline".to_string(),
                }),
                message: "Hardware device unavailable".to_string(),
                context: HashMap::new(),
                location: None,
                recovery_suggestions: vec!["Use simulator fallback".to_string()],
                severity: ErrorSeverity::Error,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ),
    ];

    for (scenario_name, context) in recovery_scenarios {
        println!("🔄 Testing: {}", scenario_name);

        let error = MyQuatError::circuit_error("Test error");
        let enhanced_error = error_handler.handle_error(error, context);

        match error_handler.attempt_recovery(&enhanced_error) {
            Ok(action) => {
                println!("  ✅ Recovery successful: {:?}", action);
            }
            Err(e) => {
                println!("  ❌ Recovery failed: {}", e);
            }
        }
    }

    println!();

    // Generate comprehensive error report
    println!("📋 Error Handler Report:");
    let report = error_handler.generate_report();
    println!("{}", report);

    Ok(())
}

/// Helper function to simulate different error types
#[allow(dead_code)]
fn simulate_circuit_error() -> Result<()> {
    // This would normally be in circuit construction code
    let max_qubits = 5;
    let requested_qubit = 10;

    if requested_qubit >= max_qubits {
        let context = create_enhanced_error(
            ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
                qubit: requested_qubit,
                max_qubits,
            }),
            "Qubit index exceeds circuit capacity",
            ErrorSeverity::Error,
        );

        // In real code, you would use the error handler here
        println!("Error context created: {}", context.message);
    }

    Ok(())
}

/// Helper function to demonstrate error recovery
#[allow(dead_code)]
fn demonstrate_error_recovery() -> Result<()> {
    let mut error_handler = ErrorHandler::default();

    // Simulate a hardware error that can be recovered
    let error = MyQuatError::circuit_error("Hardware unavailable");
    let context = ErrorContext {
        category: ErrorCategory::Hardware(HardwareErrorKind::DeviceUnavailable {
            device: "quantum_device".to_string(),
            status: "maintenance".to_string(),
        }),
        message: "Quantum device is under maintenance".to_string(),
        context: HashMap::new(),
        location: None,
        recovery_suggestions: vec![
            "Wait for maintenance to complete".to_string(),
            "Use alternative device".to_string(),
            "Switch to simulator".to_string(),
        ],
        severity: ErrorSeverity::Warning,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let enhanced_error = error_handler.handle_error(error, context);

    match error_handler.attempt_recovery(&enhanced_error) {
        Ok(RecoveryAction::Fallback { approach, .. }) => {
            println!("Successfully recovered using fallback: {}", approach);
        }
        Ok(action) => {
            println!("Recovery action: {:?}", action);
        }
        Err(e) => {
            println!("Recovery failed: {}", e);
        }
    }

    Ok(())
}
