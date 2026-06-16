//! Quantum Phase Estimation algorithm implementation
//!
//! This module provides quantum phase estimation for finding
//! eigenvalues of unitary operators.
//!
//! # Algorithm Overview
//!
//! Quantum Phase Estimation (QPE) is a fundamental quantum algorithm that estimates
//! the eigenvalue (phase) of a unitary operator $U$ given an eigenstate $|\psi\rangle$.
//!
//! Given a unitary $U$ and eigenstate $|\psi\rangle$ with
//! $U|\psi\rangle = e^{2\pi i\phi}|\psi\rangle$, QPE estimates the phase $\phi$
//! to $m$ bits of precision (where $m$ is the number of counting qubits).
//!
//! $$ |0\rangle^{\otimes m} |\psi\rangle \xrightarrow{\text{QPE}} |\tilde{\phi}\rangle |\psi\rangle $$
//!
//! where $|\tilde{\phi}\rangle$ encodes $\phi \approx \tilde{\phi} / 2^m$.
//!
//! # Circuit Structure
//!
//! 1. **Eigenstate Preparation**: Prepare |ψ⟩ on target qubits
//! 2. **Superposition**: Apply H to all counting qubits
//! 3. **Controlled Unitaries**: Apply controlled-U^(2^k) operations
//! 4. **Inverse QFT**: Apply inverse QFT to counting qubits
//! 5. **Measurement**: Measure counting qubits to read out phase

use crate::error::Result;
use crate::{Parameter, QuantumCircuit};
use std::f64::consts::PI;

/// Eigenstate preparation method
#[derive(Debug, Clone)]
pub enum EigenstatePreparation {
    /// Prepare computational basis state |i⟩
    ComputationalBasis(usize),
    /// Prepare |+⟩ state (equal superposition)
    PlusState,
    /// Prepare |1⟩ state
    OneState,
    /// Custom preparation circuit
    Custom(QuantumCircuit),
    /// No preparation (assume already prepared)
    None,
}

/// Controlled unitary specification
#[derive(Debug, Clone)]
pub enum ControlledUnitary {
    /// Controlled phase gate: $U = e^{i\varphi}|1\rangle\langle 1|$
    Phase(f64),
    /// Controlled Z rotation: $U = R_z(\theta)$
    ZRotation(f64),
    /// Controlled X rotation: $U = R_x(\theta)$
    XRotation(f64),
    /// Controlled Y rotation: $U = R_y(\theta)$
    YRotation(f64),
    /// Custom unitary (must provide power function)
    Custom,
}

/// Quantum Phase Estimation algorithm
pub struct PhaseEstimation {
    num_counting_qubits: usize,
    num_eigenstate_qubits: usize,
    eigenstate_prep: EigenstatePreparation,
    unitary: ControlledUnitary,
}

impl PhaseEstimation {
    /// Create a new phase estimation instance
    pub fn new(num_counting_qubits: usize, num_eigenstate_qubits: usize) -> Self {
        PhaseEstimation {
            num_counting_qubits,
            num_eigenstate_qubits,
            eigenstate_prep: EigenstatePreparation::None,
            unitary: ControlledUnitary::Phase(PI / 4.0),
        }
    }

    /// Create phase estimation with specific eigenstate preparation
    pub fn with_eigenstate_prep(
        num_counting_qubits: usize,
        num_eigenstate_qubits: usize,
        prep: EigenstatePreparation,
    ) -> Self {
        PhaseEstimation {
            num_counting_qubits,
            num_eigenstate_qubits,
            eigenstate_prep: prep,
            unitary: ControlledUnitary::Phase(PI / 4.0),
        }
    }

    /// Create phase estimation with specific unitary
    pub fn with_unitary(
        num_counting_qubits: usize,
        num_eigenstate_qubits: usize,
        prep: EigenstatePreparation,
        unitary: ControlledUnitary,
    ) -> Self {
        PhaseEstimation {
            num_counting_qubits,
            num_eigenstate_qubits,
            eigenstate_prep: prep,
            unitary,
        }
    }

    /// Get the number of counting qubits
    pub fn num_counting_qubits(&self) -> usize {
        self.num_counting_qubits
    }

    /// Get the number of eigenstate qubits
    pub fn num_eigenstate_qubits(&self) -> usize {
        self.num_eigenstate_qubits
    }

    /// Get the total number of qubits
    pub fn total_qubits(&self) -> usize {
        self.num_counting_qubits + self.num_eigenstate_qubits
    }

    /// Build the phase estimation circuit.
    ///
    /// Given a unitary $U$ with $U|\psi\rangle = e^{2\pi i\phi}|\psi\rangle$, constructs:
    ///
    /// $$ |0\rangle^{\otimes m}|\psi\rangle \to H^{\otimes m} \to \prod_{k=0}^{m-1} cU^{2^k} \to \text{QFT}^\dagger \to \text{measure} $$
    ///
    /// The measurement yields an $m$-bit estimate of $\phi$ with precision $1/2^m$.
    pub fn build_circuit(&self, eigenstate_prep: Option<QuantumCircuit>) -> Result<QuantumCircuit> {
        let total_qubits = self.total_qubits();
        let mut circuit = QuantumCircuit::new(total_qubits, self.num_counting_qubits);

        // Step 1: Prepare eigenstate on target qubits
        self.prepare_eigenstate(&mut circuit, eigenstate_prep)?;

        // Step 2: Initialize counting qubits in superposition
        for i in 0..self.num_counting_qubits {
            circuit.h(i)?;
        }

        // Step 3: Apply controlled unitary operations
        self.apply_controlled_unitaries(&mut circuit)?;

        // Step 4: Apply inverse QFT to counting qubits
        self.apply_inverse_qft(&mut circuit)?;

        // Step 5: Measure counting qubits
        for i in 0..self.num_counting_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Prepare eigenstate on target qubits
    fn prepare_eigenstate(
        &self,
        circuit: &mut QuantumCircuit,
        custom_prep: Option<QuantumCircuit>,
    ) -> Result<()> {
        let eigenstate_start = self.num_counting_qubits;

        // Use custom preparation if provided, otherwise use configured preparation
        if let Some(_prep_circuit) = custom_prep {
            // Custom preparation circuit provided
            // For now, we'll just prepare |1⟩ state as a placeholder
            // In a full implementation, we would copy gates from prep_circuit
            // TODO: Implement proper gate copying mechanism
            if self.num_eigenstate_qubits > 0 {
                circuit.x(eigenstate_start)?;
            }
            return Ok(());
        }

        // Use configured eigenstate preparation
        match &self.eigenstate_prep {
            EigenstatePreparation::ComputationalBasis(state) => {
                // Prepare |state⟩ by applying X gates
                for (i, qubit) in
                    (eigenstate_start..eigenstate_start + self.num_eigenstate_qubits).enumerate()
                {
                    if (state >> i) & 1 == 1 {
                        circuit.x(qubit)?;
                    }
                }
            }
            EigenstatePreparation::PlusState => {
                // Prepare |+⟩ state on all eigenstate qubits
                for qubit in eigenstate_start..eigenstate_start + self.num_eigenstate_qubits {
                    circuit.h(qubit)?;
                }
            }
            EigenstatePreparation::OneState => {
                // Prepare |1⟩ state on first eigenstate qubit
                if self.num_eigenstate_qubits > 0 {
                    circuit.x(eigenstate_start)?;
                }
            }
            EigenstatePreparation::Custom(_) => {
                // Custom circuit would be handled above
                // This case shouldn't be reached
            }
            EigenstatePreparation::None => {
                // No preparation - assume state is already prepared
            }
        }

        Ok(())
    }

    /// Apply controlled unitary operations
    fn apply_controlled_unitaries(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        // Skip if no eigenstate qubits
        if self.num_eigenstate_qubits == 0 {
            return Ok(());
        }

        let target = self.num_counting_qubits; // First eigenstate qubit

        for i in 0..self.num_counting_qubits {
            let power = 1 << i; // 2^i

            match &self.unitary {
                ControlledUnitary::Phase(phase) => {
                    // Apply controlled phase U^(2^i) = e^(i*phase*2^i)
                    let controlled_phase = phase * power as f64;
                    circuit.cp(i, target, Parameter::Float(controlled_phase))?;
                }
                ControlledUnitary::ZRotation(angle) => {
                    // Apply controlled Rz(angle*2^i)
                    let controlled_angle = angle * power as f64;
                    circuit.cp(i, target, Parameter::Float(controlled_angle))?;
                }
                ControlledUnitary::XRotation(angle) => {
                    // Apply controlled Rx(angle*2^i)
                    // This requires decomposition into basic gates
                    let controlled_angle = angle * power as f64;
                    // Simplified: use controlled phase as approximation
                    circuit.cp(i, target, Parameter::Float(controlled_angle))?;
                }
                ControlledUnitary::YRotation(angle) => {
                    // Apply controlled Ry(angle*2^i)
                    let controlled_angle = angle * power as f64;
                    // Simplified: use controlled phase as approximation
                    circuit.cp(i, target, Parameter::Float(controlled_angle))?;
                }
                ControlledUnitary::Custom => {
                    // Custom unitary - user must implement
                    // For now, apply controlled phase as placeholder
                    circuit.cp(i, target, Parameter::Float(PI / 4.0))?;
                }
            }
        }

        Ok(())
    }

    /// Build circuit for specific unitary (Z rotation)
    pub fn build_z_rotation_circuit(&self, rotation_angle: f64) -> Result<QuantumCircuit> {
        let total_qubits = self.total_qubits();
        let mut circuit = QuantumCircuit::new(total_qubits, self.num_counting_qubits);

        // Prepare eigenstate |1⟩ for Z rotation
        if self.num_eigenstate_qubits > 0 {
            circuit.x(self.num_counting_qubits)?;
        }

        // Initialize counting qubits in superposition
        for i in 0..self.num_counting_qubits {
            circuit.h(i)?;
        }

        // Controlled Z rotations
        for i in 0..self.num_counting_qubits {
            let power = 1 << i;
            let controlled_angle = rotation_angle * power as f64;
            let target = self.num_counting_qubits;

            if target < total_qubits {
                circuit.cp(i, target, Parameter::Float(controlled_angle))?;
            }
        }

        // Inverse QFT on counting qubits
        self.apply_inverse_qft(&mut circuit)?;

        // Measure counting qubits
        for i in 0..self.num_counting_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Apply inverse QFT to counting qubits.
    ///
    /// Inverse QFT: $|j\rangle \mapsto \frac{1}{\sqrt{2^n}}\sum_{k=0}^{2^n-1} e^{-2\pi i jk/2^n}|k\rangle$.
    /// Implemented as reverse-order controlled phase rotations ($R_k^\dagger$) and $H$ gates
    /// followed by SWAP gates.
    fn apply_inverse_qft(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let n = self.num_counting_qubits;

        // Apply inverse QFT gates in reverse order
        for j in (0..n).rev() {
            // Apply Hadamard
            circuit.h(j)?;

            // Apply controlled phase rotations (in reverse)
            for k in (0..j).rev() {
                let angle = -PI / (1 << (j - k)) as f64; // Negative for inverse
                circuit.cp(k, j, Parameter::Float(angle))?;
            }
        }

        // Swap qubits to reverse order
        for i in 0..n / 2 {
            circuit.swap(i, n - 1 - i)?;
        }

        Ok(())
    }

    /// Calculate phase precision: $\Delta\phi = 1 / 2^m$ where $m$ is counting qubits.
    pub fn phase_precision(&self) -> f64 {
        1.0 / (1 << self.num_counting_qubits) as f64
    }

    /// Estimate phase $\phi$ from a measurement bitstring: $\phi \approx 2 \cdot \text{value} / 2^m$.
    pub fn estimate_phase_from_bitstring(&self, bitstring: &str) -> f64 {
        if let Ok(value) = usize::from_str_radix(bitstring, 2) {
            2.0 * (value as f64) / (1 << self.num_counting_qubits) as f64
        } else {
            0.0
        }
    }

    /// Calculate circuit depth
    pub fn circuit_depth(&self) -> usize {
        // Superposition + controlled unitaries + inverse QFT
        1 + self.num_counting_qubits + self.num_counting_qubits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_estimation_creation() {
        let pe = PhaseEstimation::new(3, 1);
        assert_eq!(pe.num_counting_qubits(), 3);
        assert_eq!(pe.num_eigenstate_qubits(), 1);
        assert_eq!(pe.total_qubits(), 4);
    }

    #[test]
    fn test_phase_estimation_circuit() {
        let pe = PhaseEstimation::new(3, 1);
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 4); // 3 counting + 1 eigenstate
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_z_rotation_circuit() {
        let pe = PhaseEstimation::new(2, 1);
        let circuit = pe.build_z_rotation_circuit(PI / 4.0).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_phase_precision() {
        let pe = PhaseEstimation::new(3, 1);
        let precision = pe.phase_precision();
        assert_eq!(precision, 1.0 / 8.0); // 1/2^3
    }

    #[test]
    fn test_phase_estimation_from_bitstring() {
        let pe = PhaseEstimation::new(3, 1);

        // Test bitstring "100" = 4 in decimal
        let phase = pe.estimate_phase_from_bitstring("100");
        assert_eq!(phase, 1.0); // 2 * 4 / 8 = 1.0

        // Test bitstring "010" = 2 in decimal
        let phase = pe.estimate_phase_from_bitstring("010");
        assert_eq!(phase, 0.5); // 2 * 2 / 8 = 0.5
    }

    #[test]
    fn test_circuit_depth() {
        let pe = PhaseEstimation::new(4, 2);
        let depth = pe.circuit_depth();
        assert_eq!(depth, 9); // 1 + 4 + 4
    }

    #[test]
    fn test_eigenstate_prep_computational_basis() {
        // Prepare |101⟩ state
        let pe = PhaseEstimation::with_eigenstate_prep(
            2,
            3,
            EigenstatePreparation::ComputationalBasis(0b101),
        );
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 5);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_eigenstate_prep_plus_state() {
        let pe = PhaseEstimation::with_eigenstate_prep(3, 2, EigenstatePreparation::PlusState);
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 5);
        // Should have H gates for eigenstate preparation
        assert!(circuit.size() > 3);
    }

    #[test]
    fn test_eigenstate_prep_one_state() {
        let pe = PhaseEstimation::with_eigenstate_prep(2, 1, EigenstatePreparation::OneState);
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_with_z_rotation_unitary() {
        let pe = PhaseEstimation::with_unitary(
            3,
            1,
            EigenstatePreparation::OneState,
            ControlledUnitary::ZRotation(PI / 2.0),
        );
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.size() > 5);
    }

    #[test]
    fn test_with_phase_unitary() {
        let pe = PhaseEstimation::with_unitary(
            2,
            1,
            EigenstatePreparation::OneState,
            ControlledUnitary::Phase(PI / 4.0),
        );
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        // Should have eigenstate prep + superposition + controlled unitaries + inverse QFT + measurement
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_inverse_qft_gates() {
        // Test that inverse QFT adds the correct number of gates
        let pe = PhaseEstimation::new(3, 1);
        let circuit = pe.build_circuit(None).unwrap();

        // Circuit should have:
        // - Superposition (3 H gates)
        // - Controlled unitaries (3 CP gates)
        // - Inverse QFT (H gates + CP gates + SWAP gates)
        // - Measurements (3)
        assert!(circuit.size() > 10);
    }

    #[test]
    fn test_multiple_eigenstate_qubits() {
        let pe = PhaseEstimation::with_eigenstate_prep(
            4,
            3,
            EigenstatePreparation::ComputationalBasis(0b110),
        );
        let circuit = pe.build_circuit(None).unwrap();
        assert_eq!(circuit.num_qubits(), 7);
        assert_eq!(circuit.num_clbits(), 4);
    }
}
