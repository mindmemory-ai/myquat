//! SIMD-optimized two-qubit gate operations
//! Author: gA4ss
//!
//! Provides highly optimized implementations of common two-qubit gates
//! using AVX2/FMA SIMD instructions for maximum performance.
//!
//! On x86/x86_64: uses AVX2/FMA intrinsics.
//! On aarch64 (Apple Silicon) and other architectures: uses fallback implementations.

use crate::error::{MyQuatError, Result};
use ndarray::Array1;
use num_complex::Complex64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;

/// SIMD-optimized two-qubit gate operations
pub struct SimdTwoQubitGates;

impl SimdTwoQubitGates {
    /// Check if SIMD is available
    #[inline]
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

    /// SIMD width for complex numbers (4 complex64 = 256 bits on x86, 1 on ARM)
    #[inline]
    pub fn simd_width() -> usize {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            4
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            1
        }
    }

    /// Apply CNOT gate with SIMD optimization
    ///
    /// CNOT matrix:
    /// ```text
    /// |1 0 0 0|
    /// |0 1 0 0|
    /// |0 0 0 1|
    /// |0 0 1 0|
    /// ```
    ///
    /// # Arguments
    /// * `state` - Quantum state vector
    /// * `control` - Control qubit index
    /// * `target` - Target qubit index
    pub fn apply_cnot_simd(
        state: &mut Array1<Complex64>,
        control: usize,
        target: usize,
    ) -> Result<()> {
        let n_qubits = (state.len() as f64).log2() as usize;

        if control >= n_qubits || target >= n_qubits {
            return Err(MyQuatError::invalid_parameter("Qubit index out of range"));
        }

        if control == target {
            return Err(MyQuatError::invalid_parameter(
                "Control and target must be different",
            ));
        }

        let dim = state.len();
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        if Self::is_available() {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                Self::apply_cnot_simd_impl(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    dim,
                )
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                Self::apply_cnot_fallback(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    dim,
                )
            }
        } else {
            Self::apply_cnot_fallback(
                state.as_slice_mut().unwrap(),
                control_mask,
                target_mask,
                dim,
            )
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD implementation of CNOT
    #[target_feature(enable = "avx2,fma")]
    unsafe fn apply_cnot_simd_impl(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        dim: usize,
    ) {
        // Only process each unordered pair once: for state index i where
        // control bit is 1 and target bit is 0, swap i <-> i^target_mask.
        // This avoids the double-swap bug where iterating over ALL states
        // with control=1 swaps each pair twice (restoring the original).
        for i in 0..dim {
            // Process only when control=1 AND target=0 (so i < i^target_mask)
            if (i & control_mask) != 0 && (i & target_mask) == 0 {
                let j = i ^ target_mask; // Toggle target bit: target=0 -> target=1
                if j < dim {
                    state.swap(i, j);
                }
            }
        }
    }

    /// Fallback scalar implementation
    fn apply_cnot_fallback(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        dim: usize,
    ) {
        // Only process each unordered pair once: for state index i where
        // control bit is 1 and target bit is 0, swap i <-> i^target_mask.
        // This avoids the double-swap bug where iterating over ALL states
        // with control=1 swaps each pair twice (restoring the original).
        for i in 0..dim {
            // Process only when control=1 AND target=0 (so i < i^target_mask)
            if (i & control_mask) != 0 && (i & target_mask) == 0 {
                let j = i ^ target_mask; // Toggle target bit: target=0 -> target=1
                if j < dim {
                    state.swap(i, j);
                }
            }
        }
    }

    /// Apply CZ gate with SIMD optimization
    ///
    /// CZ matrix:
    /// ```text
    /// |1  0  0  0|
    /// |0  1  0  0|
    /// |0  0  1  0|
    /// |0  0  0 -1|
    /// ```
    pub fn apply_cz_simd(
        state: &mut Array1<Complex64>,
        control: usize,
        target: usize,
    ) -> Result<()> {
        let n_qubits = (state.len() as f64).log2() as usize;

        if control >= n_qubits || target >= n_qubits {
            return Err(MyQuatError::invalid_parameter("Qubit index out of range"));
        }

        if control == target {
            return Err(MyQuatError::invalid_parameter(
                "Control and target must be different",
            ));
        }

        let dim = state.len();
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        if Self::is_available() {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                Self::apply_cz_simd_impl(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    dim,
                )
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                Self::apply_cz_fallback(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    dim,
                )
            }
        } else {
            Self::apply_cz_fallback(
                state.as_slice_mut().unwrap(),
                control_mask,
                target_mask,
                dim,
            )
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD implementation of CZ
    #[target_feature(enable = "avx2,fma")]
    unsafe fn apply_cz_simd_impl(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        dim: usize,
    ) {
        let simd_width = Self::simd_width();
        let neg_one = _mm256_set1_pd(-1.0);

        // Process in SIMD chunks
        let mut i = 0;
        while i + simd_width <= dim {
            let mut needs_flip = [false; 4];
            let mut indices = [0usize; 4];

            for k in 0..simd_width {
                let idx = i + k;
                indices[k] = idx;
                // Flip phase if both control and target bits are 1
                needs_flip[k] = (idx & control_mask) != 0 && (idx & target_mask) != 0;
            }

            // Load real and imaginary parts
            let mut real_parts = [0.0; 4];
            let mut imag_parts = [0.0; 4];

            for k in 0..simd_width {
                real_parts[k] = state[indices[k]].re;
                imag_parts[k] = state[indices[k]].im;
            }

            let mut real_vec = _mm256_loadu_pd(real_parts.as_ptr());
            let mut imag_vec = _mm256_loadu_pd(imag_parts.as_ptr());

            // Apply phase flip where needed
            for k in 0..simd_width {
                if needs_flip[k] {
                    real_parts[k] = -real_parts[k];
                    imag_parts[k] = -imag_parts[k];
                }
            }

            real_vec = _mm256_loadu_pd(real_parts.as_ptr());
            imag_vec = _mm256_loadu_pd(imag_parts.as_ptr());

            // Store results
            _mm256_storeu_pd(real_parts.as_mut_ptr(), real_vec);
            _mm256_storeu_pd(imag_parts.as_mut_ptr(), imag_vec);

            for k in 0..simd_width {
                state[indices[k]] = Complex64::new(real_parts[k], imag_parts[k]);
            }

            i += simd_width;
        }

        // Handle remaining elements
        for idx in i..dim {
            if (idx & control_mask) != 0 && (idx & target_mask) != 0 {
                state[idx] = -state[idx];
            }
        }
    }

    /// Fallback scalar implementation of CZ
    fn apply_cz_fallback(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        dim: usize,
    ) {
        for i in 0..dim {
            if (i & control_mask) != 0 && (i & target_mask) != 0 {
                state[i] = -state[i];
            }
        }
    }

    /// Apply SWAP gate with SIMD optimization
    ///
    /// SWAP matrix:
    /// ```text
    /// |1 0 0 0|
    /// |0 0 1 0|
    /// |0 1 0 0|
    /// |0 0 0 1|
    /// ```
    pub fn apply_swap_simd(
        state: &mut Array1<Complex64>,
        qubit1: usize,
        qubit2: usize,
    ) -> Result<()> {
        let n_qubits = (state.len() as f64).log2() as usize;

        if qubit1 >= n_qubits || qubit2 >= n_qubits {
            return Err(MyQuatError::invalid_parameter("Qubit index out of range"));
        }

        if qubit1 == qubit2 {
            return Ok(()); // SWAP with itself is identity
        }

        let dim = state.len();
        let mask1 = 1 << qubit1;
        let mask2 = 1 << qubit2;

        // SWAP can be done efficiently by swapping amplitudes
        for i in 0..dim {
            let bit1 = (i & mask1) != 0;
            let bit2 = (i & mask2) != 0;

            // Only swap if bits are different
            if bit1 != bit2 {
                let j = i ^ mask1 ^ mask2;
                if i < j {
                    state.as_slice_mut().unwrap().swap(i, j);
                }
            }
        }

        Ok(())
    }

    /// Apply controlled-phase gate with SIMD optimization
    ///
    /// CP(θ) matrix:
    /// ```text
    /// |1 0 0   0  |
    /// |0 1 0   0  |
    /// |0 0 1   0  |
    /// |0 0 0 e^iθ |
    /// ```
    pub fn apply_cp_simd(
        state: &mut Array1<Complex64>,
        control: usize,
        target: usize,
        theta: f64,
    ) -> Result<()> {
        let n_qubits = (state.len() as f64).log2() as usize;

        if control >= n_qubits || target >= n_qubits {
            return Err(MyQuatError::invalid_parameter("Qubit index out of range"));
        }

        if control == target {
            return Err(MyQuatError::invalid_parameter(
                "Control and target must be different",
            ));
        }

        let dim = state.len();
        let control_mask = 1 << control;
        let target_mask = 1 << target;

        // Phase factor e^(iθ)
        let phase = Complex64::new(theta.cos(), theta.sin());

        if Self::is_available() {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                Self::apply_cp_simd_impl(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    phase,
                    dim,
                )
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                Self::apply_cp_fallback(
                    state.as_slice_mut().unwrap(),
                    control_mask,
                    target_mask,
                    phase,
                    dim,
                )
            }
        } else {
            Self::apply_cp_fallback(
                state.as_slice_mut().unwrap(),
                control_mask,
                target_mask,
                phase,
                dim,
            )
        }

        Ok(())
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    /// SIMD implementation of controlled-phase
    #[target_feature(enable = "avx2,fma")]
    unsafe fn apply_cp_simd_impl(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        phase: Complex64,
        dim: usize,
    ) {
        let phase_real = _mm256_set1_pd(phase.re);
        let phase_imag = _mm256_set1_pd(phase.im);
        let simd_width = Self::simd_width();

        let mut i = 0;
        while i + simd_width <= dim {
            let mut needs_phase = [false; 4];
            let mut indices = [0usize; 4];

            for k in 0..simd_width {
                let idx = i + k;
                indices[k] = idx;
                needs_phase[k] = (idx & control_mask) != 0 && (idx & target_mask) != 0;
            }

            // Load amplitudes
            let mut real_parts = [0.0; 4];
            let mut imag_parts = [0.0; 4];

            for k in 0..simd_width {
                real_parts[k] = state[indices[k]].re;
                imag_parts[k] = state[indices[k]].im;
            }

            let amp_real = _mm256_loadu_pd(real_parts.as_ptr());
            let amp_imag = _mm256_loadu_pd(imag_parts.as_ptr());

            // Complex multiplication: amp * phase
            let ac = _mm256_mul_pd(amp_real, phase_real);
            let bd = _mm256_mul_pd(amp_imag, phase_imag);
            let ad = _mm256_mul_pd(amp_real, phase_imag);
            let bc = _mm256_mul_pd(amp_imag, phase_real);

            let new_real = _mm256_sub_pd(ac, bd);
            let new_imag = _mm256_add_pd(ad, bc);

            // Store results
            _mm256_storeu_pd(real_parts.as_mut_ptr(), new_real);
            _mm256_storeu_pd(imag_parts.as_mut_ptr(), new_imag);

            for k in 0..simd_width {
                if needs_phase[k] {
                    state[indices[k]] = Complex64::new(real_parts[k], imag_parts[k]);
                }
            }

            i += simd_width;
        }

        // Handle remaining elements
        for idx in i..dim {
            if (idx & control_mask) != 0 && (idx & target_mask) != 0 {
                state[idx] *= phase;
            }
        }
    }

    /// Fallback scalar implementation of controlled-phase
    fn apply_cp_fallback(
        state: &mut [Complex64],
        control_mask: usize,
        target_mask: usize,
        phase: Complex64,
        dim: usize,
    ) {
        for i in 0..dim {
            if (i & control_mask) != 0 && (i & target_mask) != 0 {
                state[i] *= phase;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_simd_cnot_distinct_amplitudes() {
        // Use distinct amplitudes so a no-op swap is detectable.
        // Qubit indices in little-endian: index = |q1 q0>
        // CNOT(control=0, target=1): when q0=1, flip q1.
        //   |01> (index 1, q0=1, q1=0) <-> |11> (index 3, q0=1, q1=1)
        //   |00> (index 0, q0=0) unchanged, |10> (index 2, q0=0) unchanged
        let mut state = Array1::from_vec(vec![
            Complex64::new(1.0, 0.0), // |00⟩
            Complex64::new(2.0, 0.0), // |01⟩
            Complex64::new(3.0, 0.0), // |10⟩
            Complex64::new(4.0, 0.0), // |11⟩
        ]);

        SimdTwoQubitGates::apply_cnot_simd(&mut state, 0, 1).unwrap();

        // |00⟩ and |10⟩ unchanged (control q0=0)
        assert_relative_eq!(state[0].re, 1.0, epsilon = 1e-10);
        assert_relative_eq!(state[2].re, 3.0, epsilon = 1e-10);
        // |01⟩ ↔ |11⟩ swapped (control q0=1, target q1 flipped)
        assert_relative_eq!(state[1].re, 4.0, epsilon = 1e-10);
        assert_relative_eq!(state[3].re, 2.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_cnot_swapped_qubits() {
        // CNOT(control=1, target=0): when q1=1, flip q0.
        //   |10> (index 2, q1=1, q0=0) <-> |11> (index 3, q1=1, q0=1)
        //   |00> (index 0, q1=0) unchanged, |01> (index 1, q1=0) unchanged
        let mut state = Array1::from_vec(vec![
            Complex64::new(1.0, 0.0), // |00⟩
            Complex64::new(2.0, 0.0), // |01⟩
            Complex64::new(3.0, 0.0), // |10⟩
            Complex64::new(4.0, 0.0), // |11⟩
        ]);

        SimdTwoQubitGates::apply_cnot_simd(&mut state, 1, 0).unwrap();

        // |00⟩ and |01⟩ unchanged (control q1=0)
        assert_relative_eq!(state[0].re, 1.0, epsilon = 1e-10);
        assert_relative_eq!(state[1].re, 2.0, epsilon = 1e-10);
        // |10⟩ ↔ |11⟩ swapped (control q1=1, target q0 flipped)
        assert_relative_eq!(state[2].re, 4.0, epsilon = 1e-10);
        assert_relative_eq!(state[3].re, 3.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_cz() {
        let mut state = Array1::from_vec(vec![
            Complex64::new(0.5, 0.0), // |00⟩
            Complex64::new(0.5, 0.0), // |01⟩
            Complex64::new(0.5, 0.0), // |10⟩
            Complex64::new(0.5, 0.0), // |11⟩
        ]);

        SimdTwoQubitGates::apply_cz_simd(&mut state, 0, 1).unwrap();

        // After CZ(0,1): only |11⟩ gets phase flip
        assert_relative_eq!(state[0].re, 0.5, epsilon = 1e-10);
        assert_relative_eq!(state[1].re, 0.5, epsilon = 1e-10);
        assert_relative_eq!(state[2].re, 0.5, epsilon = 1e-10);
        assert_relative_eq!(state[3].re, -0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_swap() {
        let mut state = Array1::from_vec(vec![
            Complex64::new(1.0, 0.0), // |00⟩
            Complex64::new(0.0, 0.0), // |01⟩
            Complex64::new(0.0, 0.0), // |10⟩
            Complex64::new(0.0, 0.0), // |11⟩
        ]);

        SimdTwoQubitGates::apply_swap_simd(&mut state, 0, 1).unwrap();

        // SWAP doesn't change |00⟩
        assert_relative_eq!(state[0].re, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_simd_cp() {
        use std::f64::consts::PI;

        let mut state = Array1::from_vec(vec![
            Complex64::new(0.5, 0.0),
            Complex64::new(0.5, 0.0),
            Complex64::new(0.5, 0.0),
            Complex64::new(0.5, 0.0),
        ]);

        SimdTwoQubitGates::apply_cp_simd(&mut state, 0, 1, PI).unwrap();

        // After CP(π): |11⟩ gets phase e^(iπ) = -1
        assert_relative_eq!(state[3].re, -0.5, epsilon = 1e-10);
    }
}
