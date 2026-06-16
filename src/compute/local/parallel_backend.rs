//! Parallel Backend
//! Author: gA4ss
//!
//! Multi-threaded parallel compute backend using Rayon

use crate::compute::backend_trait::ComputeBackend;
use crate::compute::types::{
    ComputeBackendType, ExecutionResult, LocalBackendVariant, PerformanceProfile,
};
use crate::error::{MyQuatError, Result};
use crate::QuantumCircuit;
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use rayon::prelude::*;
use std::time::Instant;

/// Parallel compute backend
pub struct ParallelBackend {
    num_threads: usize,
    thread_pool: Option<rayon::ThreadPool>,
    profile: PerformanceProfile,
}

impl ParallelBackend {
    /// Create a new parallel backend with default thread count
    pub fn new() -> Result<Self> {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self::with_threads(num_threads)
    }

    /// Create a parallel backend with specific thread count
    pub fn with_threads(num_threads: usize) -> Result<Self> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| {
                MyQuatError::circuit_error(format!("Failed to create thread pool: {}", e))
            })?;

        let speedup = (num_threads as f64 * 0.8).min(num_threads as f64);

        Ok(ParallelBackend {
            num_threads,
            thread_pool: Some(thread_pool),
            profile: PerformanceProfile {
                max_qubits: 25,
                ops_per_second: 1e6 * speedup,
                startup_overhead_ms: 1.0,
                memory_per_qubit_bytes: 16,
                supports_batching: true,
                optimal_batch_size: Some(num_threads * 2),
            },
        })
    }

    /// Check if parallel backend is available
    pub fn is_available() -> bool {
        std::thread::available_parallelism()
            .map(|n| n.get() > 1)
            .unwrap_or(false)
    }

    /// Get number of threads
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
}

impl ComputeBackend for ParallelBackend {
    fn name(&self) -> &str {
        "parallel"
    }

    fn backend_type(&self) -> ComputeBackendType {
        ComputeBackendType::Local(LocalBackendVariant::Parallel)
    }

    fn is_available(&self) -> bool {
        self.thread_pool.is_some() && Self::is_available()
    }

    fn performance_profile(&self) -> PerformanceProfile {
        self.profile.clone()
    }

    fn execute(&self, circuit: &QuantumCircuit, shots: usize) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Parallel execution strategy: parallelize over shots
        let shots_per_thread = shots.div_ceil(self.num_threads);

        if let Some(ref pool) = self.thread_pool {
            let all_counts = pool.install(|| {
                (0..self.num_threads)
                    .into_par_iter()
                    .map(|thread_id| {
                        let thread_shots = if thread_id == self.num_threads - 1 {
                            shots - (self.num_threads - 1) * shots_per_thread
                        } else {
                            shots_per_thread
                        };

                        if thread_shots == 0 {
                            return Ok(std::collections::HashMap::new());
                        }

                        // Each thread runs its own simulator
                        use crate::StateVectorSimulator;
                        let mut simulator =
                            StateVectorSimulator::new(circuit.num_qubits(), circuit.num_clbits());

                        simulator.run_circuit_multiple(circuit, thread_shots)
                    })
                    .collect::<Result<Vec<_>>>()
            })?;

            // Merge results from all threads
            let mut merged_counts = std::collections::HashMap::new();
            for counts in all_counts {
                for (state, count) in counts {
                    *merged_counts.entry(state).or_insert(0) += count;
                }
            }

            let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

            Ok(ExecutionResult {
                counts: merged_counts,
                shots,
                backend_used: format!("parallel({}t)", self.num_threads),
                execution_time_ms,
                metadata: std::collections::HashMap::new(),
            })
        } else {
            Err(MyQuatError::circuit_error("Thread pool not initialized"))
        }
    }

    fn apply_single_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // For single gate application, parallel overhead is too high
        // Use CPU implementation
        use super::cpu_backend::CpuBackend;
        let cpu = CpuBackend::new();
        cpu.apply_single_qubit_gate(state, gate_matrix, qubit, num_qubits)
    }

    fn apply_two_qubit_gate(
        &self,
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        control: usize,
        target: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // For single gate application, use CPU
        use super::cpu_backend::CpuBackend;
        let cpu = CpuBackend::new();
        cpu.apply_two_qubit_gate(state, gate_matrix, control, target, num_qubits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_backend_creation() {
        if ParallelBackend::is_available() {
            let backend = ParallelBackend::new().unwrap();
            assert_eq!(backend.name(), "parallel");
            assert!(backend.num_threads() > 0);
        }
    }

    #[test]
    fn test_parallel_backend_profile() {
        if ParallelBackend::is_available() {
            let backend = ParallelBackend::new().unwrap();
            let profile = backend.performance_profile();

            // Parallel should support batching
            assert!(profile.supports_batching);
            assert!(profile.optimal_batch_size.is_some());
        }
    }
}
