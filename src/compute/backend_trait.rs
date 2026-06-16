//! Compute Backend Trait
//! Author: gA4ss
//!
//! Core trait for all compute backends

use crate::compute::types::{ComputeBackendType, ExecutionResult, PerformanceProfile};
use crate::error::Result;
use crate::QuantumCircuit;
use ndarray::{Array1, Array2};
use num_complex::Complex64;

/// Core trait for all compute backends
pub trait ComputeBackend: Send + Sync {
    /// Get the backend name
    fn name(&self) -> &str;

    /// Get the backend type
    fn backend_type(&self) -> ComputeBackendType;

    /// Check if the backend is available on this system
    fn is_available(&self) -> bool;

    /// Get the performance profile of this backend
    fn performance_profile(&self) -> PerformanceProfile;

    /// Execute a quantum circuit (high-level interface)
    fn execute(&self, circuit: &QuantumCircuit, shots: usize) -> Result<ExecutionResult>;

    /// Apply single-qubit gate (low-level interface)
    fn apply_single_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit: usize,
        num_qubits: usize,
    ) -> Result<()>;

    /// Apply two-qubit gate (low-level interface)
    fn apply_two_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        control: usize,
        target: usize,
        num_qubits: usize,
    ) -> Result<()>;

    /// Check if a circuit is compatible with this backend
    fn is_compatible(&self, circuit: &QuantumCircuit) -> bool {
        let profile = self.performance_profile();
        circuit.num_qubits() <= profile.max_qubits
    }

    /// Estimate execution time in milliseconds
    fn estimate_execution_time(&self, circuit: &QuantumCircuit, shots: usize) -> f64 {
        let profile = self.performance_profile();
        let gate_count = circuit.size();
        let ops = gate_count * shots;

        profile.startup_overhead_ms + (ops as f64 / profile.ops_per_second) * 1000.0
    }
}
