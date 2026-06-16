//! Symbolic Computation Module
//!
//! Author: gA4ss
//!
//! This module provides symbolic computation capabilities for quantum mechanics solving.
//! It defines an abstract interface and provides concrete implementations via adapters
//! to external symbolic computation libraries.
//!
//! # Architecture
//!
//! The module uses a strategy pattern with multiple backends:
//! - **Symbolica**: Full-featured symbolic backend (current default)
//! - **MySym**: Custom symbolic backend (coming in Phase 11)
//!
//! # Examples
//!
//! ```
//! use myquat::symbolic::{SymbolicConfig, create_backend, SymbolicBackend};
//!
//! // Create a Symbolica backend with default configuration
//! let config = SymbolicConfig::symbolica();
//! let backend = create_backend(&config);
//! ```

pub mod backend;
pub mod config;
pub mod factory;
pub mod mysym_adapter;
pub mod symbolica_adapter;

#[cfg(test)]
mod tests;

// Re-export core types from backend
pub use backend::{
    SubstitutionMap, SymbolicBackend, SymbolicError, SymbolicExpression, SymbolicMatrix,
    SymbolicResult, SymbolicVariable,
};

// Re-export configuration and factory
pub use config::{BackendType, SymbolicConfig};
pub use factory::{
    create_backend, create_default_backend, create_mysym_backend, create_symbolica_backend, Backend,
};

// Re-export backend implementations
pub use mysym_adapter::MySymBackend;
pub use symbolica_adapter::SymbolicaBackend;

/// Create a default symbolic backend instance (convenience function)
///
/// Returns a Symbolica backend with default configuration.
/// For more control, use `create_backend()` with a `SymbolicConfig`.
///
/// # Examples
///
/// ```
/// use myquat::symbolic::default_backend;
///
/// let backend = default_backend();
/// ```
pub fn default_backend() -> SymbolicaBackend {
    SymbolicaBackend::new()
}
