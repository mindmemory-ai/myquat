//! Hamiltonian Module
//!
//! Author: gA4ss
//!
//! This module provides complete support for quantum Hamiltonians and their
//! conversion to/from quantum circuits.
//!
//! # Architecture
//!
//! - **pauli_string**: Pauli operator strings (XYZII format)
//! - **hamiltonian**: Hamiltonian as sum of Pauli terms
//! - **circuit_analyzer**: Extract Hamiltonian from quantum circuits
//! - **trotter_compiler**: Compile Hamiltonian to quantum circuits
//! - **latex_exporter**: Export to LaTeX documents
//! - **markdown_exporter**: Export to Markdown documents
//!
//! # Examples
//!
//! ```
//! use myquat::hamiltonian::{Hamiltonian, PauliString};
//! use num_complex::Complex64;
//!
//! // Create an Ising model Hamiltonian
//! let mut h = Hamiltonian::new(3);
//!
//! // Add ZZ interaction
//! let zz = PauliString::from_str("ZZI").unwrap();
//! h.add_term(zz, Complex64::new(-1.0, 0.0)).unwrap();
//!
//! // Export to LaTeX
//! let latex = h.to_latex();
//! println!("{}", latex);
//! ```

pub mod circuit_analyzer;
pub mod diagonalisation;
pub mod fermion;
pub mod hamiltonian;
pub mod hamiltonian_compiler;
pub mod layout_aware_grouping;
pub mod molecule_db;
pub mod optimizer;
pub mod pauli_gadget;
pub mod pauli_gadget_compiler;
pub mod pauli_string;
pub mod pauli_synthesis;
pub mod symbolic_compiler;
pub mod symbolic_hamiltonian;

// Re-export core types
pub use circuit_analyzer::{CircuitAnalysis, CircuitAnalyzer, GateHamiltonianMap};
pub use fermion::{
    bravyi_kitaev_transform, fermion_to_qubit_hamiltonian, jordan_wigner_transform,
    parity_transform, ElectronicStructureHamiltonian, FermionOperator, FermionTerm, MappingMethod,
};
pub use hamiltonian::{constructors, Hamiltonian, PauliTerm};
pub use hamiltonian_compiler::{
    CompilationStrategy, CompilerConfig, HamiltonianCompiler, TrotterErrorAnalysis, TrotterOrder,
};
pub use layout_aware_grouping::{
    GroupingConfig, GroupingResult, GroupingStats, InteractionEdge, InteractionGraph,
    LayoutAwareGrouper, PauliGroup, SparsityAnalysis, SparsityPattern,
};
pub use molecule_db::{Geometry, MoleculeData, MoleculeDatabase};
pub use optimizer::{
    HamiltonianOptimizer, JordanWignerTransform, OptimizationReport, SymmetryOperator,
};
pub use pauli_gadget::{
    optimize_pauli_gadgets, GadgetOptimizationResult, GadgetOptimizationStrategy, OptimizedGadget,
};
pub use pauli_gadget_compiler::{
    gadgets_to_terms_and_map, greedy_pauli_simp, GreedyConfig, GreedyOptimizationResult,
    PauliGadgetNode,
};
pub use pauli_string::{PauliOperator, PauliString};
pub use symbolic_compiler::{SymbolicCircuit, SymbolicCompiler};
pub use symbolic_hamiltonian::{SymbolicHamiltonian, SymbolicPauliTerm};
