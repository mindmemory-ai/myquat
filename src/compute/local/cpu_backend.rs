//! CPU Backend
//! Author: gA4ss
//!
//! Pure CPU compute backend (single-threaded)

use crate::compute::backend_trait::ComputeBackend;
use crate::compute::types::{
    ComputeBackendType, ExecutionResult, LocalBackendVariant, PerformanceProfile,
};
use crate::error::Result;
use crate::QuantumCircuit;
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use std::time::Instant;

/// CPU-based compute backend
pub struct CpuBackend {
    profile: PerformanceProfile,
}

impl CpuBackend {
    /// Create a new CPU backend
    pub fn new() -> Self {
        CpuBackend {
            profile: PerformanceProfile {
                max_qubits: 25,
                ops_per_second: 1e6,
                startup_overhead_ms: 0.0,
                memory_per_qubit_bytes: 16,
                supports_batching: false,
                optimal_batch_size: None,
            },
        }
    }

    /// Check if CPU backend is available (always true)
    pub fn is_available() -> bool {
        true
    }
}

impl ComputeBackend for CpuBackend {
    fn name(&self) -> &str {
        "cpu"
    }

    fn backend_type(&self) -> ComputeBackendType {
        ComputeBackendType::Local(LocalBackendVariant::CPU)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn performance_profile(&self) -> PerformanceProfile {
        self.profile.clone()
    }

    fn execute(&self, circuit: &QuantumCircuit, shots: usize) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Use existing StateVectorSimulator for execution
        use crate::StateVectorSimulator;

        let mut simulator = StateVectorSimulator::new(circuit.num_qubits(), circuit.num_clbits());

        let counts = simulator.run_circuit_multiple(circuit, shots)?;

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ExecutionResult {
            counts,
            shots,
            backend_used: self.name().to_string(),
            execution_time_ms,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn apply_single_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // Use standard CPU implementation
        apply_single_qubit_gate_cpu(state, gate_matrix, qubit, num_qubits)
    }

    fn apply_two_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        control: usize,
        target: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // Use standard CPU implementation
        apply_two_qubit_gate_cpu(state, gate_matrix, control, target, num_qubits)
    }
}

/// Apply single-qubit gate using CPU
fn apply_single_qubit_gate_cpu(
    state: &mut Array1<Complex64>,
    gate_matrix: &Array2<Complex64>,
    qubit_index: usize,
    num_qubits: usize,
) -> Result<()> {
    let dim = state.len();
    let step = 1 << (num_qubits - 1 - qubit_index);

    let g00 = gate_matrix[[0, 0]];
    let g01 = gate_matrix[[0, 1]];
    let g10 = gate_matrix[[1, 0]];
    let g11 = gate_matrix[[1, 1]];

    for i in (0..dim).step_by(2 * step) {
        for j in 0..step {
            let idx0 = i + j;
            let idx1 = i + j + step;

            let amp0 = state[idx0];
            let amp1 = state[idx1];

            state[idx0] = g00 * amp0 + g01 * amp1;
            state[idx1] = g10 * amp0 + g11 * amp1;
        }
    }

    Ok(())
}

/// Apply two-qubit gate using CPU
fn apply_two_qubit_gate_cpu(
    state: &mut Array1<Complex64>,
    gate_matrix: &Array2<Complex64>,
    control: usize,
    target: usize,
    num_qubits: usize,
) -> Result<()> {
    let dim = state.len();
    let control_bit = 1 << (num_qubits - 1 - control);
    let target_bit = 1 << (num_qubits - 1 - target);

    for i in 0..dim {
        if (i & control_bit) == 0 {
            continue;
        }

        let idx0 = i & !target_bit;
        let idx1 = i | target_bit;

        if idx0 >= idx1 {
            continue;
        }

        let amp0 = state[idx0];
        let amp1 = state[idx1];

        state[idx0] = gate_matrix[[0, 0]] * amp0 + gate_matrix[[0, 1]] * amp1;
        state[idx1] = gate_matrix[[1, 0]] * amp0 + gate_matrix[[1, 1]] * amp1;
    }

    Ok(())
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend_availability() {
        assert!(CpuBackend::is_available());
        let backend = CpuBackend::new();
        assert!(backend.is_available());
    }

    #[test]
    fn test_cpu_backend_properties() {
        let backend = CpuBackend::new();
        assert_eq!(backend.name(), "cpu");

        let profile = backend.performance_profile();
        assert_eq!(profile.max_qubits, 25);
    }
}
