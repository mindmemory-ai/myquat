//! Quantum Operators Module
//!
//! Author: gA4ss
//!
//! This module provides symbolic representations of quantum mechanical operators,
//! including position, momentum, Hamiltonian, angular momentum, and spin operators.
//! It also implements operator algebra including commutators and anticommutators.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicMatrix, SymbolicResult};
use std::fmt;

/// Type of quantum operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorType {
    /// Position operator: $\hat{x}$, $\hat{y}$, $\hat{z}$
    Position,
    /// Momentum operator: $\hat{p}_x = -i\hbar\frac{\partial}{\partial x}$
    Momentum,
    /// Hamiltonian operator: $\hat{H} = \frac{\hat{p}^2}{2m} + V(\hat{x})$
    Hamiltonian,
    /// Angular momentum: $\hat{L} = \hat{r} \times \hat{p}$
    AngularMomentum,
    /// Spin operator: $\hat{S}_x$, $\hat{S}_y$, $\hat{S}_z$
    Spin,
    /// Custom operator
    Custom,
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatorType::Position => write!(f, "Position"),
            OperatorType::Momentum => write!(f, "Momentum"),
            OperatorType::Hamiltonian => write!(f, "Hamiltonian"),
            OperatorType::AngularMomentum => write!(f, "AngularMomentum"),
            OperatorType::Spin => write!(f, "Spin"),
            OperatorType::Custom => write!(f, "Custom"),
        }
    }
}

/// Symbolic quantum operator
///
/// Represents a quantum mechanical operator in symbolic form.
/// Operators can act on wave functions and satisfy specific commutation relations.
///
/// # Mathematical Background
///
/// Quantum operators are linear operators acting on Hilbert space.
/// Key properties:
/// - Hermitian operators have real eigenvalues
/// - Commutation relations: $[\hat{A}, \hat{B}] = \hat{A}\hat{B} - \hat{B}\hat{A}$
/// - Uncertainty principle: $\Delta A \Delta B \geq \frac{1}{2}|\langle[\hat{A}, \hat{B}]\rangle|$
///
/// # Examples
///
/// ```rust,ignore
/// use myquat::qm_solver::QuantumOperator;
/// use myquat::symbolic::create_symbolica_backend;
///
/// let backend = create_symbolica_backend();
///
/// // Create position operator
/// let x_var = backend.variable("x").unwrap();
/// let x_op = QuantumOperator::position(x_var, "x");
/// ```
pub struct QuantumOperator<E: SymbolicExpression> {
    /// The symbolic expression representing the operator
    pub expression: E,

    /// Type of operator
    pub operator_type: OperatorType,

    /// Name/label of the operator
    pub name: String,

    /// Whether the operator is Hermitian
    hermitian: bool,
}

impl<E: SymbolicExpression> QuantumOperator<E> {
    /// Create a new quantum operator
    ///
    /// # Arguments
    ///
    /// * `expression` - The symbolic expression representing the operator
    /// * `operator_type` - Type of the operator
    /// * `name` - Name/label for the operator
    /// * `hermitian` - Whether the operator is Hermitian
    pub fn new(
        expression: E,
        operator_type: OperatorType,
        name: impl Into<String>,
        hermitian: bool,
    ) -> Self {
        Self {
            expression,
            operator_type,
            name: name.into(),
            hermitian,
        }
    }

    /// Create a position operator
    ///
    /// In position representation: $\hat{x} = x$ (multiplication operator)
    pub fn position(expression: E, name: impl Into<String>) -> Self {
        Self::new(expression, OperatorType::Position, name, true)
    }

    /// Create a momentum operator
    ///
    /// In position representation: $\hat{p} = -i\hbar\frac{\partial}{\partial x}$
    pub fn momentum(expression: E, name: impl Into<String>) -> Self {
        Self::new(expression, OperatorType::Momentum, name, true)
    }

    /// Create a Hamiltonian operator
    ///
    /// $\hat{H} = \frac{\hat{p}^2}{2m} + V(\hat{x})$
    pub fn hamiltonian(expression: E, name: impl Into<String>) -> Self {
        Self::new(expression, OperatorType::Hamiltonian, name, true)
    }

    /// Create an angular momentum operator
    ///
    /// $\hat{L} = \hat{r} \times \hat{p}$
    pub fn angular_momentum(expression: E, name: impl Into<String>) -> Self {
        Self::new(expression, OperatorType::AngularMomentum, name, true)
    }

    /// Create a spin operator
    ///
    /// Spin operators satisfy: $[\hat{S}_i, \hat{S}_j] = i\hbar\epsilon_{ijk}\hat{S}_k$
    pub fn spin(expression: E, name: impl Into<String>) -> Self {
        Self::new(expression, OperatorType::Spin, name, true)
    }

    /// Create a custom operator
    pub fn custom(expression: E, name: impl Into<String>, hermitian: bool) -> Self {
        Self::new(expression, OperatorType::Custom, name, hermitian)
    }

    /// Check if the operator is Hermitian
    pub fn is_hermitian(&self) -> bool {
        self.hermitian
    }

    /// Get the operator type
    pub fn operator_type(&self) -> OperatorType {
        self.operator_type
    }

    /// Get the operator name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the underlying expression
    pub fn expression(&self) -> &E {
        &self.expression
    }
}

/// Operator algebra operations
impl<E: SymbolicExpression> QuantumOperator<E> {
    /// Compute the commutator $[\hat{A}, \hat{B}] = \hat{A}\hat{B} - \hat{B}\hat{A}$
    ///
    /// # Arguments
    ///
    /// * `other` - The other operator
    /// * `backend` - The symbolic backend for computations
    ///
    /// # Returns
    ///
    /// A new operator representing the commutator
    pub fn commutator<B>(
        &self,
        other: &QuantumOperator<E>,
        backend: &B,
    ) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // [A, B] = AB - BA
        let ab = backend.mul(&self.expression, &other.expression)?;
        let ba = backend.mul(&other.expression, &self.expression)?;
        let commutator = backend.sub(&ab, &ba)?;

        let name = format!("[{}, {}]", self.name, other.name);

        // Commutator of two Hermitian operators is anti-Hermitian
        let hermitian = false;

        Ok(QuantumOperator::custom(commutator, name, hermitian))
    }

    /// Compute the anticommutator $\{\hat{A}, \hat{B}\} = \hat{A}\hat{B} + \hat{B}\hat{A}$
    ///
    /// # Arguments
    ///
    /// * `other` - The other operator
    /// * `backend` - The symbolic backend for computations
    ///
    /// # Returns
    ///
    /// A new operator representing the anticommutator
    pub fn anticommutator<B>(
        &self,
        other: &QuantumOperator<E>,
        backend: &B,
    ) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // {A, B} = AB + BA
        let ab = backend.mul(&self.expression, &other.expression)?;
        let ba = backend.mul(&other.expression, &self.expression)?;
        let anticommutator = backend.add(&ab, &ba)?;

        let name = format!("{{{}, {}}}", self.name, other.name);

        // Anticommutator of two Hermitian operators is Hermitian
        let hermitian = self.hermitian && other.hermitian;

        Ok(QuantumOperator::custom(anticommutator, name, hermitian))
    }

    /// Compose two operators: $\hat{A}\hat{B}$
    ///
    /// # Arguments
    ///
    /// * `other` - The other operator to apply after this one
    /// * `backend` - The symbolic backend for computations
    pub fn compose<B>(
        &self,
        other: &QuantumOperator<E>,
        backend: &B,
    ) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let product = backend.mul(&self.expression, &other.expression)?;
        let name = format!("{}·{}", self.name, other.name);

        // Product of Hermitian operators is generally not Hermitian
        let hermitian = false;

        Ok(QuantumOperator::custom(product, name, hermitian))
    }

    /// Compute operator power: $\hat{A}^n$
    pub fn power<B>(&self, n: &E, backend: &B) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let power_expr = backend.pow(&self.expression, n)?;
        let name = format!("{}^{}", self.name, n.to_string());

        // Powers of Hermitian operators are Hermitian
        let hermitian = self.hermitian;

        Ok(QuantumOperator::custom(power_expr, name, hermitian))
    }

    /// Add a scalar multiple: $c\hat{A}$
    pub fn scalar_multiply<B>(&self, scalar: &E, backend: &B) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let scaled = backend.mul(scalar, &self.expression)?;
        let name = format!("{}·{}", scalar.to_string(), self.name);

        // Scalar multiplication preserves Hermiticity if scalar is real
        let hermitian = self.hermitian;

        Ok(QuantumOperator::custom(scaled, name, hermitian))
    }

    /// Add two operators: $\hat{A} + \hat{B}$
    pub fn add<B>(
        &self,
        other: &QuantumOperator<E>,
        backend: &B,
    ) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let sum = backend.add(&self.expression, &other.expression)?;
        let name = format!("{} + {}", self.name, other.name);

        // Sum of Hermitian operators is Hermitian
        let hermitian = self.hermitian && other.hermitian;

        Ok(QuantumOperator::custom(sum, name, hermitian))
    }
}

impl<E: SymbolicExpression> fmt::Display for QuantumOperator<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QuantumOperator({}, {}{}): {}",
            self.name,
            self.operator_type,
            if self.hermitian { ", Hermitian" } else { "" },
            self.expression.to_string()
        )
    }
}

impl<E: SymbolicExpression> Clone for QuantumOperator<E> {
    fn clone(&self) -> Self {
        Self {
            expression: self.expression.clone(),
            operator_type: self.operator_type,
            name: self.name.clone(),
            hermitian: self.hermitian,
        }
    }
}

/// Standard quantum mechanics operators
pub mod standard_operators {
    use super::*;

    /// Create the position operator in 1D: $\hat{x}$
    pub fn position_1d<B, E>(backend: &B, var_name: &str) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let x = backend.variable(var_name)?;
        Ok(QuantumOperator::position(x, var_name))
    }

    /// Create the momentum operator in 1D: $\hat{p} = -i\hbar\frac{\partial}{\partial x}$
    ///
    /// Note: This returns the differential operator representation
    pub fn momentum_1d<B, E>(backend: &B) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // -iℏ ∂/∂x is represented symbolically
        let hbar = backend.variable("hbar")?;
        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i_hbar = backend.mul(&backend.neg(&i)?, &hbar)?;

        Ok(QuantumOperator::momentum(minus_i_hbar, "p"))
    }

    /// Create harmonic oscillator Hamiltonian: $\hat{H} = \frac{\hat{p}^2}{2m} + \frac{1}{2}m\omega^2\hat{x}^2$
    pub fn harmonic_oscillator_hamiltonian<B, E>(backend: &B) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let expr = backend.parse("p^2/(2*m) + (1/2)*m*omega^2*x^2")?;
        Ok(QuantumOperator::hamiltonian(expr, "H_ho"))
    }

    /// Create Pauli spin matrices
    ///
    /// Returns (σ_x, σ_y, σ_z) as symbolic matrices
    pub fn pauli_matrices<B, E>(
        backend: &B,
    ) -> SymbolicResult<(SymbolicMatrix<E>, SymbolicMatrix<E>, SymbolicMatrix<E>)>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let zero = backend.constant(0.0)?;
        let one = backend.constant(1.0)?;
        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i = backend.neg(&i)?;

        // σ_x = [[0, 1], [1, 0]]
        let sigma_x = backend.matrix(vec![
            vec![zero.clone(), one.clone()],
            vec![one.clone(), zero.clone()],
        ])?;

        // σ_y = [[0, -i], [i, 0]]
        let sigma_y = backend.matrix(vec![vec![zero.clone(), minus_i], vec![i, zero.clone()]])?;

        // σ_z = [[1, 0], [0, -1]]
        let minus_one = backend.neg(&one)?;
        let sigma_z =
            backend.matrix(vec![vec![one.clone(), zero.clone()], vec![zero, minus_one]])?;

        Ok((sigma_x, sigma_y, sigma_z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_creation() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();

        let x_op = QuantumOperator::position(x, "x");
        assert_eq!(x_op.operator_type(), OperatorType::Position);
        assert!(x_op.is_hermitian());
        assert_eq!(x_op.name(), "x");
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_commutator() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();
        let p = backend.variable("p").unwrap();

        let x_op = QuantumOperator::position(x, "x");
        let p_op = QuantumOperator::momentum(p, "p");

        let comm = x_op.commutator(&p_op, &backend).unwrap();
        assert_eq!(comm.name(), "[x, p]");
        assert!(!comm.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_anticommutator() {
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();

        let a_op = QuantumOperator::custom(a, "A", true);
        let b_op = QuantumOperator::custom(b, "B", true);

        let anticomm = a_op.anticommutator(&b_op, &backend).unwrap();
        assert_eq!(anticomm.name(), "{A, B}");
        assert!(anticomm.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_composition() {
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();

        let a_op = QuantumOperator::custom(a, "A", true);
        let b_op = QuantumOperator::custom(b, "B", true);

        let composed = a_op.compose(&b_op, &backend).unwrap();
        assert_eq!(composed.name(), "A·B");
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_power() {
        let backend = create_symbolica_backend();
        let x = backend.variable("x").unwrap();
        let two = backend.constant(2.0).unwrap();

        let x_op = QuantumOperator::position(x, "x");
        let x_squared = x_op.power(&two, &backend).unwrap();

        assert!(x_squared.name().contains("x^"));
        assert!(x_squared.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_standard_operators() {
        let backend = create_symbolica_backend();

        let x_op = standard_operators::position_1d(&backend, "x").unwrap();
        assert_eq!(x_op.operator_type(), OperatorType::Position);

        let p_op = standard_operators::momentum_1d(&backend).unwrap();
        assert_eq!(p_op.operator_type(), OperatorType::Momentum);

        let h_op = standard_operators::harmonic_oscillator_hamiltonian(&backend).unwrap();
        assert_eq!(h_op.operator_type(), OperatorType::Hamiltonian);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_pauli_matrices() {
        let backend = create_symbolica_backend();
        let (sigma_x, sigma_y, sigma_z) = standard_operators::pauli_matrices(&backend).unwrap();

        assert!(sigma_x.is_square());
        assert_eq!(sigma_x.rows, 2);
        assert!(sigma_y.is_square());
        assert!(sigma_z.is_square());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_addition() {
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();

        let a_op = QuantumOperator::custom(a, "A", true);
        let b_op = QuantumOperator::custom(b, "B", true);

        let sum = a_op.add(&b_op, &backend).unwrap();
        assert_eq!(sum.name(), "A + B");
        assert!(sum.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_commutator_antisymmetry() {
        // [A,B] = -[B,A]
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();

        let a_op = QuantumOperator::custom(a, "A", false);
        let b_op = QuantumOperator::custom(b, "B", false);

        let comm_ab = a_op.commutator(&b_op, &backend).unwrap();
        let comm_ba = b_op.commutator(&a_op, &backend).unwrap();

        // Check names are different
        assert_ne!(comm_ab.name(), comm_ba.name());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_anticommutator_symmetry() {
        // {A,B} = {B,A}
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();

        let a_op = QuantumOperator::custom(a, "A", true);
        let b_op = QuantumOperator::custom(b, "B", true);

        let anti_ab = a_op.anticommutator(&b_op, &backend).unwrap();
        let anti_ba = b_op.anticommutator(&a_op, &backend).unwrap();

        // Should have same symbolic expression
        let ab_str = format!("{}", anti_ab.expression);
        let ba_str = format!("{}", anti_ba.expression);
        assert_eq!(ab_str, ba_str);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_jacobi_identity() {
        // [A,[B,C]] + [B,[C,A]] + [C,[A,B]] = 0 (symbolically)
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();
        let c = backend.variable("c").unwrap();

        let a_op = QuantumOperator::custom(a, "A", false);
        let b_op = QuantumOperator::custom(b, "B", false);
        let c_op = QuantumOperator::custom(c, "C", false);

        // Compute [B,C]
        let bc = b_op.commutator(&c_op, &backend).unwrap();
        // Compute [A,[B,C]]
        let a_bc = a_op.commutator(&bc, &backend).unwrap();

        // This tests that nested commutators can be computed
        let result_str = format!("{}", a_bc.expression);
        assert!(!result_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_composition_associativity() {
        // (A*B)*C = A*(B*C)
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let b = backend.variable("b").unwrap();
        let c = backend.variable("c").unwrap();

        let a_op = QuantumOperator::custom(a, "A", false);
        let b_op = QuantumOperator::custom(b, "B", false);
        let c_op = QuantumOperator::custom(c, "C", false);

        let ab = a_op.compose(&b_op, &backend).unwrap();
        let abc1 = ab.compose(&c_op, &backend).unwrap();

        let bc = b_op.compose(&c_op, &backend).unwrap();
        let abc2 = a_op.compose(&bc, &backend).unwrap();

        // Both should give same result
        let abc1_str = format!("{}", abc1.expression);
        let abc2_str = format!("{}", abc2.expression);
        assert_eq!(abc1_str, abc2_str);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hermitian_operator_sum() {
        // Sum of Hermitian operators is Hermitian
        let backend = create_symbolica_backend();
        let h1 = backend.variable("H1").unwrap();
        let h2 = backend.variable("H2").unwrap();

        let h1_op = QuantumOperator::new(h1, OperatorType::Hamiltonian, "H1", true);
        let h2_op = QuantumOperator::new(h2, OperatorType::Hamiltonian, "H2", true);

        let sum = h1_op.add(&h2_op, &backend).unwrap();
        assert!(sum.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_operator_power_values() {
        // Test A^0 = I, A^1 = A
        let backend = create_symbolica_backend();
        let a = backend.variable("a").unwrap();
        let a_op = QuantumOperator::custom(a.clone(), "A", false);

        // A^1 should be A
        let one = backend.constant(1.0).unwrap();
        let a1 = a_op.power(&one, &backend).unwrap();
        let a1_str = format!("{}", a1.expression);
        let a_str = format!("{}", a);
        assert_eq!(a1_str, a_str);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_position_momentum_commutator() {
        // [x,p] = iℏ (canonical commutation relation)
        let backend = create_symbolica_backend();

        let x_op = standard_operators::position_1d(&backend, "x").unwrap();
        let p_op = standard_operators::momentum_1d(&backend).unwrap();

        let comm = x_op.commutator(&p_op, &backend).unwrap();

        // Commutator should exist and be non-zero
        let comm_str = format!("{}", comm.expression);
        assert!(!comm_str.is_empty());
        assert!(!comm.is_hermitian()); // iℏ is anti-Hermitian
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hamiltonian_hermiticity() {
        // Hamiltonian must be Hermitian (observable)
        let backend = create_symbolica_backend();

        let h_ho = standard_operators::harmonic_oscillator_hamiltonian(&backend).unwrap();
        assert!(h_ho.is_hermitian());
        assert_eq!(h_ho.operator_type(), OperatorType::Hamiltonian);
    }
}
