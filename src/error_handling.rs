//! Enhanced error handling system
//!
//! This module provides comprehensive error handling capabilities including
//! detailed error classification, context information, and recovery mechanisms.
//!
//! # Status: Not Yet Integrated
//!
//! **Important:** The types in this module (`ErrorHandler`, `ErrorContext`,
//! `EnhancedError`, `ErrorCategory`, `ErrorSeverity`, `RecoveryAction`, etc.) are
//! **not currently used by any production code path**. No module in `src/`
//! instantiates `ErrorHandler` or wraps `MyQuatError` in `EnhancedError` --
//! all production error handling uses [`MyQuatError`] directly via the
//! `Result<T>` alias.
//!
//! The vision for this module is a future structured error-handling layer
//! with category-based recovery strategies and error statistics. Until that
//! integration happens, the types remain part of the public API (for the
//! `error_handling_demo.rs` example and documentation) but are marked
//! `#[deprecated]` to signal their pre-production status.
//!
//! When integrating, use the examples in `examples/error_handling_demo.rs`
//! as a starting point, and wire error collection/recovery into
//! [`ComputeBackendManager`], the optimization pipeline, and the simulator
//! hot paths.
//!
//! `#![allow(deprecated)]` is set on this module so that the internal impl
//! blocks, `create_enhanced_error` helper, and tests can reference the
//! deprecated types without warning. External callers WILL see deprecation
//! warnings.

#![allow(deprecated)]

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Enhanced error types with detailed classification
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Circuit construction and manipulation errors
    Circuit(CircuitErrorKind),
    /// Quantum gate operation errors
    Gate(GateErrorKind),
    /// Simulation and execution errors
    Simulation(SimulationErrorKind),
    /// Hardware and backend errors
    Hardware(HardwareErrorKind),
    /// Algorithm-specific errors
    Algorithm(AlgorithmErrorKind),
    /// Performance and resource errors
    Performance(PerformanceErrorKind),
    /// Configuration and parameter errors
    Configuration(ConfigurationErrorKind),
    /// I/O and serialization errors
    IO(IOErrorKind),
}

/// Circuit-related error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitErrorKind {
    /// Invalid qubit index
    InvalidQubitIndex { qubit: usize, max_qubits: usize },
    /// Invalid classical bit index
    InvalidClbitIndex { clbit: usize, max_clbits: usize },
    /// Circuit size mismatch
    SizeMismatch { expected: usize, actual: usize },
    /// Invalid circuit depth
    InvalidDepth { depth: usize, max_depth: usize },
    /// Incompatible circuit operations
    IncompatibleOperation { operation: String, reason: String },
    /// Circuit validation failure
    ValidationFailure { rule: String, details: String },
}

/// Gate operation error kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GateErrorKind {
    /// Unsupported gate type
    UnsupportedGate { gate: String, context: String },
    /// Invalid gate parameters
    InvalidParameters {
        gate: String,
        expected: usize,
        actual: usize,
    },
    /// Parameter value out of range
    ParameterOutOfRange {
        parameter: String,
        value: String,
        range: String,
    },
    /// Gate matrix dimension mismatch
    MatrixDimensionMismatch {
        expected: (usize, usize),
        actual: (usize, usize),
    },
    /// Gate decomposition failure
    DecompositionFailure { gate: String, reason: String },
}

/// Simulation error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationErrorKind {
    /// State vector dimension mismatch
    StateDimensionMismatch { expected: usize, actual: usize },
    /// Measurement on invalid qubit
    InvalidMeasurement { qubit: usize, reason: String },
    /// Simulation convergence failure
    ConvergenceFailure {
        iterations: usize,
        tolerance: String,
    },
    /// Numerical instability
    NumericalInstability { operation: String, details: String },
    /// Memory allocation failure
    MemoryAllocation { requested: usize, available: usize },
}

/// Hardware and backend error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareErrorKind {
    /// Device not available
    DeviceUnavailable { device: String, status: String },
    /// Connectivity constraint violation
    ConnectivityViolation {
        qubits: Vec<usize>,
        topology: String,
    },
    /// Hardware capability exceeded
    CapabilityExceeded {
        capability: String,
        limit: usize,
        requested: usize,
    },
    /// Backend communication failure
    CommunicationFailure { backend: String, error: String },
    /// Job submission failure
    JobSubmissionFailure { job_id: String, reason: String },
}

/// Algorithm-specific error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmErrorKind {
    /// Algorithm convergence failure
    ConvergenceFailure {
        algorithm: String,
        iterations: usize,
    },
    /// Invalid algorithm parameters
    InvalidParameters {
        algorithm: String,
        parameter: String,
        reason: String,
    },
    /// Algorithm not applicable
    NotApplicable {
        algorithm: String,
        problem: String,
        reason: String,
    },
    /// Optimization failure
    OptimizationFailure {
        optimizer: String,
        objective: String,
    },
}

/// Performance and resource error kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceErrorKind {
    /// Timeout exceeded
    TimeoutExceeded {
        operation: String,
        timeout_ms: u64,
        elapsed_ms: u64,
    },
    /// Memory limit exceeded
    MemoryLimitExceeded { limit_mb: usize, used_mb: usize },
    /// CPU limit exceeded
    CpuLimitExceeded {
        limit_percent: String,
        used_percent: String,
    },
    /// Resource contention
    ResourceContention { resource: String, contenders: usize },
}

/// Configuration and parameter error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationErrorKind {
    /// Missing required configuration
    MissingConfiguration { key: String, section: String },
    /// Invalid configuration value
    InvalidValue {
        key: String,
        value: String,
        expected: String,
    },
    /// Configuration conflict
    ConfigurationConflict { keys: Vec<String>, reason: String },
    /// Version mismatch
    VersionMismatch { expected: String, actual: String },
}

/// I/O and serialization error kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IOErrorKind {
    /// File not found
    FileNotFound { path: String },
    /// Permission denied
    PermissionDenied { path: String, operation: String },
    /// Serialization failure
    SerializationFailure { format: String, reason: String },
    /// Deserialization failure
    DeserializationFailure { format: String, reason: String },
    /// Network error
    NetworkError { endpoint: String, error: String },
}

/// Error context information
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error category and specific kind
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Additional context data
    pub context: HashMap<String, String>,
    /// Error location information
    pub location: Option<ErrorLocation>,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<String>,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Timestamp when error occurred
    pub timestamp: String,
}

/// Error location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// Source file where error occurred
    pub file: String,
    /// Line number in source file
    pub line: u32,
    /// Function or method name
    pub function: String,
}

/// Error severity levels
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational - operation can continue
    Info,
    /// Warning - potential issue but operation can continue
    Warning,
    /// Error - operation failed but system is stable
    Error,
    /// Critical - system stability may be compromised
    Critical,
    /// Fatal - system cannot continue
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

/// Enhanced error with context and recovery capabilities
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone)]
pub struct EnhancedError {
    /// Error context information
    pub context: ErrorContext,
    /// Underlying MyQuatError
    pub inner: MyQuatError,
    /// Recovery strategies
    pub recovery_strategies: Vec<RecoveryStrategy>,
}

/// Recovery strategy for error handling
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    /// Strategy name
    pub name: String,
    /// Strategy description
    pub description: String,
    /// Recovery function (simplified for serialization)
    pub strategy_type: RecoveryStrategyType,
    /// Estimated success probability
    pub success_probability: f64,
}

/// Types of recovery strategies
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategyType {
    /// Retry with backoff
    Retry,
    /// Use fallback approach
    Fallback,
    /// Graceful degradation
    Degrade,
    /// Skip operation
    Skip,
    /// Abort with cleanup
    Abort,
}

/// Recovery action result
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation
    Retry { max_attempts: usize, delay_ms: u64 },
    /// Use fallback approach
    Fallback {
        approach: String,
        data: HashMap<String, String>,
    },
    /// Graceful degradation
    Degrade {
        level: String,
        capabilities: Vec<String>,
    },
    /// Skip operation
    Skip { reason: String },
    /// Abort with cleanup
    Abort { cleanup_actions: Vec<String> },
}

/// Error handler with recovery capabilities
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
pub struct ErrorHandler {
    /// Registered recovery strategies
    strategies: HashMap<String, RecoveryStrategy>,
    /// Error statistics
    statistics: ErrorStatistics,
    /// Configuration
    config: ErrorHandlerConfig,
}

/// Error handler configuration
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlerConfig {
    /// Enable automatic recovery
    pub auto_recovery: bool,
    /// Maximum recovery attempts
    pub max_recovery_attempts: usize,
    /// Log all errors
    pub log_errors: bool,
    /// Collect error statistics
    pub collect_statistics: bool,
    /// Error reporting endpoint
    pub reporting_endpoint: Option<String>,
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        ErrorHandlerConfig {
            auto_recovery: true,
            max_recovery_attempts: 3,
            log_errors: true,
            collect_statistics: true,
            reporting_endpoint: None,
        }
    }
}

/// Error statistics
#[deprecated(
    since = "0.1.0",
    note = "ErrorHandler is not yet integrated into production paths; use MyQuatError directly"
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total error count
    pub total_errors: usize,
    /// Errors by category
    pub errors_by_category: HashMap<String, usize>,
    /// Errors by severity
    pub errors_by_severity: HashMap<String, usize>,
    /// Recovery success rate
    pub recovery_success_rate: f64,
    /// Most common errors
    pub common_errors: Vec<(String, usize)>,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new(config: ErrorHandlerConfig) -> Self {
        let mut handler = ErrorHandler {
            strategies: HashMap::new(),
            statistics: ErrorStatistics::default(),
            config,
        };

        // Register default recovery strategies
        handler.register_default_strategies();
        handler
    }

    /// Create error handler with default configuration
    pub fn default() -> Self {
        Self::new(ErrorHandlerConfig::default())
    }

    /// Register a recovery strategy
    pub fn register_strategy(&mut self, category: String, strategy: RecoveryStrategy) {
        self.strategies.insert(category, strategy);
    }

    /// Handle an error with context and recovery
    pub fn handle_error(&mut self, error: MyQuatError, context: ErrorContext) -> EnhancedError {
        // Update statistics
        if self.config.collect_statistics {
            self.update_statistics(&context);
        }

        // Log error if enabled
        if self.config.log_errors {
            self.log_error(&error, &context);
        }

        // Find applicable recovery strategies
        let recovery_strategies = self.find_recovery_strategies(&context);

        EnhancedError {
            context,
            inner: error,
            recovery_strategies,
        }
    }

    /// Attempt to recover from an error
    pub fn attempt_recovery(&mut self, enhanced_error: &EnhancedError) -> Result<RecoveryAction> {
        if !self.config.auto_recovery {
            return Err(MyQuatError::circuit_error("Auto-recovery disabled"));
        }

        // Try recovery strategies in order of success probability
        let mut strategies = enhanced_error.recovery_strategies.clone();
        strategies.sort_by(|a, b| {
            b.success_probability
                .partial_cmp(&a.success_probability)
                .unwrap()
        });

        for strategy in strategies.iter().take(self.config.max_recovery_attempts) {
            match self.execute_recovery_strategy(strategy, enhanced_error) {
                Ok(action) => {
                    self.statistics.recovery_success_rate =
                        (self.statistics.recovery_success_rate + 1.0) / 2.0;
                    return Ok(action);
                }
                Err(_) => continue,
            }
        }

        Err(MyQuatError::circuit_error("All recovery strategies failed"))
    }

    /// Execute a specific recovery strategy
    fn execute_recovery_strategy(
        &self,
        strategy: &RecoveryStrategy,
        _enhanced_error: &EnhancedError,
    ) -> Result<RecoveryAction> {
        match strategy.strategy_type {
            RecoveryStrategyType::Retry => Ok(RecoveryAction::Retry {
                max_attempts: 3,
                delay_ms: 100,
            }),
            RecoveryStrategyType::Fallback => Ok(RecoveryAction::Fallback {
                approach: "simplified_approach".to_string(),
                data: HashMap::new(),
            }),
            RecoveryStrategyType::Degrade => Ok(RecoveryAction::Degrade {
                level: "reduced_functionality".to_string(),
                capabilities: vec!["basic_operations".to_string()],
            }),
            RecoveryStrategyType::Skip => Ok(RecoveryAction::Skip {
                reason: "Operation not critical".to_string(),
            }),
            RecoveryStrategyType::Abort => Ok(RecoveryAction::Abort {
                cleanup_actions: vec!["release_resources".to_string()],
            }),
        }
    }

    /// Register default recovery strategies
    fn register_default_strategies(&mut self) {
        // Circuit error recovery
        self.register_strategy(
            "circuit".to_string(),
            RecoveryStrategy {
                name: "circuit_fallback".to_string(),
                description: "Use fallback circuit construction".to_string(),
                strategy_type: RecoveryStrategyType::Fallback,
                success_probability: 0.7,
            },
        );

        // Simulation error recovery
        self.register_strategy(
            "simulation".to_string(),
            RecoveryStrategy {
                name: "simulation_retry".to_string(),
                description: "Retry simulation with reduced precision".to_string(),
                strategy_type: RecoveryStrategyType::Retry,
                success_probability: 0.8,
            },
        );

        // Hardware error recovery
        self.register_strategy(
            "hardware".to_string(),
            RecoveryStrategy {
                name: "hardware_fallback".to_string(),
                description: "Fall back to simulator".to_string(),
                strategy_type: RecoveryStrategyType::Fallback,
                success_probability: 0.9,
            },
        );
    }

    /// Find applicable recovery strategies
    fn find_recovery_strategies(&self, context: &ErrorContext) -> Vec<RecoveryStrategy> {
        let category_key = match &context.category {
            ErrorCategory::Circuit(_) => "circuit",
            ErrorCategory::Gate(_) => "gate",
            ErrorCategory::Simulation(_) => "simulation",
            ErrorCategory::Hardware(_) => "hardware",
            ErrorCategory::Algorithm(_) => "algorithm",
            ErrorCategory::Performance(_) => "performance",
            ErrorCategory::Configuration(_) => "configuration",
            ErrorCategory::IO(_) => "io",
        };

        self.strategies
            .get(category_key)
            .map(|s| vec![s.clone()])
            .unwrap_or_default()
    }

    /// Update error statistics
    fn update_statistics(&mut self, context: &ErrorContext) {
        self.statistics.total_errors += 1;

        let category_name = format!("{:?}", context.category);
        *self
            .statistics
            .errors_by_category
            .entry(category_name)
            .or_insert(0) += 1;

        let severity_name = context.severity.to_string();
        *self
            .statistics
            .errors_by_severity
            .entry(severity_name)
            .or_insert(0) += 1;
    }

    /// Log error information
    fn log_error(&self, error: &MyQuatError, context: &ErrorContext) {
        eprintln!("[{}] {} - {}", context.severity, context.message, error);

        if let Some(location) = &context.location {
            eprintln!(
                "  Location: {}:{} in {}",
                location.file, location.line, location.function
            );
        }

        if !context.recovery_suggestions.is_empty() {
            eprintln!("  Suggestions:");
            for suggestion in &context.recovery_suggestions {
                eprintln!("    - {}", suggestion);
            }
        }
    }

    /// Get error statistics
    pub fn statistics(&self) -> &ErrorStatistics {
        &self.statistics
    }

    /// Generate error report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Error Handling Report\n\n");
        report.push_str(&format!("Total Errors: {}\n", self.statistics.total_errors));
        report.push_str(&format!(
            "Recovery Success Rate: {:.2}%\n\n",
            self.statistics.recovery_success_rate * 100.0
        ));

        report.push_str("## Errors by Category\n");
        for (category, count) in &self.statistics.errors_by_category {
            report.push_str(&format!("- {}: {}\n", category, count));
        }

        report.push_str("\n## Errors by Severity\n");
        for (severity, count) in &self.statistics.errors_by_severity {
            report.push_str(&format!("- {}: {}\n", severity, count));
        }

        report
    }
}

/// Helper function for creating enhanced errors
pub fn create_enhanced_error(
    category: ErrorCategory,
    message: &str,
    severity: ErrorSeverity,
) -> ErrorContext {
    ErrorContext {
        category,
        message: message.to_string(),
        context: HashMap::new(),
        location: Some(ErrorLocation {
            file: "unknown".to_string(),
            line: 0,
            function: "unknown".to_string(),
        }),
        recovery_suggestions: vec![],
        severity,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let circuit_error = ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
            qubit: 5,
            max_qubits: 4,
        });

        assert!(matches!(circuit_error, ErrorCategory::Circuit(_)));
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Fatal > ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
        assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
    }

    #[test]
    fn test_error_handler_creation() {
        let handler = ErrorHandler::default();
        assert!(handler.config.auto_recovery);
        assert_eq!(handler.config.max_recovery_attempts, 3);
        assert!(handler.strategies.len() > 0);
    }

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext {
            category: ErrorCategory::Circuit(CircuitErrorKind::InvalidQubitIndex {
                qubit: 5,
                max_qubits: 4,
            }),
            message: "Invalid qubit index".to_string(),
            context: HashMap::new(),
            location: None,
            recovery_suggestions: vec!["Use valid qubit index".to_string()],
            severity: ErrorSeverity::Error,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(context.message, "Invalid qubit index");
        assert_eq!(context.severity, ErrorSeverity::Error);
        assert_eq!(context.recovery_suggestions.len(), 1);
    }

    #[test]
    fn test_recovery_strategy() {
        let strategy = RecoveryStrategy {
            name: "test_strategy".to_string(),
            description: "Test recovery".to_string(),
            strategy_type: RecoveryStrategyType::Skip,
            success_probability: 0.5,
        };

        assert_eq!(strategy.name, "test_strategy");
        assert_eq!(strategy.success_probability, 0.5);
        assert_eq!(strategy.strategy_type, RecoveryStrategyType::Skip);
    }

    #[test]
    fn test_error_statistics() {
        let mut stats = ErrorStatistics::default();
        assert_eq!(stats.total_errors, 0);
        assert!(stats.errors_by_category.is_empty());

        stats.total_errors = 10;
        stats.errors_by_category.insert("circuit".to_string(), 5);

        assert_eq!(stats.total_errors, 10);
        assert_eq!(stats.errors_by_category.get("circuit"), Some(&5));
    }
}
