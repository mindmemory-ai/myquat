//! Compute Backend Framework
//! Author: gA4ss
//!
//! Unified compute backend abstraction supporting CPU, SIMD, GPU, and cloud backends

pub mod backend_manager;
pub mod backend_trait;
pub mod cloud;
pub mod cloud_config;
pub mod local;
pub mod parallel_analysis;
pub mod parallel_ops;
pub mod parallel_simulator;
pub mod simd_ops;
pub mod simd_two_qubit_gates;
pub mod types;

// Re-export commonly used types
pub use backend_manager::ComputeBackendManager;
pub use backend_trait::ComputeBackend;
pub use types::{
    CloudBackendVariant, ComputeBackendType, ExecutionHints, ExecutionResult, GpuType,
    LocalBackendVariant, PerformanceProfile, Precision, SelectionStrategy,
};

// Re-export local backends
pub use local::{
    CpuBackend, GpuAccelerationManager, GpuBackend, GpuCapability, GpuPerformanceModel,
    GpuPerformanceStats, GpuQuantumExecutor, ParallelBackend, SimdBackend,
};

// Re-export parallel utilities
pub use parallel_analysis::ParallelCircuitAnalysis;
pub use parallel_ops::ParallelStateOps;
pub use parallel_simulator::{ParallelDensityMatrixSimulator, ParallelStateVectorSimulator};

// Re-export SIMD operations
pub use simd_ops::{AdaptiveSimdOps, SimdQuantumOps};
pub use simd_two_qubit_gates::SimdTwoQubitGates;

// Re-export cloud configuration
pub use cloud_config::{
    AwsBraketConfig, AzureQuantumConfig, CloudConfig, GlobalConfig, GoogleQuantumConfig,
    IbmQuantumConfig, JobManagementConfig, LoggingConfig, PreferencesConfig,
};

// Re-export cloud backends
pub use cloud::{
    AwsBraketBackend, BackendInfo, BraketDevice, BraketDeviceType, CacheStats, DeviceCapabilities,
    IbmQuantumBackend, JobManager, JobQueue, JobStatus, QueueStatus, RetryConfig, TaskStatus,
};
