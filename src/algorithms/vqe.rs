//! Variational Quantum Eigensolver (VQE) implementation
//!
//! This module provides VQE ansatz circuits for quantum chemistry
//! and optimization problems.
//!
//! VQE minimizes the expectation value of a Hamiltonian $H$ over a parameterized
//! trial wavefunction $|\psi(\vec{\theta})\rangle$:
//!
//! $$ E(\vec{\theta}) = \langle \psi(\vec{\theta}) | H | \psi(\vec{\theta}) \rangle $$
//!
//! The ansatz circuits implement the unitary $U(\vec{\theta})$ such that
//! $|\psi(\vec{\theta})\rangle = U(\vec{\theta})|0\rangle^{\otimes n}$.
//! Supported ansatze include hardware-efficient and UCCSD-inspired constructions.

use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit};

/// Variational Quantum Eigensolver (VQE) ansatz
pub struct VQEAnsatz {
    num_qubits: usize,
    num_layers: usize,
}

impl VQEAnsatz {
    /// Create a new VQE ansatz
    pub fn new(num_qubits: usize, num_layers: usize) -> Self {
        VQEAnsatz {
            num_qubits,
            num_layers,
        }
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the number of layers
    pub fn num_layers(&self) -> usize {
        self.num_layers
    }

    /// Build a hardware-efficient ansatz circuit.
    ///
    /// Each layer applies $R_y(\theta_k)R_z(\theta_{k+1})$ on each qubit,
    /// then a ladder of $CX$ entangling gates. The circuit has
    /// $L(3n - 1)$ parameters for $L$ layers and $n$ qubits.
    ///
    /// $$ |\psi(\vec{\theta})\rangle = \prod_{\ell=1}^{L} \left[ \bigotimes_{i=0}^{n-1} R_y^i R_z^i \cdot \prod_{i=0}^{n-2} CX_{i,i+1} \cdot \bigotimes_{i=0}^{n-2} R_z^i \right] |0\rangle^{\otimes n} $$
    pub fn build_hardware_efficient_ansatz(&self, parameters: &[f64]) -> Result<QuantumCircuit> {
        let expected_params = self.num_parameters();
        if parameters.len() != expected_params {
            return Err(MyQuatError::circuit_error(format!(
                "Expected {} parameters, got {}",
                expected_params,
                parameters.len()
            )));
        }

        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);
        let mut param_idx = 0;

        for layer in 0..self.num_layers {
            // Single-qubit rotations
            for qubit in 0..self.num_qubits {
                circuit.ry(qubit, Parameter::Float(parameters[param_idx]))?;
                param_idx += 1;
                circuit.rz(qubit, Parameter::Float(parameters[param_idx]))?;
                param_idx += 1;
            }

            // Entangling gates
            for qubit in 0..(self.num_qubits - 1) {
                circuit.cx(qubit, qubit + 1)?;
            }

            // Additional parametric gates
            if layer < self.num_layers - 1 {
                for qubit in 0..(self.num_qubits - 1) {
                    circuit.rz(qubit, Parameter::Float(parameters[param_idx]))?;
                    param_idx += 1;
                }
            }
        }

        Ok(circuit)
    }

    /// Build a UCCSD-inspired ansatz circuit using single excitations.
    ///
    /// Each excitation from orbital $i$ to $j$ is implemented as
    /// $R_y^i(\theta/2) \cdot CX_{i,j} \cdot R_y^j(-\theta/2) \cdot CX_{i,j}$.
    /// Total parameters: $\binom{n}{2} = n(n-1)/2$.
    pub fn build_uccsd_ansatz(&self, parameters: &[f64]) -> Result<QuantumCircuit> {
        if parameters.len() != self.num_uccsd_parameters() {
            return Err(MyQuatError::circuit_error(
                "Invalid parameter count for UCCSD ansatz",
            ));
        }

        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);
        let mut param_idx = 0;

        // Single excitations
        for i in 0..self.num_qubits {
            for j in (i + 1)..self.num_qubits {
                let theta = parameters[param_idx];
                param_idx += 1;

                // Simplified single excitation
                circuit.ry(i, Parameter::Float(theta / 2.0))?;
                circuit.cx(i, j)?;
                circuit.ry(j, Parameter::Float(-theta / 2.0))?;
                circuit.cx(i, j)?;
            }
        }

        Ok(circuit)
    }

    /// Calculate number of parameters needed for hardware-efficient ansatz
    pub fn num_parameters(&self) -> usize {
        self.num_layers * (self.num_qubits * 2 + self.num_qubits - 1)
    }

    /// Calculate number of parameters needed for UCCSD ansatz
    pub fn num_uccsd_parameters(&self) -> usize {
        // Single excitations: n*(n-1)/2
        self.num_qubits * (self.num_qubits - 1) / 2
    }

    /// Generate random initial parameters
    pub fn random_parameters(&self) -> Vec<f64> {
        use std::f64::consts::PI;
        (0..self.num_parameters())
            .map(|i| (i as f64 * 0.1) % (2.0 * PI))
            .collect()
    }

    /// Calculate circuit depth for hardware-efficient ansatz
    pub fn circuit_depth(&self) -> usize {
        // Each layer has rotation + entangling gates
        self.num_layers * 3 // Approximate depth per layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vqe_creation() {
        let vqe = VQEAnsatz::new(4, 2);
        assert_eq!(vqe.num_qubits(), 4);
        assert_eq!(vqe.num_layers(), 2);
    }

    #[test]
    fn test_vqe_parameter_count() {
        let vqe = VQEAnsatz::new(4, 2);
        let expected = 2 * (4 * 2 + 4 - 1); // 2 layers * (8 single-qubit + 3 entangling)
        assert_eq!(vqe.num_parameters(), expected);
    }

    #[test]
    fn test_vqe_uccsd_parameter_count() {
        let vqe = VQEAnsatz::new(4, 1);
        let expected = 4 * 3 / 2; // n*(n-1)/2
        assert_eq!(vqe.num_uccsd_parameters(), expected);
    }

    #[test]
    fn test_vqe_hardware_efficient_ansatz() {
        let vqe = VQEAnsatz::new(3, 2);
        let params = vqe.random_parameters();
        let circuit = vqe.build_hardware_efficient_ansatz(&params).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_vqe_uccsd_ansatz() {
        let vqe = VQEAnsatz::new(3, 1);
        let params: Vec<f64> = (0..vqe.num_uccsd_parameters())
            .map(|i| i as f64 * 0.1)
            .collect();
        let circuit = vqe.build_uccsd_ansatz(&params).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_vqe_invalid_parameters() {
        let vqe = VQEAnsatz::new(2, 1);
        let wrong_params = vec![0.1, 0.2]; // Not enough parameters
        let result = vqe.build_hardware_efficient_ansatz(&wrong_params);
        assert!(result.is_err());
    }
}
