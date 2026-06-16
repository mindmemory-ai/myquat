//! Quantum algorithms library
//!
//! This module provides implementations of common quantum algorithms
//! organized by algorithm type for better maintainability and extensibility.

pub mod combinatorial;
pub mod grover;
pub mod grover_oracle;
pub mod optimizer;
pub mod pauli_expectation;
pub mod phase_estimation;
pub mod qaoa;
pub mod qft;
pub mod uccsd;
pub mod utils;
pub mod vqe;
pub mod vqe_core;

// Re-export main algorithm structs
pub use combinatorial::{
    Graph, MaxCutProblem, MaxSATProblem, NumberPartitionProblem, PortfolioProblem, SATClause,
};
pub use grover::{GroverSearch, IterationStrategy};
pub use grover_oracle::{GroverOracle, OracleType, SATFormula};
pub use optimizer::{
    ClassicalOptimizer, GradientDescentOptimizer, OptimizationResult, SimplexOptimizer,
};
pub use pauli_expectation::{
    GroupedPauliMeasurement, PauliExpectationComputer, SymmetryAwareExpectation,
};
pub use phase_estimation::{ControlledUnitary, EigenstatePreparation, PhaseEstimation};
pub use qaoa::QAOA;
pub use qft::QFT;
pub use uccsd::{
    double_excitation, generate_double_excitations, generate_single_excitations, single_excitation,
    UCCSDAnsatz,
};
pub use utils::AlgorithmRunner;
pub use vqe::VQEAnsatz;
pub use vqe_core::{VQEConfig, VQEResult, VQE};
