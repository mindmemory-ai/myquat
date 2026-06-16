//! Local Compute Backends
//! Author: gA4ss
//!
//! Local computation backends (CPU, SIMD, Parallel, GPU)

pub mod cpu_backend;
pub mod gpu_backend;
pub mod parallel_backend;
pub mod simd_backend;

#[cfg(feature = "cuda")]
pub mod cuda_backend;

pub use cpu_backend::CpuBackend;
pub use gpu_backend::{
    GpuAccelerationManager, GpuBackend, GpuCapability, GpuPerformanceModel, GpuPerformanceStats,
    GpuQuantumExecutor,
};
pub use parallel_backend::ParallelBackend;
pub use simd_backend::SimdBackend;

#[cfg(feature = "cuda")]
pub use cuda_backend::CudaBackend;
