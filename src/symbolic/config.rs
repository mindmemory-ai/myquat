//! Symbolic Backend Configuration
//!
//! Author: gA4ss
//!
//! This module provides configuration and selection mechanisms for symbolic computation backends.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Available symbolic computation backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackendType {
    /// MySym library backend (mysym v0.3.0 — default)
    #[default]
    MySym,
    /// Symbolica library backend (alternative)
    Symbolica,
}

impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendType::Symbolica => write!(f, "Symbolica"),
            BackendType::MySym => write!(f, "MySym"),
        }
    }
}

/// Configuration for symbolic computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicConfig {
    /// The backend to use for symbolic computation
    pub backend: BackendType,

    /// Enable automatic simplification of expressions
    pub auto_simplify: bool,

    /// Maximum recursion depth for symbolic operations
    pub max_recursion_depth: usize,

    /// Enable caching of computed expressions
    pub enable_caching: bool,
}

impl Default for SymbolicConfig {
    fn default() -> Self {
        Self {
            backend: BackendType::default(),
            auto_simplify: true,
            max_recursion_depth: 1000,
            enable_caching: true,
        }
    }
}

impl SymbolicConfig {
    /// Create a new configuration with the specified backend
    pub fn new(backend: BackendType) -> Self {
        Self {
            backend,
            ..Default::default()
        }
    }

    /// Create a configuration for Symbolica backend
    pub fn symbolica() -> Self {
        Self::new(BackendType::Symbolica)
    }

    /// Create a configuration for MySym backend (future)
    pub fn mysym() -> Self {
        Self::new(BackendType::MySym)
    }

    /// Set auto-simplification option
    pub fn with_auto_simplify(mut self, enable: bool) -> Self {
        self.auto_simplify = enable;
        self
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
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
        let config = SymbolicConfig::default();
        assert_eq!(config.backend, BackendType::MySym);
        assert!(config.auto_simplify);
        assert_eq!(config.max_recursion_depth, 1000);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_backend_types() {
        let symbolica = BackendType::Symbolica;
        let mysym = BackendType::MySym;

        assert_ne!(symbolica, mysym);
        assert_eq!(format!("{}", symbolica), "Symbolica");
        assert_eq!(format!("{}", mysym), "MySym");
    }

    #[test]
    fn test_config_builders() {
        let config = SymbolicConfig::symbolica()
            .with_auto_simplify(false)
            .with_max_recursion_depth(500)
            .with_caching(false);

        assert_eq!(config.backend, BackendType::Symbolica);
        assert!(!config.auto_simplify);
        assert_eq!(config.max_recursion_depth, 500);
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_mysym_config() {
        let config = SymbolicConfig::mysym();
        assert_eq!(config.backend, BackendType::MySym);
    }
}
