//! Quantum Fourier Transform implementation
//!
//! This module provides QFT and inverse QFT implementations
//! for quantum circuit construction.
//!
//! The QFT maps a computational basis state $|j\rangle$ to:
//!
//! $$ |j\rangle \xrightarrow{\text{QFT}} \frac{1}{\sqrt{2^n}} \sum_{k=0}^{2^n-1} e^{2\pi i jk / 2^n} |k\rangle $$
//!
//! The gate decomposition uses Hadamard gates and controlled phase rotations
//! $R_k = \operatorname{diag}(1, e^{2\pi i / 2^k})$, followed by SWAP gates to reverse
//! the qubit order. The circuit depth is $O(n)$ and gate count is $O(n^2)$ for
//! the standard construction.

use crate::error::Result;
use crate::{Parameter, QuantumCircuit};
use std::f64::consts::PI;

/// Quantum Fourier Transform implementation
pub struct QFT {
    num_qubits: usize,
}

impl QFT {
    /// Create a new QFT instance
    pub fn new(num_qubits: usize) -> Self {
        QFT { num_qubits }
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Build the QFT circuit on $n$ qubits.
    ///
    /// The circuit applies $H$ on qubit $i$, then controlled phase rotations
    /// $R_k = \operatorname{diag}(1, e^{2\pi i / 2^k})$ from qubit $j$ to $i$
    /// for $j > i$, followed by SWAP gates to reverse the output bit order.
    pub fn build_circuit(&self) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);

        // Apply QFT algorithm
        for i in 0..self.num_qubits {
            // Hadamard gate
            circuit.h(i)?;

            // Controlled phase rotations
            for j in (i + 1)..self.num_qubits {
                let angle = PI / (1 << (j - i)) as f64;
                circuit.cp(j, i, Parameter::Float(angle))?;
            }
        }

        // Swap qubits to reverse order
        for i in 0..(self.num_qubits / 2) {
            circuit.swap(i, self.num_qubits - 1 - i)?;
        }

        Ok(circuit)
    }

    /// Build the inverse QFT ($\text{QFT}^\dagger$) circuit.
    ///
    /// Reverses the QFT by first swapping qubits, then applying inverse
    /// controlled phase rotations $R_k^\dagger$ and $H$ gates in reverse order.
    /// Satisfies $\text{QFT}^\dagger \cdot \text{QFT} = I$.
    pub fn build_inverse_circuit(&self) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);

        // Reverse the QFT operations
        // First, swap qubits
        for i in 0..(self.num_qubits / 2) {
            circuit.swap(i, self.num_qubits - 1 - i)?;
        }

        // Then reverse controlled phase rotations and Hadamard
        for i in (0..self.num_qubits).rev() {
            for j in ((i + 1)..self.num_qubits).rev() {
                let angle = -PI / (1 << (j - i)) as f64; // Negative angle for inverse
                circuit.cp(j, i, Parameter::Float(angle))?;
            }
            circuit.h(i)?;
        }

        Ok(circuit)
    }

    /// Calculate the theoretical depth of the QFT circuit
    pub fn circuit_depth(&self) -> usize {
        // Each qubit has H + controlled rotations, plus swaps
        self.num_qubits + (self.num_qubits / 2)
    }

    /// Calculate the number of gates in the QFT circuit
    pub fn gate_count(&self) -> usize {
        let mut count = 0;

        // Hadamard gates
        count += self.num_qubits;

        // Controlled phase gates
        for i in 0..self.num_qubits {
            count += self.num_qubits - i - 1;
        }

        // Swap gates
        count += self.num_qubits / 2;

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qft_creation() {
        let qft = QFT::new(3);
        assert_eq!(qft.num_qubits(), 3);
    }

    #[test]
    fn test_qft_circuit_creation() {
        let qft = QFT::new(3);
        let circuit = qft.build_circuit().unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_inverse_qft_circuit() {
        let qft = QFT::new(3);
        let inv_circuit = qft.build_inverse_circuit().unwrap();
        assert_eq!(inv_circuit.num_qubits(), 3);
        assert!(inv_circuit.size() > 0);
    }

    #[test]
    fn test_qft_gate_count() {
        let qft = QFT::new(3);
        let expected_gates = 3 + 3 + 1; // H + CP + SWAP
        assert_eq!(qft.gate_count(), expected_gates);
    }

    #[test]
    fn test_qft_circuit_depth() {
        let qft = QFT::new(4);
        let depth = qft.circuit_depth();
        assert_eq!(depth, 6); // 4 qubits + 2 swaps
    }
}
