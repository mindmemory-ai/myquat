//! Linear Algebra Backend Configuration
//!
//! Author: gA4ss
//!
//! This module provides configuration and selection mechanisms for linear
//! algebra computation backends, mirroring `src/symbolic/config.rs`.

use serde::{Deserialize, Serialize};

/// Available linear algebra computation backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinalgBackendType {
    /// MyMat library backend (mymat v0.3.3 — default)
    #[default]
    MyMat,
    /// ndarray + nalgebra backend (alternative)
    NdArray,
}

impl std::fmt::Display for LinalgBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinalgBackendType::NdArray => write!(f, "NdArray"),
            LinalgBackendType::MyMat => write!(f, "MyMat"),
        }
    }
}

/// Configuration for linear algebra computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinalgConfig {
    /// The backend to use for linear algebra operations
    pub backend: LinalgBackendType,

    /// Reserved for future: enable caching of computed results (not yet wired)
    pub enable_caching: bool,
}

impl Default for LinalgConfig {
    fn default() -> Self {
        Self {
            backend: LinalgBackendType::default(),
            enable_caching: true,
        }
    }
}

impl LinalgConfig {
    /// Create a new configuration with the specified backend
    pub fn new(backend: LinalgBackendType) -> Self {
        Self {
            backend,
            ..Default::default()
        }
    }

    /// Create a configuration for the NdArray backend
    pub fn ndarray() -> Self {
        Self::new(LinalgBackendType::NdArray)
    }

    /// Create a configuration for the MyMat backend
    pub fn mymat() -> Self {
        Self::new(LinalgBackendType::MyMat)
    }

    /// Set caching option
    pub fn with_caching(mut self, enable: bool) -> Self {
        self.enable_caching = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LinalgConfig::default();
        assert_eq!(config.backend, LinalgBackendType::MyMat);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_backend_types() {
        let ndarray = LinalgBackendType::NdArray;
        let mymat = LinalgBackendType::MyMat;
        assert_ne!(ndarray, mymat);
        assert_eq!(format!("{}", ndarray), "NdArray");
        assert_eq!(format!("{}", mymat), "MyMat");
    }

    #[test]
    fn test_config_builders() {
        let config = LinalgConfig::ndarray().with_caching(false);
        assert_eq!(config.backend, LinalgBackendType::NdArray);
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_mymat_config() {
        let config = LinalgConfig::mymat();
        assert_eq!(config.backend, LinalgBackendType::MyMat);
    }
}
