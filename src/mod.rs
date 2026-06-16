//! Utility and performance optimization components
//! 
//! This module contains performance optimization tools, memory management,
//! parallel processing, and hardware acceleration utilities.

pub mod utils;
pub mod performance_config;
pub mod memory_optimized;
pub mod memory_pool;
pub mod matrix_cache;
pub mod parallel;
pub mod simd_ops;
pub mod gpu_accel;

// Re-export commonly used types
pub use performance_config::{PerformanceConfig, OptimizationLevel, HardwareConfig};
pub use memory_optimized::{MemoryEfficientState, ZeroCopyMatrixOps, MemoryStats};
pub use memory_pool::{QuantumMemoryPool, Array1Pool, Array2Pool, GlobalPoolStats};
pub use matrix_cache::{MatrixCache, MatrixCacheKey, CacheStats, global_matrix_cache};
pub use parallel::{ParallelQuantumExecutor, ParallelStateOps, ParallelCircuitAnalysis};
pub use simd_ops::{SimdQuantumOps, AdaptiveSimdOps};
pub use gpu_accel::{GpuAccelerationManager, GpuQuantumExecutor, GpuCapability, GpuBackend};
