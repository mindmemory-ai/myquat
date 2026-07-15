//! Linear Algebra Backend Factory
//!
//! Author: gA4ss
//!
//! This module provides a factory for creating linear algebra computation
//! backends based on configuration, mirroring `src/symbolic/factory.rs`.

use super::config::{LinalgBackendType, LinalgConfig};
use super::mymat_adapter::MyMatBackend;
use super::ndarray_adapter::NdArrayBackend;

/// Enum to hold different backend types.
///
/// This allows dynamic dispatch while maintaining type safety.
pub enum LinalgBackendImpl {
    /// ndarray + nalgebra backend
    NdArray(NdArrayBackend),
    /// MyMat backend (mymat-backed, default)
    MyMat(MyMatBackend),
}

/// Create a linear algebra backend based on configuration
///
/// # Arguments
/// * `config` - The configuration specifying which backend to use
///
/// # Returns
/// A backend instance wrapped in the LinalgBackendImpl enum
pub fn create_backend(config: &LinalgConfig) -> LinalgBackendImpl {
    match config.backend {
        LinalgBackendType::NdArray => LinalgBackendImpl::NdArray(NdArrayBackend::new()),
        LinalgBackendType::MyMat => LinalgBackendImpl::MyMat(MyMatBackend::new()),
    }
}

/// Create the default linear algebra backend (MyMat, backed by mymat)
pub fn create_default_backend() -> LinalgBackendImpl {
    create_backend(&LinalgConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_backend() {
        let _backend = create_default_backend();
        // Should not panic
    }

    #[test]
    fn test_create_ndarray_backend() {
        let config = LinalgConfig::ndarray();
        let _backend = create_backend(&config);
    }

    #[test]
    fn test_create_mymat_backend() {
        let config = LinalgConfig::mymat();
        let _backend = create_backend(&config);
    }
}
