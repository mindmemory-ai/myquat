//! Cloud Compute Backends
//! Author: gA4ss
//!
//! Cloud quantum computing backends for IBM, AWS, Azure, and Google

pub mod aws_backend;
pub mod ibm_backend;
pub mod job_manager;

pub use aws_backend::{
    AwsBraketBackend, BraketDevice, BraketDeviceType, DeviceCapabilities, TaskStatus,
};
pub use ibm_backend::{BackendInfo, IbmQuantumBackend, JobStatus};
pub use job_manager::{CacheStats, JobManager, JobQueue, QueueStatus, RetryConfig};
