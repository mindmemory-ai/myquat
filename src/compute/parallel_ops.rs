//! Parallel State Vector Operations
//! Author: gA4ss
//!
//! Parallel implementations of common quantum state operations using Rayon.
//! These operations are embarrassingly parallel and provide significant speedup.

use crate::error::{MyQuatError, Result};
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use rayon::prelude::*;

/// Parallel state vector operations
pub struct ParallelStateOps;

impl ParallelStateOps {
    /// Parallel tensor product of two state vectors
    ///
    /// Computes |ψ₁⟩ ⊗ |ψ₂⟩ in parallel
    pub fn tensor_product_parallel(
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Array1<Complex64> {
        let dim1 = state1.len();
        let dim2 = state2.len();
        let result_dim = dim1 * dim2;

        let result_vec: Vec<Complex64> = (0..result_dim)
            .into_par_iter()
            .map(|idx| {
                let i = idx / dim2;
                let j = idx % dim2;
                state1[i] * state2[j]
            })
            .collect();

        Array1::from_vec(result_vec)
    }

    /// Parallel computation of state probabilities
    ///
    /// Computes |⟨i|ψ⟩|² for all basis states
    pub fn compute_probabilities_parallel(state: &Array1<Complex64>) -> Array1<f64> {
        let probs_vec: Vec<f64> = state
            .par_iter()
            .map(|&amplitude| amplitude.norm_sqr())
            .collect();
        Array1::from_vec(probs_vec)
    }

    /// Parallel state normalization
    ///
    /// Normalizes state vector in-place: |ψ⟩ → |ψ⟩ / √⟨ψ|ψ⟩
    pub fn normalize_parallel(state: &mut Array1<Complex64>) {
        let norm_squared: f64 = state.par_iter().map(|&z| z.norm_sqr()).sum();

        let norm = norm_squared.sqrt();

        if norm > 1e-15 {
            state.par_iter_mut().for_each(|z| *z /= norm);
        }
    }

    /// Parallel inner product of two state vectors
    ///
    /// Computes ⟨ψ₁|ψ₂⟩ in parallel
    pub fn inner_product_parallel(
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Complex64 {
        (0..state1.len())
            .into_par_iter()
            .map(|i| state1[i].conj() * state2[i])
            .sum()
    }

    /// Parallel expectation value calculation
    ///
    /// Computes ⟨ψ|O|ψ⟩ for operator O in parallel
    pub fn expectation_value_parallel(
        state: &Array1<Complex64>,
        operator: &Array2<Complex64>,
    ) -> Result<Complex64> {
        if state.len() != operator.nrows() || operator.nrows() != operator.ncols() {
            return Err(MyQuatError::circuit_error(
                "Operator dimensions don't match state vector",
            ));
        }

        let operator_state_vec: Vec<Complex64> = (0..state.len())
            .into_par_iter()
            .map(|i| (0..state.len()).map(|j| operator[[i, j]] * state[j]).sum())
            .collect();
        let operator_state = Array1::from_vec(operator_state_vec);

        let expectation = (0..state.len())
            .into_par_iter()
            .map(|i| state[i].conj() * operator_state[i])
            .sum();

        Ok(expectation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_parallel_tensor_product() {
        let state1 = Array1::from_vec(vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)]);
        let state2 = Array1::from_vec(vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]);

        let tensor_product = ParallelStateOps::tensor_product_parallel(&state1, &state2);
        assert_eq!(tensor_product.len(), 4);
        assert_relative_eq!(tensor_product[0].re, 0.0, epsilon = 1e-10);
        assert_relative_eq!(tensor_product[1].re, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parallel_probabilities() {
        let state = Array1::from_vec(vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)]);

        let probabilities = ParallelStateOps::compute_probabilities_parallel(&state);
        assert_relative_eq!(probabilities[0], 1.0, epsilon = 1e-10);
        assert_relative_eq!(probabilities[1], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parallel_normalization() {
        let mut state = Array1::from_vec(vec![Complex64::new(2.0, 0.0), Complex64::new(0.0, 2.0)]);

        ParallelStateOps::normalize_parallel(&mut state);

        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        assert_relative_eq!(norm_squared, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parallel_inner_product() {
        let state1 = Array1::from_vec(vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)]);
        let state2 = Array1::from_vec(vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]);

        let inner_product = ParallelStateOps::inner_product_parallel(&state1, &state2);
        assert_relative_eq!(inner_product.re, 0.0, epsilon = 1e-10);

        let inner_product_self = ParallelStateOps::inner_product_parallel(&state1, &state1);
        assert_relative_eq!(inner_product_self.re, 1.0, epsilon = 1e-10);
    }
}
