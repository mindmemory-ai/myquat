//! NdArray Backend Adapter
//!
//! Author: gA4ss
//!
//! This module provides a concrete implementation of the `LinalgBackend` trait
//! using the ndarray + ndarray-linalg + nalgebra combination.

use ndarray::{s, Array1, Array2, ArrayView1, ArrayView2, ArrayViewMut1};
use ndarray_linalg::Eigh;
use num_complex::Complex64;

use super::backend::*;

/// Linear algebra backend wrapping ndarray + ndarray-linalg + nalgebra.
///
/// This is the production backend. All operations delegate directly to
/// the underlying libraries without algorithmic changes, ensuring zero
/// numeric regression from direct ndarray/nalgebra usage.
#[derive(Debug, Clone, Default)]
pub struct NdArrayBackend;

impl NdArrayBackend {
    /// Create a new NdArrayBackend
    pub fn new() -> Self {
        Self
    }
}

impl LinalgBackend for NdArrayBackend {
    type Scalar = Complex64;
    type Matrix = Array2<Complex64>;
    type Vector = Array1<Complex64>;
    type MatrixView<'a> = ArrayView2<'a, Complex64>;
    type VectorView<'a> = ArrayView1<'a, Complex64>;
    type VectorViewMut<'a> = ArrayViewMut1<'a, Complex64>;

    // ─── Construction ────────────────────────────────────────────────────

    fn zeros_matrix(&self, rows: usize, cols: usize) -> LinalgResult<Array2<Complex64>> {
        Ok(Array2::zeros((rows, cols)))
    }

    fn zeros_vector(&self, len: usize) -> LinalgResult<Array1<Complex64>> {
        Ok(Array1::zeros(len))
    }

    fn eye(&self, n: usize) -> LinalgResult<Array2<Complex64>> {
        Ok(Array2::eye(n))
    }

    fn complex_identity(&self, n: usize) -> LinalgResult<Array2<Complex64>> {
        Ok(Array2::eye(n).mapv(|x| Complex64::new(x, 0.0)))
    }

    fn from_shape_vec(
        &self,
        rows: usize,
        cols: usize,
        data: Vec<Complex64>,
    ) -> LinalgResult<Array2<Complex64>> {
        Array2::from_shape_vec((rows, cols), data)
            .map_err(|e| LinalgError::ShapeMismatch(e.to_string()))
    }

    fn from_vec(&self, data: Vec<Complex64>) -> LinalgResult<Array1<Complex64>> {
        Ok(Array1::from_vec(data))
    }

    // ─── Shape Queries ───────────────────────────────────────────────────

    fn dim(&self, matrix: &Array2<Complex64>) -> (usize, usize) {
        matrix.dim()
    }

    fn len_vector(&self, vector: &Array1<Complex64>) -> usize {
        vector.len()
    }

    // ─── Element Access ──────────────────────────────────────────────────

    fn get_matrix(
        &self,
        matrix: &Array2<Complex64>,
        i: usize,
        j: usize,
    ) -> LinalgResult<Complex64> {
        matrix
            .get((i, j))
            .copied()
            .ok_or_else(|| LinalgError::IndexOutOfBounds(i, j, matrix.nrows(), matrix.ncols()))
    }

    fn set_matrix(
        &self,
        matrix: &mut Array2<Complex64>,
        i: usize,
        j: usize,
        value: Complex64,
    ) -> LinalgResult<()> {
        let rows = matrix.nrows();
        let cols = matrix.ncols();
        *matrix
            .get_mut((i, j))
            .ok_or_else(|| LinalgError::IndexOutOfBounds(i, j, rows, cols))? = value;
        Ok(())
    }

    fn get_vector(&self, vector: &Array1<Complex64>, i: usize) -> LinalgResult<Complex64> {
        vector
            .get(i)
            .copied()
            .ok_or_else(|| LinalgError::IndexOutOfBounds(i, 0, vector.len(), 1))
    }

    fn set_vector(
        &self,
        vector: &mut Array1<Complex64>,
        i: usize,
        value: Complex64,
    ) -> LinalgResult<()> {
        let len = vector.len();
        *vector
            .get_mut(i)
            .ok_or_else(|| LinalgError::IndexOutOfBounds(i, 0, len, 1))? = value;
        Ok(())
    }

    // ─── Linear Algebra ──────────────────────────────────────────────────

    fn dot(&self, a: &Array2<Complex64>, b: &Array2<Complex64>) -> LinalgResult<Array2<Complex64>> {
        Ok(a.dot(b))
    }

    fn dot_vec(
        &self,
        a: &Array2<Complex64>,
        b: &Array1<Complex64>,
    ) -> LinalgResult<Array1<Complex64>> {
        Ok(a.dot(b))
    }

    fn transpose(&self, a: &Array2<Complex64>) -> LinalgResult<Array2<Complex64>> {
        Ok(a.t().to_owned())
    }

    fn conjugate_transpose(&self, a: &Array2<Complex64>) -> LinalgResult<Array2<Complex64>> {
        Ok(a.mapv(|z| z.conj()).t().to_owned())
    }

    fn kronecker(
        &self,
        a: &Array2<Complex64>,
        b: &Array2<Complex64>,
    ) -> LinalgResult<Array2<Complex64>> {
        let (m, n) = a.dim();
        let (p, q) = b.dim();
        let mut result = Array2::zeros((m * p, n * q));
        for i in 0..m {
            for j in 0..n {
                let a_ij = a[[i, j]];
                let mut block = b.mapv(|x| x * a_ij);
                result
                    .slice_mut(s![i * p..(i + 1) * p, j * q..(j + 1) * q])
                    .assign(&block);
            }
        }
        Ok(result)
    }

    // ─── Element-wise Operations ─────────────────────────────────────────

    fn mapv(
        &self,
        a: &Array2<Complex64>,
        f: &dyn Fn(Complex64) -> Complex64,
    ) -> LinalgResult<Array2<Complex64>> {
        Ok(a.mapv(f))
    }

    fn mapv_inplace(
        &self,
        a: &mut Array2<Complex64>,
        f: &dyn Fn(Complex64) -> Complex64,
    ) -> LinalgResult<()> {
        a.mapv_inplace(f);
        Ok(())
    }

    fn fill(&self, a: &mut Array2<Complex64>, value: Complex64) -> LinalgResult<()> {
        a.fill(value);
        Ok(())
    }

    fn assign(&self, dest: &mut Array2<Complex64>, src: &Array2<Complex64>) -> LinalgResult<()> {
        dest.assign(src);
        Ok(())
    }

    fn scalar_mul(
        &self,
        a: &Array2<Complex64>,
        scalar: Complex64,
    ) -> LinalgResult<Array2<Complex64>> {
        Ok(a.mapv(|x| x * scalar))
    }

    // ─── Views ───────────────────────────────────────────────────────────

    fn view<'a>(&self, a: &'a Array2<Complex64>) -> ArrayView2<'a, Complex64> {
        a.view()
    }

    fn view_vector<'a>(&self, v: &'a Array1<Complex64>) -> ArrayView1<'a, Complex64> {
        v.view()
    }

    fn view_mut_vector<'a>(&self, v: &'a mut Array1<Complex64>) -> ArrayViewMut1<'a, Complex64> {
        v.view_mut()
    }

    fn to_owned(&self, view: &ArrayView2<'_, Complex64>) -> LinalgResult<Array2<Complex64>> {
        Ok(view.to_owned())
    }

    // ─── Decompositions ──────────────────────────────────────────────────

    fn eigh(&self, a: &Array2<Complex64>) -> LinalgResult<(Vec<f64>, Array2<Complex64>)> {
        let (evals, evecs) = a
            .eigh(ndarray_linalg::UPLO::Upper)
            .map_err(|e| LinalgError::DecompositionFailed(e.to_string()))?;
        Ok((evals.to_vec(), evecs))
    }

    fn schur_decomposition(
        &self,
        a: &Array2<Complex64>,
        eps: f64,
        max_iter: usize,
    ) -> LinalgResult<SchurResult<Array2<Complex64>>> {
        let n = a.nrows();
        let m = a.ncols();

        // Currently only 4x4 matrices are supported (nalgebra Matrix4 bridge).
        // This covers KAK decomposition (two-qubit unitaries).
        // For other sizes, use a general-purpose Schur implementation.
        if n != 4 || m != 4 {
            return Err(LinalgError::DimensionError(format!(
                "Schur decomposition currently only supports 4x4 matrices, got {}x{}",
                n, m
            )));
        }

        // Convert ndarray → nalgebra Matrix4
        let mut nalgebra_m = nalgebra::Matrix4::<Complex64>::zeros();
        for i in 0..n {
            for j in 0..m {
                nalgebra_m[(i, j)] = a[[i, j]];
            }
        }

        let schur = nalgebra::Schur::try_new(nalgebra_m, eps, max_iter).ok_or_else(|| {
            LinalgError::DecompositionFailed("Schur decomposition failed to converge".into())
        })?;

        let (q_nalg, t_nalg) = schur.unpack();

        // Convert nalgebra → ndarray
        let mut q = Array2::zeros((n, n));
        let mut t = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                q[(i, j)] = q_nalg[(i, j)];
                t[(i, j)] = t_nalg[(i, j)];
            }
        }

        Ok(SchurResult { q, t })
    }

    fn svd(
        &self,
        a: &Array2<Complex64>,
        compute_u: bool,
        compute_vt: bool,
    ) -> LinalgResult<SvdResult<Array2<Complex64>>> {
        let n = a.nrows();
        let m = a.ncols();

        // Currently only 4x4 matrices are supported (nalgebra Matrix4 bridge).
        if n != 4 || m != 4 {
            return Err(LinalgError::DimensionError(format!(
                "SVD currently only supports 4x4 matrices, got {}x{}",
                n, m
            )));
        }

        // Convert ndarray → nalgebra Matrix4
        let mut nalgebra_m = nalgebra::Matrix4::<Complex64>::zeros();
        for i in 0..n {
            for j in 0..m {
                nalgebra_m[(i, j)] = a[[i, j]];
            }
        }

        let svd = nalgebra::linalg::SVD::new(nalgebra_m, compute_u, compute_vt);
        let singular_values: Vec<f64> = svd.singular_values.iter().map(|v| v.norm()).collect();

        let u = svd.u.map(|umat| {
            let mut result = Array2::zeros((n, n));
            for i in 0..n {
                for j in 0..n {
                    result[[i, j]] = umat[(i, j)];
                }
            }
            result
        });

        let v_t = svd.v_t.map(|vtmat| {
            let mut result = Array2::zeros((m, m));
            for i in 0..m {
                for j in 0..m {
                    result[[i, j]] = vtmat[(i, j)];
                }
            }
            result
        });

        Ok(SvdResult {
            u,
            v_t,
            singular_values,
        })
    }
}
