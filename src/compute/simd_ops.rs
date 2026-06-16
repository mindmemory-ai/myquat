//! SIMD-optimized operations for quantum computing
//!
//! This module provides vectorized operations using SIMD instructions
//! for improved performance in quantum state vector computations.
//!
//! On x86/x86_64: uses AVX2/FMA intrinsics.
//! On aarch64 (Apple Silicon) and other architectures: delegates to fallback implementations.

use crate::error::Result;
use ndarray::{Array1, Array2};
use num_complex::Complex64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;

/// SIMD-optimized quantum state operations
pub struct SimdQuantumOps;

impl SimdQuantumOps {
    /// Check if SIMD operations are available on this CPU
    pub fn is_available() -> bool {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma")
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            false
        }
    }

    /// Get the SIMD vector width for complex numbers
    pub fn simd_width() -> usize {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if Self::is_available() {
                4 // AVX2 can process 4 complex64 numbers at once (256 bits / 64 bits)
            } else {
                1 // Fallback to scalar operations
            }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            1
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized complex number multiplication
    #[target_feature(enable = "avx2,fma")]
    unsafe fn simd_complex_mul_avx2(
        a_real: __m256d,
        a_imag: __m256d,
        b_real: __m256d,
        b_imag: __m256d,
    ) -> (__m256d, __m256d) {
        // Complex multiplication: (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
        let ac = _mm256_mul_pd(a_real, b_real);
        let bd = _mm256_mul_pd(a_imag, b_imag);
        let ad = _mm256_mul_pd(a_real, b_imag);
        let bc = _mm256_mul_pd(a_imag, b_real);

        let result_real = _mm256_sub_pd(ac, bd);
        let result_imag = _mm256_add_pd(ad, bc);

        (result_real, result_imag)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized vector addition
    #[target_feature(enable = "avx2")]
    unsafe fn simd_add_avx2(
        a_real: __m256d,
        a_imag: __m256d,
        b_real: __m256d,
        b_imag: __m256d,
    ) -> (__m256d, __m256d) {
        let result_real = _mm256_add_pd(a_real, b_real);
        let result_imag = _mm256_add_pd(a_imag, b_imag);
        (result_real, result_imag)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized norm squared calculation
    #[target_feature(enable = "avx2,fma")]
    unsafe fn simd_norm_squared_avx2(real: __m256d, imag: __m256d) -> __m256d {
        // |z|² = real² + imag²
        let real_sq = _mm256_mul_pd(real, real);
        let imag_sq = _mm256_mul_pd(imag, imag);
        _mm256_add_pd(real_sq, imag_sq)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// Apply single-qubit gate with SIMD optimization
    pub fn apply_single_qubit_gate_simd(
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        if !Self::is_available() {
            return Err(MyQuatError::circuit_error(
                "SIMD operations not available on this CPU",
            ));
        }

        if gate_matrix.dim() != (2, 2) {
            return Err(MyQuatError::circuit_error(
                "Single qubit gate matrix must be 2x2",
            ));
        }

        let dim = state.len();
        let step = 1 << (num_qubits - 1 - qubit_index);

        // Extract gate matrix elements
        let g00 = gate_matrix[[0, 0]];
        let g01 = gate_matrix[[0, 1]];
        let g10 = gate_matrix[[1, 0]];
        let g11 = gate_matrix[[1, 1]];

        unsafe {
            // Broadcast gate matrix elements to SIMD vectors
            let g00_real = _mm256_set1_pd(g00.re);
            let g00_imag = _mm256_set1_pd(g00.im);
            let g01_real = _mm256_set1_pd(g01.re);
            let g01_imag = _mm256_set1_pd(g01.im);
            let g10_real = _mm256_set1_pd(g10.re);
            let g10_imag = _mm256_set1_pd(g10.im);
            let g11_real = _mm256_set1_pd(g11.re);
            let g11_imag = _mm256_set1_pd(g11.im);

            let simd_width = Self::simd_width();

            // Process state vector in chunks
            for i in (0..dim).step_by(2 * step) {
                for j in (0..step).step_by(simd_width) {
                    let remaining = (step - j).min(simd_width);

                    if remaining == simd_width {
                        // Full SIMD vector processing
                        let idx0_base = i + j;
                        let idx1_base = i + j + step;

                        // Load state amplitudes
                        let mut amp0_real = [0.0; 4];
                        let mut amp0_imag = [0.0; 4];
                        let mut amp1_real = [0.0; 4];
                        let mut amp1_imag = [0.0; 4];

                        for k in 0..simd_width {
                            if idx0_base + k < dim && idx1_base + k < dim {
                                amp0_real[k] = state[idx0_base + k].re;
                                amp0_imag[k] = state[idx0_base + k].im;
                                amp1_real[k] = state[idx1_base + k].re;
                                amp1_imag[k] = state[idx1_base + k].im;
                            }
                        }

                        let amp0_real_vec = _mm256_loadu_pd(amp0_real.as_ptr());
                        let amp0_imag_vec = _mm256_loadu_pd(amp0_imag.as_ptr());
                        let amp1_real_vec = _mm256_loadu_pd(amp1_real.as_ptr());
                        let amp1_imag_vec = _mm256_loadu_pd(amp1_imag.as_ptr());

                        // Compute g00 * amp0
                        let (g00_amp0_real, g00_amp0_imag) = Self::simd_complex_mul_avx2(
                            g00_real,
                            g00_imag,
                            amp0_real_vec,
                            amp0_imag_vec,
                        );

                        // Compute g01 * amp1
                        let (g01_amp1_real, g01_amp1_imag) = Self::simd_complex_mul_avx2(
                            g01_real,
                            g01_imag,
                            amp1_real_vec,
                            amp1_imag_vec,
                        );

                        // Compute g10 * amp0
                        let (g10_amp0_real, g10_amp0_imag) = Self::simd_complex_mul_avx2(
                            g10_real,
                            g10_imag,
                            amp0_real_vec,
                            amp0_imag_vec,
                        );

                        // Compute g11 * amp1
                        let (g11_amp1_real, g11_amp1_imag) = Self::simd_complex_mul_avx2(
                            g11_real,
                            g11_imag,
                            amp1_real_vec,
                            amp1_imag_vec,
                        );

                        // Add results: new_amp0 = g00*amp0 + g01*amp1
                        let (new_amp0_real, new_amp0_imag) = Self::simd_add_avx2(
                            g00_amp0_real,
                            g00_amp0_imag,
                            g01_amp1_real,
                            g01_amp1_imag,
                        );

                        // Add results: new_amp1 = g10*amp0 + g11*amp1
                        let (new_amp1_real, new_amp1_imag) = Self::simd_add_avx2(
                            g10_amp0_real,
                            g10_amp0_imag,
                            g11_amp1_real,
                            g11_amp1_imag,
                        );

                        // Store results back
                        let mut result0_real = [0.0; 4];
                        let mut result0_imag = [0.0; 4];
                        let mut result1_real = [0.0; 4];
                        let mut result1_imag = [0.0; 4];

                        _mm256_storeu_pd(result0_real.as_mut_ptr(), new_amp0_real);
                        _mm256_storeu_pd(result0_imag.as_mut_ptr(), new_amp0_imag);
                        _mm256_storeu_pd(result1_real.as_mut_ptr(), new_amp1_real);
                        _mm256_storeu_pd(result1_imag.as_mut_ptr(), new_amp1_imag);

                        for k in 0..simd_width {
                            if idx0_base + k < dim && idx1_base + k < dim {
                                state[idx0_base + k] =
                                    Complex64::new(result0_real[k], result0_imag[k]);
                                state[idx1_base + k] =
                                    Complex64::new(result1_real[k], result1_imag[k]);
                            }
                        }
                    } else {
                        // Handle remaining elements with scalar operations
                        for k in 0..remaining {
                            let idx0 = i + j + k;
                            let idx1 = i + j + k + step;

                            if idx0 < dim && idx1 < dim {
                                let amp0 = state[idx0];
                                let amp1 = state[idx1];

                                state[idx0] = g00 * amp0 + g01 * amp1;
                                state[idx1] = g10 * amp0 + g11 * amp1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized probability calculation
    pub fn compute_probabilities_simd(state: &Array1<Complex64>) -> Result<Array1<f64>> {
        if !Self::is_available() {
            return Err(MyQuatError::circuit_error(
                "SIMD operations not available on this CPU",
            ));
        }

        let len = state.len();
        let mut probabilities = Array1::zeros(len);
        let simd_width = Self::simd_width();

        unsafe {
            // Process in SIMD chunks
            for i in (0..len).step_by(simd_width) {
                let remaining = (len - i).min(simd_width);

                if remaining == simd_width {
                    // Load complex numbers
                    let mut real_vals = [0.0; 4];
                    let mut imag_vals = [0.0; 4];

                    for j in 0..simd_width {
                        real_vals[j] = state[i + j].re;
                        imag_vals[j] = state[i + j].im;
                    }

                    let real_vec = _mm256_loadu_pd(real_vals.as_ptr());
                    let imag_vec = _mm256_loadu_pd(imag_vals.as_ptr());

                    // Compute |z|² = real² + imag²
                    let norm_squared = Self::simd_norm_squared_avx2(real_vec, imag_vec);

                    // Store results
                    let mut results = [0.0; 4];
                    _mm256_storeu_pd(results.as_mut_ptr(), norm_squared);

                    for j in 0..simd_width {
                        probabilities[i + j] = results[j];
                    }
                } else {
                    // Handle remaining elements with scalar operations
                    for j in 0..remaining {
                        probabilities[i + j] = state[i + j].norm_sqr();
                    }
                }
            }
        }

        Ok(probabilities)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized vector normalization
    pub fn normalize_simd(state: &mut Array1<Complex64>) -> Result<()> {
        if !Self::is_available() {
            return Err(MyQuatError::circuit_error(
                "SIMD operations not available on this CPU",
            ));
        }

        // First compute the norm
        let probabilities = Self::compute_probabilities_simd(state)?;
        let norm_squared: f64 = probabilities.sum();
        let norm = norm_squared.sqrt();

        if norm < 1e-15 {
            return Ok(());
        }

        let inv_norm = 1.0 / norm;
        let len = state.len();
        let simd_width = Self::simd_width();

        unsafe {
            let inv_norm_vec = _mm256_set1_pd(inv_norm);

            // Process in SIMD chunks
            for i in (0..len).step_by(simd_width) {
                let remaining = (len - i).min(simd_width);

                if remaining == simd_width {
                    // Load complex numbers
                    let mut real_vals = [0.0; 4];
                    let mut imag_vals = [0.0; 4];

                    for j in 0..simd_width {
                        real_vals[j] = state[i + j].re;
                        imag_vals[j] = state[i + j].im;
                    }

                    let real_vec = _mm256_loadu_pd(real_vals.as_ptr());
                    let imag_vec = _mm256_loadu_pd(imag_vals.as_ptr());

                    // Multiply by inverse norm
                    let normalized_real = _mm256_mul_pd(real_vec, inv_norm_vec);
                    let normalized_imag = _mm256_mul_pd(imag_vec, inv_norm_vec);

                    // Store results
                    _mm256_storeu_pd(real_vals.as_mut_ptr(), normalized_real);
                    _mm256_storeu_pd(imag_vals.as_mut_ptr(), normalized_imag);

                    for j in 0..simd_width {
                        state[i + j] = Complex64::new(real_vals[j], imag_vals[j]);
                    }
                } else {
                    // Handle remaining elements with scalar operations
                    for j in 0..remaining {
                        state[i + j] *= inv_norm;
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD-optimized inner product calculation
    pub fn inner_product_simd(
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<Complex64> {
        if !Self::is_available() {
            return Err(MyQuatError::circuit_error(
                "SIMD operations not available on this CPU",
            ));
        }

        if state1.len() != state2.len() {
            return Err(MyQuatError::circuit_error(
                "State vectors must have the same length",
            ));
        }

        let len = state1.len();
        let simd_width = Self::simd_width();
        let mut result_real = 0.0;
        let mut result_imag = 0.0;

        unsafe {
            let mut sum_real = _mm256_setzero_pd();
            let mut sum_imag = _mm256_setzero_pd();

            // Process in SIMD chunks
            for i in (0..len).step_by(simd_width) {
                let remaining = (len - i).min(simd_width);

                if remaining == simd_width {
                    // Load complex numbers
                    let mut real1_vals = [0.0; 4];
                    let mut imag1_vals = [0.0; 4];
                    let mut real2_vals = [0.0; 4];
                    let mut imag2_vals = [0.0; 4];

                    for j in 0..simd_width {
                        real1_vals[j] = state1[i + j].re;
                        imag1_vals[j] = -state1[i + j].im; // Conjugate
                        real2_vals[j] = state2[i + j].re;
                        imag2_vals[j] = state2[i + j].im;
                    }

                    let real1_vec = _mm256_loadu_pd(real1_vals.as_ptr());
                    let imag1_vec = _mm256_loadu_pd(imag1_vals.as_ptr());
                    let real2_vec = _mm256_loadu_pd(real2_vals.as_ptr());
                    let imag2_vec = _mm256_loadu_pd(imag2_vals.as_ptr());

                    // Compute conjugate(state1[i]) * state2[i]
                    let (prod_real, prod_imag) =
                        Self::simd_complex_mul_avx2(real1_vec, imag1_vec, real2_vec, imag2_vec);

                    // Accumulate
                    sum_real = _mm256_add_pd(sum_real, prod_real);
                    sum_imag = _mm256_add_pd(sum_imag, prod_imag);
                } else {
                    // Handle remaining elements with scalar operations
                    for j in 0..remaining {
                        let prod = state1[i + j].conj() * state2[i + j];
                        result_real += prod.re;
                        result_imag += prod.im;
                    }
                }
            }

            // Horizontal sum of SIMD vectors
            let mut real_results = [0.0; 4];
            let mut imag_results = [0.0; 4];
            _mm256_storeu_pd(real_results.as_mut_ptr(), sum_real);
            _mm256_storeu_pd(imag_results.as_mut_ptr(), sum_imag);

            for j in 0..4 {
                result_real += real_results[j];
                result_imag += imag_results[j];
            }
        }

        Ok(Complex64::new(result_real, result_imag))
    }
}

/// Fallback implementations for non-SIMD systems
impl SimdQuantumOps {
    /// Fallback single-qubit gate application
    pub fn apply_single_qubit_gate_fallback(
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        // Use the zero-copy implementation from memory_optimized
        use crate::memory_optimized::ZeroCopyMatrixOps;
        ZeroCopyMatrixOps::apply_single_qubit_gate_inplace(
            state,
            gate_matrix,
            qubit_index,
            num_qubits,
        )
    }

    /// Fallback probability calculation
    pub fn compute_probabilities_fallback(state: &Array1<Complex64>) -> Array1<f64> {
        state.mapv(|z| z.norm_sqr())
    }

    /// Fallback normalization
    pub fn normalize_fallback(state: &mut Array1<Complex64>) {
        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        let norm = norm_squared.sqrt();

        if norm > 1e-15 {
            state.mapv_inplace(|z| z / norm);
        }
    }

    /// Fallback inner product
    pub fn inner_product_fallback(
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Complex64 {
        state1
            .iter()
            .zip(state2.iter())
            .map(|(&a, &b)| a.conj() * b)
            .sum()
    }
}

/// High-level SIMD interface that automatically falls back to scalar operations
pub struct AdaptiveSimdOps;

impl AdaptiveSimdOps {
    /// Apply single-qubit gate with automatic SIMD/fallback selection
    pub fn apply_single_qubit_gate(
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if SimdQuantumOps::is_available() {
                return SimdQuantumOps::apply_single_qubit_gate_simd(
                    state,
                    gate_matrix,
                    qubit_index,
                    num_qubits,
                );
            }
        }
        SimdQuantumOps::apply_single_qubit_gate_fallback(
            state,
            gate_matrix,
            qubit_index,
            num_qubits,
        )
    }

    /// Compute probabilities with automatic SIMD/fallback selection
    pub fn compute_probabilities(state: &Array1<Complex64>) -> Array1<f64> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if SimdQuantumOps::is_available() {
                return SimdQuantumOps::compute_probabilities_simd(state)
                    .unwrap_or_else(|_| SimdQuantumOps::compute_probabilities_fallback(state));
            }
        }
        SimdQuantumOps::compute_probabilities_fallback(state)
    }

    /// Normalize state with automatic SIMD/fallback selection
    pub fn normalize(state: &mut Array1<Complex64>) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if SimdQuantumOps::is_available() {
                if SimdQuantumOps::normalize_simd(state).is_err() {
                    SimdQuantumOps::normalize_fallback(state);
                }
                return;
            }
        }
        SimdQuantumOps::normalize_fallback(state);
    }

    /// Compute inner product with automatic SIMD/fallback selection
    pub fn inner_product(state1: &Array1<Complex64>, state2: &Array1<Complex64>) -> Complex64 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if SimdQuantumOps::is_available() {
                return SimdQuantumOps::inner_product_simd(state1, state2)
                    .unwrap_or_else(|_| SimdQuantumOps::inner_product_fallback(state1, state2));
            }
        }
        SimdQuantumOps::inner_product_fallback(state1, state2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_simd_availability() {
        println!("SIMD available: {}", SimdQuantumOps::is_available());
        println!("SIMD width: {}", SimdQuantumOps::simd_width());
    }

    #[test]
    fn test_simd_probabilities() {
        let state = Array1::from_vec(vec![
            Complex64::new(0.5, 0.0),
            Complex64::new(0.0, 0.5),
            Complex64::new(0.5, 0.0),
            Complex64::new(0.0, 0.5),
        ]);

        let probs = AdaptiveSimdOps::compute_probabilities(&state);

        assert_relative_eq!(probs[0], 0.25, epsilon = 1e-10);
        assert_relative_eq!(probs[1], 0.25, epsilon = 1e-10);
        assert_relative_eq!(probs[2], 0.25, epsilon = 1e-10);
        assert_relative_eq!(probs[3], 0.25, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_normalization() {
        let mut state = Array1::from_vec(vec![Complex64::new(2.0, 0.0), Complex64::new(0.0, 2.0)]);

        AdaptiveSimdOps::normalize(&mut state);

        let norm_squared: f64 = state.iter().map(|z| z.norm_sqr()).sum();
        assert_relative_eq!(norm_squared, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_inner_product() {
        let state1 = Array1::from_vec(vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)]);
        let state2 = Array1::from_vec(vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)]);

        let inner_product = AdaptiveSimdOps::inner_product(&state1, &state2);
        assert_relative_eq!(inner_product.re, 0.0, epsilon = 1e-10);

        let inner_product_self = AdaptiveSimdOps::inner_product(&state1, &state1);
        assert_relative_eq!(inner_product_self.re, 1.0, epsilon = 1e-10);
    }
}
