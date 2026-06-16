//! Quantum Mechanics Solver Module
//!
//! Author: gA4ss
//!
//! This module provides symbolic quantum mechanics solving capabilities,
//! including wave functions, operators, and equation solvers.
//!
//! # Architecture
//!
//! - **quantum_state**: Symbolic wave function representations
//! - **operators**: Quantum mechanical operators (position, momentum, Hamiltonian, etc.)
//! - **hilbert_space**: Hilbert space operations and basis transformations
//! - **tise_solver**: Time-independent Schrodinger equation solver
//! - **tdse_solver**: Time-dependent Schrodinger equation solver

pub mod angular_momentum;
pub mod circuit_compilation;
pub mod dynamics;
pub mod hilbert_space;
pub mod multi_particle;
pub mod numerical_methods;
pub mod operators;
pub mod perturbation;
pub mod quantum_chemistry;
pub mod quantum_state;
pub mod tdse_solver;
pub mod tise_solver;

// Re-export core types
pub use angular_momentum::{AngularMomentumState, SphericalHarmonic};
pub use dynamics::QuantumPicture;
pub use hilbert_space::{BasisTransformation, BasisType, QuantumBasis, TensorProductSpace};
pub use multi_particle::{ParticleStatistics, TwoParticleSystem, TwoParticleWaveFunction};
pub use numerical_methods::NumericalMethod;
pub use operators::{OperatorType, QuantumOperator};
pub use perturbation::{
    EnergyCorrection, PerturbationTheorySolver, PerturbationType, StateCorrection,
};
pub use quantum_chemistry::Molecule;
pub use quantum_state::{CoordinateSystem, SymbolicWaveFunction, WaveFunctionRepresentation};
pub use tdse_solver::{TDSESolver, TimeEvolutionMethod, TimeEvolutionOperator, TimeEvolvedState};
pub use tise_solver::{EnergyEigenstate, Potential, PotentialType, TISESolver};
