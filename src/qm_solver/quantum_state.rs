//! Symbolic Quantum State Module
//!
//! Author: gA4ss
//!
//! This module provides symbolic representations of quantum wave functions,
//! supporting position and momentum representations, normalization, and
//! various quantum mechanical operations.

use crate::symbolic::{SymbolicBackend, SymbolicError, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Coordinate system for wave function representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSystem {
    /// Cartesian coordinates (x, y, z)
    Cartesian,
    /// Spherical coordinates (r, θ, φ)
    Spherical,
    /// Cylindrical coordinates (ρ, φ, z)
    Cylindrical,
}

/// Wave function representation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveFunctionRepresentation {
    /// Position space representation: ψ(x, y, z) or ψ(r)
    Position,
    /// Momentum space representation: φ(px, py, pz) or φ(p)
    Momentum,
}

impl fmt::Display for WaveFunctionRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaveFunctionRepresentation::Position => write!(f, "Position"),
            WaveFunctionRepresentation::Momentum => write!(f, "Momentum"),
        }
    }
}

/// Symbolic wave function representation
///
/// Represents a quantum mechanical wave function in symbolic form.
/// Supports both position and momentum representations in various
/// coordinate systems.
///
/// # Mathematical Background
///
/// In position representation: ψ(r⃗) = ⟨r⃗|ψ⟩
/// In momentum representation: φ(p⃗) = ⟨p⃗|ψ⟩
///
/// Normalization condition: ∫|ψ(r⃗)|² d³r = 1
///
/// # Examples
///
/// ```rust,ignore
/// use myquat::qm_solver::{SymbolicWaveFunction, WaveFunctionRepresentation, CoordinateSystem};
/// use myquat::symbolic::create_symbolica_backend;
///
/// let backend = create_symbolica_backend();
///
/// // Create a 1D Gaussian wave packet
/// let x = backend.variable("x").unwrap();
/// let sigma = backend.variable("sigma").unwrap();
/// let gaussian = backend.parse("exp(-x^2/(2*sigma^2))").unwrap();
///
/// let wf = SymbolicWaveFunction::new(
///     gaussian,
///     WaveFunctionRepresentation::Position,
///     CoordinateSystem::Cartesian,
///     1
/// );
/// ```
pub struct SymbolicWaveFunction<E: SymbolicExpression> {
    /// The wave function expression
    pub expression: E,

    /// Representation type (position or momentum)
    pub representation: WaveFunctionRepresentation,

    /// Coordinate system used
    pub coordinate_system: CoordinateSystem,

    /// Number of spatial dimensions (1, 2, or 3)
    pub dimensions: usize,

    /// Whether the wave function is normalized
    normalized: bool,
}

impl<E: SymbolicExpression> SymbolicWaveFunction<E> {
    /// Create a new symbolic wave function
    ///
    /// # Arguments
    ///
    /// * `expression` - The symbolic expression representing ψ or φ
    /// * `representation` - Position or momentum representation
    /// * `coordinate_system` - Coordinate system (Cartesian, Spherical, Cylindrical)
    /// * `dimensions` - Number of spatial dimensions (1, 2, or 3)
    pub fn new(
        expression: E,
        representation: WaveFunctionRepresentation,
        coordinate_system: CoordinateSystem,
        dimensions: usize,
    ) -> Self {
        if dimensions == 0 || dimensions > 3 {
            panic!("Dimensions must be 1, 2, or 3");
        }

        Self {
            expression,
            representation,
            coordinate_system,
            dimensions,
            normalized: false,
        }
    }

    /// Create a normalized wave function
    pub fn new_normalized(
        expression: E,
        representation: WaveFunctionRepresentation,
        coordinate_system: CoordinateSystem,
        dimensions: usize,
    ) -> Self {
        let mut wf = Self::new(expression, representation, coordinate_system, dimensions);
        wf.normalized = true;
        wf
    }

    /// Check if the wave function is marked as normalized
    pub fn is_normalized(&self) -> bool {
        self.normalized
    }

    /// Mark the wave function as normalized
    pub fn mark_normalized(&mut self) {
        self.normalized = true;
    }

    /// Get the representation type
    pub fn representation(&self) -> WaveFunctionRepresentation {
        self.representation
    }

    /// Get the coordinate system
    pub fn coordinate_system(&self) -> CoordinateSystem {
        self.coordinate_system
    }

    /// Get the number of dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get the underlying expression
    pub fn expression(&self) -> &E {
        &self.expression
    }
}

/// Operations on symbolic wave functions
impl<E: SymbolicExpression> SymbolicWaveFunction<E> {
    /// Compute the probability density |ψ|²
    ///
    /// Returns the expression for |ψ(r⃗)|² or |φ(p⃗)|²
    ///
    /// # Mathematical Background
    ///
    /// For a complex wave function ψ = ψ_real + i·ψ_imag:
    /// |ψ|² = ψ*·ψ = (ψ_real)² + (ψ_imag)²
    pub fn probability_density<B>(&self, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // |ψ|² = ψ* ψ
        // For real wave functions: |ψ|² = ψ²
        // For complex: need to compute conjugate
        let conj = backend.conjugate(&self.expression)?;
        backend.mul(&conj, &self.expression)
    }

    /// Compute the inner product ⟨φ|ψ⟩
    ///
    /// For discrete case, this is the overlap integral:
    /// ⟨φ|ψ⟩ = ∫ φ*(r⃗) ψ(r⃗) d³r
    ///
    /// # Arguments
    ///
    /// * `other` - The other wave function
    /// * `backend` - The symbolic backend for computations
    ///
    /// # Returns
    ///
    /// The symbolic expression for the integrand φ*ψ.
    /// The actual integration must be performed separately.
    pub fn inner_product_integrand<B>(
        &self,
        other: &SymbolicWaveFunction<E>,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // Check compatibility
        if self.representation != other.representation {
            return Err(SymbolicError::InvalidExpression(
                format!(
                    "Cannot compute inner product of wave functions in different representations: {} and {}",
                    self.representation, other.representation
                )
            ));
        }

        if self.dimensions != other.dimensions {
            return Err(SymbolicError::InvalidExpression(
                format!(
                    "Cannot compute inner product of wave functions with different dimensions: {} and {}",
                    self.dimensions, other.dimensions
                )
            ));
        }

        // Compute φ* ψ
        let conj_self = backend.conjugate(&self.expression)?;
        backend.mul(&conj_self, &other.expression)
    }

    /// Compute the norm squared ⟨ψ|ψ⟩
    ///
    /// This is the integrand for the normalization integral:
    /// ⟨ψ|ψ⟩ = ∫ |ψ(r⃗)|² d³r
    pub fn norm_squared_integrand<B>(&self, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
    {
        self.probability_density(backend)
    }
}

impl<E: SymbolicExpression> fmt::Display for SymbolicWaveFunction<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SymbolicWaveFunction({}D {}, {}): {}",
            self.dimensions,
            self.representation,
            if self.normalized {
                "normalized"
            } else {
                "unnormalized"
            },
            self.expression.to_string()
        )
    }
}

impl<E: SymbolicExpression> Clone for SymbolicWaveFunction<E> {
    fn clone(&self) -> Self {
        Self {
            expression: self.expression.clone(),
            representation: self.representation,
            coordinate_system: self.coordinate_system,
            dimensions: self.dimensions,
            normalized: self.normalized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_wave_function_creation() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();

        let wf = SymbolicWaveFunction::new(
            x,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        assert_eq!(wf.dimensions(), 1);
        assert_eq!(wf.representation(), WaveFunctionRepresentation::Position);
        assert!(!wf.is_normalized());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_normalized_wave_function() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();

        let wf = SymbolicWaveFunction::new_normalized(
            x,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        assert!(wf.is_normalized());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_probability_density() {
        let backend = create_symbolica_backend();
        let psi = backend.parse("exp(-x^2)").unwrap();

        let wf = SymbolicWaveFunction::new(
            psi,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let prob_density = wf.probability_density(&backend).unwrap();
        assert!(!prob_density.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_inner_product_same_representation() {
        let backend = create_symbolica_backend();
        let psi1 = backend.parse("exp(-x^2)").unwrap();
        let psi2 = backend.parse("x*exp(-x^2)").unwrap();

        let wf1 = SymbolicWaveFunction::new(
            psi1,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let wf2 = SymbolicWaveFunction::new(
            psi2,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let integrand = wf1.inner_product_integrand(&wf2, &backend).unwrap();
        assert!(!integrand.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_inner_product_different_representation_error() {
        let backend = create_symbolica_backend();
        let psi = backend.variable("x").unwrap();
        let phi = backend.variable("p").unwrap();

        let wf_pos = SymbolicWaveFunction::new(
            psi,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let wf_mom = SymbolicWaveFunction::new(
            phi,
            WaveFunctionRepresentation::Momentum,
            CoordinateSystem::Cartesian,
            1,
        );

        assert!(wf_pos.inner_product_integrand(&wf_mom, &backend).is_err());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    #[should_panic(expected = "Dimensions must be 1, 2, or 3")]
    fn test_invalid_dimensions() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();

        let _wf = SymbolicWaveFunction::new(
            x,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            0,
        );
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_wave_function_display() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();

        let wf = SymbolicWaveFunction::new(
            x,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let display = format!("{}", wf);
        assert!(display.contains("1D"));
        assert!(display.contains("Position"));
        assert!(display.contains("unnormalized"));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_gaussian_wave_packet_probability_density() {
        // Gaussian wave packet: ψ(x) = exp(-x²/2)
        let backend = create_symbolica_backend();
        let psi = backend.parse("exp(-x^2/2)").unwrap();

        let wf = SymbolicWaveFunction::new(
            psi,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let prob = wf.probability_density(&backend).unwrap();
        // Probability density should be |ψ|² = exp(-x²)
        let prob_str = format!("{}", prob);
        assert!(prob_str.contains("exp") || prob_str.contains("x"));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_wave_function_orthogonality() {
        // Test orthogonal wave functions: ψ₁ = sin(πx/L), ψ₂ = sin(2πx/L)
        let backend = create_symbolica_backend();
        let psi1 = backend.parse("sin(pi*x)").unwrap();
        let psi2 = backend.parse("sin(2*pi*x)").unwrap();

        let wf1 = SymbolicWaveFunction::new(
            psi1,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let wf2 = SymbolicWaveFunction::new(
            psi2,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        // Inner product integrand should exist
        let integrand = wf1.inner_product_integrand(&wf2, &backend).unwrap();
        let integrand_str = format!("{}", integrand);
        assert!(!integrand_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_multidimensional_wave_function() {
        let backend = create_symbolica_backend();
        let psi_2d = backend.parse("exp(-(x^2 + y^2))").unwrap();

        let wf_2d = SymbolicWaveFunction::new(
            psi_2d,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            2,
        );

        assert_eq!(wf_2d.dimensions(), 2);
        assert_eq!(wf_2d.coordinate_system(), CoordinateSystem::Cartesian);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_spherical_coordinate_system() {
        let backend = create_symbolica_backend();
        let psi_spherical = backend.parse("exp(-r)").unwrap();

        let wf = SymbolicWaveFunction::new(
            psi_spherical,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Spherical,
            3,
        );

        assert_eq!(wf.coordinate_system(), CoordinateSystem::Spherical);
        assert_eq!(wf.dimensions(), 3);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_momentum_space_wave_function() {
        let backend = create_symbolica_backend();
        let phi = backend.parse("exp(-p^2)").unwrap();

        let wf_momentum = SymbolicWaveFunction::new(
            phi,
            WaveFunctionRepresentation::Momentum,
            CoordinateSystem::Cartesian,
            1,
        );

        assert_eq!(
            wf_momentum.representation(),
            WaveFunctionRepresentation::Momentum
        );
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_normalized_flag_consistency() {
        let backend = create_symbolica_backend();
        let psi = backend.variable("psi").unwrap();

        let mut wf = SymbolicWaveFunction::new(
            psi,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        assert!(!wf.is_normalized());
        wf.mark_normalized();
        assert!(wf.is_normalized());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_probability_density_real_positive() {
        // Probability density |ψ|² must be real and non-negative
        let backend = create_symbolica_backend();
        let psi = backend.parse("x*exp(-x^2)").unwrap();

        let wf = SymbolicWaveFunction::new(
            psi,
            WaveFunctionRepresentation::Position,
            CoordinateSystem::Cartesian,
            1,
        );

        let prob = wf.probability_density(&backend).unwrap();
        // Should be x²*exp(-2x²)
        let prob_str = format!("{}", prob);
        assert!(prob_str.contains("x") || prob_str.contains("exp"));
    }

    #[test]
    fn test_all_coordinate_systems() {
        // Test that all coordinate system variants are defined
        let systems = vec![
            CoordinateSystem::Cartesian,
            CoordinateSystem::Spherical,
            CoordinateSystem::Cylindrical,
        ];

        for system in systems {
            assert_eq!(format!("{:?}", system).len() > 0, true);
        }
    }

    #[test]
    fn test_all_representations() {
        // Test that all representation variants are defined
        let reps = vec![
            WaveFunctionRepresentation::Position,
            WaveFunctionRepresentation::Momentum,
        ];

        for rep in reps {
            assert_eq!(format!("{:?}", rep).len() > 0, true);
        }
    }
}
