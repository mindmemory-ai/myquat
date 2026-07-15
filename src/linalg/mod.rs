//! Linear Algebra Backend Module
//!
//! Author: gA4ss
//!
//! This module provides a trait-based abstraction layer for linear algebra
//! operations, mirroring the architecture of `src/symbolic/`.
//!
//! # Quick Start
//!
//! ```
//! use myquat::linalg::{LinalgBackend, NdArrayBackend};
//!
//! let backend = NdArrayBackend::new();
//! let identity = backend.complex_identity(2).unwrap();
//! ```
//!
//! # Backends
//!
//! - `NdArrayBackend` — Production backend wrapping ndarray + nalgebra
//! - `MyMatBackend` — mymat-backed backend (default)

pub mod backend;
pub mod config;
pub mod factory;
pub mod mymat_adapter;
pub mod ndarray_adapter;

#[cfg(test)]
mod tests;

pub use backend::{LinalgBackend, LinalgError, LinalgResult, LinalgScalar, SchurResult, SvdResult};
pub use config::{LinalgBackendType, LinalgConfig};
pub use factory::{create_backend, create_default_backend, LinalgBackendImpl};
pub use mymat_adapter::MyMatBackend;
pub use ndarray_adapter::NdArrayBackend;

/// Return a default MyMatBackend instance
pub fn default_backend() -> MyMatBackend {
    MyMatBackend::new()
}
