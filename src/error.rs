//! Error handling for MyQuat

use thiserror::Error;

/// Main error type for MyQuat operations
#[derive(Error, Debug, Clone)]
pub enum MyQuatError {
    #[error("Invalid qubit index: {index} (circuit has {num_qubits} qubits)")]
    InvalidQubitIndex { index: usize, num_qubits: usize },

    #[error("Invalid classical bit index: {index} (circuit has {num_clbits} classical bits)")]
    InvalidClbitIndex { index: usize, num_clbits: usize },

    #[error("Invalid gate parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Gate operation error: {message}")]
    GateError { message: String },

    #[error("Circuit construction error: {message}")]
    CircuitError { message: String },

    #[error("Matrix dimension mismatch: expected {expected}, got {actual}")]
    MatrixDimensionMismatch { expected: usize, actual: usize },

    #[error("Transpiler error: {message}")]
    TranspilerError { message: String },

    #[error("Quantum information error: {message}")]
    QuantumInfoError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Hamiltonian error: {message}")]
    HamiltonianError { message: String },
}

/// Result type alias for MyQuat operations
pub type Result<T> = std::result::Result<T, MyQuatError>;

impl MyQuatError {
    /// Create a new parameter error
    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            message: message.into(),
        }
    }

    /// Create a new gate error
    pub fn gate_error(message: impl Into<String>) -> Self {
        Self::GateError {
            message: message.into(),
        }
    }

    /// Create a new circuit error
    pub fn circuit_error(message: impl Into<String>) -> Self {
        Self::CircuitError {
            message: message.into(),
        }
    }

    /// Create a new transpiler error
    pub fn transpiler_error(message: impl Into<String>) -> Self {
        Self::TranspilerError {
            message: message.into(),
        }
    }

    /// Create a new quantum info error
    pub fn quantum_info_error(message: impl Into<String>) -> Self {
        Self::QuantumInfoError {
            message: message.into(),
        }
    }

    /// Create a new serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a new parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create a new I/O error
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError(message.into())
    }

    /// Create a new Hamiltonian error
    pub fn hamiltonian_error(message: impl Into<String>) -> Self {
        Self::HamiltonianError {
            message: message.into(),
        }
    }
}

// Manual From implementations
impl From<std::io::Error> for MyQuatError {
    fn from(err: std::io::Error) -> Self {
        MyQuatError::IoError(err.to_string())
    }
}

impl From<ndarray::ShapeError> for MyQuatError {
    fn from(err: ndarray::ShapeError) -> Self {
        MyQuatError::circuit_error(format!("Array shape error: {}", err))
    }
}
