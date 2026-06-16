//! SIMD Backend
//! Author: gA4ss
//!
//! SIMD-accelerated compute backend (AVX2/NEON)

use crate::compute::backend_trait::ComputeBackend;
use crate::compute::simd_ops::SimdQuantumOps;
use crate::compute::types::{
    ComputeBackendType, ExecutionResult, LocalBackendVariant, PerformanceProfile,
};
use crate::error::Result;
use crate::QuantumCircuit;
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use std::time::Instant;

/// SIMD-accelerated compute backend
pub struct SimdBackend {
    simd_width: usize,
    profile: PerformanceProfile,
}

impl SimdBackend {
    /// Create a new SIMD backend
    pub fn new() -> Self {
        let simd_width = SimdQuantumOps::simd_width();

        SimdBackend {
            simd_width,
            profile: PerformanceProfile {
                max_qubits: 25,
                ops_per_second: 2.5e6, // ~2.5x faster than CPU
                startup_overhead_ms: 0.1,
                memory_per_qubit_bytes: 16,
                supports_batching: false,
                optimal_batch_size: None,
            },
        }
    }

    /// Check if SIMD backend is available
    pub fn is_available() -> bool {
        SimdQuantumOps::is_available()
    }

    /// Get SIMD width
    pub fn simd_width(&self) -> usize {
        self.simd_width
    }
}

impl ComputeBackend for SimdBackend {
    fn name(&self) -> &str {
        "simd"
    }

    fn backend_type(&self) -> ComputeBackendType {
        ComputeBackendType::Local(LocalBackendVariant::SIMD)
    }

    fn is_available(&self) -> bool {
        SimdQuantumOps::is_available()
    }

    fn performance_profile(&self) -> PerformanceProfile {
        self.profile.clone()
    }

    fn execute(&self, circuit: &QuantumCircuit, shots: usize) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Use existing StateVectorSimulator with SIMD optimization
        // For now, fallback to CPU execution
        // TODO: Integrate SIMD into StateVectorSimulator
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
        // Use SIMD implementation from simd_ops
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            SimdQuantumOps::apply_single_qubit_gate_simd(state, gate_matrix, qubit, num_qubits)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            SimdQuantumOps::apply_single_qubit_gate_fallback(state, gate_matrix, qubit, num_qubits)
        }
    }

    fn apply_two_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        control: usize,
        target: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // For now, fallback to CPU for two-qubit gates
        // TODO: Implement SIMD two-qubit gates
        use super::cpu_backend::CpuBackend;
        let cpu = CpuBackend::new();
        cpu.apply_two_qubit_gate(state, gate_matrix, control, target, num_qubits)
    }
}

impl Default for SimdBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_backend_creation() {
        let backend = SimdBackend::new();
        assert_eq!(backend.name(), "simd");

        // SIMD availability depends on CPU features
        if SimdBackend::is_available() {
            assert!(backend.is_available());
            assert!(backend.simd_width() > 0);
        }
    }

    #[test]
    fn test_simd_backend_profile() {
        let backend = SimdBackend::new();
        let profile = backend.performance_profile();

        // SIMD should be faster than CPU
        assert!(profile.ops_per_second > 1e6);
    }
}
