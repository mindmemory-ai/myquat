//! Quantum Approximate Optimization Algorithm (QAOA) implementation
//!
//! This module provides QAOA circuits for combinatorial optimization
//! problems like MaxCut, Max-SAT, etc.
//!
//! QAOA alternates between the problem Hamiltonian $H_C$ (cost function) and
//! mixer Hamiltonian $H_B$ (transverse field) over $p$ layers:
//!
//! $$ |\psi(\vec{\gamma},\vec{\beta})\rangle = e^{-i\beta_p H_B} e^{-i\gamma_p H_C} \cdots e^{-i\beta_1 H_B} e^{-i\gamma_1 H_C} |+\rangle^{\otimes n} $$
//!
//! The $2p$ parameters $(\vec{\gamma}, \vec{\beta})$ are optimized classically to
//! minimize $\langle\psi|H_C|\psi\rangle$. For MaxCut, $H_C = \sum_{(i,j)\in E} Z_i Z_j$
//! and $H_B = \sum_i X_i$.

use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit};

/// Quantum Approximate Optimization Algorithm (QAOA)
pub struct QAOA {
    num_qubits: usize,
    num_layers: usize,
}

impl QAOA {
    /// Create a new QAOA instance
    pub fn new(num_qubits: usize, num_layers: usize) -> Self {
        QAOA {
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

    /// Build QAOA circuit for the MaxCut problem on a given graph.
    ///
    /// Each edge $(i, j)$ contributes $e^{-i\gamma Z_i Z_j}$ implemented as
    /// $CX_{i,j} \cdot R_z^j(\gamma) \cdot CX_{i,j}$. The mixer applies
    /// $e^{-i\beta X_i} = R_x^i(\beta)$ on each qubit.
    ///
    /// Initial state: $|+\rangle^{\otimes n} = H^{\otimes n}|0\rangle^{\otimes n}$.
    pub fn build_maxcut_circuit(
        &self,
        graph_edges: &[(usize, usize)],
        parameters: &[f64],
    ) -> Result<QuantumCircuit> {
        if parameters.len() != 2 * self.num_layers {
            return Err(MyQuatError::circuit_error(
                "QAOA requires 2p parameters for p layers",
            ));
        }

        let mut circuit = QuantumCircuit::new(self.num_qubits, self.num_qubits);

        // Initialize in equal superposition
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // QAOA layers
        for layer in 0..self.num_layers {
            let gamma = parameters[2 * layer];
            let beta = parameters[2 * layer + 1];

            // Problem Hamiltonian (cost function)
            for &(i, j) in graph_edges {
                if i < self.num_qubits && j < self.num_qubits {
                    circuit.cx(i, j)?;
                    circuit.rz(j, Parameter::Float(gamma))?;
                    circuit.cx(i, j)?;
                }
            }

            // Mixer Hamiltonian
            for i in 0..self.num_qubits {
                circuit.rx(i, Parameter::Float(beta))?;
            }
        }

        // Measure all qubits
        for i in 0..self.num_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Build QAOA circuit for Max-SAT problem
    pub fn build_maxsat_circuit(
        &self,
        clauses: &[Vec<(usize, bool)>],
        parameters: &[f64],
    ) -> Result<QuantumCircuit> {
        if parameters.len() != 2 * self.num_layers {
            return Err(MyQuatError::circuit_error(
                "QAOA requires 2p parameters for p layers",
            ));
        }

        let mut circuit = QuantumCircuit::new(self.num_qubits, self.num_qubits);

        // Initialize in equal superposition
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // QAOA layers
        for layer in 0..self.num_layers {
            let gamma = parameters[2 * layer];
            let beta = parameters[2 * layer + 1];

            // Problem Hamiltonian (each clause)
            for clause in clauses {
                self.apply_clause_operator(&mut circuit, clause, gamma)?;
            }

            // Mixer Hamiltonian
            for i in 0..self.num_qubits {
                circuit.rx(i, Parameter::Float(beta))?;
            }
        }

        // Measure all qubits
        for i in 0..self.num_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Apply clause operator for Max-SAT
    fn apply_clause_operator(
        &self,
        circuit: &mut QuantumCircuit,
        clause: &[(usize, bool)],
        gamma: f64,
    ) -> Result<()> {
        // Simplified clause operator - in practice would be more complex
        for &(var, positive) in clause {
            if var < self.num_qubits {
                if !positive {
                    circuit.x(var)?;
                }
                circuit.rz(var, Parameter::Float(gamma / clause.len() as f64))?;
                if !positive {
                    circuit.x(var)?;
                }
            }
        }
        Ok(())
    }

    /// Calculate MaxCut value for a given bitstring
    pub fn calculate_maxcut_value(&self, bitstring: &str, edges: &[(usize, usize)]) -> usize {
        let bits: Vec<char> = bitstring.chars().collect();
        let mut cut_value = 0;

        for &(i, j) in edges {
            if i < bits.len() && j < bits.len() && bits[i] != bits[j] {
                cut_value += 1;
            }
        }

        cut_value
    }

    /// Generate random initial parameters
    pub fn random_parameters(&self) -> Vec<f64> {
        use std::f64::consts::PI;
        (0..2 * self.num_layers)
            .map(|i| (i as f64 * 0.1) % PI)
            .collect()
    }

    /// Calculate circuit depth
    pub fn circuit_depth(&self) -> usize {
        // Each layer has problem + mixer Hamiltonians
        self.num_layers * 2 + 1 // +1 for initial superposition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qaoa_creation() {
        let qaoa = QAOA::new(4, 2);
        assert_eq!(qaoa.num_qubits(), 4);
        assert_eq!(qaoa.num_layers(), 2);
    }

    #[test]
    fn test_qaoa_maxcut_circuit() {
        let qaoa = QAOA::new(4, 2);
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        let params = vec![0.5, 0.3, 0.7, 0.2]; // 2 layers * 2 params each

        let circuit = qaoa.build_maxcut_circuit(&edges, &params).unwrap();
        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_qaoa_maxsat_circuit() {
        let qaoa = QAOA::new(3, 1);
        let clauses = vec![
            vec![(0, true), (1, false)], // x0 OR !x1
            vec![(1, true), (2, true)],  // x1 OR x2
        ];
        let params = vec![0.5, 0.3];

        let circuit = qaoa.build_maxsat_circuit(&clauses, &params).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_qaoa_invalid_parameters() {
        let qaoa = QAOA::new(3, 2);
        let edges = vec![(0, 1)];
        let wrong_params = vec![0.1]; // Not enough parameters

        let result = qaoa.build_maxcut_circuit(&edges, &wrong_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_maxcut_value_calculation() {
        let qaoa = QAOA::new(4, 1);
        let edges = vec![(0, 1), (1, 2), (2, 3)];

        // Test bitstring "0110" - should cut edges (0,1) and (2,3)
        let cut_value = qaoa.calculate_maxcut_value("0110", &edges);
        assert_eq!(cut_value, 2);
    }

    #[test]
    fn test_qaoa_random_parameters() {
        let qaoa = QAOA::new(3, 2);
        let params = qaoa.random_parameters();
        assert_eq!(params.len(), 4); // 2 layers * 2 params
    }
}
