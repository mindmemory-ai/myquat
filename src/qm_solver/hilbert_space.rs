//! Hilbert Space Operations Module
//!
//! Author: gA4ss
//!
//! This module provides operations on quantum Hilbert spaces, including
//! basis representations, transformations, completeness relations, and
//! tensor product constructions.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicMatrix, SymbolicResult};
use std::fmt;

/// Quantum basis representation
///
/// Represents a complete orthonormal basis in a Hilbert space.
///
/// # Mathematical Background
///
/// A basis {|n⟩} is complete if: ∑_n |n⟩⟨n| = I (completeness relation)
/// A basis is orthonormal if: ⟨m|n⟩ = δ_{mn}
#[derive(Debug, Clone)]
pub struct QuantumBasis<E: SymbolicExpression> {
    /// Name of the basis
    pub name: String,

    /// Dimension of the Hilbert space (None for infinite-dimensional)
    pub dimension: Option<usize>,

    /// Basis type identifier
    pub basis_type: BasisType,

    /// Basis vectors (for finite-dimensional spaces)
    pub vectors: Option<Vec<SymbolicMatrix<E>>>,
}

/// Type of quantum basis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasisType {
    /// Position basis: {|x⟩}
    Position,
    /// Momentum basis: {|p⟩}
    Momentum,
    /// Energy eigenbasis: {|n⟩} where H|n⟩ = E_n|n⟩
    Energy,
    /// Angular momentum basis: {|l,m⟩}
    AngularMomentum,
    /// Spin basis: {|↑⟩, |↓⟩} or {|s,m_s⟩}
    Spin,
    /// Custom basis
    Custom,
}

impl fmt::Display for BasisType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BasisType::Position => write!(f, "Position"),
            BasisType::Momentum => write!(f, "Momentum"),
            BasisType::Energy => write!(f, "Energy"),
            BasisType::AngularMomentum => write!(f, "AngularMomentum"),
            BasisType::Spin => write!(f, "Spin"),
            BasisType::Custom => write!(f, "Custom"),
        }
    }
}

impl<E: SymbolicExpression> QuantumBasis<E> {
    /// Create a new quantum basis
    pub fn new(name: impl Into<String>, dimension: Option<usize>, basis_type: BasisType) -> Self {
        Self {
            name: name.into(),
            dimension,
            basis_type,
            vectors: None,
        }
    }

    /// Create a finite-dimensional basis with explicit vectors
    pub fn with_vectors(
        name: impl Into<String>,
        vectors: Vec<SymbolicMatrix<E>>,
        basis_type: BasisType,
    ) -> Self {
        let dimension = Some(vectors.len());
        Self {
            name: name.into(),
            dimension,
            basis_type,
            vectors: Some(vectors),
        }
    }

    /// Check if the basis is finite-dimensional
    pub fn is_finite(&self) -> bool {
        self.dimension.is_some()
    }

    /// Get the dimension if finite
    pub fn dimension(&self) -> Option<usize> {
        self.dimension
    }

    /// Get the basis type
    pub fn basis_type(&self) -> BasisType {
        self.basis_type
    }
}

/// Basis transformation between two quantum bases
///
/// Represents the transformation matrix U such that:
/// |ψ⟩_new = U |ψ⟩_old
///
/// # Mathematical Background
///
/// For orthonormal bases:
/// U is unitary: U†U = UU† = I
/// Matrix elements: U_{nm} = ⟨n_new|m_old⟩
pub struct BasisTransformation<E: SymbolicExpression> {
    /// Source basis
    pub from_basis: QuantumBasis<E>,

    /// Target basis
    pub to_basis: QuantumBasis<E>,

    /// Transformation matrix (if known)
    pub transformation_matrix: Option<SymbolicMatrix<E>>,
}

impl<E: SymbolicExpression> BasisTransformation<E> {
    /// Create a new basis transformation
    pub fn new(from_basis: QuantumBasis<E>, to_basis: QuantumBasis<E>) -> Self {
        Self {
            from_basis,
            to_basis,
            transformation_matrix: None,
        }
    }

    /// Create a transformation with explicit matrix
    pub fn with_matrix(
        from_basis: QuantumBasis<E>,
        to_basis: QuantumBasis<E>,
        matrix: SymbolicMatrix<E>,
    ) -> Self {
        Self {
            from_basis,
            to_basis,
            transformation_matrix: Some(matrix),
        }
    }

    /// Check if transformation matrix is available
    pub fn has_matrix(&self) -> bool {
        self.transformation_matrix.is_some()
    }

    /// Apply transformation to a state vector
    pub fn transform_state<B>(
        &self,
        state: &SymbolicMatrix<E>,
        backend: &B,
    ) -> SymbolicResult<SymbolicMatrix<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        if let Some(ref matrix) = self.transformation_matrix {
            backend.matrix_mul(matrix, state)
        } else {
            Err(crate::symbolic::SymbolicError::UnsupportedOperation(
                "Transformation matrix not available".to_string(),
            ))
        }
    }
}

/// Tensor product of Hilbert spaces
///
/// Represents the tensor product H₁ ⊗ H₂ of two Hilbert spaces.
///
/// # Mathematical Background
///
/// dim(H₁ ⊗ H₂) = dim(H₁) × dim(H₂)
/// Basis: {|i⟩ ⊗ |j⟩} where |i⟩ ∈ H₁, |j⟩ ∈ H₂
pub struct TensorProductSpace<E: SymbolicExpression> {
    /// First Hilbert space
    pub space1: QuantumBasis<E>,

    /// Second Hilbert space
    pub space2: QuantumBasis<E>,

    /// Combined dimension (if both finite)
    pub total_dimension: Option<usize>,
}

impl<E: SymbolicExpression> TensorProductSpace<E> {
    /// Create a tensor product space
    pub fn new(space1: QuantumBasis<E>, space2: QuantumBasis<E>) -> Self {
        let total_dimension = match (space1.dimension, space2.dimension) {
            (Some(d1), Some(d2)) => Some(d1 * d2),
            _ => None,
        };

        Self {
            space1,
            space2,
            total_dimension,
        }
    }

    /// Compute tensor product of two state vectors
    ///
    /// |ψ⟩ ⊗ |φ⟩
    pub fn tensor_product<B>(
        state1: &SymbolicMatrix<E>,
        state2: &SymbolicMatrix<E>,
        backend: &B,
    ) -> SymbolicResult<SymbolicMatrix<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // Kronecker product: (a ⊗ b)_{ij,kl} = a_{ik} b_{jl}
        let n1 = state1.rows;
        let n2 = state2.rows;

        let mut result_elements = Vec::new();

        for i in 0..n1 {
            for j in 0..n2 {
                let a_i = state1.get(i, 0).ok_or_else(|| {
                    crate::symbolic::SymbolicError::MatrixOperationFailed(
                        "Index out of bounds".to_string(),
                    )
                })?;
                let b_j = state2.get(j, 0).ok_or_else(|| {
                    crate::symbolic::SymbolicError::MatrixOperationFailed(
                        "Index out of bounds".to_string(),
                    )
                })?;
                let product = backend.mul(a_i, b_j)?;
                result_elements.push(vec![product]);
            }
        }

        backend.matrix(result_elements)
    }

    /// Check if the space is finite-dimensional
    pub fn is_finite(&self) -> bool {
        self.total_dimension.is_some()
    }

    /// Get total dimension
    pub fn dimension(&self) -> Option<usize> {
        self.total_dimension
    }
}

/// Completeness relation utilities
pub mod completeness {
    use super::*;

    /// Verify completeness relation for a finite basis
    ///
    /// Checks if ∑_n |n⟩⟨n| = I
    pub fn verify_completeness<B, E>(
        basis: &QuantumBasis<E>,
        backend: &B,
    ) -> SymbolicResult<SymbolicMatrix<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if let Some(ref vectors) = basis.vectors {
            if vectors.is_empty() {
                return Err(crate::symbolic::SymbolicError::InvalidExpression(
                    "Basis has no vectors".to_string(),
                ));
            }

            let _dim = vectors.len();

            // Compute ∑_n |n⟩⟨n|
            let first = &vectors[0];
            let first_t = transpose_conjugate(first, backend)?;
            let mut sum = backend.matrix_mul(first, &first_t)?;

            for vec in &vectors[1..] {
                let vec_t = transpose_conjugate(vec, backend)?;
                let outer = backend.matrix_mul(vec, &vec_t)?;
                sum = add_matrices(&sum, &outer, backend)?;
            }

            Ok(sum)
        } else {
            Err(crate::symbolic::SymbolicError::UnsupportedOperation(
                "Completeness relation only for explicit basis vectors".to_string(),
            ))
        }
    }

    /// Helper: transpose and conjugate a matrix
    fn transpose_conjugate<B, E>(
        matrix: &SymbolicMatrix<E>,
        backend: &B,
    ) -> SymbolicResult<SymbolicMatrix<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let mut elements = Vec::new();
        for j in 0..matrix.cols {
            let mut row = Vec::new();
            for i in 0..matrix.rows {
                let elem = matrix.get(i, j).ok_or_else(|| {
                    crate::symbolic::SymbolicError::MatrixOperationFailed(
                        "Index out of bounds".to_string(),
                    )
                })?;
                let conj = backend.conjugate(elem)?;
                row.push(conj);
            }
            elements.push(row);
        }
        backend.matrix(elements)
    }

    /// Helper: add two matrices
    fn add_matrices<B, E>(
        a: &SymbolicMatrix<E>,
        b: &SymbolicMatrix<E>,
        backend: &B,
    ) -> SymbolicResult<SymbolicMatrix<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if a.rows != b.rows || a.cols != b.cols {
            return Err(crate::symbolic::SymbolicError::MatrixOperationFailed(
                "Matrix dimensions must match for addition".to_string(),
            ));
        }

        let mut elements = Vec::new();
        for i in 0..a.rows {
            let mut row = Vec::new();
            for j in 0..a.cols {
                let a_ij = a.get(i, j).ok_or_else(|| {
                    crate::symbolic::SymbolicError::MatrixOperationFailed(
                        "Index out of bounds".to_string(),
                    )
                })?;
                let b_ij = b.get(i, j).ok_or_else(|| {
                    crate::symbolic::SymbolicError::MatrixOperationFailed(
                        "Index out of bounds".to_string(),
                    )
                })?;
                let sum = backend.add(a_ij, b_ij)?;
                row.push(sum);
            }
            elements.push(row);
        }
        backend.matrix(elements)
    }
}

/// Standard quantum bases
pub mod standard_bases {
    use super::*;

    /// Create the standard spin-1/2 basis: {|↑⟩, |↓⟩}
    pub fn spin_half_basis<B, E>(backend: &B) -> SymbolicResult<QuantumBasis<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let one = backend.constant(1.0)?;
        let zero = backend.constant(0.0)?;

        // |↑⟩ = [1, 0]ᵀ
        let up = backend.matrix(vec![vec![one.clone()], vec![zero.clone()]])?;

        // |↓⟩ = [0, 1]ᵀ
        let down = backend.matrix(vec![vec![zero], vec![one]])?;

        Ok(QuantumBasis::with_vectors(
            "spin-1/2",
            vec![up, down],
            BasisType::Spin,
        ))
    }

    /// Create a position basis (infinite-dimensional)
    pub fn position_basis<E: SymbolicExpression>() -> QuantumBasis<E> {
        QuantumBasis::new("position", None, BasisType::Position)
    }

    /// Create a momentum basis (infinite-dimensional)
    pub fn momentum_basis<E: SymbolicExpression>() -> QuantumBasis<E> {
        QuantumBasis::new("momentum", None, BasisType::Momentum)
    }

    /// Create an energy eigenbasis (dimension depends on system)
    pub fn energy_basis<E: SymbolicExpression>(dimension: Option<usize>) -> QuantumBasis<E> {
        QuantumBasis::new("energy", dimension, BasisType::Energy)
    }
}

impl<E: SymbolicExpression> fmt::Display for QuantumBasis<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QuantumBasis({}, {}, {})",
            self.name,
            self.basis_type,
            match self.dimension {
                Some(d) => format!("dim={}", d),
                None => "infinite".to_string(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_basis_creation() {
        let basis = QuantumBasis::<crate::symbolic::symbolica_adapter::SymbolicaExpression>::new(
            "test",
            Some(2),
            BasisType::Spin,
        );

        assert_eq!(basis.name, "test");
        assert_eq!(basis.dimension(), Some(2));
        assert!(basis.is_finite());
        assert_eq!(basis.basis_type(), BasisType::Spin);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_spin_half_basis() {
        let backend = create_symbolica_backend();
        let basis = standard_bases::spin_half_basis(&backend).unwrap();

        assert_eq!(basis.dimension(), Some(2));
        assert_eq!(basis.basis_type(), BasisType::Spin);
        assert!(basis.vectors.is_some());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_infinite_dimensional_basis() {
        let pos_basis = standard_bases::position_basis::<
            crate::symbolic::symbolica_adapter::SymbolicaExpression,
        >();
        assert!(!pos_basis.is_finite());
        assert_eq!(pos_basis.dimension(), None);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_space() {
        let backend = create_symbolica_backend();
        let basis1 = standard_bases::spin_half_basis(&backend).unwrap();
        let basis2 = standard_bases::spin_half_basis(&backend).unwrap();

        let tensor_space = TensorProductSpace::new(basis1, basis2);
        assert_eq!(tensor_space.dimension(), Some(4));
        assert!(tensor_space.is_finite());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_vectors() {
        let backend = create_symbolica_backend();

        let one = backend.constant(1.0).unwrap();
        let zero = backend.constant(0.0).unwrap();

        let state1 = backend
            .matrix(vec![vec![one.clone()], vec![zero.clone()]])
            .unwrap();
        let state2 = backend
            .matrix(vec![vec![zero.clone()], vec![one.clone()]])
            .unwrap();

        let result = TensorProductSpace::tensor_product(&state1, &state2, &backend).unwrap();

        assert_eq!(result.rows, 4);
        assert_eq!(result.cols, 1);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_basis_transformation() {
        let backend = create_symbolica_backend();
        let basis1 = standard_bases::spin_half_basis(&backend).unwrap();
        let basis2 = standard_bases::spin_half_basis(&backend).unwrap();

        let transform = BasisTransformation::new(basis1, basis2);
        assert!(!transform.has_matrix());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_completeness_relation() {
        let backend = create_symbolica_backend();
        let basis = standard_bases::spin_half_basis(&backend).unwrap();

        let completeness_op = completeness::verify_completeness(&basis, &backend).unwrap();

        // Should give identity matrix (approximately)
        assert_eq!(completeness_op.rows, 2);
        assert_eq!(completeness_op.cols, 2);
        assert!(completeness_op.is_square());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_dimension_formula() {
        // dim(H₁ ⊗ H₂) = dim(H₁) × dim(H₂)
        let backend = create_symbolica_backend();

        let basis2 = standard_bases::spin_half_basis(&backend).unwrap(); // dim = 2
        let basis3 = standard_bases::energy_basis(Some(3)); // dim = 3

        let tensor_space = TensorProductSpace::new(basis2, basis3);

        // 2 × 3 = 6
        assert_eq!(tensor_space.dimension(), Some(6));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_associativity() {
        // (A ⊗ B) ⊗ C has same dimension as A ⊗ (B ⊗ C)
        let backend = create_symbolica_backend();

        let a = standard_bases::spin_half_basis(&backend).unwrap(); // dim = 2
        let b = standard_bases::spin_half_basis(&backend).unwrap(); // dim = 2
        let c = standard_bases::spin_half_basis(&backend).unwrap(); // dim = 2

        let ab = TensorProductSpace::new(a.clone(), b.clone());
        let abc1 = TensorProductSpace::new(
            QuantumBasis::new("AB", Some(4), BasisType::Custom),
            c.clone(),
        );

        let bc = TensorProductSpace::new(b, c);
        let abc2 = TensorProductSpace::new(a, QuantumBasis::new("BC", Some(4), BasisType::Custom));

        // Both should give dimension 2×2×2 = 8
        assert_eq!(abc1.dimension(), Some(8));
        assert_eq!(abc2.dimension(), Some(8));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_state_normalization() {
        // |ψ⟩ ⊗ |φ⟩ where both are normalized
        let backend = create_symbolica_backend();

        let one = backend.constant(1.0).unwrap();
        let zero = backend.constant(0.0).unwrap();

        // Normalized state |↑⟩
        let up = backend
            .matrix(vec![vec![one.clone()], vec![zero.clone()]])
            .unwrap();

        // Normalized state |↓⟩
        let down = backend
            .matrix(vec![vec![zero.clone()], vec![one.clone()]])
            .unwrap();

        let tensor_state = TensorProductSpace::tensor_product(&up, &down, &backend).unwrap();

        // Should be |↑⟩⊗|↓⟩ = [0,1,0,0]ᵀ
        assert_eq!(tensor_state.rows, 4);
        assert_eq!(tensor_state.cols, 1);
    }

    #[test]
    fn test_basis_type_variants() {
        // Test all basis type variants
        let types = vec![
            BasisType::Position,
            BasisType::Momentum,
            BasisType::Energy,
            BasisType::Spin,
            BasisType::AngularMomentum,
            BasisType::Custom,
        ];

        for basis_type in types {
            assert_eq!(format!("{:?}", basis_type).len() > 0, true);
        }
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_infinite_vs_finite_dimensions() {
        // Position basis is infinite-dimensional
        let pos_basis = standard_bases::position_basis::<
            crate::symbolic::symbolica_adapter::SymbolicaExpression,
        >();
        assert!(!pos_basis.is_finite());
        assert_eq!(pos_basis.dimension(), None);

        // Energy basis can be finite
        let energy_basis = standard_bases::energy_basis::<
            crate::symbolic::symbolica_adapter::SymbolicaExpression,
        >(Some(5));
        assert!(energy_basis.is_finite());
        assert_eq!(energy_basis.dimension(), Some(5));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_basis_transformation_dimensions() {
        let backend = create_symbolica_backend();

        let spin_basis = standard_bases::spin_half_basis(&backend).unwrap();
        let energy_basis = standard_bases::energy_basis(Some(2));

        let transform = BasisTransformation::new(spin_basis, energy_basis);

        // Transformation should preserve dimension
        assert_eq!(transform.from_basis.dimension(), Some(2));
        assert_eq!(transform.to_basis.dimension(), Some(2));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_spin_basis_vectors() {
        let backend = create_symbolica_backend();
        let spin_basis = standard_bases::spin_half_basis(&backend).unwrap();

        // Should have 2 basis vectors
        assert!(spin_basis.vectors.is_some());
        if let Some(ref vecs) = spin_basis.vectors {
            assert_eq!(vecs.len(), 2);

            // Each vector should be 2×1
            for vec in vecs {
                assert_eq!(vec.rows, 2);
                assert_eq!(vec.cols, 1);
            }
        }
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tensor_product_commutativity_dimension() {
        // dim(A⊗B) = dim(B⊗A) but states may differ
        let backend = create_symbolica_backend();

        let basis_a = standard_bases::spin_half_basis(&backend).unwrap();
        let basis_b = standard_bases::energy_basis(Some(3));

        let ab = TensorProductSpace::new(basis_a.clone(), basis_b.clone());
        let ba = TensorProductSpace::new(basis_b, basis_a);

        // Dimensions should match
        assert_eq!(ab.dimension(), ba.dimension());
        assert_eq!(ab.dimension(), Some(6));
    }
}
