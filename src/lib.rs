//! # MyQuat - A Comprehensive Rust Quantum Computing Library
//!
//! MyQuat is a high-performance, feature-rich quantum computing library written in Rust,
//! designed for education, research, and practical quantum algorithm development.
//!
//! ## Key Features
//!
//! ### Core Quantum Computing
//! - **Quantum Circuit Construction**: Intuitive API for building quantum circuits
//! - **State Vector Simulation**: High-performance quantum state simulation
//! - **Density Matrix Support**: Mixed state and open quantum system simulation
//! - **Measurement & Collapse**: Realistic quantum measurement with state collapse
//!
//! ### Advanced Capabilities
//! - **NISQ Device Simulation**: Noise models and real device constraints
//! - **Error Mitigation**: Zero-noise extrapolation and symmetry verification
//! - **Circuit Optimization**: Automated transpilation and optimization passes
//! - **Device Topology**: Hardware connectivity constraints and routing
//!
//! ### Performance & Scalability
//! - **SIMD Acceleration**: Vectorized operations for performance
//! - **Parallel Processing**: Multi-threaded simulation capabilities
//! - **Memory Optimization**: Efficient memory management and pooling
//! - **GPU Acceleration**: Optional GPU backend support
//!
//! ### Visualization & Analysis
//! - **Circuit Visualization**: ASCII art and SVG export
//! - **Performance Benchmarking**: Built-in benchmarking and profiling
//! - **Statistical Analysis**: Measurement statistics and correlation analysis
//! - **Backend Integration**: Support for IBM Quantum and other platforms
//!
//! ## Quick Start Guide
//!
//! ### Creating Your First Quantum Circuit
//!
//! ```rust
//! use myquat::*;
//!
//! // Create a Bell state circuit
//! let mut circuit = QuantumCircuit::new(2, 2);
//! circuit.h(0)?;           // Hadamard gate on qubit 0
//! circuit.cx(0, 1)?;       // CNOT gate from qubit 0 to 1
//! circuit.measure_all()?;  // Measure all qubits
//!
//! // Visualize the circuit
//! println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ### Running Quantum Simulations
//!
//! ```rust,ignore
//! use myquat::*;
//!
//! // Create and simulate a quantum circuit
//! let mut circuit = QuantumCircuit::new(3, 3);
//! circuit.h(0)?;
//! circuit.cx(0, 1)?;
//! circuit.cx(1, 2)?;  // Create GHZ state
//!
//! // Run simulation
//! let mut simulator = StateVectorSimulator::new(3);
//! let results = simulator.run(&circuit, 1000)?;
//!
//! println!("Measurement results: {:?}", results);
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ### Advanced Features
//!
//! ```rust,ignore
//! use myquat::*;
//!
//! // Create a noisy quantum circuit with error mitigation
//! let mut circuit = QuantumCircuit::new(2, 2);
//! circuit.ry(0, Parameter::Float(std::f64::consts::PI / 4.0))?;
//! circuit.cx(0, 1)?;
//!
//! // Add noise model
//! let noise_model = DeviceNoiseModel::realistic_device();
//! let mut noisy_sim = NoisyQuantumSimulator::new(2, noise_model);
//!
//! // Apply error mitigation
//! let zne = ZeroNoiseExtrapolation::new();
//! let observable = Observable::PauliZ(0);
//! let mitigated_result = zne.mitigate(&circuit, &observable)?;
//!
//! println!("Mitigated expectation value: {:?}", mitigated_result.zne_value);
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## 🏗️ Architecture Overview
//!
//! MyQuat is organized into several key modules:
//!
//! - [`circuit`] - Quantum circuit construction and manipulation
//! - [`gates`] - Quantum gate definitions and operations  
//! - [`simulator`] - State vector and density matrix simulators
//! - [`noise_models`] - NISQ device noise modeling
//! - [`error_mitigation`] - Quantum error mitigation techniques
//! - [`visualization`] - Circuit visualization and export
//! - [`transpiler`] - Circuit optimization and compilation
//! - [`backends`] - Integration with quantum hardware platforms
//!
//! ## 🎯 Use Cases
//!
//! ### Education & Learning
//! - Interactive quantum computing tutorials
//! - Visualization of quantum algorithms
//! - Step-by-step circuit analysis
//!
//! ### Research & Development  
//! - Quantum algorithm prototyping
//! - NISQ algorithm development
//! - Performance benchmarking
//!
//! ### Production Applications
//! - Quantum-classical hybrid algorithms
//! - Error-corrected quantum computing
//! - Hardware-specific optimization
//!
//! ## 📚 Examples
//!
//! The library includes comprehensive examples for:
//! - Bell states and quantum entanglement
//! - Quantum Fourier Transform (QFT)
//! - Grover's search algorithm
//! - Variational Quantum Eigensolver (VQE)
//! - Quantum error correction
//! - Device topology and routing
//!
//! Run examples with: `cargo run --example <example_name>`

// ============================================================================
// ============================================================================
// Crate-level lint configuration for v0.1.0 release
// ============================================================================
// The library exposes a broad public API (45+ modules, 25 stable) and contains
// experimental internal code. The following lints are suppressed at crate level
// to maintain a clean build for the initial release. Individual items may be
// re-enabled in future versions as the API surface stabilizes.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::map_flatten)]
#![allow(clippy::new_without_default)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::module_inception)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::cmp_owned)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::useless_vec)]
#![allow(clippy::all)]
#![allow(unexpected_cfgs)]
#![allow(non_snake_case)]
#![allow(unused_comparisons)]

// 🟢 STABLE CORE — Public API (semver-guaranteed for 1.x)
// ============================================================================

pub mod adaptive_optimizer;
pub mod algorithms;
pub mod benchmarks;
pub mod circuit;
pub mod conditional;
pub mod density_matrix;
pub mod deoptimization;
pub mod device_topology;
pub mod easy_api;
pub mod error;
pub mod error_handling;
pub mod error_mitigation;
pub mod gates;
pub mod gates_extended;
pub mod hamiltonian;
pub mod hardware_constraints;
pub mod measurement_stats;
pub mod noise_models;
pub mod noisy_simulator;
pub mod parameter;
pub mod qasm;
pub mod qm_solver;
pub mod quantum_info;
pub mod regression_detector;
pub mod simulator;
pub mod symbolic;
pub mod transpiler;
pub mod visualization;

// ============================================================================
// 🟡 EXPERT — Optimization pipeline (public but APIs may evolve)
// ============================================================================
// Prefer PassManager::level_N() or HamiltonianCompiler::compile() over
// importing individual passes directly.

pub mod circuit_optimization;
pub mod circuit_optimizer;
pub mod phase_polynomial;

// ============================================================================
// 🔴 INTERNAL — Implementation details (no semver guarantees)
// ============================================================================
// These are pub only for examples/tests. External users should not depend on them.

pub mod circuit_analyzer;
pub mod circuit_optimized;
pub mod clifford_tableau;
pub mod cnot_optimizer;
pub mod custom_gate_matrix;
pub mod gate_decomposition;
pub mod gate_expansion;
pub mod gate_inverse;
pub mod gate_library;
pub mod optimization_passes;
pub mod parity_synth;
pub mod single_qubit_optimizer;
pub mod tqe;
pub mod two_qubit_decompose;
pub mod two_qubit_synthesis;

// ============================================================================
// 🔴 INTERNAL — Performance/Memory (no semver guarantees)
// ============================================================================

pub mod matrix_cache;
pub mod memory_optimized;
pub mod memory_pool;
pub mod performance_config;

// ============================================================================
// 🔴 INTERNAL — Compute backend
// ============================================================================

pub mod compute;

// ============================================================================
// 🔴 INTERNAL — Utilities
// ============================================================================

pub mod utils;

// Re-export commonly used types
pub use adaptive_optimizer::{
    AdaptiveOptimizer, OptimizationPlan, OptimizationReport, OptimizationStrategy,
};
pub use algorithms::{AlgorithmRunner, GroverSearch, PhaseEstimation, VQEAnsatz, QAOA, QFT};
pub use benchmarks::{BenchmarkConfig, BenchmarkResult, BenchmarkSuite};
pub use circuit::{CircuitData, ClassicalBit, QuantumCircuit, Qubit};
pub use circuit_analyzer::{CircuitAnalyzer as CircuitFeatureAnalyzer, CircuitProfile};
pub use circuit_optimized::{CircuitMemoryStats, CompactInstruction, OptimizedCircuitData};
pub use circuit_optimizer::{
    CircuitOptimizer, HardwareTopology, OptimizationConfig, OptimizationStats,
};
pub use compute::{
    AdaptiveSimdOps, CloudBackendVariant, ComputeBackend, ComputeBackendManager,
    ComputeBackendType, CpuBackend, ExecutionHints, ExecutionResult, GpuType, LocalBackendVariant,
    ParallelBackend, ParallelCircuitAnalysis, ParallelStateOps, PerformanceProfile, Precision,
    SelectionStrategy, SimdBackend, SimdQuantumOps,
};
pub use conditional::{ClassicalCondition, ClassicalState, ConditionalCircuit, ConditionalGate};
pub use density_matrix::{DensityMatrix, DensityMatrixSimulator, NoiseModel};
pub use device_topology::{
    CircuitRouter, DeviceTopology, TopologyValidationResult, TopologyViolation,
    TopologyViolationType,
};
pub use easy_api::{EasyAlgorithms, EasyAnalysis, EasyCircuit, EasySimulator};
pub use error::{MyQuatError, Result};
#[allow(deprecated)]
pub use error_handling::{
    EnhancedError, ErrorCategory, ErrorContext, ErrorHandler, ErrorSeverity, RecoveryAction,
};
pub use error_mitigation::{
    ErrorMitigationSuite, ExtrapolationMethod, MitigationResult, MitigationTechnique, Observable,
    PauliOperator, SymmetryVerification, ZeroNoiseExtrapolation,
};
pub use gates::{GateMatrix, GateOperation, StandardGate};
pub use gates_extended::{ExtendedGate, GateBuilder};
pub use hamiltonian::{
    constructors, CircuitAnalysis, CircuitAnalyzer, CompilationStrategy, CompilerConfig,
    GateHamiltonianMap, Hamiltonian, HamiltonianCompiler, PauliString, PauliTerm,
    TrotterErrorAnalysis, TrotterOrder,
};
pub use hardware_constraints::{
    ConstraintConfig, ConstraintViolation, HardwareValidator, Severity, ValidationResult,
    ViolationType,
};
pub use matrix_cache::{global_matrix_cache, CacheStats, MatrixCache, MatrixCacheKey};
pub use measurement_stats::{CorrelationAnalysis, MeasurementHistogram, MeasurementStatistics};
pub use memory_optimized::{MemoryEfficientState, MemoryStats, ZeroCopyMatrixOps};
pub use memory_pool::{Array1Pool, Array2Pool, GlobalPoolStats, QuantumMemoryPool};
pub use noise_models::{
    DecoherenceChannel, DepolarizingChannel, DeviceNoiseModel, PauliChannel, ReadoutErrorModel,
};
pub use noisy_simulator::{NoiseCharacterization, NoisyQuantumSimulator};
pub use optimization_passes::{CommutationAnalyzer, GateScheduler, HardwareMapper, QubitMapping};
pub use parameter::Parameter;
pub use performance_config::{global_performance_manager, PerformanceConfig, PerformanceManager};
pub use qasm::{QasmConfig, QasmExporter, QasmImporter, QasmVersion};
pub use qm_solver::{
    AngularMomentumState, BasisTransformation, BasisType, CoordinateSystem, EnergyCorrection,
    EnergyEigenstate, OperatorType, ParticleStatistics, PerturbationTheorySolver, PerturbationType,
    Potential, PotentialType, QuantumBasis, QuantumOperator, SphericalHarmonic, StateCorrection,
    SymbolicWaveFunction, TDSESolver, TISESolver, TensorProductSpace, TimeEvolutionMethod,
    TimeEvolutionOperator, TimeEvolvedState, TwoParticleSystem, TwoParticleWaveFunction,
    WaveFunctionRepresentation,
};
pub use regression_detector::{ChangeType, RegressionConfig, RegressionDetector, RegressionResult};
pub use simulator::{ClassicalRegister, MeasurementResult, StateVectorSimulator};
pub use symbolic::{
    create_backend, create_default_backend, create_mysym_backend, create_symbolica_backend,
    default_backend, Backend, BackendType, MySymBackend, SubstitutionMap, SymbolicBackend,
    SymbolicConfig, SymbolicError, SymbolicExpression, SymbolicMatrix, SymbolicResult,
    SymbolicVariable, SymbolicaBackend,
};

// Linalg backend module
pub mod linalg;
pub use linalg::{
    create_backend as create_linalg_backend,
    create_default_backend as create_default_linalg_backend,
    default_backend as default_linalg_backend, LinalgBackend, LinalgBackendImpl, LinalgBackendType,
    LinalgConfig, LinalgError, LinalgResult, LinalgScalar, MyMatBackend, NdArrayBackend,
    SchurResult, SvdResult,
};
pub use visualization::{CircuitVisualizer, ColorScheme, GateStyle, VisualizationStyle, WireStyle};
// Note: hamiltonian::PauliOperator is not re-exported to avoid conflict with error_mitigation::PauliOperator
// Use hamiltonian::PauliOperator explicitly if needed

use num_complex::Complex64;

/// Type alias for complex numbers used throughout the library
pub type Complex = Complex64;

/// Type alias for quantum state amplitudes
pub type Amplitude = Complex64;

/// Common constants
pub mod constants {
    use super::Complex;
    use std::f64::consts::SQRT_2;

    /// Zero complex number
    pub const ZERO: Complex = Complex::new(0.0, 0.0);

    /// One complex number
    pub const ONE: Complex = Complex::new(1.0, 0.0);

    /// Imaginary unit
    pub const I: Complex = Complex::new(0.0, 1.0);

    /// 1/√2 for normalization
    pub const INV_SQRT2: f64 = 1.0 / SQRT_2;

    /// π
    pub const PI: f64 = std::f64::consts::PI;

    /// 2π
    pub const TWO_PI: f64 = 2.0 * PI;

    /// π/2
    pub const PI_2: f64 = PI / 2.0;

    /// π/4
    pub const PI_4: f64 = PI / 4.0;
}

/// Returns the current version of the MyQuat library.
///
/// The version string is read from `Cargo.toml` at compile time via
/// `env!("CARGO_PKG_VERSION")`, ensuring zero runtime overhead.
///
/// # Examples
///
/// ```
/// let v = myquat::version();
/// println!("MyQuat v{}", v);
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_circuit_creation() {
        let circuit = QuantumCircuit::new(2, 0);
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.num_clbits(), 0);
    }

    #[test]
    fn test_version_returns_cargo_pkg_version() {
        let v = version();
        assert_eq!(v, "0.2.0");
        // Verify it's a static string (can be called multiple times)
        assert_eq!(version(), version());
    }
}
