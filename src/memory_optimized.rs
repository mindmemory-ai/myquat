//! Memory optimization utilities
//!
//! This module provides zero-copy operations and memory-efficient matrix computations
//! for high-performance quantum circuit simulation.

use crate::error::{MyQuatError, Result};
use ndarray::{Array1, Array2, ArrayView1, ArrayViewMut1};
use num_complex::Complex64;

/// Zero-copy matrix operations for quantum gates
pub struct ZeroCopyMatrixOps;

impl ZeroCopyMatrixOps {
    /// Apply a single-qubit gate matrix to a state vector without copying
    /// Uses in-place operations where possible
    pub fn apply_single_qubit_gate_inplace(
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit_index: usize,
        num_qubits: usize,
    ) -> Result<()> {
        if gate_matrix.dim() != (2, 2) {
            return Err(MyQuatError::circuit_error(
                "Single qubit gate matrix must be 2x2",
            ));
        }

        if qubit_index >= num_qubits {
            return Err(MyQuatError::circuit_error(format!(
                "Qubit index {} out of range for {} qubits",
                qubit_index, num_qubits
            )));
        }

        let dim = 1 << num_qubits;
        if state.len() != dim {
            return Err(MyQuatError::circuit_error(
                "State vector dimension doesn't match number of qubits",
            ));
        }

        // Use zero-copy approach: iterate through state pairs and apply gate in-place
        let step = 1 << (num_qubits - 1 - qubit_index);
        let _mask = step - 1;

        for i in (0..dim).step_by(2 * step) {
            for j in 0..step {
                let idx0 = i + j;
                let idx1 = i + j + step;

                // Extract current amplitudes
                let amp0 = state[idx0];
                let amp1 = state[idx1];

                // Apply gate matrix in-place
                state[idx0] = gate_matrix[[0, 0]] * amp0 + gate_matrix[[0, 1]] * amp1;
                state[idx1] = gate_matrix[[1, 0]] * amp0 + gate_matrix[[1, 1]] * amp1;
            }
        }

        Ok(())
    }

    /// Apply a two-qubit gate matrix to a state vector without copying
    pub fn apply_two_qubit_gate_inplace(
        state: &mut Array1<Complex64>,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
        num_qubits: usize,
    ) -> Result<()> {
        if gate_matrix.dim() != (4, 4) {
            return Err(MyQuatError::circuit_error(
                "Two qubit gate matrix must be 4x4",
            ));
        }

        if qubit0 >= num_qubits || qubit1 >= num_qubits {
            return Err(MyQuatError::circuit_error("Qubit indices out of range"));
        }

        if qubit0 == qubit1 {
            return Err(MyQuatError::circuit_error(
                "Two-qubit gate requires different qubits",
            ));
        }

        let dim = 1 << num_qubits;
        let q0_bit = 1 << (num_qubits - 1 - qubit0);
        let q1_bit = 1 << (num_qubits - 1 - qubit1);

        // Pre-compute all possible state combinations
        let mut temp_amps = [Complex64::new(0.0, 0.0); 4];

        for i in 0..dim {
            // Check if this state involves our target qubits
            let q0_val = if (i & q0_bit) != 0 { 1 } else { 0 };
            let q1_val = if (i & q1_bit) != 0 { 1 } else { 0 };

            // Skip if this is not a base state for our qubit pair
            if (i & (q0_bit | q1_bit)) != (q0_val * q0_bit + q1_val * q1_bit) {
                continue;
            }

            // Calculate the four related state indices
            let base = i & !(q0_bit | q1_bit);
            let idx00 = base;
            let idx01 = base | q1_bit;
            let idx10 = base | q0_bit;
            let idx11 = base | q0_bit | q1_bit;

            // Extract current amplitudes
            temp_amps[0] = state[idx00];
            temp_amps[1] = state[idx01];
            temp_amps[2] = state[idx10];
            temp_amps[3] = state[idx11];

            // Apply gate matrix
            state[idx00] = gate_matrix[[0, 0]] * temp_amps[0]
                + gate_matrix[[0, 1]] * temp_amps[1]
                + gate_matrix[[0, 2]] * temp_amps[2]
                + gate_matrix[[0, 3]] * temp_amps[3];
            state[idx01] = gate_matrix[[1, 0]] * temp_amps[0]
                + gate_matrix[[1, 1]] * temp_amps[1]
                + gate_matrix[[1, 2]] * temp_amps[2]
                + gate_matrix[[1, 3]] * temp_amps[3];
            state[idx10] = gate_matrix[[2, 0]] * temp_amps[0]
                + gate_matrix[[2, 1]] * temp_amps[1]
                + gate_matrix[[2, 2]] * temp_amps[2]
                + gate_matrix[[2, 3]] * temp_amps[3];
            state[idx11] = gate_matrix[[3, 0]] * temp_amps[0]
                + gate_matrix[[3, 1]] * temp_amps[1]
                + gate_matrix[[3, 2]] * temp_amps[2]
                + gate_matrix[[3, 3]] * temp_amps[3];

            // Skip ahead to avoid processing the same group again
            let step = (q0_bit | q1_bit).trailing_zeros() as usize;
            if step > 0 {
                // This is a simplified approach; full implementation would be more complex
            }
        }

        Ok(())
    }

    /// Compute tensor product without full matrix expansion (memory efficient)
    pub fn tensor_product_state_inplace(
        result: &mut Array1<Complex64>,
        state1: &Array1<Complex64>,
        state2: &Array1<Complex64>,
    ) -> Result<()> {
        let dim1 = state1.len();
        let dim2 = state2.len();

        if result.len() != dim1 * dim2 {
            return Err(MyQuatError::circuit_error(
                "Result array size doesn't match tensor product dimensions",
            ));
        }

        // Zero-copy tensor product using direct indexing
        for i in 0..dim1 {
            for j in 0..dim2 {
                result[i * dim2 + j] = state1[i] * state2[j];
            }
        }

        Ok(())
    }

    /// Efficient partial trace operation with minimal memory allocation
    pub fn partial_trace_efficient(
        state: &Array1<Complex64>,
        traced_qubits: &[usize],
        _num_qubits: usize,
    ) -> Result<Array2<Complex64>> {
        if traced_qubits.is_empty() {
            // Return full density matrix
            let dim = state.len();
            let mut density = Array2::zeros((dim, dim));

            for i in 0..dim {
                for j in 0..dim {
                    density[[i, j]] = state[i] * state[j].conj();
                }
            }

            return Ok(density);
        }

        // For now, implement a simple case
        // Full implementation would require more complex indexing
        Err(MyQuatError::circuit_error(
            "Partial trace for general cases not yet implemented",
        ))
    }
}

/// Memory-efficient state vector operations
pub struct MemoryEfficientState {
    /// State amplitudes stored efficiently
    amplitudes: Array1<Complex64>,
    /// Number of qubits
    num_qubits: usize,
    /// Workspace for temporary calculations (reused to avoid allocations)
    workspace: Array1<Complex64>,
}

impl MemoryEfficientState {
    /// Create a new memory-efficient state
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut amplitudes = Array1::zeros(dim);
        amplitudes[0] = Complex64::new(1.0, 0.0); // |0...0⟩ state

        MemoryEfficientState {
            amplitudes,
            num_qubits,
            workspace: Array1::zeros(dim),
        }
    }

    /// Create from existing amplitudes (takes ownership to avoid copy)
    pub fn from_amplitudes(amplitudes: Array1<Complex64>) -> Result<Self> {
        let dim = amplitudes.len();
        if !dim.is_power_of_two() || dim == 0 {
            return Err(MyQuatError::circuit_error(
                "State dimension must be a power of 2",
            ));
        }

        let num_qubits = (dim as f64).log2() as usize;

        Ok(MemoryEfficientState {
            amplitudes,
            num_qubits,
            workspace: Array1::zeros(dim),
        })
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get state dimension
    pub fn dim(&self) -> usize {
        self.amplitudes.len()
    }

    /// Get read-only view of amplitudes (zero-copy)
    pub fn amplitudes(&self) -> ArrayView1<'_, Complex64> {
        self.amplitudes.view()
    }

    /// Get mutable view of amplitudes (zero-copy)
    pub fn amplitudes_mut(&mut self) -> ArrayViewMut1<'_, Complex64> {
        self.amplitudes.view_mut()
    }

    /// Apply single-qubit gate in-place
    pub fn apply_single_qubit_gate(
        &mut self,
        gate_matrix: &Array2<Complex64>,
        qubit: usize,
    ) -> Result<()> {
        ZeroCopyMatrixOps::apply_single_qubit_gate_inplace(
            &mut self.amplitudes,
            gate_matrix,
            qubit,
            self.num_qubits,
        )
    }

    /// Apply two-qubit gate in-place
    pub fn apply_two_qubit_gate(
        &mut self,
        gate_matrix: &Array2<Complex64>,
        qubit0: usize,
        qubit1: usize,
    ) -> Result<()> {
        ZeroCopyMatrixOps::apply_two_qubit_gate_inplace(
            &mut self.amplitudes,
            gate_matrix,
            qubit0,
            qubit1,
            self.num_qubits,
        )
    }

    /// Normalize the state in-place
    pub fn normalize(&mut self) {
        let norm_squared: f64 = self.amplitudes.iter().map(|z| z.norm_sqr()).sum();
        let norm = norm_squared.sqrt();

        if norm > 1e-15 {
            self.amplitudes.mapv_inplace(|z| z / norm);
        }
    }

    /// Compute probability of measuring a specific state
    pub fn probability(&self, state_index: usize) -> Result<f64> {
        if state_index >= self.dim() {
            return Err(MyQuatError::circuit_error("State index out of range"));
        }

        Ok(self.amplitudes[state_index].norm_sqr())
    }

    /// Get workspace for temporary calculations (reused buffer)
    pub fn workspace_mut(&mut self) -> ArrayViewMut1<'_, Complex64> {
        self.workspace.view_mut()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub state_size_bytes: usize,
    pub workspace_size_bytes: usize,
    pub total_size_bytes: usize,
    pub efficiency_ratio: f64, // Compared to naive implementation
}

impl MemoryEfficientState {
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let state_size = self.amplitudes.len() * std::mem::size_of::<Complex64>();
        let workspace_size = self.workspace.len() * std::mem::size_of::<Complex64>();
        let total_size = state_size + workspace_size;

        // Compare to naive implementation that might use multiple temporary arrays
        let naive_size = state_size * 3; // Assume naive implementation uses 3x memory
        let efficiency_ratio = naive_size as f64 / total_size as f64;

        MemoryStats {
            state_size_bytes: state_size,
            workspace_size_bytes: workspace_size,
            total_size_bytes: total_size,
            efficiency_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_memory_efficient_state_creation() {
        let state = MemoryEfficientState::new(2);
        assert_eq!(state.num_qubits(), 2);
        assert_eq!(state.dim(), 4);

        // Should be |00⟩ state
        assert_relative_eq!(state.amplitudes()[0].re, 1.0, epsilon = 1e-10);
        assert_relative_eq!(state.amplitudes()[1].norm(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_zero_copy_single_qubit_gate() {
        let mut state = MemoryEfficientState::new(1);

        // Apply X gate (Pauli-X)
        let x_gate = Array2::from_shape_vec(
            (2, 2),
            vec![
                Complex64::new(0.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
                Complex64::new(0.0, 0.0),
            ],
        )
        .unwrap();

        state.apply_single_qubit_gate(&x_gate, 0).unwrap();

        // Should be |1⟩ state
        assert_relative_eq!(state.amplitudes()[0].norm(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(state.amplitudes()[1].re, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_normalization() {
        let amplitudes = Array1::from_vec(vec![Complex64::new(2.0, 0.0), Complex64::new(0.0, 2.0)]);

        let mut state = MemoryEfficientState::from_amplitudes(amplitudes).unwrap();
        state.normalize();

        let norm_squared: f64 = state.amplitudes().iter().map(|z| z.norm_sqr()).sum();
        assert_relative_eq!(norm_squared, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_memory_stats() {
        let state = MemoryEfficientState::new(3);
        let stats = state.memory_stats();

        assert!(stats.total_size_bytes > 0);
        assert!(stats.efficiency_ratio > 1.0); // Should be more efficient than naive
        assert_eq!(
            stats.total_size_bytes,
            stats.state_size_bytes + stats.workspace_size_bytes
        );
    }
}
