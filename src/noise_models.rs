//! Advanced noise models for realistic quantum device simulation
//!
//! This module provides comprehensive noise modeling capabilities for simulating
//! real quantum devices, including decoherence, gate errors, and measurement errors.

use crate::density_matrix::DensityMatrix;
use crate::{MyQuatError, Result};
use ndarray::Array2;
use num_complex::Complex64;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive noise model for quantum devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNoiseModel {
    /// Single-qubit relaxation times (T1) in microseconds
    pub t1_times: HashMap<usize, f64>,
    /// Single-qubit dephasing times (T2) in microseconds  
    pub t2_times: HashMap<usize, f64>,
    /// Gate error rates for different gate types
    pub gate_errors: HashMap<String, f64>,
    /// Readout error probabilities (0->1 and 1->0)
    pub readout_errors: HashMap<usize, (f64, f64)>,
    /// Thermal population (ground state probability)
    pub thermal_population: f64,
    /// Gate execution times in nanoseconds
    pub gate_times: HashMap<String, f64>,
}

impl DeviceNoiseModel {
    /// Create a new device noise model
    pub fn new() -> Self {
        Self {
            t1_times: HashMap::new(),
            t2_times: HashMap::new(),
            gate_errors: HashMap::new(),
            readout_errors: HashMap::new(),
            thermal_population: 1.0, // Perfect ground state
            gate_times: HashMap::new(),
        }
    }

    /// Create a realistic noise model based on current NISQ devices
    pub fn realistic_device(num_qubits: usize) -> Self {
        let mut model = Self::new();

        // Typical T1/T2 times for superconducting qubits (in microseconds)
        for qubit in 0..num_qubits {
            // T1: 20-100 μs, T2: 10-50 μs (T2 ≤ 2*T1)
            let t1 = 50.0 + (qubit as f64 * 10.0) % 50.0; // 50-100 μs
            let t2 = (20.0 + (qubit as f64 * 5.0) % 30.0).min(2.0 * t1); // 20-50 μs

            model.t1_times.insert(qubit, t1);
            model.t2_times.insert(qubit, t2);

            // Readout errors: typically 1-5%
            let readout_error = 0.01 + (qubit as f64 * 0.005) % 0.04;
            model
                .readout_errors
                .insert(qubit, (readout_error, readout_error));
        }

        // Gate error rates (typical values)
        model.gate_errors.insert("X".to_string(), 0.001); // 0.1%
        model.gate_errors.insert("Y".to_string(), 0.001); // 0.1%
        model.gate_errors.insert("Z".to_string(), 0.0001); // 0.01% (virtual)
        model.gate_errors.insert("H".to_string(), 0.001); // 0.1%
        model.gate_errors.insert("RX".to_string(), 0.001); // 0.1%
        model.gate_errors.insert("RY".to_string(), 0.001); // 0.1%
        model.gate_errors.insert("RZ".to_string(), 0.0001); // 0.01% (virtual)
        model.gate_errors.insert("CNOT".to_string(), 0.01); // 1%
        model.gate_errors.insert("CZ".to_string(), 0.01); // 1%

        // Gate execution times (nanoseconds)
        model.gate_times.insert("X".to_string(), 20.0);
        model.gate_times.insert("Y".to_string(), 20.0);
        model.gate_times.insert("Z".to_string(), 0.0); // Virtual gate
        model.gate_times.insert("H".to_string(), 20.0);
        model.gate_times.insert("RX".to_string(), 20.0);
        model.gate_times.insert("RY".to_string(), 20.0);
        model.gate_times.insert("RZ".to_string(), 0.0); // Virtual gate
        model.gate_times.insert("CNOT".to_string(), 200.0); // Two-qubit gates slower
        model.gate_times.insert("CZ".to_string(), 200.0);

        // Realistic thermal population (99.9% ground state at ~10mK)
        model.thermal_population = 0.999;

        model
    }

    /// Set T1 relaxation time for a qubit
    pub fn set_t1(&mut self, qubit: usize, t1_us: f64) {
        self.t1_times.insert(qubit, t1_us);
    }

    /// Set T2 dephasing time for a qubit
    pub fn set_t2(&mut self, qubit: usize, t2_us: f64) {
        self.t2_times.insert(qubit, t2_us);
    }

    /// Set gate error rate
    pub fn set_gate_error(&mut self, gate: &str, error_rate: f64) {
        self.gate_errors.insert(gate.to_string(), error_rate);
    }

    /// Set readout error for a qubit
    pub fn set_readout_error(&mut self, qubit: usize, error_0_to_1: f64, error_1_to_0: f64) {
        self.readout_errors
            .insert(qubit, (error_0_to_1, error_1_to_0));
    }
}

impl Default for DeviceNoiseModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Decoherence noise channel implementing T1 and T2 processes
#[derive(Debug, Clone)]
pub struct DecoherenceChannel {
    /// T1 relaxation time (microseconds)
    pub t1: f64,
    /// T2 dephasing time (microseconds)
    pub t2: f64,
    /// Gate execution time (nanoseconds)
    pub gate_time: f64,
}

impl DecoherenceChannel {
    /// Create new decoherence channel
    pub fn new(t1_us: f64, t2_us: f64, gate_time_ns: f64) -> Self {
        Self {
            t1: t1_us,
            t2: t2_us,
            gate_time: gate_time_ns,
        }
    }

    /// Apply decoherence to a density matrix
    pub fn apply(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        if qubit >= rho.num_qubits() {
            return Err(MyQuatError::circuit_error("Invalid qubit index"));
        }

        // Convert gate time from ns to μs
        let time_us = self.gate_time / 1000.0;

        // Calculate decay probabilities
        let gamma1 = time_us / self.t1; // Relaxation rate
        let gamma2 = time_us / self.t2; // Dephasing rate
        let gamma_phi = gamma2 - gamma1 / 2.0; // Pure dephasing rate

        // Apply amplitude damping (T1 process)
        if gamma1 > 0.0 {
            self.apply_amplitude_damping(rho, qubit, gamma1)?;
        }

        // Apply pure dephasing (T2* process)
        if gamma_phi > 0.0 {
            self.apply_phase_damping(rho, qubit, gamma_phi)?;
        }

        Ok(())
    }

    /// Apply amplitude damping (T1 relaxation)
    fn apply_amplitude_damping(
        &self,
        rho: &mut DensityMatrix,
        qubit: usize,
        gamma: f64,
    ) -> Result<()> {
        // Kraus operators for amplitude damping
        // E0 = |0⟩⟨0| + √(1-γ)|1⟩⟨1|
        // E1 = √γ |0⟩⟨1|

        let p = (-gamma).exp(); // Survival probability
        let sqrt_1_minus_p = (1.0 - p).sqrt();

        // Get current density matrix
        let original = rho.matrix().clone();
        let dim = original.dim().0;

        // Apply Kraus operators
        let mut new_rho = Array2::zeros((dim, dim));

        // E0 ρ E0†
        let mut e0: Array2<Complex64> = Array2::eye(dim);
        if qubit < rho.num_qubits() {
            // Modify the (1,1) element for this qubit
            let qubit_dim = 1 << qubit;
            for i in 0..dim {
                for j in 0..dim {
                    if (i & qubit_dim) != 0 && (j & qubit_dim) != 0 {
                        e0[[i, j]] *= Complex64::new(p.sqrt(), 0.0);
                    }
                }
            }
        }

        // E0 ρ E0† contribution
        for i in 0..dim {
            for j in 0..dim {
                for k in 0..dim {
                    for l in 0..dim {
                        new_rho[[i, j]] += e0[[i, k]] * original[[k, l]] * e0[[j, l]].conj();
                    }
                }
            }
        }

        // E1 ρ E1† (decay from |1⟩ to |0⟩)
        if sqrt_1_minus_p > 0.0 {
            let qubit_dim = 1 << qubit;
            for i in 0..dim {
                for j in 0..dim {
                    // Only contribute if both states have qubit in |1⟩ state
                    if (i & qubit_dim) == 0 && (j & qubit_dim) == 0 {
                        // Find corresponding |1⟩ states
                        let i1 = i | qubit_dim;
                        let j1 = j | qubit_dim;
                        if i1 < dim && j1 < dim {
                            new_rho[[i, j]] += Complex64::new(sqrt_1_minus_p, 0.0)
                                * original[[i1, j1]]
                                * Complex64::new(sqrt_1_minus_p, 0.0);
                        }
                    }
                }
            }
        }

        *rho.matrix_mut() = new_rho;
        Ok(())
    }

    /// Apply phase damping (pure dephasing)
    fn apply_phase_damping(
        &self,
        rho: &mut DensityMatrix,
        qubit: usize,
        gamma_phi: f64,
    ) -> Result<()> {
        // Kraus operators for phase damping
        // E0 = |0⟩⟨0| + √(1-γ_φ)|1⟩⟨1|
        // E1 = √γ_φ |1⟩⟨1|

        let p = (-gamma_phi).exp(); // Coherence survival probability

        let original = rho.matrix().clone();
        let dim = original.dim().0;
        let qubit_dim = 1 << qubit;

        // Apply dephasing: reduce off-diagonal elements
        let mut new_rho = original.clone();

        for i in 0..dim {
            for j in 0..dim {
                // Check if this is an off-diagonal element in the qubit subspace
                if (i & qubit_dim) != (j & qubit_dim) {
                    new_rho[[i, j]] *= Complex64::new(p.sqrt(), 0.0);
                }
            }
        }

        *rho.matrix_mut() = new_rho;
        Ok(())
    }
}

/// Depolarizing noise channel
///
/// Applies the standard single-qubit depolarizing channel:
///   ρ → (1 - 3p/4) ρ + p/4 (X ρ X† + Y ρ Y† + Z ρ Z†)
///
/// Kraus operators:
///   E₀ = √(1 - 3p/4) I,  E₁ = √(p/4) X,  E₂ = √(p/4) Y,  E₃ = √(p/4) Z
///
/// This is a **per-qubit** channel: it acts only on the specified qubit,
/// leaving other qubits unchanged (via Kronecker product with identity).
#[derive(Debug, Clone)]
pub struct DepolarizingChannel {
    /// Depolarizing probability (must be in [0, 1])
    pub probability: f64,
}

/// Kronecker product of two matrices
fn kron(a: &Array2<Complex64>, b: &Array2<Complex64>) -> Array2<Complex64> {
    let (m, n) = a.dim();
    let (p, q) = b.dim();
    let mut result = Array2::zeros((m * p, n * q));
    for i in 0..m {
        for j in 0..n {
            for bi in 0..p {
                for bj in 0..q {
                    result[[i * p + bi, j * q + bj]] = a[[i, j]] * b[[bi, bj]];
                }
            }
        }
    }
    result
}

impl DepolarizingChannel {
    /// Create new depolarizing channel
    ///
    /// # Panics
    ///
    /// Panics if `probability` is negative, NaN, or greater than 1.
    pub fn new(probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&probability),
            "DepolarizingChannel probability must be in [0, 1], got {}",
            probability
        );
        Self { probability }
    }

    /// Apply per-qubit depolarizing noise to a density matrix.
    ///
    /// Applies the Kraus representation: ρ' = Σₖ Eₖ ρ Eₖ†
    /// where Eₖ are tensor products of single-qubit Kraus operators
    /// on the target qubit with identity on all other qubits.
    pub fn apply(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        if qubit >= rho.num_qubits() {
            return Err(MyQuatError::circuit_error("Qubit index out of range"));
        }

        let p = self.probability;
        if p <= 0.0 {
            return Ok(());
        }

        let n = rho.num_qubits();
        let dim = 1 << n;

        // Kraus operators for single-qubit depolarizing channel
        let sqrt_p4 = (p / 4.0).sqrt();
        let sqrt_1_3p4 = (1.0 - 3.0 * p / 4.0).sqrt();

        // E0 = sqrt(1-3p/4) * I
        let e0: Array2<Complex64> = Array2::eye(2) * Complex64::new(sqrt_1_3p4, 0.0);

        // E1 = sqrt(p/4) * X
        let mut e1 = Array2::zeros((2, 2));
        e1[[0, 1]] = Complex64::new(sqrt_p4, 0.0);
        e1[[1, 0]] = Complex64::new(sqrt_p4, 0.0);

        // E2 = sqrt(p/4) * Y
        let mut e2 = Array2::zeros((2, 2));
        e2[[0, 1]] = Complex64::new(0.0, -sqrt_p4);
        e2[[1, 0]] = Complex64::new(0.0, sqrt_p4);

        // E3 = sqrt(p/4) * Z
        let mut e3 = Array2::zeros((2, 2));
        e3[[0, 0]] = Complex64::new(sqrt_p4, 0.0);
        e3[[1, 1]] = Complex64::new(-sqrt_p4, 0.0);

        let kraus_ops: [Array2<Complex64>; 4] = [e0, e1, e2, e3];

        // Expand each Kraus operator to full system via Kronecker products
        let identity: Array2<Complex64> = Array2::eye(2);
        let full_ops: Vec<Array2<Complex64>> = kraus_ops
            .iter()
            .map(|op| {
                let mut full_op = Array2::eye(1);
                for i in 0..n {
                    if i == qubit {
                        full_op = kron(&full_op, op);
                    } else {
                        full_op = kron(&full_op, &identity);
                    }
                }
                full_op
            })
            .collect();

        // ρ' = Σₖ Eₖ ρ Eₖ†
        let original = rho.matrix().clone();
        let mut new_rho = Array2::zeros((dim, dim));
        for op in &full_ops {
            let op_dag = op.t().mapv(|c| c.conj());
            let temp = op.dot(&original);
            let contribution = temp.dot(&op_dag);
            new_rho = new_rho + contribution;
        }
        *rho.matrix_mut() = new_rho;

        Ok(())
    }
}

/// Pauli noise channel (random Pauli errors)
#[derive(Debug, Clone)]
pub struct PauliChannel {
    /// Probabilities for X, Y, Z errors
    pub error_probs: [f64; 3], // [P_X, P_Y, P_Z]
}

impl PauliChannel {
    /// Create new Pauli channel
    pub fn new(px: f64, py: f64, pz: f64) -> Self {
        Self {
            error_probs: [px, py, pz],
        }
    }

    /// Create symmetric Pauli channel
    pub fn symmetric(error_rate: f64) -> Self {
        let p_each = error_rate / 3.0;
        Self::new(p_each, p_each, p_each)
    }

    /// Apply Pauli noise
    pub fn apply(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        let mut rng = rng();
        let rand_val: f64 = rng.random();

        let total_error = self.error_probs.iter().sum::<f64>();
        if rand_val > total_error {
            return Ok(()); // No error
        }

        // Determine which Pauli error to apply
        let mut cumulative = 0.0;
        for (i, &prob) in self.error_probs.iter().enumerate() {
            cumulative += prob;
            if rand_val <= cumulative {
                match i {
                    0 => self.apply_pauli_x(rho, qubit)?,
                    1 => self.apply_pauli_y(rho, qubit)?,
                    2 => self.apply_pauli_z(rho, qubit)?,
                    _ => unreachable!(),
                }
                break;
            }
        }

        Ok(())
    }

    fn apply_pauli_x(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        // Apply X gate to density matrix: X ρ X†
        let original = rho.matrix().clone();
        let dim = original.dim().0;
        let qubit_dim = 1 << qubit;

        let mut new_rho = Array2::zeros((dim, dim));

        for i in 0..dim {
            for j in 0..dim {
                // Flip the qubit bit
                let i_flipped = i ^ qubit_dim;
                let j_flipped = j ^ qubit_dim;
                new_rho[[i, j]] = original[[i_flipped, j_flipped]];
            }
        }

        *rho.matrix_mut() = new_rho;
        Ok(())
    }

    fn apply_pauli_y(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        // Apply Y gate: Y = iXZ, introduces phase factors
        let original = rho.matrix().clone();
        let dim = original.dim().0;
        let qubit_dim = 1 << qubit;

        let mut new_rho = Array2::zeros((dim, dim));

        for i in 0..dim {
            for j in 0..dim {
                let i_flipped = i ^ qubit_dim;
                let j_flipped = j ^ qubit_dim;

                // Y gate phase factors: |0⟩ → i|1⟩, |1⟩ → -i|0⟩
                let phase_i = if (i & qubit_dim) == 0 {
                    Complex64::i()
                } else {
                    -Complex64::i()
                };
                let phase_j = if (j & qubit_dim) == 0 {
                    -Complex64::i()
                } else {
                    Complex64::i()
                };

                new_rho[[i, j]] = phase_i * original[[i_flipped, j_flipped]] * phase_j;
            }
        }

        *rho.matrix_mut() = new_rho;
        Ok(())
    }

    fn apply_pauli_z(&self, rho: &mut DensityMatrix, qubit: usize) -> Result<()> {
        // Apply Z gate: adds phase to |1⟩ state
        let original = rho.matrix().clone();
        let dim = original.dim().0;
        let qubit_dim = 1 << qubit;

        let mut new_rho = original.clone();

        for i in 0..dim {
            for j in 0..dim {
                // Z gate: |0⟩ → |0⟩, |1⟩ → -|1⟩
                let phase_i = if (i & qubit_dim) != 0 { -1.0 } else { 1.0 };
                let phase_j = if (j & qubit_dim) != 0 { -1.0 } else { 1.0 };

                new_rho[[i, j]] = original[[i, j]] * Complex64::new(phase_i * phase_j, 0.0);
            }
        }

        *rho.matrix_mut() = new_rho;
        Ok(())
    }
}

/// Readout error model
#[derive(Debug, Clone)]
pub struct ReadoutErrorModel {
    /// Error probability: 0 → 1
    pub error_0_to_1: f64,
    /// Error probability: 1 → 0  
    pub error_1_to_0: f64,
}

impl ReadoutErrorModel {
    /// Create new readout error model
    pub fn new(error_0_to_1: f64, error_1_to_0: f64) -> Self {
        Self {
            error_0_to_1,
            error_1_to_0,
        }
    }

    /// Create symmetric readout error
    pub fn symmetric(error_rate: f64) -> Self {
        Self::new(error_rate, error_rate)
    }

    /// Apply readout error to measurement result
    pub fn apply_to_measurement(&self, measured_bit: bool) -> bool {
        let mut rng = rng();
        let rand_val: f64 = rng.random();

        match measured_bit {
            false => {
                // Measured 0, might flip to 1
                rand_val < self.error_0_to_1
            }
            true => {
                // Measured 1, might flip to 0
                rand_val >= self.error_1_to_0
            }
        }
    }

    /// Get readout fidelity
    pub fn fidelity(&self) -> f64 {
        1.0 - (self.error_0_to_1 + self.error_1_to_0) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_noise_model_creation() {
        let model = DeviceNoiseModel::new();
        assert!(model.t1_times.is_empty());
        assert!(model.t2_times.is_empty());

        let realistic = DeviceNoiseModel::realistic_device(3);
        assert_eq!(realistic.t1_times.len(), 3);
        assert_eq!(realistic.t2_times.len(), 3);
        assert!(realistic.gate_errors.contains_key("CNOT"));
    }

    #[test]
    fn test_decoherence_channel() {
        let channel = DecoherenceChannel::new(50.0, 30.0, 20.0);
        assert_eq!(channel.t1, 50.0);
        assert_eq!(channel.t2, 30.0);
        assert_eq!(channel.gate_time, 20.0);
    }

    #[test]
    fn test_depolarizing_channel() {
        let channel = DepolarizingChannel::new(0.01);
        assert_eq!(channel.probability, 0.01);

        let mut rho = DensityMatrix::zero_state(1);
        let result = channel.apply(&mut rho, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_depolarizing_single_qubit() {
        // Verify that applying depolarizing noise to qubit 0 of a 2-qubit
        // system does NOT affect qubit 1 (only the target qubit is depolarized).
        let p = 0.5;
        let channel = DepolarizingChannel::new(p);

        // Start with |0⟩⊗|0⟩: unentangled, pure state
        let mut rho = DensityMatrix::zero_state(2);
        let original = rho.matrix().clone();

        channel.apply(&mut rho, 0).unwrap();
        let after = rho.matrix().clone();

        // The state should have changed (p > 0)
        let diff = (&after - &original).mapv(|c| c.norm()).sum();
        assert!(diff > 1e-10, "State should change when p > 0");

        // Trace should still be 1 (Kraus channel is trace-preserving)
        let trace: Complex64 = after.diag().sum();
        assert!(
            (trace.re - 1.0).abs() < 1e-10,
            "Trace should be 1, got {}",
            trace
        );
        assert!(
            trace.im.abs() < 1e-10,
            "Trace should be real, got {}",
            trace
        );

        // Partial trace over qubit 0: qubit 1 should be untouched (|0⟩⟨0|)
        // ρ₁ = Tr₀(ρ). For 2-qubit system: ρ₁[i,j] = Σₖ ρ[k*2+i, k*2+j]
        let mut rho1 = Array2::zeros((2, 2));
        for i in 0..2 {
            for j in 0..2 {
                let mut sum = Complex64::new(0.0, 0.0);
                for k in 0..2 {
                    sum = sum + after[[k * 2 + i, k * 2 + j]];
                }
                rho1[[i, j]] = sum;
            }
        }
        // With correct per-qubit depolarizing, qubit 1 stays in |0⟩ since
        // only qubit 0 is depolarized and they start unentangled
        let mut expected_q1 = Array2::zeros((2, 2));
        expected_q1[[0, 0]] = Complex64::new(1.0, 0.0);

        for i in 0..2 {
            for j in 0..2 {
                let diff = (rho1[[i, j]] - expected_q1[[i, j]]).norm();
                assert!(
                    diff < 1e-10,
                    "rho1[[{},{}]] = {:?}, expected {:?}, diff = {}",
                    i,
                    j,
                    rho1[[i, j]],
                    expected_q1[[i, j]],
                    diff
                );
            }
        }
    }

    #[test]
    #[should_panic(expected = "probability must be in [0, 1]")]
    fn test_depolarizing_rejects_negative_p() {
        DepolarizingChannel::new(-0.1);
    }

    #[test]
    #[should_panic(expected = "probability must be in [0, 1]")]
    fn test_depolarizing_rejects_p_gt_one() {
        DepolarizingChannel::new(1.5);
    }

    #[test]
    fn test_pauli_channel() {
        let channel = PauliChannel::new(0.001, 0.001, 0.001);
        assert_eq!(channel.error_probs, [0.001, 0.001, 0.001]);

        let symmetric = PauliChannel::symmetric(0.003);
        assert_eq!(symmetric.error_probs, [0.001, 0.001, 0.001]);
    }

    #[test]
    fn test_readout_error_model() {
        let model = ReadoutErrorModel::new(0.02, 0.03);
        assert_eq!(model.error_0_to_1, 0.02);
        assert_eq!(model.error_1_to_0, 0.03);

        let fidelity = model.fidelity();
        assert!((fidelity - 0.975).abs() < 1e-10);

        let symmetric = ReadoutErrorModel::symmetric(0.02);
        assert_eq!(symmetric.error_0_to_1, 0.02);
        assert_eq!(symmetric.error_1_to_0, 0.02);
    }

    #[test]
    fn test_realistic_noise_parameters() {
        let model = DeviceNoiseModel::realistic_device(5);

        // Check T1/T2 relationship: T2 ≤ 2*T1
        for qubit in 0..5 {
            let t1 = model.t1_times[&qubit];
            let t2 = model.t2_times[&qubit];
            assert!(t2 <= 2.0 * t1);
            assert!(t1 > 0.0);
            assert!(t2 > 0.0);
        }

        // Check gate error rates are reasonable
        assert!(model.gate_errors["CNOT"] > model.gate_errors["X"]);
        assert!(model.gate_errors["Z"] < model.gate_errors["X"]);
    }
}
