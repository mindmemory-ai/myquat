//! Linear Algebra Backend Trait
//!
//! Author: gA4ss
//!
//! This module provides an abstract interface for linear algebra operations
//! required for quantum computing. The actual linear algebra implementation is
//! delegated to backend implementations such as `NdArrayBackend` (ndarray+nalgebra)
//! or `MyMatBackend` (mymat-backed, default).
//!
//! # Design Philosophy
//!
//! This interface follows the Strategy pattern, mirroring `SymbolicBackend`.
//! Different linear algebra backends can be plugged in without changing the
//! quantum computing code. The interface is designed to be:
//!
//! - **Abstract**: No concrete linear algebra implementation
//! - **Flexible**: Support various linear algebra backends
//! - **Type-safe**: Leverage Rust's type system and GATs for views
//! - **Extensible**: Easy to add new linear algebra operations

use std::error::Error;
use std::fmt::{Debug, Display};

/// Scalar type bound for linear algebra operations.
///
/// Implemented for `f64` and `num_complex::Complex64`.
/// This trait centralizes all scalar-level operations (zero, one, conj, norm)
/// so that `LinalgBackend` methods don't need redundant trait bounds.
pub trait LinalgScalar: Clone + Debug + Display + 'static {
    fn zero() -> Self;
    fn one() -> Self;
    /// Create a scalar from a real f64 value.
    /// For Complex64, this creates a real-only complex number (imag=0).
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;
    fn conj(&self) -> Self;
    fn norm(&self) -> f64;
    fn norm_sqr(&self) -> f64;
}

impl LinalgScalar for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn to_f64(self) -> f64 {
        self
    }
    fn conj(&self) -> Self {
        *self
    }
    fn norm(&self) -> f64 {
        f64::abs(*self)
    }
    fn norm_sqr(&self) -> f64 {
        self * self
    }
}

impl LinalgScalar for num_complex::Complex64 {
    fn zero() -> Self {
        num_complex::Complex64::new(0.0, 0.0)
    }
    fn one() -> Self {
        num_complex::Complex64::new(1.0, 0.0)
    }
    fn from_f64(v: f64) -> Self {
        num_complex::Complex64::new(v, 0.0)
    }
    fn to_f64(self) -> f64 {
        num_complex::Complex::<f64>::norm(self)
    }
    fn conj(&self) -> Self {
        num_complex::Complex::<f64>::conj(self)
    }
    fn norm(&self) -> f64 {
        num_complex::Complex::<f64>::norm(*self)
    }
    fn norm_sqr(&self) -> f64 {
        num_complex::Complex::<f64>::norm_sqr(self)
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during linear algebra operations.
#[derive(Debug, Clone, PartialEq)]
pub enum LinalgError {
    /// Matrix/vector dimensions don't match for the requested operation
    ShapeMismatch(String),
    /// Generic dimension error
    DimensionError(String),
    /// Index out of bounds: (i, j, rows, cols)
    IndexOutOfBounds(usize, usize, usize, usize),
    /// Decomposition failed (numerical issue or non-convergence)
    DecompositionFailed(String),
    /// Matrix must be square for this operation
    NotSquare(usize, usize),
    /// Backend-specific error
    BackendError(String),
    /// Operation not yet implemented for this backend
    UnsupportedOperation(String),
}

impl Display for LinalgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinalgError::ShapeMismatch(msg) => write!(f, "Shape mismatch: {}", msg),
            LinalgError::DimensionError(msg) => write!(f, "Dimension error: {}", msg),
            LinalgError::IndexOutOfBounds(i, j, rows, cols) => {
                write!(f, "Index ({},{}) out of bounds ({}x{})", i, j, rows, cols)
            }
            LinalgError::DecompositionFailed(msg) => write!(f, "Decomposition failed: {}", msg),
            LinalgError::NotSquare(r, c) => write!(f, "Matrix not square: {}x{}", r, c),
            LinalgError::BackendError(msg) => write!(f, "Backend error: {}", msg),
            LinalgError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {}", msg)
            }
        }
    }
}

impl Error for LinalgError {}

/// Result type alias for linear algebra operations.
pub type LinalgResult<T> = Result<T, LinalgError>;

// ============================================================================
// Decomposition Result Types
// ============================================================================

/// Result of a Schur decomposition: A = Q * T * Q^H
///
/// T is upper quasi-triangular (real Schur form: 1x1 blocks for real eigenvalues,
/// 2x2 blocks for complex conjugate pairs).
pub struct SchurResult<M> {
    /// Orthogonal/unitary matrix Q
    pub q: M,
    /// Upper quasi-triangular matrix T
    pub t: M,
}

/// Result of a Singular Value Decomposition: A = U * Σ * V^H
pub struct SvdResult<M> {
    /// Left singular vectors (None if compute_u was false)
    pub u: Option<M>,
    /// Right singular vectors conjugate transpose (None if compute_vt was false)
    pub v_t: Option<M>,
    /// Singular values in descending order
    pub singular_values: Vec<f64>,
}

// ============================================================================
// LinalgBackend Trait
// ============================================================================

/// Abstract linear algebra backend.
///
/// This trait defines the interface for linear algebra operations needed
/// for quantum computing: matrix/vector construction, arithmetic,
/// decompositions, and element-wise operations.
///
/// # Type Parameters
///
/// Uses Generic Associated Types (GATs) for zero-copy views:
/// - `MatrixView<'a>` — immutable view of a matrix
/// - `VectorView<'a>` — immutable view of a vector
/// - `VectorViewMut<'a>` — mutable view of a vector
///
/// # Implementations
///
/// - `NdArrayBackend` — production backend wrapping ndarray + nalgebra
/// - `MyMatBackend` — mymat-backed backend (default)
pub trait LinalgBackend {
    /// The scalar type for matrix/vector elements
    type Scalar: LinalgScalar;

    /// Owned matrix type (e.g., `Array2<Complex64>`)
    type Matrix: Clone + Debug;

    /// Owned vector type (e.g., `Array1<Complex64>`)
    type Vector: Clone + Debug;

    /// Zero-copy immutable matrix view
    type MatrixView<'a>: Debug;

    /// Zero-copy immutable vector view
    type VectorView<'a>: Debug;

    /// Zero-copy mutable vector view
    type VectorViewMut<'a>: Debug;

    // ────────────────────────────────────────────────────────────────────
    // Construction
    // ────────────────────────────────────────────────────────────────────

    /// Create a zero-initialized matrix with given dimensions
    fn zeros_matrix(&self, rows: usize, cols: usize) -> LinalgResult<Self::Matrix>;

    /// Create a zero-initialized vector with given length
    fn zeros_vector(&self, len: usize) -> LinalgResult<Self::Vector>;

    /// Create an identity matrix of size n × n (real-valued)
    fn eye(&self, n: usize) -> LinalgResult<Self::Matrix>;

    /// Create a complex-valued identity matrix of size n × n
    fn complex_identity(&self, n: usize) -> LinalgResult<Self::Matrix>;

    /// Create a matrix from a flat row-major data vector
    fn from_shape_vec(
        &self,
        rows: usize,
        cols: usize,
        data: Vec<Self::Scalar>,
    ) -> LinalgResult<Self::Matrix>;

    /// Create a vector from data
    fn from_vec(&self, data: Vec<Self::Scalar>) -> LinalgResult<Self::Vector>;

    // ────────────────────────────────────────────────────────────────────
    // Shape Queries
    // ────────────────────────────────────────────────────────────────────

    /// Return (rows, cols) of a matrix
    fn dim(&self, matrix: &Self::Matrix) -> (usize, usize);

    /// Number of rows
    fn nrows(&self, matrix: &Self::Matrix) -> usize {
        self.dim(matrix).0
    }

    /// Number of columns
    fn ncols(&self, matrix: &Self::Matrix) -> usize {
        self.dim(matrix).1
    }

    /// Length of a vector
    fn len_vector(&self, vector: &Self::Vector) -> usize;

    // ────────────────────────────────────────────────────────────────────
    // Element Access
    // ────────────────────────────────────────────────────────────────────

    /// Get element (i, j) from a matrix
    fn get_matrix(&self, matrix: &Self::Matrix, i: usize, j: usize) -> LinalgResult<Self::Scalar>;

    /// Set element (i, j) in a matrix
    fn set_matrix(
        &self,
        matrix: &mut Self::Matrix,
        i: usize,
        j: usize,
        value: Self::Scalar,
    ) -> LinalgResult<()>;

    /// Get element i from a vector
    fn get_vector(&self, vector: &Self::Vector, i: usize) -> LinalgResult<Self::Scalar>;

    /// Set element i in a vector
    fn set_vector(
        &self,
        vector: &mut Self::Vector,
        i: usize,
        value: Self::Scalar,
    ) -> LinalgResult<()>;

    // ────────────────────────────────────────────────────────────────────
    // Linear Algebra Operations
    // ────────────────────────────────────────────────────────────────────

    /// Matrix multiplication: A * B
    fn dot(&self, a: &Self::Matrix, b: &Self::Matrix) -> LinalgResult<Self::Matrix>;

    /// Matrix-vector multiplication: A * v
    fn dot_vec(&self, a: &Self::Matrix, b: &Self::Vector) -> LinalgResult<Self::Vector>;

    /// Transpose (without conjugation): A^T
    fn transpose(&self, a: &Self::Matrix) -> LinalgResult<Self::Matrix>;

    /// Conjugate transpose: A^H = (A^*)^T
    fn conjugate_transpose(&self, a: &Self::Matrix) -> LinalgResult<Self::Matrix>;

    /// Kronecker (tensor) product: A ⊗ B
    fn kronecker(&self, a: &Self::Matrix, b: &Self::Matrix) -> LinalgResult<Self::Matrix>;

    // ────────────────────────────────────────────────────────────────────
    // Element-wise Operations
    // ────────────────────────────────────────────────────────────────────

    /// Element-wise map producing a new matrix
    fn mapv(
        &self,
        a: &Self::Matrix,
        f: &dyn Fn(Self::Scalar) -> Self::Scalar,
    ) -> LinalgResult<Self::Matrix>;

    /// In-place element-wise map
    fn mapv_inplace(
        &self,
        a: &mut Self::Matrix,
        f: &dyn Fn(Self::Scalar) -> Self::Scalar,
    ) -> LinalgResult<()>;

    /// Fill matrix with a constant value
    fn fill(&self, a: &mut Self::Matrix, value: Self::Scalar) -> LinalgResult<()>;

    /// Copy src into dest (dest = src)
    fn assign(&self, dest: &mut Self::Matrix, src: &Self::Matrix) -> LinalgResult<()>;

    /// Scalar multiplication: s * A
    fn scalar_mul(&self, a: &Self::Matrix, scalar: Self::Scalar) -> LinalgResult<Self::Matrix>;

    // ────────────────────────────────────────────────────────────────────
    // Views (zero-copy)
    // ────────────────────────────────────────────────────────────────────

    /// Create an immutable view of a matrix
    fn view<'a>(&self, a: &'a Self::Matrix) -> Self::MatrixView<'a>;

    /// Create an immutable view of a vector
    fn view_vector<'a>(&self, v: &'a Self::Vector) -> Self::VectorView<'a>;

    /// Create a mutable view of a vector
    fn view_mut_vector<'a>(&self, v: &'a mut Self::Vector) -> Self::VectorViewMut<'a>;

    /// Convert a matrix view back to an owned matrix
    fn to_owned(&self, view: &Self::MatrixView<'_>) -> LinalgResult<Self::Matrix>;

    // ────────────────────────────────────────────────────────────────────
    // Decompositions
    // ────────────────────────────────────────────────────────────────────

    /// Hermitian eigenvalue decomposition: A = V * diag(λ) * V^H
    ///
    /// Returns (eigenvalues, eigenvectors) where eigenvectors are columns of V.
    /// Eigenvalues are always real for Hermitian matrices and are returned in
    /// **ascending** order (LAPACK convention); eigenvector columns are aligned
    /// to the eigenvalue order. All backends MUST honor this ordering so they
    /// remain behaviorally interchangeable.
    fn eigh(&self, a: &Self::Matrix) -> LinalgResult<(Vec<f64>, Self::Matrix)>;

    /// Schur decomposition: A = Q * T * Q^H
    ///
    /// T is upper quasi-triangular. `eps` is the convergence tolerance,
    /// `max_iter` is the maximum number of iterations.
    fn schur_decomposition(
        &self,
        a: &Self::Matrix,
        eps: f64,
        max_iter: usize,
    ) -> LinalgResult<SchurResult<Self::Matrix>>;

    /// Singular Value Decomposition: A = U * Σ * V^H
    ///
    /// `compute_u` and `compute_vt` control whether U and V^H are computed.
    fn svd(
        &self,
        a: &Self::Matrix,
        compute_u: bool,
        compute_vt: bool,
    ) -> LinalgResult<SvdResult<Self::Matrix>>;
}
