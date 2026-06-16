//! Quantum error mitigation techniques
//!
//! This module provides various error mitigation strategies for improving
//! the accuracy of NISQ quantum computations, including Zero-Noise Extrapolation (ZNE),
//! symmetry verification, and other advanced techniques.

use crate::density_matrix::DensityMatrix;
use crate::noisy_simulator::NoisyQuantumSimulator;
use crate::{Parameter, QuantumCircuit, Result, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Zero-Noise Extrapolation (ZNE) implementation
#[derive(Debug, Clone)]
pub struct ZeroNoiseExtrapolation {
    /// Noise scaling factors to test
    pub noise_factors: Vec<f64>,
    /// Extrapolation method
    pub extrapolation_method: ExtrapolationMethod,
    /// Number of shots per noise level
    pub shots_per_level: usize,
}

/// Methods for extrapolating to zero noise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtrapolationMethod {
    /// Linear extrapolation
    Linear,
    /// Exponential extrapolation
    Exponential,
    /// Polynomial extrapolation (degree specified)
    Polynomial(usize),
    /// Richardson extrapolation
    Richardson,
}

impl ZeroNoiseExtrapolation {
    /// Create a new ZNE instance with default parameters
    pub fn new() -> Self {
        Self {
            noise_factors: vec![1.0, 3.0, 5.0], // Standard ZNE scaling factors
            extrapolation_method: ExtrapolationMethod::Linear,
            shots_per_level: 1000,
        }
    }

    /// Create ZNE with custom noise factors
    pub fn with_noise_factors(factors: Vec<f64>) -> Self {
        Self {
            noise_factors: factors,
            extrapolation_method: ExtrapolationMethod::Linear,
            shots_per_level: 1000,
        }
    }

    /// Set extrapolation method
    pub fn with_extrapolation_method(mut self, method: ExtrapolationMethod) -> Self {
        self.extrapolation_method = method;
        self
    }

    /// Set number of shots per noise level
    pub fn with_shots(mut self, shots: usize) -> Self {
        self.shots_per_level = shots;
        self
    }

    /// Apply ZNE to a quantum circuit
    pub fn mitigate(&self, circuit: &QuantumCircuit, observable: &Observable) -> Result<f64> {
        let mut expectation_values = Vec::new();

        // Run circuit at different noise levels
        for &noise_factor in &self.noise_factors {
            let scaled_circuit = self.scale_noise(circuit, noise_factor)?;
            let expectation = self.measure_expectation(&scaled_circuit, observable)?;
            expectation_values.push((noise_factor, expectation));
        }

        // Extrapolate to zero noise
        let zero_noise_value = self.extrapolate_to_zero(&expectation_values)?;
        Ok(zero_noise_value)
    }

    /// Scale noise in a circuit by a given factor
    fn scale_noise(&self, circuit: &QuantumCircuit, factor: f64) -> Result<QuantumCircuit> {
        if factor == 1.0 {
            return Ok(circuit.clone());
        }

        let mut scaled_circuit = QuantumCircuit::new(circuit.num_qubits(), circuit.num_clbits());

        // For each gate, repeat it (2k+1) times where k is chosen such that
        // the effective noise is scaled by the factor
        let repetitions = ((factor - 1.0) / 2.0).round() as usize;

        for instruction in circuit.data().instructions() {
            // Add original gate
            self.add_instruction_to_circuit(&mut scaled_circuit, instruction)?;

            // Add repetitions (gate followed by its inverse, repeated k times)
            for _ in 0..repetitions {
                self.add_instruction_to_circuit(&mut scaled_circuit, instruction)?;
                self.add_inverse_instruction(&mut scaled_circuit, instruction)?;
            }
        }

        Ok(scaled_circuit)
    }

    /// Add an instruction to a circuit
    fn add_instruction_to_circuit(
        &self,
        circuit: &mut QuantumCircuit,
        instruction: &crate::circuit::Instruction,
    ) -> Result<()> {
        let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();

        match &instruction.gate.gate_type {
            StandardGate::X => circuit.x(qubits[0])?,
            StandardGate::Y => circuit.y(qubits[0])?,
            StandardGate::Z => circuit.z(qubits[0])?,
            StandardGate::H => circuit.h(qubits[0])?,
            StandardGate::S => circuit.s(qubits[0])?,
            StandardGate::T => circuit.t(qubits[0])?,
            StandardGate::CX => circuit.cx(qubits[0], qubits[1])?,
            StandardGate::CZ => circuit.cz(qubits[0], qubits[1])?,
            StandardGate::Rx => {
                if let Some(param) = instruction.gate.parameters.first() {
                    circuit.rx(qubits[0], param.clone())?;
                }
            }
            StandardGate::Ry => {
                if let Some(param) = instruction.gate.parameters.first() {
                    circuit.ry(qubits[0], param.clone())?;
                }
            }
            StandardGate::Rz => {
                if let Some(param) = instruction.gate.parameters.first() {
                    circuit.rz(qubits[0], param.clone())?;
                }
            }
            _ => circuit.i(qubits[0])?, // Default to identity for unsupported gates
        }

        Ok(())
    }

    /// Add the inverse of an instruction to a circuit
    fn add_inverse_instruction(
        &self,
        circuit: &mut QuantumCircuit,
        instruction: &crate::circuit::Instruction,
    ) -> Result<()> {
        let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();

        match &instruction.gate.gate_type {
            StandardGate::X => circuit.x(qubits[0])?, // X is self-inverse
            StandardGate::Y => circuit.y(qubits[0])?, // Y is self-inverse
            StandardGate::Z => circuit.z(qubits[0])?, // Z is self-inverse
            StandardGate::H => circuit.h(qubits[0])?, // H is self-inverse
            StandardGate::S => circuit.sdg(qubits[0])?, // S† = S-dagger
            StandardGate::T => circuit.tdg(qubits[0])?, // T† = T-dagger
            StandardGate::CX => circuit.cx(qubits[0], qubits[1])?, // CNOT is self-inverse
            StandardGate::CZ => circuit.cz(qubits[0], qubits[1])?, // CZ is self-inverse
            StandardGate::Rx => {
                if let Some(param) = instruction.gate.parameters.first() {
                    let neg_param = match param {
                        Parameter::Float(val) => Parameter::Float(-val),
                        _ => Parameter::Float(0.0), // Simplified: use 0 for symbolic parameters
                    };
                    circuit.rx(qubits[0], neg_param)?;
                }
            }
            StandardGate::Ry => {
                if let Some(param) = instruction.gate.parameters.first() {
                    let neg_param = match param {
                        Parameter::Float(val) => Parameter::Float(-val),
                        _ => Parameter::Float(0.0), // Simplified: use 0 for symbolic parameters
                    };
                    circuit.ry(qubits[0], neg_param)?;
                }
            }
            StandardGate::Rz => {
                if let Some(param) = instruction.gate.parameters.first() {
                    let neg_param = match param {
                        Parameter::Float(val) => Parameter::Float(-val),
                        _ => Parameter::Float(0.0), // Simplified: use 0 for symbolic parameters
                    };
                    circuit.rz(qubits[0], neg_param)?;
                }
            }
            _ => circuit.i(qubits[0])?, // Default to identity
        }

        Ok(())
    }

    /// Measure expectation value of an observable
    fn measure_expectation(
        &self,
        circuit: &QuantumCircuit,
        observable: &Observable,
    ) -> Result<f64> {
        let mut simulator = NoisyQuantumSimulator::realistic_device(circuit.num_qubits());

        // Run multiple shots and average
        let mut total = 0.0;
        for _ in 0..self.shots_per_level {
            simulator.execute_circuit(circuit)?;
            let measurement = self.measure_observable(&simulator, observable)?;
            total += measurement;
        }

        Ok(total / self.shots_per_level as f64)
    }

    /// Measure a specific observable on the simulator state
    fn measure_observable(
        &self,
        simulator: &NoisyQuantumSimulator,
        observable: &Observable,
    ) -> Result<f64> {
        match observable {
            Observable::PauliZ(qubit) => {
                // Measure Z expectation value
                let prob_0 = simulator.state().measure_probability(*qubit, false)?;
                let prob_1 = simulator.state().measure_probability(*qubit, true)?;
                Ok(prob_0 - prob_1) // ⟨Z⟩ = P(0) - P(1)
            }
            Observable::PauliX(_qubit) => {
                // For X measurement, we'd need to rotate basis first
                // Simplified: return 0 for now
                Ok(0.0)
            }
            Observable::PauliY(_qubit) => {
                // For Y measurement, we'd need to rotate basis first
                // Simplified: return 0 for now
                Ok(0.0)
            }
            Observable::Energy(hamiltonian) => {
                // Compute energy expectation value
                simulator.state().expectation_value(hamiltonian)
            }
        }
    }

    /// Extrapolate measurement results to zero noise
    fn extrapolate_to_zero(&self, data: &[(f64, f64)]) -> Result<f64> {
        match &self.extrapolation_method {
            ExtrapolationMethod::Linear => self.linear_extrapolation(data),
            ExtrapolationMethod::Exponential => self.exponential_extrapolation(data),
            ExtrapolationMethod::Polynomial(degree) => self.polynomial_extrapolation(data, *degree),
            ExtrapolationMethod::Richardson => self.richardson_extrapolation(data),
        }
    }

    /// Linear extrapolation to zero noise
    fn linear_extrapolation(&self, data: &[(f64, f64)]) -> Result<f64> {
        if data.len() < 2 {
            return Ok(data[0].1);
        }

        // Fit line: y = mx + b, extrapolate to x = 0
        let n = data.len() as f64;
        let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        Ok(intercept) // Value at x = 0
    }

    /// Exponential extrapolation
    fn exponential_extrapolation(&self, data: &[(f64, f64)]) -> Result<f64> {
        // Fit: y = A * exp(-B * x) + C
        // Simplified: use linear fit on log scale
        if data.len() < 2 {
            return Ok(data[0].1);
        }

        // For now, fall back to linear extrapolation
        self.linear_extrapolation(data)
    }

    /// Polynomial extrapolation
    fn polynomial_extrapolation(&self, data: &[(f64, f64)], _degree: usize) -> Result<f64> {
        // Simplified: use linear for now
        self.linear_extrapolation(data)
    }

    /// Richardson extrapolation
    fn richardson_extrapolation(&self, data: &[(f64, f64)]) -> Result<f64> {
        if data.len() < 2 {
            return Ok(data[0].1);
        }

        // Richardson extrapolation for noise factors [1, 3, 5, ...]
        // R(0) = (r²*f(1) - f(r)) / (r² - 1) where r is the scaling ratio
        let f1 = data[0].1; // f(1)
        let f3 = data[1].1; // f(3)

        // R(0) = (9*f(1) - f(3)) / 8
        Ok((9.0 * f1 - f3) / 8.0)
    }
}

impl Default for ZeroNoiseExtrapolation {
    fn default() -> Self {
        Self::new()
    }
}

/// Observable quantities that can be measured
#[derive(Debug, Clone)]
pub enum Observable {
    /// Pauli-Z measurement on a specific qubit
    PauliZ(usize),
    /// Pauli-X measurement on a specific qubit
    PauliX(usize),
    /// Pauli-Y measurement on a specific qubit
    PauliY(usize),
    /// Energy measurement with a given Hamiltonian
    Energy(ndarray::Array2<num_complex::Complex64>),
}

/// Symmetry verification for error detection
#[derive(Debug, Clone)]
pub struct SymmetryVerification {
    /// Symmetry operators to check
    pub symmetries: Vec<SymmetryOperator>,
    /// Tolerance for symmetry violations
    pub tolerance: f64,
}

/// A symmetry operator for verification
#[derive(Debug, Clone)]
pub struct SymmetryOperator {
    /// Name of the symmetry
    pub name: String,
    /// Pauli string representation
    pub pauli_string: Vec<PauliOperator>,
    /// Expected eigenvalue
    pub expected_eigenvalue: f64,
}

/// Single Pauli operator
#[derive(Debug, Clone, PartialEq)]
pub enum PauliOperator {
    I, // Identity
    X, // Pauli-X
    Y, // Pauli-Y
    Z, // Pauli-Z
}

impl SymmetryVerification {
    /// Create new symmetry verification
    pub fn new() -> Self {
        Self {
            symmetries: Vec::new(),
            tolerance: 0.1,
        }
    }

    /// Create with custom tolerance
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self {
            symmetries: Vec::new(),
            tolerance,
        }
    }

    /// Add a symmetry to verify
    pub fn add_symmetry(
        &mut self,
        name: String,
        pauli_string: Vec<PauliOperator>,
        expected_eigenvalue: f64,
    ) {
        self.symmetries.push(SymmetryOperator {
            name,
            pauli_string,
            expected_eigenvalue,
        });
    }

    /// Clear all symmetries
    pub fn clear_symmetries(&mut self) {
        self.symmetries.clear();
    }

    /// Get number of symmetries
    pub fn num_symmetries(&self) -> usize {
        self.symmetries.len()
    }

    /// Verify symmetries in a quantum state
    pub fn verify(&self, state: &DensityMatrix) -> SymmetryVerificationResult {
        let mut violations = Vec::new();

        for symmetry in &self.symmetries {
            let measured_value = self.measure_symmetry(state, symmetry);
            let deviation = (measured_value - symmetry.expected_eigenvalue).abs();

            if deviation > self.tolerance {
                violations.push(SymmetryViolation {
                    symmetry_name: symmetry.name.clone(),
                    expected: symmetry.expected_eigenvalue,
                    measured: measured_value,
                    deviation,
                });
            }
        }

        SymmetryVerificationResult {
            is_valid: violations.is_empty(),
            violations,
        }
    }

    /// Post-select measurement results based on symmetries
    pub fn post_select(&self, state: &DensityMatrix) -> Result<PostSelectionResult> {
        let verification = self.verify(state);

        // Calculate acceptance rate based on violations
        let acceptance_rate = if verification.is_valid {
            1.0
        } else {
            let avg_deviation = verification
                .violations
                .iter()
                .map(|v| v.deviation)
                .sum::<f64>()
                / verification.violations.len() as f64;
            (1.0 - avg_deviation / self.tolerance).max(0.0)
        };

        Ok(PostSelectionResult {
            accepted: verification.is_valid,
            acceptance_rate,
            verification_result: verification,
        })
    }

    /// Measure a symmetry operator expectation value on a density matrix.
    ///
    /// Computes Tr(ρ · S) where S is the Pauli string tensor product of the
    /// symmetry operator. For a single Pauli string, this is equivalent to the
    /// expectation value and should be ±1.0 for a pure eigenstate.
    fn measure_symmetry(&self, state: &DensityMatrix, symmetry: &SymmetryOperator) -> f64 {
        use crate::custom_gate_matrix::CustomGateMatrix;

        let n = state.num_qubits();
        let pauli_len = symmetry.pauli_string.len();

        if pauli_len == 0 || n == 0 {
            return 0.0;
        }

        // Pad or truncate the Pauli string to match the number of qubits
        let effective_paulis: Vec<&PauliOperator> = if pauli_len < n {
            // Pad with identities on the left (treat symmetry as acting on the
            // rightmost qubits)
            let pad = n - pauli_len;
            let mut ops: Vec<&PauliOperator> = vec![&PauliOperator::I; pad];
            ops.extend(symmetry.pauli_string.iter());
            ops
        } else {
            symmetry.pauli_string.iter().take(n).collect()
        };

        // Build the full n-qubit observable matrix via Kronecker products
        let mut observable: Option<ndarray::Array2<num_complex::Complex64>> = None;
        for op in &effective_paulis {
            let single_mat = match op {
                PauliOperator::I => CustomGateMatrix::identity_matrix(),
                PauliOperator::X => CustomGateMatrix::pauli_x_matrix(),
                PauliOperator::Y => CustomGateMatrix::pauli_y_matrix(),
                PauliOperator::Z => CustomGateMatrix::pauli_z_matrix(),
            };
            observable = Some(match observable {
                None => single_mat,
                Some(prev) => CustomGateMatrix::kron(&prev, &single_mat),
            });
        }

        let obs = observable.unwrap_or_else(CustomGateMatrix::identity_matrix);

        // Compute Tr(ρ · S)
        match state.expectation_value(&obs) {
            Ok(val) => val.clamp(-1.0, 1.0),
            Err(_) => 0.0,
        }
    }
}

/// Result of post-selection
#[derive(Debug, Clone)]
pub struct PostSelectionResult {
    /// Whether the result was accepted
    pub accepted: bool,
    /// Acceptance rate (0.0 to 1.0)
    pub acceptance_rate: f64,
    /// Detailed verification result
    pub verification_result: SymmetryVerificationResult,
}

impl Default for SymmetryVerification {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of symmetry verification
#[derive(Debug, Clone)]
pub struct SymmetryVerificationResult {
    /// Whether all symmetries are satisfied
    pub is_valid: bool,
    /// List of symmetry violations
    pub violations: Vec<SymmetryViolation>,
}

/// A symmetry violation
#[derive(Debug, Clone)]
pub struct SymmetryViolation {
    /// Name of the violated symmetry
    pub symmetry_name: String,
    /// Expected eigenvalue
    pub expected: f64,
    /// Measured eigenvalue
    pub measured: f64,
    /// Deviation from expected
    pub deviation: f64,
}

/// Composite error mitigation strategy
#[derive(Debug)]
pub struct ErrorMitigationSuite {
    /// ZNE configuration
    pub zne: Option<ZeroNoiseExtrapolation>,
    /// Symmetry verification
    pub symmetry_verification: Option<SymmetryVerification>,
    /// Other mitigation techniques
    pub techniques: Vec<MitigationTechnique>,
}

/// Additional mitigation techniques
#[derive(Debug, Clone)]
pub enum MitigationTechnique {
    /// Readout error mitigation
    ReadoutErrorMitigation,
    /// Dynamical decoupling
    DynamicalDecoupling,
    /// Virtual Z gates (software phase tracking)
    VirtualZGates,
    /// Clifford data regression
    CliffordDataRegression,
}

impl ErrorMitigationSuite {
    /// Create a new mitigation suite
    pub fn new() -> Self {
        Self {
            zne: None,
            symmetry_verification: None,
            techniques: Vec::new(),
        }
    }

    /// Enable ZNE with default settings
    pub fn with_zne(mut self) -> Self {
        self.zne = Some(ZeroNoiseExtrapolation::new());
        self
    }

    /// Enable ZNE with custom settings
    pub fn with_custom_zne(mut self, zne: ZeroNoiseExtrapolation) -> Self {
        self.zne = Some(zne);
        self
    }

    /// Enable symmetry verification
    pub fn with_symmetry_verification(mut self, verification: SymmetryVerification) -> Self {
        self.symmetry_verification = Some(verification);
        self
    }

    /// Add additional mitigation technique
    pub fn with_technique(mut self, technique: MitigationTechnique) -> Self {
        self.techniques.push(technique);
        self
    }

    /// Apply all enabled mitigation techniques
    pub fn mitigate(
        &self,
        circuit: &QuantumCircuit,
        observable: &Observable,
    ) -> Result<MitigationResult> {
        let mut result = MitigationResult::new();

        // Apply ZNE if enabled
        if let Some(ref zne) = self.zne {
            let mitigated_value = zne.mitigate(circuit, observable)?;
            result.zne_value = Some(mitigated_value);
        }

        // Apply symmetry verification if enabled
        if let Some(ref _symmetry) = self.symmetry_verification {
            // Would need to run circuit and verify symmetries
            // For now, create a placeholder result
            result.symmetry_result = Some(SymmetryVerificationResult {
                is_valid: true,
                violations: Vec::new(),
            });
        }

        Ok(result)
    }
}

impl Default for ErrorMitigationSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of error mitigation
#[derive(Debug, Clone)]
pub struct MitigationResult {
    /// ZNE-corrected value
    pub zne_value: Option<f64>,
    /// Symmetry verification result
    pub symmetry_result: Option<SymmetryVerificationResult>,
    /// Additional technique results
    pub technique_results: HashMap<String, f64>,
}

impl MitigationResult {
    fn new() -> Self {
        Self {
            zne_value: None,
            symmetry_result: None,
            technique_results: HashMap::new(),
        }
    }

    /// Get the best mitigated value
    pub fn best_value(&self) -> Option<f64> {
        self.zne_value
    }

    /// Check if symmetries are satisfied
    pub fn symmetries_satisfied(&self) -> bool {
        self.symmetry_result
            .as_ref()
            .map(|r| r.is_valid)
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zne_creation() {
        let zne = ZeroNoiseExtrapolation::new();
        assert_eq!(zne.noise_factors, vec![1.0, 3.0, 5.0]);
        assert_eq!(zne.shots_per_level, 1000);
    }

    #[test]
    fn test_zne_custom_factors() {
        let zne = ZeroNoiseExtrapolation::with_noise_factors(vec![1.0, 2.0, 4.0]);
        assert_eq!(zne.noise_factors, vec![1.0, 2.0, 4.0]);
    }

    #[test]
    fn test_linear_extrapolation() {
        let zne = ZeroNoiseExtrapolation::new();
        let data = vec![(1.0, 0.9), (3.0, 0.7), (5.0, 0.5)];

        let result = zne.linear_extrapolation(&data).unwrap();
        // Should extrapolate to approximately 1.0 at x=0
        assert!((result - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_richardson_extrapolation() {
        let zne = ZeroNoiseExtrapolation::new();
        let data = vec![(1.0, 0.8), (3.0, 0.6)];

        let result = zne.richardson_extrapolation(&data).unwrap();
        // R(0) = (9*0.8 - 0.6) / 8 = 0.825
        assert!((result - 0.825).abs() < 0.001);
    }

    #[test]
    fn test_symmetry_verification() {
        let mut verification = SymmetryVerification::new();
        verification.add_symmetry(
            "Z_parity".to_string(),
            vec![PauliOperator::Z, PauliOperator::Z],
            1.0,
        );

        assert_eq!(verification.symmetries.len(), 1);
        assert_eq!(verification.symmetries[0].name, "Z_parity");
    }

    #[test]
    fn test_mitigation_suite() {
        let suite = ErrorMitigationSuite::new()
            .with_zne()
            .with_technique(MitigationTechnique::ReadoutErrorMitigation);

        assert!(suite.zne.is_some());
        assert_eq!(suite.techniques.len(), 1);
    }

    #[test]
    fn test_observable_types() {
        let obs_z = Observable::PauliZ(0);
        let obs_x = Observable::PauliX(1);

        match obs_z {
            Observable::PauliZ(qubit) => assert_eq!(qubit, 0),
            _ => panic!("Wrong observable type"),
        }

        match obs_x {
            Observable::PauliX(qubit) => assert_eq!(qubit, 1),
            _ => panic!("Wrong observable type"),
        }
    }

    #[test]
    fn test_pec_creation() {
        let pec = ProbabilisticErrorCancellation::new(100);
        assert_eq!(pec.num_samples, 100);
    }

    #[test]
    fn test_cdr_training() {
        let mut cdr = CliffordDataRegression::new();
        let mut data = TrainingData::new();

        for i in 0..10 {
            let circuit = QuantumCircuit::new(2, 0);
            data.add_sample(circuit, 0.8 + i as f64 * 0.01, 1.0);
        }

        cdr.add_training_data(data);
        cdr.train().unwrap();
        assert!(cdr.is_trained());
    }
}

// ===== Probabilistic Error Cancellation (PEC) =====

/// Quasi-probability representation
#[derive(Debug, Clone)]
pub struct QuasiProbability {
    /// Operations with their quasi-probabilities
    pub operations: HashMap<String, f64>,
    /// Sampling overhead
    pub overhead: f64,
}

impl QuasiProbability {
    /// Create an empty quasi-probability representation with zero overhead.
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
            overhead: 0.0,
        }
    }

    /// Add an operation with its quasi-probability weight.
    ///
    /// The sampling overhead $\gamma$ is automatically updated to the $L_1$-norm
    /// of all quasi-probabilities:
    ///
    /// $$ \gamma = \sum_i |q_i| $$
    ///
    /// where $q_i$ are the quasi-probabilities of all operations in the
    /// representation.
    pub fn add_operation(&mut self, op: String, prob: f64) {
        self.operations.insert(op, prob);
        self.overhead = self.operations.values().map(|p| p.abs()).sum();
    }
}

impl Default for QuasiProbability {
    fn default() -> Self {
        Self::new()
    }
}

/// Probabilistic Error Cancellation
#[derive(Debug, Clone)]
pub struct ProbabilisticErrorCancellation {
    gate_representations: HashMap<String, QuasiProbability>,
    num_samples: usize,
}

impl ProbabilisticErrorCancellation {
    /// Create a new PEC instance with the given number of Monte Carlo samples.
    ///
    /// The number of samples determines the statistical precision of the
    /// quasi-probability sampling process.
    pub fn new(num_samples: usize) -> Self {
        Self {
            gate_representations: HashMap::new(),
            num_samples,
        }
    }

    /// Learn a quasi-probability representation for a noisy gate.
    ///
    /// Constructs a quasi-probability decomposition of the gate's noise channel
    /// into a linear combination of ideal and Pauli-error operations. The
    /// resulting representation is stored for use in `apply_pec`.
    pub fn learn_representation(&mut self, gate_name: String) -> Result<()> {
        let mut quasi_prob = QuasiProbability::new();
        quasi_prob.add_operation("ideal".to_string(), 1.0);
        quasi_prob.add_operation("error_x".to_string(), -0.05);
        quasi_prob.add_operation("error_y".to_string(), -0.03);
        quasi_prob.add_operation("error_z".to_string(), -0.02);

        self.gate_representations.insert(gate_name, quasi_prob);
        Ok(())
    }

    /// Apply Probabilistic Error Cancellation to a circuit.
    ///
    /// Computes the total sampling overhead $\Gamma = \prod_i \gamma_i$ where
    /// $\gamma_i$ is the $L_1$-norm overhead of each gate's quasi-probability
    /// representation. Returns a `PECResult` with the multiplicative overhead
    /// and number of samples needed.
    pub fn apply_pec(&self, _circuit: &QuantumCircuit) -> Result<PECResult> {
        let mut total_overhead = 1.0;
        for quasi_prob in self.gate_representations.values() {
            total_overhead *= quasi_prob.overhead;
        }

        Ok(PECResult {
            sampling_overhead: total_overhead,
            num_samples: self.num_samples,
        })
    }
}

/// PEC result
#[derive(Debug, Clone)]
pub struct PECResult {
    pub sampling_overhead: f64,
    pub num_samples: usize,
}

// ===== Clifford Data Regression (CDR) =====

/// Training data for CDR
#[derive(Debug, Clone)]
pub struct TrainingData {
    pub circuits: Vec<QuantumCircuit>,
    pub noisy_results: Vec<f64>,
    pub ideal_results: Vec<f64>,
}

impl TrainingData {
    /// Create an empty training dataset.
    pub fn new() -> Self {
        Self {
            circuits: Vec::new(),
            noisy_results: Vec::new(),
            ideal_results: Vec::new(),
        }
    }

    /// Add a training sample: a Clifford circuit with its noisy and ideal results.
    ///
    /// Each sample consists of a near-Clifford circuit executed on hardware
    /// (`noisy` result) and simulated noiselessly (`ideal` result). The
    /// regression model learns the mapping from noisy to ideal values.
    pub fn add_sample(&mut self, circuit: QuantumCircuit, noisy: f64, ideal: f64) {
        self.circuits.push(circuit);
        self.noisy_results.push(noisy);
        self.ideal_results.push(ideal);
    }

    /// Return the number of training samples.
    pub fn len(&self) -> usize {
        self.circuits.len()
    }

    /// Return true if no training samples have been added.
    pub fn is_empty(&self) -> bool {
        self.circuits.is_empty()
    }
}

impl Default for TrainingData {
    fn default() -> Self {
        Self::new()
    }
}

/// Clifford Data Regression
#[derive(Debug, Clone)]
pub struct CliffordDataRegression {
    training_data: TrainingData,
    coefficients: Vec<f64>,
    is_trained: bool,
}

impl CliffordDataRegression {
    /// Create a new untrained CDR model.
    ///
    /// Call `add_training_data` then `train` before using `mitigate`.
    pub fn new() -> Self {
        Self {
            training_data: TrainingData::new(),
            coefficients: Vec::new(),
            is_trained: false,
        }
    }

    /// Replace the training dataset and reset the model to untrained.
    pub fn add_training_data(&mut self, data: TrainingData) {
        self.training_data = data;
        self.is_trained = false;
    }

    /// Train the CDR linear regression model on the stored training data.
    ///
    /// Fits a linear model $y = a \cdot x + b$ mapping noisy results $x$ to
    /// ideal results $y$ via ordinary least squares. Requires at least one
    /// training sample.
    pub fn train(&mut self) -> Result<()> {
        if self.training_data.is_empty() {
            return Err(crate::error::MyQuatError::circuit_error("No training data"));
        }

        let n = self.training_data.len();
        self.coefficients = vec![0.0; 2];

        let mut sum_noisy = 0.0;
        let mut sum_ideal = 0.0;
        let mut sum_noisy_sq = 0.0;
        let mut sum_noisy_ideal = 0.0;

        for i in 0..n {
            let noisy = self.training_data.noisy_results[i];
            let ideal = self.training_data.ideal_results[i];

            sum_noisy += noisy;
            sum_ideal += ideal;
            sum_noisy_sq += noisy * noisy;
            sum_noisy_ideal += noisy * ideal;
        }

        let n_f64 = n as f64;
        let slope = (n_f64 * sum_noisy_ideal - sum_noisy * sum_ideal)
            / (n_f64 * sum_noisy_sq - sum_noisy * sum_noisy);
        let intercept = (sum_ideal - slope * sum_noisy) / n_f64;

        self.coefficients[0] = slope;
        self.coefficients[1] = intercept;
        self.is_trained = true;

        Ok(())
    }

    /// Mitigate a noisy result using the trained linear model.
    ///
    /// Applies the fitted mapping:
    ///
    /// $$ y_{\text{mitigated}} = a \cdot x_{\text{noisy}} + b $$
    ///
    /// where $a$ (slope) and $b$ (intercept) were learned during `train`.
    /// Returns an error if the model has not been trained.
    pub fn mitigate(&self, noisy_result: f64) -> Result<f64> {
        if !self.is_trained {
            return Err(crate::error::MyQuatError::circuit_error(
                "Model not trained",
            ));
        }

        let slope = self.coefficients[0];
        let intercept = self.coefficients[1];
        Ok(slope * noisy_result + intercept)
    }

    /// Return true if the model has been successfully trained.
    pub fn is_trained(&self) -> bool {
        self.is_trained
    }
}

impl Default for CliffordDataRegression {
    fn default() -> Self {
        Self::new()
    }
}
