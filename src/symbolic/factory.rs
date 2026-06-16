//! Symbolic Backend Factory
//!
//! Author: gA4ss
//!
//! This module provides a factory for creating symbolic computation backends
//! based on configuration.

use super::config::{BackendType, SymbolicConfig};
use super::mysym_adapter::MySymBackend;
use super::symbolica_adapter::SymbolicaBackend;

/// Enum to hold different backend types
///
/// This allows dynamic dispatch while maintaining type safety.
pub enum Backend {
    Symbolica(SymbolicaBackend),
    MySym(MySymBackend),
}

/// Create a symbolic backend based on configuration
///
/// # Arguments
/// * `config` - The configuration specifying which backend to use
///
/// # Returns
/// A backend instance wrapped in the Backend enum
///
/// # Examples
/// ```
/// use myquat::symbolic::{create_backend, SymbolicConfig, BackendType};
///
/// let config = SymbolicConfig::symbolica();
/// let backend = create_backend(&config);
/// ```
pub fn create_backend(config: &SymbolicConfig) -> Backend {
    match config.backend {
        BackendType::Symbolica => Backend::Symbolica(SymbolicaBackend::new()),
        BackendType::MySym => Backend::MySym(MySymBackend::new()),
    }
}

/// Create the default symbolic backend
///
/// This uses the default configuration (Symbolica backend).
///
/// # Examples
/// ```
/// use myquat::symbolic::create_default_backend;
///
/// let backend = create_default_backend();
/// ```
pub fn create_default_backend() -> Backend {
    create_backend(&SymbolicConfig::default())
}

/// Create a Symbolica backend
///
/// Convenience function for creating a Symbolica backend directly.
pub fn create_symbolica_backend() -> SymbolicaBackend {
    SymbolicaBackend::new()
}

/// Create a MySym backend (placeholder)
///
/// Note: This will return a placeholder that will error on operations.
/// Full implementation coming in Phase 11.
pub fn create_mysym_backend() -> MySymBackend {
    MySymBackend::new()
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
    fn test_create_symbolica_backend() {
        let config = SymbolicConfig::symbolica();
        let _backend = create_backend(&config);
        // Should create Symbolica backend
    }

    #[test]
    fn test_create_mysym_backend_placeholder() {
        let config = SymbolicConfig::mysym();
        let _backend = create_backend(&config);
        // Should create MySym placeholder
    }

    #[test]
    fn test_backend_creation_functions() {
        let _symbolica = create_symbolica_backend();
        let _mysym = create_mysym_backend();
        // Both should create successfully
    }
}
