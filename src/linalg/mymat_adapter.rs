//! MyMat Backend Adapter
//!
//! Author: gA4ss
//!
//! Linear algebra backend backed by mymat (v0.3.3).
//! Uses `CpuSingleBackend` for deterministic single-threaded computation.
//! All `LinalgBackend` trait methods delegate to mymat's trait-based API.
//!
//! # Type bridging
//!
//! myquat uses `num_complex::Complex64` (alias for `Complex<f64>`).
//! mymat uses its own `mymat::Complex64` (newtype around `Complex<f64>`).
//! Conversion happens at the adapter boundary — transparent to callers.

use std::fmt::{self, Debug, Formatter};

use num_complex::Complex64;

use mymat::{BasicOps, DataOps, DecompositionOps};

use super::backend::*;

// ─── Type bridging: Complex64 ───────────────────────────────────────────

fn to_mymat(c: Complex64) -> mymat::Complex64 {
    mymat::Complex64::new(c.re, c.im)
}

fn from_mymat(c: mymat::Complex64) -> Complex64 {
    Complex64::new(c.0.re, c.0.im)
}

// ─── Associated types ───────────────────────────────────────────────────

/// Matrix type backed by `mymat::Matrix<mymat::Complex64>`.
#[derive(Clone)]
pub struct MyMatMatrix(pub(crate) mymat::Matrix<mymat::Complex64>);

impl Debug for MyMatMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MyMatMatrix({}x{})", self.0.rows, self.0.cols)
    }
}

/// Vector type backed by `mymat::Vector<mymat::Complex64>`.
#[derive(Clone)]
pub struct MyMatVector(pub(crate) mymat::Vector<mymat::Complex64>);

impl Debug for MyMatVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MyMatVector({})", self.0.len())
    }
}

/// Matrix view type backed by `mymat::MatrixView<'a, mymat::Complex64>`.
pub struct MyMatMatrixView<'a>(pub(crate) mymat::MatrixView<'a, mymat::Complex64>);

impl<'a> Debug for MyMatMatrixView<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MyMatMatrixView({}x{})", self.0.rows, self.0.cols)
    }
}

/// Vector view type (mymat has no VectorView, so we wrap a slice).
pub struct MyMatVectorView<'a>(pub(crate) &'a [mymat::Complex64]);

impl<'a> Debug for MyMatVectorView<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MyMatVectorView({})", self.0.len())
    }
}

/// Mutable vector view type (wraps a mutable slice).
pub struct MyMatVectorViewMut<'a>(pub(crate) &'a mut [mymat::Complex64]);

impl<'a> Debug for MyMatVectorViewMut<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MyMatVectorViewMut({})", self.0.len())
    }
}

// ─── Backend struct ─────────────────────────────────────────────────────

/// Linear algebra backend backed by `mymat::CpuSingleBackend`.
#[derive(Debug, Clone)]
pub struct MyMatBackend {
    inner: mymat::CpuSingleBackend,
}

impl MyMatBackend {
    pub fn new() -> Self {
        Self {
            inner: mymat::CpuSingleBackend::new(),
        }
    }
}

impl Default for MyMatBackend {
    fn default() -> Self {
        Self::new()
    }
}

// ─── LinalgBackend implementation ───────────────────────────────────────

impl LinalgBackend for MyMatBackend {
    type Scalar = Complex64;
    type Matrix = MyMatMatrix;
    type Vector = MyMatVector;
    type MatrixView<'a> = MyMatMatrixView<'a>;
    type VectorView<'a> = MyMatVectorView<'a>;
    type VectorViewMut<'a> = MyMatVectorViewMut<'a>;

    // ── Construction ──────────────────────────────────────────────────

    fn zeros_matrix(&self, rows: usize, cols: usize) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(mymat::Matrix::new(rows, cols)))
    }

    fn zeros_vector(&self, len: usize) -> LinalgResult<Self::Vector> {
        Ok(MyMatVector(mymat::Vector::new(len)))
    }

    fn eye(&self, n: usize) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.identity(n)))
    }

    fn complex_identity(&self, n: usize) -> LinalgResult<Self::Matrix> {
        self.eye(n)
    }

    fn from_shape_vec(
        &self,
        rows: usize,
        cols: usize,
        data: Vec<Self::Scalar>,
    ) -> LinalgResult<Self::Matrix> {
        let mymat_data: Vec<mymat::Complex64> = data.into_iter().map(to_mymat).collect();
        if rows * cols != mymat_data.len() {
            return Err(LinalgError::DimensionError(format!(
                "from_shape_vec: expected {} elements for {}x{} matrix, got {}",
                rows,
                cols,
                rows * cols,
                mymat_data.len()
            )));
        }
        Ok(MyMatMatrix(mymat::Matrix::from_raw_parts(
            rows, cols, mymat_data,
        )))
    }

    fn from_vec(&self, data: Vec<Self::Scalar>) -> LinalgResult<Self::Vector> {
        let mymat_data: Vec<mymat::Complex64> = data.into_iter().map(to_mymat).collect();
        let len = mymat_data.len();
        Ok(MyMatVector(mymat::Vector::from_raw_parts(len, mymat_data)))
    }

    // ── Shape Queries ─────────────────────────────────────────────────

    fn dim(&self, m: &Self::Matrix) -> (usize, usize) {
        (m.0.rows, m.0.cols)
    }

    fn len_vector(&self, v: &Self::Vector) -> usize {
        v.0.len()
    }

    // ── Element Access ────────────────────────────────────────────────

    fn get_matrix(&self, m: &Self::Matrix, i: usize, j: usize) -> LinalgResult<Self::Scalar> {
        if i >= m.0.rows || j >= m.0.cols {
            return Err(LinalgError::IndexOutOfBounds(i, j, m.0.rows, m.0.cols));
        }
        Ok(from_mymat(m.0.get(i, j)))
    }

    fn set_matrix(
        &self,
        m: &mut Self::Matrix,
        i: usize,
        j: usize,
        v: Self::Scalar,
    ) -> LinalgResult<()> {
        if i >= m.0.rows || j >= m.0.cols {
            return Err(LinalgError::IndexOutOfBounds(i, j, m.0.rows, m.0.cols));
        }
        m.0.set(i, j, to_mymat(v));
        Ok(())
    }

    fn get_vector(&self, v: &Self::Vector, i: usize) -> LinalgResult<Self::Scalar> {
        if i >= v.0.len() {
            return Err(LinalgError::IndexOutOfBounds(i, 0, v.0.len(), 1));
        }
        Ok(from_mymat(v.0.get(i)))
    }

    fn set_vector(&self, v: &mut Self::Vector, i: usize, val: Self::Scalar) -> LinalgResult<()> {
        if i >= v.0.len() {
            return Err(LinalgError::IndexOutOfBounds(i, 0, v.0.len(), 1));
        }
        v.0.set(i, to_mymat(val));
        Ok(())
    }

    // ── Linear Algebra ────────────────────────────────────────────────

    fn dot(&self, a: &Self::Matrix, b: &Self::Matrix) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.multiply(&a.0, &b.0)))
    }

    fn dot_vec(&self, a: &Self::Matrix, b: &Self::Vector) -> LinalgResult<Self::Vector> {
        Ok(MyMatVector(self.inner.mat_vec_mul(&a.0, &b.0)))
    }

    fn transpose(&self, a: &Self::Matrix) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.transpose(&a.0)))
    }

    fn conjugate_transpose(&self, a: &Self::Matrix) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.conjugate_transpose(&a.0)))
    }

    fn kronecker(&self, a: &Self::Matrix, b: &Self::Matrix) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.kronecker_product(&a.0, &b.0)))
    }

    // ── Element-wise ──────────────────────────────────────────────────

    fn mapv(
        &self,
        a: &Self::Matrix,
        f: &dyn Fn(Self::Scalar) -> Self::Scalar,
    ) -> LinalgResult<Self::Matrix> {
        let result =
            a.0.mapv(|x: mymat::Complex64| -> mymat::Complex64 { to_mymat(f(from_mymat(x))) });
        Ok(MyMatMatrix(result))
    }

    fn mapv_inplace(
        &self,
        a: &mut Self::Matrix,
        f: &dyn Fn(Self::Scalar) -> Self::Scalar,
    ) -> LinalgResult<()> {
        let mapped =
            a.0.mapv(|x: mymat::Complex64| -> mymat::Complex64 { to_mymat(f(from_mymat(x))) });
        a.0 = mapped;
        Ok(())
    }

    fn fill(&self, a: &mut Self::Matrix, v: Self::Scalar) -> LinalgResult<()> {
        a.0.fill_with(to_mymat(v));
        Ok(())
    }

    fn assign(&self, d: &mut Self::Matrix, s: &Self::Matrix) -> LinalgResult<()> {
        if d.0.rows != s.0.rows || d.0.cols != s.0.cols {
            return Err(LinalgError::DimensionError(format!(
                "Cannot assign {}x{} to {}x{}",
                s.0.rows, s.0.cols, d.0.rows, d.0.cols
            )));
        }
        d.0 = s.0.clone();
        Ok(())
    }

    fn scalar_mul(&self, a: &Self::Matrix, s: Self::Scalar) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(self.inner.scalar_multiply(&a.0, to_mymat(s))))
    }

    // ── Views ─────────────────────────────────────────────────────────

    fn view<'a>(&self, a: &'a Self::Matrix) -> Self::MatrixView<'a> {
        MyMatMatrixView(a.0.view())
    }

    fn view_vector<'a>(&self, v: &'a Self::Vector) -> Self::VectorView<'a> {
        MyMatVectorView(v.0.as_slice())
    }

    fn view_mut_vector<'a>(&self, v: &'a mut Self::Vector) -> Self::VectorViewMut<'a> {
        MyMatVectorViewMut(v.0.as_mut_slice())
    }

    fn to_owned(&self, view: &Self::MatrixView<'_>) -> LinalgResult<Self::Matrix> {
        Ok(MyMatMatrix(view.0.to_owned()))
    }

    // ── Decompositions ────────────────────────────────────────────────

    fn eigh(&self, a: &Self::Matrix) -> LinalgResult<(Vec<f64>, Self::Matrix)> {
        let (evals_mymat, evecs_mymat) = self
            .inner
            .eigen(&a.0)
            .map_err(|e| LinalgError::DecompositionFailed(format!("eigh failed: {}", e)))?;

        let n = evals_mymat.len();
        // Hermitian eigenvalues are real; take the real part.
        let raw_evals: Vec<f64> = (0..n).map(|i| evals_mymat.get(i).0.re).collect();

        // mymat returns eigenvalues in descending order; normalize to
        // ascending to match NdArrayBackend (LAPACK dsyev) so the two
        // backends are behaviorally interchangeable. Eigenvector columns
        // are permuted to preserve eigenpair alignment.
        let mut perm: Vec<usize> = (0..n).collect();
        perm.sort_by(|&i, &j| {
            raw_evals[i]
                .partial_cmp(&raw_evals[j])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let evals: Vec<f64> = perm.iter().map(|&i| raw_evals[i]).collect();

        let (rows, cols) = (evecs_mymat.rows, evecs_mymat.cols);
        let mut sorted_evecs = mymat::Matrix::new(rows, cols);
        for (new_col, &old_col) in perm.iter().enumerate() {
            for row in 0..rows {
                sorted_evecs.set(row, new_col, evecs_mymat.get(row, old_col));
            }
        }

        Ok((evals, MyMatMatrix(sorted_evecs)))
    }

    fn schur_decomposition(
        &self,
        a: &Self::Matrix,
        _eps: f64,
        _max_iter: usize,
    ) -> LinalgResult<SchurResult<Self::Matrix>> {
        // mymat::schur() has its own convergence criteria;
        // eps/max_iter accepted for API compatibility but not forwarded.
        let (q_mymat, t_mymat) = self
            .inner
            .schur(&a.0)
            .map_err(|e| LinalgError::DecompositionFailed(format!("schur failed: {}", e)))?;
        Ok(SchurResult {
            q: MyMatMatrix(q_mymat),
            t: MyMatMatrix(t_mymat),
        })
    }

    fn svd(
        &self,
        a: &Self::Matrix,
        compute_u: bool,
        compute_vt: bool,
    ) -> LinalgResult<SvdResult<Self::Matrix>> {
        let (u_mymat, sigma_mymat, vt_mymat) = self.inner.svd(&a.0);

        // Extract singular values from diagonal of Sigma matrix
        let min_dim = a.0.rows.min(a.0.cols);
        let singular_values: Vec<f64> = (0..min_dim)
            .map(|i| from_mymat(sigma_mymat.get(i, i)).re)
            .collect();

        Ok(SvdResult {
            u: if compute_u {
                Some(MyMatMatrix(u_mymat))
            } else {
                None
            },
            singular_values,
            v_t: if compute_vt {
                Some(MyMatMatrix(vt_mymat))
            } else {
                None
            },
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> MyMatBackend {
        MyMatBackend::new()
    }

    fn c(re: f64, im: f64) -> Complex64 {
        Complex64::new(re, im)
    }

    #[test]
    fn test_mymat_zeros_and_get_set() {
        let b = backend();
        let mut m = b.zeros_matrix(2, 3).unwrap();
        assert_eq!(b.dim(&m), (2, 3));
        b.set_matrix(&mut m, 0, 1, c(3.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&m, 0, 1).unwrap(), c(3.0, 0.0));
    }

    #[test]
    fn test_mymat_eye() {
        let b = backend();
        let eye = b.eye(3).unwrap();
        assert_eq!(b.dim(&eye), (3, 3));
        for i in 0..3 {
            for j in 0..3 {
                let val = b.get_matrix(&eye, i, j).unwrap();
                if i == j {
                    assert!((val - c(1.0, 0.0)).norm() < 1e-10);
                } else {
                    assert!((val - c(0.0, 0.0)).norm() < 1e-10);
                }
            }
        }
    }

    #[test]
    fn test_mymat_vector_ops() {
        let b = backend();
        let v = b.from_vec(vec![c(1.0, 0.0), c(2.0, 0.0)]).unwrap();
        assert_eq!(b.len_vector(&v), 2);
        assert_eq!(b.get_vector(&v, 0).unwrap(), c(1.0, 0.0));
    }

    #[test]
    fn test_mymat_dot_identity() {
        let b = backend();
        let id = b.complex_identity(2).unwrap();
        let data = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let a = b.from_shape_vec(2, 2, data).unwrap();
        let result = b.dot(&id, &a).unwrap();
        assert_eq!(b.get_matrix(&result, 0, 0).unwrap(), c(1.0, 0.0));
        assert_eq!(b.get_matrix(&result, 1, 1).unwrap(), c(4.0, 0.0));
    }

    #[test]
    fn test_mymat_dot_vec() {
        let b = backend();
        let data = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let a = b.from_shape_vec(2, 2, data).unwrap();
        let v = b.from_vec(vec![c(1.0, 0.0), c(1.0, 0.0)]).unwrap();
        let result = b.dot_vec(&a, &v).unwrap();
        assert!((b.get_vector(&result, 0).unwrap() - c(3.0, 0.0)).norm() < 1e-10);
        assert!((b.get_vector(&result, 1).unwrap() - c(7.0, 0.0)).norm() < 1e-10);
    }

    #[test]
    fn test_mymat_kronecker_identity() {
        let b = backend();
        let i2 = b.complex_identity(2).unwrap();
        let i4 = b.kronecker(&i2, &i2).unwrap();
        assert_eq!(b.dim(&i4), (4, 4));
        for i in 0..4 {
            assert!((b.get_matrix(&i4, i, i).unwrap() - c(1.0, 0.0)).norm() < 1e-10);
        }
    }

    #[test]
    fn test_mymat_transpose() {
        let b = backend();
        let data = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let a = b.from_shape_vec(2, 2, data).unwrap();
        let at = b.transpose(&a).unwrap();
        assert_eq!(b.get_matrix(&at, 0, 0).unwrap(), c(1.0, 0.0));
        assert_eq!(b.get_matrix(&at, 0, 1).unwrap(), c(3.0, 0.0));
    }

    #[test]
    fn test_mymat_conjugate_transpose() {
        let b = backend();
        let data = vec![c(1.0, 0.0), c(0.0, 1.0), c(0.0, -1.0), c(1.0, 0.0)];
        let a = b.from_shape_vec(2, 2, data).unwrap();
        let ah = b.conjugate_transpose(&a).unwrap();
        assert_eq!(b.get_matrix(&ah, 0, 1).unwrap(), c(0.0, 1.0));
        assert_eq!(b.get_matrix(&ah, 1, 0).unwrap(), c(0.0, -1.0));
    }

    #[test]
    fn test_mymat_scalar_mul() {
        let b = backend();
        let data = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let a = b.from_shape_vec(2, 2, data).unwrap();
        let result = b.scalar_mul(&a, c(2.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&result, 0, 0).unwrap(), c(2.0, 0.0));
        assert_eq!(b.get_matrix(&result, 1, 1).unwrap(), c(8.0, 0.0));
    }

    #[test]
    fn test_mymat_view_roundtrip() {
        let b = backend();
        let eye = b.complex_identity(3).unwrap();
        let view = b.view(&eye);
        let owned = LinalgBackend::to_owned(&b, &view).unwrap();
        assert_eq!(b.get_matrix(&owned, 0, 0).unwrap(), c(1.0, 0.0));
    }

    #[test]
    fn test_mymat_eigh_hermitian_pauli_z() {
        let b = backend();
        let data = vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)];
        let z = b.from_shape_vec(2, 2, data).unwrap();
        let (evals, _evecs) = b.eigh(&z).unwrap();
        assert_eq!(evals.len(), 2);
        // Sum should be 0 (1 + (-1)), product should be -1
        let sum: f64 = evals.iter().sum();
        assert!(
            (sum - 0.0).abs() < 1e-10,
            "Sum of eigenvalues should be 0, got {}",
            sum
        );
        let prod: f64 = evals.iter().product();
        assert!(
            (prod - (-1.0)).abs() < 1e-10,
            "Product should be -1, got {}",
            prod
        );
    }

    #[test]
    fn test_mymat_eigh_identity() {
        let b = backend();
        let id = b.complex_identity(3).unwrap();
        let (evals, _evecs) = b.eigh(&id).unwrap();
        assert_eq!(evals.len(), 3);
        for &v in &evals {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }
}
