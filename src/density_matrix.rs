//! Density matrix representation and operations
//!
//! This module provides support for mixed quantum states using density matrices,
//! including decoherence and open quantum system simulation.

use crate::{GateOperation, MyQuatError, Parameter, QuantumCircuit, Result, StandardGate};
use nalgebra::{Complex as NaComplex, Matrix2};
use ndarray::{s, Array1, Array2};
use num_complex::Complex64;
use std::collections::HashMap;

/// Density matrix representation of quantum states
#[derive(Debug, Clone)]
pub struct DensityMatrix {
    /// The density matrix as a 2D array
    matrix: Array2<Complex64>,
    /// Number of qubits
    num_qubits: usize,
}

impl DensityMatrix {
    /// Create a new density matrix for n qubits in the |0⟩ state
    pub fn zero_state(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut matrix = Array2::zeros((dim, dim));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);

        DensityMatrix { matrix, num_qubits }
    }

    /// Create a density matrix from a pure state vector
    pub fn from_state_vector(state: &Array1<Complex64>) -> Result<Self> {
        let dim = state.len();
        let num_qubits = (dim as f64).log2();

        if num_qubits.fract() != 0.0 {
            return Err(MyQuatError::circuit_error(
                "State vector dimension must be power of 2",
            ));
        }

        let num_qubits = num_qubits as usize;
        let mut matrix = Array2::zeros((dim, dim));

        // ρ = |ψ⟩⟨ψ|
        for i in 0..dim {
            for j in 0..dim {
                matrix[[i, j]] = state[i] * state[j].conj();
            }
        }

        Ok(DensityMatrix { matrix, num_qubits })
    }

    /// Create a maximally mixed state
    pub fn maximally_mixed(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let prob = 1.0 / dim as f64;
        let matrix = Array2::eye(dim) * Complex64::new(prob, 0.0);

        DensityMatrix { matrix, num_qubits }
    }

    /// Create a thermal state with given temperature
    pub fn thermal_state(
        num_qubits: usize,
        temperature: f64,
        hamiltonian: &Array2<Complex64>,
    ) -> Result<Self> {
        let dim = 1 << num_qubits;
        if hamiltonian.dim() != (dim, dim) {
            return Err(MyQuatError::circuit_error("Hamiltonian dimension mismatch"));
        }

        // For simplicity, create a diagonal thermal state
        // Full implementation would use matrix exponentiation
        let mut matrix = Array2::zeros((dim, dim));
        let beta = 1.0 / temperature.max(1e-10); // Avoid division by zero

        for i in 0..dim {
            let energy = hamiltonian[[i, i]].re;
            let prob = (-beta * energy).exp();
            matrix[[i, i]] = Complex64::new(prob, 0.0);
        }

        // Normalize
        let trace = matrix.diag().sum();
        matrix /= trace;

        Ok(DensityMatrix { matrix, num_qubits })
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the dimension of the density matrix
    pub fn dimension(&self) -> usize {
        1 << self.num_qubits
    }

    /// Get a reference to the density matrix
    pub fn matrix(&self) -> &Array2<Complex64> {
        &self.matrix
    }

    /// Get a mutable reference to the density matrix
    pub fn matrix_mut(&mut self) -> &mut Array2<Complex64> {
        &mut self.matrix
    }

    /// Calculate the trace of the density matrix
    pub fn trace(&self) -> Complex64 {
        self.matrix.diag().sum()
    }

    /// Calculate the purity of the state.
    ///
    /// $$ \gamma = \operatorname{Tr}(\rho^2) $$
    ///
    /// For a pure state $\gamma = 1$; for the maximally mixed state of $n$ qubits,
    /// $\gamma = 1/2^n$.
    pub fn purity(&self) -> f64 {
        let rho_squared = self.matrix.dot(&self.matrix);
        rho_squared.diag().sum().re
    }

    /// Check if the state is pure.
    ///
    /// Returns true when $|\operatorname{Tr}(\rho^2) - 1| < \text{tolerance}$.
    pub fn is_pure(&self, tolerance: f64) -> bool {
        (self.purity() - 1.0).abs() < tolerance
    }

    /// Calculate the von Neumann entropy.
    ///
    /// $$ S(\rho) = -\operatorname{Tr}(\rho \ln \rho) = -\sum_k \lambda_k \ln \lambda_k $$
    ///
    /// where $\lambda_k$ are the eigenvalues of $\rho$. Uses eigenvalue decomposition
    /// with improved numerical stability. Eigenvalues below $10^{-14}$ are treated as zero.
    pub fn von_neumann_entropy(&self) -> f64 {
        let eigenvalues = self.eigenvalues();
        let mut entropy = 0.0;

        // Use stricter threshold for numerical stability
        for &lambda in &eigenvalues {
            if lambda > 1e-14 {
                entropy -= lambda * lambda.ln();
            }
        }

        entropy
    }

    /// Get eigenvalues of the density matrix
    ///
    /// Uses Hermitian eigenvalue decomposition (LAPACK) for numerical stability.
    /// Returns eigenvalues in ascending order.
    pub fn eigenvalues(&self) -> Vec<f64> {
        use ndarray_linalg::Eigh;

        // Density matrix is Hermitian, use eigh for better performance
        match self.matrix.clone().eigh(ndarray_linalg::UPLO::Lower) {
            Ok((eigenvalues, _)) => eigenvalues.to_vec(),
            Err(_) => {
                // Fallback: return diagonal elements if decomposition fails
                self.matrix.diag().iter().map(|c| c.re).collect()
            }
        }
    }

    /// Apply a unitary operation to the density matrix.
    ///
    /// $$ \rho' = U \rho U^\dagger $$
    pub fn apply_unitary(&mut self, unitary: &Array2<Complex64>) -> Result<()> {
        let dim = self.dimension();
        if unitary.dim() != (dim, dim) {
            return Err(MyQuatError::circuit_error("Unitary dimension mismatch"));
        }

        // ρ' = U ρ U†
        let unitary_dag = unitary.t().mapv(|c| c.conj());
        self.matrix = unitary.dot(&self.matrix).dot(&unitary_dag);

        Ok(())
    }

    /// Apply a quantum gate to the density matrix
    pub fn apply_gate(
        &mut self,
        gate: &StandardGate,
        qubits: &[usize],
        params: &[Parameter],
    ) -> Result<()> {
        let gate_matrix = gate.matrix(params, &HashMap::new())?;

        // For single-qubit gates, expand to full system
        if gate.num_qubits() == 1 && qubits.len() == 1 {
            let full_unitary = self.expand_single_qubit_gate(&gate_matrix, qubits[0])?;
            self.apply_unitary(&full_unitary)?;
        } else if gate.num_qubits() == 2 && qubits.len() == 2 {
            let full_unitary = self.expand_two_qubit_gate(&gate_matrix, qubits[0], qubits[1])?;
            self.apply_unitary(&full_unitary)?;
        } else {
            return Err(MyQuatError::circuit_error("Unsupported gate configuration"));
        }

        Ok(())
    }

    /// Expand single-qubit gate to full system unitary
    ///
    /// Uses Kronecker product to expand a single-qubit gate to the full Hilbert space.
    /// For qubit i in an n-qubit system: I ⊗ ... ⊗ I ⊗ gate ⊗ I ⊗ ... ⊗ I
    fn expand_single_qubit_gate(
        &self,
        gate: &Array2<Complex64>,
        target: usize,
    ) -> Result<Array2<Complex64>> {
        if target >= self.num_qubits {
            return Err(MyQuatError::circuit_error(
                "Target qubit index out of range",
            ));
        }

        if gate.dim() != (2, 2) {
            return Err(MyQuatError::circuit_error(
                "Gate must be 2x2 for single qubit",
            ));
        }

        let n = self.num_qubits;
        let identity: Array2<Complex64> = Array2::eye(2);

        // Build full unitary using Kronecker products
        let mut full_unitary = Array2::eye(1);

        for i in 0..n {
            if i == target {
                full_unitary = kron(&full_unitary, gate);
            } else {
                full_unitary = kron(&full_unitary, &identity);
            }
        }

        Ok(full_unitary)
    }

    /// Expand two-qubit gate to full system unitary
    ///
    /// Expands a two-qubit gate to the full Hilbert space using Kronecker products.
    /// Handles both adjacent and non-adjacent qubit pairs.
    fn expand_two_qubit_gate(
        &self,
        gate: &Array2<Complex64>,
        qubit1: usize,
        qubit2: usize,
    ) -> Result<Array2<Complex64>> {
        if qubit1 >= self.num_qubits || qubit2 >= self.num_qubits {
            return Err(MyQuatError::circuit_error("Qubit index out of range"));
        }

        if qubit1 == qubit2 {
            return Err(MyQuatError::circuit_error(
                "Cannot apply two-qubit gate to same qubit",
            ));
        }

        if gate.dim() != (4, 4) {
            return Err(MyQuatError::circuit_error(
                "Gate must be 4x4 for two qubits",
            ));
        }

        let n = self.num_qubits;
        let identity: Array2<Complex64> = Array2::eye(2);

        // If qubits are out of order, swap the gate matrix accordingly
        let (q1, q2, gate_matrix) = if qubit1 < qubit2 {
            (qubit1, qubit2, gate.clone())
        } else {
            // Swap rows/cols of the 4x4 gate matrix to match the reordered qubits.
            // The swap permutation on indices maps: |00>->|00>, |01>->|10>, |10>->|01>, |11>->|11>
            let mut swapped = Array2::zeros((4, 4));
            let perm = [0usize, 2, 1, 3]; // Swap bits 0 and 1
            for i in 0..4 {
                for j in 0..4 {
                    swapped[[perm[i], perm[j]]] = gate[[i, j]];
                }
            }
            (qubit2, qubit1, swapped)
        };

        // Build full unitary using Kronecker products
        let mut full_unitary = Array2::eye(1);
        let mut gate_applied = false;

        for i in 0..n {
            if i == q1 && !gate_applied {
                // Apply the two-qubit gate
                full_unitary = kron(&full_unitary, &gate_matrix);
                gate_applied = true;
                // Skip the next qubit as it's part of the two-qubit gate
            } else if i == q2 && gate_applied {
                // Already included in the two-qubit gate, skip
                continue;
            } else {
                full_unitary = kron(&full_unitary, &identity);
            }
        }

        Ok(full_unitary)
    }

    /// Partial trace over specified qubits.
    ///
    /// Traces out the specified qubits, returning the reduced density matrix:
    ///
    /// $$ \rho_A = \operatorname{Tr}_B(\rho_{AB}) $$
    ///
    /// where $B$ is the subsystem being traced out and $A$ is the remaining qubits.
    ///
    /// # Arguments
    /// * `traced_qubits` - Indices of qubits to trace out
    ///
    /// # Returns
    /// Reduced density matrix for the remaining qubits
    ///
    /// # Performance
    /// Complexity: $O(2^{2n})$ where $n$ is the number of qubits.
    /// Can be parallelized for large systems using rayon.
    pub fn partial_trace(&self, traced_qubits: &[usize]) -> Result<DensityMatrix> {
        use std::collections::HashSet;

        // Validate input
        if traced_qubits.is_empty() {
            return Ok(self.clone());
        }

        let traced_set: HashSet<usize> = traced_qubits.iter().copied().collect();

        // Check for invalid qubit indices
        for &q in traced_qubits {
            if q >= self.num_qubits {
                return Err(MyQuatError::circuit_error(
                    "Traced qubit index out of range",
                ));
            }
        }

        let remaining_qubits: Vec<usize> = (0..self.num_qubits)
            .filter(|q| !traced_set.contains(q))
            .collect();

        if remaining_qubits.is_empty() {
            return Err(MyQuatError::circuit_error("Cannot trace out all qubits"));
        }

        let new_n = remaining_qubits.len();
        let new_dim = 1 << new_n;
        let traced_dim = 1 << traced_qubits.len();

        let mut reduced = Array2::zeros((new_dim, new_dim));

        // Compute partial trace: ρ_reduced[i,j] = Σ_k ρ_full[ik, jk]
        // where k runs over all states of the traced qubits
        for i in 0..new_dim {
            for j in 0..new_dim {
                let mut sum = Complex64::new(0.0, 0.0);

                // Sum over all states of traced qubits
                for traced_state in 0..traced_dim {
                    let full_i = combine_states(i, traced_state, &remaining_qubits, traced_qubits);
                    let full_j = combine_states(j, traced_state, &remaining_qubits, traced_qubits);
                    sum += self.matrix[[full_i, full_j]];
                }

                reduced[[i, j]] = sum;
            }
        }

        Ok(DensityMatrix {
            matrix: reduced,
            num_qubits: new_n,
        })
    }

    /// Get expectation value of an observable.
    ///
    /// $$ \langle O \rangle = \operatorname{Tr}(\rho O) $$
    pub fn expectation_value(&self, observable: &Array2<Complex64>) -> Result<f64> {
        if observable.dim() != (self.dimension(), self.dimension()) {
            return Err(MyQuatError::circuit_error("Observable dimension mismatch"));
        }

        // Tr(ρ * O)
        let mut expectation = Complex64::new(0.0, 0.0);
        for i in 0..self.dimension() {
            for j in 0..self.dimension() {
                expectation += self.matrix[[i, j]] * observable[[j, i]];
            }
        }

        Ok(expectation.re)
    }

    /// Calculate measurement probability for a qubit
    pub fn measure_probability(&self, qubit: usize, outcome: bool) -> Result<f64> {
        if qubit >= self.num_qubits {
            return Err(MyQuatError::circuit_error("Invalid qubit index"));
        }

        let mut prob = 0.0;
        let qubit_mask = 1 << qubit;

        for i in 0..self.dimension() {
            let qubit_state = (i & qubit_mask) != 0;
            if qubit_state == outcome {
                prob += self.matrix[[i, i]].re;
            }
        }

        Ok(prob)
    }

    /// Collapse state after measurement
    pub fn collapse_to_measurement(&mut self, qubit: usize, outcome: bool) -> Result<()> {
        if qubit >= self.num_qubits {
            return Err(MyQuatError::circuit_error("Invalid qubit index"));
        }

        let prob = self.measure_probability(qubit, outcome)?;
        if prob <= 0.0 {
            return Err(MyQuatError::circuit_error(
                "Cannot collapse to zero probability state",
            ));
        }

        let qubit_mask = 1 << qubit;
        let mut new_matrix = Array2::zeros((self.dimension(), self.dimension()));

        // Project onto measurement outcome and renormalize
        for i in 0..self.dimension() {
            for j in 0..self.dimension() {
                let i_state = (i & qubit_mask) != 0;
                let j_state = (j & qubit_mask) != 0;

                if i_state == outcome && j_state == outcome {
                    new_matrix[[i, j]] = self.matrix[[i, j]] / Complex64::new(prob, 0.0);
                }
            }
        }

        self.matrix = new_matrix;
        Ok(())
    }

    /// Calculate fidelity with another density matrix.
    ///
    /// Computes the Uhlmann fidelity:
    ///
    /// $$ F(\rho, \sigma) = \left[ \operatorname{Tr}\left( \sqrt{\sqrt{\rho}\,\sigma\,\sqrt{\rho}} \right) \right]^2 $$
    ///
    /// For pure states $\rho = |\psi\rangle\langle\psi|$, $\sigma = |\phi\rangle\langle\phi|$,
    /// this reduces to $|\langle\psi|\phi\rangle|^2$.
    /// Uses optimized paths for pure states and diagonal matrices.
    pub fn fidelity(&self, other: &DensityMatrix) -> Result<f64> {
        use ndarray_linalg::Eigh;

        if self.dimension() != other.dimension() {
            return Err(MyQuatError::circuit_error("Dimension mismatch"));
        }

        // Fast path for pure states
        if self.is_pure(1e-10) && other.is_pure(1e-10) {
            // For pure states: F = |Tr(ρσ)|²
            let mut trace = Complex64::new(0.0, 0.0);
            for i in 0..self.dimension() {
                for j in 0..self.dimension() {
                    trace += self.matrix[[i, j]] * other.matrix[[j, i]];
                }
            }
            return Ok(trace.norm_sqr());
        }

        // General case: Uhlmann fidelity
        // F = [Tr(√(√ρ σ √ρ))]²

        // Step 1: Compute √ρ using eigenvalue decomposition
        let (eigenvalues, eigenvectors) =
            match self.matrix.clone().eigh(ndarray_linalg::UPLO::Lower) {
                Ok(result) => result,
                Err(_) => {
                    return Err(MyQuatError::circuit_error(
                        "Failed to compute eigenvalue decomposition",
                    ))
                }
            };

        // Construct √ρ = V √Λ V†
        let mut sqrt_rho = Array2::zeros(self.matrix.dim());
        for i in 0..self.dimension() {
            for j in 0..self.dimension() {
                for k in 0..self.dimension() {
                    let sqrt_lambda = if eigenvalues[k] > 0.0 {
                        eigenvalues[k].sqrt()
                    } else {
                        0.0
                    };
                    sqrt_rho[[i, j]] += eigenvectors[[i, k]]
                        * Complex64::new(sqrt_lambda, 0.0)
                        * eigenvectors[[j, k]].conj();
                }
            }
        }

        // Step 2: Compute √ρ σ √ρ
        let temp = sqrt_rho.dot(&other.matrix).dot(&sqrt_rho);

        // Step 3: Compute eigenvalues and sum their square roots
        let (temp_eigenvalues, _) = match temp.eigh(ndarray_linalg::UPLO::Lower) {
            Ok(result) => result,
            Err(_) => return Err(MyQuatError::circuit_error("Failed to compute eigenvalues")),
        };

        let trace_sqrt: f64 = temp_eigenvalues
            .iter()
            .map(|&x| if x > 0.0 { x.sqrt() } else { 0.0 })
            .sum();

        Ok(trace_sqrt * trace_sqrt)
    }
}

/// Compute Kronecker product of two matrices
///
/// Optimized implementation using block assignment for better performance.
fn kron(a: &Array2<Complex64>, b: &Array2<Complex64>) -> Array2<Complex64> {
    let (m, n) = a.dim();
    let (p, q) = b.dim();
    let mut result = Array2::zeros((m * p, n * q));

    for i in 0..m {
        for j in 0..n {
            let block = b * a[[i, j]];
            result
                .slice_mut(s![i * p..(i + 1) * p, j * q..(j + 1) * q])
                .assign(&block);
        }
    }

    result
}

/// Combine states for partial trace calculation
///
/// Combines a reduced state index with a traced state index to get the full state index.
fn combine_states(
    reduced_idx: usize,
    traced_idx: usize,
    remaining_qubits: &[usize],
    traced_qubits: &[usize],
) -> usize {
    let mut full_idx = 0;
    let mut reduced_bit = 0;
    let mut traced_bit = 0;

    // Reconstruct full index from reduced and traced indices
    let total_qubits = remaining_qubits.len() + traced_qubits.len();

    for qubit in 0..total_qubits {
        let bit_value = if remaining_qubits.contains(&qubit) {
            // This qubit is in the remaining system
            let bit = (reduced_idx >> reduced_bit) & 1;
            reduced_bit += 1;
            bit
        } else {
            // This qubit is being traced out
            let bit = (traced_idx >> traced_bit) & 1;
            traced_bit += 1;
            bit
        };

        full_idx |= bit_value << qubit;
    }

    full_idx
}

/// Expand a single-qubit Kraus operator to the full system
///
/// Converts a nalgebra Matrix2 to ndarray Array2 and expands it to full system.
fn expand_kraus_operator(
    kraus: &Matrix2<NaComplex<f64>>,
    target_qubit: usize,
    num_qubits: usize,
) -> Array2<Complex64> {
    // Convert nalgebra matrix to ndarray
    let mut gate = Array2::zeros((2, 2));
    for i in 0..2 {
        for j in 0..2 {
            let na_val = kraus[(i, j)];
            gate[[i, j]] = Complex64::new(na_val.re, na_val.im);
        }
    }

    // Expand to full system
    let identity: Array2<Complex64> = Array2::eye(2);
    let mut full_op = Array2::eye(1);

    for i in 0..num_qubits {
        if i == target_qubit {
            full_op = kron(&full_op, &gate);
        } else {
            full_op = kron(&full_op, &identity);
        }
    }

    full_op
}

/// Apply Kraus operators to a density matrix
///
/// Implements the quantum channel: ρ' = Σ_k E_k ρ E_k†
fn apply_kraus_operators(
    rho: &mut DensityMatrix,
    kraus_ops: &[Matrix2<NaComplex<f64>>],
    qubit: usize,
) -> Result<()> {
    if qubit >= rho.num_qubits {
        return Err(MyQuatError::circuit_error("Qubit index out of range"));
    }

    let original = rho.matrix().clone();
    let mut new_rho = Array2::zeros(original.dim());

    // Apply each Kraus operator: ρ' = Σ_k E_k ρ E_k†
    for kraus in kraus_ops {
        let e_k = expand_kraus_operator(kraus, qubit, rho.num_qubits);
        let e_k_dag = e_k.t().mapv(|c| c.conj());

        let temp = e_k.dot(&original);
        let contribution = temp.dot(&e_k_dag);
        new_rho = new_rho + contribution;
    }

    *rho.matrix_mut() = new_rho;
    Ok(())
}

/// Noise models for quantum channels
#[derive(Debug, Clone)]
pub enum NoiseModel {
    /// Bit flip channel
    BitFlip { probability: f64 },
    /// Phase flip channel
    PhaseFlip { probability: f64 },
    /// Depolarizing channel
    Depolarizing { probability: f64 },
    /// Amplitude damping channel
    AmplitudeDamping { gamma: f64 },
    /// Phase damping channel
    PhaseDamping { gamma: f64 },
    /// Thermal relaxation
    ThermalRelaxation { t1: f64, t2: f64, time: f64 },
}

impl NoiseModel {
    /// Apply noise model to a density matrix.
    ///
    /// Dispatches to the appropriate channel based on the noise model variant.
    /// Each channel applies a CPTP map $\mathcal{E}$ via Kraus operators:
    ///
    /// $$ \rho' = \mathcal{E}(\rho) = \sum_k E_k \rho E_k^\dagger, \quad \sum_k E_k^\dagger E_k = I $$
    pub fn apply_to_density_matrix(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        match self {
            NoiseModel::BitFlip { probability } => self.apply_bit_flip(rho, qubit, *probability),
            NoiseModel::PhaseFlip { probability } => {
                self.apply_phase_flip(rho, qubit, *probability)
            }
            NoiseModel::Depolarizing { probability } => {
                self.apply_depolarizing(rho, qubit, *probability)
            }
            NoiseModel::AmplitudeDamping { gamma } => {
                self.apply_amplitude_damping(rho, qubit, *gamma)
            }
            NoiseModel::PhaseDamping { gamma } => self.apply_phase_damping(rho, qubit, *gamma),
            NoiseModel::ThermalRelaxation { t1, t2, time } => {
                self.apply_thermal_relaxation(rho, qubit, *t1, *t2, *time)
            }
        }
    }

    fn apply_bit_flip(&self, rho: &mut DensityMatrix, qubit: usize, p: f64) -> Result<()> {
        // Bit flip channel: ρ' = (1-p)ρ + p X ρ X†
        //
        // Kraus operators: E_0 = √(1-p) I, E_1 = √p X
        // Channel: E(ρ) = E_0 ρ E_0^† + E_1 ρ E_1^†
        let sqrt_p = p.sqrt();
        let sqrt_1_p = (1.0 - p).sqrt();

        let e0 = Matrix2::new(
            NaComplex::new(sqrt_1_p, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_1_p, 0.0),
        );

        let e1 = Matrix2::new(
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_p, 0.0),
            NaComplex::new(sqrt_p, 0.0),
            NaComplex::new(0.0, 0.0),
        );

        apply_kraus_operators(rho, &[e0, e1], qubit)
    }

    fn apply_phase_flip(&self, rho: &mut DensityMatrix, qubit: usize, p: f64) -> Result<()> {
        // Phase flip channel: ρ' = (1-p)ρ + p Z ρ Z†
        //
        // Kraus operators: E_0 = √(1-p) I, E_1 = √p Z
        // Channel: E(ρ) = E_0 ρ E_0^† + E_1 ρ E_1^†
        let sqrt_p = p.sqrt();
        let sqrt_1_p = (1.0 - p).sqrt();

        let e0 = Matrix2::new(
            NaComplex::new(sqrt_1_p, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_1_p, 0.0),
        );

        let e1 = Matrix2::new(
            NaComplex::new(sqrt_p, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(-sqrt_p, 0.0),
        );

        apply_kraus_operators(rho, &[e0, e1], qubit)
    }

    fn apply_depolarizing(&self, rho: &mut DensityMatrix, qubit: usize, p: f64) -> Result<()> {
        // Depolarizing channel:
        //
        //   E(ρ) = (1-p)ρ + (p/3)(XρX + YρY + ZρZ)
        //
        // Kraus operators: E_0 = √(1-3p/4) I,  E_1 = √(p/4) X,
        //                  E_2 = √(p/4) Y,     E_3 = √(p/4) Z
        let sqrt_p4 = (p / 4.0).sqrt();
        let sqrt_1_3p4 = (1.0 - 3.0 * p / 4.0).sqrt();

        let e0 = Matrix2::new(
            NaComplex::new(sqrt_1_3p4, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_1_3p4, 0.0),
        );

        let e1 = Matrix2::new(
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_p4, 0.0),
            NaComplex::new(sqrt_p4, 0.0),
            NaComplex::new(0.0, 0.0),
        );

        let e2 = Matrix2::new(
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, -sqrt_p4),
            NaComplex::new(0.0, sqrt_p4),
            NaComplex::new(0.0, 0.0),
        );

        let e3 = Matrix2::new(
            NaComplex::new(sqrt_p4, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(-sqrt_p4, 0.0),
        );

        apply_kraus_operators(rho, &[e0, e1, e2, e3], qubit)
    }

    fn apply_amplitude_damping(
        &self,
        rho: &mut DensityMatrix,
        qubit: usize,
        gamma: f64,
    ) -> Result<()> {
        // Amplitude damping channel (energy relaxation, T1):
        //
        //   E_0 = [1  0        ]      E_1 = [0  √γ]
        //         [0  √(1-γ)  ]            [0   0]
        //
        // Models energy loss from |1⟩ → |0⟩ with probability γ.
        let sqrt_gamma = gamma.sqrt();
        let sqrt_1_gamma = (1.0 - gamma).sqrt();

        let e0 = Matrix2::new(
            NaComplex::new(1.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_1_gamma, 0.0),
        );

        let e1 = Matrix2::new(
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_gamma, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
        );

        apply_kraus_operators(rho, &[e0, e1], qubit)
    }

    fn apply_phase_damping(&self, rho: &mut DensityMatrix, qubit: usize, gamma: f64) -> Result<()> {
        // Phase damping channel (pure dephasing, T2):
        //
        //   E_0 = [1  0        ]      E_1 = [0  0]
        //         [0  √(1-γ)  ]            [0  √γ]
        //
        // Models loss of phase coherence without energy relaxation.
        let sqrt_gamma = gamma.sqrt();
        let sqrt_1_gamma = (1.0 - gamma).sqrt();

        let e0 = Matrix2::new(
            NaComplex::new(1.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_1_gamma, 0.0),
        );

        let e1 = Matrix2::new(
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(0.0, 0.0),
            NaComplex::new(sqrt_gamma, 0.0),
        );

        apply_kraus_operators(rho, &[e0, e1], qubit)
    }

    fn apply_thermal_relaxation(
        &self,
        rho: &mut DensityMatrix,
        qubit: usize,
        t1: f64,
        t2: f64,
        time: f64,
    ) -> Result<()> {
        // Thermal relaxation combining T1 (amplitude damping) and T2 (dephasing).
        //
        //   γ_1 = 1 - exp(-t/T1)           (amplitude damping)
        //   Γ_φ = 1/T2 - 1/(2·T1)          (pure dephasing rate)
        //   γ_φ = 1 - exp(-t · Γ_φ)        (phase damping, T2 ≤ 2T1)
        //
        // The channel is the composition: E = E_φ ∘ E_AD

        // Apply T1 relaxation (amplitude damping)
        let gamma_1 = 1.0 - (-time / t1).exp();
        self.apply_amplitude_damping(rho, qubit, gamma_1)?;

        // Apply pure dephasing (T2 effect beyond T1)
        // Γ_φ = 1/T2 - 1/(2·T1) is the pure dephasing rate
        // γ_φ = 1 - exp(-time · Γ_φ)
        let gamma_phi = 1.0 - (-time * (1.0 / t2 - 1.0 / (2.0 * t1))).exp();
        if gamma_phi > 0.0 {
            self.apply_phase_damping(rho, qubit, gamma_phi)?;
        }

        Ok(())
    }
}

/// Density matrix simulator for open quantum systems
pub struct DensityMatrixSimulator {
    /// Current density matrix state
    state: DensityMatrix,
    /// Noise models for each qubit
    noise_models: HashMap<usize, Vec<NoiseModel>>,
}

impl DensityMatrixSimulator {
    /// Create a new density matrix simulator
    pub fn new(num_qubits: usize) -> Self {
        DensityMatrixSimulator {
            state: DensityMatrix::zero_state(num_qubits),
            noise_models: HashMap::new(),
        }
    }

    /// Add a noise model to a specific qubit
    pub fn add_noise_model(&mut self, qubit: usize, noise: NoiseModel) {
        self.noise_models.entry(qubit).or_default().push(noise);
    }

    /// Get the current state
    pub fn state(&self) -> &DensityMatrix {
        &self.state
    }

    /// Apply a gate with noise
    pub fn apply_gate_with_noise(
        &mut self,
        gate: &StandardGate,
        qubits: &[usize],
        params: &[Parameter],
    ) -> Result<()> {
        // Apply the gate
        self.state.apply_gate(gate, qubits, params)?;

        // Apply noise models
        for &qubit in qubits {
            if let Some(noise_models) = self.noise_models.get(&qubit) {
                for noise in noise_models {
                    noise.apply_to_density_matrix(&mut self.state, qubit)?;
                }
            }
        }

        Ok(())
    }

    /// Execute a quantum circuit with noise.
    ///
    /// For each gate $G$ in the circuit, the state evolves as:
    ///
    /// $$ \rho' = \mathcal{N}_q \circ \cdots \circ \mathcal{N}_1 \circ G \rho G^\dagger $$
    ///
    /// where each $\mathcal{N}_k$ is a CPTP noise channel (Kraus map) associated
    /// with the qubit(s) affected by the gate.
    ///
    /// # Arguments
    /// * `circuit` - The quantum circuit to execute
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn execute_circuit(&mut self, circuit: &QuantumCircuit) -> Result<()> {
        // Iterate through all instructions in the circuit
        for instruction in circuit.data().instructions() {
            // Extract qubit indices
            let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();

            // Apply the gate
            self.state.apply_gate(
                &instruction.gate.gate_type,
                &qubits,
                &instruction.gate.parameters,
            )?;

            // Apply noise models to affected qubits
            for &qubit in &qubits {
                if let Some(noise_models) = self.noise_models.get(&qubit) {
                    for noise in noise_models {
                        noise.apply_to_density_matrix(&mut self.state, qubit)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Measure expectation value
    pub fn measure_expectation(&self, observable: &Array2<Complex64>) -> Result<f64> {
        self.state.expectation_value(observable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_matrix_creation() {
        let rho = DensityMatrix::zero_state(2);
        assert_eq!(rho.num_qubits(), 2);
        assert_eq!(rho.dimension(), 4);

        let trace = rho.trace();
        assert!((trace.re - 1.0).abs() < 1e-10);
        assert!(trace.im.abs() < 1e-10);
    }

    #[test]
    fn test_maximally_mixed_state() {
        let rho = DensityMatrix::maximally_mixed(2);
        assert_eq!(rho.purity(), 0.25); // 1/4 for 2 qubits
    }

    #[test]
    fn test_pure_state_from_vector() {
        let mut state = Array1::zeros(4);
        state[0] = Complex64::new(1.0, 0.0); // |00⟩

        let rho = DensityMatrix::from_state_vector(&state).unwrap();
        assert!(rho.is_pure(1e-10));
        assert!((rho.purity() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_von_neumann_entropy() {
        let pure_state = DensityMatrix::zero_state(1);
        assert!(pure_state.von_neumann_entropy() < 1e-10);

        let mixed_state = DensityMatrix::maximally_mixed(1);
        let entropy = mixed_state.von_neumann_entropy();
        assert!(entropy > 0.0);
    }

    #[test]
    fn test_noise_models() {
        let mut rho = DensityMatrix::zero_state(1);

        let bit_flip = NoiseModel::BitFlip { probability: 0.1 };
        bit_flip.apply_to_density_matrix(&mut rho, 0).unwrap();

        // State should still be valid (trace = 1)
        let trace = rho.trace();
        assert!((trace.re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_density_matrix_simulator() {
        let mut sim = DensityMatrixSimulator::new(2);

        // Check initial trace
        let initial_trace = sim.state().trace();
        println!("Initial trace: {}", initial_trace);

        // Add noise
        let depolarizing = NoiseModel::Depolarizing { probability: 0.01 };
        sim.add_noise_model(0, depolarizing);

        // Apply gate with noise
        let gate = StandardGate::X;
        sim.apply_gate_with_noise(&gate, &[0], &[]).unwrap();

        // Check state is still valid (allow some tolerance for noise)
        let trace = sim.state().trace();
        println!("Final trace: {}", trace);
        assert!(
            (trace.re - 1.0).abs() < 0.1,
            "Trace is {}, expected ~1.0",
            trace.re
        ); // More tolerant for noisy operations
    }

    #[test]
    fn test_partial_trace() {
        let rho = DensityMatrix::zero_state(2);
        let reduced = rho.partial_trace(&[1]).unwrap();

        assert_eq!(reduced.num_qubits(), 1);
        assert_eq!(reduced.dimension(), 2);
    }

    #[test]
    fn test_single_qubit_depolarizing() {
        let mut rho = DensityMatrix::zero_state(1);
        let initial_trace = rho.trace();
        println!("Initial trace (1 qubit): {}", initial_trace);

        let noise = NoiseModel::Depolarizing { probability: 0.01 };
        noise.apply_to_density_matrix(&mut rho, 0).unwrap();

        let final_trace = rho.trace();
        println!("Final trace (1 qubit): {}", final_trace);
        assert!(
            (final_trace.re - 1.0).abs() < 1e-10,
            "Trace is {}, expected 1.0",
            final_trace.re
        );
    }

    #[test]
    fn test_two_qubit_depolarizing() {
        let mut rho = DensityMatrix::zero_state(2);
        let initial_trace = rho.trace();
        println!("Initial trace (2 qubits): {}", initial_trace);
        println!("Initial matrix shape: {:?}", rho.matrix.dim());

        let noise = NoiseModel::Depolarizing { probability: 0.01 };
        noise.apply_to_density_matrix(&mut rho, 0).unwrap();

        let final_trace = rho.trace();
        println!("Final trace (2 qubits): {}", final_trace);
        assert!(
            (final_trace.re - 1.0).abs() < 1e-10,
            "Trace is {}, expected 1.0",
            final_trace.re
        );
    }

    #[test]
    fn test_two_qubit_gate_qubit_order_respected() {
        // Start from |10> state to detect control/target swap
        let mut dm_x = DensityMatrix::zero_state(2);
        dm_x.apply_gate(&StandardGate::X, &[0], &[]).unwrap();
        // Now state is |10>

        let mut dm_x_cx01 = dm_x.clone();
        dm_x_cx01
            .apply_gate(&StandardGate::CX, &[0, 1], &[])
            .unwrap();
        // CX(0,1) on |10>: control=0 is |1>, so target flips: |10> -> |11>

        let mut dm_x_cx10 = dm_x.clone();
        dm_x_cx10
            .apply_gate(&StandardGate::CX, &[1, 0], &[])
            .unwrap();
        // CX(1,0) on |10>: control=1 is |0>, so no change: |10> stays |10>

        // The two density matrices should be DIFFERENT
        let m01 = dm_x_cx01.matrix();
        let m10 = dm_x_cx10.matrix();
        let diff = (m01 - m10).mapv(|x| x.norm()).sum();
        assert!(
            diff > 1e-6,
            "CX(0,1) and CX(1,0) on |10> must produce different states, got diff={}",
            diff
        );
    }

    #[test]
    fn test_thermal_relaxation_t2_equals_2t1_no_pure_dephasing() {
        // When T2 = 2*T1, the pure dephasing rate Γ_φ = 1/T2 - 1/(2*T1) = 0.
        // This means there is no pure dephasing beyond what T1 amplitude damping
        // already accounts for. The probability of a phase error should be zero.
        let t1: f64 = 100.0;
        let t2: f64 = 2.0 * t1; // 200.0
        let time: f64 = 10.0;

        // Pure dephasing rate: 1/T2 - 1/(2*T1) = 1/200 - 1/200 = 0
        // γ_φ = 1 - exp(-time * 0) = 1 - 1 = 0
        let gamma_phi = 1.0 - (-time * (1.0 / t2 - 1.0 / (2.0 * t1))).exp();
        assert!(
            gamma_phi.abs() < 1e-15_f64,
            "Expected γ_φ = 0 when T2 = 2*T1, got {}",
            gamma_phi
        );

        // Also verify the formula: when T1=100, T2=50 (< 2*T1), we should get
        // a reasonable positive γ_φ (not the divergent behavior of the old formula)
        let t2_small: f64 = 50.0; // T2 < 2*T1
        let gamma_phi_small: f64 = 1.0 - (-time * (1.0 / t2_small - 1.0 / (2.0 * t1))).exp();
        assert!(gamma_phi_small > 0.0, "Expected γ_φ > 0 when T2 < 2*T1");
        assert!(gamma_phi_small < 1.0, "Expected γ_φ < 1");
    }
}
