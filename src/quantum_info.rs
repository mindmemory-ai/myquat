//! Quantum information theory utilities
//!
//! This module provides tools for quantum information processing,
//! including state manipulation, entanglement measures, and quantum channels.

use crate::error::{MyQuatError, Result};
use crate::linalg::{LinalgBackend, LinalgResult, LinalgScalar, NdArrayBackend};
use ndarray::{Array1, Array2};
use num_complex::Complex64;

/// A quantum state vector
#[derive(Debug, Clone)]
pub struct QuantumState {
    /// State amplitudes
    amplitudes: Array1<Complex64>,
    /// Number of qubits
    num_qubits: usize,
}

impl QuantumState {
    /// Create a new quantum state from amplitudes
    pub fn new(amplitudes: Array1<Complex64>) -> Result<Self> {
        let dim = amplitudes.len();

        // Check if dimension is a power of 2
        if !dim.is_power_of_two() || dim == 0 {
            return Err(MyQuatError::quantum_info_error(format!(
                "State dimension {} is not a power of 2",
                dim
            )));
        }

        let num_qubits = (dim as f64).log2() as usize;

        // Check normalization
        let norm_squared: f64 = amplitudes.iter().map(|z| z.norm_sqr()).sum();
        if (norm_squared - 1.0).abs() > 1e-10 {
            return Err(MyQuatError::quantum_info_error(format!(
                "State is not normalized: norm^2 = {}",
                norm_squared
            )));
        }

        Ok(QuantumState {
            amplitudes,
            num_qubits,
        })
    }

    /// Create a normalized quantum state (automatically normalizes the input)
    pub fn new_normalized(mut amplitudes: Array1<Complex64>) -> Result<Self> {
        let dim = amplitudes.len();

        if !dim.is_power_of_two() || dim == 0 {
            return Err(MyQuatError::quantum_info_error(format!(
                "State dimension {} is not a power of 2",
                dim
            )));
        }

        // Normalize
        let norm: f64 = amplitudes.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();
        if norm < 1e-15 {
            return Err(MyQuatError::quantum_info_error(
                "Cannot normalize zero state",
            ));
        }

        amplitudes.mapv_inplace(|z| z / norm);

        let num_qubits = (dim as f64).log2() as usize;

        Ok(QuantumState {
            amplitudes,
            num_qubits,
        })
    }

    /// Create a quantum state from a backend vector
    pub fn new_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        amplitudes: &B::Vector,
        backend: &B,
    ) -> LinalgResult<Self> {
        let len = backend.len_vector(amplitudes);
        let mut nda = Array1::zeros(len);
        for i in 0..len {
            nda[i] = backend.get_vector(amplitudes, i)?;
        }
        Self::new(nda).map_err(|e| crate::linalg::LinalgError::BackendError(e.to_string()))
    }

    /// Create the |0⟩ state for n qubits
    pub fn zero_state(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut amplitudes = Array1::zeros(dim);
        amplitudes[0] = Complex64::new(1.0, 0.0);

        QuantumState {
            amplitudes,
            num_qubits,
        }
    }

    /// Create the |+⟩ state (equal superposition) for n qubits
    pub fn plus_state(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let amplitude = Complex64::new(1.0 / (dim as f64).sqrt(), 0.0);
        let amplitudes = Array1::from_elem(dim, amplitude);

        QuantumState {
            amplitudes,
            num_qubits,
        }
    }

    /// Create a computational basis state |i⟩
    pub fn computational_basis_state(num_qubits: usize, state: usize) -> Result<Self> {
        let dim = 1 << num_qubits;
        if state >= dim {
            return Err(MyQuatError::quantum_info_error(format!(
                "State index {} is out of range for {} qubits",
                state, num_qubits
            )));
        }

        let mut amplitudes = Array1::zeros(dim);
        amplitudes[state] = Complex64::new(1.0, 0.0);

        Ok(QuantumState {
            amplitudes,
            num_qubits,
        })
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the state dimension
    pub fn dim(&self) -> usize {
        self.amplitudes.len()
    }

    /// Get the amplitudes
    pub fn amplitudes(&self) -> &Array1<Complex64> {
        &self.amplitudes
    }

    /// Get a specific amplitude
    pub fn amplitude(&self, index: usize) -> Result<Complex64> {
        if index >= self.dim() {
            return Err(MyQuatError::quantum_info_error(format!(
                "Index {} is out of range for state dimension {}",
                index,
                self.dim()
            )));
        }
        Ok(self.amplitudes[index])
    }

    /// Get the probability of measuring a specific computational basis state
    pub fn probability(&self, index: usize) -> Result<f64> {
        Ok(self.amplitude(index)?.norm_sqr())
    }

    /// Get all probabilities
    pub fn probabilities(&self) -> Array1<f64> {
        self.amplitudes.mapv(|z| z.norm_sqr())
    }

    /// Apply a unitary matrix to the state
    pub fn apply_unitary(&mut self, unitary: &Array2<Complex64>) -> Result<()> {
        if unitary.dim() != (self.dim(), self.dim()) {
            return Err(MyQuatError::quantum_info_error(format!(
                "Unitary matrix dimension {:?} doesn't match state dimension {}",
                unitary.dim(),
                self.dim()
            )));
        }

        self.amplitudes = unitary.dot(&self.amplitudes);
        Ok(())
    }

    /// Compute the inner product with another state
    pub fn inner_product(&self, other: &QuantumState) -> Result<Complex64> {
        if self.num_qubits != other.num_qubits {
            return Err(MyQuatError::quantum_info_error(
                "Cannot compute inner product of states with different numbers of qubits",
            ));
        }

        let product = self
            .amplitudes
            .iter()
            .zip(other.amplitudes.iter())
            .map(|(a, b)| a.conj() * b)
            .sum();

        Ok(product)
    }

    /// Compute the fidelity with another state
    pub fn fidelity(&self, other: &QuantumState) -> Result<f64> {
        let inner = self.inner_product(other)?;
        Ok(inner.norm())
    }

    /// Compute the trace distance with another state
    pub fn trace_distance(&self, other: &QuantumState) -> Result<f64> {
        if self.num_qubits != other.num_qubits {
            return Err(MyQuatError::quantum_info_error(
                "Cannot compute trace distance of states with different numbers of qubits",
            ));
        }

        let diff: Array1<Complex64> = &self.amplitudes - &other.amplitudes;
        let trace_distance = 0.5 * diff.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt();

        Ok(trace_distance)
    }

    /// Measure the state in the computational basis
    pub fn measure(&self) -> (usize, f64) {
        use rand::Rng;
        let mut rng = rand::rng();
        let random_value: f64 = rng.random();

        let mut cumulative_prob = 0.0;
        for (i, amplitude) in self.amplitudes.iter().enumerate() {
            cumulative_prob += amplitude.norm_sqr();
            if random_value <= cumulative_prob {
                return (i, amplitude.norm_sqr());
            }
        }

        // Fallback (should not happen with proper normalization)
        let last_idx = self.amplitudes.len() - 1;
        (last_idx, self.amplitudes[last_idx].norm_sqr())
    }

    /// Partial trace over specified qubits
    pub fn partial_trace(&self, traced_qubits: &[usize]) -> Result<DensityMatrix> {
        // For now, implement a simple case
        if traced_qubits.is_empty() {
            return Ok(DensityMatrix::from_state(self));
        }

        // This is a complex operation that would require careful indexing
        // For now, return an error for non-trivial cases
        Err(MyQuatError::quantum_info_error(
            "Partial trace not yet implemented for general cases",
        ))
    }

    /// Compute the Von Neumann entropy of the state (treating it as a pure state)
    pub fn entropy(&self) -> f64 {
        // For a pure state, the entropy is 0
        0.0
    }

    /// Check if the state is separable (for multi-qubit states)
    pub fn is_separable(&self) -> bool {
        if self.num_qubits <= 1 {
            return true;
        }

        // This is a complex problem in general
        // For now, return false (assume entangled) for multi-qubit states
        false
    }
}

/// A density matrix representation of a quantum state
#[derive(Debug, Clone)]
pub struct DensityMatrix {
    /// Density matrix
    matrix: Array2<Complex64>,
    /// Number of qubits
    num_qubits: usize,
}

impl DensityMatrix {
    /// Create a density matrix from a matrix
    pub fn new(matrix: Array2<Complex64>) -> Result<Self> {
        let (rows, cols) = matrix.dim();
        if rows != cols {
            return Err(MyQuatError::quantum_info_error(
                "Density matrix must be square",
            ));
        }

        if !rows.is_power_of_two() || rows == 0 {
            return Err(MyQuatError::quantum_info_error(format!(
                "Matrix dimension {} is not a power of 2",
                rows
            )));
        }

        let num_qubits = (rows as f64).log2() as usize;

        // Check if matrix is Hermitian (approximately)
        let is_hermitian = matrix
            .iter()
            .zip(matrix.t().iter())
            .all(|(a, b)| (a - b.conj()).norm() < 1e-10);

        if !is_hermitian {
            return Err(MyQuatError::quantum_info_error(
                "Density matrix must be Hermitian",
            ));
        }

        // Check if trace is 1 (approximately)
        let trace: Complex64 = (0..rows).map(|i| matrix[[i, i]]).sum();
        if (trace.re - 1.0).abs() > 1e-10 || trace.im.abs() > 1e-10 {
            return Err(MyQuatError::quantum_info_error(format!(
                "Density matrix trace is {}, should be 1",
                trace
            )));
        }

        Ok(DensityMatrix { matrix, num_qubits })
    }

    /// Create a density matrix from a pure state
    pub fn from_state(state: &QuantumState) -> Self {
        let dim = state.dim();
        let mut matrix = Array2::zeros((dim, dim));

        for i in 0..dim {
            for j in 0..dim {
                matrix[[i, j]] = state.amplitudes[i] * state.amplitudes[j].conj();
            }
        }

        DensityMatrix {
            matrix,
            num_qubits: state.num_qubits,
        }
    }

    /// Create a maximally mixed state
    pub fn maximally_mixed(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let value = Complex64::new(1.0 / dim as f64, 0.0);
        let matrix = Array2::from_elem((dim, dim), Complex64::new(0.0, 0.0))
            + Array2::<Complex64>::eye(dim).mapv(|_| value);

        DensityMatrix { matrix, num_qubits }
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the matrix dimension
    pub fn dim(&self) -> usize {
        self.matrix.nrows()
    }

    /// Get the density matrix
    pub fn matrix(&self) -> &Array2<Complex64> {
        &self.matrix
    }

    /// Compute the trace of the density matrix
    pub fn trace(&self) -> Complex64 {
        (0..self.dim()).map(|i| self.matrix[[i, i]]).sum()
    }

    /// Compute the purity of the state
    pub fn purity(&self) -> f64 {
        let trace_rho_squared = self
            .matrix
            .dot(&self.matrix)
            .diag()
            .iter()
            .sum::<Complex64>();
        trace_rho_squared.re
    }

    /// Check if the state is pure
    pub fn is_pure(&self, tolerance: f64) -> bool {
        (self.purity() - 1.0).abs() < tolerance
    }

    /// Compute the Von Neumann entropy
    pub fn entropy(&self) -> Result<f64> {
        // For now, implement a simple approximation
        // In a full implementation, we would compute eigenvalues
        let purity = self.purity();
        if purity > 0.999 {
            Ok(0.0) // Nearly pure state
        } else {
            // Rough approximation for mixed states
            Ok(-purity * purity.log2())
        }
    }

    /// Apply a unitary operation
    pub fn apply_unitary(&mut self, unitary: &Array2<Complex64>) -> Result<()> {
        if unitary.dim() != (self.dim(), self.dim()) {
            return Err(MyQuatError::quantum_info_error(
                "Unitary matrix dimension doesn't match density matrix dimension",
            ));
        }

        // ρ' = U ρ U†
        let u_rho = unitary.dot(&self.matrix);
        self.matrix = u_rho.dot(&unitary.t().mapv(|z| z.conj()));

        Ok(())
    }

    /// Compute the fidelity with another density matrix
    pub fn fidelity(&self, other: &DensityMatrix) -> Result<f64> {
        if self.num_qubits != other.num_qubits {
            return Err(MyQuatError::quantum_info_error(
                "Cannot compute fidelity of states with different numbers of qubits",
            ));
        }

        // For now, implement a simple case for pure states
        // Full implementation would require matrix square root
        if self.is_pure(1e-10) && other.is_pure(1e-10) {
            // For pure states, fidelity is |⟨ψ|φ⟩|²
            let overlap = self
                .matrix
                .iter()
                .zip(other.matrix.iter())
                .map(|(a, b)| a * b.conj())
                .sum::<Complex64>();
            Ok(overlap.norm())
        } else {
            // Rough approximation for mixed states
            let trace_product = self
                .matrix
                .dot(&other.matrix)
                .diag()
                .iter()
                .sum::<Complex64>();
            Ok(trace_product.re.abs())
        }
    }
}

/// Quantum information utilities
pub struct QuantumInfo;

impl QuantumInfo {
    /// Compute the tensor product of two states
    pub fn tensor_product(state1: &QuantumState, state2: &QuantumState) -> QuantumState {
        let dim1 = state1.dim();
        let dim2 = state2.dim();
        let mut amplitudes = Array1::zeros(dim1 * dim2);

        for i in 0..dim1 {
            for j in 0..dim2 {
                amplitudes[i * dim2 + j] = state1.amplitudes[i] * state2.amplitudes[j];
            }
        }

        QuantumState {
            amplitudes,
            num_qubits: state1.num_qubits + state2.num_qubits,
        }
    }

    /// Compute the concurrence of a two-qubit state (entanglement measure)
    pub fn concurrence(state: &QuantumState) -> Result<f64> {
        if state.num_qubits() != 2 {
            return Err(MyQuatError::quantum_info_error(
                "Concurrence is only defined for two-qubit states",
            ));
        }

        let _rho = DensityMatrix::from_state(state);

        // For a pure state |ψ⟩ = a|00⟩ + b|01⟩ + c|10⟩ + d|11⟩
        // Concurrence = 2|ad - bc|
        let a = state.amplitudes[0]; // |00⟩
        let b = state.amplitudes[1]; // |01⟩
        let c = state.amplitudes[2]; // |10⟩
        let d = state.amplitudes[3]; // |11⟩

        let concurrence = 2.0 * (a * d - b * c).norm();
        Ok(concurrence)
    }

    /// Generate a random quantum state
    pub fn random_state(num_qubits: usize) -> QuantumState {
        use rand::Rng;
        let mut rng = rand::rng();
        let dim = 1 << num_qubits;

        let mut amplitudes = Array1::zeros(dim);
        for i in 0..dim {
            let real: f64 = rng.random_range(-1.0..1.0);
            let imag: f64 = rng.random_range(-1.0..1.0);
            amplitudes[i] = Complex64::new(real, imag);
        }

        QuantumState::new_normalized(amplitudes).unwrap()
    }

    /// Create a Bell state
    pub fn bell_state(which: usize) -> Result<QuantumState> {
        if which >= 4 {
            return Err(MyQuatError::quantum_info_error(
                "Bell state index must be 0, 1, 2, or 3",
            ));
        }

        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut amplitudes = Array1::zeros(4);

        match which {
            0 => {
                // |Φ+⟩ = (|00⟩ + |11⟩)/√2
                amplitudes[0] = Complex64::new(inv_sqrt2, 0.0);
                amplitudes[3] = Complex64::new(inv_sqrt2, 0.0);
            }
            1 => {
                // |Φ-⟩ = (|00⟩ - |11⟩)/√2
                amplitudes[0] = Complex64::new(inv_sqrt2, 0.0);
                amplitudes[3] = Complex64::new(-inv_sqrt2, 0.0);
            }
            2 => {
                // |Ψ+⟩ = (|01⟩ + |10⟩)/√2
                amplitudes[1] = Complex64::new(inv_sqrt2, 0.0);
                amplitudes[2] = Complex64::new(inv_sqrt2, 0.0);
            }
            3 => {
                // |Ψ-⟩ = (|01⟩ - |10⟩)/√2
                amplitudes[1] = Complex64::new(inv_sqrt2, 0.0);
                amplitudes[2] = Complex64::new(-inv_sqrt2, 0.0);
            }
            _ => unreachable!(),
        }

        Ok(QuantumState {
            amplitudes,
            num_qubits: 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_zero_state() {
        let state = QuantumState::zero_state(2);
        assert_eq!(state.num_qubits(), 2);
        assert_eq!(state.dim(), 4);

        assert_eq!(state.amplitude(0).unwrap(), Complex64::new(1.0, 0.0));
        for i in 1..4 {
            assert_eq!(state.amplitude(i).unwrap(), Complex64::new(0.0, 0.0));
        }
    }

    #[test]
    fn test_plus_state() {
        let state = QuantumState::plus_state(2);
        let expected_amplitude = Complex64::new(0.5, 0.0);

        for i in 0..4 {
            assert_relative_eq!(
                state.amplitude(i).unwrap().re,
                expected_amplitude.re,
                epsilon = 1e-10
            );
        }
    }

    #[test]
    fn test_bell_state() {
        let bell = QuantumInfo::bell_state(0).unwrap(); // |Φ+⟩
        let inv_sqrt2 = Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0);

        assert_eq!(bell.amplitude(0).unwrap(), inv_sqrt2);
        assert_eq!(bell.amplitude(1).unwrap(), Complex64::new(0.0, 0.0));
        assert_eq!(bell.amplitude(2).unwrap(), Complex64::new(0.0, 0.0));
        assert_eq!(bell.amplitude(3).unwrap(), inv_sqrt2);

        // Test concurrence (should be 1 for maximally entangled state)
        let concurrence = QuantumInfo::concurrence(&bell).unwrap();
        assert_relative_eq!(concurrence, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_density_matrix() {
        let state = QuantumState::zero_state(1);
        let rho = DensityMatrix::from_state(&state);

        assert!(rho.is_pure(1e-10));
        assert_relative_eq!(rho.purity(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rho.entropy().unwrap(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_fidelity() {
        let state1 = QuantumState::zero_state(1);
        let state2 = QuantumState::zero_state(1);
        let state3 = QuantumState::computational_basis_state(1, 1).unwrap();

        assert_relative_eq!(state1.fidelity(&state2).unwrap(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(state1.fidelity(&state3).unwrap(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_tensor_product() {
        let state1 = QuantumState::zero_state(1);
        let state2 = QuantumState::computational_basis_state(1, 1).unwrap();

        let product = QuantumInfo::tensor_product(&state1, &state2);
        assert_eq!(product.num_qubits(), 2);
        assert_eq!(product.dim(), 4);

        // Should be |01⟩
        assert_eq!(product.amplitude(0).unwrap(), Complex64::new(0.0, 0.0));
        assert_eq!(product.amplitude(1).unwrap(), Complex64::new(1.0, 0.0));
        assert_eq!(product.amplitude(2).unwrap(), Complex64::new(0.0, 0.0));
        assert_eq!(product.amplitude(3).unwrap(), Complex64::new(0.0, 0.0));
    }
}

// ============================================================================
// Sparse State Vector (Phase 4.2, Task 4.2.1)
// ============================================================================

use std::collections::HashMap;

/// Sparse quantum state vector representation
///
/// Stores only non-zero amplitudes for memory efficiency.
/// Useful for states with low entanglement or computational basis states.
///
/// # Memory Efficiency
/// - Dense state: O(2^n) memory
/// - Sparse state: O(k) memory, where k = number of non-zero amplitudes
///
/// # Use Cases
/// - Computational basis states (k=1)
/// - Low-entanglement states
/// - Large qubit systems with sparse structure
#[derive(Debug, Clone)]
pub struct SparseQuantumState {
    /// Non-zero amplitudes: index -> amplitude
    amplitudes: HashMap<usize, Complex64>,
    /// Number of qubits
    num_qubits: usize,
    /// Sparsity threshold for considering amplitude as zero
    threshold: f64,
}

impl SparseQuantumState {
    /// Create a new sparse quantum state
    pub fn new(num_qubits: usize) -> Self {
        Self {
            amplitudes: HashMap::new(),
            num_qubits,
            threshold: 1e-15,
        }
    }

    /// Create from dense state vector
    pub fn from_dense(state: &QuantumState, threshold: f64) -> Self {
        let mut sparse = Self::new(state.num_qubits());
        sparse.threshold = threshold;

        for (i, &amp) in state.amplitudes().iter().enumerate() {
            if amp.norm_sqr() > threshold * threshold {
                sparse.amplitudes.insert(i, amp);
            }
        }

        sparse
    }

    /// Create the |0⟩ state
    pub fn zero_state(num_qubits: usize) -> Self {
        let mut state = Self::new(num_qubits);
        state.amplitudes.insert(0, Complex64::new(1.0, 0.0));
        state
    }

    /// Create a computational basis state |i⟩
    pub fn computational_basis_state(num_qubits: usize, index: usize) -> Result<Self> {
        let dim = 1 << num_qubits;
        if index >= dim {
            return Err(MyQuatError::quantum_info_error(format!(
                "Index {} out of range for {} qubits",
                index, num_qubits
            )));
        }

        let mut state = Self::new(num_qubits);
        state.amplitudes.insert(index, Complex64::new(1.0, 0.0));
        Ok(state)
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get state dimension
    pub fn dim(&self) -> usize {
        1 << self.num_qubits
    }

    /// Get number of non-zero amplitudes
    pub fn num_nonzero(&self) -> usize {
        self.amplitudes.len()
    }

    /// Get sparsity ratio (non-zero / total)
    pub fn sparsity(&self) -> f64 {
        self.num_nonzero() as f64 / self.dim() as f64
    }

    /// Get amplitude at index
    pub fn amplitude(&self, index: usize) -> Complex64 {
        self.amplitudes
            .get(&index)
            .copied()
            .unwrap_or(Complex64::new(0.0, 0.0))
    }

    /// Set amplitude at index
    pub fn set_amplitude(&mut self, index: usize, amplitude: Complex64) {
        if amplitude.norm_sqr() > self.threshold * self.threshold {
            self.amplitudes.insert(index, amplitude);
        } else {
            self.amplitudes.remove(&index);
        }
    }

    /// Get probability at index
    pub fn probability(&self, index: usize) -> f64 {
        self.amplitude(index).norm_sqr()
    }

    /// Convert to dense state
    pub fn to_dense(&self) -> QuantumState {
        let dim = self.dim();
        let mut amplitudes = Array1::zeros(dim);

        for (&index, &amp) in &self.amplitudes {
            amplitudes[index] = amp;
        }

        QuantumState::new_normalized(amplitudes)
            .unwrap_or_else(|_| QuantumState::zero_state(self.num_qubits))
    }

    /// Normalize the state
    pub fn normalize(&mut self) {
        let norm_sq: f64 = self.amplitudes.values().map(|z| z.norm_sqr()).sum();
        if norm_sq > 1e-15 {
            let norm = norm_sq.sqrt();
            for amp in self.amplitudes.values_mut() {
                *amp /= norm;
            }
        }
    }

    /// Get all non-zero indices
    pub fn nonzero_indices(&self) -> Vec<usize> {
        let mut indices: Vec<_> = self.amplitudes.keys().copied().collect();
        indices.sort_unstable();
        indices
    }

    /// Memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        // HashMap overhead + entries
        std::mem::size_of::<HashMap<usize, Complex64>>()
            + self.num_nonzero() * (std::mem::size_of::<usize>() + std::mem::size_of::<Complex64>())
    }
}

#[cfg(test)]
mod sparse_tests {
    use super::*;

    #[test]
    fn test_sparse_zero_state() {
        let state = SparseQuantumState::zero_state(3);
        assert_eq!(state.num_qubits(), 3);
        assert_eq!(state.dim(), 8);
        assert_eq!(state.num_nonzero(), 1);
        assert_eq!(state.amplitude(0), Complex64::new(1.0, 0.0));
        assert_eq!(state.amplitude(1), Complex64::new(0.0, 0.0));
    }

    #[test]
    fn test_sparse_basis_state() {
        let state = SparseQuantumState::computational_basis_state(4, 5).unwrap();
        assert_eq!(state.num_qubits(), 4);
        assert_eq!(state.num_nonzero(), 1);
        assert_eq!(state.amplitude(5), Complex64::new(1.0, 0.0));
        assert_eq!(state.sparsity(), 1.0 / 16.0);
    }

    #[test]
    fn test_sparse_from_dense() {
        let dense = QuantumState::zero_state(2);
        let sparse = SparseQuantumState::from_dense(&dense, 1e-10);

        assert_eq!(sparse.num_qubits(), 2);
        assert_eq!(sparse.num_nonzero(), 1);
        assert_eq!(sparse.amplitude(0), Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_sparse_to_dense() {
        let sparse = SparseQuantumState::computational_basis_state(2, 3).unwrap();
        let dense = sparse.to_dense();

        assert_eq!(dense.num_qubits(), 2);
        assert_eq!(dense.amplitude(3).unwrap(), Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_sparse_memory_efficiency() {
        let sparse = SparseQuantumState::zero_state(20);
        let dense_memory = (1 << 20) * std::mem::size_of::<Complex64>();
        let sparse_memory = sparse.memory_usage();

        // Sparse should use much less memory
        assert!(sparse_memory < dense_memory / 1000);
    }
}

// ============================================================================
// State Vector Compression (Phase 4.2, Task 4.2.3)
// ============================================================================

/// State vector compression utilities
///
/// Provides methods to compress quantum states by:
/// - Amplitude truncation: Remove small amplitudes below threshold
/// - Adaptive compression: Maintain fidelity while reducing memory
/// - Lossless conversion: Convert to sparse representation
pub struct StateCompression;

impl StateCompression {
    /// Compress state by truncating small amplitudes
    ///
    /// # Arguments
    /// * `state` - The quantum state to compress
    /// * `threshold` - Amplitudes with |α|² < threshold are set to zero
    ///
    /// # Returns
    /// Compressed sparse state and compression statistics
    pub fn truncate_amplitudes(
        state: &QuantumState,
        threshold: f64,
    ) -> Result<(SparseQuantumState, CompressionStats)> {
        let start_time = std::time::Instant::now();
        let original_size = state.dim();

        // Convert to sparse representation
        let mut sparse = SparseQuantumState::from_dense(state, threshold);

        // Renormalize after truncation
        sparse.normalize();

        let compressed_size = sparse.num_nonzero();
        let compression_ratio = compressed_size as f64 / original_size as f64;

        // Calculate fidelity with original state
        let fidelity = Self::calculate_fidelity(state, &sparse.to_dense());

        let stats = CompressionStats {
            original_size,
            compressed_size,
            compression_ratio,
            fidelity,
            threshold,
            compression_time: start_time.elapsed(),
        };

        Ok((sparse, stats))
    }

    /// Adaptive compression to achieve target fidelity
    ///
    /// Automatically finds the optimal threshold to achieve the desired fidelity
    /// while maximizing compression.
    pub fn adaptive_compress(
        state: &QuantumState,
        target_fidelity: f64,
        max_iterations: usize,
    ) -> Result<(SparseQuantumState, CompressionStats)> {
        if !(0.0..=1.0).contains(&target_fidelity) {
            return Err(MyQuatError::quantum_info_error(
                "Target fidelity must be between 0 and 1",
            ));
        }

        // Binary search for optimal threshold
        let mut low = 0.0;
        let mut high = 1.0;
        let mut best_result = None;

        for _ in 0..max_iterations {
            let threshold = (low + high) / 2.0;
            let (sparse, stats) = Self::truncate_amplitudes(state, threshold)?;

            if stats.fidelity >= target_fidelity {
                // Can compress more
                best_result = Some((sparse, stats));
                low = threshold;
            } else {
                // Compressed too much
                high = threshold;
            }

            if (high - low) < 1e-10 {
                break;
            }
        }

        best_result
            .ok_or_else(|| MyQuatError::quantum_info_error("Failed to find compression threshold"))
    }

    /// Calculate fidelity between two states
    fn calculate_fidelity(state1: &QuantumState, state2: &QuantumState) -> f64 {
        if state1.num_qubits() != state2.num_qubits() {
            return 0.0;
        }

        let overlap: Complex64 = state1
            .amplitudes()
            .iter()
            .zip(state2.amplitudes().iter())
            .map(|(a, b)| a.conj() * b)
            .sum();

        overlap.norm_sqr()
    }

    /// Lossless compression to sparse format
    ///
    /// Converts dense state to sparse without any approximation.
    /// Only removes exact zeros.
    pub fn lossless_compress(state: &QuantumState) -> (SparseQuantumState, CompressionStats) {
        let start_time = std::time::Instant::now();
        let original_size = state.dim();

        let sparse = SparseQuantumState::from_dense(state, 0.0);
        let compressed_size = sparse.num_nonzero();

        let stats = CompressionStats {
            original_size,
            compressed_size,
            compression_ratio: compressed_size as f64 / original_size as f64,
            fidelity: 1.0,
            threshold: 0.0,
            compression_time: start_time.elapsed(),
        };

        (sparse, stats)
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    /// Original state size (number of amplitudes)
    pub original_size: usize,
    /// Compressed state size (number of non-zero amplitudes)
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub compression_ratio: f64,
    /// Fidelity with original state
    pub fidelity: f64,
    /// Threshold used for compression
    pub threshold: f64,
    /// Time taken for compression
    pub compression_time: std::time::Duration,
}

impl CompressionStats {
    /// Print compression statistics
    pub fn print(&self) {
        println!("Compression Statistics:");
        println!("  Original size: {} amplitudes", self.original_size);
        println!("  Compressed size: {} amplitudes", self.compressed_size);
        println!(
            "  Compression ratio: {:.2}%",
            self.compression_ratio * 100.0
        );
        println!("  Fidelity: {:.6}", self.fidelity);
        println!("  Threshold: {:.2e}", self.threshold);
        println!("  Compression time: {:?}", self.compression_time);
        println!(
            "  Memory saved: {:.2}%",
            (1.0 - self.compression_ratio) * 100.0
        );
    }
}

#[cfg(test)]
mod compression_tests {
    use super::*;

    #[test]
    fn test_lossless_compression() {
        let state = QuantumState::zero_state(3);
        let (sparse, stats) = StateCompression::lossless_compress(&state);

        assert_eq!(stats.fidelity, 1.0);
        assert_eq!(sparse.num_nonzero(), 1);
        assert!(stats.compression_ratio < 0.2);
    }

    #[test]
    fn test_truncate_amplitudes() {
        let state = QuantumState::plus_state(3);
        let result = StateCompression::truncate_amplitudes(&state, 0.01);

        assert!(result.is_ok());
        let (sparse, stats) = result.unwrap();
        assert!(stats.fidelity > 0.9);
        assert!(sparse.num_nonzero() <= state.dim());
    }

    #[test]
    fn test_adaptive_compression() {
        // Use a state with varying amplitudes for better compression
        let mut amplitudes = Array1::zeros(16);
        amplitudes[0] = Complex64::new(0.9, 0.0);
        amplitudes[1] = Complex64::new(0.3, 0.0);
        amplitudes[2] = Complex64::new(0.2, 0.0);
        amplitudes[3] = Complex64::new(0.1, 0.0);
        let state = QuantumState::new_normalized(amplitudes).unwrap();

        let result = StateCompression::adaptive_compress(&state, 0.95, 20);

        assert!(result.is_ok());
        let (sparse, stats) = result.unwrap();
        assert!(stats.fidelity >= 0.95);
        assert!(sparse.num_nonzero() <= state.dim());
    }

    #[test]
    fn test_compression_fidelity() {
        let state1 = QuantumState::zero_state(2);
        let state2 = QuantumState::zero_state(2);

        let fidelity = StateCompression::calculate_fidelity(&state1, &state2);
        assert!((fidelity - 1.0).abs() < 1e-10);
    }
}
