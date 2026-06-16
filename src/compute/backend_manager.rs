//! Compute Backend Manager
//! Author: gA4ss
//!
//! Manages multiple compute backends and provides intelligent selection

use crate::compute::backend_trait::ComputeBackend;
use crate::compute::types::{
    ComputeBackendType, ExecutionHints, ExecutionResult, PerformanceProfile, SelectionStrategy,
};
use crate::error::{MyQuatError, Result};
use crate::QuantumCircuit;
use std::collections::HashMap;
use std::sync::Arc;

/// Compute backend manager
pub struct ComputeBackendManager {
    /// Registered backends
    backends: HashMap<String, Arc<dyn ComputeBackend>>,
    /// Default backend name
    default_backend: Option<String>,
    /// Selection strategy
    selection_strategy: SelectionStrategy,
    /// Performance cache
    performance_cache: HashMap<String, PerformanceProfile>,
}

impl ComputeBackendManager {
    /// Create a new empty backend manager
    pub fn new() -> Self {
        ComputeBackendManager {
            backends: HashMap::new(),
            default_backend: None,
            selection_strategy: SelectionStrategy::Auto,
            performance_cache: HashMap::new(),
        }
    }

    /// Auto-detect and register all available backends
    pub fn auto_detect() -> Self {
        let mut manager = Self::new();

        // Register CPU backend (always available)
        use crate::compute::local::CpuBackend;
        if CpuBackend::is_available() {
            let backend = Arc::new(CpuBackend::new());
            manager.register(backend);
        }

        // Try to register SIMD backend
        use crate::compute::local::SimdBackend;
        if SimdBackend::is_available() {
            let backend = Arc::new(SimdBackend::new());
            manager.register(backend);
        }

        // Try to register Parallel backend
        use crate::compute::local::ParallelBackend;
        if ParallelBackend::is_available() {
            if let Ok(backend) = ParallelBackend::new() {
                manager.register(Arc::new(backend));
            }
        }

        // GPU backend would be registered when GPU support is added
        // Cloud backends would be registered from config/env
        // Not auto-detected due to authentication requirements

        manager
    }

    /// Register a backend
    pub fn register(&mut self, backend: Arc<dyn ComputeBackend>) {
        let name = backend.name().to_string();

        // Cache performance profile
        self.performance_cache
            .insert(name.clone(), backend.performance_profile());

        // Set as default if first backend
        if self.default_backend.is_none() {
            self.default_backend = Some(name.clone());
        }

        self.backends.insert(name, backend);
    }

    /// Get a backend by name
    pub fn get_backend(&self, name: &str) -> Result<&Arc<dyn ComputeBackend>> {
        self.backends
            .get(name)
            .ok_or_else(|| MyQuatError::circuit_error(format!("Backend '{}' not found", name)))
    }

    /// Get backend by name (optional)
    pub fn get_backend_opt(&self, name: &str) -> Option<&Arc<dyn ComputeBackend>> {
        self.backends.get(name)
    }

    /// List all available backends
    pub fn list_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    /// Set default backend
    pub fn set_default(&mut self, name: String) -> Result<()> {
        if !self.backends.contains_key(&name) {
            return Err(MyQuatError::circuit_error(format!(
                "Backend '{}' not registered",
                name
            )));
        }
        self.default_backend = Some(name);
        Ok(())
    }

    /// Set selection strategy
    pub fn set_selection_strategy(&mut self, strategy: SelectionStrategy) {
        self.selection_strategy = strategy;
    }

    /// Select backend based on circuit and hints
    pub fn select_backend(
        &self,
        circuit: &QuantumCircuit,
        hints: &ExecutionHints,
    ) -> Result<&Arc<dyn ComputeBackend>> {
        match &self.selection_strategy {
            SelectionStrategy::Manual(name) => self.get_backend(name),
            SelectionStrategy::Auto => self.auto_select(circuit, hints),
            SelectionStrategy::Benchmark => {
                // For now, fallback to auto
                // TODO: Implement runtime benchmarking
                self.auto_select(circuit, hints)
            }
        }
    }

    /// Auto-select backend based on heuristics
    fn auto_select(
        &self,
        circuit: &QuantumCircuit,
        hints: &ExecutionHints,
    ) -> Result<&Arc<dyn ComputeBackend>> {
        // Check for user preference
        if let Some(ref prefer) = hints.prefer_backend {
            if let Some(backend) = self.get_backend_opt(prefer) {
                if backend.is_available() && backend.is_compatible(circuit) {
                    return Ok(backend);
                }
            }
        }

        let num_qubits = circuit.num_qubits();
        let _gate_count = circuit.size();

        // Rule 1: Prefer cloud if explicitly requested
        if hints.prefer_cloud {
            return self.select_cloud_backend(circuit);
        }

        // Rule 2: Small circuits (<10 qubits) - prefer SIMD or CPU
        if num_qubits < 10 {
            // Try SIMD first
            if let Some(simd) = self.get_backend_opt("simd") {
                if simd.is_available() && simd.is_compatible(circuit) {
                    return Ok(simd);
                }
            }

            // Fallback to CPU
            if let Some(cpu) = self.get_backend_opt("cpu") {
                if cpu.is_compatible(circuit) {
                    return Ok(cpu);
                }
            }
        }

        // Rule 3: Medium circuits (10-20 qubits) - prefer GPU or Parallel
        if (10..20).contains(&num_qubits) {
            // Try GPU if allowed
            if hints.allow_gpu {
                if let Some(gpu) = self.get_backend_opt("gpu") {
                    if gpu.is_available() && gpu.is_compatible(circuit) {
                        return Ok(gpu);
                    }
                }
            }

            // Try parallel
            if let Some(parallel) = self.get_backend_opt("parallel") {
                if parallel.is_available() && parallel.is_compatible(circuit) {
                    return Ok(parallel);
                }
            }
        }

        // Rule 4: Large circuits (>=20 qubits) - GPU or cloud
        if num_qubits >= 20 {
            // Try GPU first
            if hints.allow_gpu {
                if let Some(gpu) = self.get_backend_opt("gpu") {
                    if gpu.is_available() && gpu.is_compatible(circuit) {
                        return Ok(gpu);
                    }
                }
            }

            // Try cloud
            if let Ok(cloud) = self.select_cloud_backend(circuit) {
                return Ok(cloud);
            }
        }

        // Fallback: use default backend
        if let Some(ref default) = self.default_backend {
            return self.get_backend(default);
        }

        Err(MyQuatError::circuit_error("No suitable backend found"))
    }

    /// Select a cloud backend
    fn select_cloud_backend(&self, circuit: &QuantumCircuit) -> Result<&Arc<dyn ComputeBackend>> {
        // Try to find any available cloud backend
        for backend in self.backends.values() {
            if let ComputeBackendType::Cloud(_) = backend.backend_type() {
                if backend.is_available() && backend.is_compatible(circuit) {
                    return Ok(backend);
                }
            }
        }

        Err(MyQuatError::circuit_error("No cloud backend available"))
    }

    /// Execute circuit using selected backend
    pub fn execute(
        &self,
        circuit: &QuantumCircuit,
        shots: usize,
        hints: &ExecutionHints,
    ) -> Result<ExecutionResult> {
        let backend = self.select_backend(circuit, hints)?;
        backend.execute(circuit, shots)
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        let mut summary = String::from("Available Backends:\n");
        summary.push_str(&format!("{:-<60}\n", ""));

        for (name, backend) in &self.backends {
            let profile = backend.performance_profile();
            let available = if backend.is_available() { "✓" } else { "✗" };

            summary.push_str(&format!(
                "{} {} - max_qubits: {}, ops/s: {:.2e}\n",
                available, name, profile.max_qubits, profile.ops_per_second
            ));
        }

        if let Some(ref default) = self.default_backend {
            summary.push_str(&format!("\nDefault: {}\n", default));
        }

        summary
    }
}

impl Default for ComputeBackendManager {
    fn default() -> Self {
        Self::auto_detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = ComputeBackendManager::new();
        assert!(manager.backends.is_empty());
        assert!(manager.default_backend.is_none());
    }

    #[test]
    fn test_auto_detect() {
        let manager = ComputeBackendManager::auto_detect();
        // Should have at least CPU backend if compiled with compute feature
        #[cfg(feature = "compute")]
        assert!(!manager.backends.is_empty());
    }
}
