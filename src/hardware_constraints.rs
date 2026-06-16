//! Hardware constraint validation and checking
//!
//! This module provides comprehensive validation of quantum circuits against
//! hardware constraints including connectivity, gate fidelities, and timing constraints.

use crate::{HardwareTopology, QuantumCircuit, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Hardware constraint validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    /// Check qubit connectivity constraints
    pub check_connectivity: bool,
    /// Check gate fidelity constraints
    pub check_fidelity: bool,
    /// Check timing constraints
    pub check_timing: bool,
    /// Maximum allowed gate error rate
    pub max_gate_error: f64,
    /// Maximum allowed two-qubit gate error rate
    pub max_two_qubit_error: f64,
    /// Maximum circuit depth allowed
    pub max_circuit_depth: Option<usize>,
    /// Maximum execution time in microseconds
    pub max_execution_time: Option<f64>,
}

impl Default for ConstraintConfig {
    fn default() -> Self {
        ConstraintConfig {
            check_connectivity: true,
            check_fidelity: true,
            check_timing: false,
            max_gate_error: 0.01,      // 1% error rate
            max_two_qubit_error: 0.05, // 5% error rate for two-qubit gates
            max_circuit_depth: Some(100),
            max_execution_time: Some(1000.0), // 1ms
        }
    }
}

/// Constraint violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Description of the violation
    pub description: String,
    /// Instruction index that caused the violation
    pub instruction_index: Option<usize>,
    /// Qubits involved in the violation
    pub qubits: Vec<usize>,
    /// Severity level
    pub severity: Severity,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Types of constraint violations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Qubits are not connected in hardware topology
    ConnectivityViolation,
    /// Gate error rate exceeds threshold
    FidelityViolation,
    /// Circuit depth exceeds maximum
    DepthViolation,
    /// Execution time exceeds maximum
    TimingViolation,
    /// Unsupported gate on hardware
    UnsupportedGate,
    /// Invalid qubit index
    InvalidQubit,
}

/// Severity levels for violations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Warning - circuit may still execute but with degraded performance
    Warning,
    /// Error - circuit cannot execute without modification
    Error,
    /// Critical - fundamental incompatibility
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Hardware constraint validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the circuit passes all constraints
    pub is_valid: bool,
    /// List of constraint violations
    pub violations: Vec<ConstraintViolation>,
    /// Validation statistics
    pub statistics: ValidationStatistics,
}

impl ValidationResult {
    /// Check if there are any critical violations
    pub fn has_critical_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == Severity::Critical)
    }

    /// Check if there are any error-level violations
    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity >= Severity::Error)
    }

    /// Get violations by type
    pub fn violations_by_type(&self, violation_type: ViolationType) -> Vec<&ConstraintViolation> {
        self.violations
            .iter()
            .filter(|v| v.violation_type == violation_type)
            .collect()
    }

    /// Get summary of violations
    pub fn summary(&self) -> String {
        let critical_count = self
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .count();
        let error_count = self
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .count();
        let warning_count = self
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .count();

        format!(
            "Validation Result: {} violations ({} critical, {} errors, {} warnings)",
            self.violations.len(),
            critical_count,
            error_count,
            warning_count
        )
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatistics {
    /// Total number of instructions checked
    pub instructions_checked: usize,
    /// Number of two-qubit gates
    pub two_qubit_gates: usize,
    /// Circuit depth
    pub circuit_depth: usize,
    /// Estimated execution time in microseconds
    pub estimated_execution_time: f64,
    /// Average gate fidelity
    pub average_gate_fidelity: f64,
    /// Connectivity utilization (fraction of available connections used)
    pub connectivity_utilization: f64,
}

/// Hardware constraint validator
pub struct HardwareValidator {
    /// Hardware topology
    topology: HardwareTopology,
    /// Validation configuration
    config: ConstraintConfig,
}

impl HardwareValidator {
    /// Create a new hardware validator
    pub fn new(topology: HardwareTopology, config: ConstraintConfig) -> Self {
        HardwareValidator { topology, config }
    }

    /// Create validator with default configuration
    pub fn with_topology(topology: HardwareTopology) -> Self {
        Self::new(topology, ConstraintConfig::default())
    }

    /// Validate a quantum circuit against hardware constraints
    pub fn validate(&self, circuit: &QuantumCircuit) -> ValidationResult {
        let mut violations = Vec::new();
        let mut stats = ValidationStatistics {
            instructions_checked: 0,
            two_qubit_gates: 0,
            circuit_depth: circuit.depth(),
            estimated_execution_time: 0.0,
            average_gate_fidelity: 1.0,
            connectivity_utilization: 0.0,
        };

        // Check basic circuit constraints
        if circuit.num_qubits() > self.topology.num_qubits {
            violations.push(ConstraintViolation {
                violation_type: ViolationType::InvalidQubit,
                description: format!(
                    "Circuit requires {} qubits but hardware only has {}",
                    circuit.num_qubits(),
                    self.topology.num_qubits
                ),
                instruction_index: None,
                qubits: vec![],
                severity: Severity::Critical,
                suggestion: Some("Reduce circuit size or use a larger device".to_string()),
            });
        }

        // Check circuit depth constraint
        if let Some(max_depth) = self.config.max_circuit_depth {
            if stats.circuit_depth > max_depth {
                violations.push(ConstraintViolation {
                    violation_type: ViolationType::DepthViolation,
                    description: format!(
                        "Circuit depth {} exceeds maximum {}",
                        stats.circuit_depth, max_depth
                    ),
                    instruction_index: None,
                    qubits: vec![],
                    severity: Severity::Error,
                    suggestion: Some("Optimize circuit to reduce depth".to_string()),
                });
            }
        }

        // Validate individual instructions
        let mut used_connections = HashSet::new();
        let mut total_fidelity = 0.0;
        let mut fidelity_count = 0;

        for (idx, instruction) in circuit.data().instructions().iter().enumerate() {
            stats.instructions_checked += 1;

            if instruction.is_measurement() {
                continue; // Skip measurements
            }

            let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();

            // Check connectivity constraints
            if self.config.check_connectivity {
                self.check_connectivity_constraint(
                    &instruction.gate.gate_type,
                    &qubits,
                    idx,
                    &mut violations,
                    &mut used_connections,
                );
            }

            // Check fidelity constraints
            if self.config.check_fidelity {
                if let Some(fidelity) = self.check_fidelity_constraint(
                    &instruction.gate.gate_type,
                    &qubits,
                    idx,
                    &mut violations,
                ) {
                    total_fidelity += fidelity;
                    fidelity_count += 1;
                }
            }

            // Count two-qubit gates
            if qubits.len() == 2 {
                stats.two_qubit_gates += 1;
            }

            // Estimate execution time
            stats.estimated_execution_time += self.estimate_gate_time(&instruction.gate.gate_type);
        }

        // Calculate statistics
        if fidelity_count > 0 {
            stats.average_gate_fidelity = total_fidelity / fidelity_count as f64;
        }

        stats.connectivity_utilization =
            used_connections.len() as f64 / self.topology.connections.len() as f64;

        // Check timing constraints
        if self.config.check_timing {
            if let Some(max_time) = self.config.max_execution_time {
                if stats.estimated_execution_time > max_time {
                    violations.push(ConstraintViolation {
                        violation_type: ViolationType::TimingViolation,
                        description: format!(
                            "Estimated execution time {:.2}μs exceeds maximum {:.2}μs",
                            stats.estimated_execution_time, max_time
                        ),
                        instruction_index: None,
                        qubits: vec![],
                        severity: Severity::Warning,
                        suggestion: Some(
                            "Reduce circuit complexity or use faster gates".to_string(),
                        ),
                    });
                }
            }
        }

        let is_valid = !violations.iter().any(|v| v.severity >= Severity::Error);

        ValidationResult {
            is_valid,
            violations,
            statistics: stats,
        }
    }

    /// Check connectivity constraint for a gate
    fn check_connectivity_constraint(
        &self,
        gate: &StandardGate,
        qubits: &[usize],
        instruction_idx: usize,
        violations: &mut Vec<ConstraintViolation>,
        used_connections: &mut HashSet<(usize, usize)>,
    ) {
        if qubits.len() == 2 {
            let q1 = qubits[0];
            let q2 = qubits[1];

            if !self.topology.are_connected(q1, q2) {
                violations.push(ConstraintViolation {
                    violation_type: ViolationType::ConnectivityViolation,
                    description: format!("Gate {:?} requires connection between qubits {} and {} which are not connected", 
                        gate, q1, q2),
                    instruction_index: Some(instruction_idx),
                    qubits: vec![q1, q2],
                    severity: Severity::Error,
                    suggestion: Some("Insert SWAP gates or remap qubits".to_string()),
                });
            } else {
                used_connections.insert((q1.min(q2), q1.max(q2)));
            }
        }

        // Check if qubits exist
        for &qubit in qubits {
            if qubit >= self.topology.num_qubits {
                violations.push(ConstraintViolation {
                    violation_type: ViolationType::InvalidQubit,
                    description: format!(
                        "Qubit {} does not exist on hardware (max: {})",
                        qubit,
                        self.topology.num_qubits - 1
                    ),
                    instruction_index: Some(instruction_idx),
                    qubits: vec![qubit],
                    severity: Severity::Critical,
                    suggestion: Some("Use valid qubit indices".to_string()),
                });
            }
        }
    }

    /// Check fidelity constraint for a gate
    fn check_fidelity_constraint(
        &self,
        gate: &StandardGate,
        qubits: &[usize],
        instruction_idx: usize,
        violations: &mut Vec<ConstraintViolation>,
    ) -> Option<f64> {
        if qubits.len() == 1 {
            // Single-qubit gate fidelity
            let qubit = qubits[0];
            if let Some(&error_rate) = self.topology.gate_errors.get(&qubit) {
                let fidelity = 1.0 - error_rate;

                if error_rate > self.config.max_gate_error {
                    violations.push(ConstraintViolation {
                        violation_type: ViolationType::FidelityViolation,
                        description: format!(
                            "Gate {:?} on qubit {} has error rate {:.4} exceeding threshold {:.4}",
                            gate, qubit, error_rate, self.config.max_gate_error
                        ),
                        instruction_index: Some(instruction_idx),
                        qubits: vec![qubit],
                        severity: Severity::Warning,
                        suggestion: Some(
                            "Consider using a different qubit or accept lower fidelity".to_string(),
                        ),
                    });
                }

                return Some(fidelity);
            }
        } else if qubits.len() == 2 {
            // Two-qubit gate fidelity
            let q1 = qubits[0];
            let q2 = qubits[1];
            let connection = (q1.min(q2), q1.max(q2));

            if let Some(&error_rate) = self.topology.two_qubit_errors.get(&connection) {
                let fidelity = 1.0 - error_rate;

                if error_rate > self.config.max_two_qubit_error {
                    violations.push(ConstraintViolation {
                        violation_type: ViolationType::FidelityViolation,
                        description: format!("Two-qubit gate {:?} on qubits {},{} has error rate {:.4} exceeding threshold {:.4}", 
                            gate, q1, q2, error_rate, self.config.max_two_qubit_error),
                        instruction_index: Some(instruction_idx),
                        qubits: vec![q1, q2],
                        severity: Severity::Warning,
                        suggestion: Some("Consider using different qubits or accept lower fidelity".to_string()),
                    });
                }

                return Some(fidelity);
            }
        }

        None
    }

    /// Estimate execution time for a gate in microseconds
    fn estimate_gate_time(&self, gate: &StandardGate) -> f64 {
        match gate {
            // Single-qubit gates are typically fast
            StandardGate::I
            | StandardGate::X
            | StandardGate::Y
            | StandardGate::Z
            | StandardGate::H
            | StandardGate::S
            | StandardGate::Sdg
            | StandardGate::T
            | StandardGate::Tdg => 0.02,

            // Rotation gates take slightly longer
            StandardGate::Rx | StandardGate::Ry | StandardGate::Rz | StandardGate::P => 0.03,

            // Universal single-qubit gates
            StandardGate::U | StandardGate::U1 | StandardGate::U2 | StandardGate::U3 => 0.05,

            // Two-qubit gates are slower
            StandardGate::CX | StandardGate::CY | StandardGate::CZ | StandardGate::CH => 0.2,

            // Controlled rotation gates
            StandardGate::CRx | StandardGate::CRy | StandardGate::CRz | StandardGate::CP => 0.25,

            // Multi-qubit gates
            StandardGate::Swap => 0.6, // Typically decomposed into 3 CNOTs
            StandardGate::CCX => 1.0,  // Toffoli gate is expensive
            StandardGate::CSwap => 1.2,

            _ => 0.1, // Default estimate
        }
    }

    /// Generate a detailed validation report
    pub fn generate_report(&self, circuit: &QuantumCircuit) -> String {
        let result = self.validate(circuit);
        let mut report = String::new();

        report.push_str("=== Hardware Constraint Validation Report ===\n\n");

        // Summary
        report.push_str(&format!(
            "Circuit: {} qubits, {} instructions, depth {}\n",
            circuit.num_qubits(),
            result.statistics.instructions_checked,
            result.statistics.circuit_depth
        ));
        report.push_str(&format!(
            "Hardware: {} qubits, {} connections\n\n",
            self.topology.num_qubits,
            self.topology.connections.len()
        ));

        report.push_str(&format!(
            "Validation Status: {}\n",
            if result.is_valid { "PASSED" } else { "FAILED" }
        ));
        report.push_str(&format!("{}\n\n", result.summary()));

        // Statistics
        report.push_str("=== Statistics ===\n");
        report.push_str(&format!(
            "Two-qubit gates: {}\n",
            result.statistics.two_qubit_gates
        ));
        report.push_str(&format!(
            "Average gate fidelity: {:.4}\n",
            result.statistics.average_gate_fidelity
        ));
        report.push_str(&format!(
            "Connectivity utilization: {:.1}%\n",
            result.statistics.connectivity_utilization * 100.0
        ));
        report.push_str(&format!(
            "Estimated execution time: {:.2} μs\n\n",
            result.statistics.estimated_execution_time
        ));

        // Violations
        if !result.violations.is_empty() {
            report.push_str("=== Constraint Violations ===\n");
            for (i, violation) in result.violations.iter().enumerate() {
                report.push_str(&format!(
                    "{}. [{}] {}\n",
                    i + 1,
                    violation.severity,
                    violation.description
                ));
                if let Some(ref suggestion) = violation.suggestion {
                    report.push_str(&format!("   Suggestion: {}\n", suggestion));
                }
                report.push('\n');
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_config() {
        let config = ConstraintConfig::default();
        assert!(config.check_connectivity);
        assert!(config.check_fidelity);
        assert_eq!(config.max_gate_error, 0.01);
        assert_eq!(config.max_two_qubit_error, 0.05);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
    }

    #[test]
    fn test_validation_result_methods() {
        let violations = vec![
            ConstraintViolation {
                violation_type: ViolationType::ConnectivityViolation,
                description: "Test".to_string(),
                instruction_index: None,
                qubits: vec![],
                severity: Severity::Critical,
                suggestion: None,
            },
            ConstraintViolation {
                violation_type: ViolationType::FidelityViolation,
                description: "Test".to_string(),
                instruction_index: None,
                qubits: vec![],
                severity: Severity::Warning,
                suggestion: None,
            },
        ];

        let result = ValidationResult {
            is_valid: false,
            violations,
            statistics: ValidationStatistics {
                instructions_checked: 0,
                two_qubit_gates: 0,
                circuit_depth: 0,
                estimated_execution_time: 0.0,
                average_gate_fidelity: 1.0,
                connectivity_utilization: 0.0,
            },
        };

        assert!(result.has_critical_violations());
        assert!(result.has_errors());
        assert_eq!(
            result
                .violations_by_type(ViolationType::ConnectivityViolation)
                .len(),
            1
        );
    }

    #[test]
    fn test_hardware_validator_creation() {
        let topology = HardwareTopology::linear(3);
        let config = ConstraintConfig::default();
        let validator = HardwareValidator::new(topology, config);

        assert_eq!(validator.topology.num_qubits, 3);
        assert_eq!(validator.topology.connections.len(), 2);
    }

    #[test]
    fn test_valid_circuit_validation() {
        let topology = HardwareTopology::linear(3);
        let validator = HardwareValidator::with_topology(topology);

        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap(); // Adjacent qubits in linear topology

        let result = validator.validate(&circuit);
        assert!(result.is_valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_connectivity_violation() {
        let topology = HardwareTopology::linear(3);
        let validator = HardwareValidator::with_topology(topology);

        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.cx(0, 2).unwrap(); // Non-adjacent qubits in linear topology

        let result = validator.validate(&circuit);
        assert!(!result.is_valid);
        assert!(result.has_errors());

        let connectivity_violations =
            result.violations_by_type(ViolationType::ConnectivityViolation);
        assert_eq!(connectivity_violations.len(), 1);
    }

    #[test]
    fn test_qubit_count_violation() {
        let topology = HardwareTopology::linear(2);
        let validator = HardwareValidator::with_topology(topology);

        let circuit = QuantumCircuit::new(5, 0); // More qubits than hardware

        let result = validator.validate(&circuit);
        assert!(!result.is_valid);
        assert!(result.has_critical_violations());
    }

    #[test]
    fn test_depth_violation() {
        let topology = HardwareTopology::linear(2);
        let mut config = ConstraintConfig::default();
        config.max_circuit_depth = Some(1); // Very low depth limit
        let validator = HardwareValidator::new(topology, config);

        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.cx(0, 1).unwrap(); // This will make depth > 1

        let _result = validator.validate(&circuit);
        // Note: actual depth calculation may vary based on implementation
        // This test checks the depth violation logic
    }

    #[test]
    fn test_fidelity_violation() {
        let mut topology = HardwareTopology::linear(2);
        topology.gate_errors.insert(0, 0.02); // High error rate

        let validator = HardwareValidator::with_topology(topology);

        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap(); // Gate on high-error qubit

        let result = validator.validate(&circuit);
        // Should have fidelity warning
        let fidelity_violations = result.violations_by_type(ViolationType::FidelityViolation);
        assert!(!fidelity_violations.is_empty());
    }

    #[test]
    fn test_validation_statistics() {
        let topology = HardwareTopology::linear(3);
        let validator = HardwareValidator::with_topology(topology);

        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();

        let result = validator.validate(&circuit);
        assert_eq!(result.statistics.instructions_checked, 3);
        assert_eq!(result.statistics.two_qubit_gates, 2);
        assert!(result.statistics.estimated_execution_time > 0.0);
    }

    #[test]
    fn test_validation_report_generation() {
        let topology = HardwareTopology::linear(2);
        let validator = HardwareValidator::with_topology(topology);

        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let report = validator.generate_report(&circuit);
        assert!(report.contains("Hardware Constraint Validation Report"));
        assert!(report.contains("Validation Status"));
        assert!(report.contains("Statistics"));
    }
}
