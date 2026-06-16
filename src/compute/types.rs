//! Compute Backend Types
//! Author: gA4ss
//!
//! Core types and enums for compute backend abstraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of compute backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeBackendType {
    /// Local computation on CPU/GPU
    Local(LocalBackendVariant),
    /// Cloud quantum computing service
    Cloud(CloudBackendVariant),
}

impl std::fmt::Display for ComputeBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeBackendType::Local(variant) => write!(f, "local:{}", variant),
            ComputeBackendType::Cloud(variant) => write!(f, "cloud:{}", variant),
        }
    }
}

/// Local backend variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocalBackendVariant {
    /// Pure CPU (single-threaded)
    CPU,
    /// SIMD-accelerated (AVX2/NEON)
    SIMD,
    /// Multi-threaded parallel execution
    Parallel,
    /// GPU-accelerated
    GPU(GpuType),
}

impl std::fmt::Display for LocalBackendVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalBackendVariant::CPU => write!(f, "cpu"),
            LocalBackendVariant::SIMD => write!(f, "simd"),
            LocalBackendVariant::Parallel => write!(f, "parallel"),
            LocalBackendVariant::GPU(gpu_type) => write!(f, "gpu_{}", gpu_type),
        }
    }
}

/// GPU types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuType {
    /// NVIDIA CUDA
    CUDA,
    /// OpenCL
    OpenCL,
    /// WebGPU compute shaders
    ComputeShader,
    /// Apple Metal
    Metal,
}

impl std::fmt::Display for GpuType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuType::CUDA => write!(f, "cuda"),
            GpuType::OpenCL => write!(f, "opencl"),
            GpuType::ComputeShader => write!(f, "compute_shader"),
            GpuType::Metal => write!(f, "metal"),
        }
    }
}

/// Cloud backend variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudBackendVariant {
    /// IBM Quantum
    IBMQuantum,
    /// AWS Braket
    AWSBraket,
    /// Azure Quantum
    AzureQuantum,
    /// Google Quantum AI
    GoogleQuantum,
}

impl std::fmt::Display for CloudBackendVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudBackendVariant::IBMQuantum => write!(f, "ibm_quantum"),
            CloudBackendVariant::AWSBraket => write!(f, "aws_braket"),
            CloudBackendVariant::AzureQuantum => write!(f, "azure_quantum"),
            CloudBackendVariant::GoogleQuantum => write!(f, "google_quantum"),
        }
    }
}

/// Performance profile of a compute backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Maximum number of qubits supported
    pub max_qubits: usize,
    /// Estimated operations per second
    pub ops_per_second: f64,
    /// Startup overhead in milliseconds
    pub startup_overhead_ms: f64,
    /// Memory overhead per qubit (bytes)
    pub memory_per_qubit_bytes: usize,
    /// Whether the backend supports batching
    pub supports_batching: bool,
    /// Optimal batch size (if batching supported)
    pub optimal_batch_size: Option<usize>,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        PerformanceProfile {
            max_qubits: 25,
            ops_per_second: 1e6,
            startup_overhead_ms: 0.0,
            memory_per_qubit_bytes: 16, // 2^n * 16 bytes per complex64
            supports_batching: false,
            optimal_batch_size: None,
        }
    }
}

/// Execution hints for backend selection
#[derive(Debug, Clone, Default)]
pub struct ExecutionHints {
    /// Prefer cloud backend
    pub prefer_cloud: bool,
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Precision requirement
    pub precision: Precision,
    /// Allow GPU acceleration
    pub allow_gpu: bool,
    /// Prefer specific backend by name
    pub prefer_backend: Option<String>,
}

/// Precision requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Precision {
    /// Low precision (faster)
    Low,
    /// Medium precision
    #[default]
    Medium,
    /// High precision (slower)
    High,
}

/// Execution result from a compute backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Measurement counts (basis state -> count)
    pub counts: HashMap<String, usize>,
    /// Total number of shots
    pub shots: usize,
    /// Backend used for execution
    pub backend_used: String,
    /// Execution time in milliseconds
    pub execution_time_ms: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ExecutionResult {
    /// Create a new execution result
    pub fn new(backend_name: String, shots: usize) -> Self {
        ExecutionResult {
            counts: HashMap::new(),
            shots,
            backend_used: backend_name,
            execution_time_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Add a measurement result
    pub fn add_measurement(&mut self, state: String) {
        *self.counts.entry(state).or_insert(0) += 1;
    }

    /// Get probability of a specific outcome
    pub fn probability(&self, state: &str) -> f64 {
        self.counts.get(state).copied().unwrap_or(0) as f64 / self.shots as f64
    }

    /// Get most likely outcome
    pub fn most_likely(&self) -> Option<(String, usize)> {
        self.counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(state, &count)| (state.clone(), count))
    }
}

/// Backend selection strategy
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SelectionStrategy {
    /// Manually specify backend name
    Manual(String),
    /// Automatically select based on circuit properties
    #[default]
    Auto,
    /// Run benchmarks and select fastest
    Benchmark,
}
